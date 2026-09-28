# Component Layer Structure — Findings

## Baseline
- Baseline: `2fb54db0` (`main` before this feature branch).
- Select previously mixed egui rendering with popup keyboard/state decisions, accessible-label construction, and width calculations.
- Most exported components are standalone `.rs` files; complex families already use subdirectories but inconsistent file boundaries.

## Decisions
- `mod.rs` is the public component entry/exporter.
- Every public component is in scope and must have the named layers plus a usage README.
- UI owns egui rendering/layout and applies egui side effects; handler owns decision logic and calculations called by UI.
- Config owns component-specific defaults; shared design tokens/theme stay centralized.
- Migrate in batches to keep behavior and API stable; empty handler/config files are not accepted, so implementation must move meaningful responsibilities into them.

## Findings
- P2 — Select mixed UI and logic. Its keyboard navigation, trigger/accessibility labels, widths, option selection, dismissal, load-more decisions and dropdown geometry now live in typed handler functions.
- P2 — AspectRatio mixed ratio validation and size calculation into its egui allocation method. The pure decisions now live in typed handler functions with regression tests; UI still owns allocation, clipping and child rendering.
- AgentComposer UI review (source-only at HEAD `81238b4df2b173e6faa15180eddecddbbc36bdf2` plus worktree): P1 blank keyboard submission bypassed prompt validation; fixed by shared handler decision. Prompt label and wrapped toolbar added. P2 native narrow-width/accessibility evidence remains pending.
- AgentPrimitives UI review (source-only at HEAD `81238b4df2b173e6faa15180eddecddbbc36bdf2` plus worktree): initial P1 removable-chip hit target, destructive approval hierarchy and long-title overlap were fixed; disclosure headers gained keyboard/focus/accessibility semantics and unconditional repaint removed. Follow-up reports P0=0/P1=0, with P2 runtime interaction/layout evidence pending.
- Alert UI review (source-only at HEAD `81238b4df2b173e6faa15180eddecddbbc36bdf2` plus worktree): initial P1 destructive-by-default/backdrop dismissal, narrow-width sizing and modal identity/focus risks were addressed by opt-in destructive mode, backdrop policy, Escape, Cancel focus request, id salt and width-aware layout. Follow-up reports P0=0/P1=0; P2 focus trap/restoration and runtime verification remain open.
- Calendar review (source-only at HEAD `81238b4df2b173e6faa15180eddecddbbc36bdf2` plus worktree): P1 month/day normalization, lost view navigation state, and disabled popup mutation were fixed; current-local-date default, selection close and Escape close were added. Follow-up review reports P0=0/P1=0, with P2 egui interaction/accessibility and popup placement evidence pending.
- Database source review at implementation SHA `5ff99485b3205dc58369714e2c76ea31c9e51bb5`: initial P2 long-text overflow and repeated Connect while Connecting were fixed with shrink-aware labels/tooltips and disabled pending action; Error now labels recovery Retry while preserving the Connect action. Follow-up ACCEPT WITH P2 (P0=0/P1=0): extremely narrow cards may still be narrower than fixed status/SSL affordances; runtime viewports remain unverified.
- No provider/database backend impact; this is native UI architecture work.

## Unresolved
- Inventory each public component's meaningful handler/config responsibilities during its batch; do not duplicate shared settings or fabricate placeholder logic.
- Explain review at source SHA `fc1c4f815887c25696bbad13fd72bc9d6b1ae17d`: ACCEPT, introduced/inherited P0/P1/P2 all 0. Previous per-loop correctness and deep/wide traversal findings were fixed in commits `64096e67`, `49791a98`, and `fc1c4f81`; truncation findings persist across repeated heuristic application and render once.
- Explain provider scope: current query-output parser is PostgreSQL EXPLAIN JSON; SQLite EXPLAIN normalization remains unsupported/pending and was not changed. Required runtime screenshots/accessibility evidence and workspace-wide gates remain open plan-level items.
- Feedback was already organized into the five required layers before this batch; the inventory omission was stale. Source follow-up at `7a119f968ca2c4c14f21ded5fb2e9059de525583` sanitizes progress inputs, adds ProgressIndicator accessibility values/labels, and guards beam-edge overflow. Initial review P2s were fixed; final source review ACCEPT with introduced/inherited P0/P1/P2 all 0. Runtime screenshots/accessibility tree evidence remains pending.