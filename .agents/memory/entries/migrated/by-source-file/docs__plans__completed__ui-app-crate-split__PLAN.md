---
schema_version: 1
id: "MEM-SRC-745bca07bc5fb75c"
title: "Source mirror: docs/plans/completed/ui-app-crate-split/PLAN.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/completed/ui-app-crate-split/PLAN.md"
source_sha256: "745bca07bc5fb75c72171a7e6d1867825aa877e100f9995d00350f5eda0028b6"
mirror_body_sha256: "745bca07bc5fb75c72171a7e6d1867825aa877e100f9995d00350f5eda0028b6"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Split db-pro-ui → db-pro-ui (widgets) + db-pro-app (app layer)

## Model

```
db-pro-native        host binary (main.rs, menus, capture, translate)
  ├── db-pro-app     DbProApp + 21 feature modules + app plumbing (was ui/src/app, agent, table, …)
  │     └── db-pro-ui
  └── db-pro-ui      pure widgets + protocol + theme (shadcn layer)
        └── db-pro-core
        db-pro-runtime → db-pro-infrastructure → db-pro-core (unchanged)
```

## Crate contents

| db-pro-ui (`crates/ui`) | db-pro-app (new `crates/app`) |
|---|---|
| `components/` | `app/` (mod.rs + app_modules + app_* + capability_lookup + command_dispatch + test_frame + tests/) |
| `theme/` (+`theme/tokens/`) | `agent/ component_gallery/ connection/ diagram_surface/ explorer/ files/ monitoring/ palette/ query_workbench/ result_grid/ schema/ security/ settings/ shell/ table/ tools/ visual_query_builder/ workspace/` |
| `editor/ diagram/ chart/ query/ dev_tools/` | `runtime_app/` = events.rs, event_router.rs, operation_events.rs, runtime_event_handlers.rs (from `ui/runtime/`) |
| `i18n/` (`rust-i18n` + locales stay) | `policy/` |
| `runtime/` = protocol, *_types, task_bridge only | |

`#[path]` cannot cross crates → everything `app/app_modules.rs` declares must move.
`i18n` stays in ui (owns `rust-i18n` dep + `locales/`); `db-pro-app` gets
`rust-i18n` dep too + `i18n!("../ui/locales")` — shared catalog, per-crate macro
binding. `UiLanguage` re-exported via ui.

## Import rewrite rule

Inside moved files: `use crate::<x>` → `use db_pro_ui::<x>` when `<x>` ∈
{components, tokens, theme, editor, diagram, query, chart, dev_tools,
DbProTheme, UiLanguage, t!, i18n root names: Ui* RequestId TaskBridge
GridProjectionKey Cache GridColumnWindow grid_* cell_text* compare_ui_cells
displayed_row_number filtered_sorted_indexes}.
`crate::app`, `crate::connection`, `crate::result_grid`, `crate::agent`… stay
`crate::` (now db-pro-app internal). `use super::*` chains preserved — app/mod.rs
keeps re-exporting what siblings need via `pub(crate) use db_pro_ui::…`.

`lib.rs` of db-pro-app: `mod app; mod agent? — no: agent/types moved → check`.
NOTE: `agent/mod.rs` (AgentProvider etc.) is declared as root `mod agent` in
ui/lib.rs but ALSO uses app-state types? — verify; if app-internal, its root
mod moves to app crate.

`dev_tools` used by app (mod.rs: `dev_tools` field) → `pub mod dev_tools` in ui;
app references `db_pro_ui::dev_tools::…`.

`policy`: `pub(crate)` items used inside app files → move `policy/` to app
crate (keeps `pub(crate)`), drop from ui/lib.rs.

`result_grid`: whole folder → app. Pure grid fns (cell_text etc.) remain
crate::result_grid::*; bench moves to crates/app/benches.

`GridSelectionLookup` path comment in ui/lib.rs — bench now lives in app
crate; re-export through app's lib.rs.

## Cargo

- `crates/app/Cargo.toml`: deps = everything ui has minus image/sqlparser?
  (keep identical set first, prune later) + `db-pro-ui = { path = "../ui" }`.
- `crates/ui/Cargo.toml`: drop `[[bench]]` (moves to app) — criterion may stay
  for other benches (none → drop dev-dep).
- `crates/native-app`: add `db-pro-app = { path = "../app" }`; main.rs
  `use db_pro_ui::{DbProApp, ComponentGalleryState, respond…, Agent*}` →
  `use db_pro_app::…` for those, keep `db_pro_ui::{Ui*, TaskBridge, DbProTheme…}`.
- root `Cargo.toml` workspace members += `crates/app`.

## Phases

1. Scaffold `crates/app` (Cargo.toml, lib.rs = ui/lib.rs minus ui-module decls,
   `i18n!` macro call, `#[macro_use] extern crate rust_i18n;`).
2. `git mv` all app-side dirs listed above → `crates/app/src/` (incl.
   `runtime_app/` from `ui/runtime/{events,event_router,operation_events,
   runtime_event_handlers}.rs` — keep names, put under `app/src/runtime_app/`
   or keep flat at `src/`? → `src/` root of app: they're plumbing, name them
   `src/runtime/` mirroring ui? They are app-side event plumbing → `crates/app/src/runtime/`).
   Wait — app/src already has `app/` dir (the composition root). Keep
   `runtime/` for the 4 files.
3. ui/lib.rs: delete moved mods + exports (agent, app, dev_tools? keep,
   result_grid, policy, test_frame). Keep `pub mod` for components, diagram,
   editor, i18n, query, tokens, theme, chart, runtime, dev_tools.
4. app/lib.rs: declare `mod app; mod agent; mod result_grid; mod policy;
   mod runtime (the 4 event files); #[cfg(test)] test_frame` + re-exports
   (named, matching old lib.rs surface: DbProApp, ComponentGalleryState,
   Agent*, respond, grid helpers, Ui* NOT — Ui* stay in ui; app re-export
   only if native-app needs — native-app uses db_pro_ui::Ui* directly).
   Wait: does app need a public `mod agent` for `agent/mod.rs` types? That
   file defines AgentProvider etc. and was root-level. If moved to app crate
   root as `mod agent`, `crate::agent::` references inside feature files were
   actually `app::agent`?? No — they were `crate::agent` at ui root. Moving
   whole agent/ dir incl mod.rs into app/src/agent/ keeps `crate::agent`
   valid in app crate. But `use crate::agent::` in ui crate files (none —
   ui dirs don't import agent). OK.
   IMPORTANT: `agent/mod.rs` currently root mod — but the other agent_*.rs
   files are submodules of `app`, not of `agent`. So `crate::agent` refs in
   app files resolve to app's root mod agent = agent/mod.rs (the provider).
   Consistent.
5. Rewrite imports in moved files (rule above) + fix `crate::policy`→`crate::policy`,
   `crate::runtime::MAX_RUNTIME_EVENTS_PER_FRAME`→ stays via app runtime module.
   `runtime/mod.rs` in ui: drop `pub(crate) use task_bridge::MAX_RUNTIME_EVENTS_PER_FRAME`? — events.rs
   (moved) needs it → re-export `pub` or `pub(crate)`? Cross-crate → must be `pub`.
   Add `pub const`/re-export in ui runtime.
6. `#[path]` fixups inside app/app_modules.rs (`../runtime/` → `../runtime/` stays).
   app/mod.rs `include!("app_modules.rs")` unchanged.
   `crate::runtime_event_handlers` refs → `crate::runtime::…`.
7. include_str! fixes: `components/mod.rs` (ui) references `../schema_workbench.rs` etc. — those files now in app crate → include_str! can't cross crates. Fix: those asserts verify components only; drop the entries OR move the test. Choose: trim list to files still in ui (components + theme), move feature-file assertions to app crate tests.
   security_activity_view.rs include_str! `../../../infrastructure/…` → app/src/security → `../../../infrastructure/…` — wait new depth same (crates/app/src/security = same depth as crates/ui/src/security? No: ui/src/security/X.rs → app/src/security/X.rs same depth from crates root → `../../../infrastructure` still correct? path is relative to FILE: crates/app/src/security/../..../infrastructure = crates/app/../infrastructure = crates/infrastructure? `../../../` from app/src/security/file.rs → app/src → app → crates → then infrastructure = crates/infrastructure? ../../../ = crates/app/src/security/../../.. = crates/app/ .. wait: security → src → app → crates. So `../../../infrastructure` = crates/app/../infrastructure? No — ../../../ from `crates/app/src/security/` = `crates/` then `..` — count: security/ .. src/ .. app/ .. crates/ = 3 ups → crates/; then +infrastructure → crates/infrastructure ✓ same as before.
   table/tests include_str ../query_workbench — same folder moves together ✓.
   components/mod.rs — the include_str! list references app-side files — must trim.
8. check-ui-architecture.sh repoint (app file paths → crates/app/src/…).
9. app_tests / AGENT_EVIDENCE refs; `scripts/check-ui-architecture.sh` paths.
10. Gates: cargo +nightly fmt --all, check --workspace, clippy, test -p db-pro-ui + db-pro-app, bash scripts/check-ui-architecture.sh, build native.
11. Commits: scaffold+move → import rewrite → bench/docs → gates green.

## Risks

- `use super::*` in feature files relies on `app/mod.rs` re-exports — ui
  items must be re-exported via `pub(crate) use db_pro_ui::{…}` at top of
  app/mod.rs so `super::*` still resolves them.
- `t!` macro needs `rust_i18n` in scope of every callsite file — previously
  `#[macro_use] extern crate rust_i18n;` at ui root leaked it. In app crate
  add same to app/lib.rs.
- `include!("app_modules.rs")` in app/mod.rs — file moves together, fine.
- `#[cfg(debug_assertions)] mod dev_tools;` — app uses dev_tools field →
  `#[cfg]` gate in app must call `db_pro_ui::dev_tools` — keep dev_tools pub in ui.
- GridSelectionLookup re-export for benches — now app crate re-exports it;
  bench moved to app.
