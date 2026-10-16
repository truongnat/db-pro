# ER Diagram Self-Referencing FK Routing

State: REVIEW.
Issues: truongnat/db-pro#512 and #450 (same self-FK rendering defect).
Branch: `fix/diagram-self-fk-loop`.

## Goal

Render a self-referencing foreign-key relationship outside its table card so it remains visible and does not cross card contents.

## Evidence and failure

`paint_scene_edges` compares source and target node centers to choose opposite anchor sides. For a self-FK both indices are equal, so this picks left and right anchors on the same card; the midpoint bend falls inside the card. Edges are painted before nodes, hiding the middle segment.

## Scope

- Detect `edge.source == edge.target`.
- Put both endpoints on one card side and route the orthogonal bend beyond the card edge, preserving per-edge lanes.
- Keep normal two-node edge routing unchanged.
- Add geometry regression tests for lane and zoom bounds.

## Non-goals

- No graph layout, node interaction, color, typography, or database schema changes.
- No live database behavior change; the relationship data is already represented in `ErGraph`.

## Acceptance

- [x] Self-FK route bends outside its node at supported zoom levels and edge lanes.
- [x] Normal FK midpoint routing is unchanged.
- [x] Diagram tests, formatting, performance scan, and required visual evidence are recorded; interactive performance timing remains unmeasured.
