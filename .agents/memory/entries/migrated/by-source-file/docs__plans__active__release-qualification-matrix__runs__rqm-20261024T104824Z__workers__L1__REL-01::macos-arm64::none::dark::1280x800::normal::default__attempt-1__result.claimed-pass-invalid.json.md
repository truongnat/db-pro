---
schema_version: 1
id: "MEM-SRC-6b145372529d8ed8"
title: "Source mirror: docs/plans/active/release-qualification-matrix/runs/rqm-20261024T104824Z/workers/L1/REL-01::macos-arm64::none::dark::1280x800::normal::default/attempt-1/result.claimed-pass-invalid.json"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/release-qualification-matrix/runs/rqm-20261024T104824Z/workers/L1/REL-01::macos-arm64::none::dark::1280x800::normal::default/attempt-1/result.claimed-pass-invalid.json"
source_sha256: "6b145372529d8ed88cf023c4e7f78d685596fad3f58f6f1eb4bbad281ec0a0b7"
mirror_body_sha256: "3e34114b9f0831741905b91ac3b739c1a0a085a9f17e792c91f90453f539b47c"
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
  "case_id": "REL-01",
  "instance_id": "REL-01::macos-arm64::none::dark::1280x800::normal::default",
  "attempt": 1,
  "lane": "L1",
  "worker": "lead-L1",
  "started_at": "2026-10-24T10:50:05Z",
  "finished_at": "2026-10-24T10:50:05Z",
  "artifact": {
    "source_sha": "dd2bf5e4090e6d509d34c0c8c58ca95015d91640",
    "executable_sha256": "60e3192dfafe2406de5666ad10ae0e20aa6184e1bc0ec87e2374d267c87a5ade",
    "archive_sha256": "ad731c2b9227de433e3aff70f67bf68f587873b58f9bbd0c8ed62b1c60c41b78",
    "version": "0.1.0",
    "role": "release",
    "compiler": "rustc 1.99.0 (b940084d7 2026-09-28)",
    "features": [],
    "source_dirty": [
      "M docs/plans/STATUS.md",
      "?? docs/plans/active/release-qualification-matrix/",
      "?? docs/release/2026-10-24-release-readiness.md",
      "?? docs/release/evidence/readiness-2026-10-24/"
    ],
    "pid": null,
    "window_id": null
  },
  "variant": {
    "platform": "macos-arm64",
    "os_version": "macOS",
    "provider": "none",
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
    "input_method": "cli",
    "locale": "en"
  },
  "fixture_manifest": {
    "id": "F-BASE",
    "hash": null
  },
  "lease_id": null,
  "status": "PASS",
  "reason": "",
  "checks": [
    {
      "id": "oracle-01",
      "expected": "Release archive built from candidate",
      "actual": "ad731c2b9227de433e3aff70f67bf68f587873b58f9bbd0c8ed62b1c60c41b78",
      "performed": true,
      "status": "PASS",
      "evidence_refs": [
        "ev-archive"
      ]
    }
  ],
  "evidence": [
    {
      "id": "ev-archive",
      "kind": "file",
      "path": "docs/plans/active/release-qualification-matrix/runs/rqm-20261024T104824Z/artifacts/macos-arm64/db-pro-v0.1.0-macos-arm64.tar.gz",
      "sha256": "ad731c2b9227de433e3aff70f67bf68f587873b58f9bbd0c8ed62b1c60c41b78",
      "inspected": true,
      "redacted": false
    }
  ],
  "defects": [],
  "cleanup": {
    "status": "not_applicable",
    "actions": [],
    "sentinels_unchanged": true,
    "evidence_refs": []
  },
  "limits": {
    "phase": "1",
    "note": "lead execution; Task subagents unavailable on host"
  }
}

```
