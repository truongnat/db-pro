# Design Specification: ChatGPT / OpenAI Monochrome Aesthetic for DB Pro

## Problem Statement
How might we transform DB Pro's entire design token system and core component primitives into the minimalist, breathing, high-contrast monochrome aesthetic of the ChatGPT macOS desktop application, while preserving the high-density precision required by a professional database IDE?

---

## 1. Design Direction: OpenAI Monochrome Minimalist

The interface is calibrated to match the visual language of the **ChatGPT macOS Desktop Application**:
- **Canvas / Surfaces**: Warm, deep pitch-charcoal background (`#212121` app, `#171717` sidebar, `#2F2F2F` elevated cards/bubbles).
- **Primary Accent**: Pure high-contrast monochrome (`#ECECEC` solid pill with `#0D0D0D` crisp text in Dark Mode; `#0D0D0D` solid with `#FFFFFF` text in Light Mode).
- **Secondary & Ghost Controls**: Borderless or ultra-fine borders (`rgba(255, 255, 255, 0.08)`), transitioning smoothly into soft translucent washes (`rgba(255, 255, 255, 0.06)`) on hover.
- **Corner Geometry**: Softer, organic radii: `8px` for cards/dialogs, `12px–16px` for query containers and composer boxes, `9999px` (full pill) for buttons, status chips, and model selectors.
- **Atmospheric Depth**: No harsh 1px borders between panels; elevation and separation are communicated through subtle background value steps and soft shadows.

---

## 2. Color Palette & Token Calibration

### Dark Mode (ChatGPT Canvas)

```
┌──────────────────────────────┬───────────┬────────────────────────────────────────────┐
│ Token Role                   │ Hex Value │ Semantic Description                       │
├──────────────────────────────┼───────────┼────────────────────────────────────────────┤
│ `surface_app`                │ `#212121` │ Main workspace canvas & editor background  │
│ `surface_panel`              │ `#171717` │ Sidebar drawer & navigation background     │
│ `surface_elevated`           │ `#2F2F2F` │ Cards, modals, chat bubbles, dialog bodies │
│ `surface_hover`              │ `#343434` │ Interactive hover wash                     │
│ `surface_active`             │ `#424242` │ Selected rows, active tabs, pressed state  │
│ `border_subtle`              │ `#2C2C2C` │ Ultra-soft 1px container boundary          │
│ `border_default`             │ `#3A3A3A` │ Input fields, dropdown borders             │
│ `border_focus`               │ `#ECECEC` │ Crisp focus ring (Monochrome White)        │
│ `text_primary`               │ `#ECECEC` │ Headers, active labels, primary body text  │
│ `text_secondary`             │ `#B4B4B4` │ Descriptions, column types, secondary copy │
│ `text_muted`                 │ `#707070` │ Placeholders, timestamps, row indices      │
│ `accent` (Primary Action)    │ `#ECECEC` │ White button fill, active status dot       │
│ `accent_foreground`          │ `#0D0D0D` │ Pure black text on primary white button    │
│ `accent_soft`                │ `#2E2E2E` │ Subtle selection tint                      │
│ `success`                    │ `#10A37F` │ OpenAI signature emerald (succeeded status)│
│ `warning`                    │ `#EAB308` │ Yellow-amber for caution / explain analyze │
│ `danger`                     │ `#EF4444` │ Soft coral-red for drop/delete operations  │
│ `info`                       │ `#60A5FA` │ Sky blue for information chips             │
└──────────────────────────────┴───────────┴────────────────────────────────────────────┘
```

### Light Mode (ChatGPT White Canvas)

```
┌──────────────────────────────┬───────────┬────────────────────────────────────────────┐
│ Token Role                   │ Hex Value │ Semantic Description                       │
├──────────────────────────────┼───────────┼────────────────────────────────────────────┤
│ `surface_app`                │ `#FFFFFF` │ Pure white canvas                          │
│ `surface_panel`              │ `#F9F9F9` │ Off-white sidebar                          │
│ `surface_elevated`           │ `#F4F4F4` │ Elevated cards & chat containers           │
│ `surface_hover`              │ `#EBEBEB` │ Hover fill                                 │
│ `surface_active`             │ `#E2E2E2` │ Active / pressed selection                 │
│ `border_subtle`              │ `#E5E5E5` │ Hairline divider                           │
│ `border_default`             │ `#D4D4D4` │ Input field borders                        │
│ `border_focus`               │ `#0D0D0D` │ Black focus ring                           │
│ `text_primary`               │ `#0D0D0D` │ Crisp near-black body and headers          │
│ `text_secondary`             │ `#525252` │ Secondary descriptions                     │
│ `text_muted`                 │ `#9E9E9E` │ Placeholders and hints                     │
│ `accent`                     │ `#0D0D0D` │ Solid black primary action button          │
│ `accent_foreground`          │ `#FFFFFF` │ White text on solid black button           │
└──────────────────────────────┴───────────┴────────────────────────────────────────────┘
```

---

## 3. Core Component Redesign Blueprint

### 1. `Button` (`crates/ui/src/components/button/`)
- **Default (Primary)**: Rounded pill shape (`CornerRadius::same(18)` or `9999`), solid `#ECECEC` fill with `#0D0D0D` strong typography in dark mode. Hover smoothly lightens with scale `1.0 -> 0.98` press.
- **Secondary**: Translucent surface `#2F2F2F` with `#ECECEC` text and fine border `#3A3A3A`.
- **Ghost**: No border or background; shows `#343434` fill only on hover with `PointerHand` cursor.
- **Destructive**: Soft red `#EF4444` with white text.

### 2. `Input` & `SearchInput` (`crates/ui/src/components/input/`)
- Rounded capsule container (`CornerRadius::same(8)`), filled with `#2F2F2F` in dark mode.
- Internal padding `10px 12px`, clean monochrome magnifying glass icon (`Icon::Search`).
- Clear button (`Icon::X`) appears on right when text is entered.
- When focused, paints crisp `#ECECEC` focus ring.

### 3. `Card` & `MetricCard` (`crates/ui/src/components/card/`)
- Rounded `8px–12px` corner radius.
- Background `#2F2F2F` (subtle contrast against `#212121` app background) without harsh 1px borders.

### 4. `Badge` & `StatusBadge` (`crates/ui/src/components/badge/`)
- Full pill rounding (`9999px`).
- Compact typography (`11px` medium).
- Subtle status dots (OpenAI `#10A37F` green for success, `#EF4444` for error).

### 5. `AgentComposer` & Chat Threads (`crates/ui/src/components/agent_composer/`)
- **Floating Pill Container**: Centered or dock-wide rounded box (`CornerRadius::same(16)`) with subtle border `#3A3A3A`.
- **Top Row / Header**: Clean model selector pill (e.g. `[ Claude 3.5 Sonnet ▾ ]` or `[ GPT-4o ▾ ]`).
- **Input Area**: Multiline auto-expanding textarea without native borders.
- **Action Buttons**: Circular Send button (black arrow inside white circle `#ECECEC`), transitioning to square Stop icon while generating.

### 6. `CodeBlock` & `DiffViewer` (`crates/ui/src/components/code/`)
- Dark rounded code container (`#171717` or `#1E1E1E` background).
- Top header bar displaying language tag in uppercase monospace and a clean `[ Copy code ]` button with `Icon::Copy`.

### 7. `Table` & Data Grid (`crates/ui/src/components/table/`)
- Clean, uncluttered header with borderless cells.
- Alternating subtle row hover wash (`#2F2F2F`).
- Column headers with subtle sort indicators.

---

## 4. Key Assumptions & Stress Testing

1. **Assumption 1**: High-contrast monochrome buttons (`#ECECEC` on dark background) create a modern, distraction-free IDE experience.
   - *Validation*: Verify readability and visual balance across Query Editor, Table Grid, and ER Diagram.
2. **Assumption 2**: Replacing dark-blue/neutral-gray tints with warm pitch-charcoal (`#212121` / `#171717`) reduces eye strain for long database sessions.
   - *Validation*: Test contrast ratios against WCAG AAA standard (≥ 7:1 for text).

---

## 5. Implementation Roadmap

- **Slice 1: Semantic Token & Theme Calibration (`theme.rs` & `semantic.rs`)**
  - Update `DbProTheme::dark()` and `DbProTheme::light()` color definitions to the ChatGPT palette.
  - Update radius tokens for buttons, cards, inputs, and chips.
- **Slice 2: Component Primitives Visual Refresh**
  - Button pill geometry and monochrome variants.
  - Input field background and focus ring styling.
  - Agent Composer floating capsule layout.
  - Card & CodeBlock modern header & copy button.
- **Slice 3: Verification & Visual Proof**
  - Run all automated quality gates (`cargo test`, `cargo clippy`, `cc-scan`).
  - Build and launch the native desktop application.
