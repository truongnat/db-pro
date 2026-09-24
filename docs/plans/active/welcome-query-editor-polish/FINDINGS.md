# Findings

Baseline: `d52e752fd1f7becdd5abbcdf6a12662959c08a72`.

## P2 — Welcome is a short block on an empty canvas

`welcome_surface_view` inserted a fixed 32px top gap and left Start as bare rows beside a Connections card. On a 1440×900 capture the rest of the central panel was empty surface.

Decision: scale the top inset with spare height (clamped 24–96) and put Start in the same card frame as Connections.

## P2 — Empty Explorer still shows a filter

`ExplorerSurfaceContext::draw` always painted the object filter, including when `catalog` is empty. The first screen then showed "Filter objects…" above "No connections".

Decision: draw the toolbar only when the catalog has a connection. The existing border test uses one connection and still paints the field.

## P2 — Query Run is a status-bar control

`draw_status_bar` was the only Run/Stop control. Explain and Format lived in the overflow menu. A query editor without a toolbar reads as unfinished.

Decision: draw Run, Explain, and Format on the context strip. The status bar keeps Ln/Col, driver, schema, transaction, parameters, and diagnostics.

## P2 — Current line and gutter numbers are too faint

Dark current-line fill used alpha 10. Inactive line numbers were rgb(90,90,90) on the gutter.

Decision: raise the wash to alpha 22 (dark) / 16 (light) and inactive dark numbers to rgb(120,120,120). Editor top padding is 12px. Line-number size tracks the editor font.
