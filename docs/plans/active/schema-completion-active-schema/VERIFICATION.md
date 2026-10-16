# #511 Verification (append-only)

## Fail-before (chưa fix, đã có test mới)
- `cargo test -p db-pro-ui --lib 511` → **2 failed / 1 passed**:
  - `completion_prefers_active_schema_for_duplicate_table_511` FAILED —
    gợi ý `["id", "tenant_a_only", ...]`, thiếu `tenant_b_only` (đúng bug issue).
  - `ai_context_prefers_active_schema_for_duplicate_table_511` FAILED (cùng nguyên nhân).
  - `completion_falls_back_when_active_schema_has_no_match_511` passed (guard giữ hành vi cũ).

## Pass-after (sau helper `find_completion_table` + 3 call-site)
- `cargo test -p db-pro-ui --lib 511` → **3/3 PASS**.
- `cargo test -p db-pro-ui --lib query` → **89 passed / 0 failed**.
- `cargo +nightly fmt --all -- --check` → exit 0 (đã `fmt` 1 lần cho `if let` mới).
- `cargo clippy -p db-pro-ui --all-targets` → không warning nào trong `schema_completion`.
- `python3 tools/cc-scan.py schema_completion.rs schema_completion_tests.rs --fail-on error` → **100/100, error=0**.
- `cargo check -p db-pro-ui` → Finished OK.

## Chưa làm (khai báo trung thực)
- `consume_qualified` drop schema prefix (`completion_scope.rs:380-391`) giữ nguyên —
  `FROM tenant_b.users` vẫn resolve qua bare name + active_schema; qualified-ref
  triệt để là follow-up riêng.
- Không chạy full `cargo test --workspace`; không có screenshot dropdown completion.
- Issue #511 để OPEN chờ owner quyết định.
