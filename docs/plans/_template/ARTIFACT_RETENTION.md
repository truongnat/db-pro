# Artifact retention — `<run_id>`

Dùng khi run **invalidated** hoặc **frozen**; đồng bộ với `docs/ARTIFACT_POLICY.md`.

## Tracked in git (do not delete without disposition)

- `coordinator/RUN_INVALIDATION.md` — list paths below
- `workers/**/attempt-*/result.claimed-pass-invalid.json` (if any)
- Worker evidence explicitly named in invalidation (e.g. `launch.png`, `orca-app-state.json`)
- `package-*.log`, `artifact-ready.marker` — rebuild path for gitignored binaries

## Local only (gitignored)

- `artifacts/**/*.tar.gz`
- `artifacts/**/extracted/**/MacOS/db-pro-native`

## Before adding `gitignore` rules

1. Update this file + `RUN_INVALIDATION.md` § Retained evidence.
2. Run `scripts/verify-artifact-policy.sh` (extend script if new frozen run).
3. Grep report links (`VERIFICATION.md`, `docs/release/*.md`).
