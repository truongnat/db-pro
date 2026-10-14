# VERIFICATION — ui-rules-compliance

## W1 — §1 component-first

| Gate | Command | Result |
|---|---|---|
| `RULES.md` §6 | `grep -rn 'ui\.checkbox\|ui\.button(\|ui\.radio\|ui\.toggle_value\|egui::ComboBox\|egui::Slider\|ui\.text_edit_' crates/ui/src --include='*.rs' \| grep -v 'components/\|dev_tools/\|ui-rules:'` | PENDING |
| Architecture script | `bash scripts/check-ui-architecture.sh` | PENDING |
| Format | `cargo fmt --all -- --check` | PENDING |
| Check | `cargo check --workspace` | PENDING |
| Clippy | `cargo clippy --workspace --all-targets -- -D warnings` | PENDING |
| Tests | `cargo test --workspace` | PENDING |
| Release build | `cargo build --release --locked -p db-pro-native` | PENDING |
| Extended §1 scan | deterministic scanner over all clusters | PENDING |

## Runtime evidence

| Surface | Sizes | Light | Dark | Notes |
|---|---|---|---|---|
| PENDING | 1280×800 / 1440×900 / 1920×1080 | PENDING | PENDING | `DB_PRO_CAPTURE_TO=... DB_PRO_WINDOW_SIZE=... ./target/debug/db-pro-native` |

## Behaviour-preservation review

PENDING — list any intentional visual delta with before/after screenshots; otherwise state
"no visual delta observed".
