# new-connection-secret-and-input — Checklist

**State: COMPLETED** (runtime-evidence gate closed via the `capture` feature; see `VERIFICATION.md` + `screenshots/`)

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
- [x] VERIFICATION.md contains actual commands/evidence — all four gates recorded with output, plus the capture-evidence run
- [x] STATUS.md matches state/folder — flipped to COMPLETED after the capture-evidence commit
- [x] UI runtime screenshots at 1280×800 / 1440×900 / 1920×1080 — **CAPTURED** via the `capture` feature (`--features capture`); the New Connection dialog renders with the password input + eye toggle and **no Configuration Error** on the disabled-keyring path (PNGs in `screenshots/`)
