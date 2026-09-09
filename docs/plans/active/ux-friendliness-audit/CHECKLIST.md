# UX Friendliness Audit — Checklist

- [x] Read STATUS.md, FEATURE_LIFECYCLE.md, RC1 PLAN/FINDINGS
- [x] Inventory frontend routes, shell, modules; inventory Tauri commands
- [x] Per-feature friendliness verdict with source evidence
- [x] Rank UX-P1 vs UX-P2, propose fix waves
- [x] Runtime A4 search-first check: PostgreSQL fixture with 500 temporary tables opened as a 510-table ER search-first workspace; Light/Dark/System theme states observed
- [ ] Remaining runtime verification: packaged desktop smoke, full PG+SQLite walkthrough, ER table selection/neighborhood/Show All/Fit, keyboard-only pass
- [x] Implement and verify feasible source fixes; user-directed work is on `main`, while runtime/provider/API gaps remain a separate gate
