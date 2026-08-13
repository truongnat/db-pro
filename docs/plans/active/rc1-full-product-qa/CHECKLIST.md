# RC1 Full Product QA — Remediation Checklist

Baseline audited: `6e0a04ad675eaa85cae08bbe1a066270596a18db`

## Gate 0 — Audit integrity

- [x] Freeze source baseline at RC1 SHA
- [x] Keep audit docs off frozen `main`
- [x] Read `AGENTS.md` and `REVIEW.md`
- [x] Audit connection, workspace, query, grid, schema/ER, shell and provider boundaries
- [x] Separate source-confirmed findings from runtime-only claims
- [ ] Independent Kilo/Cubic challenge of the audit findings

## Gate 1 — QA-W1 Precision & mutation identity

### QA-P1-01 BIGINT/i64 precision
- [x] Define lossless IPC contract for i64/BIGINT
- [x] Remove frontend `int64: number` unsafe representation
- [x] Ensure QueryParam mutation path accepts exact 64-bit values
- [x] Verify Data Grid PK identity uses exact values
- [x] Verify sort/copy/export do not round
- [x] Add PostgreSQL INT8 exact round-trip tests
- [x] Add SQLite >2^53 exact round-trip tests
- [x] Test `i64::MAX` / `i64::MIN`

### QA-P1-02 Preview/staged cross-resource contamination
- [x] Preview with staged changes cannot be silently replaced
- [x] First staged edit promotes preview or marks it protected
- [x] Preview replacement clears all tab-scoped state when safe
- [x] Quick Open preview navigation obeys same rule
- [x] Test A staged → B preview cannot apply A mutation to B

### QA-P1-03 Dirty/close lifecycle
- [x] Define shared `hasUnsavedWork(tabId)`
- [x] Staged edits participate in dirty/close guard
- [x] Close one tab prompts when staged work exists
- [x] Close Others/Right/All include staged work
- [x] Preview replacement includes staged work guard
- [x] Reassign connection includes staged work guard
- [x] Closing/discarding garbage-collects staged changes
- [x] Restart preserves pending state only when intentional and visible

### QA-P1-04 SQLite metadata
- [x] Table Data uses introspected SQLite column metadata
- [x] INTEGER/BLOB/text metadata represented correctly enough for edit policy
- [x] Unknown/dynamic SQLite values have safe fallback
- [x] Editing unsupported/ambiguous types is disabled with reason
- [x] SQLite typed-edit regression tests

## Gate 2 — QA-W2 Connection correctness

### QA-P1-05 SQLite password
- [x] SQLite create does not require password
- [x] SQLite edit does not show meaningless password requirement
- [x] Test/Test+Save work with empty password

### QA-P1-06 SSH disable
- [x] Uncheck Use SSH → model `sshTunnel = undefined`
- [x] Save persists disabled state
- [x] Re-open shows disabled state

### QA-P1-07 SSH default model
- [x] Enable SSH initializes complete default model
- [x] Port 22 exists in submitted state without touching field
- [x] Required Rust SSH fields are represented in TS state
- [x] Validation tests for host/port/user/key

### QA-P1-08 stale connection-dialog async callback
- [x] Introduce per-open operation/session generation
- [x] Late callbacks from previous session are ignored
- [x] New Connection always begins with no persisted edit identity
- [x] Test close-before-create-resolves → reopen → must create, never update old record

### QA-P1-09 session restoration
- [x] Make connection-list fetch side-effect free
- [x] Move startup restore to explicit one-shot coordinator
- [x] Restore active IDs exactly once per app startup/session hydration
- [x] Normal invalidation/refetch does not reconnect
- [x] Connect success does not immediately double-connect
- [x] Failed reconnect does not block other reconnects
- [x] Lifecycle tests cover connect/disconnect/reconnect/refetch

## Gate 3 — QA-W3 Workspace recovery

### QA-P1-10 orphan dirty close
- [ ] Orphan Close uses shared close guard
- [ ] Dirty SQL is never discarded without confirmation
- [ ] Staged grid work is never discarded without confirmation

### QA-P1-11 orphan reassignment
- [ ] Query reassignment still clears old execution/runtime context
- [ ] DB Object reassignment validates target schema/object
- [ ] Schema Workspace reassignment validates target schema
- [ ] PostgreSQL → SQLite incompatible schema is not silently retained
- [ ] Reassignment does not carry stale results/staged mutations
- [ ] Add provider-crossing recovery tests

## Gate 4 — QA-W4 ER large-schema

### QA-P1-12 true search-first mode
- [ ] >200 tables initially renders bounded node set / search prompt
- [ ] Initial open does not Dagre-layout full schema
- [ ] “Show all N” is explicit opt-in
- [ ] Search result/seed produces bounded neighborhood
- [ ] Test 500/1000 table initial node count invariant

### QA-P1-13 safe first-paint LOD
- [ ] Large schema initializes tier 0/1 before first commit
- [ ] Full column rows never mount before viewport state is known
- [ ] Selected/focused table can hydrate full detail
- [ ] Benchmark includes rendering strategy, not only metadata indexing

## Gate 5 — QA-W5 Provider type matrix

### QA-P1-14 PostgreSQL non-basic types
- [ ] Define supported PostgreSQL result type matrix
- [ ] NUMERIC/DECIMAL has lossless representation
- [ ] BIGINT shares lossless integer contract
- [ ] DATE
- [ ] TIME/TIMETZ
- [ ] TIMESTAMP/TIMESTAMPTZ
- [ ] INTERVAL
- [ ] INET
- [ ] UUID
- [ ] JSON/JSONB
- [ ] BYTEA
- [ ] enum/domain behavior
- [ ] arrays behavior or explicit unsupported UI
- [ ] unsupported type cannot crash/fail whole result without actionable reason where safe fallback exists

## Gate 6 — P2 Workspace/Shell

- [ ] QA-P2-01 canonical pinned tab ordering
- [ ] QA-P2-02 platform shortcut labels in tab context menu
- [ ] QA-P2-03 conditional macOS traffic-light inset
- [ ] QA-P2-04 visible Agent Preview/Coming Soon state
- [ ] QA-P2-05 platform-correct Agent shortcut hint
- [ ] QA-P2-06 migrate product-visible hardcoded English to i18n

## Gate 7 — P2 Data Grid

- [ ] QA-P2-07 Columns checkbox toggles once per click
- [ ] QA-P2-08 readonly connection disables edit/delete affordances early
- [ ] QA-P2-09 context menu viewport clamp + Escape + outside click + focus semantics
- [ ] QA-P2-10 keyboard-accessible resize
- [ ] QA-P2-11 resize cleanup on unmount/interruption
- [ ] QA-P2-12 large result sort budget / threshold strategy
- [ ] QA-P2-13 SearchView debounce/index/virtualization where needed
- [ ] QA-P2-14 Explorer large schema virtualization/capped rendering

## Gate 8 — P2 Connections

- [ ] QA-P2-15 Test Connection result resets after config edit
- [ ] QA-P2-16 preserve backend test error detail
- [ ] QA-P2-17 SQLite Browse failure feedback
- [ ] QA-P2-18 driverChanged reflects current vs initial driver
- [ ] QA-P2-19 duplicate credential semantics explicit
- [ ] QA-P2-20 favorite optimistic rollback
- [ ] QA-P2-21 SQLite welcome/recent display uses file-oriented metadata

## Gate 9 — P2 Query/ER

- [ ] QA-P2-22 Export enabled only when exportable result exists
- [ ] QA-P2-23 replace `window.confirm` with app confirmation component
- [ ] QA-P2-24 ER search disambiguation/explicit selection
- [ ] QA-P2-25 memoize ER derived schema table list
- [ ] QA-P2-25 verify dynamic LOD edge handles
- [ ] QA-P2-25 replace synthetic Fit View key with React Flow API

## Gate 10 — Hidden/deferred surface

- [ ] QA-D1 atomic saved-query rename before Saved Queries is exposed

## Gate 11 — Automated verification

- [ ] Frontend typecheck
- [ ] Frontend lint
- [ ] Frontend format check
- [ ] Token drift check
- [ ] Frontend full tests
- [ ] Frontend build
- [ ] Rust fmt
- [ ] Rust check
- [ ] Rust clippy `-D warnings`
- [ ] Rust full tests
- [ ] PostgreSQL integration suite
- [ ] SQLite integration suite
- [ ] Kilo review P0=0/P1=0
- [ ] Cubic review P0=0/P1=0

## Gate 12 — Runtime verification

### Home/local
- [ ] launch/restart
- [ ] SQLite browse/connect/query
- [ ] PostgreSQL available local fixture smoke if possible
- [ ] dirty/staged close guards
- [ ] preview replacement safety
- [ ] query run/stop/selection/current/all
- [ ] read-only UX + backend block
- [ ] theme light/dark baseline

### Office real-data
- [ ] 500+ table ER initial open
- [ ] search-first behavior
- [ ] neighborhood 2-hop
- [ ] Show All behavior
- [ ] pan/zoom/frame responsiveness
- [ ] LOD tier transitions
- [ ] dynamic relationship anchors after LOD transitions
- [ ] memory does not grow unbounded
- [ ] PostgreSQL type matrix against representative production-like schema
- [ ] BIGINT exact identity fixture

### Cross-platform/package
- [ ] macOS packaged build smoke
- [ ] Windows packaged build smoke or recorded delegated evidence
- [ ] Linux packaged build smoke or recorded delegated evidence
- [ ] shortcut labels/top chrome platform-correct

## Final release gate

- [ ] P0 = 0
- [ ] P1 = 0
- [ ] P2 accepted/fixed/deferred explicitly
- [ ] no unresolved review thread against latest head
- [ ] final docs commit only after runtime evidence
- [ ] exact final SHA CI green
- [ ] tag `v0.1.0`
