# Checklist: Favorite Toggle Optimistic Update Rollback (QA-P2-20)

- [ ] Implement `onError` callback in `useToggleFavorite`
- [ ] Add unit test in `connection-queries.test.tsx` for error rollback
- [ ] Pass frontend quality gates (`typecheck`, `lint`, `format:check`, `check:tokens`, `test`, `build`)
- [ ] Pass Rust quality gates (`cargo fmt`, `cargo check`, `cargo clippy`, `cargo test`)
