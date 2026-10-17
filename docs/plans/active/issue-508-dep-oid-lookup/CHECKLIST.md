# #508 checklist

- [x] WHERE clause is `d.objid = to_regclass($1)` (OID equality)
- [x] qualified `"schema"."object"` bound as one param via `quote_ident`
- [x] no `LIKE`, no `|| '.%'` pattern concatenation
- [x] `pg_namespace` join removed (no longer needed; filter is OID)
- [x] `d.classid = 'pg_catalog.pg_class'::regclass` filter kept
- [x] tests/cross_connection_tests.rs added; mod decl in source file
- [x] `cargo check -p db-pro-infrastructure` clean
- [x] `cargo test -p db-pro-infrastructure --lib` green: 167 passed / 0 failed
