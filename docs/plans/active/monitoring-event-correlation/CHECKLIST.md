# Checklist

## Implementation
- [x] Idle repaint scheduled only for active, connected, auto-refresh Monitor.
- [x] Snapshot/workload request identity tracked and validated.
- [x] Users/privileges/memberships/RLS request identity tracked and validated.
- [x] Audit, pg_settings, FDW, replication, and event-trigger loaded responses validate request and active connection.
- [x] Data-diff responses validate request, source, target, and active source connection.
- [x] Mutation-triggered inventory refreshes replace pending request identity.
- [x] Audit state is cleared on disconnect and connection switch.
- [x] Loaded events preserve the originating connection ID across runtime/native protocol.

## Verification
- [x] Monitoring repaint delay regression test.
- [x] Request/connection matching regression test.
- [x] Monitoring and security focused tests pass.
- [x] App/runtime/native cross-crate check and formatting pass.
- [x] Diff/callsite audit complete.

## Delivery
- [ ] Rebase on latest `origin/main`.
- [ ] Commit, push, and draft PR linked with `Fixes #489` and `Fixes #473`.
