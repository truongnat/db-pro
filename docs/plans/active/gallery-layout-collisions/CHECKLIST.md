# Checklist

- [x] Remove diagnostic `eprintln!` instrumentation (surfaces + database card)
- [x] `explain/ui.rs`: allocate stat region ≥ measured stat text width (+ header truncate + ScrollArea id_salt)
- [x] `database/ui.rs`: include `item_spacing.x` in the SSL gap reservation
- [x] `workspace/ui.rs`: bound left status items against right block + ellipsis
- [x] `component_gallery_rendering.rs`: theme-derived glyph color for alpha rows
- [x] Component scroll areas salted (`diff`, `dev_tools`, `explain`) — Id collisions resolved
- [x] Unit tests: `right_status_block_width`, `left_status_items_limit`
- [x] `cargo fmt --all -- --check`
- [x] `cargo check --workspace --offline`
- [x] `cargo test -p db-pro-ui --offline` — 949/0/0
- [x] `cargo clippy -p db-pro-ui --all-targets --offline -- -D warnings`
- [x] `bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci` — 14/2 warn/0 fail
- [x] `bash .skills/perf-audit/scripts/perf-scan.sh` — PASS (partial)
- [x] Native captures: `database-shell`, `devtools`, `rendering` @1280×800 dark+light + 1440/1920 clamped
- [x] Visual verification of all four fixes (+ Id-collision overlays gone)
- [x] FINDINGS.md + VERIFICATION.md + AGENT_EVIDENCE.md
- [ ] STATUS.md row — added in worktree; left for the pending `quick-open` staged commit to carry (avoids mixing scopes)
