# CHECKLIST — QA-P2-08 Read-Only Connection Data Grid Visual Affordance

- [x] Check connection `readonly` flag in `DataSection`
- [x] Guard `handleCellSave`, `handleDeleteRow`, `handleBatchDelete`, `handleRowSave` when connection is read-only
- [x] Pass `isReadonlyConnection` to `DataGrid`
- [x] Calculate `canEdit = pkColumns.length > 0 && !isReadonlyConnection` in `DataGrid`
- [x] Render read-only connection banner in `DataGrid`
- [x] Add i18n key `dataGrid.readOnlyConnection` in `en.json` and `ja.json`
- [x] Add unit tests in `data-grid.test.tsx`
- [x] Run all frontend quality gates (`typecheck`, `lint`, `format:check`, `check:tokens`, `test`)
- [x] Run backend quality gates (`cargo test`)
