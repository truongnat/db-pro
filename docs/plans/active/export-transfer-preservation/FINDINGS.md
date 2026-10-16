# Findings — Export Transfer Preservation

## P1 — Existing export could be deleted by a failed replacement (#436)

- **Evidence:** `FileTarget::cleanup_partial` previously removed both the run's temporary output and its final destination. `TransferService` calls cleanup on cancellation and source/target errors, so a failed transfer could delete an older successful export.
- **Disposition:** Fixed. CSV/TSV, JSONL, and Excel cleanup now removes only the current run's `.partial` file. Regression coverage verifies the prior destination bytes survive cancellation and the partial is removed.

## P2 — Empty CSV/TSV export omitted its schema (#437)

- **Evidence:** Header emission happened only in `write_batch`; an empty source never calls that method, and `finish` previously published a zero-byte file.
- **Disposition:** Fixed. Delimited `finish` emits the header once if no batch has written it. Regression coverage verifies headers for empty CSV/TSV. JSONL remains record-only: an empty export correctly has zero lines, not a pseudo-header.
