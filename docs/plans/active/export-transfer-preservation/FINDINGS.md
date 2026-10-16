# Findings — Export Transfer Preservation

## Root Cause

Each file target's `cleanup_partial` removed both its temporary file and the final destination. TransferService invokes cleanup on cancellation and source/target errors, so an older successful export could be deleted before a new transfer reached `finish`.

Delimited output emitted headers only in `write_batch`, which never runs for an empty source. `finish` then published a zero-byte CSV/TSV instead of a header row. JSONL is record-oriented and has no header line; zero rows correctly serialize to zero lines.

## Fix Strategy

Remove only the per-run temporary file in cleanup for delimited, JSONL, and Excel targets. In `DelimitedFileTarget::finish`, emit the guarded header once if no batch wrote it yet, then flush and publish normally.
