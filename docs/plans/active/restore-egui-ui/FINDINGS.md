# Findings

Owner chose the existing egui Query appearance as the UI baseline. Integration commits after `c0c1f5525b20a810913d1eee13c7ee2dd15b6664` exclusively introduced rs-ui adapters/rendering and their support code; restoring the affected source/manifests to that revision is a coherent rollback. No history reset is needed. Existing rs-ui implementation evidence remains historical; migration is deferred.

Provider impact: PostgreSQL/SQLite backend code unchanged; no new provider behavior claimed. Preserve egui virtual row windows, persisted column widths and existing selection ownership. No performance improvement claim is made without measurements.

Reusable lesson: when a contiguous integration is deferred, restore its affected source/manifests from the verified pre-integration SHA and compare the entire code path to that SHA; preserve historical evidence and mark it deferred instead of resetting history. This task demonstrates an empty source diff against the reference SHA.
