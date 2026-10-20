# Tasks — BSN Real-Data Findings Remediation

- [x] Establish scope, feature inventory, privacy rules and the BSN read-only safety boundary.
- [x] Verify saved-profile connection and safe runtime reads without retaining row contents.
- [x] Build the per-feature evidence manifest; mark unavailable native screens and states explicitly.
- [x] Trace feature paths, safety gates, existing evidence and initial source-only candidates.
- [ ] Add a failure-injection regression for table-copy `Replace`; make all planned target writes one provider transaction and verify rollback on SQLite and disposable PostgreSQL. Failure injection/source cutover/SQLite are verified; PostgreSQL runtime is blocked because the local fixture login is unavailable.
- [x] Reconcile duplicate locale mappings, check key/placeholder parity, and verify affected diagram translations in all supported languages.
- [x] Localize confirmed high-impact native UI strings and correct the Korean export wording; preserve existing actions and layout. Scope: audited Settings General, destructive/unsaved Query dialogs, Table Data, row conflict and insert-row flows.
- [ ] Revalidate historical table clipping candidates on the current source/capture path; fix only reproduced issues.
- [x] Run focused tests, workspace/build gates and native capture smoke where available; record exact status. Workspace tests/build pass; Clippy has inherited failures; native fixture capture timed out without producing an image.
- [x] Update per-feature checklist, findings, verification, status, and agent evidence with exact results and Vietnamese handoff. Remaining provider/native visual blockers are explicit; plan stays active.
