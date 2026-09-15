# AI egress disclosure — implemented in-app and in the release-facing registry (#242)

- Session: issue-queue pass 5, 2026-09-15
- Issue: **#242** ([P2][RC1][Trust] The AI features are the app's only egress and nothing in the
  product says so), filed from the #122 trust-boundary audit
  (`docs/release/audit-security-boundaries.md` §5, finding **T-1**)
- **Base head:** `main @ 482f014` (worktree clean at session start)
- **Outcome:** all four acceptance items are met. The data flow is now stated **in the product**
  (agent panel's API-key section and the AI prediction control), in the **registry** (LIM-019), in the
  **release notes** and in the **README**; the prediction default is **kept at `Eager`** with that
  decision recorded, and the per-connection opt-out plan is stated. Every in-app claim is pinned by a
  test that reads the **painted frame**, not the constant.

## 1. The egress paths on this tree (`file:line`, verified this session)

| Path | Where | What it sends | Trigger |
|---|---|---|---|
| Provider construction | `crates/runtime/src/agent.rs:121-136` (`from_env`), `:138-146` (`new`) | — | A key is required for any egress: `GROQ_API_KEY` (Groq, default endpoint) or `OPENAI_API_KEY` (OpenAI), or a key saved from the UI |
| No-key guard | `crates/runtime/src/worker.rs:1118-1127` | — | With no provider the runtime emits `SqlPredictionFailed { message: "AI provider is not configured" }` and `continue`s: **no request is sent** |
| Endpoints | `crates/runtime/src/agent.rs:13` `https://api.openai.com/v1/responses`, `:15` `https://api.groq.com/openai/v1/responses` | — | HTTPS enforced at `:153` (`if !endpoint.starts_with("https://")` → error) |
| HTTP client | `crates/runtime/src/agent.rs:8` (`reqwest::Client`) — the only `reqwest` use in the workspace | — | — |
| **Inline SQL prediction** | dispatch `crates/ui/src/query_view.rs` (`UiCommand::RequestSqlPrediction` with `context: ai_context`) → `crates/runtime/src/worker.rs:1092-1117` | The SQL text around the cursor plus the referenced schema metadata | `PredictionMode` defaults to **`Eager`** (`crates/ui/src/editor/prediction.rs:5-11`); scheduled 300 ms after an edit or cursor move (`crates/ui/src/query/query_document.rs:13`, `:272`) |
| **Agent panel** | `crates/runtime/src/agent.rs:395-413` (provider input array), `crates/core/src/domain/agent_context.rs:45-100` (result summary) | The conversation so far and, for agent query tool runs, an `AgentResultSummary` — **at most 20 sample rows × 50 columns, each cell truncated to 256 characters** (`crates/core/src/domain/agent.rs:10-12`) | Any agent run |
| Key reuse without re-entry | `crates/native-app/src/main.rs:203-220` (`seed_groq_api_key_from_keyring`, service `com.dbpro.app`, account `agent/groq_api_key`) | — | Startup, when `GROQ_API_KEY` is not already in the environment |

**Correction to the issue text, carried through everywhere it was quoted:** the agent result summary
is bounded by `MAX_AGENT_SAMPLE_ROWS = 20`, `MAX_AGENT_RESULT_COLUMNS = 50` and
`MAX_AGENT_CELL_CHARS = 256` (`crates/core/src/domain/agent.rs:10-12`), applied in
`agent_context.rs:84-100`. The issue and the audit's T-1 row say "20 × 12"; that is stale, and the
row in `audit-security-boundaries.md` is corrected in the same commit. The disclosure text and
LIM-019 use the code-derived numbers.

**Only-egress claim, re-verified rather than inherited:** `grep -rln "reqwest\|TcpStream\|UdpSocket"
--include="*.rs" crates/` returns exactly two files — `crates/runtime/src/agent.rs` (the AI provider)
and `crates/infrastructure/src/ssh/tunnel.rs` (the user's own SSH host). No telemetry, update check,
licence check or remote-asset path exists. That is what licenses the sentence "Your database and SSH
connections are the app's only other outbound connections" in the UI note.

## 2. Acceptance item by item

| #242 acceptance item | State | Where |
|---|---|---|
| Release notes **and** `known-limitations` state in one place what leaves the machine: which AI features, to which endpoints, what each payload contains, and that no other component has network capability | **met** | `docs/release/known-limitations.md` **LIM-019** (new entry, structured registry fields: actual behavior, endpoints, payload bounds, key-seeding, the only-egress statement, safe release-note wording, must-not-contradict list, evidence); summary-by-status updated 11 → **12** `Accepted v0.1`. `docs/release/0.1.0-release-notes.md` §Known limitations carries the user-facing version of the same facts |
| The in-app surface that enables the AI path carries a short, visible note about SQL text, schema metadata and sample rows | **met, twice** — the agent panel's API-key section **and** the editor's AI prediction control, i.e. both surfaces named by the issue | `crates/ui/src/agent_view.rs` `AI_EGRESS_DISCLOSURE` (muted caption directly under the key-handling caption it already had, same `font_caption()` + `text_muted` idiom); `crates/ui/src/query_view.rs` `AI_PREDICTION_EGRESS_NOTE` (muted caption under the `AI prediction` Off/Subtle/Eager row) |
| A decision is recorded on the prediction default | **met — `Eager` is kept** | Recorded in LIM-019 ("Status: Accepted v0.1", reason, and the `Eager` default with its trigger and its toggle), in the audit's T-1 row, and in the closing comment. Rationale: egress requires a configured key (`worker.rs:1118`), the disclosure now exists, and the toggle is one click from the control (`query_view.rs`); reversing the default is a one-line change plus the pinned test described below. **This is the part the owner can reverse without re-doing any of the disclosure work** |
| If the default stays automatic, the plan for a per-session or per-connection opt-out is stated | **met** | LIM-019 `Target issue` field: per-connection/per-session AI opt-out is **deferred to the post-v0.1 backlogs (#34 agent-native operations, #32 MCP)**; the shipped opt-out is the mode selector (`Off` cancels in-flight prediction for the document, `query_view.rs`), plus leaving the key unset |

## 3. Tests

| Test | Pins |
|---|---|
| `agent_key_section_discloses_what_the_ai_path_sends` (`agent_view.rs`) | the **painted frame** of `draw_agent_settings` contains the disclosure text, and the pre-existing keychain caption is still painted |
| `agent_egress_disclosure_names_the_provider_payload_and_limits` | the disclosure states prompts / SQL / schema context / 20 rows / 50 columns / 256 characters / only-other-connections, and contains **no privacy claim** (`private`, `anonymous`, `never leaves`, `encrypted`, `secure`) — the audit's finding was silence, so a note that promised privacy would contradict LIM-019 |
| `ai_prediction_control_discloses_its_egress` (`query_view.rs`) | the **painted frame** of the editor actions menu contains the disclosure next to the still-painted `AI prediction` control |
| `prediction_default_stays_eager_with_the_note_visible` | the recorded decision: `PredictionMode::Eager` is the default, so the note is what makes the default informed. Changing the default fails this test on purpose |

Falsification, both probes reverted (the tree carries neither):

1. **Labels removed, constants kept** — deleting the two `ui.label(...)` calls fails
   `agent_key_section_discloses_what_the_ai_path_sends` and
   `ai_prediction_control_discloses_its_egress` (2 failed / 2 passed). This is why the assertions read
   the rendered frame instead of the constant.
2. **Invented claim added** — prefixing the disclosure with "This feature is fully private." passed on
   the first version of the wording test, which only checked that the required facts were present. The
   guard was strengthened with the overclaim list above and the probe then failed
   (1 failed / 1 passed). Recorded rather than quietly fixed: the first version was a weaker test than
   it looked.

## 4. Gates (raw totals, this host)

| Gate | Command | Result | Baseline | Delta |
|---|---|---|---|---|
| Format | `cargo fmt --all -- --check` | exit **0** | — | — |
| Check | `cargo check --workspace` | exit **0** | — | — |
| Clippy | `cargo clippy --workspace --all-targets -- -D warnings` | exit **0**, no warnings | — | — |
| Tests | `cargo test --workspace` | **899 passed / 0 failed / 27 ignored** | 895 / 0 / 27 | **+4** |
| CI-mirror (fixture up) | `DATABASE_URL=postgres://dbpro:<redacted>@127.0.0.1:55432/dbpro_fixture cargo test --all -- --include-ignored` | **926 passed / 0 failed / 0 ignored** | 922 / 0 / 0 | **+4** |
| Release build | `cargo build --release --locked -p db-pro-native` | exit **0** | — | — |
| Perf scan | `bash .skills/perf-audit/scripts/perf-scan.sh` | **PASS (partial)**, exit **0** — 4 executed passed, 0 warnings, 0 failed, 4 not executed; binary 22.7 MB, sha256 `6a95d2db0f949f34…` | same shape | — |

The +4 is exactly the four new tests (2 + 2). The PostgreSQL fixture `dbpro-v01-pg-fixture` was
started for the CI-mirroring run and **stopped afterwards**; credentials are redacted here. No
`qltx-*` container was touched. No behaviour changed: two explanatory labels were added, and the
prediction default, the request scheduling, the payloads and the endpoints are untouched.

## 5. Documentation surfaces touched

| File | Change |
|---|---|
| `docs/release/known-limitations.md` | **LIM-019** entry + header note + summary-by-status 11 → 12 |
| `docs/release/0.1.0-release-notes.md` | one bullet under §Known limitations with the endpoints, the payload contents, the key-reuse behaviour and the only-egress statement |
| `README.md` | §Agent provider: key can also be entered in-app, plus the egress paragraph and the LIM-019 pointer |
| `docs/release/audit-security-boundaries.md` | T-1 disposition `Fix RC1 → FIXED by #242`, with the 20×12 correction and the recorded default decision |

The registry's own rules were followed: LIM-019 has every field the other 18 entries have, no entry
was deleted or rewritten, and the `Must not contradict` list names the four surfaces that must stay
consistent with it.
