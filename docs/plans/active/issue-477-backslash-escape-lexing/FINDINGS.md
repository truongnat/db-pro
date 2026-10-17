# #477 findings

- Root cause: `safety_lexer::skip_quoted` unconditionally skipped `\x` inside
  `'…'` for every dialect. PG default (`standard_conforming_strings=on`) ends
  `'x\'` at the quote, so `SELECT 'x\'; DROP TABLE t; --` was ONE statement to
  the splitter and `reject_multi_statement` passed it.
- The same lexer feeds `classify_*`, `top_level_where_clause`,
  `contains_sql_keyword`, `matching_parenthesis_end`, `tokenize_sql` — all now
  take the escape flag so classification and splitting agree per dialect.
- `E'…'` literals opt into escapes on their own; detected by `is_escape_string_start`
  which requires the `E` not to be glued to an identifier (`someE'…'` is an
  identifier + plain string).
- Driver plumbing: `ConnectionConfig.driver` is the canonical source.
  `QueryService::driver_for` fetches it once per entry point; ExportService
  already fetched config (moved above the gate); SchemaService fetches per
  call. `backslash_escapes_for_driver` maps only `DriverType::Mysql` to true.
- Default-safe API: `split_statements`, `classify_statement_safety`,
  `classify_script_safety`, `validate_against_policy` keep the Postgres
  (standard-strings) reading — correct everywhere except MySQL, where it can
  only over-split/over-classify, never hide a statement.
- Behavior change worth noting: `execute`/`explain`/`execute_for_export` now
  hit the connection repo before the multi-statement gate; an unknown
  connection reports "not found" rather than "empty SQL statement". Test
  helpers (`build_service`) got a `get_config` mock accordingly.
