---
schema_version: 1
id: "MEM-SRC-8f4bdaaabd1d783f"
title: "Source mirror: docs/plans/active/backend-four-issue-wave/dod.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/backend-four-issue-wave/dod.md"
source_sha256: "8f4bdaaabd1d783fca17c1315b7fb3dcd241a5b9cbadadfb2734da3e1c4667b6"
mirror_body_sha256: "8f4bdaaabd1d783fca17c1315b7fb3dcd241a5b9cbadadfb2734da3e1c4667b6"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Definition of Done

- [ ] D1 R1: real PG event trigger built from bare/qualified inputs fires, metadata verifies selected function; malformed inputs rejected.
- [ ] D2 R2: real PG exact dependencies across search_path, same names/wildcards/quoted identifiers, missing relation deterministic.
- [ ] D3 R3: actual MySQL auth/query/test_connection with reserved/Unicode fields, negative password and unchanged SSL mapping.
- [ ] D4 R4: real UTF-8 BOM/no-BOM CSV/TSV previews + streamed rows/SQLite transfer; no speculative double-strip.
- [ ] D5 all: introduced P0/P1 resolved, meaningful regressions/gates observed, exact published SHAs, individual GitHub closure evidence, expected140 open checked without falsely claiming concurrent changes.
