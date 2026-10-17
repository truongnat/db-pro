# Issue #365: auto_increment overrides type wrong — SQLite INTEGER PK / Postgres Alter SERIAL

Status: VERIFIED-FIXED (already fixed in 9fe8706e)

No code change required. `build_mutation_request` rejects `auto_increment && !is_pk` on all drivers, rejects Postgres `Alter + auto_increment` (SERIAL is create-only), and only rewrites the type for the (now PK-only) create path.
