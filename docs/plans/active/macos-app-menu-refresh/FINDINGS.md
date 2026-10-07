# Findings

Baseline inspected at `main@3dd988e2ff06ac71942c7ec331acf66389d3c5cb`.

- **P2 — Native app menu was absent.** Fixed in the working tree by installing the native app/File/Edit/View/Window/Help menus and dispatching their actions to the UI. Live menu behavior remains pending.
- **P2 — Explicit cache refresh was not exposed.** Fixed in the working tree by adding a typed cache-invalidation flag and invalidating before forced introspection. Automated UI dispatch tests pass; SQLite/PostgreSQL runtime behavior remains pending.

No P0/P1 finding is in scope. These are source findings, not live UI observations.
