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
