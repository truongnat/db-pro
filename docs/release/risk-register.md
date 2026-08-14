# Central Release Risk, Decision & Blocker Register — v0.1

> One canonical release-risk and decision ledger for DB Pro v0.1.
> Baseline SHA: `65bbca3`
> Issue: #136
> Parent Goal: #14

## Status

In Progress — control register

## What belongs here

Only release-significant items:
- P0/P1 blockers
- P2 items requiring Fix/Accept/Defer decision
- Product-scope decisions that affect release claims
- Platform support reductions
- Signing/license/security decisions
- Candidate-freeze invalidation events
- Runtime smoke defects
- Public identity/rename blockers
- Unresolved evidence/provenance gaps

## Risk records

```
RISK R001
Source issue: #14
Discovered on SHA: n/a (external research)
Area: Brand
Severity: P1
Type: product-truth
Statement: Public name "DB Pro" collides with existing dbpro.app product
Evidence: #137 (dbpro.app teardown), #141 (synthesis)
Impact: Legal/branding risk if published under current name
Disposition: OPEN
Owner issue: #101, #104
Blocks: Public release, README (#110), release notes (#105)
Decision/evidence: Pending brand decision
```

```
RISK R002
Source issue: #134
Discovered on SHA: 65bbca3
Area: Packaging
Severity: P1
Type: correctness
Statement: Missing libsecret dependency in deb/rpm package declarations
Evidence: tauri.conf.json deb.depends, rpm.depends; keyring crate requires libsecret
Impact: Credential storage fails on minimal Linux installs
Disposition: OPEN
Owner issue: #134
Blocks: Linux .deb/.rpm smoke (#91-#93)
Decision/evidence: Fix needed — add libsecret-1-0 to deb depends, libsecret to rpm depends
```

```
RISK R003
Source issue: #118
Discovered on SHA: n/a
Area: Trust
Severity: P2
Type: release
Statement: No code signing or notarization configured for any platform
Evidence: tauri.conf.json has no signing block
Impact: macOS Gatekeeper, Windows SmartScreen warnings; reduced user trust
Disposition: OPEN
Owner issue: #118
Blocks: None (release can proceed unsigned, but with degraded UX)
Decision/evidence: Pending signing decision
```

```
RISK R004
Source issue: #119
Discovered on SHA: n/a
Area: Release
Severity: P1
Type: governance
Statement: No LICENSE file; redistribution policy unresolved
Evidence: No LICENSE file at repo root
Impact: Cannot legally distribute; blocks public release
Disposition: OPEN
Owner issue: #119
Blocks: Public release
Decision/evidence: Pending license decision
```

```
RISK R005
Source issue: #68
Discovered on SHA: n/a
Area: Schema
Severity: P2
Type: product-truth
Statement: CHECK constraint introspection disposition unresolved
Evidence: No check_constraints field in SchemaCapabilities
Impact: May show incorrect constraint info or miss edge cases
Disposition: OPEN
Owner issue: #68
Blocks: Schema introspection accuracy claims
Decision/evidence: Pending keep/defer decision
```

```
RISK R006
Source issue: #134
Discovered on SHA: 65bbca3
Area: Packaging
Severity: P2
Type: release
Statement: pg_dump/pg_restore not bundled; backup/restore requires them on PATH
Evidence: crates/infrastructure/src/backup/pg_dump.rs shells out to pg_dump
Impact: Backup silently fails if pg_dump not installed; no in-app guidance
Disposition: OPEN
Owner issue: #134
Blocks: Backup feature claims in release notes
Decision/evidence: Accept RC1 with documentation; or bundle pg_dump
```

```
RISK R007
Source issue: Gate 4
Discovered on SHA: 65bbca3
Area: Gate4
Severity: P1
Type: release
Statement: Gate 4 (large-schema ER) is release P1 until merged/verified
Evidence: PR #36 draft; Slice A (596f285) + Slice B (65bbca3) merged to feature branch
Impact: Large-schema rendering not qualified for release
Disposition: FIX RC1
Owner issue: #15, #16, #17, #18, #19, #20
Blocks: Gate 5 (#21-#25)
Decision/evidence: In progress — Slice A+B done; C+D pending
```

```
RISK R008
Source issue: Gate 5
Discovered on SHA: n/a
Area: Gate5
Severity: P1
Type: release
Statement: Gate 5 (value types) is release P1 and blocked by Gate 4
Evidence: No Gate 5 implementation yet
Impact: Value type rendering/verification not qualified
Disposition: OPEN
Owner issue: #21-#25
Blocks: Release
Decision/evidence: Blocked by Gate 4 completion
```

```
RISK R009
Source issue: #134
Discovered on SHA: 65bbca3
Area: Packaging
Severity: P2
Type: reliability
Statement: SSH tunnel not E2E qualified; shells out to system ssh binary
Evidence: crates/infrastructure/src/ssh/tunnel.rs
Impact: SSH tunneling may not work reliably across platforms
Disposition: ACCEPT RC1
Owner issue: N/A
Blocks: SSH feature claims in release notes
Decision/evidence: Accept as experimental; document limitation (LIM-006)
```

```
RISK R010
Source issue: #134
Discovered on SHA: 65bbca3
Area: Packaging
Severity: P2
Type: release
Statement: No WebView2 bootstrapper for Windows; older systems may fail
Evidence: Tauri 2 does not bundle WebView2
Impact: Windows 10 systems without WebView2 cannot run the app
Disposition: ACCEPT RC1
Owner issue: #134
Blocks: Windows distribution claims
Decision/evidence: Accept RC1; WebView2 is pre-installed on Win11 and recent Win10
```

```
RISK R011
Source issue: #142
Discovered on SHA: a2cf4a3
Area: Security
Severity: P1
Type: correctness
Statement: keyring v3 compiled without platform credential-store features; production bootstrap used encrypted-file fallback with key derived from non-secret service name
Evidence: Cargo.toml `keyring = "3"` (no features); lib.rs `.with_fallback()` in production
Impact: Secrets stored in mock in-memory store or weakly-encrypted file; credentials lost on restart
Disposition: FIX RC1
Owner issue: #142
Blocks: Production credential storage, packaged app smoke
Decision/evidence: Fix in progress — enable apple-native/linux-native-sync-persistent/windows-native features; remove .with_fallback() from production; credential roundtrip test replaces weak Entry::new check. #142 reopened: exit evidence must demonstrate native backend on each platform, not just mock store
```

```
RISK R012
Source issue: #143 (escaped from #123)
Discovered on SHA: a2cf4a3
Area: Frontend
Severity: P1
Type: correctness
Statement: No React error boundary — any render crash causes white screen of death
Evidence: No ErrorBoundary component in frontend/src
Impact: Unhandled render errors crash the entire app with no recovery path
Disposition: FIX RC1
Owner issue: #143
Blocks: Packaged app reliability
Decision/evidence: Pending implementation
```

```
RISK R013
Source issue: #144 (escaped from #125)
Discovered on SHA: a2cf4a3
Area: Security
Severity: P1
Type: correctness
Statement: PostgreSQL remote connections are silently plaintext by default (SslMode::Disable)
Evidence: domain/connection.rs SslMode default = Disable; no TLS enforcement for remote hosts
Impact: Credentials and data sent in plaintext over network to remote PG servers
Disposition: FIX RC1
Owner issue: #144
Blocks: Remote connection security claims
Decision/evidence: Pending — require TLS for remote connections; localhost may remain plaintext
```

```
RISK R014
Source issue: #145 (escaped from #128)
Discovered on SHA: a2cf4a3
Area: Data Safety
Severity: P1
Type: correctness
Statement: SQLite backup/restore uses tokio::fs::copy — not snapshot-safe for live/WAL databases
Evidence: infrastructure/backup/sqlite_backup.rs is a file copy; no WAL checkpoint before copy
Impact: Backup of live SQLite database may be inconsistent; restore may overwrite current data
Disposition: FIX RC1
Owner issue: #145
Blocks: Backup feature claims
Decision/evidence: Pending — need snapshot mechanism, WAL correctness, partial-failure safety
```

```
RISK R015
Source issue: #146 (escaped from #130)
Discovered on SHA: a2cf4a3
Area: Governance
Severity: P1
Type: governance
Statement: main branch is completely unprotected — no PR requirement, no CI checks, no force-push prevention
Evidence: GitHub API shows no branch protection rules or rulesets on main
Impact: Anyone with push access can force-push, delete main, or merge without review
Disposition: FIX RC1
Owner issue: #146
Blocks: Release integrity
Decision/evidence: Pending — require PR, Rust checks, Frontend checks; no force push; no deletion
```

```
RISK R016
Source issue: #129 (corrected finding)
Discovered on SHA: ae2f738
Area: Query Execution
Severity: P1
Type: product-truth
Statement: Multi-statement execution in query editor is sequential stop-on-failure with partial writes, NOT atomic
Evidence: query_service.rs execute_multi() splits and executes sequentially; no wrapping transaction
Impact: Users may assume atomic behavior; earlier writes are committed even if later statements fail
Disposition: FIX RC1
Owner issue: #129
Blocks: Query execution semantics documentation
Decision/evidence: Product decision needed — accept current semantics (document clearly) or wrap in transaction
```

## Decision records

```
DECISION D001
Question: Should v0.1 support MySQL or other providers?
Source issues: #132
Decision: PostgreSQL + SQLite only for v0.1
Rationale: Scope management; these two providers cover the target audience
Effective from SHA/date: 65bbca3 / 2026-08-14
Impacts: Code, docs, release notes
Follow-up issues: None
Supersedes: None
```

```
DECISION D002
Question: What rendering path for large schemas (>200 tables)?
Source issues: #15, #17
Decision: Search-first entry UX with three-phase state machine (search → neighborhood → overview)
Rationale: Full layout of 1000+ tables blocks main thread; search-first keeps initial paint fast
Effective from SHA/date: 65bbca3 / 2026-08-14
Impacts: Code (er-diagram.tsx), tests
Follow-up issues: #18 (bounded rendering), #19 (safe LOD)
Supersedes: None
```

```
DECISION D003
Question: What renderer for large-schema overview?
Source issues: #15
Decision: Cytoscape for L/XL tier; React Flow for S/M tier
Rationale: Cytoscape handles 1000+ nodes at 60fps; React Flow degrades above ~200
Effective from SHA/date: 596f285 / 2026-08-14
Impacts: Code (lazy-loaded CytoscapeErView)
Follow-up issues: None
Supersedes: None
```

## Candidate invalidation log

No invalidation events recorded yet.

## Severity rules

### P0
Release/publication must stop immediately.

### P1
Blocks affected gate/release until fixed or scope removed.

### P2
Must be explicitly Fix/Accept/Defer before freeze.

### P3
Backlog/quality item; does not affect release decision.

## Completion criteria

This register closes when:
- P0 count = 0
- P1 count = 0
- All P2 have final disposition (FIX/ACCEPT/DEFER)
- No unresolved candidate-integrity events
- #111 can verify alignment

## Current summary

| Severity | Open | Fix RC1 | Accept RC1 | Defer | Resolved |
|---|---|---|---|---|---|
| P0 | 0 | 0 | 0 | 0 | 0 |
| P1 | 6 | 4 | 0 | 0 | 0 |
| P2 | 5 | 0 | 2 | 0 | 0 |

**Blockers to public release:** R001 (name), R004 (license), R007 (Gate 4), R008 (Gate 5), R011 (keyring), R012 (error boundary), R013 (TLS default), R014 (SQLite backup), R015 (branch protection), R016 (multi-statement semantics)
