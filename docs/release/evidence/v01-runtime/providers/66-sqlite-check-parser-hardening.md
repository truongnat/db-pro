# SQLite CHECK parser hardening (#69)

- Depends on: #68 KEEP
- Commit lands with named-constraint extraction and additional fixtures

## Coverage vs acceptance

| Acceptance item | Evidence |
|---|---|
| Nested parentheses | Existing `introspection_extracts_each_nested_check_constraint` + nested AND form in literal/comment test |
| Quoted literals with parentheses/keywords | Literal/comment test + escaped `O''Brien` fixture |
| Escaped quotes | `introspection_preserves_named_check_constraints_and_escaped_quotes` |
| Case-insensitive CHECK | `parse_check_constraints_is_case_insensitive_and_ordered` |
| Named and unnamed | Named `CONSTRAINT chk_name` preserved; unnamed → `{table}_check_{idx}` |
| Multiple CHECKs | Both integration fixtures |
| Column-level vs table-level | Column `age INTEGER CHECK` + table `CONSTRAINT … CHECK` |
| Deterministic order | CREATE TABLE scan order |
| Malformed fails safely | `parse_check_constraints_stops_safely_on_unbalanced_parentheses` — truncated CHECK not fabricated |

## Tests

```text
cargo test -p db-pro-infrastructure --lib check_constraint
# 4 passed (plus existing nested/literal tests under introspection_*)
```
