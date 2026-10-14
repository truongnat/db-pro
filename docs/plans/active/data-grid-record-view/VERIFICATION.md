# Verification

Baseline/current HEAD: `3dd988e2ff06ac71942c7ec331acf66389d3c5cb`, main; new implementation is uncommitted. Source patch and native artifact hashes: evidence/manifest.json.

Final presentation: shared Table, exactly Field and Value columns. Native fixture screenshots are in evidence/. Native capture is real eframe framebuffer output; no live provider or OS clipboard claim.

Validation logs are stored in evidence/. Six focused tests cover copying field/value, staged edits, NULL/empty, single selection and invalid editor, filtered selection, result reset, and full-height content. Workspace tests previously ran with 1656 pass / 2 fail / 41 ignored; both failures independently reproduced at baseline (182 pass / 2 fail), so they are inherited. Final simplification: six focused tests passed again; fmt, workspace check/clippy, clean-code ratchet, locked native release and capture builds passed. Exact command logs are recorded in final-* logs.

Prior checks: fmt, workspace check/clippy, standard locked release build, capture build passed. Quick performance scan passed four executed checks; native benchmark attempt failed due unchanged libtest --quick handling. Benchmark/provider measurements were skipped; no performance claim.

Captures requested 1280×800, 1440×900, 1920×1080. Host caps larger window heights at 838 logical pixels; actual image dimensions in manifest. Normal/Grid/loading/error/empty fixture states captured. Exact 900/1080 height acceptance remains pending.

Remaining: live PostgreSQL and SQLite interaction, physical clipboard/accessibility verification, exact larger viewport heights, independent review, inherited workspace test fixes. Feature remains RUNTIME_VERIFY; source inspection and fixtures do not satisfy all runtime gates. No commit, PR, CI run, merge, or push.

Footer follow-up: cargo fmt --all -- --check, workspace check/clippy, locked release and capture builds passed. No new tests or full workspace rerun for styling-only changes. Six fresh native captures cover changed and unchanged data at three widths; same host height cap applies. Direct rustfmt check of include-based toolbar surface reports existing unrelated formatting; left untouched to avoid scope expansion. Latest footer/source artifact hashes in manifest, per-image binary provenance retained.

## Audit evidence update — 2026-10-13

`docs/plans/_audit-2026-10-13/03-data-grid-table-editor.md` verdict: **EVIDENCE_GAP**
(code fully verified; live/interactive evidence still missing). Confirmed at HEAD:
40px fixed rail left of `#`, Record bottom-anchored, exactly-one-row gate, staged
effective values, click-to-copy via `OutputCommand::CopyText`; all 7 record tests
pass (`cargo test -p db-pro-ui --lib record`).

**Stale-fact correction:** the "2 inherited workspace failures" above are now fixed —
`new_postgresql_connection_defaults_to_tls_require` and
`explicit_disable_selection_is_preserved_on_submit` (`app_tests.rs:372,467`) both
**PASS at HEAD**; the 1656/992 "2 fail" totals are stale on that fact.

Fresh captures: `docs/plans/_audit-2026-10-13/captures/table-record_{1280x800,1440x900,1920x1080}.png` —
fixed left rail + Field/Value single-record layout confirmed visually (larger heights
capped at 838). Remaining per earlier record: live PG/SQLite record flow, physical
OS clipboard + accessibility pass, exact 900/1080 heights, independent review. State
stays RUNTIME_VERIFY.
