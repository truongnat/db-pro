# Issue #503 — Verification

- `cargo check -p db-pro-core` — clean.
- `cargo test -p db-pro-core` — 502 passed, 0 failed, including:
  - `row_rule_matching_is_case_insensitive`: header `Email` masked by profile
    rule `email` → `"***"`.
  - `unmatched_rules_are_surfaced`: `unmatched_mask_rules` returns `["ssn"]`
    for headers `["id","email"]`; `preview.unmatched_rules` and
    `preview.message` both name the missed column; the matching `Email` rule
    still applied (`"[masked]"`).
- `cargo test -p db-pro-app masking` — 2 passed.
- `cargo check --workspace` — clean.
