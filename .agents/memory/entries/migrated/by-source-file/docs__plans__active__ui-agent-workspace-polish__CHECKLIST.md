---
schema_version: 1
id: "MEM-SRC-5b59064ca0cea1e0"
title: "Source mirror: docs/plans/active/ui-agent-workspace-polish/CHECKLIST.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/ui-agent-workspace-polish/CHECKLIST.md"
source_sha256: "5b59064ca0cea1e01eefd5d1d8f7f66a4ef77c0403fcb5c0b982aebd2faa501e"
mirror_body_sha256: "5b59064ca0cea1e01eefd5d1d8f7f66a4ef77c0403fcb5c0b982aebd2faa501e"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Checklist — UI10 Agent Workspace Polish

- [x] Standardize agent header and mode selector actions in `agent_view.rs`.
- [x] Standardize agent settings / API key controls in `agent_view.rs`.
- [x] Standardize agent context quick actions in `agent_view.rs`.
- [x] Standardize empty state starter prompts and action triggers in `agent_view.rs`.
- [x] Standardize query result cards and tool activity in `agent_view.rs`.
- [x] Standardize confirmation / execution approval / retry cards in `agent_view.rs`.
- [x] Standardize composer states and button variants in `agent_view.rs` / `agent_composer.rs`.
- [x] Include `agent_view.rs` in `primary_native_surfaces_do_not_reintroduce_raw_buttons` test.
- [x] Run full workspace quality gates (`cargo fmt`, `cargo check`, `cargo clippy`, `cargo test`, `cargo build --release --locked -p db-pro-native`).
- [x] Record verification evidence.
