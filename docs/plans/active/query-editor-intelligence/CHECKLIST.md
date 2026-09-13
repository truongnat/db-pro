# Checklist

- [x] Confirm current HEAD and preserve existing uncommitted Query Editor work.
- [x] Add document-scoped prediction request/event metadata.
- [x] Route prediction through native translator and runtime worker.
- [x] Add cancellation and debounce behavior.
- [x] Add deterministic context fingerprinting, bounded cache, and overlap-safe insertion.
- [x] Add manual prediction trigger and subtle/eager reveal behavior.
- [x] Carry replacement ranges through UI → native → runtime → UI.
- [x] Record local debounce/total latency metrics and provider latency in debug logs.
- [x] Make stale response rejection explicit and tested.
- [x] Verify per-document execution/explain routing.
- [x] Run formatter, workspace checks, clippy, tests, native release build, and performance scan.
- [ ] Verify a live provider request and stale/cancel behavior with native UI interaction.
- [ ] Record native UI evidence at the required viewport/state matrix.
