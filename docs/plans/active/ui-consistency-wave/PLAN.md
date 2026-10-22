# UI consistency wave

State: REVIEW. Baseline SHA: `89095cc7d4b056a9fcc19dd9368f7c2863c8759f`. Fix commit:
`0e370498`; gates and runtime captures recorded in `VERIFICATION.md`.

## Scope

Fix the systemic UI consistency findings from the two audit rounds (see `REPORT.md`):
the shared Select contract, affected call sites, the three P1 layout defects, and the
concrete report-1 layout/state findings that share the same components.

## Acceptance

- P1: explorer Refresh no longer overlaps Filter; agent composer renders top-down;
  schema picker scrolls and is keyboard-reachable.
- Select: no layout re-anchoring, no light-theme default, disabled visual, draft
  arrow selection with Escape revert, option semantics, single-row label contract,
  menu rows truncate instead of overflow.
- Call sites: one height per row context, widths from tokens, row caps localized,
  sidebar filters right-aligned, engine select themed in dark.
- Runtime evidence: light + dark at 1280×800, 1440×900, 1920×1080 for affected
  surfaces, including an open Select menu.
- Gates: fmt, check, clippy `-D warnings`, workspace tests, locked release build,
  `scripts/check-ui-architecture.sh`, clean-code diff ratchet.

## Non-goals

- Full ~417-string i18n sweep (tracked as UI-22; only strings touched by this wave are
  localized).
- Full dialog-width preset refactor (UI-20) — documented, not bundled with Select work.
- Provider/runtime behavior changes; no DB mutations.

## Plan docs

- `CHECKLIST.md` — execution checklist
- `FINDINGS.md` — severity and root-cause summary
- `VERIFICATION.md` — commands, counts, captures
- `REPORT.md` — consolidated audit report
