# Checklist

- [x] Research official DBeaver docs and native source at exact baseline SHA.
- [x] Report before implementation; owner authorized implementation and click-to-copy.
- [x] Fixed left rail with Grid at top, Record at bottom, shared selection/result state.
- [x] Single-record view; click field label/value to copy corresponding text.
- [x] Use effective staged values; validation before switching; no automatic database Save.
- [x] Six focused regressions: copy/NULL/empty/long values, staged edits, invalid input, sort/filter/offset, result replacement, full-height composition.
- [x] fmt/check/clippy/native release build; clean-code scan reviewed; quick perf scan PASS (partial).
- [x] Full release workspace suite executed: 1656 pass / 2 inherited fail / 41 ignored.
- [x] Reproduce both connection test failures on clean baseline source export.
- [x] Fifteen native fixture captures: Grid, Record, loading/error/empty at three requested widths.
- [ ] Workspace suite completely green: two baseline test-fixture failures remain outside this patch.
- [ ] Exact 1440×900 and 1920×1080 captures: this macOS host caps height to 838 logical px.
- [ ] Interactive native OS clipboard and accessibility verification.
- [ ] Independently verify SQLite and PostgreSQL loaded-result flows against live databases.
- [ ] Independent review before release/merge; no PR created in this task.
