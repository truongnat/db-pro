---
schema_version: 1
id: "MEM-PLAN-workspace-rename-expanded-state-10368e44c8be"
title: "Plan archive: Workspace Rename Expanded State"
type: decision
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, decision, workspace-rename-expanded-state]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "10368e44c8be66c487a8cbce2273c8e2e91dca98b3c0fc46cb0363cef1ca7eba"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/workspace-rename-expanded-state/PLAN.md"
  - "docs/plans/active/workspace-rename-expanded-state/FINDINGS.md"
  - "docs/plans/active/workspace-rename-expanded-state/VERIFICATION.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/workspace-rename-expanded-state/`. Loại suy luận: `decision`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# #513 — Workspace rename remaps expanded state

## Goal
`rename_path` trên dir đang mở không làm collapse file tree: `expanded` phải remap prefix cũ → mới, refresh sau đó vẫn hiển thị node mới ở trạng thái mở.

## Scope
- `crates/app/src/files/ide_workspace.rs::rename_path` — remap `expanded` sau `fs::rename` thành công.
- `delete_path` — prune `expanded` entries của subtree bị xóa (stale entries cùng class, tránh rò rỉ phantom).
- Không đổi key scheme (`expanded` vẫn là plain `relative_path` như `scan_directory` sinh ra); không chuyển sang inode/guid (đề xuất trong issue là alternative, ngoài scope P2 này).

## Non-goals
- Multi-root qualified keys (`root-id::path` trong `expanded`) — hiện `expanded` là plain, tree view chỉ render primary root.
- Async/background scan remap — `rename_path` là sync path duy nhất đổi tên trên disk.

## Architecture
- `WorkspaceFileNode.relative_path`: plain, `/`-separated (`scan_directory`).
- `expanded: BTreeSet<String>`: plain keys, toggle qua `ToggleDirectory(relative_path)`.
- `rename_path(relative, new_name)`: chấp nhận qualified hoặc plain qua `root_and_relative_path`; tính `old_key` (plain) và `new_key` (plain dest strip prefix root); sau `fs::rename` OK thì remap mọi entry `== old_key` hoặc `starts_with(old_key + "/")` sang `new_key` (+ suffix), giữ boundary `/` để `src` không ăn `src2`.
- Entries qualified `id::suffix` (phòng thủ tương lai): remap phần suffix theo cùng quy tắc, giữ prefix id.

## Acceptance
- Rename `src` → `source` khi `expanded={src, src/sub1}` ⇒ `expanded={source, source/sub1}`, tree sau refresh match.
- Rename nested `src/sub1` → `src/renamed` ⇒ chỉ subtree đó remap, `src` và `src/other` giữ nguyên.
- Rename file không đụng `expanded` của dir cha.
- `src` rename không remap `src2` (boundary `/`).
- Delete dir prune subtree khỏi `expanded`.


## Source: `FINDINGS.md`

# #513 Findings

## Evidence (source, SHA `55dc35a4` lúc điều tra)
- `crates/app/src/files/ide_workspace.rs:274-301` (`rename_path`): `std::fs::rename(&absolute, &dest)` rồi `self.refresh()?`, không chạm `self.expanded`.
- `crates/app/src/files/files_tree_view.rs:89-91,100`: render iff `self.expanded.contains(&node.relative_path)`.
- `crates/app/src/files/ide_workspace_scan.rs:86-90`: `relative_path` là plain `/`-separated (strip prefix root).
- `crates/app/src/workspace/workspace_files_state.rs:104-108`: `toggle_directory` insert/remove plain key.
- `crates/app/src/files/ide_workspace_types.rs:177`: `expanded: BTreeSet<String>`.

## Failure scenario
1. Tree có `src/` (expanded) chứa `src/sub1/` (expanded).
2. User rename `src` → `source` (qua context action → `rename_path`).
3. Disk đổi tên OK, `refresh()` quét lại tree mới có `source`, `source/sub1`.
4. `expanded` vẫn `{src, src/sub1}` → `contains("source") == false` → cây collapse; user phải mở lại tay.

## Severity
P2 (theo issue): không mất dữ liệu, không sai mutation; gãy UX flow rename-dir-đang-mở. Không phải P0/P1.

## Scope decision
Nhỏ nhất coherent: remap prefix trong `rename_path` + prune trong `delete_path` (cùng class stale-`expanded`). Không đổi key scheme sang inode/guid.

## Provider impact
n/a — local filesystem workspace, không phải PostgreSQL/SQLite provider path.

## Runtime testability
Unit test với temp dir thật (`std::fs` + `open_root` + `expanded.insert` + `rename_path` + assert set). Không cần DB, không cần egui render.


## Source: `VERIFICATION.md`

# #513 Verification (append-only)

## Fail-before (2026-10-16, HEAD `55dc35a4` + repro test, chưa fix)
- `cargo test -p db-pro-app --lib repro_513` → FAILED:
  `expanded should remap src -> source, got {"src", "src/sub1"}`
  (`crates/app/src/files/tests/ide_workspace_tests.rs:151`)
- Chứng minh đúng failure scenario trong FINDINGS: `rename_path` fs::rename + refresh nhưng giữ `expanded` cũ.

## Pass-after (cùng máy, sau fix `remap_expanded_prefix` + `prune_expanded_subtree`)
- `cargo test -p db-pro-app --lib 513` → **5 passed / 0 failed**:
  `rename_dir_remaps_expanded_state_513`,
  `rename_nested_dir_remaps_only_its_subtree_513`,
  `rename_dir_does_not_touch_sibling_prefix_513`,
  `rename_file_leaves_parent_expanded_513`,
  `delete_dir_prunes_expanded_subtree_513`.
- `cargo test -p db-pro-app --lib ide_workspace` → **11 passed / 0 failed** (5 mới + 6 cũ: traversal, symlink, migrations, multi-root, notebook).
- `cargo test -p db-pro-app --lib files` → 3 passed (workspace_files_state).
- `cargo +nightly fmt --all -- --check` → exit 0.
- `cargo clippy -p db-pro-app --all-targets` → finish OK; 7 warnings đều ngoài file chạm (vd `schema_workbench_tests.rs` field_reassign); `grep ide_workspace` trong output = rỗng.
- `python3 tools/cc-scan.py crates/app/src/files/ide_workspace.rs crates/app/src/files/tests/ide_workspace_tests.rs --fail-on error` → score 100/100, error=0.
- `cargo check -p db-pro-app` → Finished OK.

## Chưa làm (khai báo trung thực)
- Không có UI runtime evidence (egui render tree sau rename) — cần owner verify trên build thật hoặc capture.
- Không chạy full `cargo test --workspace` (chỉ focused crate); không chạy `cargo check --workspace`.
- Issue #513 để OPEN chờ owner review/đóng.

