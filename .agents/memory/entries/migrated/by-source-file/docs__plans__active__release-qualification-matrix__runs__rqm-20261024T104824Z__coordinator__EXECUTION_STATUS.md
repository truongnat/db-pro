---
schema_version: 1
id: "MEM-SRC-513ac8edbc984954"
title: "Source mirror: docs/plans/active/release-qualification-matrix/runs/rqm-20261024T104824Z/coordinator/EXECUTION_STATUS.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/release-qualification-matrix/runs/rqm-20261024T104824Z/coordinator/EXECUTION_STATUS.md"
source_sha256: "513ac8edbc984954f425bdbb3be4c7607aad4adfe44f57b1884bb36eae8d7cb0"
mirror_body_sha256: "513ac8edbc984954f425bdbb3be4c7607aad4adfe44f57b1884bb36eae8d7cb0"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Execution status — `rqm-20261024T104824Z`

**State:** `INVALIDATED` — **halted**; no further PASS/GUI on this candidate.

## Audit summary (2026-10-24)

| Check | Outcome |
|---|---|
| Compiler pin 1.95.0 | **FAIL** — build used rustc **1.99.0** |
| `RESULT.schema.json` (3 results) | **FAIL** — prior PASS JSON invalid (see `result.claimed-pass-invalid.json`) |
| AGGREGATION §2 semantic | **FAIL** — ENV-01/LIF-01 oracles & evidence insufficient; LIF screenshot not pixel-reviewed |
| Authoritative statuses | ENV-01 `NOT_TESTED`, REL-01 `BLOCKED`, LIF-01 `NOT_TESTED` |

Evidence **retained**; see [RUN_INVALIDATION.md](./RUN_INVALIDATION.md).

## Scale (unchanged)

3,663 candidate instances; 1,221 macos-arm64 eligible when a **valid** run starts.

## Next

Preflight only: [../rqm-next-pin195/preflight.json](../rqm-next-pin195/preflight.json) — new run after `+1.95.0` rebuild.
