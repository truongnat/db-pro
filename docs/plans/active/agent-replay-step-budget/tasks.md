# Tasks — #448

- [x] T1 / R1–R4: Scope approved; baseline runtime reproduction recorded (D4).
- [x] T2 / R1–R3: Add bounded replay/error/rejection and limit-boundary regression coverage to the existing adjacent test file; observe a failing regression before production edits (D1–D4). Baseline: 15 passed, 3 failed.
- [x] T3 / R1–R3: Move existing budget enforcement before completed-call cache replay; retain pending-confirmation handling and all execution caches (D1–D3). Depends on T2.
- [x] T4 / R4: Run affected suite, independent public-orchestrator smoke, and Rust quality gates (D4–D5). All executed; workspace Clippy fails on unchanged app sources, other gates pass.
- [x] T5 / R1–R4: Record actual acceptance/gates/limitations and handoff state (D6). Scope implemented; lifecycle REVIEW, not COMPLETED or merge-ready.
