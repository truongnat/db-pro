---
schema_version: 1
id: "MEM-SRC-6843b80d1b9ae9ac"
title: "Source mirror: docs/plans/active/bsn-real-data-native-audit/evidence/runtime/MANIFEST.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/bsn-real-data-native-audit/evidence/runtime/MANIFEST.md"
source_sha256: "6843b80d1b9ae9ac3ca93b54cbc8656c53b6ebf3034c445d2b30c9b92a7225e0"
mirror_body_sha256: "6843b80d1b9ae9ac3ca93b54cbc8656c53b6ebf3034c445d2b30c9b92a7225e0"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Live BSN Runtime Evidence

- Provider/profile: saved `BSN` PostgreSQL profile; profile is configured writable (`readonly=false`), therefore the manual session performed no DML, DDL, export, transfer, administrative action, or settings change.
- Source baseline: `cb6e00d633772de538498384a8b14d32d25f7915` plus pre-existing uncommitted changes. No product source/test fix remains from this audit.
- Initial live failure captures used `/data/cargo-target/release/db-pro-native` SHA-256 `b819ca8375973c8429a73e8795501241e9aadbd1d345f5953d10238b5abafbc1`.
- Final-source query/focus captures used the rebuilt `/data/cargo-target/release/db-pro-native` SHA-256 `45314f7aebddae65e2c01e8a3f227c7e9d79faa41378722a1103798c6241d2e8`. An intermediate experiment build was overwritten and was not used for captures.
- Runtime: native DB Pro window on Xvfb `:102`; `DB_PRO_DATA_DIR=/home/vietis/.local/share/db-pro`, `DB_PRO_DISABLE_KEYRING=1`, `DB_PRO_WINDOW_SIZE=1280x800`, X11 backend. Actual captured framebuffer: 1280×800.
- Safe interaction: connected to the saved BSN profile; expanded the `master` schema; opened `master.m_all_bank` (empty state); navigated to `master.m_article` and observed a query error referring to the prior table’s `all_bank_cd` column. On the final-source run, typed and executed only `SELECT 1` via Ctrl+Return; one integer row appeared. A Tab-focus screenshot records one visible focus state; comprehensive traversal was not established. No row values were retained. No mutating UI action was activated.

| Capture | SHA-256 | Provenance / state |
|---|---|---|
| `bsn-welcome-light-1280x800.png` | `8331c61e59d55c63150e61d67f82427b0b47bf22b0532a2ae3a064492e3540f8` | LIVE; connected status and schema counts, Welcome surface |
| `bsn-explorer-master-expanded-light-1280x800.png` | `13f5ba89398e79c55ecb3d70d498667efea79c4a999cab4c295cf9c0d4feb001` | LIVE; expanded BSN master schema, table tree, selected table |
| `bsn-master-empty-light-1280x800.png` | `859aab53fd09ff6032a30a168796d72833431cb9fe843ef6255188d55538dea1` | LIVE; empty `m_all_bank` Table Data state |
| `bsn-table-switch-error-light-1280x800.png` | `fb3464ecf70fdfa22cd7f531f9e6f47c9237d4ff076dd05e1f382a715009537a` | LIVE; `m_article` view with stale-column query failure; no customer rows shown |
| `bsn-query-select1-result-light-1280x800.png` | `e1d8e7a983e1aee182a843b8c401b0fb7f589bd7d575a25089c9865c7cdf0375` | LIVE on final-source binary; safe `SELECT 1`, single integer result; no sensitive rows |
| `bsn-query-focus-tab-light-1280x800.png` | `5e8035cf554dd00ad81530db1e4105bddb771d77168c74cac348658ecc9f5096` | LIVE on final-source binary; one visible shell focus-ring state only |
| `bsn-table-structure-master-light-1280x800.png` | `f2cf91de245e02ce9c55aefdb37fc102b7baffce1a7454c0bd1736d52572c3e4` | LIVE metadata; 13 columns, PK, data-type truncation visible; empty of row data |
| `bsn-table-structure-type-tooltip-light-1280x800.png` | `bd0a272548cebfadedfbd4a61f8e90f265eec50336fe758bc94606a13860bbdf` | LIVE metadata; hover reveals one full PostgreSQL type |
| `bsn-table-indexes-master-light-1280x800.png` | `3bb2b5861f2af66e497de1de2749164fe434226e960c264ad8b2ed5e043306e2` | LIVE metadata; one primary-key btree index |
| `bsn-table-fks-empty-master-light-1280x800.png` | `88aed8e33732c0e015a09270b0c4a47b2c3a9bfb8081fa41b6fdc98118c9f651` | LIVE metadata; zero outgoing foreign keys, empty-state message |
| `bsn-er-diagram-filtered-search-blank-canvas-light-1280x800.png` | `f885310777b813e91fd3c9b6e8097bcdb0cc14ae8c1c9a18038328a7c18e946b` | LIVE BSN 1,302-table schema; search badge says 5 tables, canvas blank after 1-hop, 100% zoom, five-second wait |

## Additional continued captures

- The full-screen captures below were taken from the final-source binary SHA-256 `45314f7aebddae65e2c01e8a3f227c7e9d79faa41378722a1103798c6241d2e8`, on Xvfb `:2`, Light mode, 1280×800. Capture method: `DISPLAY=:2 import -window 0x200004 <path>` followed by privacy-safe ImageMagick crops where noted. Filenames retain the requested 1280×800 viewport; crop dimensions and SHA-256 are recorded per artifact. No customer row values, private saved-query contents, or endpoint/credential data were retained.

| Capture | SHA-256 | Provenance / state |
|---|---|---|
| `bsn-tools-hub-live-light-1280x800.png` | `40c397a7c119b2ab445205274571bc7648fc34ff7de40fd7ff9aa22811ec1765` | LIVE BSN; full 1280×800 top Tools hub and Welcome workspace; status still says `Security · 18 role(s)` after navigation |
| `bsn-tools-hub-lower-live-light-1280x800.png` | `ebcf23c7f2b9b3ef65d3d5c855adb291e12f15a9b59574eae313fa67292df8a9` | LIVE BSN; full 1280×800 scrolled Tools hub; Transfer & Migration, Saved Tasks & Backups, Diagnostics & Problems entries visible |
| `bsn-schema-workbench-default-live-light-1280x800.png` | `1e18e486ed63e689b8869371a36f96303824cb4da41eb6e22fca3e22b158b859` | LIVE BSN; default Schema Workbench; no DDL generated or applied |
| `bsn-history-empty-live-light-1280x800.png` | `9e2c36ef9b653c933813d76bcbffc3a2983f6edd35484c1ff61e2dfc77a53dc5` | LIVE BSN; empty Execution History; no SQL or query text shown |
| `bsn-settings-general-live-light-1280x800.png` | `99b9b63c73d43b198e766351d8465d8cb514aead777dc8dcf4b15c2a517d72a0` | LIVE BSN; General Settings language and toggles inspected; no values changed |
| `bsn-status-stale-success-light-1280x800.png` | `81474ace27443c3d6178400658870730d1f10279c20bae92fc4e2cc215dc4225` | LIVE BSN; full 1280×800 statusbar after Security metadata refresh; redacted duplicate view-state evidence for stale activity feedback |
| `bsn-monitor-health-list-crop-live-light-1280x800.png` | `2e41caa067504a9593e68d51af760ba5ab9f2adc6e6744553fba319c4f40f1a5` | LIVE BSN; actual crop 320×770 from 1280×800; shows 8-finding snapshot and two warning headings; relation names/evidence details masked; Auto-refresh unchecked |
| `bsn-security-role-drop-one-row-crop-live-light-1280x800.png` | `974caca7796158a932f18338983c93c81bd7015cd2c4647df3b2aa44575a45e7` | LIVE BSN; actual crop 310×207 from 1280×800 after role refresh; one role name masked, destructive Drop button visible; no action activated |
| `bsn-agent-offline-panel-crop-live-light-1280x800.png` | `54cdf3226d92e459a32540dcf009f05d4268c094b2fcd4d47b033fafeec8e89b` | LIVE BSN; actual crop 350×720 from 1280×800; Agent offline-draft/provider-unconfigured state; connection/schema/table context masked; no provider call |
| `bsn-focus-forward-button-crop-live-light-1280x800.png` | `ecccf4febdfc22efd9e04c5cec83412ecf562bd4c44da863c104014da7a3732e` | LIVE BSN; actual crop 160×40 from 1280×800; focus treatment on shell forward navigation button after three Tab presses |
| `statusbar-overflow-crop-synthetic-light-1280x800.png` | `4e01bc243558d6772bf7961c0aa288daebc10e96a645ac72ebc5d58d28e21cf2` | SYNTH presentation fixture; actual crop 1280×28 from a 1280×800 SQLite Results screen; fixture identity/data masked; `0 functions` touches `SQL Editor` |
| `bsn-schema-compare-empty-live-light-1280x800.png` | `3043af6bacad6874bcdc5ab6d5d0895b1a0f9aa78ca48a6938cdfd6ad65e553d` | LIVE BSN; 1280×680 crop; Schema Compare empty state, no snapshot/diff or apply action |
| `bsn-workbench-view-actions-crop-live-light-1280x800.png` | `d9a547c6ce1a076cf44a1a7533f0d9b9705257f4172baef580949ed842363b9d` | LIVE BSN; 965×80 crop; Schema Workbench Views action row clips Aggregate Summary at the right edge; no preset activated |
| `bsn-query-history-pane-live-light-1280x800.png` | `3177ee0c229a8fff1abe2d8f85281983172acaf4ac8d994eeba71e78e9f8a74a` | LIVE BSN; 960×680 crop; safe `SELECT 1` query result and in-session History entry only |
| `bsn-transfer-seed-action-live-light-1280x800.png` | `0d832ed211110945c2b8c66243961035d036b7fdb4e726ea361e84d176ccab1c` | LIVE BSN; 320×600 crop; synthetic table seed and active Apply INSERT button shown; no table selected, no action, Production checkbox absent on connected profile |
| `bsn-saved-tasks-empty-live-light-1280x800.png` | `1aff4a11f12fab57fd84335eaea7bacccac55229417e89a13934f9d6d236d224` | LIVE BSN; 300×650 crop; no saved tasks; create and backup-task controls visible; no task saved |
| `bsn-sql-task-draft-live-light-1280x800.png` | `e14d428d4db89c719c78cf7c174db0135e486c1d0f43a8e87b62b765cad4b280` | LIVE BSN; 300×430 crop; unsaved SQL task draft defaulting to `SELECT 1`; Save not activated |
| `bsn-backup-task-draft-live-light-1280x800.png` | `fe95e8036c23d965fd2b4b2925466c80848e6b31ee35edc456189a8f59f37838` | LIVE BSN; 300×450 crop; unsaved backup-task form; no output path supplied or task saved |
| `bsn-problems-empty-live-light-1280x800.png` | `b99b796f136dec8c10cfd6f1ba7c15fab891ddddff3f93293c8d7218465e7e46` | LIVE BSN; 300×360 crop; Problems activity reports 0 errors/0 warnings and no open-document problems |
| `bsn-files-empty-live-light-1280x800.png` | `1f0c29d3df6fb915445a0eac261252e23c850bab61db83e26955cf2de482e6ea` | LIVE BSN; 300×360 crop; Files empty state; no folder opened |
| `bsn-data-sidebar-empty-live-light-1280x800.png` | `ded3763db22edb633fe2e5169796d5b54f8d7c47c9fd6130828c612e28117153` | LIVE BSN; 300×360 crop; Data sidebar reports 0 pinned and 0 recent tables; no data pinned |
| `bsn-history-activity-empty-live-light-1280x800.png` | `50248b56ee35c436b756b86a1e438764685dab3c351b66a7ecc7cb70cafb4997` | LIVE BSN; 300×360 crop; History activity empty state captured before the safe query; query workbench History was later populated by `SELECT 1` |
- All thirty-three retained PNGs were visually inspected at native resolution: eleven initial full-window BSN captures, six additional full-window captures, fifteen privacy-redacted live crops, and one explicitly synthetic statusbar crop. Captures cover live BSN Light-mode 1280×800 plus cropped regions. Dark mode, other full viewport sizes, full app-wide keyboard traversal and remaining feature states are NOT VERIFIED. The table-switch failure screenshots predate the audit’s temporary source experiment; the final source remains unfixed.
