# UI Core Audit — P0 Resolution (2026-09-23)

Implementation SHA: `b3ab280c` (branch `cursor/ui-core-audit-modal-p0-304e`, PR #307)
Audited baseline: `bae4765e` (see `UI_EXECUTIVE_SUMMARY_2026-09-16.md`)
Scope: the four modal **P0** findings only. P1/P2 remain open (see below); this feature stays in
`docs/plans/active/`.

## What the audit reported

The audit's build could not run a GUI in that CLI environment, so the P0s were source-only and the
"build failed / runtime unverifiable" caveat was really an environment limit, not a code defect. In
this environment `db-pro-native` builds and runs, so the P0s were verified at runtime.

| # | P0 finding | Status |
|---|---|---|
| P0-1 | Modal has no focus trap — `Tab` can move focus out of the dialog | **FIXED** |
| P0-2 | Backdrop does not disable interaction with the window behind | **ALREADY PRESENT**, kept + scoped to topmost |
| P0-3 | `Esc` behaviour inconsistent across dialogs | **FIXED** (topmost dialog closes on Esc) |
| P0-4 | Multiple dialogs overlap with no z-order / single-active rule | **FIXED** (exactly one topmost owner per pass) |

## Change

All app dialogs funnel through one choke point — `components::dialog::Dialog::show_framed_impl`
(`crates/ui/src/components/dialog/modal.rs`) — so the fix is centralized in a new
`crates/ui/src/components/dialog/modal_guard.rs`:

- `register(ctx, id) -> bool` — per-pass modal registry. Returns whether `id` is the single topmost
  dialog (the last one rendered in the previous pass, matching the existing `move_to_top` z-order).
- `trap_focus(ctx, card_layer, anchor_id)` — layer-scoped focus trap. If keyboard focus lands on a
  widget whose `Response.layer_id` is not the dialog's card layer, focus is surrendered and pulled
  back to a hidden focusable anchor rendered as the first widget in the card. Runs before the content
  is drawn so a dialog that explicitly focuses its first field still wins over the anchor.
- `Esc` and backdrop-close now fire only for the topmost dialog.

No call sites changed; every existing dialog/confirmation inherits the behaviour.

## Evidence

Quality gates (this branch):

- `cargo fmt --all -- --check` — PASS
- `cargo clippy -p db-pro-ui --all-targets -- -D warnings` — PASS (0 warnings)
- `cargo build --release -p db-pro-native` — PASS
- `cargo test -p db-pro-ui` — new `modal_guard` unit tests PASS (topmost selection: single / stacked / takeover)
- Full-workspace `cargo clippy --workspace --all-targets` and `cargo test --workspace` — run on this branch (see PR #307).

Native-UI runtime evidence (egui, DISPLAY :1, release binary):

- **Focus trap:** opened the New Connection dialog, focused the first field, pressed `Tab` 16×. Focus
  cycled only through in-dialog controls and, after the last control (Save Connection), wrapped back
  to the top (PostgreSQL engine button). It never landed on the sidebar, `New query`, Welcome tiles,
  or window chrome. Verified independently via video review.
- **Esc:** closed the dialog and returned to the Welcome page.
- **Backdrop:** clicking the dimmed area outside the card closed the dialog and the click was absorbed
  (no background control activated).

Recording: `dbpro_modal_focus_trap_esc_backdrop.mp4` (attached to the PR / agent summary).

## Provider matrix

| Provider | Supported | Automated | Runtime | Capability gate |
|---|---|---|---|---|
| PostgreSQL | n/a | n/a | n/a | Pure native-UI change; no database code touched |
| SQLite | n/a | n/a | n/a | Pure native-UI change; no database code touched |

## Remaining findings (out of scope for this PR)

Still open from the audit and keeping this feature in `active`:

- P1: accessibility tree for grid/tree/dialog; responsive dialog width; visible focus indicator
  styling; overlay fade-in/out animation.
- P2: layout-token inconsistencies (row height, input padding, spacing tokens, tree indent, resize
  hit target, badge alignment); overlay/layout perf.

## Lifecycle

- State: remains `IMPLEMENTING`/`RUNTIME_VERIFY` for the broader audit; **P0 = 0** after this change.
- The feature is NOT `COMPLETED` (P1/P2 remain), so its plan directory stays under
  `docs/plans/active/ui-core-audit/`.

## Tổng kết bằng tiếng Việt

Đã sửa xong 4 lỗi **P0** về modal của `ui-core-audit`: thêm **focus trap** (Tab không thể thoát khỏi
dialog, chạm control cuối thì quay vòng về đầu), **Esc** và **click nền (backdrop)** chỉ tác động lên
dialog trên cùng, và nền chặn tương tác với cửa sổ phía sau. Sửa tập trung tại một điểm chung
`components/dialog/modal_guard.rs` nên mọi dialog đều được áp dụng. Đã kiểm chứng runtime trên egui
(DISPLAY :1) bằng video, cùng fmt/clippy/test xanh. P1/P2 vẫn còn nên feature vẫn ở `active`.
