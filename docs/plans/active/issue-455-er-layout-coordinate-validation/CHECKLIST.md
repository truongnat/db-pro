# Issue #455 — CHECKLIST
- Issue: https://github.com/truongnat/db-pro/issues/455
- Fix: Sanitizes coordinates in restore_diagram_layouts and apply_position_overrides, rejecting NaN, Inf, and coordinates > 1,000,000 to prevent canvas blanking.
