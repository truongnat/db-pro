---
schema_version: 1
id: "MEM-SRC-2ebf5bd37f4072b7"
title: "Source mirror: docs/plans/active/ui-rules-compliance/CHECKLIST.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/ui-rules-compliance/CHECKLIST.md"
source_sha256: "2ebf5bd37f4072b75fe295ee417a1c4667aefaf578d0f7c7205e725180c46b8a"
mirror_body_sha256: "2ebf5bd37f4072b75fe295ee417a1c4667aefaf578d0f7c7205e725180c46b8a"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# CHECKLIST — ui-rules-compliance

## W1 — §1 component-first (blocking gate: `RULES.md` §6 empty)

- [ ] W1a `query editor surface` + `query intelligence` — 23 sites
- [ ] W1b `table data editor` — 16 sites
- [ ] W1c `result grid / output` + `component gallery` — 14 sites (+ inline `ui-rules:` for the gallery reference baseline)
- [ ] W1d `shell / workspace` + `explorer / sidebar` + `palette / quick open` — 12 sites (+ inline `ui-rules:` for `shortcut_catalog_tests.rs`)
- [ ] W1e `agent` + `files / tasks ide` + `monitoring` + `management / admin` + `settings` — 11 sites
- [ ] W1f `schema workbench / compare` + `diagram` + `connection dialog` + `security` + `runtime bridge / events` — 8 sites
- [ ] W1-gate `RULES.md` §6 command returns empty
- [ ] W1-gate `scripts/check-ui-architecture.sh` PASS
- [ ] W1-gate `cargo fmt --all -- --check`
- [ ] W1-gate `cargo check --workspace`
- [ ] W1-gate `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] W1-gate `cargo test --workspace`
- [ ] W1-runtime `capture` screenshots light+dark at 1280×800 / 1440×900 / 1920×1080 for affected surfaces

## W2 — §3 tokens (non-blocking, separate wave)

- [ ] W2a Inventory literal colours/metrics per cluster, classify canvas-paint vs layout
- [ ] W2b Map literals to `SPACE_*` / `RADIUS_*` / `STROKE_*` / `font_*()` where a token exists
- [ ] W2c `Color32::` literals in view code removed (except `TRANSPARENT`/`PLACEHOLDER`)
- [ ] W2d `components/` §3 violations fixed (button/select/selection/legacy/agent_primitives/dialog)

## W3 — enforcement (after W1 is green)

- [ ] W3a Extend `scripts/check-ui-architecture.sh` regex to the full §1 pattern list
- [ ] W3b Align script path exemptions with `RULES.md` §6 (tests, gallery)
- [ ] W3c Add a §3 literal check with a frozen baseline (ratchet, new violations fail)

## Done

- [ ] Plan moved to `docs/plans/completed/`, `docs/plans/STATUS.md` row added
