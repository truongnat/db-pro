---
description: Coding quality gate for db-pro changes
trigger: always_on
---
# Coding Rules
Schema: antigrav.rule@v1
```json
{
  "no_unused_imports": true,
  "require_tests_for_new_feature": true,
  "forbid_unrelated_file_changes": true,
  "require_memory_index_update": false,
  "require_structured_commit_message": true,
  "commit_format": "type(scope): summary"
}
```

## Policy Intent
- Optimize for minimal, reviewable diffs with strong regression protection.
- Keep commits semantically meaningful and parseable by tooling.
- Block opportunistic refactors unless explicitly requested by task scope.

## Practical Enforcement
- New feature work should include tests or an explicit justification when tests are not feasible.
- Any touched file must be directly tied to the requested behavior change.
