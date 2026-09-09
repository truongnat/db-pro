# UX Friendliness Audit — Checklist

- [x] Read STATUS.md, FEATURE_LIFECYCLE.md, RC1 PLAN/FINDINGS
- [x] Inventory frontend routes, shell, modules; inventory Tauri commands
- [x] Per-feature friendliness verdict with source evidence
- [x] Rank UX-P1 vs UX-P2, propose fix waves
- [x] Runtime A4 search-first check: PostgreSQL fixture with 500 temporary tables opened as a 510-table ER search-first workspace; Light/Dark/System theme states observed
- [x] Runtime verification: ER table selection, bounded neighborhood, Show All, and Fit View control activation on the 510-table PostgreSQL fixture
- [ ] Runtime verification: keyboard-only end-to-end pass (blocked by the current Wayland desktop provider's inability to focus the Tauri window for key injection)
- [x] Implement and verify feasible source fixes; user-directed work is on `main`, while runtime/provider/API gaps remain a separate gate
