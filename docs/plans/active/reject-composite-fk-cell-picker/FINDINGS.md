# Findings

## Confirmed — composite FK values are collapsed into one cell

- Issues: #496 and #447 describe the same root cause in the grid picker.
- A two-column FK lookup returns a row tuple and the picker joins it as `"v1, v2"`, then stages that text into whichever one source column is being edited.
- Composite referenced-row navigation already maps each source column to its corresponding destination column; this path must remain supported.
- Fix: expose a per-cell picker only for exactly one source and one destination column, without narrowing the general FK lookup used for row navigation.
