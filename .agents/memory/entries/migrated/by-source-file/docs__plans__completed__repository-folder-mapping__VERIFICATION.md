---
schema_version: 1
id: "MEM-SRC-562e41abe11a9f26"
title: "Source mirror: docs/plans/completed/repository-folder-mapping/VERIFICATION.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/completed/repository-folder-mapping/VERIFICATION.md"
source_sha256: "562e41abe11a9f266aaed17ea6c9e80c92aed983b97840bec4107a41a56e8beb"
mirror_body_sha256: "562e41abe11a9f266aaed17ea6c9e80c92aed983b97840bec4107a41a56e8beb"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Repository Folder Mapping Verification

## Source revisions
- Baseline: `be2e431b07a8fcfbf39bf16fa1deefd600273633`.
- Verified source relocation commit: `a1730a3e7156dc75fffa219f5f6f9eb286542e4c`.
- Documentation relocation is committed separately; no Rust logic/API/UI rendering changes.

## Conservation and mapping
- All **93/93** mapped destinations exist; no mapped old file remains and no destination collides.
- Source: **40 moves**, **35 byte-identical**. Five test files differ only in relative locale/child-test paths or updated location comments.
- Documentation/evidence: **53 moves**, **48 byte-identical**. Five moved text files include navigation/inventory updates.
- All **27 moved PNG/HTML assets** retain their pre-move SHA-256.
- All **19 Stitch inventory entries** resolve to existing files.
- Pre-move SHA-256 and byte sizes: `MAPPING.json`; human navigation: `MAPPING.md`.
- Empty old source/doc directories were removed only after files were conserved.

## Executed gates
The following ran as one direct `&&` chain, with no output-filter pipeline masking exit statuses. Chain exit: **0**. Full raw output: `evidence/rust-gates.txt`.

| Command | Observed result |
|---|---|
| `cargo +nightly fmt --all -- --check` | PASS |
| `cargo check --workspace` | PASS |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS, no warnings |
| `cargo test --workspace` | **2,262 passed / 0 failed / 65 ignored**, 27 result suites |
| `cargo build --release --locked -p db-pro-native` | PASS |

Separate gate chain, exit **0**:
- `bash scripts/check-ui-architecture.sh`: PASS.
- `bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci`: **12 pass / 4 warning categories / 0 failures**. Warnings concern existing parameter count, long functions/files and clone counts in relocated code; no code was added to suppress them.

Rust-analyzer capabilities probing failed to start the configured server (exit code 0); no successful LSP refactor is claimed. Logical symbols were not renamed; explicit path changes were compiler/test verified.

## Actual release runtime smoke
Launched the newly built release binary with:

```text
DB_PRO_DATA_DIR=/private/tmp/dbpro-folder-map-smoke.rOX3nw
DB_PRO_DISABLE_KEYRING=1
DB_PRO_WINDOW_SIZE=1280x800
./target/release/db-pro-native
```

- App PID 836, Orca window 91998; actual window 1280×832 including titlebar; screenshots 2560×1664 at 2×.
- Connected to the **new temporary Sample E-Commerce SQLite fixture**. Worker logs and rendered UI show schema introspection completed: **11 tables / 2 views / 1 trigger / 0 functions**.
- This exercises the relocated native command/event translation and runtime worker → SQLite → introspection → refreshed UI path.
- Inspected `evidence/release-smoke-light.png`, then activated the identified theme button and inspected `evidence/release-smoke-dark.png`. Both show the native Welcome shell and loaded SQLite schema with fonts/icons present. No UI redesign or full visual acceptance matrix is claimed.
- Service readiness regex `capture: resolved window size override` timed out; this was not treated as smoke success. Subsequent actual OS window/accessibility snapshots and inspected screenshots established rendering and runtime completion.
- Stopped only the smoke process through its named service, not other applications.

### Preference/storage safety check
`DB_PRO_DATA_DIR` isolates runtime data, not eframe preferences by itself. This was checked before deleting the temporary fixture:
- `cargo tree --locked -p db-pro-native -e features -i eframe` lists only accesskit/default_fonts/glow/wayland/x11; **eframe persistence is not enabled in this binary**.
- eframe 0.36.2 `native/epi_integration.rs:130-145` returns no storage when the persistence feature is absent. The `save` implementation therefore has no eframe file store to write in this launch.
- If enabled with app title `DB Pro`, `native/file_storage.rs:32-35` calculates `~/Library/Application Support/DB-Pro/app.ron`; that file was confirmed absent after the smoke. It is distinct from runtime's `DB Pro` directory.
- No user-preference restoration or deletion was performed. Runtime smoke files were confined to the temporary directory above.

### Retained hashes
```text
480bf463f880cbafcf518811acd004d34e90a1ece605d1f910e74d06bcf77ffd  target/release/db-pro-native
0b43107e8fd880b8a4c3634badf0b793f51143c8fab37b1097508b0ba4f65a9b  evidence/release-smoke-dark.png
714207ed3adfcf1c7a77be1278105f8341f3d3da6cc1b635cb62cf87a2d2df4f  evidence/release-smoke-light.png
```

## Provider matrix
| Provider | Implementation effect | Runtime evidence in this cleanup |
|---|---|---|
| SQLite | Unchanged | Native release connection/schema refresh on newly seeded temporary fixture |
| PostgreSQL | Unchanged | Not independently exercised; ignored provider tests are not treated as runtime passes |

## Independent review and inherited limitations
Two separate read-only review lanes accepted source and documentation mappings: introduced **P0=0 / P1=0 / P2=0**. See `REVIEW.md`. No remote CI, push, PR or VPS/Kilo approval was performed.

Inherited missing docs/01–06 references and the registry's old query-workspace-zed-shell path remain documented rather than silently changing historical plan states. Revision-pinned old source citations and raw command transcripts intentionally retain their historical paths; consult the mapping for present-day locations.
