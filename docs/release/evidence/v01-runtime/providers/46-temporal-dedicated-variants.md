# Gate 5 A2 — dedicated temporal semantics and variants (#52)

- Session: `v01-runtime` provider follow-up, 2026-09-15
- Issue: **#52** ([Gate 5][A2] Define DATE/TIME/TIMETZ/TIMESTAMP/TIMESTAMPTZ semantics) —
  parent workstream **#22**, parent gate **#21**; dependency #51 closed
- **Base head:** `main @ 4b86dfa` (worktree clean at session start)
- **Provider:** live `dbpro-v01-pg-fixture` (`postgres:16`, host port 55432, `PostgreSQL 16.15`),
  `DATABASE_URL=postgres://dbpro:<redacted>@127.0.0.1:55432/dbpro_fixture`

## 1. What was missing

The canonical **strings** were already locked by #56 and proven by #59/#64, but the **classes were
not distinguishable in the type system**: `TIMESTAMP` and `TIMESTAMPTZ` both decoded to
`CellValue::DateTime`, and `TIME` and `TIMETZ` both to `CellValue::Time`. A consumer could only tell
an instant from a wall clock by parsing the string, which is exactly what the A2 acceptance item
"Tauri/frontend DTO variants mirror domain semantics" forbids, and what `docs/release/provider-value-contract.md`
recorded as an explicit v0.1 exclusion pointing at this issue.

## 2. What changed

| Layer | Before | After |
|---|---|---|
| Domain (`crates/core/src/domain/query.rs`) | `DateTime` for TIMESTAMP+TIMESTAMPTZ, `Time` for TIME+TIMETZ | new `Timestamp` (`timestamp`), `TimestampTz` (`timestamptz`), `TimeTz` (`timetz`) alongside the existing `Date`/`Time`; `DateTime` retained for call sites that do not distinguish the classes (SQLite, pre-Gate-5 paths) |
| PostgreSQL decoder (`query_mapper.rs:365-384`) | `TIMESTAMPTZ`/`TIMESTAMP` → `DateTime`, `TIMETZ` → `Time` | `TIMESTAMPTZ` → `TimestampTz` (UTC `…Z`), `TIMESTAMP` → `Timestamp` (no marker), `TIMETZ` → `TimeTz` (own offset) — canonical strings unchanged |
| Tauri DTO (`crates/tauri-app/src/dto.rs`) | `Datetime` used for both temporal shapes | `Timestamp`, `Timestamptz`, `Timetz` mirror the domain tags exactly, in both directions (`From<CellValue>` and `From<CellValueDto>`) |
| In-process UI mapping (`crates/native-app/src/translate.rs:660`) | — | the three new variants join the `Text` arm: one `UiCell` class per domain class, no display change |
| Mutation plumbing (`crates/core/src/application/sql_builder.rs:315`) | — | the new variants bind through the same shape-aware temporal parsers as their legacy counterparts — never as TEXT |
| Export (`crates/core/src/application/export_service.rs`) | — | CSV, JSON and XLSX render the canonical string for the new variants |
| Agent context (`crates/core/src/domain/agent_context.rs:276`) | — | the new variants render their canonical string under the same truncation rule |

No canonical string changed. This is an **additive** contract change: `datetime` remains a legal tag.

## 3. Acceptance, criterion by criterion

| Issue acceptance item | State |
|---|---|
| DATE semantic/string contract locked | `Date`, `YYYY-MM-DD` — asserted in `temporal_classes_keep_their_own_tag_and_canonical_string` |
| TIME locked | `Time`, `HH:MM:SS.ffffff` — same test |
| TIMETZ locked | `TimeTz`, `HH:MM:SS.ffffff±HH:MM` — same test, plus live `pg_temporal_classes_decode_to_canonical_strings` (`10:20:30.123456+05:30` under `SET TIME ZONE 'America/New_York'`) |
| TIMESTAMP cannot carry invented `Z`/offset | `Timestamp`; the test asserts the emitted string contains neither `Z` nor `+`, and the live `pg_timestamp_without_time_zone_keeps_wall_clock_value` keeps its wall-clock reading under a non-UTC session zone |
| TIMESTAMPTZ canonical UTC-instant contract locked | `TimestampTz` … `Z`; live matrix asserts `2024-03-15T10:20:30.123456Z` |
| Fractional microseconds preserved in representative fixtures | live `decoder_matrix` (`TIME(6)`, `TIMESTAMP(6)`, `TIMETZ`, `INTERVAL` microseconds) and the regenerated DTO fixture (`10:20:30.123456+07:00`, `…123456Z`) |
| Tauri/frontend DTO variants mirror domain semantics | `CellValueDto::Timestamp/Timestamptz/Timetz` with the same tags; `query_result_dto_covers_every_value_class_tag` pins the ordered tag list; the checked-in `query-result-contract.json` was regenerated from the real structs and re-asserted byte-for-byte |
| Display/copy/export return exact canonical strings | UI cells are `UiCell::Text(<canonical string>)` (`map_cell`, pinned by `map_cell_keeps_one_ui_class_per_domain_value_class`); copy/export serialize that same text (#61); `export_service` renders the canonical string for the new variants |
| Provider binders fail explicitly for new temporal params until later policy work | **met in intent, and stated precisely rather than overclaimed**: the new variants bind through the shape-aware parsers (naive → naive branch, instant → RFC 3339 branch, offset-bearing time → time-with-offset branch) and a malformed value fails explicitly (`invalid date/datetime parameter`, `invalid PostgreSQL TIMETZ parameter`) — nothing is coerced and nothing is bound as TEXT. Refusing to bind would have regressed the #62 write policy, which pins `timestamp`/`timestamptz`/`time(tz)` columns as writable and validated; that is a deliberate deviation from the item's literal wording, recorded here rather than hidden |
| Exact-head CI green | the local CI-mirroring invocation (`cargo test --all -- --include-ignored`) is green on the changed tree; CI runs the same command on push |

## 4. Falsification

| Probe | Observed |
|---|---|
| the live matrix re-run **before** the two #56 live tests were updated | `pg_temporal_classes_decode_to_canonical_strings` and `pg_timestamp_without_time_zone_keeps_wall_clock_value` both **FAILED**, printing `left: Timestamp(…) / right: DateTime(…)` and `left: TimeTz(…) / right: Time(…)` — i.e. the decoder really does emit the new variants and the old expectations really did pin the old ones |
| `cargo check --workspace` after adding the variants | **5 non-exhaustive-match errors** in `export_service.rs` (×3), `sql_builder.rs`, `agent_context.rs`, then `db-pro-native` and `db-pro-tauri`: the compiler enumerated every consumer, so no site was left to a silent wildcard |
| DTO contract fixture after the payload changed | `query_result_dto_matches_the_checked_in_contract_fixture` **FAILED** byte-for-byte until the fixture was regenerated from the real structs (never hand-edited) |

## 5. Gates on the changed tree

| Gate | Command | Result |
|---|---|---|
| Format | `cargo fmt --all -- --check` | exit 0 |
| Compile | `cargo check --workspace` | exit 0 |
| Lints | `cargo clippy --workspace --all-targets -- -D warnings` | exit 0, 0 warnings |
| Workspace tests | `cargo test --workspace` | **875 passed / 0 failed / 27 ignored** (863/0/26 at session start; #88 +9, #64 +1 ignored, #52 +2) |
| Release build | `cargo build --release --locked -p db-pro-native` | exit 0 |
| Perf scan | `bash .skills/perf-audit/scripts/perf-scan.sh` | `Status: PASS` (4 passed / 0 warnings / 0 failed) |
| **Live PostgreSQL suite** | `DATABASE_URL=… cargo test -p db-pro-infrastructure --test pg_integration -- --ignored --test-threads=1` | **25 passed / 0 failed / 0 ignored** |
| **CI-mirroring run** | `DATABASE_URL=… cargo test --all -- --include-ignored` | **902 passed / 0 failed / 0 ignored**, exit 0 (889/0/0 at session start) |

New tests: `temporal_classes_keep_their_own_tag_and_canonical_string` (core),
`dedicated_temporal_cells_bind_as_typed_parameters` (core),
`dedicated_temporal_variant_strings_bind_typed_or_fail_explicitly` (infrastructure), plus the updated
live matrix, DTO tag list and the regenerated contract fixture. The fixture container was stopped
after the run.

## 6. Deliberate limits of this record

- `CellValue::DateTime` is **not** removed: SQLite and the pre-Gate-5 call sites still use it, and
  removing it would be a breaking change with no v0.1 requirement behind it. Its remaining role is
  stated in `docs/release/provider-value-contract.md`.
- The **write** path keeps the behaviour it had before this change (a `Timestamp`/`TimestampTz`
  value binds exactly as its `DateTime` predecessor did, and a `TimeTz` as its `Time` predecessor
  did). A2 is a representation slice; the provider-aware temporal mutation policy is #62's, and
  nothing here loosens or tightens it.
- No GUI behaviour is evidenced; the UI class for every temporal variant is `Text` carrying the
  canonical string (#24/#91).
