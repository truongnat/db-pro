# Findings

## F-1 — Disclosure headers had divergent visual systems

**Severity:** P2 (UI consistency)

Collapsible and Accordion each painted their own hover fill, colors, geometry,
badge spacing, and body margins. Accordion also added a persistent divider while
Collapsible did not. The same interaction therefore looked like two unrelated
components.

**Decision:** Move header surface, state colors, focus ring, body margin, and
chevron treatment into `components/disclosure.rs` and consume it from both
components.

## F-2 — Body animation faded but did not animate layout height

**Severity:** P2 (interaction quality)

The old implementation rendered the full body layout as soon as openness passed
the threshold, then only changed opacity. Closing could leave reserved blank
space while the body faded.

**Decision:** Use egui `CollapsingState::show_body_unindented` for clipped height
and persist the measured body height; apply the shared opacity and inset inside
that clipped body.

## Self-review

- P0: 0
- P1: 0
- P2: 2 addressed in this slice.
- Provider impact: none.
