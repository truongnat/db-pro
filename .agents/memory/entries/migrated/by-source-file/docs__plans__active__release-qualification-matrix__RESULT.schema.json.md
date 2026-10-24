---
schema_version: 1
id: "MEM-SRC-a7736ec7869e1abe"
title: "Source mirror: docs/plans/active/release-qualification-matrix/RESULT.schema.json"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/release-qualification-matrix/RESULT.schema.json"
source_sha256: "a7736ec7869e1abe28c6e974f36f0d10aaec97bd178a8415b4d6778d24734476"
mirror_body_sha256: "1d9493744bbb8768628829e90aebc5db0ca0aa5866cf2517e0a32404338074e5"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://db-pro.invalid/qa/release-qualification-result-v1.schema.json",
  "title": "One attempted release-qualification instance; not a template result",
  "type": "object",
  "additionalProperties": false,
  "required": ["schema_version", "run_id", "case_id", "instance_id", "attempt", "lane", "worker", "started_at", "finished_at", "artifact", "variant", "fixture_manifest", "lease_id", "status", "reason", "checks", "evidence", "defects", "cleanup", "limits"],
  "properties": {
    "schema_version": {"const": 1},
    "run_id": {"type": "string", "pattern": "^[A-Za-z0-9_-]+$"},
    "case_id": {"type": "string", "pattern": "^[A-Z]+-[0-9]{2}$"},
    "instance_id": {"type": "string", "minLength": 20},
    "attempt": {"type": "integer", "minimum": 1},
    "lane": {"type": "string", "pattern": "^L(?:[0-9]|10|11)$"},
    "worker": {"type": "string", "minLength": 1},
    "started_at": {"type": "string", "format": "date-time"},
    "finished_at": {"type": "string", "format": "date-time"},
    "artifact": {
      "type": "object", "additionalProperties": false,
      "required": ["source_sha", "executable_sha256", "archive_sha256", "version", "role", "compiler", "features", "source_dirty", "pid", "window_id"],
      "properties": {
        "source_sha": {"type": "string", "pattern": "^[0-9a-f]{40}$"},
        "executable_sha256": {"$ref": "#/$defs/digest"},
        "archive_sha256": {"anyOf": [{"$ref": "#/$defs/digest"}, {"type": "null"}]},
        "version": {"type": "string", "minLength": 1},
        "role": {"enum": ["release", "capture-supplemental", "debug-supplemental"]},
        "compiler": {"type": "string", "minLength": 1},
        "features": {"type": "array", "items": {"type": "string"}, "uniqueItems": true},
        "source_dirty": {"type": "boolean"},
        "pid": {"type": ["integer", "null"], "minimum": 1},
        "window_id": {"type": ["string", "null"]}
      }
    },
    "variant": {
      "type": "object", "additionalProperties": false,
      "required": ["platform", "os_version", "provider", "provider_version", "theme", "requested_content_size", "actual_content_size", "actual_window_size", "screenshot_pixels", "scale", "state", "subvariant", "input_method", "locale"],
      "properties": {
        "platform": {"enum": ["macos-arm64", "windows-x64", "linux-x64"]},
        "os_version": {"type": "string", "minLength": 1},
        "provider": {"enum": ["none", "sqlite", "postgres", "mysql", "sqlserver"]},
        "provider_version": {"type": ["string", "null"]},
        "theme": {"enum": ["dark", "light"]},
        "requested_content_size": {"$ref": "#/$defs/size"},
        "actual_content_size": {"anyOf": [{"$ref": "#/$defs/size"}, {"type": "null"}]},
        "actual_window_size": {"anyOf": [{"$ref": "#/$defs/size"}, {"type": "null"}]},
        "screenshot_pixels": {"anyOf": [{"$ref": "#/$defs/size"}, {"type": "null"}]},
        "scale": {"type": ["number", "null"], "exclusiveMinimum": 0},
        "state": {"type": "string", "minLength": 1},
        "subvariant": {"type": "string", "minLength": 1},
        "input_method": {"enum": ["real-keyboard", "real-pointer", "AXPress", "AXSetValue-verified", "mixed-verified", "cli", "unverified"]},
        "locale": {"type": "string", "minLength": 1}
      }
    },
    "fixture_manifest": {"type": "string", "minLength": 1},
    "lease_id": {"type": ["string", "null"]},
    "status": {"$ref": "#/$defs/status"},
    "reason": {"type": "string", "minLength": 8},
    "checks": {
      "type": "array",
      "items": {
        "type": "object", "additionalProperties": false,
        "required": ["id", "expected", "actual", "performed", "status", "evidence_refs"],
        "properties": {
          "id": {"type": "string", "minLength": 1},
          "expected": {"type": "string", "minLength": 1},
          "actual": {"type": "string"},
          "performed": {"type": "boolean"},
          "status": {"$ref": "#/$defs/status"},
          "evidence_refs": {"type": "array", "items": {"type": "string"}, "uniqueItems": true}
        }
      }
    },
    "evidence": {
      "type": "array", "items": {
        "type": "object", "additionalProperties": false,
        "required": ["id", "kind", "path", "sha256", "inspected", "redacted"],
        "properties": {
          "id": {"type": "string", "minLength": 1},
          "kind": {"enum": ["screenshot", "video", "ax-tree", "action-log", "process-log", "oracle", "manifest", "source", "measurement", "ci"]},
          "path": {"type": "string", "minLength": 1},
          "sha256": {"$ref": "#/$defs/digest"},
          "inspected": {"type": "boolean"},
          "redacted": {"type": "boolean"}
        }
      }
    },
    "defects": {"type": "array", "items": {
      "type": "object", "additionalProperties": false,
      "required": ["id", "severity", "confidence", "reproduction", "expected", "actual", "evidence_refs", "regression_candidate"],
      "properties": {
        "id": {"type": "string", "minLength": 1},
        "severity": {"enum": ["P0", "P1", "P2"]},
        "confidence": {"enum": ["CONFIRMED", "LIKELY", "RISK", "DESIGN_CONCERN"]},
        "reproduction": {"type": "array", "minItems": 1, "items": {"type": "string"}},
        "expected": {"type": "string", "minLength": 1},
        "actual": {"type": "string", "minLength": 1},
        "evidence_refs": {"type": "array", "minItems": 1, "items": {"type": "string"}},
        "regression_candidate": {"type": "string", "minLength": 1}
      }
    }},
    "cleanup": {
      "type": "object", "additionalProperties": false,
      "required": ["status", "actions", "sentinels_unchanged", "evidence_refs"],
      "properties": {
        "status": {"enum": ["DONE", "PARTIAL", "BLOCKED", "NOT_NEEDED"]},
        "actions": {"type": "array", "items": {"type": "string"}},
        "sentinels_unchanged": {"type": ["boolean", "null"]},
        "evidence_refs": {"type": "array", "items": {"type": "string"}}
      }
    },
    "limits": {"type": "array", "items": {"type": "string"}},
    "blocked_by": {"type": "array", "items": {"type": "string"}},
    "na_approval": {"type": ["string", "null"]}
  },
  "allOf": [
    {"if": {"properties": {"status": {"const": "PASS"}}}, "then": {
      "properties": {
        "checks": {"minItems": 1, "items": {"properties": {"performed": {"const": true}, "status": {"const": "PASS"}, "evidence_refs": {"minItems": 1}}}},
        "evidence": {"minItems": 1},
        "cleanup": {"properties": {"status": {"enum": ["DONE", "NOT_NEEDED"]}}}
      }
    }},
    {"if": {"properties": {"status": {"const": "FAIL"}}}, "then": {"properties": {"checks": {"minItems": 1}, "evidence": {"minItems": 1}, "defects": {"minItems": 1}}}},
    {"if": {"properties": {"status": {"const": "BLOCKED"}}}, "then": {"required": ["blocked_by"], "properties": {"blocked_by": {"minItems": 1}}}},
    {"if": {"properties": {"status": {"const": "N_A"}}}, "then": {"required": ["na_approval"], "properties": {"na_approval": {"type": "string", "minLength": 1}}}}
  ],
  "$defs": {
    "digest": {"type": "string", "pattern": "^[0-9a-f]{64}$"},
    "status": {"enum": ["PASS", "FAIL", "BLOCKED", "NOT_TESTED", "N_A"]},
    "size": {"type": "array", "prefixItems": [{"type": "integer", "minimum": 1}, {"type": "integer", "minimum": 1}], "minItems": 2, "maxItems": 2}
  }
}

```
