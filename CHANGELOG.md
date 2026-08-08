# Changelog

## [0.1.0] — 2026-08-XX

### Initial Release

DB Pro 0.1.0 — premium desktop Database IDE with Agent-native architecture.

**Database Support:**
- PostgreSQL: connect, test, disconnect, reconnect, credential management
- SQLite: file-based open/reopen

**Schema Explorer:**
- Tree-view with schemas, tables, views
- Metadata introspection with refresh
- DB Object workbench: Data, Columns, Indexes, Relations, DDL tabs

**Query Workbench:**
- Monaco SQL editor with syntax highlighting
- Statement-aware execution (cursor position)
- Selection execution (Cmd/Ctrl+Enter)
- Multi-statement execution (Cmd/Ctrl+Shift+Enter, F5)
- Explain / Explain Analyze
- SQL formatting
- Result grid with row count and timing
- Query history with favorites
- Export to CSV and Excel

**Workspace:**
- Multi-tab workspace with query and table tabs
- Pin/unpin, preview mode
- Workspace persistence across sessions
- Quick Open (Cmd/Ctrl+P) and Command Palette (Cmd/Ctrl+Shift+P)

**Safety:**
- Destructive query confirmation (DROP, TRUNCATE, DELETE without WHERE)
- Read-only connection mode
- SQL safety classifier with CTE awareness

**Architecture:**
- Action Platform with canonical runtime and audit trail
- Command Palette integration via action-to-command adapter
- Agent workspace panel (Preview / Coming Soon)
- OS keyring credential storage with encrypted fallback

**Tech Stack:**
- Tauri 2, Rust, React 19, TypeScript, Vite, shadcn/ui, Radix, Tailwind CSS 4, Monaco

---

## [Unreleased]

### Fixes

**Auto-prune stale recent connections** (`db175e4`)
Recent connection entries persisted in localStorage even after the underlying connection was deleted (e.g., via API sync or another device). The Welcome screen now cross-references recent entries against the live connection list on load and silently removes any orphans. No more ghost entries accumulating over time.

**AlertDialog replaces native `confirm()` for delete** (`db175e4`)
The delete connection flow previously used the browser's native `confirm()` dialog, which broke the visual consistency of the shadcn-based UI. Replaced with a styled `AlertDialog` matching the rest of the app — same typography, spacing, and button treatment. Includes a destructive-styled confirm button and proper i18n support (en + ja).

**Error toast on failed reconnection** (`db175e4`)
Clicking a recent connection on the Welcome screen or selecting one from the Quick Open palette (Ctrl+K / Ctrl+P) now shows an error snackbar if the connection fails. Previously, failures were silent — the status badge changed but no message appeared. The error message comes from the backend when available, falling back to a localized "Failed to connect" string.

### Testing

- Added 2 new test cases for stale entry pruning and error feedback (`72c0056`)
- 190/190 tests passing
- TypeScript: clean
