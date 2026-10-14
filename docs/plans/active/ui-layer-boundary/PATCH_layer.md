# Changes

## 1. `scripts/check-ui-architecture.sh` — append widget-layer boundary check

Insert immediately BEFORE `echo "UI architecture boundary: PASS"` (L436; after the `raw_widgets` block ending L433, replacing the blank line L435):

```bash
# Widget layer boundary: components/, theme/ (incl. theme/tokens/), editor/ and
# diagram/ are pure widgets — they may depend on each other and on
# crate::{components,tokens,theme,i18n,policy,runtime,editor,diagram,result_grid,
# chart} plus crate-root items such as crate::DbProTheme, but must never import
# feature modules. `tokens/` is not a top-level dir (it lives inside theme/), so
# scanning theme/ covers it. Denylist covers three use-path shapes rustfmt can
# emit under imports_granularity="Crate": `use crate::<feat>(::|;)`, first item
# of `use crate::{<feat>::…`, and a feature path nested anywhere in a merged
# brace group (`use crate::{components::X, <feat>::Y}` — the inner `[^;{]*?`
# prefix forbids an intervening `{`, so `tokens::{component::table::…}` and
# `components::table::…` are not flagged). `dev_tools` and `query` are
# intentionally not listed (dev tooling / shared query widgets are permitted).
widget_layer_leaks=$(rg --pcre2 -U -n \
  'use crate::(app|agent|query_workbench|table|schema|explorer|files|shell|tools|settings|security|monitoring|palette|workspace|visual_query_builder|diagram_surface|component_gallery|connection)\b|use crate::\{\s*(app|agent|query_workbench|table|schema|explorer|files|shell|tools|settings|security|monitoring|palette|workspace|visual_query_builder|diagram_surface|component_gallery|connection)::|use crate::\{[^;{]*?[{,]\s*(app|agent|query_workbench|table|schema|explorer|files|shell|tools|settings|security|monitoring|palette|workspace|visual_query_builder|diagram_surface|component_gallery|connection)::' \
  "$repo_root/crates/ui/src/components" \
  "$repo_root/crates/ui/src/theme" \
  "$repo_root/crates/ui/src/editor" \
  "$repo_root/crates/ui/src/diagram" \
  --glob '*.rs' \
  | rg -v 'ui-rules:' \
  | rg -v '#\[cfg\(test\)\]' || true)
if [[ -n "$widget_layer_leaks" ]]; then
  echo "UI architecture check failed: widget layer (components/theme/editor/diagram/tokens) must not depend on feature modules." >&2
  echo "$widget_layer_leaks" >&2
  exit 1
fi
```

## 2. `AGENTS.md` — Rust quality gates: fmt on nightly (L211–218)

Old (verbatim):

````markdown
## Rust quality gates

```bash
cargo fmt --all -- --check
cargo check --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```
````

New (verbatim):

````markdown
## Rust quality gates

```bash
cargo +nightly fmt --all -- --check
cargo check --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```
````

# Verification

Ran against the current tree (2026-10-14):

- Deviation from the hint: `dev_tools` and `query` are not in the denylist because they are not in the PLAN/assignment feature list — `dev_tools` is a tooling folder and `query` is a shared query-widget folder; PLAN.md §5 only names the 18 modules used here.
- Denylist `rg --pcre2 -U` over the four widget dirs, piped through the `ui-rules:` and `#[cfg(test)]` exclusions → **empty** (no current violations).
- Synthetic negative tests via stdin — all caught: `use crate::app::DbProApp;`, `use crate::app;`, `use crate::{app::Foo, components::Bar};`, multiline `use crate::{components::Bar, app::Foo, tokens::X};`, `use crate::agent::State;`.
- Synthetic allowlist tests — all pass clean: `use crate::{components::{self, table::Cell}, DbProTheme};`, `use crate::{tokens::{component::table::X}, DbProTheme};`, `use crate::components::table::Cell;`, `self.app.foo();` (field access), `use crate::{dev_tools,query,result_grid,chart,policy,i18n}::…`.
- NOTE: the naive single-alternation pattern from the task hint was tried first and false-positived on `tokens::{component::table::TABLE_ROW_HEIGHT_COMPACT, …}` in `components/select/ui/option.rs` — the three-alternation pattern above fixes that while still catching brace-merged violations (needed because rustfmt.toml sets `imports_granularity = "Crate"`).
- Deviation from the hint: `dev_tools` and `query` are excluded from the denylist — sibling slice contract lists them as allowed feature-ish deps? No — they're absent from the PLAN/assignment feature list; `dev_tools` is a tooling folder and `query` is a shared query-widget folder, and PLAN.md §5 only names the 18 modules used here.

Parent should run after integrating all slices: `bash scripts/check-ui-architecture.sh` → expect `UI architecture boundary: PASS`; then `git apply`-free smoke: temporarily add `use crate::app::DbProApp;` under `src/components/` → expect failure message; remove and confirm PASS.
