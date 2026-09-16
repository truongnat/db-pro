# Native UI Upgrade Execution Map

Parent goal issue: to be created from this baseline.

Recommended sequence:

1. UI visual QA harness and screenshot evidence contract
2. Design-system consolidation / raw egui cleanup
3. Shell + Activity Bar + Sidebar hierarchy
4. Explorer tree + search + context menus
5. Query workspace/editor/output composition
6. Data/result grid + editing states
7. Table/object workbench + schema forms
8. Dialogs/forms/confirmations
9. Files/IDE workspace
10. Agent workspace
11. ER workspace
12. Settings + keyboard/accessibility
13. Empty/loading/error/status feedback
14. Responsive density + overflow hardening

The core rule is runtime-first: each ticket must be manually traversed in the native app and include visual evidence. No UI ticket closes on source inspection alone.
