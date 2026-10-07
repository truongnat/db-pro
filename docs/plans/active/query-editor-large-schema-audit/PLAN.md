# Query Editor large-schema audit

State: PLANNING. Audit lane Done; implementation not requested in this audit. Date 2026-10-06. Baseline/current HEAD: `3dd988e2ff06ac71942c7ec331acf66389d3c5cb`, main. Existing dirty Grid, Select, Welcome, runtime invalidation and macOS menu changes preserved.

Owner reports PostgreSQL company DB with over 500 tables: load crash and lag across Query features, including column suggestions. Scope: inspect connect/schema summary/UI reducer/completion/popup/result render path; collect executable synthetic performance and panic evidence. No company DB access, no credential retrieval, no production code changes.

Architecture: PostgreSQL metadata queries run in runtime/infrastructure. Native UiEvent reducer builds symbol index synchronously. Query Editor resolves completion on UI thread and paints floating popup; output pane receives a cloned UiQueryResult. Provider-neutral UiSchemaSummary feeds editor.

Deliverable: severity findings, exact source SHA/anchors, fixture measurements, repro script, supported/pending provider matrix, prioritized minimal remediation. A full company crash cause requires actual crash/backtrace and reproduction scenario; synthetic failure is not equivalent to reproducing the reported crash.
