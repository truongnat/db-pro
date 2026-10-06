# Agent evidence — Table Detail follow-up tabs audit

## 1. Claim

| Field | Value |
|---|---|
| Agent identity | Codex · native UI audit |
| Issue(s) | none provided |
| Task state | Review |
| Baseline SHA | `710ba002612c6d71ef2605f99f2a49743b51c3a4` |
| Branch / PR | main / no PR |
| Scope interpretation | Audit Table Detail → Foreign Keys, Constraints, Dependencies, and DDL for information visibility, layout behavior, and navigation. |
| Out of scope | Implementing fixes, provider behavior verification, and Data, Structure, Profile, and Indexes tabs. |

## 2. Progress checkpoint

- Current HEAD: `710ba002612c6d71ef2605f99f2a49743b51c3a4`.
- Completed acceptance rows: `[x]` source-path audit and severity assessment for Foreign Keys, Constraints, Dependencies, and DDL.
- Remaining acceptance rows: `[ ]` native screenshot review at standard and constrained widths.
- Findings / risks: P2 findings below; all are existing behavior in this audit pass, which changed no product source.
- Tests already run: none; read-only audit.
- Dependency / blocker changes: native visual inspection unavailable because `orca skills get computer-use --json` failed with `zsh:1: command not found: orca`; computer-use instructions prohibit falling back to another GUI tool.

## 3. Implementation handoff / review request

| Field | Value |
|---|---|
| Exact SHA | `710ba002612c6d71ef2605f99f2a49743b51c3a4` |
| Commit list | none |
| File / surface inventory | Audit only; no product source changed. Findings reference current source blobs in §5. |
| Acceptance mapping | Foreign Keys surface audit → source path trace and findings in §5. |
| Commands and counts | `orca skills get computer-use --json` → unavailable, exit 127. No tests run. |
| CI run IDs / status | not run |
| Known limitations | No runtime capture; action wrapping/clipping is a source-derived constrained-width finding pending visual confirmation. |
| Migrations / config implications | none |
| Out-of-scope changes | no implementation changes |

## 4. Review outcome

| Field | Value |
|---|---|
| Reviewed SHA | `710ba002612c6d71ef2605f99f2a49743b51c3a4` |
| Verdict | `BLOCK` for visual acceptance pending resolution or disposition of P2 findings and runtime verification. |
| P0 / P1 / P2 counts | introduced by this audit: 0 / 0 / 0; existing: 0 / 0 / 6. |
| Findings | Cross-schema navigation drops the target schema; search omits visible relation/type fields; relation key lists and metadata expressions can be clipped; narrow Action cells can wrap into the fixed-height row; Constraint Name uses synthetic identifiers; singular counts use incorrect copy. |
| CI disposition | not run; native runtime capture not collected |
| Next task(s) unblocked | Address findings and capture Foreign Keys at standard and constrained widths. |

## 5. Research / audit handoff

- Source date: 2026-10-04.
- Source references (HEAD SHA above; current working-tree blob SHA in parentheses where files contain unrelated local edits):
  - `crates/ui/src/table_relations_surface_view.rs` — `ca5771745583feb37f5fb4a01d038b9c02eb9497`.
  - `crates/ui/src/table_relations_view.rs` — `17059ed4f7eac6adb7f6e8ea72f077432996aa88`.
  - `crates/ui/src/workspace_actions.rs` — current blob `083b762eafbd00d0f485ea3ffbe70dfb00df3a2e`.
  - `crates/ui/src/table_editor_view.rs` — current blob `fcb02fb218c2314fa27446f456133d16cc10504d`.
  - `crates/ui/src/table_metadata_surface_view.rs` — `7937729af03b621856ecf4d7f5eb31c9f30c1669`.
  - `crates/ui/src/table_ddl_view.rs` — `269810d99965ed06cac5f0091f2b48c5d2eb7fd8`.
  - `crates/ui/src/table_ddl_surface_view.rs` — `c329246f2101359f5f3ee5ec54bc26c4b881801a`.
  - `crates/ui/src/table_view.rs` — `48c21ba3f1295351c8e2a0a6a842d9218ebfceb1`.
  - `crates/ui/src/components/table/ui.rs` — `bc156200e68848a3ce9290c14e725ae5ae37a4a0`.
  - `crates/ui/src/components/table/handler.rs` — `255df701ce3566251f33da68bb8f48a15aaa8135`.
  - `crates/ui/src/components/table/config.rs` — `0946606857298a65bbfd2156422cb54bd61d4609`.
- Factual findings:
  - **P2 — Cross-schema navigation loses schema context.** The action stores only `to_table` (`table_relations_surface_view.rs:11,175`, blob `ca5771745583feb37f5fb4a01d038b9c02eb9497`); the view calls `open_table(table)` (`table_relations_view.rs:18-19`, blob `17059ed4f7eac6adb7f6e8ea72f077432996aa88`). `open_table` and `request_table_info` resolve the request with the currently active schema and never receive `to_schema` (`workspace_actions.rs:54-82`, current blob `083b762eafbd00d0f485ea3ffbe70dfb00df3a2e`; `table_editor_view.rs:121-136`, current blob `fcb02fb218c2314fa27446f456133d16cc10504d`). A foreign key targeting another schema can therefore navigate to the wrong same-named table or fail to load the intended target.
  - **P2 — Narrow Action cells can clip wrapped actions.** The row height is fixed at 34pt while the Action column is flexible (`table_relations_surface_view.rs:81-105`, blob `ca5771745583feb37f5fb4a01d038b9c02eb9497`). The shared table gives a flex column an 80pt minimum (`components/table/handler.rs:30-48`, blob `255df701ce3566251f33da68bb8f48a15aaa8135`; `components/table/config.rs:26-29`, blob `0946606857298a65bbfd2156422cb54bd61d4609`). The action text and open button are placed in `horizontal_wrapped` (`table_relations_surface_view.rs:153-177`, same blob), so long deferrability labels can wrap while the table clips vertical content to the 34pt cell.
  - **P2 — Singular totals use plural copy.** Foreign Keys always formats `Total: {} foreign keys` (`table_relations_surface_view.rs:70-75`, blob `ca5771745583feb37f5fb4a01d038b9c02eb9497`); Dependencies does the same for `dependencies` (`table_metadata_surface_view.rs:157-171`, blob `7937729af03b621856ecf4d7f5eb31c9f30c1669`). A count of one reads `1 foreign keys` / `1 dependencies`.
  - Search evidence: the filter checks name, target table, and source columns only (`table_relations_surface_view.rs:184-192`, same blob); it omits target schema and target columns even though both are displayed.
- **Constraints tab findings:**
  - **P2 — Constraint Name contains generated labels, not database names.** Primary-key rows use `pk_{table}` (`table_metadata_surface_view.rs:195-208`, blob `7937729af03b621856ecf4d7f5eb31c9f30c1669`); NOT NULL rows similarly synthesize `nn_{table}_{column}` (same file, lines 245-260). The table header calls these “Constraint Name”, which implies they came from the database. Unique rows also combine unique indexes and constraints in one category (same file, lines 217-230).
  - Constraint expression/details cells render raw labels without truncation or hover text (`table_metadata_surface_view.rs:290-305`, same blob).
  - Constraint search uses `ConstraintRow::matches`, which checks name/expression/details but not the visible `kind` field (`table_metadata_surface_view.rs:322-327`, same blob). This is grouped with the search omissions below.
- **Dependencies tab findings:**
  - The cross-schema navigation finding above also applies here: the Open Table action sends only `dependency.name`, omitting `dependency.schema` (`table_metadata_surface_view.rs:383-398`, blob `7937729af03b621856ecf4d7f5eb31c9f30c1669`; handler `table_relations_view.rs:47-59`, blob `17059ed4f7eac6adb7f6e8ea72f077432996aa88`).
  - Dependency object name/details cells likewise lack width-aware truncation and tooltips (`table_metadata_surface_view.rs:383-410`, same blob).
  - Dependency search likewise omits its visible kind and direction labels (`table_metadata_surface_view.rs:338-342`, same blob; direction has a separate filter control).
- **DDL tab observation:** the Apply DDL button is rendered when writable but is always disabled by `DDL_APPLY_ENABLED = false` (`table_ddl_surface_view.rs:10-12,138-165`, blob `c329246f2101359f5f3ee5ec54bc26c4b881801a`). Its tooltip explains that this is unsupported and directs users to Open in Query; this is transparent, but the unavailable control still occupies space next to the active path.
- Inference: The exact viewport at which wrapped action text becomes visibly clipped is not established without a runtime capture. The constrained-width risk follows from the fixed row height, flex-column minimum, wrapping, and cell clip bounds above.
- **P2 — Search does not cover visible relation/type fields consistently.** Foreign Keys omits target schema/columns; Constraints and Dependencies omit visible kind labels from their text matching. The source evidence is listed above. Users can see a value in the row or type badge yet receive no result when filtering for it.
- **P2 — Long key/expression/details values are clipped without disclosure.** The missing truncation/tooltip cases are evidenced at `table_relations_surface_view.rs:127-151` (blob `ca5771745583feb37f5fb4a01d038b9c02eb9497`) and `table_metadata_surface_view.rs:290-305,383-410` (blob `7937729af03b621856ecf4d7f5eb31c9f30c1669`). The shared table clips children to each cell (`components/table/ui.rs:370-388`, blob `bc156200e68848a3ce9290c14e725ae5ae37a4a0`), so long composite keys and expressions cannot be fully inspected from the row.
- Inference: Constraint category toolbar uses a single non-wrapping horizontal row (`table_metadata_surface_view.rs:129-147`, blob `7937729af03b621856ecf4d7f5eb31c9f30c1669`), and the DDL toolbar does too (`table_ddl_surface_view.rs:31-75`, blob `c329246f2101359f5f3ee5ec54bc26c4b881801a`). These may crowd/clamp at narrow widths; capture is required to confirm.
- Inference: The constraint-name cell uses a horizontal icon/label pair without explicit local spacing (`table_relations_surface_view.rs:117-125`, blob `ca5771745583feb37f5fb4a01d038b9c02eb9497`). The adjacent Indexes surface required explicit `SPACE_SM` after a measured icon/name gap complaint; capture this surface to determine whether the same spacing issue appears here.
- Decision / recommendation: PENDING owner decision. Recommended changes: carry `(schema, table)` through navigation; disclose clipped key/expression/details values; avoid wrapping Action content in a fixed-height row; distinguish generated display labels from database constraint names.
- Unresolved questions: actual appearance at 1280×800 and narrow width; whether cross-schema table names collide in the active providers; whether hiding the disabled Apply DDL button is preferred.
- Downstream tasks activated: none.

## 6. Tổng kết (Vietnamese summary)

Đã audit source của Foreign Keys, Constraints, Dependencies và DDL tại SHA `710ba002612c6d71ef2605f99f2a49743b51c3a4`: ghi nhận sáu P2 về điều hướng mất schema, search bỏ sót trường đang hiển thị, nội dung metadata bị clip, Action có thể wrap trong row cố định, tên constraint tự tạo gây hiểu nhầm và copy số ít sai ngữ pháp. DDL còn một nút Apply luôn disabled nhưng có tooltip giải thích. Chưa có native capture vì `orca` không có trong môi trường; rủi ro hẹp cần xác nhận trực quan.
