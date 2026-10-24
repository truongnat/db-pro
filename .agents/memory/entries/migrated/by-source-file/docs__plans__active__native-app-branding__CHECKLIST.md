---
schema_version: 1
id: "MEM-SRC-6163498a1058c120"
title: "Source mirror: docs/plans/active/native-app-branding/CHECKLIST.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/native-app-branding/CHECKLIST.md"
source_sha256: "6163498a1058c12062463dc249ffc4bd2281cbb662bbc96dfdaac87333cd81c8"
mirror_body_sha256: "6163498a1058c12062463dc249ffc4bd2281cbb662bbc96dfdaac87333cd81c8"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Native App Branding — Checklist

## Assets
- [x] Copy supplied logo into a writable canonical PNG asset.
- [x] Produce macOS `.icns` asset.
- [x] Produce Windows `.ico` asset.
- [x] Produce Linux PNG and `.desktop` metadata.

## Native integration
- [x] Embed PNG through `ViewportBuilder::with_icon`.
- [x] Add square/decoding regression test.
- [ ] Verify packaged bundle metadata when installer pipeline exists.

## Verification
- [x] `cargo fmt --all -- --check`
- [x] `cargo test -p db-pro-native` (22 passed; binary package has no library target)
- [x] `cargo check --workspace`
- [x] `cargo clippy --workspace --all-targets -- -D warnings`
- [x] `cargo test --workspace` (all runnable tests passed; expected provider/SSH fixtures ignored)
- [x] `cargo build --release --locked -p db-pro-native`
- [x] Launch rebuilt native app; runtime started SQLite demo connection and introspection successfully.

## Tổng kết bằng tiếng Việt
Đã chuẩn bị asset và wiring runtime; còn cần chạy toàn bộ gate sau thay đổi logo và xác nhận bundle metadata khi repo có installer pipeline.
