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

/// Keeps keyboard focus inside `card_layer`, pulling it back to `anchor_id`
/// whenever it escapes to a widget in another layer (a `Tab`-out to the window
/// behind the dialog).
///
/// Runs before the dialog content is drawn so that any explicit `request_focus`
/// inside the content (e.g. focusing the first text field) issued later in the
/// same pass wins over the anchor fallback.
pub(super) fn trap_focus(ctx: &Context, card_layer: LayerId, anchor_id: Id) {
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
mod tests {
    use super::*;

    #[test]
    fn single_modal_is_always_topmost() {
        let ctx = egui::Context::default();
        let solo = Id::new("solo");
        let _ = ctx.run(Default::default(), |ctx| {
            assert!(register(ctx, solo));
        });
        let mut topmost = false;
        let _ = ctx.run(Default::default(), |ctx| {
            topmost = register(ctx, solo);
        });
        assert!(topmost, "a lone dialog must stay topmost across passes");
    }

    #[test]
    fn last_registered_modal_is_topmost() {
        let ctx = egui::Context::default();
        let lower = Id::new("lower");
        let upper = Id::new("upper");

        // Warm-up pass establishes the reference stack [lower, upper].
        let _ = ctx.run(Default::default(), |ctx| {
            register(ctx, lower);
            register(ctx, upper);
        });

        let mut lower_topmost = true;
        let mut upper_topmost = false;
        let _ = ctx.run(Default::default(), |ctx| {
            lower_topmost = register(ctx, lower);
            upper_topmost = register(ctx, upper);
        });

        assert!(!lower_topmost, "the lower dialog must not own Esc/backdrop");
        assert!(upper_topmost, "the last-rendered dialog must be topmost");
    }

    #[test]
    fn stacked_modal_becomes_topmost_on_next_pass() {
        let ctx = egui::Context::default();
        let existing = Id::new("existing");
        let fresh = Id::new("fresh");

        // Pass 1: only `existing` is open.
        let _ = ctx.run(Default::default(), |ctx| {
            register(ctx, existing);
        });

        // Pass 2: `fresh` opens on top; the previous stack is still [existing], so
        // `existing` owns this pass and exactly one dialog is topmost.
        let _ = ctx.run(Default::default(), |ctx| {
            assert!(register(ctx, existing), "existing owns the pass it stacked on");
            assert!(!register(ctx, fresh), "a freshly stacked dialog waits one pass");
        });

        // Pass 3: previous stack is now [existing, fresh]; the newer dialog takes over.
        let mut fresh_topmost = false;
        let mut existing_topmost = true;
        let _ = ctx.run(Default::default(), |ctx| {
            existing_topmost = register(ctx, existing);
            fresh_topmost = register(ctx, fresh);
        });

        assert!(
            fresh_topmost,
            "the newer dialog becomes topmost once it is last rendered"
        );
        assert!(!existing_topmost, "only one dialog is topmost at a time");
    }
}
