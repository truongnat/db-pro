//! Shared modal behaviour for [`super::modal::Dialog`].
//!
//! egui 0.29 has no built-in modal, so a bare inset dialog leaks in two ways the
//! `ui-core-audit` flagged as P0:
//!
//! * **Focus trap (P0-1):** `Tab` cycles through every widget that registered
//!   focus interest this pass. Background widgets are drawn before the dialog, so
//!   `Tab` from the last dialog control wraps straight into the window behind it.
//! * **Topmost routing (P0-3/P0-4):** every open dialog independently reacted to
//!   `Esc` and backdrop clicks, so a single `Esc` closed *all* stacked dialogs and
//!   there was no notion of which overlay was actually on top.
//!
//! This module centralises both: a per-pass registry names the topmost dialog, and
//! a layer-scoped focus trap keeps keyboard focus inside the active dialog.

use egui::{Context, Id, LayerId};

/// Per-pass registry of the modal dialogs currently painting, in render order.
#[derive(Clone, Default)]
struct ModalStack {
    pass: u64,
    ids: Vec<Id>,
}

fn stack_key() -> Id {
    Id::new("dbpro_modal_stack")
}

fn prev_key() -> Id {
    Id::new("dbpro_modal_stack_prev")
}

/// Records `id` as an active modal for this pass and reports whether it is the
/// topmost one.
///
/// "Topmost" is the last dialog rendered in the previous completed pass — which is
/// also the one raised last by `move_to_top`, so z-order and input routing always
/// agree and exactly one dialog owns Esc/backdrop/focus. When no dialog was open in
/// the previous pass, the first one to register this pass is topmost, so a lone
/// dialog is interactive on its very first frame. A dialog stacked on top of an
/// existing one becomes topmost on the next pass (once it is the last rendered).
pub(super) fn register(ctx: &Context, id: Id) -> bool {
    let pass = ctx.cumulative_pass_nr();
    let prev = ctx.data_mut(|d| {
        let mut cur: ModalStack = d.get_temp(stack_key()).unwrap_or_default();
        if cur.pass != pass {
            // A new pass started: the stack accumulated during the previous pass
            // becomes the reference used to decide who is topmost this pass.
            d.insert_temp(prev_key(), cur.ids.clone());
            cur = ModalStack { pass, ids: Vec::new() };
        }
        if !cur.ids.contains(&id) {
            cur.ids.push(id);
        }
        d.insert_temp(stack_key(), cur);
        d.get_temp::<Vec<Id>>(prev_key()).unwrap_or_default()
    });

    match prev.last() {
        Some(top) => *top == id,
        None => true,
    }
}
/// Reports whether any modal dialog was painting last pass or has already
/// registered this pass. `handle_shortcuts` runs in `prepare_frame` — before
/// dialogs draw — so it must consult this instead of a hand-kept overlay
/// blocklist, otherwise global chords and Esc leak through open modals.
pub fn any_open(ctx: &Context) -> bool {
    ctx.data(|d| {
        let cur: ModalStack = d.get_temp(stack_key()).unwrap_or_default();
        let pass = ctx.cumulative_pass_nr();
        if cur.pass == pass && !cur.ids.is_empty() {
            return true;
        }
        !d.get_temp::<Vec<Id>>(prev_key()).unwrap_or_default().is_empty()
    })
}

/// Keeps keyboard focus inside `card_layer`, pulling it back to `anchor_id`
/// whenever it escapes to a widget in another layer (a `Tab`-out to the window
/// behind the dialog).
///
/// Runs before the dialog content is drawn so that any explicit `request_focus`
/// inside the content (e.g. focusing the first text field) issued later in the
/// same pass wins over the anchor fallback.
pub(super) fn trap_focus(ctx: &Context, card_layer: LayerId, anchor_id: Id) {
    // Let egui apply Tab or Shift+Tab to its focus chain; the next pass restores
    // focus if traversal landed outside the topmost dialog.
    if ctx.input(|i| i.key_pressed(egui::Key::Tab)) {
        return;
    }

    let focused = ctx.memory(|m| m.focused());
    let inside = match focused {
        Some(fid) if fid == anchor_id => true,
        Some(fid) => ctx
            .read_response(fid)
            .map(|resp| resp.layer_id == card_layer)
            .unwrap_or(false),
        None => false,
    };

    if !inside {
        if let Some(fid) = focused {
            ctx.memory_mut(|m| m.surrender_focus(fid));
        }
        ctx.memory_mut(|m| m.request_focus(anchor_id));
    }
}

#[cfg(test)]
#[path = "tests/modal_guard_tests.rs"]
mod tests;
