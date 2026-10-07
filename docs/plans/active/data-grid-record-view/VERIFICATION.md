# Verification

Baseline/current HEAD: `3dd988e2ff06ac71942c7ec331acf66389d3c5cb`, main; new implementation is uncommitted. Source patch and native artifact hashes: evidence/manifest.json.

Final presentation: shared Table, exactly Field and Value columns. Native fixture screenshots are in evidence/. Native capture is real eframe framebuffer output; no live provider or OS clipboard claim.

Validation logs are stored in evidence/. Six focused tests cover copying field/value, staged edits, NULL/empty, single selection and invalid editor, filtered selection, result reset, and full-height content. Workspace tests previously ran with 1656 pass / 2 fail / 41 ignored; both failures independently reproduced at baseline (182 pass / 2 fail), so they are inherited. Final simplification: six focused tests passed again; fmt, workspace check/clippy, clean-code ratchet, locked native release and capture builds passed. Exact command logs are recorded in final-* logs.

Prior checks: fmt, workspace check/clippy, standard locked release build, capture build passed. Quick performance scan passed four executed checks; native benchmark attempt failed due unchanged libtest --quick handling. Benchmark/provider measurements were skipped; no performance claim.

Captures requested 1280×800, 1440×900, 1920×1080. Host caps larger window heights at 838 logical pixels; actual image dimensions in manifest. Normal/Grid/loading/error/empty fixture states captured. Exact 900/1080 height acceptance remains pending.

Remaining: live PostgreSQL and SQLite interaction, physical clipboard/accessibility verification, exact larger viewport heights, independent review, inherited workspace test fixes. Feature remains RUNTIME_VERIFY; source inspection and fixtures do not satisfy all runtime gates. No commit, PR, CI run, merge, or push.

Footer follow-up: cargo fmt --all -- --check, workspace check/clippy, locked release and capture builds passed. No new tests or full workspace rerun for styling-only changes. Six fresh native captures cover changed and unchanged data at three widths; same host height cap applies. Direct rustfmt check of include-based toolbar surface reports existing unrelated formatting; left untouched to avoid scope expansion. Latest footer/source artifact hashes in manifest, per-image binary provenance retained.
