# #508 verification

- `quote_ident_wraps_and_doubles_inner_quotes` — plain, spaced, dotted, and quoted
  identifiers all render as single quoted parts.
- `qualified_name_binds_schema_and_object_verbatim` — `%`/`_` stay literal inside
  `"my%schema"."tab_le"`.
- `dependency_lookup_filters_by_oid_not_like_pattern` — source-shape regression:
  `get_object_dependencies` body contains `d.objid = to_regclass($1)`; contains no
  `LIKE` and no `$1 ||`; qualified param built via `quote_ident`.
- `cargo check -p db-pro-infrastructure` — clean.
- `cargo test -p db-pro-infrastructure --lib` — 167 passed / 0 failed, including the three
  tests above.
