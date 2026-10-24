---
schema_version: 1
id: "MEM-SRC-6a0a28acc7bc2b46"
title: "Source mirror: docs/release/evidence/v01-runtime/providers/62-pg-secure-default-tls-require.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/release/evidence/v01-runtime/providers/62-pg-secure-default-tls-require.md"
source_sha256: "6a0a28acc7bc2b4680893842eea7afcca50bc58008b9f6c46a519a90fca75fbf"
mirror_body_sha256: "6a0a28acc7bc2b4680893842eea7afcca50bc58008b9f6c46a519a90fca75fbf"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# PostgreSQL new connections default to TLS Require (#144)

- Session: 2026-09-15
- Issue: **#144** ([P1][RC1][Security] Make PostgreSQL remote connections secure by default)
- Locked contract: owner IMPLEMENTATION CONTRACT on the issue (new PG form defaults to `require`; do **not** change Rust `SslMode::default()` / serde fallback)
- Related prior work: #236 edit/duplicate SSL preservation (`aba947b`); triage record `providers/27-pg-secure-default.md`

## 1. What changed

| Location | Change |
|---|---|
| `crates/ui/src/runtime.rs` `UiConnectionDraft::default` | `ssl_mode: UiSslMode::Require` (was `Disable`) |
| `crates/ui/src/connection_view.rs` `select_connection_driver` | SQLite → PostgreSQL re-initializes TLS to `Require` |
| `crates/ui/src/connection_view.rs` SSL selector | Per-mode guidance; `Disable` uses danger colour |
| `crates/core/.../connection.rs` `SslMode` | **unchanged** — `#[default] Disable` and `#[serde(default)]` stay so persisted records that omit the field keep decoding as Disable |

## 2. Tests (fail-before / pass-after intent)

| Test | Asserts |
|---|---|
| `new_postgresql_connection_defaults_to_tls_require` | New dialog + submitted create command carry `Require` |
| `editing_a_disable_connection_keeps_disable_until_the_user_changes_it` | Saved `Disable` is not rewritten |
| `editing_a_connection_preserves_its_stored_ssl_mode` / `duplicating_…` | `#236` regressions still green |
| `switching_sqlite_to_postgresql_initializes_tls_require` | Driver switch resets TLS to `Require` |
| `explicit_disable_selection_is_preserved_on_submit` | User opt-out of TLS still submits `Disable` |
| `ssl_mode_guidance_names_the_plaintext_risk_for_disable` | Guidance strings present for all four modes |
| `default_draft_neutral_fields_are_profile_independent` | Updated expectation to `Require` |

```text
cargo test -p db-pro-ui --lib -- tls_require switching_sqlite editing_a_disable explicit_disable ssl_mode_guidance default_draft_neutral editing_a_connection_preserves duplicating_a_connection
# 8 passed; 0 failed
```

## 3. Deliberate non-claims

- No CA / client-cert UI (contract forbids inventing it here).
- No change to Rust domain default / serde fallback.
- Packaged-runtime / interactive GUI observation of the warning text is not claimed (`R-GUI-SMOKE`).
