---
schema_version: 1
id: "MEM-SRC-51a000b1b52a46e7"
title: "Source mirror: docs/plans/active/release-qualification-matrix/runs/rqm-20261024T104824Z/coordinator/host-blocked-summary.json"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/release-qualification-matrix/runs/rqm-20261024T104824Z/coordinator/host-blocked-summary.json"
source_sha256: "51a000b1b52a46e7df3b0adc7c5c6a9a5f1950389f32d5a764e5e4f4c39db49e"
mirror_body_sha256: "8edeb73b151d375137295ed391dbe6b87817b6d55eb33f1700ef6e1b93af6fb1"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
```json
{
  "run_id": "rqm-20261024T104824Z",
  "status": "BLOCKED_HOST",
  "count": 2442,
  "reason": "windows-x64/linux-x64 instances require assigned target hosts; not executed on macOS Orca host",
  "reference": "expected-instances.json host_execution_eligibility"
}

```
