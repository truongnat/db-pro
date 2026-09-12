# Checklist: Table Details Workspace

- [x] 1. Switch default table view to Data on selection and automatically trigger data fetching
- [x] 2. Modernize table workspace toolbar with breadcrumbs, badges, common `Button`, and `SegmentedTabs`
- [x] 3. Implement interactive Structure tab with column search filter, metrics summary, and rich columns table
- [x] 4. Enhance Data Grid with in-place cell editing, staged change highlights, add row dialog, delete row, and pagination controls
- [x] 4a. Add selected-row set, Shift range selection, Cmd/Ctrl toggling, selected-row copy, and multi-row delete staging
- [x] 4b. Add typed filter operators including NULL predicates and keep header sorting synchronized with the table query
- [ ] 4c. Apply inserts/updates/deletes atomically through a parameterized PostgreSQL/SQLite transaction command
- [x] 5. Integrate SQL Editor actions & prefilled query generation (SELECT, INSERT, UPDATE, DELETE)
- [x] 6. Integrate DDL tab with Copy and Apply actions
- [x] 7. Add full context menu support for tables (Generate SQL templates, Copy Qualified Name, Ask Agent, Refresh)
- [ ] 8. Verify the full workspace after the current behavior slice; current focused UI tests pass, runtime multi-resolution evidence remains pending
