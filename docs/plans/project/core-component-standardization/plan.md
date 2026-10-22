# Implementation Plan: 100% Core Component Standardization

## Overview
Standardize all UI surfaces across the entire DB Pro native application (`crates/ui`) to use 100% canonical design system components (`crates/ui/src/components/*`) with strict parity to the Component Gallery. This eliminates divergent custom button/input painting, ad-hoc frames, and all call-sites of `crates/ui/src/components/legacy.rs`. Every surface will adhere to shared tokens, focus rings, accessible labels, interactive states, and semantic themes.

## Architecture Decisions
- **Single Source of Truth**: All UI primitives (`Button`, `Badge`, `Input`, `Textarea`, `Card`, `Table`, `Dialog`, `Alert`, `Select`, `Toggle`, `EmptyState`, `SectionHeader`, `Spinner`, `Progress`, `ToastManager`) must come exclusively from `crates/ui/src/components/*`.
- **Vertical Slicing by Workstation Domain**: Migrations are executed surface-by-surface (Shell → Explorer → Query → Table → Diagram → Compare/Secondary) so the application remains compilable, functional, and testable at every single commit.
- **ActivityBar & StatusBar Evolution**: Extend `components::workspace::{ActivityBar, StatusBar}` to support the full active feature matrix (all 9 workspace activities + session context) so the shell uses the real core component without local ad-hoc reimplementation.
- **Deprecation of `legacy.rs`**: Systematically migrate calls away from `compact_button`, `secondary_button_with_icon`, `input_full_width`, `section_label`, `panel_frame`, etc., then deprecate/remove the obsolete helpers.
- **Zero Regression on Database Safety & Shortcuts**: Retain all hotkeys (⌘↵, Escape), focus management, accessible labels, context menus, and execution confirmations throughout the refactor.

## Task List

### Phase 1: Shell & Navigation Chrome
- [ ] Task 1: Standardize ActivityBar, TopBar, and StatusBar to Core Workspace Primitives

### Checkpoint: Phase 1
- [ ] Tests pass (`cargo test -p db-pro-ui`)
- [ ] Native build clean (`cargo check -p db-pro-native`)
- [ ] Activity rail and status bar render with unified tokens and responsive layout

### Phase 2: Explorer & Object Tree Surfaces
- [ ] Task 2: Standardize Explorer Toolbar, Filter, and Schema Feedback Surfaces
- [ ] Task 3: Standardize Object Folders, Agent Context, and Empty States

### Checkpoint: Phase 2
- [ ] Tree navigation and object filtering function without visual regression
- [ ] Empty state and loading indicators use canonical `EmptyState` and `Spinner`

### Phase 3: Query Workspace & Editor Surfaces
- [ ] Task 4: Standardize Query Editor Actions, Run Controls, and Context Header
- [ ] Task 5: Standardize Query Output Actions, Tabs, and Results Surface Controls
- [ ] Task 6: Standardize Query Dialogs (Save, Destructive Run, Folder Delete)

### Checkpoint: Phase 3
- [ ] Query execution, cancellation, explain plan, and history search work identically
- [ ] Confirmations and destructive warnings display canonical `AlertDialog` / `DestructiveOperationDialog`

### Phase 4: Table Workstation Surfaces
- [ ] Task 7: Standardize Table Structure, Indexes, Metadata, and Relations Panels
- [ ] Task 8: Standardize Table Profile, DDL, and Mutation Toolbars

### Checkpoint: Phase 4
- [ ] All table inspection grids use canonical `Table` and `Badge`
- [ ] Mutation toolbar condition input and WHERE badges preserve inline query editing

### Phase 5: Specialized Workstations
- [ ] Task 9: Standardize ER Diagram Toolbar, Controls, and Design Panels
- [ ] Task 10: Standardize Schema Compare, Data Diff, and Sync Preview Surfaces
- [ ] Task 11: Standardize Audit, FDW, and Event Trigger Management Surfaces

### Checkpoint: Phase 5
- [ ] ER zoom, pan, fit, and design draft controls use canonical `Button`
- [ ] Schema compare chips, diff filters, and input fields use canonical primitives

### Phase 6: Legacy Codebase Elimination
- [ ] Task 12: Audit Call-sites and Deprecate/Remove `crates/ui/src/components/legacy.rs`

### Checkpoint: Complete
- [ ] Zero usages of legacy button/input/frame helpers across `crates/ui/src/`
- [ ] All workspace quality gates pass (`cargo clippy`, `cargo test`, `cargo build --release -p db-pro-native`)

## Risks and Mitigations
| Risk | Impact | Mitigation |
|------|--------|------------|
| Layout clipping from button padding differences | Medium | Use `ButtonSize::Sm` or `ButtonSize::IconSm` for compact toolbars; verify bounding boxes against existing screenshots. |
| Context menu trigger loss during button refactoring | High | Ensure `.show(ui).context_menu(...)` or `dropdown_menu` is preserved on every migrated button. |
| Keyboard shortcut / IME regression on Input refactor | High | Validate `id_salt`, Tab-order, and IME handling on replaced singleline/multiline inputs. |
| ActivityBar tab state drift when unifying rail | Medium | Add unit tests verifying `ActivityBarItemKind` bidirectional mapping to `Activity` enum. |

## Open Questions
- None blocking planning. All target core components exist in `crates/ui/src/components/` and are verified in Component Gallery.
