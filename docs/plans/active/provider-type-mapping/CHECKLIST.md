# Checklist: QA-P1-14 PostgreSQL Type Mapping & Decoding

- [x] Create plan documentation (`PLAN.md`, `CHECKLIST.md`, `FINDINGS.md`, `VERIFICATION.md`)
- [x] Create feature branch `fix/provider-type-mapping`
- [x] Expand type mapping in `crates/infrastructure/src/postgres/query_mapper.rs`
- [x] Add graceful cell-level fallback in `map_row`
- [x] Add unit tests in `crates/infrastructure/src/postgres/query_mapper.rs` for `map_row` & decoding fallback
- [x] Run Rust quality gates (`cargo fmt`, `cargo check`, `cargo clippy`, `cargo test`)
- [x] Run Frontend quality gates (`npm run typecheck`, `npm run lint`, `npm run format:check`, `npm run check:tokens`, `npm run test`, `npm run build`)
- [x] Complete pre-commit steps
- [ ] Submit Pull Request
