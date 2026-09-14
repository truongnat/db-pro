# PostgreSQL complex/custom class decoding — unsupported classes (#58)

- Session: `v01-runtime` provider follow-up, 2026-09-15
- Tree: `main @ 46b2bb5` (clean at the start of the change); fix commit recorded in `LEDGER.md`
- Issue: **#58** ([Gate 5][B4] Implement enum/domain/array/INTERVAL/custom-type safe decoding) —
  parent workstream **#23**, parent gate **#21**
- Provider: live fixture container `dbpro-v01-pg-fixture` (`postgres:16`, host port 55432,
  `PostgreSQL 16.15`), `DATABASE_URL=postgres://dbpro:<redacted>@127.0.0.1:55432/dbpro_fixture`

## 1. The defect, measured before the fix

`decode_textual_value` (`crates/infrastructure/src/postgres/query_mapper.rs:412`) was the fallback for
every class without an explicit arm, and it called `raw.as_str()` **without looking at the wire
format**. sqlx requests binary result formats, and `as_str` does not check the format — it runs
`from_utf8` over whatever bytes arrived. Measured with a temporary probe over the live server:

| Query | Before the fix | Consequence |
|---|---|---|
| `SELECT '[1,10)'::int4range` | `Text("\u{2}\0\0\0\u{4}\0\0\0\u{1}\0\0\0\u{4}\0\0\0\n")` | **silent corruption** — binary bytes served as a string value |
| `SELECT ARRAY['a','b']::text[]` | `Text("\0\0\0\u{1}\0\0\0\0\0\0\0\u{19}…")` | **silent corruption** (a `TEXT[]` fixture column, `products.tags`) |
| `SELECT to_tsvector('hello world')` | `Text("\0\0\0\u{2}hello\0\0\u{1}…")` | **silent corruption** |
| `SELECT '12.34'::money` | `Err(QueryFailed("cannot decode PostgreSQL MONEY as text: incomplete utf-8 byte sequence from index 7"))` | the **whole query failed**, so every readable column in the row was lost |
| `SELECT ROW(1,'x')::record` | `Err(QueryFailed("… RECORD as text: invalid utf-8 sequence of 1 bytes from index 19"))` | same |

That is exactly what the parent gate's non-negotiables forbid ("safe readable fallback should preserve
a result when one value/class is unsupported") and what #58's acceptance names ("unsupported complex
value does not fail unrelated readable columns/rows").

## 2. The fix

`decode_textual_value` is now format- and type-aware, and the decision is made from the column's own
type, not from guessing about the bytes:

- **text format** → the provider's canonical text (unchanged);
- **binary format + the payload *is* text** → `CellValue::Text`, where "is text" means the decoded kind
  (`sqlx::postgres::PgTypeKind`) is `Enum` (an enum's type name is arbitrary; its binary payload is the
  label), or the type name is a character type (`TEXT`, `VARCHAR`, `CHARACTER VARYING`, `BPCHAR`,
  `CHAR`, `CHARACTER`, `NAME`, `XML`, `CITEXT`, and the same names behind a domain);
- **binary format otherwise** → `CellValue::Bytes`, byte-exact. The UI renders bytes as read-only
  `\x` hex (the #62 policy blocks editing them), so an unsupported class can no longer be mistaken for
  a string value, and it no longer fails the row.

**Verified type-name → representation matrix** (`SELECT … ` with one column per class, measured):

| Declared type | Binary payload is | Resulting class |
|---|---|---|
| `TEXT`, `VARCHAR`, `CHAR`, `xml` | the UTF-8 text | `Text` |
| an enum type (`order_status`) | the UTF-8 label | `Text` |
| a domain over varchar / integer (`information_schema.yes_or_no`, `cardinal_number`) | base type's payload (sqlx resolves the domain) | `Text` / `Int64` |
| `INT4RANGE` | range framing + big-endian bounds | `Bytes` |
| `TEXT[]` | array framing + element payloads | `Bytes` |
| `MONEY` | int64 cents | `Bytes` |
| `RECORD` (composite) | field OIDs + payloads | `Bytes` |
| `tsvector` | lexeme framing | `Bytes` |

## 3. Tests

| Test | Kind | What it pins |
|---|---|---|
| `text_like_type_names_cover_character_types_only` | unit (`query_mapper` tests, no server) | the name rule: character types (with length suffixes, any case) are text; `INT4RANGE`, `TEXT[]`, `MONEY`, `RECORD`, `tsvector`, `POINT`, an enum name are not |
| `pg_unsupported_binary_classes_stay_exact_and_keep_the_row_readable` | live | a row mixing `int4range` (exact bytes `[0x02,0,0,0,4,0,0,0,1,0,0,0,4,0,0,0,0x0A]`), `money` (8 bytes, big-endian cents `1234`), a composite record and a `text[]` with `'ok'::text` and `42`: the unsupported classes are `Bytes` and the readable columns of the same row survive |
| `pg_enum_and_domain_values_decode_to_canonical_values` | live | enum literal and enum column → canonical label text; a domain over text → canonical text; a domain over integer → `Int64` through its base type; a NULL domain → `Null` |

**Fails before / passes after** (same command, source file stashed and restored):

```
$ git stash push -- crates/infrastructure/src/postgres/query_mapper.rs
$ DATABASE_URL=… cargo test -p db-pro-infrastructure --test pg_integration -- --ignored --test-threads=1 \
    pg_unsupported_binary_classes pg_enum_and_domain
test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 21 filtered out
  … an unsupported class must not fail the readable columns of the row:
    QueryFailed("cannot decode PostgreSQL MONEY as text: incomplete utf-8 byte sequence from index 7")
$ git stash pop
$ … same command
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 21 filtered out
```

## 4. Verification

- Live suite: `DATABASE_URL=… cargo test -p db-pro-infrastructure --test pg_integration -- --ignored
  --test-threads=1` → **23 passed / 0 failed / 0 ignored** (21 before; the two new cases are #58's, and
  every pre-existing case — including the enum/numeric and temporal ones — stays green, so no text-like
  class regressed).
- Gate line for this change: `cargo fmt --all -- --check` exit 0, `cargo check --workspace` exit 0,
  `cargo clippy --workspace --all-targets -- -D warnings` exit 0, `cargo test --workspace`
  **863 passed / 0 failed / 24 ignored** (baseline 862/0/22 — +1 unit test, +2 `#[ignore]`d live cases),
  `cargo build --release --locked -p db-pro-native` exit 0, `bash .skills/perf-audit/scripts/perf-scan.sh`
  `Status: PASS` (4/0/0).

## 5. Scope boundary (deliberate, not an oversight)

- **No array element parsing.** An array's fallback is byte-exact (`Bytes`), not the invented
  `{a,b}` rendering; the display-only representation for arrays is recorded in the A3 contract
  (`docs/release/provider-value-contract.md`, #53) and element parsing stays post-v0.1. #58's own
  acceptance allows this ("array dimensions/elements **or fallback text follow contract exactly**").
- **No new decode arms beyond the fallback rule.** The explicit arms for numeric, temporal, JSON,
  UUID, BYTEA, network and INTERVAL were already in place (#55/#56/#57); enum labels and domains were
  already correct via the text path and are now pinned by a live test instead of assumed.
- **`Bytes` for an unsupported class is read-only in the UI** by the #62 policy, so the fallback cannot
  be edited back into the database as a string.
