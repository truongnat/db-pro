---
schema_version: 1
id: "MEM-20261024-release-compiler-pin"
title: "Release qualification bắt buộc rustc pin 1.95.0 — không dùng cargo +stable"
type: environment
scope: project
status: active
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "2026-10-24"
confidence: high
tags: [release, rqm, rustc, toolchain, macos-arm64]
applies_to:
  repo: "db-pro"
  stack: "rust / db-pro-native"
  versions: "RQM PLAN pin 1.95.0; formatting nightly theo repo"
sources:
  - "docs/plans/active/release-qualification-matrix/PLAN.md"
  - "docs/plans/active/release-qualification-matrix/runs/rqm-20261024T104824Z/coordinator/RUN_INVALIDATION.md"
related:
  - "docs/plans/active/release-qualification-matrix/runs/rqm-next-pin195/preflight.json"
supersedes: null
---

# Tóm tắt

Artifact build với `cargo +stable` (rustc 1.99.0) **không** đủ điều kiện `role: release` trong Release Qualification Matrix. Qualifying build phải dùng **`cargo +1.95.0`** và ghi compiler thực tế trong manifest/ENV-01.

## Khi nào áp dụng

- Chuẩn bị package macOS/windows/linux cho RQM, ENV-01, REL-01, hoặc bất kỳ gate ghi `compiler_pin_required`.
- **Không** áp dụng cho dev hàng ngày trừ khi đang reproduce lỗi toolchain.

## Bối cảnh và bằng chứng

- **Quan sát được:** Run `rqm-20261024T104824Z` invalidated; PASS retracted; `artifact-ready.marker` flagged.
- **Đã xác nhận:** PLAN/RUNBOOK yêu cầu pin **1.95.0**; build disqualified dùng **1.99.0**.

## Nguyên nhân gốc / quyết định

`RUSTUP_TOOLCHAIN=stable` hoặc ambient stable **không** thay qualifying pin. Release evidence phải khớp pin đã khai báo.

## Cách xử lý / quy trình đề xuất

1. `cargo +1.95.0 build --release -p db-pro-native --locked`
2. Repackage theo `scripts/release/`; ghi `rustc +1.95.0 --version` vào run-manifest và result ENV-01.
3. Tạo **run_id mới**; không tiếp tục PASS trên candidate đã invalidated.

## Kiểm chứng

- **Lệnh:** `rustc +1.95.0 --version`; `cargo +1.95.0 build --release -p db-pro-native --locked`
- **Kỳ vọng:** Compiler string khớp 1.95.0; `RESULT.schema.json` validate trước khi claim PASS.

## Phương án thay thế và cạm bẫy

- Dùng stable “vì mới hơn” → disqualify toàn bộ release lane.
- Claim PASS chỉ từ static audit / screenshot thiếu semantic §2.

## Lịch sử thay đổi

- 2026-10-24 — Tạo từ invalidation audit run `rqm-20261024T104824Z`.
