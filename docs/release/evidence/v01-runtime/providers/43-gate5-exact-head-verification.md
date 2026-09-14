# Gate 5 D3 — exact-head verification for the type matrix (#66)

- Session: `v01-runtime` provider follow-up, 2026-09-15
- Issue: **#66** ([Gate 5][D3] Run exact-head Rust/frontend/provider verification for type matrix) —
  parent verification workstream **#25**, parent gate **#21**
- **Verified head:** `main @ d24eb6a` (worktree clean; the record commit that follows is
  documentation-only and changes no code)
- **Provider:** live `dbpro-v01-pg-fixture` (`postgres:16`, host port 55432, `PostgreSQL 16.15`),
  `DATABASE_URL=postgres://dbpro:<redacted>@127.0.0.1:55432/dbpro_fixture`

## 1. Required gates, all on `d24eb6a`

| Gate | Command | Result |
|---|---|---|
| Format | `cargo fmt --all -- --check` | exit 0 |
| Compile | `cargo check --workspace` | exit 0 |
| Lints | `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| Workspace tests | `cargo test --workspace` | **863 passed / 0 failed / 26 ignored** |
| Release build | `cargo build --release --locked -p db-pro-native` | exit 0 |
| Perf scan | `bash .skills/perf-audit/scripts/perf-scan.sh` | `Status: PASS` (4 passed / 0 warnings / 0 failed) |
| **PostgreSQL integration matrix** | `DATABASE_URL=… cargo test -p db-pro-infrastructure --test pg_integration -- --ignored --test-threads=1` | **25 passed / 0 failed / 0 ignored** |
| **CI-mirroring run** (CI's own invocation, `ci.yml:127`) | `DATABASE_URL=… cargo test --all -- --include-ignored` | **889 passed / 0 failed / 0 ignored** |

The four value-class tests behind the matrix, all green on this head:

| Test | Covers |
|---|---|
| `pg_decoder_matrix_covers_every_value_class` | the 26-column matrix: int2/int4/int8, float4/8, numeric(24,4), date/time/timetz/timestamp/timestamptz, interval, uuid, json/jsonb, bytea, inet/cidr, enum, two domains, array, range, composite, NULLs |
| `pg_query_decodes_structured_and_binary_value_classes` | uuid, json/jsonb, bytea (incl. non-UTF8), cidr, `NULL::jsonb` vs `'null'::jsonb` |
| `pg_unsupported_binary_classes_stay_exact_and_keep_the_row_readable` | unsupported classes stay byte-exact and no longer fail the row |
| `pg_enum_and_domain_values_decode_to_canonical_values` | enum labels, domains over text/integer, NULL domains |

## 2. The frontend half is void, not skipped

The issue lists a frontend gate block (typecheck, lint, format, tokens, tests, production build). It is
**void on this tree and stated as such rather than summarized as green**:

- the React/Vite frontend was retired on 2026-09-11 (`_archive/README.md`) and there is no
  Node/pnpm/TypeScript quality gate left — `ci.yml` says so in the comment where the job used to be
  ("The legacy React/Vite frontend job was removed when the frontend was archived.");
- the shipping consumer is the native egui UI, whose value-channel mapping is pinned by
  `map_cell_keeps_one_ui_class_per_domain_value_class` and
  `query_result_translation_keeps_every_field_and_value_class` (`crates/native-app/src`), and whose
  DTO boundary is pinned by the whole-result contract fixture (`providers/37-whole-result-dto-contract.md`).

## 3. P0/P1 source-review findings at this SHA

None recorded for Gate 5: the Gate 5 issues **#51, #53, #54, #55, #56, #57, #58, #59** are closed with
evidence, and the decoder/contract P1 chain (#21/#22/#23/#24) has no open P0/P1 *source-review* finding.
The register's remaining P1 rows are distribution/host/GUI, not Gate 5:
`docs/release/risk-register.md:355-357` = `R-LICENSE` (public distribution), `R-WINLINUX`,
`R-GUI-SMOKE`, plus `F1` (restore-status reporting) — none of them a type-matrix code finding.
This record does not claim a *reviewer disposition*: that is CI's or the owner's to give, and no reviewer
exists in this environment; what the issue asks to be proven — the gates on one SHA — is proven above.

## 4. CI

- The head is pushed: run **34902131743** (`d24eb6a`, "CI") was queued at the time of writing; the runs
  for the immediately preceding commits are **34900199588** (`acc142c`) = success and
  **34900407192** (`46b2bb5`) = failure.
- That failure is explained and fixed, not hidden: it was the introspection race this session found and
  repaired in `0d3aa84` (#237, `providers/42-introspection-under-concurrent-ddl.md`), after which the
  CI-mirroring invocation above is green (889/0/0) with the integration binary re-run in parallel three
  times.
- Historical runs for the earlier candidate are recorded in `docs/release/risk-register.md:3-15`
  (run 34860902181, nine jobs green) — those are the *release* candidate's runs, not this head's.

## 5. What this record does not claim

- Windows/Linux runtime behaviour: measured on macOS against `postgres:16` only.
- The packaged-runtime smoke rows: none executed (no GUI session; #91/HD-007).
- A reviewer disposition (see §3).
