---
schema_version: 1
id: "MEM-PLAN-native-shadcn-ui-system-93b6a954e041"
title: "Plan archive: Native Shadcn Ui System"
type: pattern
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, pattern, native-shadcn-ui-system]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "93b6a954e041987d3dd699b0ed3e93d90c94070debd4975cee93eb86fc3bad68"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/native-shadcn-ui-system/PLAN.md"
  - "docs/plans/active/native-shadcn-ui-system/FINDINGS.md"
  - "docs/plans/active/native-shadcn-ui-system/VERIFICATION.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/native-shadcn-ui-system/`. Loại suy luận: `pattern`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# Native Shadcn-like Common UI System & Component Gallery

## 1. Context & Motivation
DB Pro Native (`crates/ui`) is built with `egui` and Rust. While the web frontend uses `shadcn/ui` and Tailwind CSS, the native desktop application currently relies on ad-hoc buttons and input helpers in `components.rs`. To make the native desktop IDE look as polished, modern, and unified as `shadcn/ui`, we need a formal **Native Component Design System** and a live **Component Showcase / Preview Screen** (Gallery) where developers and designers can inspect, test, and interact with every UI primitive across Light and Dark themes.

## 2. Goals & Deliverables
1. **Shadcn-inspired Token Contract in Rust**:
   - Explicit variants, sizes, and interaction states for every primitive.
   - 100% theme-adaptive using `DbProTheme` (Light & Dark tokens).
   - Pixel-perfect typography (Lucide vector icons + proportional font pairing).
2. **Comprehensive Component Library**:
   - **Buttons**: `Default` (Primary), `Secondary`, `Outline`, `Ghost`, `Destructive`, `Link`; sizes: `Sm`, `Default`, `Lg`, `Icon`; states: `normal`, `hover`, `active`, `disabled`, `loading` (with animated spinner).
   - **Badges / Status Pills**: `Default`, `Secondary`, `Outline`, `Destructive`, `Success`, `Warning`, `Info`; with optional dot indicator or leading Lucide icon.
   - **Form Controls**:
     - Single-line Input with placeholder, helper text, error text/state, leading/trailing icon, and clear button.
     - Search Input with `⌘K` badge and clear action.
     - Password Input with eye reveal toggle.
     - Multiline Textarea with character count and auto/min height.
     - Custom Checkbox with check icon, label, description, and indeterminate state.
     - iOS/Shadcn-style animated Toggle Switch.
     - Radio Group with circle indicators.
     - Slider with live numerical bubble/readout.
     - Select / Dropdown with popover menu, search filter, and checkmark.
   - **Cards & Containers**:
     - `Card`, `CardHeader`, `CardTitle`, `CardDescription`, `CardContent`, `CardFooter`.
     - Stat / Metric Card (title, large value, trend badge +/- %).
   - **Feedback & Alerts**:
     - Alert (`Default`, `Info`, `Success`, `Warning`, `Destructive`) with Lucide icon, title, message body, and dismiss action.
     - Progress Bar (determinate % and indeterminate pulse).
     - Spinner loading indicator.
     - Tooltip helpers.
   - **Navigation & Layout**:
     - Segmented Control Tabs (pill style).
     - Line Tabs (underline style with active indicator).
     - Breadcrumbs with `/` or `>` chevrons.
     - Kbd shortcut tags (`⌘K`, `⌘P`, `Esc`, `↵ Enter`).
     - Separators (horizontal, vertical, with optional label like "OR").
   - **Data Display**:
     - Styled Table / Mini Grid with column headers, striped alternating rows, status badges, and action buttons.
3. **Interactive Component Gallery Screen**:
   - A dedicated screen (`WorkspaceTab::ComponentShowcase` / Activity Bar icon / Command Palette `⌘⇧P -> Open Component Gallery`).
   - Categorized side navigation (Overview, Buttons, Badges, Forms & Inputs, Feedback, Cards, Navigation, Data Display).
   - Real-time Light / Dark mode toggle switch right in the gallery header.
   - Interactive live controls (type into inputs, toggle switches, adjust sliders, click buttons with feedback).

## 3. Non-Goals
- Replacing the database engine or IPC bridge.
- Altering existing database querying or introspection logic.

## 4. Architecture & File Structure
```text
crates/ui/src/
├── components/           # Modular Shadcn-like UI system
│   ├── mod.rs            # Re-exports and prelude
│   ├── button.rs         # Button variants, sizes, loading states
│   ├── badge.rs          # Badges & status pills
│   ├── input.rs          # Text inputs, search, password, textarea
│   ├── selection.rs      # Checkbox, switch, radio group, slider
│   ├── select.rs         # Dropdown select & popover menu
│   ├── card.rs           # Card composite primitives & metric cards
│   ├── alert.rs          # Alert banners & notices
│   ├── feedback.rs       # Progress bars, spinners, kbd badges, separators
│   ├── tabs.rs           # Segmented pill tabs & underline tabs
│   └── table.rs          # Styled data table component
├── component_gallery_view.rs # The full interactive preview / showcase screen
├── components.rs         # Backward compatibility re-exports for existing views
└── app.rs                # Integration into WorkspaceTab & Palette
```

## 5. Acceptance Criteria
1. All components render with clean 1px borders, harmonious padding, and exact color contrast in both Light and Dark mode.
2. The Component Gallery is accessible directly via topbar tab / quick action / command palette.
3. Every component has interactive state demonstrations (hover, active, disabled).
4. `cargo check --workspace` and `cargo test --workspace` pass with 0 warnings/errors.
5. Runtime screenshot evidence captured and verified.


## Source: `FINDINGS.md`

# Findings: Native Shadcn-like Common UI System

## Initial Findings & Baseline Analysis

### Finding 1 (Design Consistency)
- **Status**: Identified.
- **Evidence**: `crates/ui/src/components.rs` currently contains only primitive button helpers (`primary_button`, `secondary_button`, `ghost_button`, `compact_button`) and rudimentary inputs without explicit variants, size scales, loading states, error boundaries, or helper text.
- **Severity**: P2 (Design System & Polish).
- **Resolution**: Build a dedicated modular component system mirroring the standard API conventions of `shadcn/ui` (Variants, Sizes, Compound parts).

### Finding 2 (Component Discoverability & Developer Experience)
- **Status**: Identified.
- **Evidence**: Previously there was no visual gallery or Storybook-like environment in the native app to inspect and test UI components in isolation across Light and Dark themes.
- **Severity**: P2 (Developer Experience & Visual Quality).
- **Resolution**: Provide a dedicated `ComponentShowcase` workspace tab in `DbProApp` that acts as a full-featured living style guide and component playground.


## Source: `VERIFICATION.md`

# Verification: Native Shadcn-like Common UI System

## Quality Gates Checklist
- [x] `cargo check --workspace` -> Clean 0 errors
- [x] `cargo test --workspace` -> All 56 tests passed
- [x] `cargo fmt --all -- --check` -> Clean, formatted perfectly
- [x] `cargo clippy --workspace --all-targets` -> Clean 0 warnings
- [x] Component Gallery renders without visual defects
- [x] Light & Dark themes verified with screenshots

## Evidence Log
1. **Light Mode Preview**:
   - Location: `/Users/truongdev/.gemini/antigravity-cli/brain/25cd50c1-1256-4ed3-b038-791470b13b9e/component_gallery_final_light.png`
   - Verified: Clean layout, proper button sizing, primary/secondary/outline/ghost/destructive/link buttons, badges with dot indicators, 2-column form layout, error message styling, clear buttons, password reveal, checkboxes, animated switches, radio buttons, select dropdown, and slider.
2. **Dark Mode Preview**:
   - Location: `/Users/truongdev/.gemini/antigravity-cli/brain/25cd50c1-1256-4ed3-b038-791470b13b9e/component_gallery_dark.png`
   - Verified: Seamless theme adaptability, dark surface elevation tokens, subtle borders, high contrast text, and consistent purple accent `#7C3AED` across both modes.
3. **Interactive Capabilities**:
   - Filter tabs: Filter between "All", "Buttons", "Badges", "Forms & Inputs", "Selection", "Cards", "Alerts", "Feedback", "Navigation", "Data Tables".
   - Theme toggle: Live flip between Light and Dark mode directly within the window.
   - Live inputs: Typing, clearing text with `x`, toggling password eye, adjusting sliders.

