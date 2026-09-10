# UX Friendliness Audit — Checklist

- [x] Read STATUS.md, FEATURE_LIFECYCLE.md, RC1 PLAN/FINDINGS
- [x] Inventory frontend routes, shell, modules; inventory Tauri commands
- [x] Per-feature friendliness verdict with source evidence
- [x] Rank UX-P1 vs UX-P2, propose fix waves
- [x] Runtime A4 search-first check: PostgreSQL fixture with 500 temporary tables opened as a 510-table ER search-first workspace; Light/Dark/System theme states observed
- [x] Runtime verification: ER table selection, bounded neighborhood, Show All, and Fit View control activation on the 510-table PostgreSQL fixture
- [ ] Runtime verification: keyboard-only end-to-end pass (blocked by the current Wayland desktop provider's inability to focus the Tauri window for key injection)
- [x] Implement and verify feasible source fixes; user-directed work is on `main`, while runtime/provider/API gaps remain a separate gate
- [x] Deep static UI/UX audit using UI/UX, accessibility, desktop-HIG, and design-review skills
- [x] Record systemic keyboard, discoverability, contrast, motion, semantics, localization, and stress-state findings
- [x] D1/D2: keyboard parity and focus-visible discoverability for custom/hover-only actions
- [x] D3: rendered contrast matrix for both themes and semantic states
- [x] D4: reduced-motion policy and keyboard-safe snackbar dismissal
- [x] D5/D6: semantic tab/landmark model and remaining EN/JA copy sweep
- [ ] D7: narrow-window, 200% text, long-identifier, RTL/locale, reduced-motion, and keyboard runtime matrix
