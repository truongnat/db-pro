# Disposition of the two stale PostgreSQL decode PRs (#179, #180)

- Session: `v01-runtime` provider follow-up, 2026-09-15
- Tree: `main @ 00a3ce9` (`git pull --ff-only origin main` from `46acef7`; HEAD == `origin/main`,
  clean tree at the start of the analysis)
- Subject: open PR **#180** (`fix/postgres-binary-text-decoding-4130642313309925911`, head `bc08819`)
  and open PR **#179** (`fix/pg-binary-decode-fallback-6477489898429962857`, head `c35dfb9`)
- Provider: live fixture container `dbpro-v01-pg-fixture` (`postgres:16`, host port 55432,
  `PostgreSQL 16.15`), `DATABASE_URL=postgres://dbpro:<redacted>@127.0.0.1:55432/dbpro_fixture`
- Outcome: **the defect both PRs target is already fixed on `main`; nothing was still valuable, so no
  product code was ported and no `fix(...)` commit exists for this task.** The evidence file plus one
  ledger row are the whole change

## 1. Scope and method

Both PRs claim the same P1 defect: `decode_textual_value` (and, in #179, an older `decode_cell`
fallback) in `crates/infrastructure/src/postgres/query_mapper.rs` calling `raw.as_str()` on a
`PgValueFormat::Binary` payload.

The question asked was not "does the code look right" but "is the defect still reachable on `main`".
So the method was: (a) prove on a live server that the *precondition* still holds — that these columns
really do arrive in binary format on the path the app uses; (b) run the real decode path and record the
`CellValue` produced; (c) only then classify each PR hunk against the live tree.

The PRs were not merged, closed, commented on, or otherwise mutated. No branch or PR was created.
Only `main` was written to.

### 1.1 The facts as given, re-verified

| Claim | Verification | Result |
|---|---|---|
| #180 head `bc08819`, 4 commits, 181 behind `main` | `git rev-list --left-right --count origin/main...origin/fix/postgres-binary-text-decoding-4130642313309925911` | **`181 4`** — confirmed |
| #179 head `c35dfb9`, 4 commits, 335 behind `main` | same for the #179 branch | **`335 4`** — confirmed |
| Neither branch's commits appear on `main` | `git merge-base --is-ancestor <sha> origin/main` for `aa723b1 d71300c 0700807 bc08819 4548cce d2f4679 ca9ca3d c35dfb9` | **all eight return non-zero** — confirmed |
| `#180`'s `ci.yml` python-escaping fix landed separately as `7d7043c` | `git log --oneline -- .github/workflows/ci.yml` | **confirmed** (`7d7043cd fix(core): remap table mutation failure index and fix CI workflow syntax error`) |
| Neither PR's net diff applies to `main` | `git apply --check` on each | **both exit 1** (see §4) |

## 2. Reproduction: the defect is fixed on `main`

### 2.1 The precondition still holds — every column arrives binary

Two temporary probes were compiled against the workspace, run with `--nocapture`, and **deleted before
any commit** (the tree carries no probe file; they existed only to produce the output below).

`sqlx-postgres 0.8.6` asks for binary results on every prepared statement —
`src/connection/executor.rs:244-247` sends `formats: &[PgValueFormat::Binary]` and
`result_formats: &[PgValueFormat::Binary]` in the `Bind`, and `:279` returns `PgValueFormat::Binary`
with the comment `// prepared statements are binary`. `PostgresConnector::query` uses
`sqlx::query_with(sql, pg_args).fetch(&pool)` (`crates/infrastructure/src/postgres/connector.rs:151`),
which is a prepared statement. So the binary path is not an edge case here — it is *every* query.

Measured (`format()` and the legacy unconditional `raw.as_str()` call, per column):

```
=== WIRE FORMAT AND LEGACY as_str() BEHAVIOUR ===
column               pg_type.name()     format       as_str()   as_str() value / error
enum_label           order_status       Binary       OK         "shipped"
                       kind = Enum(["pending", "processing", "shipped", "delivered", "cancelled"])
domain_over_text     VARCHAR            Binary       OK         "YES"
                       kind = Simple
text_array           TEXT[]             Binary       OK         "\0\0\0\u{1}\0\0\0\0\0\0\0\u{19}\0\0\0\u{2}\0\0\0\u{1}\0\0\0\u{1}a\0\0\0\u{1}b"
                       kind = Array(PgTypeInfo(Text))
tsvector_value       tsvector           Binary       OK         "\0\0\0\u{2}cat\0\0\u{1}\0\u{3}rat\0\0\u{1}\0\u{2}"
                       kind = Simple
range_value          INT4RANGE          Binary       OK         "\u{2}\0\0\0\u{4}\0\0\0\u{1}\0\0\0\u{4}\0\0\0\n"
                       kind = Range(PgTypeInfo(Int4))
money_value          MONEY              Binary       ERR        incomplete utf-8 byte sequence from index 7
                       kind = Simple
interval_value       INTERVAL           Binary       OK         "\0\0\0\0\0\0\0\0\0\0\0\u{1}\0\0\0\0"
                       kind = Simple
plain_text           TEXT               Binary       OK         "ok"
                       kind = Simple
```

This is the defect's precondition, reproduced: **all eight columns are `Binary`**, and the old
unconditional `as_str()` yields mojibake for `text[]`, `tsvector`, `int4range` and `interval`, and a
hard error for `money`.

### 2.2 Today's `main` decodes each class correctly

Same row through the public API (`PostgresConnector::query`, the real provider path):

```
=== TODAY'S main: CellValue PER BINARY-FORMAT CLASS (via PostgresConnector::query) ===
enum_label         Text("shipped")
domain_over_text   Text("YES")
text_array         Bytes([0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 25, 0, 0, 0, 2, 0, 0, 0, 1, 0, 0, 0, 1, 97, 0, 0, 0, 1, ...
tsvector_value     Bytes([0, 0, 0, 2, 99, 97, 116, 0, 0, 1, 0, 3, 114, 97, 116, 0, 0, 1, 0, 2])
range_value        Bytes([2, 0, 0, 0, 4, 0, 0, 0, 1, 0, 0, 0, 4, 0, 0, 0, 10])
money_value        Bytes([0, 0, 0, 0, 0, 0, 4, 210])
interval_value     Interval("1 days")
plain_text         Text("ok")
answer             Int64(42)
```

Every class is either canonical text (enum label, domain over text) or byte-exact
(`money` = `0x4D2` = 1234 cents; `int4range` = flags `0x02` then big-endian 1 and 10). No mojibake,
and the row is readable — the other columns survive alongside the opaque ones.

### 2.3 `main` already carries permanent regression coverage

The reproduction above is not a one-off probe: `main` has three live tests that pin exactly this, and
they pass on the fixture **today**:

```
$ DATABASE_URL=postgres://dbpro:<redacted>@127.0.0.1:55432/dbpro_fixture \
  cargo test -p db-pro-infrastructure --test pg_integration -- --ignored --test-threads=1 \
  pg_unsupported_binary_classes_stay_exact_and_keep_the_row_readable \
  pg_enum_and_domain_values_decode_to_canonical_values \
  pg_decoder_matrix_covers_every_value_class

running 3 tests
test pg_decoder_matrix_covers_every_value_class ... ok
test pg_enum_and_domain_values_decode_to_canonical_values ... ok
test pg_unsupported_binary_classes_stay_exact_and_keep_the_row_readable ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 22 filtered out; finished in 0.25s
```

| Test (`crates/infrastructure/tests/pg_integration.rs`) | Pins |
|---|---|
| `pg_unsupported_binary_classes_stay_exact_and_keep_the_row_readable` (`:854`) | `int4range` byte-exact against `[0x02, 0,0,0,4, 0,0,0,1, 0,0,0,4, 0,0,0,10]`, `money` as an 8-byte int64 = 1234 cents, a composite record and a `text[]` as non-empty bytes, and the `text`/`int4` columns of the same row intact |
| `pg_enum_and_domain_values_decode_to_canonical_values` (`:924`) | enum label as canonical text (literal and column), domain-over-text, domain-over-int through its base type, and a NULL domain stays NULL |
| `pg_decoder_matrix_covers_every_value_class` (`:972`) | the fixture's per-class matrix (`fixtures/postgres/decoder_matrix`), plus a NULL row |

These assertions are byte-level, which is what makes them decisive rather than vacuous: the `money`
and `int4range` expectations can only be produced from a *binary* payload. A `Text`-format
implementation would have failed them. So the tests are themselves the proof that the binary path is
exercised.

The fix that landed is `crates/infrastructure/src/postgres/query_mapper.rs:417-437` — `decode_textual_value`
matches on `raw.format()` with a `Text` arm, a `Binary if binary_payload_is_text(row, i)` arm
(`:443`), and a `Binary` fallback that returns `CellValue::Bytes`. The two PRs are therefore competing
with an implementation that is already on `main` and is **strictly more correct than either of them**
(§3, §4).

### 2.4 One factual correction to both PRs' premise

PR #180's `FINDINGS.md` and `PLAN.md` state that `raw.as_str()` fails with
`ColumnDecode("expected text format")` "because sqlx's `ValueRef::as_str` implementation checks
`self.format() == PgValueFormat::Text`". **That is false for the pinned sqlx version.** In
`sqlx-postgres 0.8.6`, `src/value.rs:71-73`:

```rust
pub fn as_str(&self) -> Result<&'r str, BoxDynError> {
    Ok(from_utf8(self.as_bytes()?)?)
}
```

There is no format check. `as_str()` just reinterprets the payload as UTF-8, so it **succeeds** on any
binary payload that happens to be valid UTF-8 — which is exactly what §2.1 shows for `text[]`,
`tsvector`, `int4range` and `interval`. The real failure mode is therefore *silent mojibake*, with a
hard error only for payloads that are not valid UTF-8 (`money`). This matters for the disposition,
because it means the fix PR #180 proposes (`Binary` → `std::str::from_utf8(bytes)`) cannot work:

- for `text[]` / `tsvector` / `int4range` / `interval`, `from_utf8` succeeds, so the value is still
  served as mojibake — **#180 does not fix the class of bug it describes**;
- for `money`, `from_utf8` fails, so the query still fails — only the error message changes.

## 3. Per-file, per-hunk classification — PR #180

Branch `fix/postgres-binary-text-decoding-4130642313309925911`, head `bc08819`, 4 commits
(`aa723b1` → `d71300c` → `0700807` → `bc08819`), 181 behind `main`. Net diffstat: 7 files, +85/-5.
`git apply --check` on the net diff: **exit 1, all three code files fail to apply.**

| File | Hunk | Class | Evidence |
|---|---|---|---|
| `.github/workflows/ci.yml` | `ssh_port=` python escaping (line 77) | **ALREADY_ON_MAIN** | `main` line 77 already reads `s.bind(("127.0.0.1", 0))` with no backslash escapes; landed as `7d7043c` |
| `crates/core/src/application/table_data_service.rs` | `#[allow(clippy::result_large_err)]` on `apply_mutations_detailed` | **ALREADY_ON_MAIN** | present at `crates/core/src/application/table_data_service.rs:216`, on that exact function (`:217`) |
| `crates/infrastructure/src/postgres/query_mapper.rs` | `decode_textual_value`: `Text` arm + `Binary` arm via `std::str::from_utf8` | **SUPERSEDED_BY_NEWER_WORK** | `main :417-437` has the format match with three arms; the `Binary if binary_payload_is_text` arm (`:426`) and the byte-exact `Bytes` fallback (`:432`) are strictly stronger. #180's version cannot pass `pg_integration.rs:854`, which asserts byte-exactness, and does not fix the mojibake it describes (§2.4) |
| `crates/infrastructure/src/postgres/query_mapper.rs` | new unit test `binary_text_payload_decodes_as_utf8` | **NO_LONGER_APPLICABLE** (no value) | a tautology: it asserts `std::str::from_utf8(b"custom_enum_val") == "custom_enum_val"` and never calls `decode_textual_value`. Superseded by the live tests at `:854` / `:924` / `:972` |
| `docs/plans/active/postgres-binary-text-decoding/{PLAN,CHECKLIST,FINDINGS,VERIFICATION}.md` | whole workstream dir | **NO_LONGER_APPLICABLE** | the workstream is complete on `main` under #58/#59 with its own evidence (`docs/release/evidence/v01-runtime/providers/38-pg-structured-class-decoding.md`, `39-pg-unsupported-class-fallback.md`, `40-pg-decoder-matrix-fixture.md`). The docs also assert the false premise of §2.4 and the diff does not apply |

Nothing in #180 is `STILL_VALUABLE`. Its CI fix and its `result_large_err` allow are already on `main`
by other routes, and its one substantive change is a weaker version of what `main` already has.

## 4. Per-file, per-hunk classification — PR #179

Branch `fix/pg-binary-decode-fallback-6477489898429962857`, head `c35dfb9`, 4 commits
(`4548cce` → `d2f4679` → `ca9ca3d` → `c35dfb9`), 335 behind `main`. Net diffstat: 11 files, +136/-24.
`git apply --check` on the net diff: **exit 1, every file fails to apply** (the first is
`crates/ui/src/components.rs: No such file or directory`).

| File | Hunk | Class | Evidence |
|---|---|---|---|
| `crates/infrastructure/src/postgres/query_mapper.rs` | old `decode_cell` `.unwrap_or_else(...)` fallback: `Text` → `from_utf8`, `Binary` → literal `<unsupported binary value: {data_type}>` | **NO_LONGER_APPLICABLE / SUPERSEDED** | the targeted code shape does not exist on `main`: `decode_cell` (`:329`) returns `Result<CellValue, DbError>` and propagates errors through `map_row` (`:324`); there is no placeholder fallback. Where the two do overlap, #179 is weaker — it *discards* the value as a placeholder string, which would fail the byte-exact assertions at `pg_integration.rs:854-905` (`CellValue::Bytes == [0x02, …, 10]`, `money` as int64 cents) |
| `crates/infrastructure/src/postgres/query_mapper.rs` | new unit test `fallback_format_check_behavior` | **NO_LONGER_APPLICABLE** (no value) | a tautology: it defines a local closure `format_fallback_message` and asserts on the closure, never on the mapper |
| `crates/infrastructure/src/sqlite/actor.rs` | `#[allow(clippy::result_large_err)]` on `execute_transaction` | **ALREADY_ON_MAIN** | present at `crates/infrastructure/src/sqlite/actor.rs:262`, on that exact function (`:263`) |
| `crates/ui/src/components.rs` | `Stroke::new(1.0, …)` → `1.0_f32` in `primary_button`, `primary_button_with_icon`, `danger_button` (3 hunks) | **NO_LONGER_APPLICABLE** | the file no longer exists — it is a module now (`crates/ui/src/components/`); the three functions live at `crates/ui/src/components/legacy.rs:231`, `:241`, `:343` with the bare literals at `:235`, `:245`, `:347` |
| `crates/ui/src/diagram_view.rs` | 5 hunks, `1.0`→`1.0_f32` / `1.2`→`1.2_f32` | **NO_LONGER_APPLICABLE** | `cargo clippy --workspace --all-targets -- -D warnings` is **clean on `main`** with these literals. 107 bare `Stroke::new(1.0, …)` sites remain repo-wide (`crates/ui/src/diagram_view.rs:469`, `:480`, `:583`, `:610`, `:729`, `crates/ui/src/theme.rs:297`…, `crates/ui/src/components/*.rs`), so porting a 5-file subset would make `main` *less* consistent, not more |
| `crates/ui/src/result_grid_view.rs` | 1 hunk, `1.0`→`1.0_f32` on a divider `vline` | **NO_LONGER_APPLICABLE** | the target line is gone; the current strokes there are `:862`, `:985`, `:1942` (different expressions) |
| `crates/ui/src/theme.rs` | 18 hunks, `Stroke::new(1.0, …)` → `1.0_f32` | **NO_LONGER_APPLICABLE** | same float-literal class; `main` `:297`, `:313`, `:314`, `:320`, `:324`, `:325`, `:329`, `:330`, `:334`, `:335`, `:337` are unchanged and clippy-clean |
| `crates/ui/src/workspace_view.rs` | 1 hunk, `1.5`→`1.5_f32` in `paint_tab_indicator` | **NO_LONGER_APPLICABLE** | the `Stroke::new(1.5, theme.accent)` anchor is gone from that file entirely |
| `docs/plans/active/pg-binary-decode-fallback/{PLAN,CHECKLIST,FINDINGS,VERIFICATION}.md` | whole workstream dir | **NO_LONGER_APPLICABLE** | `PLAN.md` self-declares `IMPLEMENTING`; the `CHECKLIST.md`/`VERIFICATION.md` gate on the **retired React frontend** (`pnpm run generate:routes`, `pnpm run typecheck`, `pnpm run test`) and `frontend/` is absent on `main` (archived to `_archive/frontend`; `package.json` keeps only the Tauri scripts) |

Nothing in #179 is `STILL_VALUABLE` either. Its infra allow is already on `main`; its query-mapper
change targets a code shape that no longer exists and, where comparable, is weaker than what replaced
it; its UI changes are a repo-wide style sweep applied to 5 of 107 sites that clippy does not require;
its UI file `components.rs` no longer exists; and its planning docs gate on a frontend that has been
archived.

### 4.1 Why "float literal ambiguity" is not ported

Worth stating separately, since it is the only hunk class in either PR that is not purely beaten by an
existing implementation. The change is `1.0` → `1.0_f32` inside `Stroke::new(…)`. On the pinned
toolchain (`rust-toolchain.toml`, exact `1.95.0`) clippy does not flag these: the workspace gate
passes on `main` as-is (§6). Applying it to 5 files while 107 sites keep the bare literal would leave
the codebase inconsistent for no gate benefit, and re-deciding the convention repo-wide is a separate
style decision with its own diff — not something to smuggle in through a stale PR.

## 5. What was ported

**Nothing.** Every hunk of both PRs is classified `ALREADY_ON_MAIN`,
`SUPERSEDED_BY_NEWER_WORK`, or `NO_LONGER_APPLICABLE`; there is no `STILL_VALUABLE` row. Therefore:

- no product code changed on `main`;
- there is no `fix(...)` commit for this task;
- the only commits are this evidence file and the ledger row (§7);
- the product-code gate set is consequently not re-run as a prerequisite of a change. The suites were
  still executed to report the real numbers on `00a3ce9` rather than inherit a stale baseline (§6).

No new regression test was added because the defect is already pinned by three live tests (§2.3). A
test would have had to duplicate `pg_integration.rs:854` to say anything, and the task's own rule was to
add one only where it was missing.

One observation, **not** a PR artifact and deliberately not actioned here: the classes pinned by those
three tests are range / money / record / `text[]` / enum / domain / the decoder matrix
(`bytea`, `inet`, `interval`, `json`, `jsonb`, `uuid`). `tsvector` is the one binary-textual class the
§2.2 sweep exercises that no committed test names — its behavior on `main` is correct
(`Bytes([0,0,0,2, 99,97,116, 0,0,1, 0,3, 114,97,116, 0,0,1, 0,2])`, verified above), so this is a
coverage gap, not a defect. Recorded for the coordinator; adding it is a separate workstream decision,
not a port from these PRs.

## 6. Measured state on `main @ 00a3ce9`

Executed without any local modification (probes deleted before these runs; tree clean).

| Gate | Command | Result | Baseline | Delta |
|---|---|---|---|---|
| Workspace tests | `cargo test --workspace` | **889 passed / 0 failed / 27 ignored** | 886 / 0 / 27 | **+3** |
| CI-mirror (fixture up) | `DATABASE_URL=… cargo test --all -- --include-ignored` | **916 passed / 0 failed / 0 ignored** | 913 / 0 | **+3** |
| Clippy | `cargo clippy --workspace --all-targets -- -D warnings` | **clean**, exit 0 | — | — |

The +3 in both rows is the #245 merge that `git pull` brought in (`00a3ce9`), which added exactly three
tests to `crates/ui` — `cell_text_as_str_returns_borrowed_slices`,
`iso_temporal_heuristic_filters_non_date_strings`, `cell_contains_filter_handles_ascii_and_unicode`
(`crates/ui/src/result_grid.rs`) — confirmed by diffing `46acef7..00a3ce9`. No other suite changed.

The remaining gates (`cargo fmt --all -- --check`, `cargo check --workspace`,
`cargo build --release --locked -p db-pro-native`, `bash .skills/perf-audit/scripts/perf-scan.sh`) were
not run, per the task's rule that the gate set applies "only if you changed production code". They are
not implied by a documentation-only change, and running the release build and perf scan would produce
numbers this task does not consume. Stated plainly rather than left implicit.

## 7. Recommendation (not acted on)

| PR | Recommendation | Reason |
|---|---|---|
| **#180** | **Close as superseded** | Both non-infra hunks are already on `main` (`ci.yml` via `7d7043c`; the `result_large_err` allow at `table_data_service.rs:216`). The substantive `query_mapper.rs` change is a strictly weaker version of `main :417-437` that, per §2.4, does not fix the mojibake class it was written for, and it cannot satisfy the byte-exact assertions already enforced at `pg_integration.rs:854`. Its added unit test is a tautology. Its planning docs assert a premise that is false for the pinned sqlx and describe work already completed under #58/#59 |
| **#179** | **Close as superseded** | The `sqlite/actor.rs` allow is already on `main` (`:262`). The query-mapper hunk targets a `decode_cell` fallback shape that no longer exists, and its placeholder approach (`<unsupported binary value: …>`) discards the value where `main` keeps it byte-exact. The UI hunks are a partial (5 of 107 sites) style sweep that clippy does not require on the pinned toolchain, and one of the files it edits (`crates/ui/src/components.rs`) no longer exists. Its verification gates reference the archived React frontend |

Both PRs are, in effect, two independent attempts at a defect that the #58/#59 decoder work has since
fixed better than either proposed. Neither carries a unique regression test, a unique code path, or a
doc that records anything `main` does not already have.

Neither PR was closed, merged, commented on, or otherwise mutated by this task.
