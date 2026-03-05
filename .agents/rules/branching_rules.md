---
description: Branching strategy for db-pro thread workflows
trigger: always_on
---
# Branching Rules
Schema: antigrav.rule@v1
```json
{
  "strategy": "feature-branch-per-thread",
  "prefix": "thread/db-pro/",
  "allow_auto_create": true,
  "allow_auto_checkout": true,
  "cleanup_after_merge": false
}
```

## Policy Intent
- Every thread runs on an isolated branch for traceability.
- Auto-create/checkout is enabled to reduce setup friction during workflow launches.
- Branch cleanup is manual by default to preserve debugging context after merges.

## Naming Guidance
- Branches follow: `thread/db-pro/<thread_id_sanitized>`.
- Thread IDs should be descriptive and map to the active objective.
