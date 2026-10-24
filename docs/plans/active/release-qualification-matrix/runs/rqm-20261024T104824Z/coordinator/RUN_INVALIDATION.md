# Run invalidation — `rqm-20261024T104824Z`

**State:** `INVALIDATED` (2026-10-24 audit)

## Why this run cannot qualify release

1. **Compiler pin violation:** Artifact built with ambient `cargo +stable` → **rustc 1.99.0**. PLAN.md / RUNBOOK.md require qualifying pin **1.95.0** (`cargo +1.95.0`). `artifact-ready.marker` and any `role: release` claims from this build are **disqualified** for release gates.
2. **Premature PASS claims:** Three `attempt-1` results were marked PASS without valid `RESULT.schema.json` compliance or AGGREGATION §2 semantic review (independent oracles, inspected evidence kinds, cleanup, geometry).
3. **LIF-01:** Screenshot retained under `workers/L2/.../evidence/` but **not** pixel-reviewed; no action transcript; no startup terminal-state oracle; `inspected: true` was overstated.

## Retained evidence (do not delete)

- `workers/**/attempt-1/result.claimed-pass-invalid.json` — original invalid PASS JSON
- `workers/L2/LIF-01::.../evidence/launch.png`, `orca-app-state.json`
- `artifacts/macos-arm64/*` — **engineering reference only**, not qualifying release artifact

## Downgraded authoritative statuses (`result.json`)

| Instance | Status | Notes |
|---|---|---|
| ENV-01 (macos) | `NOT_TESTED` | Retracted; contract failure |
| REL-01 (macos) | `BLOCKED` | Compiler pin / release-role |
| LIF-01 (macos) | `NOT_TESTED` | Retracted; insufficient oracle evidence |

## Next run (not started here)

New `run_id` required after **`cargo +1.95.0 build --release -p db-pro-native --locked`** and repackage; record actual compiler string matching pin. See `runs/rqm-next-pin195/preflight.json`.
