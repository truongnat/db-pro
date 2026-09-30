# Findings

Baseline: `44d157e2e4c6ad9d83be9ec69414f508f91d38d8`.

| Severity | Finding | Scope |
|---|---|---|
| P2 | Error frame and focus chrome both render a border on focused invalid text/password inputs. | Shared input primitives. |
| P2 | Secondary dismiss button gives destructive alerts a gray square behind the close icon. | Shared alert primitive. |
| P2 | Checkbox/radio cursor behavior reported incorrect in Component Gallery; source already requests PointingHand, so inspect runtime after making the request explicit. | Shared selection primitive. |

Provider impact: PostgreSQL and SQLite are not involved. External PR coverage: not checked; fix is isolated to the native UI branch.

Self-review: P0 0 / P1 0 / P2 1 remaining. The remaining P2 is the native interactive focus/cursor pass; screenshots and headless cursor tests do not prove the OS pointer and the focused invalid field together. The existing gallery layout can extend horizontally at 1280px, so the dismiss icon is positioned against the visible clip edge. This branch leaves that broader layout issue outside its focused scope.
