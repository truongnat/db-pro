---
schema_version: 1
id: "MEM-PLAN-repository-folder-mapping-ef898351641d"
title: "Plan archive: Repository Folder Mapping"
type: decision
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, decision, repository-folder-mapping]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "ef898351641dce981ed8f3e96492496c93bf2161b927fc8b7fcc34e29de762ee"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/completed/repository-folder-mapping/PLAN.md"
  - "docs/plans/completed/repository-folder-mapping/FINDINGS.md"
  - "docs/plans/completed/repository-folder-mapping/VERIFICATION.md"
  - "docs/plans/completed/repository-folder-mapping/AGENT_EVIDENCE.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/completed/repository-folder-mapping/`. Loại suy luận: `decision`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# Repository Folder Mapping Implementation Plan

## State
- State: COMPLETED
- Branch: main (owner workflow override; no worktree)
- Baseline SHA: be2e431b07a8fcfbf39bf16fa1deefd600273633
- Verified source SHA: a1730a3e7156dc75fffa219f5f6f9eb286542e4c

## Goal
Organize source, tests, documentation and retained evidence by ownership; keep all product behavior and public Rust module paths unchanged.

## Architecture
Retain the six workspace crates and established feature/component directories. Move misplaced test bodies into adjacent tests/<source-stem>_tests.rs files with explicit #[path] declarations. Group only cohesive implementation files whose ownership is already clear. Keep active plans, historical release planning, architecture references, design assets and audit reports distinct.

## Scope and mapping rules
- Source: inspect every crate; relocate misplaced tests and cohesive source families, preserving logical module names, visibility and public exports. Update relative include and module paths.
- Root documentation: put reports with their audit/quality owner; design references under docs/design; original interlinked project/release plans under docs/plans/project.
- tasks/plan.md and tasks/todo.md: retain contents and acceptance states under docs/plans/project/core-component-standardization; do not infer completion or invent an active lifecycle transition from relocation.
- Design assets: group HTML mockups and Stitch downloads under docs/design; move retained screenshots beside their owning plan.
- Exact file-to-folder mapping is recorded in this directory before implementation.
- Update incoming references and relative links inside moved files. Preserve historical claims and SHA provenance.

## Non-goals
No product behavior/UI redesign, API rename, crate/layer redesign, data deletion, dependency changes, database operations or remediation of unrelated audit findings. Do not move Cargo/config/tool-discovery files, instruction files, agent skills, review infrastructure, build outputs, secrets or user databases. Keep canonical lifecycle/status files and required entry-point documents at their established paths.

## Execution
1. Inventory misplaced files; record exact old -> new mapping and ownership.
2. Move source/tests and documentation in independent owned slices; update references without compatibility shims or duplicate files.
3. Integrate source-path references in docs/scripts, update repository navigation, and check all mapped destinations and links.
4. Run formatting, workspace check/clippy/tests, UI architecture check and locked native release build. Smoke the native app to detect runtime asset/path regressions. No new behavior tests for pure filesystem moves.

## Acceptance
- [x] Every applied move has a mapping row; no destination collision or silent content loss.
- [x] No mapped old file remains as a duplicate/shim.
- [x] Logical Rust module/public paths and test inventory are preserved.
- [x] Moved documents retain content and working relative/incoming links.
- [x] Root navigation explains canonical folder ownership.
- [x] Gates and actual smoke evidence recorded in VERIFICATION.md.

## Provider impact
PostgreSQL and SQLite implementation behavior is unchanged. Workspace tests ran with ignored provider tests reported separately. The actual native release smoke seeded/connected/introspected a new temporary SQLite fixture only; PostgreSQL was not independently runtime-tested.

## Review focus
Relative include_str/include_bytes paths; #[path] resolution; script path assumptions; moved Markdown links and JSON inventories; historical records accidentally relabeled completed.


## Source: `FINDINGS.md`

# Repository Folder Mapping Findings

Baseline: be2e431b07a8fcfbf39bf16fa1deefd600273633.

## Evidence and decisions
- Root plans/ contains eight interlinked documents. Its index calls 00, 01 and 07 current release references; do not label the entire collection obsolete. Move together to docs/plans/project, not archived/completed.
- Root tasks/plan.md and tasks/todo.md both describe core-component-standardization, but have inconsistent historical acceptance states. Move together under docs/plans/project/core-component-standardization without merging them into legacy-component-sunset or declaring completion.
- QA_AUDIT_REPORT.md belongs to the active bsn-real-data-native-audit plan. Retain its findings and scope as a historical revision-pinned report.
- open-api-style.md is a UI design specification despite its filename; open-ai-refer.md is a component reference. Both belong under docs/design/reference.
- docs/stitch_screens/inventory.json embeds repository-root file paths; update all 19 entries when moving exports under docs/design.
- Loose query-workspace-zed-shell screenshots have an existing completed-plan owner and non-colliding filenames; move there without deleting existing captures.
- Keep docs/10-egui-native-migration-plan.md and docs/DESIGN.md at their established entry-point paths. Preserve raw historical command transcripts and SHA-qualified old-source provenance; only repair live navigation/references.
- Rust language server: configured rust-analyzer failed to start (server exited code 0) on capabilities request. Mechanical path moves must use explicit module/include path inspection and compiler/test verification; do not rename logical symbols.

## Severity
P0/P1: none identified in the organization task. Existing product findings are out of scope and are neither resolved nor downgraded by moving reports.

## Known inherited navigation debt
The plan registry refers to active/query-workspace-zed-shell although its directory is completed/query-workspace-zed-shell. This move does not change lifecycle state; navigation should point to the actual directory without asserting new runtime evidence.


## Source: `VERIFICATION.md`

# Repository Folder Mapping Verification

## Source revisions
- Baseline: `be2e431b07a8fcfbf39bf16fa1deefd600273633`.
- Verified source relocation commit: `a1730a3e7156dc75fffa219f5f6f9eb286542e4c`.
- Documentation relocation is committed separately; no Rust logic/API/UI rendering changes.

## Conservation and mapping
- All **93/93** mapped destinations exist; no mapped old file remains and no destination collides.
- Source: **40 moves**, **35 byte-identical**. Five test files differ only in relative locale/child-test paths or updated location comments.
- Documentation/evidence: **53 moves**, **48 byte-identical**. Five moved text files include navigation/inventory updates.
- All **27 moved PNG/HTML assets** retain their pre-move SHA-256.
- All **19 Stitch inventory entries** resolve to existing files.
- Pre-move SHA-256 and byte sizes: `MAPPING.json`; human navigation: `MAPPING.md`.
- Empty old source/doc directories were removed only after files were conserved.

## Executed gates
The following ran as one direct `&&` chain, with no output-filter pipeline masking exit statuses. Chain exit: **0**. Full raw output: `evidence/rust-gates.txt`.

| Command | Observed result |
|---|---|
| `cargo +nightly fmt --all -- --check` | PASS |
| `cargo check --workspace` | PASS |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS, no warnings |
| `cargo test --workspace` | **2,262 passed / 0 failed / 65 ignored**, 27 result suites |
| `cargo build --release --locked -p db-pro-native` | PASS |

Separate gate chain, exit **0**:
- `bash scripts/check-ui-architecture.sh`: PASS.
- `bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci`: **12 pass / 4 warning categories / 0 failures**. Warnings concern existing parameter count, long functions/files and clone counts in relocated code; no code was added to suppress them.

Rust-analyzer capabilities probing failed to start the configured server (exit code 0); no successful LSP refactor is claimed. Logical symbols were not renamed; explicit path changes were compiler/test verified.

## Actual release runtime smoke
Launched the newly built release binary with:

```text
DB_PRO_DATA_DIR=/private/tmp/dbpro-folder-map-smoke.rOX3nw
DB_PRO_DISABLE_KEYRING=1
DB_PRO_WINDOW_SIZE=1280x800
./target/release/db-pro-native
```

- App PID 836, Orca window 91998; actual window 1280×832 including titlebar; screenshots 2560×1664 at 2×.
- Connected to the **new temporary Sample E-Commerce SQLite fixture**. Worker logs and rendered UI show schema introspection completed: **11 tables / 2 views / 1 trigger / 0 functions**.
- This exercises the relocated native command/event translation and runtime worker → SQLite → introspection → refreshed UI path.
- Inspected `evidence/release-smoke-light.png`, then activated the identified theme button and inspected `evidence/release-smoke-dark.png`. Both show the native Welcome shell and loaded SQLite schema with fonts/icons present. No UI redesign or full visual acceptance matrix is claimed.
- Service readiness regex `capture: resolved window size override` timed out; this was not treated as smoke success. Subsequent actual OS window/accessibility snapshots and inspected screenshots established rendering and runtime completion.
- Stopped only the smoke process through its named service, not other applications.

### Preference/storage safety check
`DB_PRO_DATA_DIR` isolates runtime data, not eframe preferences by itself. This was checked before deleting the temporary fixture:
- `cargo tree --locked -p db-pro-native -e features -i eframe` lists only accesskit/default_fonts/glow/wayland/x11; **eframe persistence is not enabled in this binary**.
- eframe 0.36.2 `native/epi_integration.rs:130-145` returns no storage when the persistence feature is absent. The `save` implementation therefore has no eframe file store to write in this launch.
- If enabled with app title `DB Pro`, `native/file_storage.rs:32-35` calculates `~/Library/Application Support/DB-Pro/app.ron`; that file was confirmed absent after the smoke. It is distinct from runtime's `DB Pro` directory.
- No user-preference restoration or deletion was performed. Runtime smoke files were confined to the temporary directory above.

### Retained hashes
```text
480bf463f880cbafcf518811acd004d34e90a1ece605d1f910e74d06bcf77ffd  target/release/db-pro-native
0b43107e8fd880b8a4c3634badf0b793f51143c8fab37b1097508b0ba4f65a9b  evidence/release-smoke-dark.png
714207ed3adfcf1c7a77be1278105f8341f3d3da6cc1b635cb62cf87a2d2df4f  evidence/release-smoke-light.png
```

## Provider matrix
| Provider | Implementation effect | Runtime evidence in this cleanup |
|---|---|---|
| SQLite | Unchanged | Native release connection/schema refresh on newly seeded temporary fixture |
| PostgreSQL | Unchanged | Not independently exercised; ignored provider tests are not treated as runtime passes |

## Independent review and inherited limitations
Two separate read-only review lanes accepted source and documentation mappings: introduced **P0=0 / P1=0 / P2=0**. See `REVIEW.md`. No remote CI, push, PR or VPS/Kilo approval was performed.

Inherited missing docs/01–06 references and the registry's old query-workspace-zed-shell path remain documented rather than silently changing historical plan states. Revision-pinned old source citations and raw command transcripts intentionally retain their historical paths; consult the mapping for present-day locations.


## Source: `AGENT_EVIDENCE.md`

# Agent Evidence — Repository Folder Mapping

## 1. Claim
| Field | Value |
|---|---|
| Agent identity | omp implementer; independent source/doc review lanes |
| Issue(s) | User-requested repo-wide folder/file organization |
| Task state | Done |
| Baseline SHA | be2e431b07a8fcfbf39bf16fa1deefd600273633 |
| Verified source SHA | a1730a3e7156dc75fffa219f5f6f9eb286542e4c |
| Branch / PR | main under owner override; no PR/push |
| Scope interpretation | Physical source/test moves and ownership-based documentation/evidence relocation, preserving Rust public paths, behavior and historical acceptance states. |
| Out of scope | UI redesign, logic/API renames, dependency/config changes, instruction/skills/review infrastructure, existing audit defects, user databases. |

## 2. Progress checkpoint
- Source commit: a1730a3e7156dc75fffa219f5f6f9eb286542e4c.
- Completed: all 93 mapped moves, incoming/relative path repairs, 19 Stitch inventory entries, root navigation, asset conservation, Rust gates, actual release smoke and independent review.
- Remaining task acceptance rows: none.
- Introduced findings: P0=0 / P1=0 / P2=0.
- Inherited navigation debt: absent docs/01–06 documents and an old STATUS plan path; recorded, not disguised as fixed/completed product work.

## 3. Implementation handoff / review request
| Field | Value |
|---|---|
| Exact source SHA | a1730a3e7156dc75fffa219f5f6f9eb286542e4c |
| Commit list | a1730a3e — colocate Rust source families and adjacent unit tests; separate documentation commit contains report relocation and this handoff. |
| File / surface inventory | MAPPING.md lists all 40 source + 53 documentation/evidence moves; MAPPING.json records exact pre-move SHA-256/byte sizes. Module declaration owners update only physical #[path]/include locations. README, CHANGELOG and STATUS update navigation/ownership. |
| Acceptance mapping | Conservation → 93 destination checks and 27 unchanged asset hashes; Rust module/test preservation → fmt/check/clippy/tests/release; doc navigation → independent doc review and 19 inventory resolutions; runtime → inspected release Light/Dark captures with fresh SQLite schema. |
| Commands and counts | Direct fmt/check/clippy/test/release chain exit 0; workspace 2,262 passed / 0 failed / 65 ignored, 27 suites. UI architecture exit 0; clean-code ratchet exit 0, 12 pass / 4 inherited warning categories / 0 failures. Raw Rust output retained under evidence/. |
| CI run IDs / status | Not run; no push/PR. |
| Known limitations | Historical raw transcripts/SHA-qualified citations intentionally keep old paths. PostgreSQL not independently runtime exercised; skipped tests are not provider passes. |
| Migrations / config implications | Internal physical repository paths changed according to manifest; no runtime schema migration, config/key rename, public Rust namespace change or compatibility shim. |
| Out-of-scope changes | No product behavior, SQL/policy, theme/layout or unrelated audit remediation. |

## 4. Review outcome
- Reviewed baseline: be2e431b07a8fcfbf39bf16fa1deefd600273633 plus uncommitted patch subsequently committed as source SHA above.
- Source lane: ACCEPT. Documentation lane: ACCEPT.
- Introduced P0/P1/P2: 0/0/0. Independent review details and inherited navigation debt: REVIEW.md.
- Initial source-review child-test path concern was withdrawn after exact current declarations were checked; no code edit was required.
- These local independent reviews are not remote VPS/Kilo PR approval.

## 5. Research / audit handoff
- References: PLAN.md, MAPPING.md/json, REVIEW.md, VERIFICATION.md, evidence/rust-gates.txt and two retained release screenshots.
- Observed: all six crates inspected; 93 file mappings resolved; 27 moved PNG/HTML assets byte-identical; actual release connected/introspected a fresh temporary SQLite fixture and rendered Light/Dark.
- Native preference safety checked: DB_PRO_DATA_DIR isolates runtime data only, but eframe persistence is disabled in the verified binary (cargo tree and eframe dependency source); computed DB-Pro/app.ron absent. No user preference restore/delete was performed.
- No speculative inference or product audit finding is claimed resolved by relocation.
- Downstream tasks activated: none.

## 6. Tổng kết (Vietnamese summary)
Đã sắp xếp 93 file đúng folder sở hữu, cập nhật path/module/link và giữ nguyên behavior/API. Gate Rust, 2.262 tests, release build và smoke SQLite trên app thật đều đạt; hai reviewer chấp nhận. Mapping đầy đủ và evidence được giữ trong plan; không đổi trạng thái các plan lịch sử hay sửa lỗi sản phẩm ngoài phạm vi.

