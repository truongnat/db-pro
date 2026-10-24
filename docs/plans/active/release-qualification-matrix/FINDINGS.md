# Findings — release qualification matrix planning

State: **PLANNING**. Source baseline for all code anchors: `dd2bf5e4090e6d509d34c0c8c58ca95015d91640`. Planning decisions, historical failures and unexecuted hypotheses are distinct. Không QA runtime test hoặc fix app trong lượt này.

## 1. Historical release blockers to retain, not re-prove by prose

- Prior [release assessment](../../../release/2026-10-24-release-readiness.md) records actual non-capture macOS release close → SIGABRT/NSRangeException; source/artifact/host and raw log in that evidence folder. LIF-02/03/04 address new-candidate regression/lifecycle. Không seed case PASS, không gọi capture-only hoặc count as new execution.
- Notice/license wording, tag/version mismatch, old candidate dossier and test guard-skip evidence are R-02..05 in assessment. REL-02/04, SEC-03, CON-07/09 and aggregation guard checks cover them.
- Prior current-SHA workspace/provider suite PASS hỗ trợ test-basis, **không qualify native mutation/auth/restore/platform cases trong matrix mới**. 12 MySQL harness passed included3 early-return guard cases; preserve that distinction.
- Assessment P0/P1 count means none identified within its scope, not exhaustive zero-defect guarantee or owner release approval.

## 2. Planning source discoveries and decisions

### P-01 — UI-storage/persistence identity needs explicit qualification

- Runtime data dir in `native-app/main.rs:171–184` is not proof of GUI/settings storage isolation; `main.rs:585–607` calls eframe with app identity `DB Pro` and creation_context.storage, while `app_lifecycle.rs:44–138` writes settings/docs/session layout through eframe Storage.
- Planning command `cargo +1.95.0 tree --locked -p db-pro-native -e features -i eframe` shows accesskit/default_fonts/glow/wayland/x11, **no persistence feature**. This is source/dependency evidence only. It raises a concrete persistence risk, not a fresh runtime diagnosis of all stored-state behavior.
- Prior audit's temp DB_PRO_DATA_DIR qualification proves runtime metadata isolation/SQLite connection identity only. It must **not be generalized to user GUI preferences or full workspace isolation/persistence**. New matrix requires OS sandbox/verified GUI-storage namespace and actual normal save/Quit/restart.
- No code toggle/fix added; LIF-04/06, FIL-01, SET-01/03, ER-02 measure actual behavior. If missing storage causes observable requirement failure in valid sandbox, report FAIL, not an infrastructure exemption.

### P-02 — Actual affordances differ from generic QA assumptions

- Query parameters are inline `Parameters · N` panel, not a guessed modal (`query_parameters_view.rs:13–94`). Values intentionally memory-only; GUI restart must clear parameter values rather than require their persistence.
- Native data edits need connected/writable table with complete PK; no-PK capability alone is not editable-grid support (`table_editor_context.rs:10–69`). Composite positive and no-PK/view negative oracles are separate.
- Table toolbar action is localized **Save**; dirty navigation offers **Apply Changes / Discard Changes / Cancel**. Conflict/retry must preserve unrelated pending edits (`table_mutation_actions.rs:409–567`).
- Schema Compare is snapshot/diff/migration preview+apply; Data Compare sync is preview-only (`schema_compare_view.rs:382–461`). Do not demand a nonexistent sync Apply control or silently replay SQL via CLI as UI PASS.
- Transfer real **Copy tables** uses actual connection-ID fields and open source/target handles. Nearby synthetic/export/import/cancel harness controls do not prove real database transfer (`transfer_activity_surface_view.rs:390–526`).
- Current copy string-binds values with first-column conflict key (`core/application/table_copy.rs:50–266`); baseline copy fixture uses first-column PK/text, typed copy boundary not silently equated with typed query fidelity.
- Backup/Restore real UI in Settings; SQLite restore requires disconnected writable target; PostgreSQL restore explicitly not atomic (`settings_backup_view.rs:22–170`, `core/application/backup_service.rs:39–151`, `infrastructure/backup/pg_dump.rs:125–143,316–341`).
- Saved Tasks scheduling is in-app and UI Schedule60s; no arbitrary interval/daemon or New export/maintenance task inferred from payload enum (`saved_tasks_surface_view.rs:57–302`). Builder Apply SQL → editor does not execute (`visual_query_builder_surface_view.rs:276–329`).

### P-03 — Fixture assets are not ready environments

Seed/count contracts in FIXTURES source exist; conflict second-writer, remote99/100/101, positive TLS/CA/mTLS, MySQL reserved-field special server, standalone admin, fault-controller and GUI storage/OS profile still need preparation. Auto-seeded sample is readonly, not suitable for writable positive cases. Missing fixture blocks case, not proof product defect.

Provision script reuse is not pristine reset: persistent DB volumes, host5432 topology assumptions, user known_hosts writes and prewritten PASS snippets. No worker may execute it blindly. New run manifests include server identity, owned namespace/sentinels and independently verified baseline/reset.

### P-04 — GUI concurrency and automation limitations

One desktop has shared focus/pointer/keyboard/AX state. Default schedule serializes all GUI lanes even with independent apps/DBs; parallel only independent desktops or noninterfering read-only tasks. AX success without observable state is not PASS; keyboard/pointer tests require actual modality.

Native close crash may make AX report permission/window errors afterward; independent process exit/stderr disambiguates. Do not global reset Orca/permissions or send unchecked keystrokes into another app. Host clamp is a geometry blocker, not passing a renamed target image.

## 3. Requirements applicability and matrix size

118 definitions contain105 functional and13 visual scenarios; full raw Cartesian candidate list is **3,663 instances** (963 functional,2,700 visual;1,221 per platform). These are planning cells, **not executed tests or promises every static sub-surface has each async state**. Lead preflight approves state/capability applicability with source proof and retains N_A records in denominator accounting. No host/permission/fixture absence removed from candidate set.

High cell count is auditable scope, not instruction to run every combination simultaneously. Dispatch can split ordered waves/independent hosts without losing original IDs or assertions. Do not hide remaining cells behind lane PASS. Actual execution may remain BLOCKED when platform/trust/OS/API prerequisites are unavailable; final report still lists them.

## 4. Open decisions / readiness blockers for future execution

| Item | Owner / required input | Execution effect |
|---|---|---|
| Execution authorization and candidate/channel | Repository owner | No QA run started by this planning request |
| Shipped providers/platforms/preview exceptions | Owner + QA Lead | Default matrix retained; no worker scope reduction |
| GUI/default-keyring/upgrade OS sandbox | Fixture owner + authorized OS admin | No global user state changes; relevant native cases BLOCKED until safe |
| Windows/Linux hosts + Orca support | QA Lead | Cross compile does not close runtime variants |
| Real TLS/SSH/fault/admin fixtures | Fixture owner | Explicit readiness vs BLOCKED, not fake generic localhost |
| AI key/endpoint/synthetic payload/spend | Owner of API account | No external call/billing without separate permission |
| Performance budgets proposed in RUNBOOK | Owner approves plan; lead freezes before sample | No retrospective threshold selection |
| Platform signing/unsigned channel policy | Release owner | Cannot manufacture certs/notarization/trust from local build |

These are prerequisites, not questions about repo-provided facts. Planning deliverable is complete when its integrity checks pass; product release/readiness is not changed by documentation.
