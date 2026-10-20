# Context — BSN Real-Data Native Audit

- Product UI is native `eframe`/`egui` in `crates/ui` + `crates/app`; `crates/native-app` bridges typed `UiCommand`/`UiEvent` to `db-pro-runtime`. Tauri/web frontend is archived/legacy.
- Source baseline: `cb6e00d633772de538498384a8b14d32d25f7915`.
- App inventory: `Activity` has 15 user-facing routes (`crates/app/src/app/app_types.rs:57-74`); Explorer, Query, Table, Schema, Diagram, Compare, Files, Tools, Monitoring, Security, Transfers, History, Problems, Tasks, Settings; Agent is an additional panel. Exact feature matrix is in `CHECKLIST.md`.
- The persisted normal-profile row named `BSN` is PostgreSQL with DB Pro `readonly=false`. Therefore the audit itself must enforce a stricter manual read-only boundary; never rely on the UI connection flag to protect live business data.
- `docs/architecture/security-boundaries.md` says credentials stay outside metadata/logs and safety policy is backend-enforced. Do not print or persist credentials.
- `crates/native-app/src/capture.rs` routes `DB_PRO_CAPTURE_*` scenarios through capture openers; `crates/app/src/workspace/data_cleanup_capture.rs` explicitly labels its contents presentation fixtures. Those captures are synthetic, not BSN runtime evidence.
- Prior reference plans: `ux-friendliness-audit/` has the broad 13-area source/UX inventory; `native-feature-ui-cleanup/` audits shared layout; `native-ui-focus-consistency/` audits keyboard focus. Reuse narrow evidence only when its SHA/path/state matches; do not count old Tauri/fixture screenshots as current BSN proof.
- BSN read-only runtime smoke now succeeded: `SELECT 1`, plain non-ANALYZE `EXPLAIN`, introspection summary and one-row fetch from a populated table. The row values were not displayed, logged or saved. No write, export, or backup was performed. Native UI remains unverified because existing capture evidence is synthetic and same-environment interaction failed earlier; see `VERIFICATION.md`.
