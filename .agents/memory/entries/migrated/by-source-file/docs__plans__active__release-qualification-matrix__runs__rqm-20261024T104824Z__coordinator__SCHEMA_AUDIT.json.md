---
schema_version: 1
id: "MEM-SRC-ce1ee188117e23d0"
title: "Source mirror: docs/plans/active/release-qualification-matrix/runs/rqm-20261024T104824Z/coordinator/SCHEMA_AUDIT.json"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/release-qualification-matrix/runs/rqm-20261024T104824Z/coordinator/SCHEMA_AUDIT.json"
source_sha256: "ce1ee188117e23d0ed31e6bfe4595eadca4b0c38586bac7f030628cc3ab68173"
mirror_body_sha256: "c071a763d61126b25eb2285ac412015c9880a8f437d85b8a9625fc3aeda4e17b"
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
  "audited_at_utc": "2026-10-24T10:55:00Z",
  "tool": "jsonschema Draft202012Validator on RESULT.schema.json",
  "claimed_pass_files": [
    "workers/L0/ENV-01::macos-arm64::none::dark::1280x800::normal::default/attempt-1/result.claimed-pass-invalid.json",
    "workers/L1/REL-01::macos-arm64::none::dark::1280x800::normal::default/attempt-1/result.claimed-pass-invalid.json",
    "workers/L2/LIF-01::macos-arm64::sqlite::dark::1280x800::normal::default/attempt-1/result.claimed-pass-invalid.json"
  ],
  "prior_pass_schema_valid": false,
  "prior_pass_common_defects": [
    "source_dirty array vs boolean",
    "cleanup.status not in enum",
    "evidence.kind not in allowed set",
    "fixture_manifest object vs string",
    "limits object vs string array",
    "PASS cleanup not DONE/NOT_NEEDED",
    "compiler 1.99.0 vs pin 1.95.0"
  ],
  "authoritative_result_schema_valid": true,
  "downgraded": {
    "ENV-01::macos-arm64::none::dark::1280x800::normal::default": "NOT_TESTED",
    "REL-01::macos-arm64::none::dark::1280x800::normal::default": "BLOCKED",
    "LIF-01::macos-arm64::sqlite::dark::1280x800::normal::default": "NOT_TESTED"
  }
}

```
