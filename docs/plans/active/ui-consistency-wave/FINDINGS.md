# Findings — UI consistency wave

Consolidated detail: `REPORT.md`. Persistent audit memory:
`.skills/ui-consistency-audit/references/findings-ledger.md`.

## Counts at baseline `89095cc7`

- P0: 0
- P1: 3 (UI-01 explorer toolbar overlap, UI-02 composer order, UI-03 schema picker
  reachability)
- P2: 22

## Root causes

1. `Select::show` re-anchors itself with `with_layout(top_down(Align::Min))`
   (`select.rs:170`) and defaults to the light theme (`select.rs:83`).
2. Component state coverage is incomplete: disabled, error/required, option
   semantics, Escape-revert.
3. Container layouts are hand-rolled and nest RTL/LTR incorrectly (explorer toolbar,
   history filter) or reverse authored order (composer `bottom_up`).
4. Strings, sizes, and widths are duplicated as literals instead of tokens/locales.

## Resolution (commit `0e370498` on top of `89095cc7`)

- P1: 3/3 fixed (explorer toolbar, composer order, schema picker scroll).
- P2: 20/22 fixed; UI-20 (dialog width presets + destructive footer) and UI-22
  (~417-string i18n sweep) deferred with rationale in `VERIFICATION.md`.
- Evidence: 42 runtime captures in `/tmp/ui-audit/wave2/`; gates recorded in
  `VERIFICATION.md`. No new P0/P1 introduced.

## Baseline vs regression

No regression is introduced by this wave; every finding is inherited from the baseline.
Verification must re-measure the same geometry and report inherited issues that remain.
