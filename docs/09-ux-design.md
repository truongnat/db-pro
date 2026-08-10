# 09 — DB Client — UX Design

---

## 1. UX Principles

### 1.1 Core Principles

| Principle | Description |
|---|---|
| **Efficiency first** | Every interaction should minimize clicks and keystrokes. Power users should be able to work without taking hands off the keyboard. |
| **Progressive disclosure** | Show simple options first, reveal advanced options on demand. Never overwhelm the user with all features at once. |
| **Immediate feedback** | Every action should produce visible feedback within 100ms. Long operations should show progress indicators. |
| **Forgiveness** | Every destructive action (delete, drop, truncate) requires confirmation. Every edit should be undoable. |
| **Consistency** | Same patterns, same terminology, same interaction models across all features. Don't surprise the user. |
| **Clarity over cleverness** | UI should be immediately understandable without documentation. If a feature needs explanation, it's not intuitive enough. |
| **Accessibility** | All features must be usable with keyboard only. Screen reader support is non-negotiable. |
| **Performance as UX** | Slow UI is a UX bug. Every interaction must feel instant. Virtualize everything that can grow. |

### 1.2 User Personas

| Persona | Description | Goals | Pain Points |
|---|---|---|---|
| **DBA** | Experienced database administrator | Manage schemas, optimize queries, monitor performance | Needs advanced features, keyboard shortcuts, bulk operations |
| **Developer** | Application developer working with databases | Write queries, browse data, debug issues | Needs fast query execution, good error messages, autocomplete |
| **Analyst** | Data analyst exploring databases | Browse tables, run ad-hoc queries, export data | Needs simple UI, good filtering, easy export |
| **Ops Engineer** | Operations engineer managing database infrastructure | Monitor connections, manage users, backup/restore | Needs connection management, monitoring, automation |

### 1.3 User Goals (Top 10)

1. Connect to a database quickly with saved credentials
2. Browse database schema (tables, columns, types)
3. Write and execute SQL queries efficiently
4. View and edit table data inline
5. Export query results to CSV/JSON/Excel
6. Manage connections (save, organize, test)
7. Debug slow queries with EXPLAIN ANALYZE
8. Manage database objects (create/edit/delete tables, views, functions)
9. Monitor database health and active connections
10. Automate repetitive queries with saved scripts

## 2. User Journeys

### 2.1 First-Time User Onboarding

```
1. Open app → Welcome screen with "New Connection" button
2. Click "New Connection" → Connection editor opens
3. Fill in connection details (host, port, database, user, password)
4. Click "Test Connection" → Verify connection works
5. Click "Save" → Connection saved to sidebar
6. Click connection in sidebar → Connect to database
7. App shows schema tree on left, empty query editor on right
8. User clicks a table in schema tree → Table details shown
9. User clicks "Browse Data" → Data grid opens with first 25 rows
10. User clicks a cell → Inline editing activates
11. User presses Enter → UPDATE executed, cell shows new value
12. User clicks "Save Script" → Query saved for later
13. User closes app → Settings persisted, connections saved
```

### 2.2 Daily Workflow: Running Queries

```
1. Open app → Auto-connect to last used connection (or select from sidebar)
2. Click on schema tree → Browse tables, columns, indexes
3. Click "New Query" tab → Empty editor opens
4. Type SQL query → Autocomplete suggests tables, columns, keywords
5. Press Ctrl+Enter → Query executes
6. Result grid shows data → Scroll through virtualized rows
7. Click column header → Sort ascending/descending
8. Click filter icon on column header → Filter dropdown appears
9. Type filter value → Grid filters in real-time
10. Click cell → Inline editing activates
11. Edit value → Press Enter → UPDATE executed
12. If error → Cell highlights red, error tooltip appears
13. If success → Cell shows new value, brief blue flash
14. Click "Save" → Script saved to local history
15. Click "Export" → Choose format, download file
```

### 2.3 Daily Workflow: Managing Connections

```
1. Open app → Sidebar shows list of saved connections
2. Click "+" button → Connection editor opens
3. Fill in: name, host, port, database, user, password
4. Optionally configure SSL mode and SSH tunnel
5. Click "Test Connection" → Status indicator shows "Testing..."
6. If success → Green checkmark, "Connection successful" toast
7. If error → Red indicator, error message displayed below form
8. Click "Save" → Connection appears in sidebar with status indicator
9. Click connection in sidebar → Connects to database
10. Right-click connection → Context menu: Edit, Delete, Duplicate, Connect, Disconnect
11. Connection status updates in real-time (connected/disconnected/error)
```

### 2.4 Daily Workflow: Data Editing

```
1. Navigate to a table in schema tree
2. Click "Browse Data" → Data grid loads first page
3. Find row to edit → Click cell → Cell becomes editable
4. Type new value → Press Enter
5. If valid → UPDATE executed, cell shows new value
6. If invalid → Cell highlights red, error tooltip shows reason
7. To add new row → Click "Add Row" button → Empty row appears at bottom
8. Fill in values → Press Enter → INSERT executed
9. If error → Row highlights red, error tooltip
10. To delete row → Select row → Click delete button → Confirm dialog
11. If confirmed → DELETE executed, row removed from grid
12. To refresh data → Click refresh button → Grid reloads from DB
```

### 2.5 Daily Workflow: Schema Management

```
1. Open schema tree → Expand database → Expand schema → Expand tables
2. Right-click table → Context menu: View DDL, Edit Table, Delete Table, Browse Data
3. Click "View DDL" → DDL viewer shows CREATE TABLE statement
4. Click "Edit Table" → DDL editor opens with form
5. Add/remove columns, change types, set PK/FK
6. Click "Preview" → See generated DDL
7. Click "Execute" → ALTER TABLE executed
8. If success → Schema tree refreshes, toast shows success
9. If error → Error displayed inline, DDL editor keeps current state
10. Click "Create Index" → Form appears: select table, columns, unique, name
11. Click "Execute" → CREATE INDEX executed
12. View ERD diagram → See table relationships visually
```

### 2.6 Daily Workflow: Export Data

```
1. Execute query → Results appear in grid
2. Click "Export" button → Export dialog opens
3. Select format: CSV / JSON / Excel
4. Configure options: include headers, delimiter (CSV), encoding
5. Click "Export" → Progress dialog shows
6. If streaming (>10k rows) → Progress updates incrementally
7. When complete → Success toast with file path
8. File saved to default download directory (configurable)
9. Option to open file immediately after export
```

## 3. Interaction Patterns

### 3.1 Navigation Patterns

| Pattern | Usage | Implementation |
|---|---|---|
| Sidebar navigation | Switch between features (Connection, Query, Schema, Data, Export) | Click sidebar item, route changes |
| Tab navigation | Multiple query editors, result sets | Click tab to switch, Ctrl+Shift+T to create new |
| Breadcrumb navigation | Navigate back in schema hierarchy | Click breadcrumb item to go up |
| Command palette | Quick access to any feature | Ctrl+K opens palette, type to filter |
| Keyboard shortcuts | All actions accessible via keyboard | See keyboard shortcuts table in UI design |
| Context menus | Right-click for feature-specific actions | Right-click on any element for context menu |

### 3.2 Data Interaction Patterns

| Pattern | Usage | Implementation |
|---|---|---|
| Inline editing | Edit cell value directly in grid | Click cell → input appears → Enter to save, Esc to cancel |
| Batch operations | Select multiple rows → perform action | Checkbox column, select all, action bar appears |
| Drag and drop | Reorder columns, move rows | HTML5 drag and drop API |
| Context menu | Right-click for options | Custom context menu with feature-specific actions |
| Keyboard navigation | Navigate grid with arrow keys | Arrow keys move cell selection, Enter edits |
| Copy/paste | Copy data to/from clipboard | Ctrl+C/V, with column headers option |
| Undo/Redo | Undo local edits | Ctrl+Z/Y, local state only (not synced to DB) |

### 3.3 Feedback Patterns

| Pattern | Usage | Implementation |
|---|---|---|
| Toast notification | Brief, non-blocking feedback | Snackbar component, auto-dismiss after timeout |
| Inline error | Error shown next to the element | Red highlight + tooltip |
| Progress indicator | Long-running operation feedback | Progress bar in dialog or status bar |
| Loading skeleton | Placeholder while data loads | Skeleton screens matching content layout |
| Confirmation dialog | Destructive action requires confirmation | Modal dialog with "Cancel" and "Confirm" |
| Status bar | Persistent, at-a-glance information | Bottom bar showing connection, rows, time |
| Spinners | Loading state for specific elements | Spinner component in relevant location |

### 3.4 Form Patterns

| Pattern | Usage | Implementation |
|---|---|---|
| Inline validation | Validate as user types | Zod schema, show error on blur |
| Required field indicator | Mark required fields | Asterisk (*) next to label |
| Help text | Explain field purpose | Small text below field |
| Conditional fields | Show/hide based on selection | Dynamic form rendering |
| Auto-save | Save form state automatically | Debounced save to localStorage |
| Reset button | Reset form to initial values | "Reset" button that clears all fields |

## 4. Error Handling UX

### 4.1 Error Categories

| Category | Example | UX Response |
|---|---|---|
| Connection error | Cannot reach database | Toast error, connection editor shows error below form, retry button |
| Query error | Syntax error in SQL | Inline error on result grid, error message in toast |
| Validation error | Invalid connection config | Inline error below field, red border on input |
| Permission error | User lacks SELECT privilege | Toast error with "Request permission" link |
| Timeout error | Query took too long | Toast warning, "Query timed out" message, cancel button |
| Network error | Tauri bridge disconnected | Toast error, "Reconnect" button |
| Data error | Duplicate key violation | Inline error on affected cell, error tooltip |
| Export error | File write failed | Error dialog with retry button |

### 4.2 Error Display Rules

1. **Never show raw error messages to users** — always use i18n keys
2. **Show error at the point of failure** — inline for grid errors, toast for global errors
3. **Provide actionable recovery** — every error should have a "what to do next" suggestion
4. **Log errors with context** — timestamp, action, connection_id, SQL (if applicable)
5. **Don't block the user** — errors should not prevent the user from continuing to work
6. **Group related errors** — if multiple queries fail, show summary with details expandable

### 4.3 Error Recovery Flow

```
Error occurs
  → Show inline error (if applicable)
  → Show toast notification
  → Log error with context
  → Offer recovery action (retry, cancel, undo)
  → If unrecoverable → Show error dialog with details
  → User can dismiss error and continue working
```

## 5. Performance UX Guidelines

### 5.1 Perceived Performance

| Goal | Target | Technique |
|---|---|---|
| App startup | < 3 seconds | Lazy load non-critical modules |
| First query result | < 500ms for < 1k rows | Show skeleton while loading |
| Grid render (10k rows) | < 100ms | Virtualized rendering |
| Grid render (100k rows) | < 200ms | Virtualized rendering + pagination |
| SQL autocomplete | < 100ms after typing | Debounced introspection cache |
| Schema tree load | < 1 second | Lazy loading of tree nodes |
| Inline edit response | < 200ms | Optimistic UI update + background sync |
| Export start | < 500ms | Show progress immediately, stream results |

### 5.2 Loading Strategies

| Strategy | When to Use | Implementation |
|---|---|---|
| Skeleton screen | Initial page load, first data load | Show placeholder matching content layout |
| Spinner | Short operations (< 2s) | Show in relevant UI element |
| Progress bar | Long operations (> 2s) | Show in dialog or status bar |
| Optimistic update | Edits that are likely to succeed | Update UI immediately, revert on error |
| Lazy loading | Large trees, paginated data | Load on demand, show loading indicator |
| Virtualization | Large result sets (> 100 rows) | Only render visible rows |
| Debouncing | Search, filter, autocomplete | Wait for user to stop typing |
| Caching | Introspection results, query history | Cache in meta-store, invalidate on connection change |

### 5.3 Empty States

| Context | Empty State |
|---|---|
| No connections saved | Illustration + "No connections yet. Click + to create one." |
| No query history | Illustration + "No queries executed yet." |
| No query results | Illustration + "Execute a query to see results." |
| No schema objects | Illustration + "This schema has no tables or views." |
| No search results | Illustration + "No results found for your search." |
| No filters applied | Show all data (no empty state needed) |
| No selected row | Disable edit/delete buttons, show hint |

## 6. Mobile / Responsive Considerations

### 6.1 Desktop-First Design

The DB Client is primarily a desktop application (Ubuntu). The design is optimized for:

- **Screen width**: 1024px minimum, 1920px ideal
- **Screen height**: 768px minimum, 1080px ideal
- **Input**: Keyboard + mouse (no touch optimization)
- **Window size**: Resizable, minimum 1024x768

### 6.2 Responsive Breakpoints

| Breakpoint | Width | Layout Changes |
|---|---|---|
| `lg` (default) | ≥ 1280px | Full layout: sidebar + content |
| `md` | 960px - 1279px | Sidebar collapsed to icons, content fills width |
| `sm` | 600px - 959px | Sidebar hidden, hamburger menu, content full-width |
| `xs` | < 600px | Not officially supported (desktop app) |

### 6.3 Sidebar Behavior

| State | Width | Content |
|---|---|---|
| Expanded | 240px | Icons + labels |
| Collapsed | 64px | Icons only, tooltips on hover |
| Hidden | 0px | Hamburger menu to toggle |

## 7. Design System Governance

### 7.1 Component Library

- All components live in `frontend/src/commons/components/`
- Components are documented with Storybook (future)
- Each component has: implementation, tests, documentation, examples
- Components are versioned independently

### 7.2 Design Tokens

- All design tokens defined in `frontend/src/commons/design-tokens.ts`
- Tokens map to CSS custom properties
- Light and dark themes use the same tokens with different values
- Tokens are consumed via MUI theme

### 7.3 Component Review Process

1. Component is proposed with design mockup
2. Implementation follows design system patterns
3. Component is tested for accessibility (keyboard, screen reader, contrast)
4. Component is tested for all states (default, hover, focused, disabled, error, loading)
5. Component is reviewed by at least one other developer
6. Component is documented with usage examples

### 7.4 UX Review Process

1. New feature proposed with user journey map
2. UX review checks: consistency, accessibility, performance, error handling
3. UX review happens before implementation starts
4. UX issues are tracked as tickets with `ux` label
5. UX review sign-off required before feature is marked complete
