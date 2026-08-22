# Plan: Fix Query Export Enablement (QA-P2-22) and SQLite Recent Connection Subtitle (QA-P2-21)

## Overview
Remediate two confirmed P2 runtime/UX defects in DB Pro:
1. **QA-P2-22**: Query Export enablement in `QueryCommandBar` is currently tied to `hasSql` (`disabled={!hasSql}`) instead of checking whether execution results actually exist (`hasResults`).
2. **QA-P2-21**: Recent connections list in `WelcomeView` renders `:0 / <path>` for SQLite connections due to empty host and zero port.

## Scope & Changes

### 1. Plan Documentation
Maintain plan files in `docs/plans/active/qa-p2-query-export-and-sqlite-subtitle/`:
- `PLAN.md`
- `CHECKLIST.md`
- `FINDINGS.md`
- `VERIFICATION.md`

### 2. Query Export Enablement (`QA-P2-22`)
- Update `QueryCommandBarProps` in `frontend/src/modules/query/components/query-command-bar.tsx` to accept `hasResults?: boolean`.
- Update the Export Results menu item in `frontend/src/modules/query/components/query-command-bar.tsx` to `disabled={!hasResults}` (instead of `disabled={!hasSql}`).
- Pass `hasResults={!!result}` from `frontend/src/modules/query/components/query-tab-content.tsx`.

### 3. SQLite Recent Connection Subtitle (`QA-P2-21`)
- Update `frontend/src/commons/components/welcome-view.tsx` line 183 to render `{conn.driver === "sqlite" ? conn.database : `${conn.host}:${conn.port} / ${conn.database}`}`.

### 4. Tests
- Add a test in `frontend/src/commons/components/__tests__/welcome-view.test.tsx` to verify SQLite recent connection rendering (renders file path without `:0 /`).
- Add a test in `frontend/src/modules/query/__tests__/query-toolbar.test.tsx` to verify toolbar menu behavior.

### 5. Pre-commit Quality Gates
- Execute all frontend quality gates (`npm run typecheck`, `npm run lint`, `npm run format:check`, `npm run check:tokens`, `npm run test`, `npm run build`).
- Execute Rust quality gates (`cargo fmt --all -- --check`, `cargo check --workspace`, `cargo clippy --workspace --all-targets`, `cargo test --workspace`).

### 6. Submission
- Create feature branch `fix/qa-p2-query-export-and-sqlite-subtitle`.
- Publish Pull Request against main without merging.
