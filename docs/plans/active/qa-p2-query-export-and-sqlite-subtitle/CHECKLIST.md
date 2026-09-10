# Remediation Checklist: QA-P2-22 & QA-P2-21

## QA-P2-22: Query Export Enablement
- [ ] Add `hasResults?: boolean` to `QueryCommandBarProps` in `query-command-bar.tsx`
- [ ] Update Export Results menu item to `disabled={!hasResults}`
- [ ] Pass `hasResults={!!result}` from `query-tab-content.tsx`
- [ ] Add unit test verifying Export menu item enablement based on result existence

## QA-P2-21: SQLite Connection Subtitle
- [ ] Update `WelcomeView` connection subtitle rendering logic for SQLite driver
- [ ] Add unit test verifying SQLite connection displays database path without `:0 /`

## Quality Gates
- [ ] `cd frontend && npm run typecheck`
- [ ] `cd frontend && npm run lint`
- [ ] `cd frontend && npm run format:check`
- [ ] `cd frontend && npm run check:tokens`
- [ ] `cd frontend && npm run test`
- [ ] `cd frontend && npm run build`
- [ ] `cargo fmt --all -- --check`
- [ ] `cargo check --workspace`
- [ ] `cargo clippy --workspace --all-targets`
- [ ] `cargo test --workspace`
