# RC1 P2-C — in-UI qualification caveat for the SSH tunnel control (#239)

- Session: issue-queue pass 4, 2026-09-15
- Issue: **#239** ([P2][RC1][UX] SSH tunnel control ships with no in-UI qualification caveat)
  — filed by the #77 audit (`docs/release/rc1-p2-release-dispositions.md` §C)
- **Base head:** `main @ ba2f515` (worktree clean at session start)
- **Commit:** `fix(ui): disclose the unqualified SSH tunnel path (#239)`
- **Outcome:** caveat implemented, pinned by three tests, all gates green; issue closed.

## 1. The defect, re-verified on the current tree

| Site (before this commit) | State |
|---|---|
| `crates/ui/src/connection_view.rs:712-717` (the issue's `:713`) | `draw_postgres_connection_fields` rendered the checkbox `Connect via SSH Bastion Tunnel` inside the SSH card for every PostgreSQL draft, with no qualification text at all |
| `crates/ui/src/connection_view.rs:149,178` | the dialog's prefill keeps `ssh_tunnel_enabled: false`, so the control starts off — the defect is the impression the control makes, not its default |

The repository states the truth consistently and only outside the shipped UI:
`docs/release/known-limitations.md` LIM-006 ("SSH tunnel not E2E qualified"; "implementation exists
(shells out to `ssh` binary) but has not been end-to-end tested"; "SSH tunneling may not work
reliably"), the readiness row (`docs/release/0.1.0-readiness.md:110`, "not E2E qualified — runtime
suite `BLOCKED`"), `docs/release/risk-register.md` `R009` (`DEFERRED` — do not market SSH tunnels as a
qualified capability) and `docs/release/brand-differentiators-inventory.md:45`
("`IMPLEMENTED BUT NOT RELEASE-QUALIFIED` … **must be framed as unqualified**"). A user of the
packaged app sees none of them.

## 2. The change

One muted inline caption in the SSH card, in the idiom this dialog already uses for inline qualifiers
(`connection_view.rs:693-697` "Credentials encrypted with AES-256-GCM…",
`:834-838` "(Disallows INSERT, UPDATE, DELETE mutations)") — no new component, no new state, no
behaviour change:

- `SSH_QUALIFICATION_HINT` (`connection_view.rs:5-15`, doc comment then the string) — the wording, with
  the LIM-006 mapping recorded in the doc comment:
  `"Unqualified in v0.1: the tunnel has not been end-to-end tested and may not work reliably."`
- rendered in the SSH card (`connection_view.rs:732-740`) immediately **below the checkbox**
  (`:724-729` after the change), so it is visible with the control itself, before the tunnel is
  switched on. The #77 audit's note that the caveat is "about the impression once a user enables it" is
  why it also stays visible while enabled — it is rendered on both paths, and it was placed outside the
  `if ssh_tunnel_enabled` block (`:742`) so the option cannot be read as a qualified peer of the other
  connection fields.

Wording traceability — every claim in the string is one LIM-006 already makes, in the same order:

| Hint fragment | Source |
|---|---|
| "Unqualified in v0.1" | LIM-006 title "SSH tunnel not E2E qualified" + `Status: Accepted v0.1`; matches the readiness row's "not E2E qualified" |
| "has not been end-to-end tested" | LIM-006 actual behaviour: "…but has not been end-to-end tested" |
| "may not work reliably" | LIM-006 user-visible impact: "SSH tunneling may not work reliably" |

No release-note or capability-matrix document was changed: they already state this, and the point of
the issue is that the *app* did not.

## 3. Tests — three, falsified before being trusted

`crates/ui/src/connection_view.rs`, `mod tests` (the connection dialog had no tests before this):

| Test | Pins |
|---|---|
| `ssh_section_discloses_its_unqualified_v0_1_status` | the caveat is painted in the SSH section **before** the tunnel is enabled (`ssh_tunnel_enabled = false`) |
| `ssh_caveat_is_non_blocking_and_keeps_the_control_usable` | with `ssh_tunnel_enabled = true` the caveat is still painted **and** the SSH fields still render — the caveat removes no affordance |
| `ssh_caveat_wording_tracks_the_recorded_limitation` | the string keeps LIM-006's three claims instead of drifting into a new one |

The two render tests assert on the **painted frame**, not on the constant: `rendered_texts` runs
`draw_postgres_connection_fields` in a real `egui::Context` (the `DbProTheme::install_fonts` +
`ctx.run` + `CentralPanel` pattern already used by
`components/badge.rs::badge_pill_is_compact_like_codex_status_chip`) and walks
`FullOutput.shapes` for `Shape::Text` galleys. Deleting the label therefore fails the test even
though the constant remains.

Falsification record:

```
# Probe 1 — label removed from the render path (`let _ = SSH_QUALIFICATION_HINT;` in its place)
$ cargo test -p db-pro-ui --lib connection_view
test result: FAILED. 1 passed; 2 failed; 0 ignored; 0 measured; 379 filtered out
  ssh_section_discloses_its_unqualified_v0_1_status
  ssh_caveat_is_non_blocking_and_keeps_the_control_usable
  (painted texts dump ends at "Connect via SSH Bastion Tunnel" — no caveat, no SSH fields, as expected)

# Probe 2 — wording replaced with an invented claim ("fully qualified and production ready")
$ cargo test -p db-pro-ui --lib connection_view
test result: FAILED. 2 passed; 1 failed; 0 ignored; 0 measured; 379 filtered out
  ssh_caveat_wording_tracks_the_recorded_limitation

# Probes reverted; final state
$ cargo test -p db-pro-ui --lib connection_view
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 379 filtered out
```

## 4. Gates on the change (all raw, all exit 0)

| Gate | Command | Result |
|---|---|---|
| 1 | `cargo fmt --all -- --check` | exit 0, no output |
| 2 | `cargo check --workspace` | exit 0, `Finished dev profile` |
| 3 | `cargo clippy --workspace --all-targets -- -D warnings` | exit 0, no warnings |
| 4 | `cargo test --workspace` | exit 0, **886 passed / 0 failed / 27 ignored** (baseline 883/0/27, **+3** — exactly the three new tests) |
| 5 | `cargo build --release --locked -p db-pro-native` | exit 0, 23,777,216 bytes |
| 6 | `bash .skills/perf-audit/scripts/perf-scan.sh` | exit 0, `PASS (partial) — 4 executed check(s) passed; not run: Native UI benchmarks, Rust backend benchmarks, ER diagram runtime, DB query performance`; binary 22.7MB sha256 `b5dffcedcbbfedb9…` |
| CI-mirror | `DATABASE_URL=postgres://dbpro:<redacted>@127.0.0.1:55432/dbpro_fixture cargo test --all -- --include-ignored` | exit 0, **913 passed / 0 failed / 0 ignored** (baseline 910/0, **+3**); fixture `dbpro-v01-pg-fixture` started for the run and stopped after it |

No benchmark budget regressed: the release binary is unchanged in size class and no perf section
reported a warning. The caveat adds one `text` shape per frame in the connection dialog only.

## 5. Scope note — this does not preempt `HD-005`

`docs/release/0.1.0-human-decisions.md` `HD-005` ("Accept the unqualified SSH/backup status") is
`PENDING`. This caveat is compatible with all three of its options and takes no position on them:
under **(a) accept** it is the in-app half of the disclosure that option describes (and the
recommended default); under **(b) provision and qualify before release** it is removed or reworded when
the qualification lands, and it is not false meanwhile because the runtime suite is `BLOCKED` and
unrun as of this commit; under **(c) remove the SSH tunnel from the shipped feature set** it becomes
moot. It also satisfies the acceptance the issue was filed for — "v0.1 does not accidentally imply
fully qualified SSH support" — which is a project-level acceptance rather than a support-level choice.
Recorded here so the closing comment does not read as a decision the owner has not made.
