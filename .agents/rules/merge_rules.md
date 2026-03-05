---
description: Merge safety policy for db-pro thread lifecycle
trigger: always_on
---
# Merge Rules
Schema: antigrav.rule@v1
```json
{
  "require_validation_before_merge": true,
  "analyze_conflicts": true,
  "auto_conflict_resolution_assist": true,
  "auto_conflict_resolution_strategy": "ours",
  "auto_conflict_resolution_max_attempts": 2,
  "delete_feature_branch_after_merge": false,
  "protected_branches": ["main", "master"],
  "require_rebase_before_merge": true
}
```

## Policy Intent
- No merge without validation evidence.
- Conflict analysis is mandatory; auto-resolution can assist but is bounded.
- Protected branches require disciplined integration (rebase + verified checks).

## Escalation Rules
- If conflicts persist after auto-resolution attempts, require explicit manual resolution guidance.
- If validation fails post-merge attempt, merge is blocked until fixed.
