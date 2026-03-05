# Sprint 1 - S1-05 Incident-Friendly Error Surfaces

## Goal

Make error surfaces actionable instead of generic by classifying failures into stable categories.

## Taxonomy

| Category | Trigger (match rules) | UI Headline | Suggested Action |
|---|---|---|---|
| `cancelled` | message contains `cancelled` | Query cancelled | Adjust SQL or rerun |
| `timeout` | message contains `timed out` / `timeout` | Query timeout | Reduce dataset, add LIMIT, or increase timeout |
| `authentication` | auth failure patterns | Authentication failed | Verify credentials/permissions |
| `network` | connection refused / DNS / route errors | Network connection issue | Check host/port/VPN/firewall |
| `connection` | generic connection failed | Connection failed | Validate connection profile |
| `unsupported` | pushdown/wrapped limitations | Filter/sort pushdown unavailable | Use `SELECT/WITH` for filter/sort pushdown |
| `sql` | syntax/query/execution failures | SQL execution error | Review SQL/object names |
| `unknown` | fallback | Unexpected error | Review logs and retry |

## Implementation

- New module: `src/features/query/errors.ts`
  - `formatError(...)`
  - `isTimeoutError(...)`
  - `classifyQueryError(...)`
- Integrated in `src/App.tsx`:
  - `applyErrorStatus(...)` now maps raw backend/frontend errors into classified `headline + details + action`.

## UX Impact

- Status bar now reports categorized errors.
- Error banner now includes remediation guidance.
- Timeout auto-cancel flow remains unchanged and still triggers backend cancel request.

## Validation (local)

- `npm run -s build`
- `cargo check --manifest-path src-tauri/Cargo.toml`
- `cargo run --manifest-path /Users/truongdq/Documents/GitHub/agentic-sdlc/Cargo.toml -- workflow check`

All passed on 2026-03-05.
