---
schema_version: 1
id: "MEM-SRC-39a731aa03bcd4f0"
title: "Source mirror: docs/plans/active/issue-537-539-edit-export-safety/FINDINGS.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-537-539-edit-export-safety/FINDINGS.md"
source_sha256: "39a731aa03bcd4f0c5a5020ddf8b718123f59a42abf16d8186d6dae531c3162b"
mirror_body_sha256: "39a731aa03bcd4f0c5a5020ddf8b718123f59a42abf16d8186d6dae531c3162b"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Findings — #537 / #538 / #539

Baseline SHA: `0e22aae7`.

## #537 — truncated cells were writable (P2, data loss)

- Runtime mapper (`crates/native-app/src/translate_map.rs`) capped text/JSON at
  65,536 bytes and bytea at 32 KiB and appended `…`, but `UiCell` carried no
  truncation bit. `open_cell_inspector` and `begin_data_cell_edit` checked only
  connection write access, so the grid offered an editor seeded with the
  display prefix; `stage_data_cell_edit` then staged that prefix as the new
  value.
- Fix: `UiCell::is_truncated()` + shared `MAX_CELL_TEXT_BYTES`; editor entry
  opens the inspector with a localized warning instead of the inline editor;
  the inspector hides Export bytes; `stage_data_cell_edit` rejects truncated
  source cells even if a caller bypasses the UI gate.
- Residual: the inspector still shows the capped prefix (it cannot recover the
  full value), clearly marked as truncated.

## #538 — driver-unaware SQL export (P2, wrong SQL)

- `format_result_sql_insert` / `format_result_copy` hard-coded `"ident"` and
  only doubled single quotes. MySQL needs backticks and backslash escaping;
  SQL Server needs brackets and `1/0` booleans.
- Fix: the export path passes `active_driver()` into both writers; identifiers
  go through `DriverDialect::quote_identifier`, literals escape per driver
  type, SQL Server booleans emit `1/0`. COPY remains PostgreSQL-only (already
  gated in the dialog) and uses dialect quoting.
- Residual: MySQL string escaping does not set `NO_BACKSLASH_ESCAPES`; a server
  running with that mode would need doubled backslashes. Documented, not
  changed here.

## #539 — overwrite prompt not bound to the effective path (P2, wrong file)

- `export_overwrite_pending: bool` stayed armed across Format and path changes,
  and the prompt rendered the raw typed path while the write used
  `effective_export_path()` (format extension appended). `Overwrite` could
  replace a file the user never confirmed.
- Fix: token is `Option<PathBuf>` compared against the effective path; Format,
  path edit, Browse, Cancel, Keep existing and the COPY fallback all clear it;
  the prompt shows the effective path.
- Residual: none known.

## #540 — schema table-info loading message (P2, already fixed)

- Verified at `0e22aae7`: keys `table.info_loading` / `table.info_load_failed`
  exist in all five locales and are used by `table_metadata_surface_view.rs`.
  Closed with evidence comment; no code change in this wave.

## Severity

- #537 P2 (requires a >64 KiB cell and an explicit Apply).
- #538 P2 (export produces invalid SQL on two drivers; no data is mutated).
- #539 P2 (requires an existing destination, a Format/path change and clicking
  Overwrite; the overwritten file is user data).
