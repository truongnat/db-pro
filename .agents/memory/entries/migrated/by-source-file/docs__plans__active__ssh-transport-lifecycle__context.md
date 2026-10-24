---
schema_version: 1
id: "MEM-SRC-da0e0fee70451fb1"
title: "Source mirror: docs/plans/active/ssh-transport-lifecycle/context.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/ssh-transport-lifecycle/context.md"
source_sha256: "da0e0fee70451fb1aeabd03d0d8642d50ecdfe522a5ad9348342f97487a49375"
mirror_body_sha256: "da0e0fee70451fb1aeabd03d0d8642d50ecdfe522a5ad9348342f97487a49375"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Context — #435/#451/#475; #400 partial

Source baseline: `76b120724a7542de8b587a002811ddacad28156a`.

Reuse shared process builder and existing domain unsupported-driver validation. Defer direct preflight for tunneled configs rather than invent SSH success. No credential persistence/host-key relaxation or UI redesign.

Detailed source pointers/alternatives and inherited limits: PLAN.md, FINDINGS.md.
