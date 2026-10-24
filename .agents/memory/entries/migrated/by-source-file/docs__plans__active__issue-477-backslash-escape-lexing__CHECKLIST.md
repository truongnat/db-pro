---
schema_version: 1
id: "MEM-SRC-7ab2cc1e974b8cb8"
title: "Source mirror: docs/plans/active/issue-477-backslash-escape-lexing/CHECKLIST.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-477-backslash-escape-lexing/CHECKLIST.md"
source_sha256: "7ab2cc1e974b8cb822bbb8816d7a67b8ad4c232daaa8aab15a9c584e01e513f4"
mirror_body_sha256: "7ab2cc1e974b8cb822bbb8816d7a67b8ad4c232daaa8aab15a9c584e01e513f4"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
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
