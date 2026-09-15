# Checklist — #147

- [x] Reproduce the defect with a live probe (PostgreSQL 16 fixture + SQLite), measure all shapes
- [x] Implement `transaction_control_verb()` in `core/domain/safety.rs` + unit tests
- [x] Implement the dispatch guard in `QueryService::execute_multi` + unit tests + rejection message
- [x] Live SQLite integration tests (product refusal / contract / connector characterisations)
- [x] Live PostgreSQL integration tests (product refusal / contract / connector characterisation)
- [x] Probe files removed; no scope creep (single statements left to #224, connectors unchanged)
- [x] Docs: §5 "what a batch may contain" + execution-safety matrix row (docs/09)
- [x] Evidence file per AGENT_EVIDENCE template
- [x] Gates: fmt / check / clippy -D warnings / test workspace / native release build / perf-scan
- [x] Clean branch from `a9244a2`, only #147 files committed
- [x] PR published + claim comment updated with SHA and evidence
