# Welcome and query editor polish — Checklist

## Planning
- [x] Evidence and failure scenario recorded
- [x] Scope/non-goals explicit
- [x] PostgreSQL/SQLite support matrix explicit

## Implementation
- [x] Smallest coherent implementation complete
- [x] Safety/capability rules respected
- [x] Cache/state invalidation accounted for (n/a — presentation only)

## Tests
- [x] `welcome_top_inset_keeps_the_start_block_in_the_upper_third`
- [x] `the_filter_field_border_is_painted_inside_the_field` still passes with a connection present
- [x] PostgreSQL: N/A (no SQL change)
- [x] SQLite: N/A (no SQL change)

## Review
- [x] Architecture review complete (self)
- [x] Correctness/security review complete (self)
- [x] P0 = 0
- [x] P1 = 0

## Runtime
- [x] Capture screenshots of Welcome and Query at the three gate sizes
- [x] VERIFICATION.md contains the commands actually run
- [x] STATUS.md row added
