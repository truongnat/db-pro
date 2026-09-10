# Findings: QA-P2-22 & QA-P2-21

## QA-P2-22 — Query Export enablement is tied to SQL text instead of actual result state
- **Severity**: P2
- **Area**: Query execution / Toolbar & Menu
- **Evidence**: `frontend/src/modules/query/components/query-command-bar.tsx` disables the Export Results menu item based on `!hasSql`.
- **Impact**: Clearing the query editor after executing a query disables Export Results despite valid rows being present on screen. Conversely, typing SQL without running it enables Export Results despite no rows existing.
- **Fix**: Update `QueryCommandBar` to check `hasResults` (`!!result`) instead of `hasSql`.

## QA-P2-21 — SQLite recent connection subtitle renders meaningless host/port `:0`
- **Severity**: P2
- **Area**: Desktop UI / Welcome View
- **Evidence**: `frontend/src/commons/components/welcome-view.tsx` renders `{conn.host}:{conn.port} / {conn.database}` for all connection drivers.
- **Impact**: SQLite connections (which have empty `host` and `port: 0`) render as `:0 / <path>`.
- **Fix**: Format SQLite connection subtitles as `{conn.database}`.
