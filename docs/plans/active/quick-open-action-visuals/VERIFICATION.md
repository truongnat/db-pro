# Verification

Historical record — 2026-10-02 implementation source SHA: `78d914887642dd400c7e2048db9a24d0403619c8`.
The automated results, captures, and lifecycle notes below belong only to that earlier implementation.
Current candidate results (including failed workspace/Clippy gates) are in the 2026-10-19 section.

## Automated

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | PASS |
| `cargo check --workspace` | PASS |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS |
| `cargo test --workspace` | PASS — 1606 passed, 0 failed, 41 ignored |
| `cargo test -p db-pro-ui selected_scope_label_uses_the_theme_selection_foreground -- --nocapture` | PASS — 1 passed for light and dark themes; before the fix it failed with `#1A1C1F` vs `#FFFFFF` |
| `cargo test -p db-pro-ui action_variants_stay_borderless_through_hover_loading_and_disabled_states -- --nocapture` | PASS — 1 passed |
| `cargo clippy -p db-pro-native --features capture --all-targets -- -D warnings` | PASS |
| `cargo build --release --locked -p db-pro-native` | PASS |
| `git diff --check` | PASS |

## Native captures

The capture-enabled native app used the seeded Sample E-Commerce SQLite connection. Captures are in `evidence/`:

- `quick-open-dark-1280x800.png`: dark Quick Open, selected Schema scope, populated results; 1280×800 logical at 2× pixels.
- `quick-open-empty-light-1280x800.png`: light Quick Open empty state; 1280×800 logical at 2× pixels.
- `action-buttons-dark-1280x800.png`: dark Button Gallery with variants, loading, and disabled states; 1280×800 logical at 2× pixels.
- `quick-open-light-1440x838-clamped.png`: light Quick Open with populated results; the requested 1440×900 viewport was clamped to 1440×838 logical.
- `quick-open-dark-1920x838-clamped.png`: dark Quick Open with populated results; the requested 1920×1080 viewport was clamped to 1920×838 logical.

The host display cannot provide the full requested heights at 1440 and 1920 logical widths. Quick Open has no loading or error state; its empty state is captured. Capture runs emitted pre-existing egui warnings about replacement glyphs (`◻`/`?`).

## Lifecycle and limitations

At historical SHA `78d914887642dd400c7e2048db9a24d0403619c8`, source and automated gates were recorded as passing. Its native visual evidence confirms selected text, action strokes, loading/disabled Buttons, and empty Quick Open at 1280×800. The full-height 1440×900 and 1920×1080 visual gate remained pending, so the plan stayed `RUNTIME_VERIFY`. No cache or rasterization performance metrics were collected; that change did not alter font or renderer paths.

## 2026-10-19 owner-reported regressions

Baseline HEAD: `95de1c9d172e565e5378b9f1cc72fdcadd69b2f8`.
Candidate: uncommitted changes on main; exact file SHA-256 values are recorded in
`evidence/2026-10-19/source-manifest.json`. No commit or PR was created.

### Commands actually executed

| Command | Observed result |
|---|---|
| `cargo test -p db-pro-ui search_keeps_results_visible_in_a_tall_container -- --nocapture` | Before fix: 0 passed / 1 failed; first result y=796. After: 1 passed / 0 failed / 0 ignored |
| `cargo test -p db-pro-app toolbar_and_settings_switch_rendered_theme_and_persist_the_same_choice -- --nocapture` | 1 passed / 0 failed / 0 ignored; actual pointer events target accessible toolbar/radio bounds, paint selected theme, then save/reload |
| `cargo test -p db-pro-ui applying_theme_selects_matching_egui_style_across_frames -- --nocapture` | 1 passed / 0 failed / 0 ignored |
| `cargo test -p db-pro-app empty_palette_keeps_keyboard_help_inside_the_dialog -- --nocapture` | Before footer fix: 0 passed / 1 failed; after: 1 passed / 0 failed / 0 ignored |
| `cargo test -p db-pro-app -p db-pro-ui` | Final source: 1299 passed (599 app + 700 UI), 0 failed, 0 ignored; exit 0 |
| `cargo test -p db-pro-ui -p db-pro-runtime` | 742 passed (700 UI + 42 runtime), 0 failed, 0 ignored; exit 0; before palette-only follow-up |
| `cargo test --workspace` | Failed, exit 101: 1366 passed / 1 failed / 55 ignored before abort; native `sample_seed_creates_and_verifies_only_marked_fixture` cannot open its SQLite fixture; UI/runtime suites were subsequently exercised separately |
| `cargo +nightly fmt --all -- --check` | Final source PASS, exit 0 |
| `cargo check --workspace` | Final source PASS, exit 0; warning: unused DbConnector import in untouched SQL Server introspection |
| `bash scripts/check-ui-architecture.sh` | Final source PASS, exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | Failed, exit 101, 65 errors in core (large transaction error + async-trait double-must-use); not modified here |
| `cargo clippy -p db-pro-ui -p db-pro-app --all-targets --no-deps -- -D warnings` | Failed, exit 101, untouched UI diagnostics: formatting borrow, manual div_ceil, chunks_exact and test borrows; not suppressed |
| `cargo build --locked -p db-pro-native --features capture` | Final source PASS, exit 0 |
| `cargo build --release --locked -p db-pro-native` | Final source PASS, exit 0 |
| `python3 tools/cc-scan.py <14 changed Rust paths> --fail-on error` | Exit 0, 0 errors / 6 warnings; before final palette/footer follow-up |
| `bash .skills/perf-audit/scripts/perf-scan.sh` | WARN, exit 2: 3 checks passed, 1 warning (Clippy), 4 measurements not executed; release binary 32.1MB at that run; no performance improvement claimed |
| `orca skills get computer-use --json` | Unavailable: `command not found: orca`, exit 127; no native OS-pointer interaction claim |

Workspace/Clippy failures are outside this patch's changed behavior. A baseline checkout
was not executed; these results are not a fully measured baseline-versus-candidate comparison.

### Native visual evidence

Final captures: `evidence/2026-10-19/final/`, with per-run sizes/exits in `captures.json`.
36 runs, 36 PNGs, all exit 0. Both light/dark were inspected as original key images and
six contact sheets covering every final capture:

- shell query/editor, including empty saved-query state;
- Agent panel empty state and subtle shell boundary;
- Quick Open focused empty query, selected Schema scope;
- Quick Open typed no-match state;
- connection test pending (fixture, not a live-provider success claim);
- connection validation error (fixture).

| Requested logical size | Actual framebuffer pixels | Effective logical size |
|---|---|---|
| 1280×800 | 2560×1600 | 1280×800 |
| 1440×900 | 2880×1676 | 1440×838 |
| 1920×1080 | 3840×1676 | 1920×838 |

The 900/1080-height filenames encode **requests**, not achieved heights. No image was
stretched/padded to claim a full-height viewport. Quick Open search is now one compact row,
the no-match message is visible, and keyboard help stays in the footer. Light shell lines
are subtle rather than black; dark shell boundaries use the corresponding semantic tones.
Focused search and selected scope states are visible. Hover/press is exercised in the
headless pointer regression; native OS hover and splitter dragging remain unverified.

Example invocation (change the matching mode/theme variables for other rows):

```bash
HOME=/tmp/db-pro-capture-home-20261019 \
DB_PRO_DATA_DIR=/tmp/db-pro-quick-open-theme-20261019 \
DB_PRO_DISABLE_KEYRING=1 DB_PRO_CAPTURE_SETTLE_FRAMES=20 \
DB_PRO_CAPTURE_QUICK_OPEN=1 DB_PRO_CAPTURE_QUICK_OPEN_LIGHT=1 \
DB_PRO_WINDOW_SIZE=1280x800 \
DB_PRO_CAPTURE_TO="$PWD/docs/plans/active/quick-open-action-visuals/evidence/2026-10-19/final/quick-open-light-1280x800.png" \
./target/debug/db-pro-native
```

Agent light evidence uses `DB_PRO_CAPTURE_AGENT=1 DB_PRO_CAPTURE_AGENT_LIGHT=1`.
Normal launches are unchanged by capture-only environment variables.

Initial non-final runs: two PNG-producing processes exited with AppKit NSTouchBar
NSRangeException; six 1440×900 runs timed out, followed by a bounded retry strategy and
the final successful matrix. Their earlier logs/metadata remain one directory above `final/`.
The app still restored a saved PostgreSQL connection despite attempted HOME separation;
its database-not-found status is background context, not a claim about provider correctness.
No secrets were exposed by the masked password fields.

### Disposition

Plan remains `RUNTIME_VERIFY`: requested full heights, independent review and native pointer
resize/hover evidence are not complete. PostgreSQL and SQLite are independently n/a for
the changed presentation logic. No SQL generation, execution, or connection behavior was changed.

## Border follow-up after owner rejection — 2026-10-19

Same baseline `95de1c9d172e565e5378b9f1cc72fdcadd69b2f8` plus uncommitted follow-up.
The earlier fix deliberately retained subtle frame strokes; the owner rejected their
remaining visible boundaries. Topbar/statusbar/output frame flags are now borderless;
Agent and activity-rail callers explicitly suppress their outer frame strokes.
Global input and card styles are unchanged.

- `cargo test -p db-pro-app shell_panel_boundaries_stay_borderless_in_both_themes -- --nocapture`: 1 passed / 0 failed / 0 ignored, exit 0. Checks actual painted large boundary rectangles over repeated frames with Agent and output open/closed.
- `cargo build --locked -p db-pro-native --features capture`: exit 0.
- `evidence/2026-10-19/border-followup/`: 12 PNGs, all exit 0, light/dark shell and Agent at the three requested widths. Actual logical sizes remain 1280×800, 1440×838, 1920×838. Original 1280 Agent images and a contact sheet containing all twelve were inspected.
- Exact source snapshot: that directory's `source-manifest.json`.
- Full package/release verification follows in the separate native feature-cleanup pass; no claim that previous run counts cover this later patch.

This corrects the earlier insufficient border fix; it is not described as a theme regression.
