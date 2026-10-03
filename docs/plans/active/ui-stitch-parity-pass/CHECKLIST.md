# Native UI Stitch Parity Pass — Checklist

## Planning
- [x] Evidence and failure scenario recorded (spec deltas vs current egui surfaces)
- [x] Scope/non-goals explicit (7 time-boxed tasks)
- [x] PostgreSQL/SQLite support matrix explicit (UI-only; no provider contract changes)

## T0 — Baseline audit (~25m)
- [x] Captures: table data, schema compare, queries sidebar, files workspace, ER diagram, history/output panel — dark+light @1280×800 (partial: compare/queries/files audited via source)
- [x] FINDINGS.md delta table filled per surface (gap → P1/P2 → owner task)

## T1 — Table workspace chrome (~45m)
- [x] Status strip: `Ping: <ms>` label + `UTF-8` on all surfaces (Ready/Active-Session/Auto-fallback already present; server version not in model — honestly omitted)
- [x] Dark capture (`evidence/t1-statusbar.png` shows `Ping: 1 ms · UTF-8`)
- [x] Commit `7ede34a5`

## T2 — Queries activity sidebar (~45m)
- [x] `OPEN QUERIES (N)` count header
- [x] Snippet trigger codes rendered as trailing affordance
- [x] Saved-item "Source:" line intentionally omitted — `UiSavedQuerySummary` has no connection binding; fabricated context rejected
- [x] Commit `73340682`

## T3 — Execution history (~60m)
- [x] Unified history surface: outcome icon, conn·schema meta, duration, day grouping (Today/Yesterday/date)
- [x] Status filter chips (All / Succeeded / Failed / Cancelled)
- [x] Record details: read-only SQL, Open-as-new-query, Copy
- [x] Dark capture (`evidence/t3-history-dark.png`)
- [x] Commit `153756e2`

## T4 — Explorer object filter workbench (~45m)
- [x] Filter popover: name match + evaluation mode (Contains/Prefix) + object-kind toggles (tables/views/functions/triggers)
- [x] Active-filter accent dot indicator on the toolbar button
- [x] Disabled kinds hidden from the tree (folder gating verified: Views folder removed)
- [x] Mode-aware match tests + nav-cache `mode` field
- [x] Dark capture (`evidence/t4-filter.png`)
- [x] Commit `bb137c33`

## T5 — Schema compare (~40m)
- [x] Diff summary counts: `N Added · N Changed · N Removed` next to total
- [x] Target-mismatch safety lock: plan records `connection · schema` target; banner + disabled Apply on divergence; `prepare_migration_sql` refuses before dispatch (defence in depth)
- [x] Focused tests: lock blocks on divergence / passes on match / diff resets the target
- [x] Capture helper `DB_PRO_CAPTURE_COMPARE` added (seeds real snapshot→diff→plan with divergent target)
- [ ] Visual capture — **pending: host windowing stopped producing framebuffer PNGs mid-session** (known-good HISTORY env also produced no PNG; process alive, no `capture: wrote framebuffer` log)
- [x] Commit `34601996`

## T6 — ER diagram polish (~30m, optional)
- [x] Deferred — T0 audit showed no obvious spec-10 delta at 1280×800; capture environment outage made visual verification impossible

## T7 — Final sweep (~20m)
- [x] fmt / clippy / test-ui / clean-code / perf-scan all executed and logged (VERIFICATION.md)
- [x] Evidence captures copied to `evidence/`
- [x] VERIFICATION.md + STATUS.md updated

## Review
- [x] P0 = 0, P1 = 0 on every commit
- [x] No token hard-coding; all colors via DbProTheme (accent/success/warning/danger roles)

## Runtime
- [x] Captures are UI runtime evidence; provider runtime N/A — plan touched only `prepare_migration_sql`'s pre-dispatch gate, no provider path
- [x] STATUS.md row matches final state
