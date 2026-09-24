# Welcome and query editor polish

State: RUNTIME_VERIFY
Branch: `cursor/welcome-query-editor-polish-e577`

## Goal

The first workspace tab (Welcome) and the query editor read as a finished database IDE: a composed start page, and query actions that sit on the editor instead of a 24px status row.

## Evidence / baseline

- Baseline: `d52e752fd1f7becdd5abbcdf6a12662959c08a72`
- Welcome content started 32px from the top of a large empty canvas. Start actions had no surface; Connections sat in a card.
- With an empty catalog the Explorer still painted "Filter objects…" above "No connections".
- Query Run lived only in the status bar. Explain and Format were inside the overflow menu. The current-line wash and inactive line numbers were barely visible on the dark buffer.

## Scope

- Welcome vertical inset and matched Start / Connections cards
- Hide the Explorer filter when the catalog is empty
- Query toolbar: Run, Explain, Format, with the connection chip
- Stronger current-line fill and readable gutter numbers

## Non-goals

- SQL execution, completion, or provider behavior
- Activity-rail icon set
- Pixel-perfect Zed clone

## Provider matrix

| Provider | Supported | Required proof |
|---|---|---|
| PostgreSQL | unchanged | UI chrome is provider-neutral |
| SQLite | unchanged | UI chrome is provider-neutral |

## Acceptance criteria

- [x] Welcome start block sits in the upper portion, with Start and Connections as paired cards
- [x] Empty Explorer does not show the object filter
- [x] Run, Explain, and Format are on the query toolbar
- [x] Status bar keeps cursor, driver, schema, and transaction metadata
- [x] Capture evidence at 1280×800, 1440×900, and 1920×1080
