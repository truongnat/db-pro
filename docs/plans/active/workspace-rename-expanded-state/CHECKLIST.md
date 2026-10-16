# #513 Checklist

- [x] PLAN/FINDINGS/CHECKLIST/VERIFICATION scaffold
- [x] Repro test (fail-before): rename dir collapses expanded
- [x] Fix `rename_path` remap prefix với boundary `/`
- [x] `delete_path` prune subtree
- [x] Unit tests: top-level rename, nested rename, file rename no-op, `src` vs `src2`, delete prune
- [x] `cargo test -p db-pro-app files::` PASS (+ ide_workspace 11/11)
- [x] `cargo fmt --check` / clippy focused PASS (không warning mới)
- [ ] `gh issue close` hay comment evidence (chờ owner quyết định — mặc định để OPEN + comment)
