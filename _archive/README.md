# _archive

This directory holds source that is **no longer part of the active product** but is
kept for reference, history, and parity comparison. Nothing here is built, tested,
linted, or packaged by CI.

## Contents

| Path | Archived on | Reason |
|---|---|---|
| `frontend/` | 2026-09-11 | Superseded by the native egui UI in `crates/ui` + `crates/native-app` |
| `bench/` | 2026-09-11 | React Flow / Cytoscape ER-renderer comparison harnesses for the retired web UI |

## `frontend/` — legacy React/Vite/Tauri webview UI

The React 19 + TypeScript + Vite application was the original presentation layer,
hosted inside the Tauri 2 webview (`crates/tauri-app`). It has been replaced by a
native Rust UI:

```text
crates/native-app   db-pro-native — eframe/egui binary (the shipped application)
crates/ui           db-pro-ui     — native shell, views, AppState, task bridge
crates/runtime      db-pro-runtime — bootstrap, services, worker/event bridge
crates/core         db-pro-core    — domain types, application services, ports
crates/infrastructure              — PostgreSQL, SQLite, secrets, metadata, SSH
```

### Why it was archived

- The native UI foundation and native IDE redesign are recorded as `COMPLETED`
  in `docs/plans/STATUS.md`.
- `docs/10-egui-native-migration-plan.md` selects `eframe`/`egui` as the target
  presentation layer and recommends dropping the Tauri webview from the production
  runtime.
- Keeping a second, unmaintained UI in the build path caused CI, token-contract,
  and quality-gate drift with no product value.

### What is still tracked

Only the frontend **source** is tracked here. Generated artifacts
(`node_modules/`, `dist/`, `.tanstack/`, `src/routeTree.gen.ts`) are intentionally
excluded via `.gitignore` because they are reproducible from
`pnpm install --frozen-lockfile` + `pnpm run build`.

### Restoring / running the archived app

The archived frontend still needs the Tauri host in `crates/tauri-app`.
`crates/tauri-app/tauri.conf.json` now points its `frontendDist` and `before*Command`
values at `_archive/frontend/`, so **no config change is needed to run it in place**:

```bash
# 1. Install dependencies inside the archive (this needs Node + pnpm again)
cd _archive/frontend && pnpm install --frozen-lockfile && cd -

# 2. Run the Tauri host; it builds/serves the archived frontend
pnpm tauri dev
```

If you prefer to move it back to the repository root, then also restore the `frontend/`
paths in `.gitignore` and in `crates/tauri-app/tauri.conf.json`
(`../../_archive/frontend` → `../../frontend`).

Use it as a **read-only reference** for expected behavior, not as a base for new
feature work. New UI work belongs in `crates/ui` and `crates/native-app`.

## `bench/` — legacy React-era ER renderer benchmarks

Standalone browser harnesses that compared the React-based ER diagram renderers
(React Flow / xyflow vs Cytoscape) before the UI moved to native `eframe`/`egui`.
They vendor their own React, xyflow, Cytoscape, and dagre UMD bundles and share no
code with the product.

- The ER diagram is now a native `egui::Painter` in `crates/ui/src/diagram_view.rs`.
- Nothing here is built, tested, or packaged by CI.
- It is kept only as a record of the renderer comparison that informed the original
  ER design, plus the retired shadcn/Tailwind `token-contract.html` demo.

See `_archive/bench/README.md` for the file inventory. Current ER performance
targets and the measurement method live in `.skills/perf-audit/references/perf-budgets.md`.
