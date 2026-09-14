# Gate 5 D1 — PostgreSQL fixture → domain decoder → Tauri DTO end-to-end (#64)

- Session: `v01-runtime` provider follow-up, 2026-09-15
- Issue: **#64** ([Gate 5][D1] Verify PostgreSQL fixture -> domain decoder -> Tauri DTO end-to-end) —
  parent verification workstream **#25**, parent gate **#21**; dependencies #54 and #59, both closed
- **Base head:** `main @ 0d02f1c` (worktree clean at session start)
- **Provider:** live `dbpro-v01-pg-fixture` (`postgres:16`, host port 55432, `PostgreSQL 16.15`),
  `DATABASE_URL=postgres://dbpro:<redacted>@127.0.0.1:55432/dbpro_fixture`

## 1. What was missing

Each half of the path was already covered, but nothing joined them:

| Half | Where it stopped |
|---|---|
| `pg_decoder_matrix_covers_every_value_class` (`crates/infrastructure/tests/pg_integration.rs:971`) | asserts decoded `CellValue`s, but never leaves the infrastructure crate |
| `query_result_dto_matches_the_checked_in_contract_fixture` (`crates/tauri-app/src/dto.rs:1631`) | asserts the emitted JSON, but starts from a hand-built `QueryResult` |

So "the fixture's row reaches the frontend contract intact" was an inference from two disjoint
tests, not a measured fact — the gap this issue names. `decoder_matrix` is the only fixture table
carrying every class in one row, and nothing selected it through the DTO boundary.

## 2. What was added

`crates/tauri-app/src/dto.rs` — one new live test,
`live_fixture_query_survives_the_provider_to_dto_path` (TAURI crate, because `QueryResultDto` is
private to `db_pro_tauri_lib::dto` and tauri-app depends on `db-pro-infrastructure`), plus two
helpers: `live_fixture_credentials()` (parses `DATABASE_URL` into a `ConnectionConfig`) and
`sorted_object_keys()`.

It runs `SELECT * FROM decoder_matrix ORDER BY id` on a real server, then asserts, on the same
in-memory result:

1. **#54 payload contract on real data** — top-level key set `columns/durationMs/rowCount/rows`,
   `rowCount` = 2, `durationMs` numeric, 26 columns each with exactly
   `dataType/name/nullable`, and the declared provider type names `TEXT[]`, `INT4RANGE`,
   `decoder_pair`, `NUMERIC`, `TIMETZ` still present in the DTO (custom/fallback metadata survives).
2. **Class-by-class matrix** — the 26 ordered emitted tags are compared against one labelled
   expectation each: int2/int4/int8/domain-over-int4 → `int64`; float4/float8 → `float64`;
   numeric(24,4) → `decimal`; date → `date`; time and timetz → `time`; timestamp and timestamptz →
   `datetime`; interval → `interval`; uuid → `uuid`; json/jsonb → `json`; bytea → `bytes`;
   inet/cidr → `inet`; enum and domain-over-text → `text`; array/range/composite → `bytes`;
   NULL → `null`. No class is skipped.
3. **Loss-sensitive values** — `big_count int8` = `"9223372036854775807"` and `amount numeric(24,4)`
   = `"12345678901234567890.1234"` as exact JSON strings; `local_stamp` =
   `"2024-03-15T10:20:30.123456"` (no invented zone) stays distinct from `instant` =
   `"2024-03-15T10:20:30.123456Z"`; `zoned_time` keeps `+07:00`; `blob bytea` = `[222,173,190,239]`;
   enum label `shipped`; domain `7`; NULL emitted as `{"type":"null"}` with no `value` key.
4. **No loss anywhere** — for all 52 cells of both rows, the domain `CellValue`'s own JSON equals the
   DTO's JSON, and deserializing the emitted cell into `CellValueDto` → `CellValue` and
   re-serializing reproduces the same JSON byte for byte.

## 3. Result of the first run, stated as measured

The test was **green on its first run — the remainder was missing coverage, not a defect**, exactly
as recorded for #57. Nothing in the provider→domain→DTO path needed changing, and no product code
was touched by this session.

```
test dto::tests::live_fixture_query_survives_the_provider_to_dto_path ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 29 filtered out; finished in 0.10s
```

## 4. Falsification — the test cannot pass by accident

| Probe | Observed |
|---|---|
| `DATABASE_URL` pointing at the server with a wrong password | `panicked … PG connect failed: AuthFailed(...)` → **0 passed; 1 failed** |
| `DATABASE_URL` unset | `panicked … DATABASE_URL must be set for the live provider-to-DTO path` → **0 passed; 1 failed** (an unrun live leg cannot read as a pass) |
| one class expectation mutated (`count int4` expected `text` instead of `int64`) on the live run | `provider -> domain -> DTO class mismatch: ["count int4: expected text, emitted int64"]` → **0 passed; 1 failed**; restored afterwards |

## 5. Gates on the changed tree

| Gate | Command | Result |
|---|---|---|
| Format | `cargo fmt --all -- --check` | exit 0 |
| Compile | `cargo check --workspace` | exit 0 |
| Lints | `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| Workspace tests | `cargo test --workspace` | **872 passed / 0 failed / 27 ignored** (863/0/26 at session start; #88 added 9 passes) |
| Release build | `cargo build --release --locked -p db-pro-native` | exit 0 |
| Perf scan | `bash .skills/perf-audit/scripts/perf-scan.sh` | `Status: PASS` (4 passed / 0 warnings / 0 failed) |
| **CI-mirroring run** | `DATABASE_URL=… cargo test --all -- --include-ignored` | **899 passed / 0 failed / 0 ignored**, exit 0 (session start 889/0/0; +9 from #88, +1 from this test) |

The `pg_integration` live suite itself is unchanged (25 cases) — this test lives at the DTO boundary,
which is where the class-by-class join was missing. The fixture container was stopped after the run.

## 6. Deliberate limits of this record

- The path is verified from the provider adapter to the serialized payload. The consumer on this
  side of the wire in v0.1 is the in-process native app (`crates/native-app/src/translate.rs`), whose
  mapping is pinned separately by the #54 tests; no frontend render is claimed here (#65, #21).
- No interactive/GUI behaviour is evidenced (#91).
- The fixture rows are the ones `fixtures/postgres/001_schema.sql` defines; a provider-value class
  the fixture does not carry is out of scope by construction, which is why the class matrix is the
  fixture's own column list rather than a free-form list.
