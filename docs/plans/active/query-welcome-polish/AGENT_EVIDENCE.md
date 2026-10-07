# Agent evidence — Query Select and Welcome

## 1. Claim

Agent Codex root; task Review. Baseline SHA 3dd988e2ff06ac71942c7ec331acf66389d3c5cb, branch main, PR n/a. User-authorized UI scope: shared Query row-cap Select and static Welcome introduction. Prior uncommitted Grid/footer changes preserved.

## 2. Progress checkpoint

Implementation, targeted tests, check/clippy/build and twelve native captures complete. Feature RUNTIME_VERIFY; exact taller viewports, interactive selection and independent review pending. Native egui only. Provider semantics unchanged; no database commands added.

## 3. Implementation handoff / review request

Query uses shared Select index/value adapter. Welcome removes duplicate composer/actions/connections and their private state. Capture-only hooks open Query popup and Welcome fixture. Exact cumulative source patch, binary and PNG hashes: evidence/manifest.json. Validation details: VERIFICATION.md.

## 4. Review outcome

Self-review only; independent approval n/a. Introduced P0/P1: none identified. CI not run. Self-review verdict ACCEPT WITH P2 for source handoff; no introduced P0/P1 identified, runtime acceptance pending. Final workspace suite is not green: 1653 passed / 2 baseline-reproduced failures / 41 ignored. Both failure names and baseline evidence are in VERIFICATION.md; no introduced test failure identified. Actual gate outcomes are reported in VERIFICATION.md.

## 5. Research / audit handoff

External research n/a; local current source is sufficient. Findings and reusable lessons recorded in FINDINGS.md; global memory unchanged.

Select variant follow-up: public Default/Sm sizes and Outline/Ghost variants now support context-specific presentation. Sm shares compact Button tokens; Query uses Sm + Ghost. Two real geometry regressions added. Selected suite 12/12; workspace 1655 passed / 2 inherited failures / 41 ignored. fmt/check/clippy/build passed. Twelve fresh Query open/closed native captures; provenance in manifest. Independent review and exact-height/interactive gates still pending.

## 6. Tổng kết (Vietnamese summary)

Dropdown Query chuyển sang Select dùng chung. Welcome chỉ còn phần giới thiệu, bỏ các khối thao tác trùng lặp. Các test liên quan đạt, fmt/check/clippy/build đạt và có 12 ảnh native light/dark. Exact height và thao tác chọn/keyboard thực tế còn pending; giữ RUNTIME_VERIFY.

Select chung đã có size/variant; Query dùng kiểu toolbar 28px, bỏ khung form lớn. Geometry tests chứng minh chiều cao/căn giữa đồng bộ với Button, ảnh native đã kiểm tra đóng/mở.
