# Verification — QA-P1-09 Connection-list query reconnect side-effect

## Verification Plan

### Automated Unit Tests
- `frontend/src/modules/connection/__tests__/session-restoration.test.tsx`: Verify `restoreSession` behavior and one-shot execution.
- `frontend/src/modules/connection/__tests__/connection-queries.test.tsx`: Verify refetching `useConnectionList` does not trigger `service.connect()`.

### Quality Gates
- `cd frontend && npm run test`
- `cargo check -p db-pro-core -p db-pro-infrastructure`
