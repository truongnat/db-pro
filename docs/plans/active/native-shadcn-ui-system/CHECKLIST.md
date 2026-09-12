# Checklist: Native Shadcn-like Common UI System & Component Gallery

- [x] **Phase 1: Component Primitives Architecture (`crates/ui/src/components/`)**
  - [x] `button.rs`: Implement `ButtonVariant` (Default, Secondary, Outline, Ghost, Destructive, Link), `ButtonSize` (Sm, Default, Lg, Icon, IconSm), and `ShadcnButton` builder with loading spinner & disabled states.
  - [x] `badge.rs`: Implement `BadgeVariant` (Default, Secondary, Outline, Destructive, Success, Warning, Info), leading dots, and icons.
  - [x] `input.rs`: Implement styled `ShadcnInput`, `PasswordInput` with toggle, `SearchInput` with clear button & `⌘K` badge, and `ShadcnTextarea`.
  - [x] `selection.rs`: Implement `ShadcnCheckbox`, iOS-style `ShadcnSwitch`, `ShadcnRadioGroup`, and smooth `ShadcnSlider`.
  - [x] `select.rs`: Implement `ShadcnSelect` dropdown with custom trigger, popover list, and checkmark.
  - [x] `card.rs`: Implement `ShadcnCard`, `CardHeader`, `CardTitle`, `CardDescription`, and `MetricCard`.
  - [x] `alert.rs`: Implement `ShadcnAlert` with variants (`Default`, `Info`, `Success`, `Warning`, `Destructive`), icon, title, description, and dismiss action.
  - [x] `feedback.rs`: Implement `ShadcnProgress`, `ShadcnSpinner`, `KbdBadge`, and `separator_with_text`.
  - [x] `tabs.rs`: Implement `SegmentedTabs` and `UnderlineTabs`.
  - [x] `table.rs`: Implement styled mini data table with headers, alternating rows, and column alignment.
  - [x] `mod.rs` & `legacy.rs`: Export all primitives and maintain 100% backward compatibility with previous call sites.

- [x] **Phase 2: Interactive Component Gallery Screen (`component_gallery_view.rs`)**
  - [x] Create `ComponentGalleryState` with live state values (text input, slider, toggles, active tab, selected option).
  - [x] Implement categorized layout (Category filter tabs: All, Buttons, Badges, Forms, Selection, Cards, Alerts, Feedback, Navigation, Tables).
  - [x] Implement live Dark/Light mode toggle directly in the gallery toolbar.
  - [x] Implement interactive controls (loading button simulation, password reveal, clear buttons, live slider value, select dropdown).

- [x] **Phase 3: Integration into DB Pro Native Shell**
  - [x] Add `WorkspaceTab::ComponentGallery` in `app.rs`.
  - [x] Add entry point in Activity Bar (palette icon) and Command Palette (`⌘⇧P -> Open Component Gallery`).
  - [x] Add quick-access button on the Welcome screen to open the gallery.
  - [x] Tab management supports switching, closing, and re-opening.

- [x] **Phase 4: Automated Verification & Quality Gates**
  - [x] Run `cargo check --workspace` -> PASSED (0 errors).
  - [x] Run `cargo test --workspace` -> PASSED (56 unit tests passed).
  - [x] Run `cargo fmt --all -- --check` -> PASSED.
  - [x] Run `cargo clippy --workspace --all-targets` -> PASSED (0 warnings).

- [x] **Phase 5: Runtime Verification & Visual Proof**
  - [x] Run native app on macOS with `cargo run -p db-pro-native`.
  - [x] Capture screenshot of Component Gallery in Light mode (`component_gallery_final_light.png`).
  - [x] Capture screenshot of Component Gallery in Dark mode (`component_gallery_dark.png`).
  - [x] Record pixel evidence in `VERIFICATION.md`.
