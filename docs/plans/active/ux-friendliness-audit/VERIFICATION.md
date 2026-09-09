# UX Friendliness Audit — Verification

- Source evidence: STATUS.md, FEATURE_LIFECYCLE.md, RC1 PLAN/FINDINGS/WAVE1_REPORT read; frontend `src/routes`, `commons/components/shell`, `modules/{connection,query,data-grid,schema,er-diagram,export,backup,user-management}` inventoried; `crates/tauri-app/src/lib.rs:157-213` command groups inventoried.
- Automated: none executed in this planning step.
- Provider runtime: PENDING (PG and SQLite independently).
- UI runtime: PENDING (packaged desktop smoke, keyboard-only, light/dark, 500-table ER).
- No feature marked COMPLETED. Next: runtime walkthrough, then focused `fix/` branches per Wave A/B/C.
