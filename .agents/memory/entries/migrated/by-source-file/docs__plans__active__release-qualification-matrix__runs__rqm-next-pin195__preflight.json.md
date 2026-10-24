---
schema_version: 1
id: "MEM-SRC-9b70f5f5fc00a007"
title: "Source mirror: docs/plans/active/release-qualification-matrix/runs/rqm-next-pin195/preflight.json"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/release-qualification-matrix/runs/rqm-next-pin195/preflight.json"
source_sha256: "9b70f5f5fc00a0071e6ac7968558a5eff9e38da53b378c63e62cc0697b1742ab"
mirror_body_sha256: "7966118fb84c3dc330efb469fea38a11a260a14766c02bf62441b3b85e8aa388"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
```json
{
  "status": "NOT_STARTED",
  "purpose": "Successor qualification run after compiler pin correction",
  "prerequisites": [
    "cargo +1.95.0 build --release -p db-pro-native --locked",
    "Record rustc +1.95.0 --version in run-manifest and ENV-01 artifact.compiler",
    "Repackage macOS archive; new executable/archive SHA256",
    "New run_id (do not reuse rqm-20261024T104824Z expected-instances digest with wrong artifact)",
    "No PASS until RESULT.schema.json + AGGREGATION §2 semantic review"
  ],
  "forbidden": [
    "RUSTUP_TOOLCHAIN=stable or ambient default overriding +1.95.0 for qualifying build",
    "Treating rqm-20261024T104824Z artifacts as release-role"
  ],
  "source_baseline": "dd2bf5e4090e6d509d34c0c8c58ca95015d91640"
}

```
