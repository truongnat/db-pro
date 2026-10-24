---
schema_version: 1
id: "MEM-SRC-d35ff1e128b36f78"
title: "Source mirror: docs/plans/active/workspace-rename-expanded-state/CHECKLIST.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/workspace-rename-expanded-state/CHECKLIST.md"
source_sha256: "d35ff1e128b36f78bd22f33dd1eab56cd84893ff078a83800e5e136f07bcbd90"
mirror_body_sha256: "d35ff1e128b36f78bd22f33dd1eab56cd84893ff078a83800e5e136f07bcbd90"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# #513 Checklist

- [x] PLAN/FINDINGS/CHECKLIST/VERIFICATION scaffold
- [x] Repro test (fail-before): rename dir collapses expanded
- [x] Fix `rename_path` remap prefix với boundary `/`
- [x] `delete_path` prune subtree
- [x] Unit tests: top-level rename, nested rename, file rename no-op, `src` vs `src2`, delete prune
- [x] `cargo test -p db-pro-app files::` PASS (+ ide_workspace 11/11)
- [x] `cargo fmt --check` / clippy focused PASS (không warning mới)
- [ ] `gh issue close` hay comment evidence (chờ owner quyết định — mặc định để OPEN + comment)
