# #484 Checklist

- [x] PLAN/FINDINGS/CHECKLIST/VERIFICATION scaffold
- [x] `MAX_EXPORT_ROWS` + `ensure_within_export_cap` in export_formats.rs
- [x] Enforcement in `execute_for_export` (seam where row data enters export)
- [x] Defense-in-depth checks in render_csv / render_json / render_excel
- [x] Tests: boundary (== cap ok, cap+1 rejected), renderer-level, service-level
- [x] `cargo test -p db-pro-core` PASS (487) + `cargo check` PASS
- [ ] Comment/close issue (awaiting owner decision)
