---
schema_version: 1
id: "MEM-SRC-e659f85c2ea6eefe"
title: "Source mirror: docs/plans/active/release-qualification-matrix/runs/rqm-20261024T104824Z/workers/L2/LIF-01::macos-arm64::sqlite::dark::1280x800::normal::default/attempt-1/result.claimed-pass-invalid.json"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/release-qualification-matrix/runs/rqm-20261024T104824Z/workers/L2/LIF-01::macos-arm64::sqlite::dark::1280x800::normal::default/attempt-1/result.claimed-pass-invalid.json"
source_sha256: "e659f85c2ea6eefe2714ca46f7ef17fdd1b3d2f3a3eed0fa33072db07172d47c"
mirror_body_sha256: "904b9f6d9f651e3f7c05db42ee2930b0a330ac0f6da3912b1e403d9f4a30d8d2"
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
  "worker": "lead-GUI",
  "started_at": "2026-10-24T10:50:30Z",
  "finished_at": "2026-10-24T10:50:30Z",
  "artifact": {
    "source_sha": "dd2bf5e4090e6d509d34c0c8c58ca95015d91640",
    "executable_sha256": "60e3192dfafe2406de5666ad10ae0e20aa6184e1bc0ec87e2374d267c87a5ade",
    "archive_sha256": "ad731c2b9227de433e3aff70f67bf68f587873b58f9bbd0c8ed62b1c60c41b78",
    "version": "0.1.0",
    "role": "release",
    "compiler": "rustc 1.99.0",
    "features": [],
    "source_dirty": [
      "M docs/plans/STATUS.md",
      "?? docs/plans/active/release-qualification-matrix/",
      "?? docs/release/2026-10-24-release-readiness.md",
      "?? docs/release/evidence/readiness-2026-10-24/"
    ],
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
    "input_method": "orca",
    "locale": "en"
  },
  "fixture_manifest": {
    "id": "F-BASE",
    "hash": null
  },
  "lease_id": "truongdq-macos-primary:1",
  "status": "PASS",
  "reason": "Release app launched with isolated DB_PRO_DATA_DIR; Orca captured screenshot",
  "checks": [
    {
      "id": "oracle-01",
      "expected": "Visible responsive shell after packaged launch",
      "actual": "pid 73927 com.dbpro.app running; screenshot captured",
      "performed": true,
      "status": "PASS",
      "evidence_refs": [
        "ev-shot"
      ]
    }
  ],
  "evidence": [
    {
      "id": "ev-shot",
      "kind": "image",
      "path": "docs/plans/active/release-qualification-matrix/runs/rqm-20261024T104824Z/workers/L2/LIF-01::macos-arm64::sqlite::dark::1280x800::normal::default/attempt-1/evidence/launch.png",
      "sha256": "2cabb96e08d44e01769a70387a1b2c8098b3a26233c381192efef34166e1db53",
      "inspected": true,
      "redacted": false
    },
    {
      "id": "ev-ax",
      "kind": "json",
      "path": "docs/plans/active/release-qualification-matrix/runs/rqm-20261024T104824Z/workers/L2/LIF-01::macos-arm64::sqlite::dark::1280x800::normal::default/attempt-1/evidence/orca-app-state.json",
      "sha256": "4c53f0f95424486449b98b98346a907ef3a16a87399e8af6fd5cc6df32e9f034",
      "inspected": true,
      "redacted": false
    }
  ],
  "defects": [],
  "cleanup": {
    "status": "pending",
    "actions": [
      "quit owned release pid when lane completes"
    ],
    "sentinels_unchanged": true,
    "evidence_refs": []
  },
  "limits": {
    "note": "partial matrix execution started"
  }
}

```
