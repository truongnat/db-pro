# Issue #364: Schema Workbench Apply doesn't verify fingerprint

Status: VERIFIED-FIXED (already fixed in 9fe8706e)

No code change required. Same fix as #353: apply rebuilds the pending request from the current form and compares fingerprints before dispatch.
