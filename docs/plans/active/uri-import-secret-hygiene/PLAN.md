# URI Import Secret Hygiene — PLAN

Baseline: `origin/main` at start of work; implementation branch `fix/clear-uri-password-from-draft`.

## Goal

Prevent a successfully imported database URI from remaining visible in the plain-text connection form after its credentials have been copied into dedicated fields.

## Issue triage

- #479 is confirmed as an on-screen plaintext-retention issue: successful import fills the password field but leaves the original URI in `cloud_snippet`.
- The issue's claims about serialized/session persistence or backend/log exposure were not substantiated by the inspected code; see `FINDINGS.md`.

## Scope

On successful parse/import, clear `UiConnectionDraft.cloud_snippet`. On parse failure preserve it so the user can correct the URI. Add tests for both paths. Do not redesign fields, alter URI parsing, or change stored connection credentials.

## UI/design constraints

Use the existing connection-dialog layout and `DbProTheme`; no new visual styling. The import action consumes a plaintext URI, so clearing it on success reduces exposure while the dedicated password field retains its existing masking. Preserve the current form geometry, labels, and error behavior.

## Verification

Automated tests plus native runtime captures for the connection dialog at 1280×800, 1440×900, and 1920×1080 in light/dark themes. Build gate: `cargo build --release --locked -p db-pro-native`. Do not claim a visual pass unless the captured images are inspected.
