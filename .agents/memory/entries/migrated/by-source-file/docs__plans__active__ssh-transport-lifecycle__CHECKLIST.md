---
schema_version: 1
id: "MEM-SRC-fe87670bae1cd77b"
title: "Source mirror: docs/plans/active/ssh-transport-lifecycle/CHECKLIST.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/ssh-transport-lifecycle/CHECKLIST.md"
source_sha256: "fe87670bae1cd77ba276a858c76d71648ce03d539420fe4afe2d5f54b6065cf9"
mirror_body_sha256: "fe87670bae1cd77ba276a858c76d71648ce03d539420fe4afe2d5f54b6065cf9"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# SSH transport lifecycle — Checklist

Baseline: `76b120724a7542de8b587a002811ddacad28156a`
State: REVIEW

## Requirements and source work

- [x] Read issues #435/#451/#400/#475 and distinguish stale issue text from actual baseline paths.
- [x] Restrict transport changes to assigned modules; parent authorized adjacent core diagnostics expansion.
- [x] Put password environment on spawned `sshpass`; omit empty identity for tunnel and SSH test.
- [x] Hold whole active connection through awaited provider disconnect; drop tunnel after success/error.
- [x] Preserve network-driver SSH intent in native Test/Create/Update translation; domain validation rejects SQL Server.
- [x] Skip direct preflight and unobserved transport successes for tunneled configs.
- [x] Add behavioral process, socket-lifetime, routing, native translation/domain validation and diagnostic regressions.
- [x] Record PostgreSQL/MySQL/SQL Server/SQLite matrix and live-provider prerequisites.

## Parent-owned execution / review

- [x] Focused selectors executed: SSH command 5/0/0, connector tunnel 3/0/0, native translation 3/0/0, core diagnostics 5/0/0 (`artifact://133`).
- [x] Integrated fmt/check/workspace tests executed (1,999/0/43); final gate failures/limits reported separately, not suppressed.
- [x] Independent reviewer ACCEPT for #435/#451/#475, no introduced findings.
- [ ] Real SSH bastion authentication remains NOT VERIFIED; local process/relay is explicitly not that evidence.
- [ ] Publish/close individually proven #435/#451/#475.
- [x] Keep #400 OPEN for missing separate stage-diagnostic acceptance.
- [ ] Parent records exact published SHA and closure comments.

No execution gate was run by this worker; checked source-work rows are not passing-test claims.
