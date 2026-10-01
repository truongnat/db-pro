# Accordion design

`mod.rs` defines the public item model and re-exports `Accordion`. `ui.rs` allocates and paints each header and delegates disclosure presentation to the shared `disclosure` helper. `handler.rs` owns single/multi expansion transitions and disabled activation policy. `config.rs` stores Accordion-owned badge and chevron geometry; shared font and spacing values come from tokens.

The caller owns either `Option<String>` for a single open item or `BTreeSet<String>` for multiple open items. A focused enabled header consumes Enter/Space and sends the same activation signal as a click. The handler applies the transition, then egui's `CollapsingState` drives body visibility and openness; `show_multi` synchronizes that result back into the caller's set. Disabled item toggles are inert and preserve prior set membership.

Each item measures its title and optional badge in the current frame, reserves chevron/badge space, and clips the title to the remaining width. Content renders through a padded disclosure body. This cost is proportional to rendered items and their visible content; there is no item virtualization.

Headers expose `CollapsingHeader` role, enabled state, and expanded state, and draw a focus ring. IDs must be unique within the parent scope. Reduced motion makes open/close and hover changes immediate and renders the body without animated clipping or opacity. Shared theme tokens provide light/dark text and surfaces.
