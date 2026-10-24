---
schema_version: 1
id: "MEM-SRC-3da50197bea8d8a8"
title: "Source mirror: docs/plans/active/workspace-rename-expanded-state/VERIFICATION.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/workspace-rename-expanded-state/VERIFICATION.md"
source_sha256: "3da50197bea8d8a8dc3cac07e00f55b4e5ad5ec70951216a3a1667695ebf923a"
mirror_body_sha256: "3da50197bea8d8a8dc3cac07e00f55b4e5ad5ec70951216a3a1667695ebf923a"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
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
