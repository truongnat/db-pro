# 08 — DB Client — UI Design

---

## 1. Design System Overview

### 1.1 Design Philosophy

The DB Client UI follows the same design principles as OPASS Fab:

- **Clarity over cleverness**: UI should be immediately understandable without documentation
- **Consistency**: Same patterns, same terminology, same interaction models across all features
- **Efficiency**: Power users need keyboard shortcuts and quick actions; beginners need guided flows
- **Forgiveness**: Every action should be undoable or confirmable; errors should be clear and actionable
- **Responsiveness**: App should feel instant; loading states must never be ambiguous

### 1.2 Design Tokens

| Token | Value | Usage |
|---|---|---|
| `--color-primary` | `#1976d2` | Primary actions, links, active states |
| `--color-primary-dark` | `#1565c0` | Primary hover, pressed states |
| `--color-primary-light` | `#e3f2fd` | Primary backgrounds, tints |
| `--color-secondary` | `#7c4dff` | Secondary actions, accents |
| `--color-success` | `#2e7d32` | Success indicators, confirmed actions |
| `--color-success-light` | `#e8f5e9` | Success backgrounds |
| `--color-error` | `#c62828` | Error states, failed operations |
| `--color-error-light` | `#ffebee` | Error backgrounds |
| `--color-warning` | `#f57f17` | Warning indicators |
| `--color-warning-light` | `#fff8e1` | Warning backgrounds |
| `--color-info` | `#0277bd` | Info indicators |
| `--color-info-light` | `#e1f5fe` | Info backgrounds |
| `--color-text-primary` | `#212121` | Primary text |
| `--color-text-secondary` | `#757575` | Secondary text, hints |
| `--color-text-disabled` | `#bdbdbd` | Disabled text |
| `--color-border` | `#e0e0e0` | Borders, dividers |
| `--color-border-focus` | `#1976d2` | Focus ring |
| `--color-background` | `#ffffff` | Page background (light) |
| `--color-background-alt` | `#f5f5f5` | Alternating rows, card backgrounds |
| `--color-surface` | `#ffffff` | Cards, dialogs, panels |
| `--color-overlay` | `rgba(0, 0, 0, 0.5)` | Backdrop for dialogs |
| `--font-family` | `Roboto, sans-serif` | Body text |
| `--font-family-mono` | `JetBrains Mono, Fira Code, monospace` | Code, SQL editor |
| `--font-size-xs` | `0.75rem` (12px) | Captions, labels |
| `--font-size-sm` | `0.875rem` (14px) | Body text |
| `--font-size-md` | `1rem` (16px) | Subheadings |
| `--font-size-lg` | `1.25rem` (20px) | Headings |
| `--font-size-xl` | `1.5rem` (24px) | Page titles |
| `--spacing-xs` | `4px` | Tight spacing |
| `--spacing-sm` | `8px` | Small spacing |
| `--spacing-md` | `16px` | Default spacing |
| `--spacing-lg` | `24px` | Section spacing |
| `--spacing-xl` | `32px` | Page spacing |
| `--border-radius-sm` | `4px` | Small components |
| `--border-radius-md` | `8px` | Cards, dialogs |
| `--border-radius-lg` | `12px` | Panels, sheets |
| `--shadow-sm` | `0 1px 3px rgba(0,0,0,0.12)` | Subtle elevation |
| `--shadow-md` | `0 4px 6px rgba(0,0,0,0.1)` | Cards, dropdowns |
| `--shadow-lg` | `0 10px 25px rgba(0,0,0,0.15)` | Dialogs, modals |
| `--transition-fast` | `150ms ease` | Hover, micro-interactions |
| `--transition-normal` | `250ms ease` | State changes |
| `--transition-slow` | `400ms ease` | Page transitions |

### 1.3 Dark Theme Tokens

| Token | Light Value | Dark Value |
|---|---|---|
| `--color-background` | `#ffffff` | `#1e1e1e` |
| `--color-background-alt` | `#f5f5f5` | `#2c2c2c` |
| `--color-surface` | `#ffffff` | `#2e2e2e` |
| `--color-text-primary` | `#212121` | `#e0e0e0` |
| `--color-text-secondary` | `#757575` | `#a0a0a0` |
| `--color-border` | `#e0e0e0` | `#424242` |
| `--color-text-disabled` | `#bdbdbd` | `#616161` |
| `--color-overlay` | `rgba(0, 0, 0, 0.5)` | `rgba(0, 0, 0, 0.7)` |

## 2. Layout System

### 2.1 App Layout

```
┌─────────────────────────────────────────────────────────────┐
│  AppBar (height: 56px)                                      │
│  [Logo] [Connection Selector ▾] [Search ⌘K] [Settings ⚙]   │
├──────────┬──────────────────────────────────────────────────┤
│          │                                                  │
│  Sidebar  │  Main Content Area                              │
│  (240px)  │  (flex)                                        │
│  width:   │                                                  │
│  240px    │  [Feature-specific content]                      │
│          │                                                  │
│  - Nav    │                                                  │
│    items  │                                                  │
│  - Active │                                                  │
│    indicator                                                  │
│  - Icons  │                                                  │
│          │                                                  │
├──────────┴──────────────────────────────────────────────────┤
│  StatusBar (height: 28px)                                   │
│  [Connection: my-db] [Rows: 1,234] [Time: 12ms] [UTF-8]    │
└─────────────────────────────────────────────────────────────┘
```

### 2.2 Page Layout

```
┌─────────────────────────────────────────────────────────────┐
│  Page Header                                                │
│  [Title] [Actions...]                                       │
│  [Breadcrumb]                                               │
├─────────────────────────────────────────────────────────────┤
│  Content Area                                               │
│  ┌───────────────────────────────────────────────────────┐  │
│  │  [Toolbar / Filters / Tabs]                           │  │
│  ├───────────────────────────────────────────────────────┤  │
│  │                                                       │  │
│  │  [Main Content]                                       │  │
│  │                                                       │  │
│  │                                                       │  │
│  └───────────────────────────────────────────────────────┘  │
│  ┌───────────────────────────────────────────────────────┐  │
│  │  [Status Bar / Progress / Messages]                    │  │
│  └───────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

### 2.3 Responsive Breakpoints

| Breakpoint | Width | Layout |
|---|---|---|
| `sm` | ≥ 600px | Sidebar collapsed to icons only |
| `md` | ≥ 960px | Sidebar visible, 2-column layouts |
| `lg` | ≥ 1280px | Full layout, sidebar + content |
| `xl` | ≥ 1920px | Max content width 1600px, centered |

### 2.4 Grid System

- 12-column grid with 24px gutter
- Container max-width: 1600px (xl), 1280px (lg), 100% (md and below)
- Sidebar: fixed 240px width, collapsible to 64px (icons only)
- Content area: fills remaining space

## 3. Component Design System

### 3.1 Atoms (Basic Components)

| Component | Description | Variants | States |
|---|---|---|---|
| `Button` | Clickable action | `primary`, `secondary`, `outlined`, `text`, `icon` | default, hover, pressed, disabled, loading |
| `Input` | Text input field | `default`, `search`, `number`, `password` | default, focused, error, disabled, filled |
| `Label` | Form field label | `default`, `required` | default, error |
| `Typography` | Text rendering | `h1`-`h6`, `body`, `caption`, `code`, `overline` | default, primary, secondary, disabled |
| `Spinner` | Loading indicator | `small`, `medium`, `large` | spinning, complete |
| `Icon` | SVG icon | `default`, `small`, `large` | default, hover, active, disabled |
| `Badge` | Status indicator | `default`, `success`, `error`, `warning`, `info` | default, dot |
| `Chip` | Tag/pill | `default`, `filter`, `tag` | default, selected, disabled |
| `Divider` | Visual separator | `horizontal`, `vertical` | default |
| `Tooltip` | Hover information | `default` | visible, hidden |
| `Avatar` | User/connection icon | `default`, `small`, `large` | default |
| `Skeleton` | Loading placeholder | `text`, `rect`, `circle` | loading |

### 3.2 Molecules (Composite Components)

| Component | Composed From | Description |
|---|---|---|
| `SearchBar` | `Input` + `Icon` + `Button` | Search with clear button and keyboard shortcut |
| `FilterBar` | `Chip` + `Button` + `Select` | Active filters with remove and add |
| `StatusIndicator` | `Badge` + `Typography` | Connection status with label |
| `Pagination` | `Button` + `Select` + `Typography` | Page navigation with page size selector |
| `DataTableHeader` | `Typography` + `Button` + `Icon` | Column header with sort, resize, visibility |
| `DataCell` | `Typography` + `Tooltip` | Cell value with copy, edit, error states |
| `ConfirmDialog` | `Dialog` + `Button` + `Typography` | Confirmation dialog with cancel/confirm |
| `Alert` | `Badge` + `Typography` + `Icon` | Inline alert with severity and dismiss |
| `Toast` | `Alert` + `Snackbar` | Temporary notification with auto-dismiss |
| `ProgressBar` | `Divider` + `Spinner` | Progress indicator for long operations |
| `EmptyState` | `Icon` + `Typography` + `Button` | Empty data state with call-to-action |
| `ErrorState` | `Alert` + `Typography` + `Button` | Error state with retry button |
| `LoadingState` | `Skeleton` + `Spinner` | Loading placeholder |

### 3.3 Organisms (Feature Components)

| Component | Description | Key Features |
|---|---|---|
| `DataGrid` | Virtualized data grid | Sort, filter, resize, hide/show, reorder columns, inline edit, pagination, row selection |
| `QueryEditor` | SQL editor with Monaco | Syntax highlighting, autocomplete, line numbers, bracket matching, multiple cursors |
| `ResultGrid` | Query result display | Virtualized rows, column resize, sort, filter, copy with headers, export |
| `SchemaTree` | Hierarchical schema browser | Expand/collapse schemas, tables, columns; drag-and-drop; search |
| `ConnectionList` | Saved connections list | Status indicators, color coding, tags, quick connect, context menu |
| `ConnectionEditor` | Connection configuration form | All fields from CO03001, validation, test connection |
| `TransactionBar` | Transaction control bar | BEGIN/COMMIT/ROLLBACK buttons, auto-commit toggle, status indicator |
| `ExplainPlanTree` | EXPLAIN ANALYZE visualization | Cost tree, expand/collapse nodes, hover for details |
| `ExportDialog` | Export configuration dialog | Format selection, options, progress, cancel |
| `DDLEditor` | DDL creation/editing form | Table name, columns (name/type/nullable/default/PK/FK), indexes |
| `ERDDiagram` | Entity relationship diagram | Nodes for tables, edges for foreign keys, zoom/pan |
| `Sidebar` | Navigation sidebar | Collapsible, icons + labels, active indicator, nested items |
| `AppBar` | Top application bar | Connection selector, search, settings, notifications |
| `StatusBar` | Bottom status bar | Connection info, row count, query time, encoding |
| `Modal` | Dialog overlay | Configurable size, close button, overlay click-to-close |
| `Snackbar` | Notification toast | Auto-dismiss, action button, severity-based styling |

### 3.4 Templates (Page Layouts)

| Template | Description | Used By |
|---|---|---|
| `PageLayout` | Standard page with header + content + status | All feature pages |
| `AppLayout` | Full app layout with sidebar + app bar + content | Main application shell |
| `EditorLayout` | Split-pane editor with toolbar + content | Query editor, DDL editor |
| `GridLayout` | Full-width data grid with toolbar + filters | Data browsing, query results |
| `FormLayout` | Form with sections, validation, actions | Connection editor, settings |
| `WizardLayout` | Multi-step wizard with progress indicator | Migration wizard, import wizard |

## 4. Visual Design Patterns

### 4.1 Color Usage

| Context | Color | Rationale |
|---|---|---|
| Primary action buttons | `--color-primary` | Consistent CTA color |
| Success states | `--color-success` | Positive feedback |
| Error states | `--color-error` | Negative feedback, draws attention |
| Warning states | `--color-warning` | Caution, needs attention |
| Info states | `--color-info` | Neutral information |
| Active navigation item | `--color-primary-light` | Subtle highlight |
| Selected row in grid | `--color-primary-light` | Consistent with active state |
| Error cell in grid | `--color-error-light` | Red tint for failed operations |
| Alternating rows | `--color-background-alt` | Zebra striping for readability |
| Focus ring | `--color-border-focus` | Accessible focus indicator |

### 4.2 Typography

| Element | Font | Size | Weight | Line Height |
|---|---|---|---|---|
| Page title | Roboto | 24px | 600 | 1.3 |
| Section heading | Roboto | 18px | 600 | 1.4 |
| Subsection heading | Roboto | 16px | 600 | 1.4 |
| Body text | Roboto | 14px | 400 | 1.5 |
| Caption | Roboto | 12px | 400 | 1.4 |
| Code (inline) | JetBrains Mono | 13px | 400 | 1.5 |
| Code (block) | JetBrains Mono | 13px | 400 | 1.6 |
| SQL Editor | JetBrains Mono | 14px | 400 | 1.6 |
| Data cell | Roboto | 13px | 400 | 1.4 |
| Column header | Roboto | 13px | 600 | 1.4 |
| Button text | Roboto | 14px | 500 | 1.4 |
| Menu item | Roboto | 14px | 400 | 1.4 |
| Tooltip text | Roboto | 12px | 400 | 1.4 |

### 4.3 Iconography

- **Icon set**: Material Icons (consistent with MUI)
- **Icon size**: 18px default, 16px small, 24px large
- **Icon color**: Inherits from parent text color
- **Icon with label**: Always paired with text label for clarity
- **Icon-only buttons**: Must have `aria-label` for accessibility
- **Custom icons**: SVG inline for app-specific icons (connection status, etc.)

### 4.4 Spacing & Alignment

| Element | Spacing |
|---|---|
| Between sections | `--spacing-lg` (24px) |
| Between related elements | `--spacing-md` (16px) |
| Between inline elements | `--spacing-sm` (8px) |
| Between tight elements | `--spacing-xs` (4px) |
| Page padding | `--spacing-xl` (32px) |
| Card padding | `--spacing-md` (16px) |
| Input padding | `--spacing-sm` (8px) vertical, `--spacing-md` (16px) horizontal |
| Grid cell padding | `--spacing-sm` (8px) vertical, `--spacing-md` (16px) horizontal |
| Sidebar item padding | `--spacing-sm` (8px) vertical, `--spacing-md` (16px) horizontal |
| App bar height | 56px |
| Status bar height | 28px |
| Sidebar width (expanded) | 240px |
| Sidebar width (collapsed) | 64px |

### 4.5 Shadows & Elevation

| Level | Shadow | Usage |
|---|---|---|
| 0 | None | Default, flat elements |
| 1 | `--shadow-sm` | Cards, dropdowns |
| 2 | `--shadow-md` | Dialogs, popovers, tooltips |
| 3 | `--shadow-lg` | Modals, sheets, snackbar |
| 4 | `--shadow-lg` + border | App bar, sidebar |

### 4.6 Border Radius

| Size | Value | Usage |
|---|---|---|
| Small | 4px | Buttons, inputs, badges |
| Medium | 8px | Cards, dialogs, menus |
| Large | 12px | Panels, sheets |
| Full | 9999px | Pills, chips, avatars |

## 5. SQL Editor UI Design

### 5.1 Editor Layout

```
┌─────────────────────────────────────────────────────────────┐
│  [Toolbar: Execute ▶] [Format] [Templates] [History] [Save]│
├─────────────────────────────────────────────────────────────┤
│  ┌───────────────────────────────────────────────────────┐  │
│  │  SELECT * FROM users                                  │  │
│  │  WHERE created_at > '2024-01-01'                      │  │
│  │  ORDER BY id DESC;                                    │  │
│  │                                                       │  │
│  │  SELECT * FROM orders                                 │  │
│  │  WHERE status = 'pending';                            │  │
│  │                                                       │  │
│  └───────────────────────────────────────────────────────┘  │
│  [Line numbers] [Autocomplete popup] [Error markers]       │
└─────────────────────────────────────────────────────────────┘
```

### 5.2 Editor Features

| Feature | UI Pattern |
|---|---|
| Syntax highlighting | Monaco built-in PostgreSQL grammar |
| Line numbers | Left gutter, monospace, dimmed |
| Autocomplete | Dropdown below cursor, filtered by context |
| Bracket matching | Highlight matching bracket, subtle underline |
| Error markers | Red squiggly underline, tooltip on hover |
| Minimap | Right side, optional toggle |
| Word wrap | Toggle, default off for SQL |
| Font size | Ctrl+Scroll or setting, default 14px |
| Tab size | 2 spaces, configurable |
| Theme | Matches app theme (light/dark) |
| Keyboard shortcuts | Standard Monaco shortcuts + custom |

### 5.3 Editor Toolbar

| Button | Icon | Shortcut | Description |
|---|---|---|---|
| Execute Selected | ▶ | Ctrl+Enter | Run selected text or current statement |
| Execute All | ▶▶ | Ctrl+Shift+Enter | Run entire editor content |
| Format | ⚡ | Ctrl+Shift+F | Auto-format SQL |
| Templates | 📋 | Ctrl+Alt+T | Insert code snippet template |
| History | 🕐 | Ctrl+Shift+H | Show query history |
| Save | 💾 | Ctrl+S | Save current script |
| Clear | 🗑️ | — | Clear editor content |
| Cancel | ⏹️ | Escape | Cancel running query |

## 6. Data Grid UI Design

### 6.1 Grid Layout

```
┌─────────────────────────────────────────────────────────────┐
│  [Filter Bar] [Column Settings ▾] [Refresh ↻] [Export ↓]   │
├─────────────────────────────────────────────────────────────┤
│  ▼ id  │  name      │  email              │  status │ ... │
│  ───── │ ────────── │ ─────────────────── │ ─────── │     │
│  1     │ John Doe   │ john@example.com    │ Active  │ ... │
│  2     │ Jane Smith │ jane@example.com    │ Active  │ ... │
│  3     │ Bob Brown  │ bob@example.com     │ Inactive│ ... │
│  ...   │ ...        │ ...                 │ ...     │     │
│         │            │                     │         │     │
├─────────────────────────────────────────────────────────────┤
│  ← 1  2  3  4  5 →  [25] rows/page  [1,234 total rows]   │
└─────────────────────────────────────────────────────────────┘
```

### 6.2 Column Header

| Element | Description |
|---|---|
| Column name | Bold, left-aligned |
| Sort indicator | Arrow icon (▲ ASC, ▼ DESC) |
| Resize handle | Right edge, drag to resize |
| Visibility toggle | Click header menu to show/hide |
| Column type | Tooltip on hover showing data type |
| Filter icon | Click to open column filter |

### 6.3 Cell States

| State | Visual |
|---|---|
| Normal | Default text color, no background |
| Hover | `--color-background-alt` background |
| Selected | `--color-primary-light` background |
| Editing | Border highlight, input field appears |
| Error | `--color-error-light` background, red border, tooltip |
| Modified (unsaved) | Blue dot indicator on left edge |
| NULL | `NULL` label in italic, gray color |
| JSON/Array | Collapsible tree view, `{...}` or `[...]` summary |
| Long text | Truncated with ellipsis, full text on hover tooltip |

### 6.4 Inline Editing

```
Normal:   │ John Doe      │
Edit:     │ [John Doe     │]│  ← Input field appears
Error:    │ [John         │]│  ← Red border, tooltip: "Name cannot be empty"
Success:  │ John Doe      │  ← Blue flash briefly, then normal
```

### 6.5 Pagination

| Control | Description |
|---|---|
| Page buttons | ← 1 2 3 4 5 → with current page highlighted |
| Page size selector | Dropdown: 25 / 50 / 100 / 200 |
| Total rows | "1,234 total rows" |
| Jump to page | Input field for direct page navigation |
| First/Last buttons | « and » buttons for first/last page |

## 7. Connection Editor UI Design

### 7.1 Form Layout

```
┌─────────────────────────────────────────────────────────────┐
│  Connection Editor                                          │
├─────────────────────────────────────────────────────────────┤
│  General                                                   │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ Name:     [My Database          ]                   │   │
│  │ Driver:   [PostgreSQL        ▾]                     │   │
│  │ Host:     [localhost           ]                   │   │
│  │ Port:     [5432              ]                     │   │
│  │ Database: [mydb              ]                     │   │
│  │ Username: [postgres          ]                     │   │
│  │ Password: [•••••••••••••••••] [Show]               │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                           │
│  SSL                                                       │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ SSL Mode: [Disable        ▾]                        │   │
│  │ CA Certificate: [Choose File...]                    │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                           │
│  SSH Tunnel                                                │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ Enable SSH: [○]                                    │   │
│  │ Host:     [—]                                      │   │
│  │ Port:     [22              ]                        │   │
│  │ User:     [—]                                      │   │
│  │ Private Key: [Choose File...]                       │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                           │
│  [Test Connection] [Save] [Cancel]                         │
└─────────────────────────────────────────────────────────────┘
```

### 7.2 Connection Status Indicators

| Status | Icon | Color | Description |
|---|---|---|---|
| Connected | ● | Green | Active connection, ready for queries |
| Disconnected | ○ | Gray | No active connection |
| Connecting | ⟳ (spinning) | Blue | Attempting to connect |
| Error | ✕ | Red | Connection failed, click for details |
| Testing | ⟳ (spinning) | Blue | Testing connection, please wait |

## 8. Notification & Feedback Design

### 8.1 Toast Notifications

| Severity | Icon | Color | Auto-dismiss |
|---|---|---|---|
| Success | ✓ | `--color-success` | 4 seconds |
| Error | ✕ | `--color-error` | 8 seconds (or manual) |
| Warning | ⚠ | `--color-warning` | 6 seconds |
| Info | ℹ | `--color-info` | 4 seconds |

### 8.2 Inline Feedback

| Action | Feedback |
|---|---|
| Cell edit success | Brief blue flash on cell, value updated |
| Cell edit error | Red background, error tooltip on hover |
| Row add success | New row appears with green highlight, fades after 2s |
| Row add error | New row highlighted red, error tooltip |
| Row delete success | Row fades out, success toast |
| Row delete error | Row highlighted red, error tooltip |
| Query executing | Spinner in status bar, "Executing..." text |
| Query complete | Status bar shows row count + duration |
| Query error | Inline error on result grid, error toast |
| Export started | Progress dialog with cancel button |
| Export complete | Success toast with file path |
| Export error | Error dialog with retry button |

### 8.3 Loading States

| Context | Pattern |
|---|---|
| Initial page load | Skeleton screens for grid, tree |
| Query execution | Spinner in toolbar + "Executing..." in status bar |
| Data loading (pagination) | Skeleton rows in grid |
| Connection test | Spinner in editor + "Testing connection..." text |
| Export | Progress dialog with percentage and cancel |
| Save | Brief spinner on save button |
| Background refresh | Subtle refresh icon spinning in status bar |

## 9. Dialog & Modal Design

| Type | Width | Height | Behavior |
|---|---|---|---|
| Confirm dialog | 400px | Auto | Close on overlay click, Esc to cancel |
| Form dialog | 600px | 80vh | Scrollable body, sticky footer with actions |
| Progress dialog | 400px | Auto | Cancel button, cannot close by clicking overlay |
| Error dialog | 500px | Auto | Show error details in expandable section |
| Settings dialog | 700px | 80vh | Tabs for different settings categories |
| Export dialog | 500px | Auto | Format selector, options, progress |

## 10. Keyboard Shortcuts

| Shortcut | Action | Context |
|---|---|---|
| `Ctrl+Enter` | Execute selected/current statement | Query editor |
| `Ctrl+Shift+Enter` | Execute all statements | Query editor |
| `Ctrl+S` | Save current script | Query editor |
| `Ctrl+Shift+F` | Format SQL | Query editor |
| `Ctrl+K` | Open command palette | Global |
| `Ctrl+Shift+O` | Quick search (database objects) | Global |
| `Ctrl+Shift+H` | Show query history | Query editor |
| `Ctrl+Shift+T` | New query tab | Query editor |
| `Escape` | Cancel running query / Close dialog | Query editor / Global |
| `F5` | Refresh current data | Data grid |
| `Ctrl+Shift+E` | Export current result | Data grid |
| `Ctrl+Shift+C` | Copy selected rows with headers | Data grid |
| `Delete` | Delete selected row (with confirmation) | Data grid |
| `Enter` | Edit selected cell / Save edited cell | Data grid |
| `Tab` | Move to next cell / Next field | Data grid / Forms |
| `Shift+Tab` | Move to previous cell / Previous field | Data grid / Forms |
| `Ctrl+Z` | Undo (local edits) | Data grid / Editor |
| `Ctrl+Y` | Redo (local edits) | Data grid / Editor |
| `Ctrl+F` | Find in editor / Find in grid | Editor / Grid |
| `Ctrl+H` | Replace in editor | Editor |
| `Alt+0` | Zoom in on result grid | Data grid |
| `Alt+9` | Zoom out on result grid | Data grid |
| `Ctrl+Shift+P` | Toggle auto-commit | Query editor |
| `Ctrl+Shift+B` | Begin transaction | Query editor |
| `Ctrl+Shift+C` | Commit transaction | Query editor |
| `Ctrl+Shift+R` | Rollback transaction | Query editor |

## 11. i18n Design

### 11.1 Translation Key Structure

```
db.<feature>.<action>[<.context>]

Examples:
- db.connection.save.success
- db.connection.save.error
- db.connection.test.success
- db.connection.test.error
- db.query.execute.success
- db.query.execute.error
- db.query.execute.timeout
- db.grid.edit.success
- db.grid.edit.error
- db.grid.add.success
- db.grid.add.error
- db.grid.delete.success
- db.grid.delete.error
- db.export.csv.success
- db.export.csv.error
- db.schema.introspect.success
- db.schema.introspect.error
```

### 11.2 Pluralization

```
db.grid.rows.selected:
  one: "{{count}} row selected"
  other: "{{count}} rows selected"

db.query.results:
  one: "{{count}} row returned"
  other: "{{count}} rows returned"
```

### 11.3 Date/Number Formatting

- Use `Intl.DateTimeFormat` with locale from settings store
- Use `Intl.NumberFormat` with locale from settings store
- Dates: `YYYY-MM-DD HH:mm:ss` in query results (UTC)
- Numbers: Grouped by thousands per locale

## 12. Accessibility Design

### 12.1 Keyboard Navigation

| Feature | Keyboard Support |
|---|---|
| All buttons | Tab to focus, Enter/Space to activate |
| Data grid | Tab between cells, Enter to edit, Arrow keys to navigate |
| Schema tree | Arrow keys to navigate, Enter to expand/collapse |
| Dialog | Tab between focusable elements, Esc to close |
| Menu | Arrow keys to navigate, Enter to select, Esc to close |
| Query editor | Standard Monaco shortcuts + custom shortcuts above |
| All interactive elements | Focus indicator visible (`:focus-visible`) |

### 12.2 Screen Reader Support

| Element | ARIA Attribute |
|---|---|
| Data grid | `role="grid"`, `aria-rowcount`, `aria-colcount` |
| Grid cell | `role="gridcell"`, `aria-colindex`, `aria-rowindex` |
| Grid column header | `role="columnheader"`, `aria-sort` |
| Schema tree | `role="tree"`, `aria-expanded` on expandable items |
| Dialog | `role="dialog"`, `aria-modal="true"`, `aria-labelledby` |
| Alert/Toast | `role="alert"`, `aria-live="polite"` |
| Loading spinner | `role="status"`, `aria-label="Loading"` |
| Error message | `role="alert"`, `aria-describedby` on related input |
| All icons | `aria-label` or `aria-hidden="true"` if decorative |
| All interactive elements | `aria-label` or `aria-labelledby` |

### 12.3 Color Contrast

| Element | Foreground | Background | Ratio |
|---|---|---|---|
| Primary text | `#212121` | `#ffffff` | 16.1:1 ✓ |
| Secondary text | `#757575` | `#ffffff` | 4.54:1 ✓ |
| Primary button text | `#ffffff` | `#1976d2` | 4.63:1 ✓ |
| Error text | `#c62828` | `#ffffff` | 5.75:1 ✓ |
| Success text | `#2e7d32` | `#ffffff` | 5.71:1 ✓ |
| Link text | `#1976d2` | `#ffffff` | 4.63:1 ✓ |
| Disabled text | `#bdbdbd` | `#ffffff` | 3.07:1 ✗ (only for disabled elements) |

All contrast ratios meet WCAG AA (4.5:1 for normal text, 3:1 for large text).

### 12.4 Reduced Motion

- Respect `prefers-reduced-motion` media query
- Disable or reduce animations when user preference is set
- Transitions: `transition: none` when reduced motion is preferred
- Animations: Skip animation, show final state immediately
