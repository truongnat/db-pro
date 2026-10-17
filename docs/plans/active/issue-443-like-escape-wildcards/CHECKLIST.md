# Issue #443 — CHECKLIST
- Issue: https://github.com/truongnat/db-pro/issues/443
- Fix: escape_like_pattern escapes %, _, and \ in user input for Contains/StartsWith/EndsWith filters, and text_like appends ESCAPE clause to prevent wildcards from treating user literals as wildcards.
