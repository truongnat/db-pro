# Table Profile Quality — Checklist

## Planning
- [x] Screenshot findings and source failure scenarios recorded.
- [x] Scope, non-goals, and PostgreSQL/SQLite applicability recorded.

## Implementation
- [x] Exact numeric Min/Max ordering.
- [x] Consistent date/time extrema formatting.
- [x] Sample-scoped pattern labels and empty state.
- [x] Two-axis scrolling and bounded long-value presentation.

## Tests
- [x] Existing Profile unit test reviewed; its current assertions remain compatible.
- [x] Targeted compile check and release capture build executed; unit tests not run in this request.
- [x] PostgreSQL and SQLite runtime work is not applicable to the presentation logic; live UI evidence remains pending.

## Review
- [x] Changed code and diff reviewed.
- [x] P0 = 0.
- [x] P1 = 0.

## Runtime
- [ ] Native UI capture reviewed at all requested viewport sizes and states (1280×800 loaded state captured; remaining sizes/states pending).
- [x] `VERIFICATION.md` records only evidence actually collected.
- [x] `STATUS.md` matches the plan state (`IMPLEMENTING`).

## Profile width / Structure badge clipping — 2026-10-06
- [x] Use shared Table width allocation for Profile.
- [x] Retain bounded cells, horizontal overflow and hover disclosure.
- [x] Preserve full badge strokes within shared table cell padding.
- [x] Add regression test for complete 1px badge border in cell clip.
- [ ] Runtime loading/error/empty matrix remains pending.
