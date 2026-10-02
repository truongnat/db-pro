# Agent evidence — rs-ui Runtime Integration

## 1. Claim

| Field | Value |
|---|---|
| Agent identity | Codex, implementation lane |
| Issue(s) | n/a |
| Task state | Review |
| Baseline SHA | DB Pro `977a5dd00e656e3529e18372edbc121496cb700e` |
| Source snapshot tree | `e317464c083188b75c04d5be1e1a9f4517086b38` (isolated index containing current Cargo.lock, crates/ui and crates/native-app; main index unchanged) |
| Canonical rs-ui SHA | `727d65d3957eb7bbfbd316e68a272ca123ab411c` |
| Branch / PR | main / no PR |
| Scope interpretation | Explorer tree viewport paint only, including rows, icons, selection/hover, indentation, chevrons, scroll viewport and feedback. Preserve DB Pro state/actions and the current host. |
| Out of scope | Sidebar chrome/search/context menus, topbar, tabs, editor, result grid, status bar, providers, CI and any next migration. |

## 2. Progress checkpoint

- Implemented: canonical font loading/named families; exact Git pin; rs-ui offscreen Explorer paint with cached Glow texture; canonical ScrollState offset and scrollbar input.
- Retained: existing row hitboxes, stable tree keys, DB Pro selection/actions, egui expansion ownership and its original persistent child namespace.
- Source gates and explicit GPU input/benchmark results are recorded in VERIFICATION.md. Native comparison covers three logical viewports, 1x/2x and normal/loading/error/empty.
- Remaining: visual sign-off for lighter text and revised feedback presentation; physical HiDPI monitor verification. The overall integration remains IMPLEMENTING and the Explorer slice is RUNTIME_VERIFY.
- Historical Welcome-only spike findings remain in FINDINGS.md and VERIFICATION.md; they describe earlier SHA f6e798d, not the current Explorer source.

## 3. Implementation handoff / review request

| Field | Value |
|---|---|
| Exact source | Snapshot tree `e317464c083188b75c04d5be1e1a9f4517086b38`; canonical rs-ui `727d65d3957eb7bbfbd316e68a272ca123ab411c` |
| Commit list | rs-ui `727d65d`: feat: support host fonts and named text families (owner-authorized push). DB Pro changes are uncommitted, preserving preceding Welcome WIP. |
| File / surface inventory | Explorer painter/scroll/feedback adapters; shared row geometry constants; native host initialization; deterministic capture fixtures; manifests/lock and existing active plan. |
| Acceptance mapping | Canonical text/renderer paint the tree viewport. GPU row input, wheel/thumb scrolling, persistent IDs and existing semantics/actions are checked. Visual superiority is not asserted. |
| Commands / artifacts | VERIFICATION.md; logs under /home/vietis/.agents/outputs/db-pro/testresults/explorer-renderer; images under /home/vietis/.agents/outputs/db-pro/artifacts/explorer-renderer |
| CI disposition | Not run; no CI configuration changed. |
| Known limitations | Changed frames incur synchronous GPU readback and host upload; 10k synthetic roots exceed a 16ms preparation budget; native primary GPU backend required; simulated rather than physical HiDPI; generic monospace can differ from host font. |
| Provider impact | PostgreSQL / SQLite: n/a, no service/domain/provider changes. Capture fixtures do not connect to a database. |

## 4. Review outcome

| Field | Value |
|---|---|
| Reviewed source | Tree `e317464c083188b75c04d5be1e1a9f4517086b38` and rs-ui `727d65d3957eb7bbfbd316e68a272ca123ab411c`; self-review only |
| Verdict | BLOCK for final visual acceptance; implementation available for review, no independent approval recorded |
| P0 / P1 / P2 | P0 0 / P1 0 / P2 3: text-weight acceptance, bridge/large-tree frame cost, release size above 50MiB target |
| Baseline vs regression | Existing large-tree traversal and binary-size debt remain. The bridge adds readback/upload cost and dependencies. It avoids full offscreen text materialization and skips unchanged framebuffer copies. |
| Resolved findings | EGL enumeration conflict; clipped error detail; changed persistent tree namespace. |
| Next task(s) | Review Explorer screenshots. Do not start text input, editor, grid paint or wider renderer migration. |

## 5. Research / audit handoff

- Date: 2026-10-02.
- Sources: DB Pro source tree and canonical rs-ui SHA above; installed egui_glow 0.29.1 shader; actual native framebuffer captures and benchmark logs.
- Facts: canonical coverage blending is linear into sRGB; egui_glow multiplies texture/color in gamma space. Matched-font captures have visibly different text weight.
- Inference: matching font files alone does not guarantee identical perceived text weight. Sharp placement is verified visually; sharper-than-egui remains unproven.
- Measurements exclude host Glow GPU upload/presentation in the headless GPU benchmark. Native window screenshots verify actual host composition.
- Physical Retina behavior and independent visual approval remain unverified.

## 6. Tổng kết (Vietnamese summary)

Đã triển khai riêng Explorer tree viewport bằng rs-ui, giữ state/actions và namespace expand/collapse hiện tại. API font đã push vào rs-ui và DB Pro pin exact SHA mới. Có tests, benchmark GPU và ảnh đối chiếu; chưa chốt visual acceptance vì chữ nhẹ nét hơn egui. Không migrate thêm surface hoặc cấu hình CI.
