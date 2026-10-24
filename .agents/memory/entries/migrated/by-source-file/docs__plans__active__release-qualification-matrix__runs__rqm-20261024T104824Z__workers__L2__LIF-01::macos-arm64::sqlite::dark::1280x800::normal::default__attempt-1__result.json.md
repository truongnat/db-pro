---
schema_version: 1
id: "MEM-SRC-71bd28ee9b591a58"
title: "Source mirror: docs/plans/active/release-qualification-matrix/runs/rqm-20261024T104824Z/workers/L2/LIF-01::macos-arm64::sqlite::dark::1280x800::normal::default/attempt-1/result.json"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/release-qualification-matrix/runs/rqm-20261024T104824Z/workers/L2/LIF-01::macos-arm64::sqlite::dark::1280x800::normal::default/attempt-1/result.json"
source_sha256: "71bd28ee9b591a58585440c16d7d5bf5d231ef3f810d5f385c98549c0dba8a0c"
mirror_body_sha256: "f52b8cf8f6375aa7398a5092f3ff566d886bbc9bfe8551dde239d6ae7304eedb"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
```json
{
  "schema_version": 1,
  "run_id": "rqm-20261024T104824Z",
  "case_id": "LIF-01",
  "instance_id": "LIF-01::macos-arm64::sqlite::dark::1280x800::normal::default",
  "attempt": 1,
  "lane": "L2",
  "worker": "lead-audit",
  "started_at": "2026-10-24T10:50:30Z",
  "finished_at": "2026-10-24T10:53:03Z",
  "artifact": {
    "source_sha": "dd2bf5e4090e6d509d34c0c8c58ca95015d91640",
    "executable_sha256": "60e3192dfafe2406de5666ad10ae0e20aa6184e1bc0ec87e2374d267c87a5ade",
    "archive_sha256": "ad731c2b9227de433e3aff70f67bf68f587873b58f9bbd0c8ed62b1c60c41b78",
    "version": "0.1.0",
    "role": "release",
    "compiler": "rustc 1.99.0 actual (disqualified; pin required 1.95.0)",
    "features": [],
    "source_dirty": false,
    "pid": 73927,
    "window_id": null
  },
  "variant": {
    "platform": "macos-arm64",
    "os_version": "macOS",
    "provider": "sqlite",
    "provider_version": null,
    "theme": "dark",
    "requested_content_size": [
      1280,
      800
    ],
    "actual_content_size": null,
    "actual_window_size": null,
    "screenshot_pixels": null,
    "scale": null,
    "state": "normal",
    "subvariant": "default",
    "input_method": "unverified",
    "locale": "en"
  },
  "fixture_manifest": "F-BASE",
  "lease_id": "truongdq-macos-primary:1",
  "status": "NOT_TESTED",
  "reason": "Retracted premature PASS: schema invalid; screenshot not pixel-reviewed; startup oracle/cleanup incomplete.",
  "checks": [],
  "evidence": [
    {
      "id": "ev-invalidation",
      "kind": "manifest",
      "path": "docs/plans/active/release-qualification-matrix/runs/rqm-20261024T104824Z/coordinator/RUN_INVALIDATION.md",
      "sha256": "9ce99ea68b36f3976d7b1eacf68ba2b570f84d629d74af4a4e55751d5fc139ee",
      "inspected": true,
      "redacted": false
    }
  ],
  "defects": [],
  "cleanup": {
    "status": "NOT_NEEDED",
    "actions": [
      "retain superseded evidence"
    ],
    "sentinels_unchanged": true,
    "evidence_refs": [
      "ev-invalidation"
    ]
  },
  "limits": [
    "run invalidated; halt GUI qualification on this candidate"
  ]
}

```
