# Provider-value contract (Gate 5 A1–A4)

**Issue:** #53 (A3, the complex/fallback classes) — this document is the single in-tree contract the
A1–A4 slices were locked against; the sibling issues #51 (A1 numeric), #52 (A2 temporal), #54 (A4
serialization) record their own slice inside it.

**Baseline:** written on `main @ 9424ff8`. Every row cites the code that implements it and the evidence
file that proves it; where a class is *not* supported in v0.1 the row says so instead of leaving it
implicit.

## 1. The rules this contract enforces

Taken verbatim from the parent gate #21's non-negotiable list:

1. **Precision-sensitive values never become unsafe JavaScript numbers.**
   `Int64` and `Decimal` cross every boundary as exact strings
   (`crates/core/src/domain/query.rs:5-20` `string_i64`, `int64_serializes_as_string_for_lossless_ipc`).
2. **Timestamp-without-time-zone never gains invented timezone semantics.**
   `format_naive_timestamp` (`crates/infrastructure/src/postgres/query_mapper.rs:547`) emits
   `%Y-%m-%dT%H:%M:%S%.6f` with no marker; `format_utc_instant` (`:538`) emits the `Z` form for an
   instant. Proven with the session timezone forced to a non-UTC zone (#56,
   `providers/30-pg-temporal-decoder-classes.md`).
3. **A safe readable fallback preserves the result when one value or class is unsupported.**
   An unsupported class degrades to a byte-exact cell, never to mojibake, and never takes the other
   columns of the row down with it (#58, `providers/39-pg-unsupported-class-fallback.md`).
4. **Frontend editability is type/capability aware.**
   `ColumnWritePolicy` (`crates/ui/src/policy.rs:33`) is the one write gate; every edit entry point
   consults it (#62, `providers/32-type-aware-editability-policy.md`).
5. **Representation is decided here, not in the decoder or the UI.** §2 is the decision table; a class
   that is not in it has no v0.1 contract and must not be guessed at by a consumer.
6. **Nothing is locked on a claim.** Each row names the test that would fail if the row stopped being
   true on an exact head.

## 2. The class contract (read side)

`CellValue` (`crates/core/src/domain/query.rs:53-84`) is the single domain representation; the decoder
is `decode_cell` (`crates/infrastructure/src/postgres/query_mapper.rs:329` for PostgreSQL,
`crates/infrastructure/src/sqlite/query_mapper.rs:33-37` for SQLite, which maps the five SQLite storage
classes).

| Provider class | Domain value | Canonical text / shape | UI class (`UiCell`, `crates/ui/src/runtime.rs:545`) | Editability (`crates/ui/src/policy.rs:33`) | Proof |
|---|---|---|---|---|---|
| `NULL` (any class) | `Null` | — | `Null` | writable (`Set to NULL`) | `providers/38`, `providers/40` (NULL row) |
| `BOOL` | `Bool` | `true`/`false` | `Boolean` | writable | `providers/40` |
| `INT2`/`INT4`/`INT8`/`OID` | `Int64` | exact decimal digits, no `f64` hop | `Number(<exact text>)` | writable | #55 (`providers/07`), `providers/40` |
| `NUMERIC`/`DECIMAL` | `Decimal` | PG display scale preserved, never a float | `Number(<exact text>)` | writable | `binary_numeric_uses_postgres_display_scale`, `providers/40` (`12345678901234567890.1234`) |
| `FLOAT4`/`FLOAT8` | `Float64` | shortest round-trip form | `Number` | writable | `providers/40` |
| `TEXT`/`VARCHAR`/`CHAR`/`NAME`/`CITEXT` | `Text` | the provider text | `Text` | writable | `providers/38` |
| `UUID` | `Uuid` | canonical lowercase text | `Text` | writable (validated) | #57 (`providers/38`) |
| `DATE` | `Date` | `YYYY-MM-DD` | `Text` | writable | #56 (`providers/30`, `providers/40`) |
| `TIME` | `Time` | `HH:MM:SS.ffffff` | `Text` | writable | #56, `providers/40` |
| `TIMETZ` | `Time` | `HH:MM:SS.ffffff±HH:MM` (offset kept) | `Text` | writable | #56, `providers/40` |
| `TIMESTAMP` | `DateTime` | `YYYY-MM-DDTHH:MM:SS.ffffff` — **no marker** | `Text` | writable | #56, `providers/40` |
| `TIMESTAMPTZ` | `DateTime` | `YYYY-MM-DDTHH:MM:SS.ffffffZ` | `Text` | writable | #56, `providers/40` |
| `INTERVAL` | `Interval` | `N mons N days HH:MM:SS.ffffff` (microseconds) | `Text` | writable | `format_interval` (`:558`), `providers/40` |
| `JSON`/`JSONB` | `Json` | structured `serde_json::Value` | `Json(<compact text>)` | writable | #57, `providers/40` |
| `BYTEA`/BLOB | `Bytes` | byte-exact; display as `\x` hex | `Bytes("\\x…")` | **read-only** (`ColumnWriteBlock::Binary`) | #57, #62 |
| `INET`/`CIDR` | `Inet` | `address/prefix` (full prefix kept) | `Text` | writable | `decode_inet` (`:487`), `providers/40` |
| `ENUM` (any) | `Text` | the canonical label | `Text` | writable (validated) | #58, `providers/40` |
| `DOMAIN` (over a base type) | the **base type's** class | as the base type | as the base type | writable | #58, `providers/40` (`positive_quantity`→`Int64`, `postal_code`→`Text`) |
| generated column | any of the above | as the base type | read-only (`ColumnWriteBlock::Generated`) | #62 |

### Fallback contract for classes with no explicit decoder arm

This is the A3 decision (#53) and it is **format-driven, not heuristic**:

| Situation | Representation | Rationale |
|---|---|---|
| Provider sent **text** | `CellValue::Text(canonical text)` | text format *is* the provider's canonical rendering |
| Provider sent **binary**, and the column type says the payload is text: sqlx `PgTypeKind::Enum`, or the type name is a character type (`TEXT`, `VARCHAR`, `CHARACTER VARYING`, `BPCHAR`, `CHAR`, `CHARACTER`, `NAME`, `XML`, `CITEXT`, incl. behind a domain) | `CellValue::Text(utf8 payload)` | the binary payload of a character type or enum label is exactly its UTF-8 text |
| Provider sent **binary**, anything else (`INT4RANGE`, `TEXT[]`, `MONEY`, composite/`RECORD`, `tsvector`, `POINT`, …) | `CellValue::Bytes(raw bytes)` — **byte-exact**, displayed read-only as `\x` hex | no invented text: previously these were served as mojibake (`int4range`, `text[]`, `tsvector`) or failed the whole query (`money`, composite), see #58 |

Implementation: `decode_textual_value` (`:415`), `binary_payload_is_text` (`:441`),
`is_text_like_type_name` (`:458`). The class list is pinned by the unit test
`text_like_type_names_cover_character_types_only` (no server needed) and by the live matrix in
`providers/40`.

### Explicitly **not** in the v0.1 contract

| Class / behaviour | v0.1 state | Where it is tracked |
|---|---|---|
| Array **elements** (`{a,b}` rendering, dimensions) | not parsed; the v0.1 representation is the byte-exact fallback above (display-only) | post-v0.1; the issue's own text allows "fallback text follows contract" (#58) |
| Dedicated `timestamp` / `timetz` / `timestamptz` domain **variants** | TIMESTAMP and TIMESTAMPTZ share `CellValue::DateTime`; TIME and TIMETZ share `CellValue::Time` — the *canonical strings* distinguish them, the variants do not | #52 |
| Ranges, composites, geometric types, `tsvector` as structured values | byte-exact fallback only | post-v0.1 |
| Temporal **mutation parameters** | binders accept the canonical strings and fail explicitly on anything they cannot parse (`bind_params_fails_on_invalid_uuid` `crates/infrastructure/src/postgres/query_mapper.rs:703`, `bind_params_fails_on_invalid_datetime` `:710`) — no silent coercion | #52/#62 boundary |

## 3. The serialization contract (A4)

| Boundary | Shape | Proof |
|---|---|---|
| Domain → in-process UI (the **shipping** channel) | `UiQueryResult` (`crates/ui/src/runtime.rs:555`) via `map_query_result` (`crates/native-app/src/translate.rs:847`) and `map_cell` (`:660`); one `UiCell` class per domain class | `query_result_translation_keeps_every_field_and_value_class`, `map_cell_keeps_one_ui_class_per_domain_value_class` |
| Domain → JSON (the legacy Tauri DTO) | `QueryResultDto` (`crates/tauri-app/src/dto.rs:257`) with `CellValueDto` (`:320`) as an adjacently tagged `{type, value}` enum; `Int64`/`Decimal` as JSON strings | `query_result_dto_serializes_the_locked_camel_case_shape`, `…_covers_every_value_class_tag`, `…_never_exposes_precision_sensitive_values_as_numbers`, `…_matches_the_checked_in_contract_fixture` |
| Checked-in whole-result fixture | `crates/tauri-app/tests/fixtures/query-result-contract.json` — generated from the real structs, re-asserted byte-for-byte, every cell round-trips through `CellValueDto` | the fixture test above (#54, `providers/37`) |
| Introspection payload | `IntrospectResultDto` + `crates/tauri-app/tests/fixtures/introspect-contract.json` | #72, `providers/33` |

There is **no TypeScript mirror in v0.1**: the React frontend was retired on 2026-09-11
(`_archive/README.md`) and no Node/TS gate exists, so the Rust↔UI mapping tests above are the pinned
substitute for "Rust ↔ TS field mapping" (#60/#63 superseded).

## 4. What would falsify this document

- a class in §2 whose canonical text changes → the per-class tests in
  `crates/infrastructure/tests/pg_integration.rs` (matrix test, live) go red;
- a class re-tagged or collapsed in the DTO → the whole-result fixture test goes red;
- a UI class collapsing a domain class → the two native mapping tests go red;
- a fallback that serves binary bytes as text again → the live unsupported-class test goes red
  (it failed with `cannot decode PostgreSQL MONEY as text` before the fix);
- an edit path that ignores `ColumnWritePolicy` → the #62 policy/interaction tests go red.

## 5. Evidence index

| Slice | Issue | Evidence |
|---|---|---|
| A1 numeric/integer | #51, #55 | `providers/07-pg-integration-live.txt`, `crates/core/src/domain/query.rs:305` |
| A2 temporal | #52, #56 | `providers/30-pg-temporal-decoder-classes.md` |
| A3 complex/custom/fallback | #53, #57, #58 | `providers/38-pg-structured-class-decoding.md`, `providers/39-pg-unsupported-class-fallback.md` |
| B5 decoder matrix fixture | #59 | `providers/40-pg-decoder-matrix-fixture.md` |
| A4 serialization | #54 | `providers/37-whole-result-dto-contract.md` |
| C2/C3 frontend value policy | #61, #62 | `providers/31-canonical-value-copy-export.md`, `providers/32-type-aware-editability-policy.md` |
