# Type-aware editability and safe mutation policy (#62)

- Session: `v01-runtime` provider follow-up, 2026-09-15
- Tree: `main @ baa4462` (clean at the start of the change); fix commit recorded in `LEDGER.md`
- Issue: **#62** ([Gate 5][C3] Enforce type-aware editability and safe mutation policy) — parent
  workstream **#24**, parent gate **#21**; builds on #61 and #56 in this session
- Policy sources: #52 (A2 temporal semantics), #53 (A3 "unsupported mutation flows are blocked/read-only
  rather than guessed"), #51 (A1 exact numeric text)

## 1. Measured behaviour before the change

The data editor gated every edit on three things only: connection writable, table has a primary key,
and (for updates) `is_binary_type` on the column
(`crates/ui/src/table_editor_view.rs:1613-1618` before the change). Measured gaps:

| # | Scenario | Behaviour before | Verdict |
|---|---|---|---|
| E1 | Update a `GENERATED` column (e.g. `total numeric GENERATED ALWAYS AS … STORED`) | the edit staged and the UPDATE was sent to the database, which rejected it at apply time | **DEFECT** — the UI guessed instead of refusing |
| E2 | Insert a row whose dialog carries a value for a generated column | staged; PostgreSQL rejects the insert | DEFECT |
| E3 | Duplicate Row on a table with a generated column | the generated value was prefilled into the insert dialog and submitted | DEFECT |
| E4 | Binary column reached through the commit path | `submit_data_cell_edit` duplicated the binary check of `begin_data_cell_edit` with a different message | incoherent policy, duplicated check |
| E5 | Insert dialog for a blocked column | offered a text field, a NULL button and a sample generator that can never be valid | misleading UI |

"Set to NULL" on a blocked column had no guard of its own; it was reachable only through
`submit_data_cell_edit`, so the same gaps applied.

## 2. The fix

A single value-class policy replaces the scattered checks: `ColumnWritePolicy` /
`ColumnWriteBlock` in the new `crates/ui/src/policy.rs`.

**The editability table (value class → write policy):**

| Class (type name pattern) | Write policy | Reason / mechanism |
|---|---|---|
| BYTEA / BLOB / binary / varbinary | **read-only** | no editor can preserve the bytes; `reason()` says so |
| `is_generated = true` (any type) | **read-only** | computed by the database; `reason()` says so |
| `is_identity` | writable | PostgreSQL accepts explicit identity values (`OVERRIDING` is the user's choice) |
| integer / bigserial / serial / smallint | writable, validated | range-checked parse to the exact digits |
| numeric(p,s) / decimal(p,s) | writable, validated | `BigDecimal` parse + precision/scale enforcement, exact text (`parse_decimal_value`) |
| float / real / double | writable, validated | `f64` parse |
| bool | writable, validated | true/false/1/0/yes/no |
| text / varchar / char / citext | writable | untrimmed text; empty and NULL distinct (`parse_update_value`) |
| uuid | writable, validated | `Uuid::parse_str` |
| timestamp (with/without tz) | writable, validated | chrono format validation, staged as the canonical text; the #56 decoder round-trips it |
| date / time / timetz | writable, validated | chrono format validation |
| interval / inet / enum / domain / `T[]` / json | writable | typed parameter binding, validated where a validator exists (JSON); PostgreSQL checks the value |
| unknown/custom | writable | deterministic text binding; PostgreSQL rejects invalid text, which is the safe fallback per #53 |
| NULL into NOT NULL | refused at staging | `submit_data_cell_edit` + `submit_insert_row` |

**Enforcement points** (all resolve the policy from the open table's `UiTableInfo`):

| Point | Change |
|---|---|
| `DbProApp::column_write_policy` / `column_write_block` (`table_editor_view.rs`) | the lookup every other point shares |
| `begin_data_cell_edit` | refuses a blocked column with the policy reason instead of the old binary-only message |
| `submit_data_cell_edit` | the same policy instead of the duplicated binary check — nothing blocked ever reaches `stage_update` |
| `submit_insert_row` | blocked column + empty input → skipped (a generated column computes itself); blocked column + input → deterministic error, nothing staged |
| `open_duplicate_row` | leaves blocked columns empty instead of prefilling them |
| insert dialog | blocked columns render `Read-only — <reason>` with no text field / NULL / Gen buttons |
| grid context menu | "Edit Cell" becomes a muted "Read-only Column" entry with the reason on hover; "Set to NULL" disappears |
| `handle_grid_edit_input` (paste) | pasting into a blocked cell sets the reason in the copy status and never calls the editor |

The no-primary-key and readonly-connection semantics are untouched and stay independent:
`can_edit_table_rows` still requires a writable connection **and** a primary key
(`crates/ui/src/table_editor_view.rs`), pinned by the pre-existing
`no_primary_key_table_blocks_safe_row_mutations`.

## 3. Verification

`cargo test -p db-pro-ui --lib` → **372 passed / 0 failed / 0 ignored** (364 before; 4 policy unit
tests + 4 interaction tests added, the pre-existing editor suite untouched and green).

| Test | What it pins |
|---|---|
| `policy::tests::binary_and_generated_columns_are_read_only` | bytea/BLOB/binary varying → `Binary`; numeric/tsvector with `is_generated` → `Generated` |
| `policy::tests::every_class_the_editor_can_round_trip_stays_writable` | the 20-class table above (serial, numeric(p,s), float, bool, text, varchar, citext, uuid, timestamps, date, time(tz), interval, inet, `text[]`, enum, jsonb) |
| `policy::tests::identity_columns_stay_writable` | explicit identity values are allowed |
| `policy::tests::blocked_columns_explain_themselves` | both reasons are user-facing text |
| `app::tests::binary_cell_edit_is_refused_with_a_reason` | `begin_data_cell_edit` opens no editor, shows the reason, stages nothing |
| `app::tests::generated_column_edit_is_refused_before_staging` | `submit_data_cell_edit` returns false, sets `data_edit_error`, stages nothing |
| `app::tests::generated_column_is_never_staged_by_insert` | empty generated column skipped on insert; a filled one refused deterministically |
| `app::tests::duplicated_row_leaves_blocked_columns_empty` | `open_duplicate_row` prefills `["", ""]` for `(id, total)` |

Gate line for this change: `cargo fmt --all -- --check`, `cargo check --workspace`,
`cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`,
`cargo build --release --locked -p db-pro-native`, `bash .skills/perf-audit/scripts/perf-scan.sh`
(recorded numbers in `LEDGER.md`).

## 4. Not claimed here

- The #53 A3 *read* representations for array/domain/custom classes (explicit DTO variants) — this
  change enforces the write side of the policy; the read side stays with #52/#53/#54.
- A dedicated binary cell editor (the class is read-only until one exists, per the issue's own scope).
- Any change to the SQL builder or the runtime mutation path — the UI refuses before staging, so the
  builder's existing behaviour is unchanged.
