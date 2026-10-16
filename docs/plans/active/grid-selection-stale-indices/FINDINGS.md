# Findings

## Confirmed P2 — selection identity can leave the current projection

- Issues: #480 and #505.
- Select option index persisted after its options shrank; arrow keys could remain outside the new range.
- Grid keyboard navigation defaulted a missing selected row to visible position 0; ArrowDown then advanced to position 1 and skipped the first currently visible row.
- Fix: clamp select state at navigation boundary and explicitly choose the relevant edge when a selected grid row is absent from the current projection.
