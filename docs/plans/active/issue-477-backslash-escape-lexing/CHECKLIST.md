# #477 checklist

- [x] `skip_quoted` takes `escapes_enabled`; `\x` skipped only when set
- [x] `E'…'`/`e'…'` prefix detection (`is_escape_string_start`), escape mode
      for that literal regardless of driver; identifier-glued `someE'` excluded
- [x] `backslash_escapes` threaded through `special_token_end`,
      `tokenize_sql`, `matching_parenthesis_end`, `contains_sql_keyword`,
      `compact_clause`, `strip_outer_parentheses`, `top_level_where_clause`,
      `where_clause_is_tautology`, `classify_explain_safety`,
      `classify_merge_safety`, `statement_lacks_effective_where`,
      `has_top_level_sql_keyword`, `classify_cte_safety`
- [x] Driver reaches the gate: `driver_for` in QueryService; config.driver in
      ExportService/SchemaService; `reject_multi_statement(sql, driver)`
- [x] MySQL keeps `\'` escape (`backslash_escapes_for_driver(Mysql)`)
- [x] Regression tests in `safety/tests.rs` + `sql_policy_tests.rs`
- [x] `cargo test -p db-pro-core` green (502 passed)
- [x] `cargo check --workspace` green
