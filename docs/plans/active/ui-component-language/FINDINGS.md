# Findings

## P2 — Primary native controls still bypassed canonical components

At baseline `8da3c108ce237811eef6931d6c081cef9bf85447`, the value inspector
used raw `ui.selectable_label` and `ui.button` controls, while the Explorer
toolbar hand-built a search frame/TextEdit and a raw refresh context button.
The repository already provides `SegmentedTabs`, `Button`, `SearchInput`, and
`ctx_menu_item` for these patterns.

This is a contained visual-consistency gap, not a data or provider defect. The
minimal fix is to reuse those primitives on the two named surfaces and leave
the broader #288 inventory for later focused slices.
