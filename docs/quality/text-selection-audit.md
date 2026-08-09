# Text Selection Audit Matrix

**Date:** 2026-08-09  
**Wave:** P2.1

## Policy

| Classification | CSS Class | Behavior | Use For |
|---|---|---|---|
| **select-text** | `select-text` / `select-content` | User can select text | Data values, SQL, DDL, IDs, metadata, definitions |
| **select-none** | `select-none` / `select-chrome` | Text selection disabled | Buttons, menus, tabs, toolbar, badges, chrome |
| **navigation-first** | `select-none` + context menu copy | No text selection; right-click → Copy Name | Tree items (schema/table/view in explorer) |

## Audit Matrix

| Surface | Example | Classification | Reason |
|---------|---------|----------------|--------|
| SQL query editor | Monaco editor content | `select-text` | User needs to select/copy SQL |
| DDL viewer | Monaco read-only DDL | `select-text` | User needs to copy DDL statements |
| Trigger body | Monaco code viewer | `select-text` | Code content must be selectable |
| Data grid cells | Query result values | `select-text` | Data values are content |
| Column names (list) | `column-list.tsx` | `select-text` | Column names are data/metadata |
| Column data types | Type labels in column list | `select-text` | Type definitions are metadata |
| Index names | `index-manager.tsx` | `select-text` | Index names are metadata |
| Index columns | Column list in index manager | `select-text` | Definitions are metadata |
| FK columns | `foreign-key-list.tsx` | `select-text` | Relation definitions are metadata |
| FK target table/column | Target table and column names | `select-text` | Relation definitions are metadata |
| Error messages | Error details, constraint errors | `select-text` | User needs to read/copy errors |
| JSON viewer | JSON metadata values | `select-text` | Data content |
| IDs (UUID, integer) | Primary key values, row IDs | `select-text` | Identifiers are data |
| Host/database/username | Connection dialog fields | `select-text` | Connection metadata |
| Execution info | Query timing, row count | `select-text` | Runtime metadata |
| Explain plan | Query plan details | `select-text` | Diagnostic content |
| Button labels | All button text | `select-none` | Application chrome |
| Menu labels | Context menu, dropdown items | `select-none` | Application chrome (Radix) |
| Tab bar | Workspace tab labels | `select-none` | Tab chrome; drag would conflict with selection |
| Toolbar labels | Data toolbar, filter/sort labels | `select-none` | Application chrome |
| Section titles | "Columns", "Indexes", etc. | `select-none` | Navigation chrome |
| Badges | Table count, column count | `select-none` | Application chrome |
| Status bar | Connection status | `select-none` | Application chrome |
| Activity bar | Sidebar icons | `select-none` | Navigation chrome |
| Pagination controls | Page numbers, per-page | `select-none` | Application chrome |
| Shortcut labels | Keyboard shortcut hints | `select-none` | Application chrome |
| Icons (all) | Lucide icons | `select-none` | Non-textual |
| Explorer tree items | Schema/table/view names | `select-none` + context menu | Navigation-first; Copy Name via right-click |
| Explorer tree metadata | Column count badges | `select-none` | Chrome accompanying navigation |
| Filter/sort rows | Column/operator selectors | `select-none` | Form controls are chrome |
| Filter/sort values | Value input fields | `select-text` | User types values in inputs (native) |
| Dialog titles | Dialog headers | `select-none` | Application chrome |
| Form labels | Input labels | `select-none` | Application chrome |
| Tooltips | Tooltip content | `select-none` | Ephemeral chrome |
| ER diagram nodes | Table names, column names | `select-text` | Schema metadata (data surface) |
| ER diagram handles | Connection points | `select-none` | Interactive chrome |
| MiniMap | Diagram minimap | `select-none` | Navigation chrome |

## Implementation Notes

- **shadcn/Radix components** (`button`, `dropdown-menu`, `context-menu`, `select`, `label`, `scroll-area`, `avatar`) already include `select-none` in their base styles.
- **Data surfaces** (`column-list`, `index-manager`, `foreign-key-list`) explicitly add `select-text` on data cells.
- **Explorer tree** uses `select-none` on tree items with context menu "Copy Name" / "Copy Qualified Name" actions.
- **Monaco editor** handles its own text selection internally — no utility class needed.
- **Input/textarea elements** natively allow text selection regardless of parent `select-none`.
