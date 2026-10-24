---
schema_version: 1
id: "MEM-SRC-1598e5a6a9fa3893"
title: "Source mirror: docs/plans/completed/native-ai-agent/FINDINGS.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/completed/native-ai-agent/FINDINGS.md"
source_sha256: "1598e5a6a9fa3893ac436a40dc369fd226a959a65686cb58bcb5368a27d89461"
mirror_body_sha256: "1598e5a6a9fa3893ac436a40dc369fd226a959a65686cb58bcb5368a27d89461"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Findings

- The native panel was presentation-only: its input was disabled and it had no message state or command path.
- The existing frontend has template-agent behavior, but native schema DTOs currently expose only summary table and column names. The native first slice therefore generates safe, inspectable drafts from that summary and does not pretend to have foreign-key context it does not receive.
- Native architecture forbids putting credentials in UI state. Provider configuration and network calls remain a later runtime concern.
