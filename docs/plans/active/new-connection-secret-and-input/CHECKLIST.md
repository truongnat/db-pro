# new-connection-secret-and-input — Checklist

## Planning
- [x] Evidence and failure scenario recorded
- [x] Scope/non-goals explicit
- [x] PostgreSQL/SQLite support matrix explicit

## Implementation
- [x] Smallest coherent implementation complete (one match arm + two deleted blocks)
- [x] Safety/capability rules respected (no SQL, no mutated secrets; fallback semantics preserved)
- [x] Cache/state invalidation accounted for (no cached state touched)

## Tests
- [x] Regression tests added for important invariants (keyring fallback + input click paths)
- [x] Regression tests revert-verified (each fails without its fix)
- [x] PostgreSQL evidence recorded or explicitly pending/N/A — headless unit verified; live N/A/pending
- [x] SQLite evidence recorded or explicitly pending/N/A — headless unit verified; live N/A/pending

## Review
- [x] Architecture review complete (secret store + egui widget, no cross-layer contract change)
- [x] Correctness/security review complete (data-safe; no swallowed errors introduced)
- [x] P0 = 0
- [x] P1 = 0 (the one P1 was fixed and guarded)

## Runtime
- [x] UI → command → backend → DB → introspection → UI verified when applicable (headless; live DB not required for these paths)
- [ ] VERIFICATION.md contains actual commands/evidence — yes, all four gates recorded with output
- [ ] STATUS.md matches state/folder — pending: set to RUNTIME_VERIFY after this commit
- [ ] UI runtime screenshots at 1280×800 / 1440×900 / 1920×1080 — **SKIPPED** (no capture driver; TCC blocks screencapture). Recorded as a gap, not marked passed.
