# Findings

## Confirmed P2 — self-referencing FK route crosses its own card

- Evidence: `paint_scene_edges` used horizontal center comparison to select opposite endpoint sides. A self-edge has identical source/target centers, selecting left/right endpoints and a bend at their shared x coordinate within the card.
- Impact: self-referencing relationships (e.g. parent/manager columns) become obscured by the node painted above the edge.
- Scope: route the self-edge on a single side and offset its bend beyond the card, retaining deterministic per-edge lanes.
- Confidence: high; both issue #512 and #450 describe the same path.
