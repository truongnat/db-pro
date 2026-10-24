---
schema_version: 1
id: "MEM-SRC-dc1d14a500e59204"
title: "Source mirror: docs/plans/active/workspace-rename-expanded-state/FINDINGS.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/workspace-rename-expanded-state/FINDINGS.md"
source_sha256: "dc1d14a500e59204c849024163f821138723be4893a518586e78042631275a5f"
mirror_body_sha256: "dc1d14a500e59204c849024163f821138723be4893a518586e78042631275a5f"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
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
