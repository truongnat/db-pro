# Repository Folder Mapping Implementation Plan

## State
- State: COMPLETED
- Branch: main (owner workflow override; no worktree)
- Baseline SHA: be2e431b07a8fcfbf39bf16fa1deefd600273633
- Verified source SHA: a1730a3e7156dc75fffa219f5f6f9eb286542e4c

## Goal
Organize source, tests, documentation and retained evidence by ownership; keep all product behavior and public Rust module paths unchanged.

## Architecture
Retain the six workspace crates and established feature/component directories. Move misplaced test bodies into adjacent tests/<source-stem>_tests.rs files with explicit #[path] declarations. Group only cohesive implementation files whose ownership is already clear. Keep active plans, historical release planning, architecture references, design assets and audit reports distinct.

## Scope and mapping rules
- Source: inspect every crate; relocate misplaced tests and cohesive source families, preserving logical module names, visibility and public exports. Update relative include and module paths.
- Root documentation: put reports with their audit/quality owner; design references under docs/design; original interlinked project/release plans under docs/plans/project.
- tasks/plan.md and tasks/todo.md: retain contents and acceptance states under docs/plans/project/core-component-standardization; do not infer completion or invent an active lifecycle transition from relocation.
- Design assets: group HTML mockups and Stitch downloads under docs/design; move retained screenshots beside their owning plan.
- Exact file-to-folder mapping is recorded in this directory before implementation.
- Update incoming references and relative links inside moved files. Preserve historical claims and SHA provenance.

## Non-goals
No product behavior/UI redesign, API rename, crate/layer redesign, data deletion, dependency changes, database operations or remediation of unrelated audit findings. Do not move Cargo/config/tool-discovery files, instruction files, agent skills, review infrastructure, build outputs, secrets or user databases. Keep canonical lifecycle/status files and required entry-point documents at their established paths.

## Execution
1. Inventory misplaced files; record exact old -> new mapping and ownership.
2. Move source/tests and documentation in independent owned slices; update references without compatibility shims or duplicate files.
3. Integrate source-path references in docs/scripts, update repository navigation, and check all mapped destinations and links.
4. Run formatting, workspace check/clippy/tests, UI architecture check and locked native release build. Smoke the native app to detect runtime asset/path regressions. No new behavior tests for pure filesystem moves.

## Acceptance
- [x] Every applied move has a mapping row; no destination collision or silent content loss.
- [x] No mapped old file remains as a duplicate/shim.
- [x] Logical Rust module/public paths and test inventory are preserved.
- [x] Moved documents retain content and working relative/incoming links.
- [x] Root navigation explains canonical folder ownership.
- [x] Gates and actual smoke evidence recorded in VERIFICATION.md.

## Provider impact
PostgreSQL and SQLite implementation behavior is unchanged. Workspace tests ran with ignored provider tests reported separately. The actual native release smoke seeded/connected/introspected a new temporary SQLite fixture only; PostgreSQL was not independently runtime-tested.

## Review focus
Relative include_str/include_bytes paths; #[path] resolution; script path assumptions; moved Markdown links and JSON inventories; historical records accidentally relabeled completed.
