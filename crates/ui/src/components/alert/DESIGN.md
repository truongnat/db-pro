# Alert design

`mod.rs` exposes `Alert`, `AlertDialog`, `AlertDialogAction`, and `AlertVariant`. `ui.rs` owns the alert row and confirmation dialog layout; `handler.rs` maps variants to semantic theme colors/icons, sizes the dialog against the viewport, and resolves close actions; `config.rs` holds alert-local dimensions.

## Event flow

`Alert::show` paints a themed frame and returns the close button response only when the alert is dismissable. The caller decides when to remove the alert. `AlertDialog::show` first checks caller-owned `open`, then consumes Escape, draws a clickable dim backdrop, and paints the centered action dialog. Cancel paths return `Cancel`; the primary action returns `Confirm`. The `destructive` option prevents backdrop dismissal and styles the primary action as dangerous, while Escape and the explicit Cancel action remain cancellation paths.

The confirmation dialog is independent from the shared `Dialog` stack/focus guard. It requests initial focus on Cancel only when egui has no focused widget; it does not cycle focus or restore focus after close. Multiple simultaneous alert dialogs need unique `.id_salt(...)` values.

## Rendering and accessibility

An alert costs one frame plus its labels and optional close button. The dialog draws backdrop and card only while open. Descriptions and titles wrap within the available viewport width. Dismiss buttons expose a named button response; dialog actions use the shared Button semantics. Keep text concise enough for a narrow window and provide a specific title/description for confirmation. The component does not animate its modal transition or expose busy state; callers should prevent repeated confirmation while an action is pending.
