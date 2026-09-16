# Competitor Parity, Gap, and Positioning Evidence Map

**Document**: `docs/brand/competitor-parity-gap-positioning-map.md`  
**Status**: Completed  
**Baseline Commit**: `e3df4896`  
**Related Issue**: #141 (Parent #96, Supports #97, #98, #120)

---

## 1. Executive Synthesis

This document synthesizes competitor benchmarks across traditional desktop IDEs (DBeaver, DataGrip, TablePlus, Beekeeper Studio), modern local-first/lightweight clients, and emerging AI/agent database tools.

---

## 2. Market Capability & Parity Matrix

| Capability | Market Baseline Expectation | DB Pro v0.1 Status | Release Implication |
| :--- | :--- | :--- | :--- |
| **Multi-Tab SQL Editor** | Syntax highlight, autocomplete, execution timer | Shipped (tree-sitter & native highlight) | Commodity baseline; do not market as novel |
| **Schema Introspection** | Tables, columns, PK/FK, indexes, views | Shipped (PostgreSQL + SQLite) | Solid parity for core workflows |
| **Visual Result Grid** | Sorting, pagination, inline cell edit, copy TSV/JSON | Shipped (virtualized egui grid) | Fast, native 60fps performance |
| **ER Diagram** | Auto-layout FK graphs, zoom/pan, detail inspector | Shipped (interactive canvas) | Differentiator in responsiveness and lightness |
| **Agent / Copilot Interface** | AI SQL generation, schema explanation | Shipped (native ACP / prompt-driven) | Core product wedge; typed safety execution |
| **Cross-Platform Native** | Fast startup, low RAM, no Electron lag | Shipped (pure Rust eframe/egui) | Key UX advantage over Electron/Java |

---

## 3. Competitor Strengths & DB Pro Response Strategy

| Competitor | Core Strength | DB Pro Response |
| :--- | :--- | :--- |
| **DBeaver** | Hundreds of JDBC drivers, enterprise plugins | **Defer / Match Core**: Focus deeply on PostgreSQL + SQLite first with superior ergonomics and zero Java bloat. |
| **DataGrip (JetBrains)** | Unmatched SQL refactoring, deep schema analysis | **Differentiate**: Native lightweight desktop speed combined with native ACP Agent execution loops. |
| **TablePlus** | Clean, fast native macOS/Windows UI | **Match & Differentiate**: Match native snappy feel while providing first-class AI Agent workspace & open schema tooling. |
| **Chat2DB / Defog** | AI SQL chat wrapper | **Differentiate**: Typed schema safety, local credential privacy, human confirmation for destructive queries, no cloud lock-in. |

---

## 4. Safe Claims vs Red Lines

### Safe Claims in v0.1
* "Instant native startup and sub-50MB memory footprint powered by pure Rust."
* "Zero telemetry credential safety with OS keyring and encrypted storage."
* "Human-in-the-loop AI Agent workflows with explicit destructive query gates."

### Red Lines (Do Not Claim in v0.1)
* Do *not* claim "Replaces all 100+ DBeaver JDBC drivers" (v0.1 is focused on PostgreSQL & SQLite).
* Do *not* claim "Full automated autonomous schema migration without human review".
* Do *not* claim "Cloud team collaboration" (v0.1 is strictly local-first desktop).
