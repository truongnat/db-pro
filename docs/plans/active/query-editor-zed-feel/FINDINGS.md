# Findings

Baseline for this implementation pass: `f1a484059dbaf4915c96089715a03d9291507f28`.

## Resolved earlier — P1 editor content size unused

Long queries were clipped because computed content dimensions were not driving editor scroll state. Manual two-axis scrolling and keep-caret-visible behavior are now present; runtime evidence remains pending.

## Resolved earlier — P2 static caret and heavy chrome

The editor now has an idle blink cycle, recent-input solid period, softer gutter/current-line treatment and editor-focused spacing.

## P1 — stale completion ranges can mutate the wrong buffer segment

A completion item stores a byte replacement range. Before this pass, that range had no source document version and the popup could survive a subsequent edit. Accepting the item could therefore replace a valid but unrelated range, presenting as text jumping or being inserted unpredictably.

Decision: bind every completion session to `TextBuffer::version()` and fail closed if the current version differs or the range is invalid/not on UTF-8 boundaries.

## P1 — persisted prediction mode can keep automatic AI scheduling active

Changing only the default prediction mode to `Off` does not affect users with an older persisted `Subtle` value. The query panel still scheduled prediction after ordinary edits/caret movement.

Decision: automatic scheduling is removed. Prediction requires an explicit editor action and a non-`Off` mode.

## P2 — completion UI exposes labels but not useful explanation

Completion items already carry `detail` and `documentation`, but the popup only used truncated label/detail tooltips. Users could not understand why a suggestion was offered.

Decision: selected suggestions get a persistent explanation area; hovering a row shows the same structured explanation.

## Remaining P2

- Continuous completion filtering currently closes stale sessions rather than recomputing them. This is intentionally conservative for typing correctness and should be added as a version-safe session update.
- General SQL symbol hover and function signature help are not yet implemented.
