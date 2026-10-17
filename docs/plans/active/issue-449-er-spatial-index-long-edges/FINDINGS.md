# Issue #449 — FINDINGS
- Issue: https://github.com/truongnat/db-pro/issues/449
- Fix: insert_edge samples cells across the span when spanning > 32 cells rather than truncating from min_corner, ensuring far endpoints and mid-spans remain queryable.
