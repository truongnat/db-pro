# Trust-boundary audit — local-first security, credentials, execution paths (#122)

- **Audited baseline:** `main @ b49d667` (worktree clean at session start; issue-queue pass 3, 2026-09-15)
- **Issue:** #122 ([RC1][Trust] Audit local-first security boundaries, credential handling, and unsafe
  execution paths) — parent **#14**, supports #27/#28/#29/#105/#111
- **Related, not duplicated:** #126 owns the keyring/encrypted-fallback *lifecycle* matrix
  (`docs/release/audit-keyring-secret-lifecycle.md`); #129 owns destructive-SQL *confirmation* boundaries;
  #128 owns export/import/backup integrity. Findings that belong to those are cross-referenced here.
- **Method:** source and configuration inspection only. No behaviour was patched by this task (the issue
  body says so explicitly); every proven gap became a focused child issue.
- **Shipped crate set:** `crates/core`, `crates/infrastructure`, `crates/runtime`, `crates/ui`,
  `crates/native-app` — the release workflow builds `-p db-pro-native` only
  (`.github/workflows/release.yml:403`). `crates/tauri-app` is a legacy workspace member, not in the
  shipping graph (§4.3).

## 1. Network and data-flow inventory ("what can leave the machine")

**Complete list of outbound endpoints in the shipped crates** — `grep -rn "https\?://"` over the five
crates returns six matches, two of them endpoints:

| Endpoint | Declared at | Used by | Reachable in a release build |
|---|---|---|---|
| `https://api.openai.com/v1/responses` | `crates/runtime/src/agent.rs:13` | `CodexProvider` (reqwest, `bearer_auth`, `agent.rs:240-247`) | only when an API key is present |
| `https://api.groq.com/openai/v1/responses` | `agent.rs:15` | same | only when an API key is present |
| — (guard) | `agent.rs:153` | rejects non-HTTPS endpoints | — |
| `https://example.com/…`, `https://picsum.photos/…` | `crates/ui/src/table_editor_view.rs:768,774` | generated sample *cell values* in the insert form | never fetched |

Everything else is loopback or embedded: SSH tunnels bind `127.0.0.1:0` and probe their own local port
(`crates/infrastructure/src/ssh/tunnel.rs:51-67,165,182`), the four UI fonts are `include_bytes!`
(`crates/ui/src/theme.rs:3-6`, 656 KB on disk with `OFL.txt`), and there is no runtime asset download
(`from_uri|load_texture|include_image` → 0 hits). `websocket|ws://|wss://|hyper|ureq|curl|isahc|surf` over
the shipped crates → **0 hits**; the only HTTP client is `reqwest 0.12.28` with
`default-features = false, features = ["json","rustls-tls"]` (`Cargo.toml:45`, sole source user
`crates/runtime/src/agent.rs:8`), with `openssl`/`native-tls` absent from `Cargo.lock`.

**What actually crosses the wire, and when:**

| Flow | Trigger | Payload | Verified at |
|---|---|---|---|
| Inline SQL prediction | **automatic**: any editor change or cursor move, 300 ms after the change, when the prediction mode is not Off | editor text before/after the cursor (capped 4000/500 chars), dialect, active schema, referenced tables/aliases/columns, FK neighbours, CTE names | `crates/ui/src/query_view.rs:993-1053` (`UiCommand::RequestSqlPrediction` at `:1045`), `crates/ui/src/query/query_document.rs:13` (`SQL_PREDICTION_DEBOUNCE = 300 ms`), `crates/runtime/src/worker.rs:1092-1183` |
| Agent draft / tool loop | explicit: the user submits a prompt in the agent panel | connection **name**, driver, schema, selected table/columns (≤200 tables / ≤400 columns), current SQL, result summary, explain plan, last error, prompt; and, for each tool call, the typed tool result — including `AgentResultSummary` with `sample_rows` (**≤20 rows × ≤12 columns**, `crates/core/src/domain/agent_context.rs:41-52`, `crates/core/src/domain/agent.rs:10`) serialized at `crates/runtime/src/agent.rs:403-413` | `agent.rs:277-336,382-417`; unit test `context_input_contains_metadata_but_no_secret_field` (`agent.rs:759`) asserts no password field |
| Database connection | automatic on launch for the **first stored connection** (`crates/ui/src/events.rs:133-146`), otherwise a click | the user's own SQL/TLS traffic to the user's own server | `crates/ui/src/events.rs:133-146` → `RuntimeCommand::Connect` (`crates/runtime/src/worker.rs:1279`) |

**The prediction default is the finding.** `PredictionMode` is `#[derive(Default)]` with
`#[default] Eager` (`crates/ui/src/editor/prediction.rs:6-11`), and the key can arrive without the user
typing it in this session: `seed_groq_api_key_from_keyring()` injects `GROQ_API_KEY` into the process
environment at startup from keyring service `com.dbpro.app`, account `agent/groq_api_key`
(`crates/native-app/src/main.rs:198-220`). The only control is the Off/Subtle/Eager selector inside the
editor's overflow menu (`crates/ui/src/query_view.rs:823-836`).

**No disclosure exists.** `grep -rniE "leaves your machine|sent to|uploaded|third.party|privacy|network
call|outbound"` over `crates/ui/src`, `crates/runtime/src`, `README.md`,
`docs/release/known-limitations.md` and `docs/release/0.1.0-release-notes.md` → **0 hits**. The agent
panel carries a `Preview` badge (`crates/ui/src/agent_view.rs:224`) but nothing tells the user that the
AI features are the app's only egress, or what they send.

**No telemetry, no update check, no licence check, no remote assets** — verified by search over the
shipped crates: `telemetry` 0, `sentry` 0, `posthog` 0, `mixpanel` 0, `amplitude` 0, `bugsnag` 0,
`crash_report` 0, `update_check`/`check_for_update`/`version_check` 0, `auto_update` 0,
`license_key`/`phone_home` 0; the six `analytics` hits are test/schema names
(`crates/ui/src/app_tests.rs:3826,3828,3843`, `crates/ui/src/query/schema_completion.rs:1810,1832,1835`).
The startup path opens no socket: `crates/native-app/src/main.rs:27-117` initialises tracing, reads the
keyring locally, resolves the data dir, opens `meta.db`, spawns the worker (which builds a `reqwest`
client without sending anything) and enters `eframe`. The first frame then auto-connects the first stored
connection — the user's own database, not a third party.

## 2. Credential handling

| Question | Answer (evidence) |
|---|---|
| Where are secrets stored? | `SecretStore` port (`crates/core/src/ports/secret_store.rs:9-15`) → `KeyringVault` (`crates/infrastructure/src/secret/keyring_vault.rs`): OS keyring + in-memory session fallback in every build, encrypted `secrets.json` only when opted in. Wiring `crates/runtime/src/lib.rs:99-110` |
| Naming | service `com.dbpro.app` (`crates/runtime/src/lib.rs:62`), per-secret keys `connection/{id}/password`, `connection/{id}/ssh_password` (`crates/core/src/application/connection_service.rs:41-47`), AI key `agent/groq_api_key` (`crates/native-app/src/main.rs:200`) |
| Fallback activation | `file_secret_fallback_enabled_for(debug_build, opt_in)` (`crates/runtime/src/lib.rs:75-91`): debug builds, or release with `DB_PRO_ALLOW_FILE_SECRET_FALLBACK` = `1`/`true`; fail-closed otherwise (`require_fallback`, `keyring_vault.rs:93-104`) |
| Fallback crypto | AES-GCM file with a key derived from the **service name** (documented DEV-ONLY, `keyring_vault.rs:106-120`); atomic temp+rename (`fallback.rs:175-193`). Full lifecycle in #126's matrix |
| Read order | file fallback → session → OS keyring, with `NoEntry` falling through (`keyring_vault.rs:187-228`) |
| Do secrets reach `meta.db`? | **No.** `connection_repo.rs:11-18` persists `serde_json::to_string(connection)`; `ConnectionConfig` has no password field (`crates/core/domain/connection.rs:79-105`) and the SSH password is `#[serde(skip_serializing, default)]` (`:63`) and additionally nulled before save (`connection_service.rs:95-97`) |
| UI summaries | `ConnectionSummary`/`UiConnectionSummary` carry id/name/host/port/db/user/driver/ssl_mode/readonly — no password, no `secret_ref` (`crates/runtime/src/api.rs:128-140,754-771`); edit/duplicate prefill `password: String::new()` (`crates/ui/src/connection_view.rs:142,171`) |
| IPC | the shipped app is in-process (`tokio::sync::mpsc`); the plaintext password crosses that channel by design (`worker.rs:155-159,170-174`). The only true DTO/IPC layer, `crates/tauri-app/src/dto.rs`, is not in the shipping binary |
| Logs | `grep -E "(log::|tracing::|println!|eprintln!|dbg!)" | grep -iE "password|secret|api_key|token"` → 7 hits, all *about* keyring unavailability or cleanup failures, none interpolating a value (`connection_service.rs:90,102,107,259`; `keyring_vault.rs:203,223,256`). 48 log sites in the shipped crates in total; stderr only |
| Debug impls | `SshTunnelConfig` -> `has_password: bool` (`connection.rs:67-77`); `CodexProvider` -> `api_key: "[redacted]"` (`agent.rs:109-118`); `KeyringVault` -> `service_name: "[REDACTED]"` (`keyring_vault.rs:25-33`) |
| Connection URLs in errors | there is no URL constructor: the Postgres path builds typed `PgConnectOptions` (`crates/infrastructure/src/postgres/connection_string.rs:13-19`), so no error string can embed a password. Auth errors are value-free (`"password not found in secret store"`, `connection_service.rs:340,443`) |
| Clipboard / export | no clipboard path serializes a connection config (all writes copy SQL, DDL, cell/row/CSV/JSON text or object names); the only "connection string" shown is built **without** the password (`crates/ui/src/explorer_view.rs:455-466`). `ExportService` writes *query results* and is not wired into the shipped UI (`grep export_api()` over native/ui/runtime → 0 hits) |

**Dead redaction helpers (P3 finding).** `redact_sensitive`
(`crates/core/src/domain/diagnostics.rs:136-148`), `RedactedSecret` and `ConnectionSecret::redacted()`
(`crates/core/src/domain/secret.rs:48-93`) and `domain::secret::secret_key` have **zero call sites**
outside their own modules. Secret safety is achieved by *omission* (the field never exists in the
persisted/summary shapes) — which is stronger than redaction, but the unused helpers read like live
controls and should either be used or removed.

## 3. Execution boundaries

| Boundary | State |
|---|---|
| Classification | `StatementSafety { Read, Write, Ddl, Destructive }` with the batch reduced to its maximum severity (`crates/core/src/domain/safety.rs:53-128`); header comment states it is "best-effort heuristic, not a full SQL parser". `DO/CALL/EXECUTE` classify as `Destructive`; unknown keywords default to `Write` (fail-safe) |
| Enforcement | independent of the UI: `validate_against_policy` is called by `query_service.rs:145,154,199,306`, `schema_service.rs:343,362`, `table_data_service.rs:127,162,187,239`, `backup_service.rs:63`, `export_service.rs:48`; the policy derives from the persisted `readonly` flag. Multi-statement scripts are rejected on the single-statement path (`sql_policy.rs:3-14`) |
| Data-grid staging | edits/deletes/inserts are **staged** (`table_editor_view.rs:1731,1785,1031`) and only committed by an explicit Apply → one parameterized transaction (`table_data_service.rs:213-250`, readonly re-checked). No shipped UI path sends the immediate `UpdateTableRow`/`DeleteTableRow`/`InsertTableRow` commands (`grep` → only the legacy translate layer references them) |
| no-PK / read-only | `can_mutate_active_connection()` and `can_edit_table_rows()` gate six UI entry points (`table_editor_view.rs:1506,1516` + call sites at `:36,668,705,1618,1348,1759`; `table_ddl_view.rs:16`; `result_grid_view.rs:74`), and the backend re-checks both (`table_data_service.rs:127-152,162-181,187-191,239-244`, `query_service.rs:124-135`) |
| Agent path | the agent's SQL goes through the same backend policy **and** its own gate: `AgentSqlSafety::classify` + `execution_decision` (`crates/core/src/domain/agent.rs:357-402`) require explicit approval unless the mode is `Agent` **and** "auto-run read-only queries" is on (default `false`) **and** the batch classifies as Read; `ensure_execution_confirmation` (`crates/runtime/src/agent_executor.rs:368-390`), workflow confirm bound to run id + document version (`agent_workflow.rs:315-388`). The default UI mode is `Ask`, in which the agent cannot run SQL at all |
| DDL from the UI | the table editor's DDL tab is a **preview**; `DDL_APPLY_ENABLED = false` (`crates/ui/src/table_editor_view.rs:15`) and `ddl_execute_confirmation` is never set true, so no shipped UI path sends `UiCommand::ExecuteDdl`. DDL execution is the query editor, as `docs/release/known-limitations.md:87` states — see #129 for the confirmation-boundary question that follows from it |

**No direct mutation path bypassing the expected boundaries was found.** The query editor is the one
surface where a destructive statement runs with no confirmation dialog, and that is the *documented*
DDL path plus the readonly policy — not a bypass. It is recorded as a P2 confirmation-boundary finding
for #129 rather than as a P1 bypass, because nothing there skips a policy check.

## 4. Host boundary

### 4.1 External processes (four programs, argv-only, no shell)

| Program | Where | Arguments | User-controlled parts |
|---|---|---|---|
| `ssh` | `crates/infrastructure/src/ssh/tunnel.rs:51-67` | `-N -o ExitOnForwardFailure=yes -o ServerAliveInterval=30 -o ServerAliveCountMax=3 -i <key> -L <l>:<h>:<r> -p <port> <user>@<host>` | key path, host, user, port, remote host/port |
| `sshpass` | `tunnel.rs:71-88` | same argv, password via `SSHPASS` env | same |
| `pg_dump` | `crates/infrastructure/src/backup/pg_dump.rs:85-112` | `-h -p -U -d -f --format [-n|-t]…`, password via `PGPASSWORD` env | config fields + the user-typed output path |
| `psql` / `pg_restore` | `pg_dump.rs:142-163` | plain `psql -f <path> …` / `pg_restore <path> …` | user-typed input path + config |

`grep -rn "\bsh -c\b"` over the shipped crates → only two `#[cfg(test)]` helpers. No process is spawned
at startup. Residual surface: **argument** injection, since `ssh_target = "{user}@{host}"` and
`port_forward = "{l}:{h}:{r}"` are single argv elements — a `remote_host` containing a colon changes the
forward spec, and `-n`/`-t` take schema/table names as typed (P3; SSH is not release-qualified, LIM-006).
Secrets are passed through the child environment (`cmd.env`), never as argv, and a test pins the
`sshpass -e` shape (`tunnel.rs:199-218`).

### 4.2 Filesystem

Written: the resolved data dir and its `secrets/`, `meta.db`, `secrets/secrets.json` (opt-in only),
backup/restore targets (user-typed or file-dialog, `create_new(true)` for backups with delete-on-failure),
SQLite temp/sidecar files (`.db-pro-<uuid>.tmp`, `-wal`/`-shm`/`-journal` removal), and eframe's app-state
file (SQL text and query history — **no credentials**, `crates/ui/src/app.rs:445-478`).
`grep -rn "canonicalize|symlink|read_link"` → **0 hits**: no path canonicalisation or symlink handling
anywhere, and `~` is not expanded (the SSH key field's placeholder is literal text,
`crates/ui/src/connection_view.rs:768`). Existence-based validation only. P3, recorded.

### 4.3 Legacy Tauri crate

`crates/tauri-app` is a workspace member (so CI lints it: `ci.yml:118,121`) but **is not in the shipping
graph**: `crates/native-app/Cargo.toml` does not depend on `tauri`, the release build is
`-p db-pro-native` (`release.yml:403`, and `release.yml:391` says the job "does not build the legacy
crates/tauri-app"). Its `invoke_handler` nonetheless exposes real capability (`execute_ddl`,
`execute_ddl_batch`, `create_index`/`drop_index`, `create_trigger`/`drop_trigger`, `export_*`,
`drop_role`/`grant`/`revoke`, data-diff, partitions) and its config carries a localhost dev-server CSP
(`crates/tauri-app/tauri.conf.json:8,37`, `frontendDist` pointing into `_archive/frontend/dist`). No
shipped code path can reach it (no shared IPC, separate binary), so this is **`Accept RC1` with the
containment stated**: if that crate is ever shipped, it needs its own boundary review.

### 4.4 Memory safety

`grep -rn "unsafe"` over the shipped crates → two textual matches, both non-code (a `SAFETY` comment on
`std::env::set_var` at `crates/native-app/src/main.rs:215` and a test string). **No `unsafe` blocks, no
`unsafe impl`, no `unsafe fn`** in any crate, including `crates/tauri-app`. No `forbid(unsafe_code)`
attribute either — recorded as an observation, not a finding.

## 5. Findings, severity and disposition

| # | Severity | Finding | Disposition |
|---|---|---:|---|
| T-1 | **P2** | The AI features are the app's only egress and the app never says so: inline prediction is on by default (`PredictionMode::Eager`), fires automatically 300 ms after any editor change, and the API key can be seeded from the OS keyring at startup without the user entering it in that session. The agent path additionally sends up to 20 rows × 50 columns of **query result data** back to the provider (this audit's "20×12" was corrected against `crates/core/src/domain/agent.rs:10-12` while implementing #242). No UI text, README line or limitation entry disclosed any of it (`grep` in §1 → 0 hits) | **`FIXED` by #242** (the fix commit is the one this row is recorded in, `fix(ui): disclose the AI data flow the product never mentioned (#242)`): in-app disclosure in the agent panel's API-key section and at the AI prediction control, registry entry LIM-019 (`known-limitations.md`), release-notes and README statements, each pinned by a rendered-frame test. The prediction default is **kept at `Eager`** and that decision is recorded in LIM-019 — with egress requiring a configured key and the toggle one click away, changing the default is a product call the owner can still reverse, and reversing it is a one-line change (`crates/ui/src/editor/prediction.rs:5-11`) plus the `prediction_default_stays_eager_with_the_note_visible` test. Evidence: `docs/release/evidence/v01-runtime/providers/54-ai-egress-disclosure.md` |
| T-2 | **P3** | Dead security helpers: `redact_sensitive`, `RedactedSecret`, `ConnectionSecret::redacted()`, `domain::secret::secret_key` have zero call sites, so the tree contains redaction code that never runs while the real protection is omission | **`Accept RC1`**, recorded for a post-v0.1 cleanup (delete or wire up) |
| T-3 | **P3** | Argument-injection residual in the SSH/pg_dump argv construction (colon in `remote_host` changes the `-L` spec; `-n`/`-t` take user text) and no path canonicalisation/symlink handling, no `~` expansion | **`Accept RC1`** — argv-only, no shell, secrets never in argv, and SSH is unqualified (LIM-006); the path items are recorded for post-v0.1 |
| T-4 | **P3** | `crates/tauri-app` is compiled and linted by CI while exposing DDL/role/export commands and a localhost dev CSP, and is not in the shipping artifact | **`Accept RC1`** with the containment stated (separate binary, no shared IPC, not built by the release workflow) |
| T-5 | **P2** | The query editor runs a destructive statement with no confirmation dialog; only the readonly policy and the agent path's approval gate exist today | **cross-referenced to #129**, which owns destructive-SQL confirmation boundaries — not re-counted here |
| T-6 | **P2** | Keyring/fallback lifecycle details (Linux persistence semantics, stale-secret behaviour, packaged per-platform proof) | **cross-referenced to #126**; the storage *shape* facts are in §2 and do not duplicate its lifecycle matrix |

**No P0 and no P1 was found.** Nothing in this audit is a data-loss, credential-leak or policy-bypass
path: secrets are absent from persisted config, summaries, logs, errors and clipboard by construction;
mutation paths are staged and re-validated in the core; the agent cannot execute SQL without an approval
unless the user turned on auto-run for read-only SQL in the one mode that allows it.

## 6. Acceptance check against the issue's own list

| Acceptance item | State |
|---|---|
| no secret-storage/logging claim is based on docs alone | met: every row of §2 names the source that implements or omits it (store wiring, repo serialization, summary mapping, log sites, `Debug` impls, the absence of a URL constructor) |
| the local-first claim has a concrete network/data-flow inventory | met: §1 lists every endpoint, every trigger, every payload and the greps that prove the negatives (no telemetry, no update check, no remote assets, no startup socket) |
| any direct mutation path bypassing expected safety boundaries is P1 until resolved/dispositioned | met: none found; §3 records the enforcement points on both the UI and the core side, and the one unconfirmed destructive surface (query editor) is dispositioned to #129 with its reasoning |
| SSH / Agent Preview limitations are explicitly distinguished from production-qualified paths | met: the agent's approval gate is described as shipped behaviour, the `Preview` badge is cited, SSH is named unqualified (LIM-006) everywhere it appears (T-3) |
| findings feed #27 P2 or escape immediately as P0/P1 blockers | met: one P2 → child issue #242; the rest are P3 (`Accept RC1`) or cross-referenced to #126/#129 |

## 7. Not claimed

- No dynamic testing, fuzzing or packet capture: this is a source/config audit. The endpoint list is what
  the code can call, not a capture of what a running process sent.
- No claim about the AI providers' own handling of the payload beyond what is sent (that is a third-party
  data-processing question, not a source property).
- No per-platform keyring verification (that is #126/#92/#93 and needs Windows/Linux hosts).
- The `secrets.json` fallback build is described as *gated and DEV-ONLY keyed*, not as secure: its key
  derivation is service-name-based by design (`keyring_vault.rs:106-120`).
