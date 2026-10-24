---
schema_version: 1
id: "MEM-SRC-0f8c7efadc20207f"
title: "Source mirror: docs/plans/active/ui-agent-workspace-polish/PLAN.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/ui-agent-workspace-polish/PLAN.md"
source_sha256: "0f8c7efadc20207f012d7443d06245287ae374977cfc9a4911fcac1c6cda7563"
mirror_body_sha256: "0f8c7efadc20207f012d7443d06245287ae374977cfc9a4911fcac1c6cda7563"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Plan — UI10 Agent Workspace Polish

## Objective
Standardize and modernize the Agent workspace surfaces to feel like a first-class IDE surface with clear conversation hierarchy, distinct tool activity, safe action visibility, and compliant design tokens.

## Scope
- `crates/ui/src/agent_view.rs`: Standardize header actions, mode dropdown, settings panel, context chips & quick actions, empty state suggestions, message bubble frames, tool-call activity items, execution approval/confirmation cards, failure/retry states, and composer toolbar.
- Standardize all legacy button helpers into canonical `Button` primitives.
- Add `include_str!("../agent_view.rs")` to the non-regression raw-button test in `crates/ui/src/components/mod.rs`.
- Verify full test suite and native release build.
