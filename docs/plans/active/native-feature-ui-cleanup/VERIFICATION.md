# Verification

Baseline HEAD: `95de1c9d172e565e5378b9f1cc72fdcadd69b2f8`. Candidate is uncommitted on main.

Skill searches executed successfully; output fit/rejection recorded in FINDINGS.md.
Implementation-specific checks and runtime captures: pending, not claimed as passing.

Known earlier-session gate results (not this candidate's final gates): full-workspace tests failed at native SQLite sample seeding; strict Clippy failed in untouched core/UI paths. Exact-height captures are capped to 838 logical pixels at larger widths. See ../quick-open-action-visuals/VERIFICATION.md for commands, source snapshots and original artifacts.

## 2026-10-19 Transfers sidebar clip

Source SHA remains `95de1c9d172e565e5378b9f1cc72fdcadd69b2f8`; the layout fix is uncommitted.

Evidence-backed defect: at 1280×800 the Transfers sidebar cut copy labels and harness buttons (`Target`/`Format` shown as `Tar`/`For`, `Run then cancel` shown as `Ru`). The 260px CentralPanel test did not catch it. A wider-than-clip max rect was not measured, so that is not claimed as the cause.

Fix, presentation only: constrain the sidebar scroll body and the Transfers draw root to `min(available_width, clip_rect.width)`; request each copy-field group's real width; wrap the synthetic input and action rows.

Checked:

- `cargo test -p db-pro-app transfer_choice_groups_and_masking_actions_reflow_within_sidebar` — passed
- `cargo test -p db-pro-app session_actions_stay_inside_the_sidebar` — passed
- `cargo build --locked -p db-pro-native --features capture` — built
- Native captures, both themes. Requested 1280×800 rendered 2560×1600; 1440×900 rendered 2880×1676; 1920×1080 rendered 3840×1676. Height stays clamped at 838 logical pixels, not the requested 900/1080. Files: `evidence/transfers-{light,dark}-{1280x800,1440x900,1920x1080}.png`.
- Inspected at 1280, both themes: Source, Target, Tables, Format, Conflict, and Run then cancel stay inside the sidebar. Inspected at 1440 dark and 1920 light/dark: those fields plus Suggest cols, Preview sample, Demo masked CSV export, Run synthetic harness, and CSV export harness also stay inside. JSONL and later actions sit below the clamped viewport and scroll.
- Not inspected at 1440/1920: the synthetic seed card. `capture_focus_once` scrolls the sidebar to masking, so that card is above the viewport. Do not treat those captures as a pass for synthetic inputs/actions.

Not claimed: shared quiet-frame sweep, empty/loading/error states, workspace `cargo test`/`clippy`, independent review. Transfers width clips are inspected at the three requested widths; height remains clamped at 838 logical pixels.

## 2026-10-19 recheck gates

- `cargo test -p db-pro-app welcome_cards_fit_both_sides_of_the_grid_breakpoint` — passed
- `bash scripts/check-ui-architecture.sh` — PASS
- Welcome, Schema Object, Security, and Settings General captures at requested 1280×800 (rendered 2560×1600) inspected. No new layout change. See FINDINGS.md recheck.

## 2026-10-19 monitor header overlap

`evidence/monitoring-dark-1280x800.png` before the wrap showed `Connected` painted under `Auto-refresh`. After reserving the checkbox width and dropping the right-to-left row, both themes at requested 1280×800 (rendered 2560×1600) show `Disconnected` and `Auto-refresh` on separate lines with intact labels: `evidence/monitoring-dark-1280x800.png`, `evidence/monitoring-light-1280x800.png`. The fixture was disconnected, so the Refresh button was not on screen. `connected_status_does_not_share_a_row` passed. `cargo build --release --locked -p db-pro-native` passed again after this header edit.

Limitation: capture status bar still reports `database "fullstack_starter" does not exist`. That is the debug-only developer preset, not a layout defect. This change does not create the database or retarget the preset. Local Postgres on this host has `postgres`, `dbpro_fixture`, `enterprise_db`, and `recordings`.
