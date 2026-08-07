# Changelog

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
