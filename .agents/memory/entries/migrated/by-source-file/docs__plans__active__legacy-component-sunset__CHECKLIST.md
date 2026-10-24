---
schema_version: 1
id: "MEM-SRC-5a8acd62194ed8b8"
title: "Source mirror: docs/plans/active/legacy-component-sunset/CHECKLIST.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/legacy-component-sunset/CHECKLIST.md"
source_sha256: "5a8acd62194ed8b8daac21f420401b93048fef0501f41527b47148c2c4091ef6"
mirror_body_sha256: "5a8acd62194ed8b8daac21f420401b93048fef0501f41527b47148c2c4091ef6"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Checklist — Legacy Component Sunset

- [x] Wave 0: delete 11 verified-dead legacy fns + `UiLevel`/`level_colors` — workspace check PASS
- [x] Wave 1: `Stroke::new(<literal>)` → `STROKE_*` (23 sites) + focus-ring unification — committed `2985e3fb`
- [x] Wave 2: floating-surface radii → `RADIUS_*` roles + overlay spacing consts — `e89d555f`
- [x] Wave 3: config const aliases → tokens; `toggle::ICON_TEXT_GAP` rename; `INPUT_AUX_GAP` deleted — `1c2586f0`
- [x] Wave 4: `selection/ui.rs` literals → config/tokens — `11ff76d7`
- [x] Wave 5: 77 button sites + 39 helper sites migrated; `legacy.rs` deleted entirely — `a67107a6`, `fef890eb`, `da4c7bfa`
- [x] Wave 6 gates: fmt ✓ clippy 0 ✓ tests 2257/0 ✓ ui-architecture PASS ✓ release build PASS ✓ cc-ratchet 0 fail ✓
- [x] Focus-ring per-site visual check — manual Tab pass (orca/cliclick): ring verified on topbar icon (`nav/ui.rs:81`) + Welcome card (`welcome_surface_view.rs:204`) — `evidence/focus_ring_{topbar,card}.png`
- [ ] Pending (RUNTIME_VERIFY): open-popover radius visual check (Quick Open capture hook hangs — pre-existing infra), 1440×900/1920×1080 captures
