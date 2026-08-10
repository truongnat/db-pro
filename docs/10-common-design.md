# 10 — DB Client — Common Design

---

## 1. Design System Foundation

### 1.1 Design Token Architecture

All design tokens follow a 4-step hierarchy:

```
Primitive Tokens → Alias Tokens → Component Tokens → CSS Custom Properties
```

#### Primitive Tokens (raw values)

| Token | Value |
|---|---|
| `--color-blue-50` | `#e3f2fd` |
| `--color-blue-100` | `#bbdefb` |
| `--color-blue-500` | `#1976d2` |
| `--color-blue-700` | `#1565c0` |
| `--color-red-50` | `#ffebee` |
| `--color-red-500` | `#c62828` |
| `--color-green-50` | `#e8f5e9` |
| `--color-green-500` | `#2e7d32` |
| `--color-gray-50` | `#fafafa` |
| `--color-gray-100` | `#f5f5f5` |
| `--color-gray-300` | `#e0e0e0` |
| `--color-gray-500` | `#9e9e9e` |
| `--color-gray-700` | `#616161` |
| `--color-gray-900` | `#212121` |
| `--spacing-1` | `4px` |
| `--spacing-2` | `8px` |
| `--spacing-3` | `16px` |
| `--spacing-4` | `24px` |
| `--spacing-5` | `32px` |
| `--radius-sm` | `4px` |
| `--radius-md` | `8px` |
| `--radius-lg` | `12px` |
| `--radius-full` | `9999px` |
| `--shadow-sm` | `0 1px 3px rgba(0,0,0,0.12)` |
| `--shadow-md` | `0 4px 6px rgba(0,0,0,0.1)` |
| `--shadow-lg` | `0 10px 25px rgba(0,0,0,0.15)` |
| `--font-size-xs` | `0.75rem` |
| `--font-size-sm` | `0.875rem` |
| `--font-size-md` | `1rem` |
| `--font-size-lg` | `1.25rem` |
| `--font-size-xl` | `1.5rem` |
| `--font-family-body` | `Roboto, sans-serif` |
| `--font-family-mono` | `JetBrains Mono, Fira Code, monospace` |

#### Alias Tokens (semantic names)

| Token | Maps To | Usage |
|---|---|---|
| `--color-primary` | `--color-blue-500` | Primary actions, links |
| `--color-primary-hover` | `--color-blue-700` | Hover state |
| `--color-primary-bg` | `--color-blue-50` | Primary backgrounds |
| `--color-success` | `--color-green-500` | Success states |
| `--color-success-bg` | `--color-green-50` | Success backgrounds |
| `--color-error` | `--color-red-500` | Error states |
| `--color-error-bg` | `--color-red-50` | Error backgrounds |
| `--color-text` | `--color-gray-900` | Primary text |
| `--color-text-secondary` | `--color-gray-500` | Secondary text |
| `--color-text-disabled` | `--color-gray-300` | Disabled text |
| `--color-border` | `--color-gray-300` | Borders |
| `--color-bg` | `--color-gray-100` | Page backgrounds |
| `--color-surface` | `#ffffff` | Cards, dialogs |
| `--spacing-unit` | `--spacing-3` | Default spacing unit |
| `--radius-default` | `--radius-md` | Default border radius |

#### Component Tokens (component-specific)

| Token | Value | Component |
|---|---|---|
| `--button-height` | `36px` | Button |
| `--button-padding-x` | `--spacing-3` | Button |
| `--input-height` | `--button-height` | Input |
| `--input-padding-x` | `--spacing-2` | Input |
| `--card-padding` | `--spacing-3` | Card |
| `--card-radius` | `--radius-md` | Card |
| `--sidebar-width` | `240px` | Sidebar |
| `--sidebar-collapsed-width` | `64px` | Sidebar |
| `--appbar-height` | `56px` | AppBar |
| `--statusbar-height` | `28px` | StatusBar |
| `--dialog-max-width` | `600px` | Dialog |
| `--toast-duration` | `4000ms` | Toast |
| `--transition-fast` | `150ms ease` | Micro-interactions |
| `--transition-normal` | `250ms ease` | State changes |

#### CSS Custom Properties (runtime)

All alias tokens are exposed as CSS custom properties on `:root` for use in any component:

```css
:root {
  --color-primary: var(--alias-color-primary);
  --color-primary-hover: var(--alias-color-primary-hover);
  /* ... all alias tokens ... */
}

[data-theme="dark"] {
  --color-text: var(--alias-color-text-dark);
  --color-bg: var(--alias-color-bg-dark);
  /* ... dark theme overrides ... */
}
```

### 1.2 Theme System

#### Light Theme (default)

```css
:root {
  --color-bg: #f5f5f5;
  --color-surface: #ffffff;
  --color-text: #212121;
  --color-text-secondary: #757575;
  --color-border: #e0e0e0;
  --color-overlay: rgba(0, 0, 0, 0.5);
}
```

#### Dark Theme

```css
[data-theme="dark"] {
  --color-bg: #1e1e1e;
  --color-surface: #2e2e2e;
  --color-text: #e0e0e0;
  --color-text-secondary: #a0a0a0;
  --color-border: #424242;
  --color-overlay: rgba(0, 0, 0, 0.7);
}
```

#### Theme Switching

- Theme stored in Zustand store (`theme.store.ts`)
- Persisted to `localStorage` under key `db-client-theme`
- Applied via `data-theme` attribute on `<html>` element
- MUI theme created from CSS custom properties
- Transition: `transition: background-color var(--transition-normal)`

## 2. Icon System

### 2.1 Icon Library

- **Primary**: Material Icons (included in MUI)
- **Secondary**: Custom SVG icons for app-specific icons
- **Size**: 18px default, 16px small, 24px large
- **Color**: Inherits from parent `color` CSS property

### 2.2 Icon Naming Convention

```
db-icon-{category}-{name}

Examples:
- db-icon-action-connect
- db-icon-action-execute
- db-icon-action-export
- db-icon-status-connected
- db-icon-status-disconnected
- db-icon-status-error
- db-icon-nav-connections
- db-icon-nav-query
- db-icon-nav-schema
- db-icon-nav-data
- db-icon-nav-export
```

### 2.3 Icon Usage Rules

- Always pair icon with text label unless icon is universally understood (e.g., ✓, ✕)
- Icon-only buttons must have `aria-label`
- Decorative icons must have `aria-hidden="true"`
- Icons must have `focusable="false"` when used in interactive elements
- Never use icon as the only indicator of state (always add text or badge)

## 3. Animation & Motion Design

### 3.1 Animation Principles

| Principle | Description |
|---|---|
| Purposeful | Every animation must serve a functional purpose (feedback, transition, emphasis) |
| Fast | Animations should complete within 300ms |
| Natural | Use easing curves that match physical motion |
| Consistent | Same animation patterns used across the app |
| Respect preferences | Disable animations when `prefers-reduced-motion` is set |

### 3.2 Animation Specifications

| Animation | Duration | Easing | Usage |
|---|---|---|---|
| Fade in | 200ms | `ease-out` | Dialogs, tooltips, popovers |
| Fade out | 150ms | `ease-in` | Dialogs, toasts, popovers |
| Slide in (right) | 250ms | `ease-out` | Sidebar, panels |
| Slide out (right) | 200ms | `ease-in` | Sidebar, panels |
| Slide in (down) | 200ms | `ease-out` | Dropdowns, menus |
| Slide out (up) | 150ms | `ease-in` | Dropdowns, menus |
| Scale in | 200ms | `ease-out` | Tooltips, badges |
| Scale out | 150ms | `ease-in` | Tooltips, badges |
| Highlight | 300ms | `ease-out` | Success/error feedback on cells |
| Skeleton shimmer | 1.5s | `linear` | Loading placeholders |
| Spinner | 1s | `linear` (infinite) | Loading states |
| Page transition | 250ms | `ease-out` | Route changes |

### 3.3 Easing Curves

| Name | Curve | Usage |
|---|---|---|
| `ease-out` | `cubic-bezier(0.0, 0.0, 0.2, 1)` | Enter animations, fades |
| `ease-in` | `cubic-bezier(0.4, 0.0, 1.0, 1)` | Exit animations |
| `ease-in-out` | `cubic-bezier(0.4, 0.0, 0.2, 1)` | State transitions |
| `linear` | `linear` | Spinners, progress bars |

### 3.4 Reduced Motion

```css
@media (prefers-reduced-motion: reduce) {
  *, *::before, *::after {
    animation-duration: 0.01ms !important;
    animation-iteration-count: 1 !important;
    transition-duration: 0.01ms !important;
  }
}
```

## 4. Typography System

### 4.1 Font Stack

| Usage | Font Family | Fallback |
|---|---|---|
| Body text | `Roboto` | `sans-serif` |
| Code / SQL | `JetBrains Mono` | `Fira Code`, `monospace` |
| UI elements | `Roboto` | `sans-serif` |

### 4.2 Type Scale

| Token | Size | Weight | Line Height | Usage |
|---|---|---|---|---|
| `--text-xs` | 12px | 400 | 1.4 | Captions, labels, hints |
| `--text-sm` | 14px | 400 | 1.5 | Body text, table cells |
| `--text-md` | 16px | 400 | 1.5 | Subheadings, form labels |
| `--text-lg` | 18px | 600 | 1.4 | Section headings |
| `--text-xl` | 24px | 600 | 1.3 | Page titles |
| `--text-code` | 13px | 400 | 1.5 | Inline code, grid data |
| `--text-code-block` | 14px | 400 | 1.6 | SQL editor, DDL viewer |

### 4.3 Text Colors

| Token | Color | Usage |
|---|---|---|
| `--text-primary` | `--color-text` | Primary text |
| `--text-secondary` | `--color-text-secondary` | Secondary text, hints |
| `--text-disabled` | `--color-text-disabled` | Disabled text |
| `--text-link` | `--color-primary` | Links |
| `--text-error` | `--color-error` | Error text |
| `--text-success` | `--color-success` | Success text |

### 4.4 Text Utilities

| Class | Effect |
|---|---|
| `.text-truncate` | Single line truncation with ellipsis |
| `.text-wrap` | Allow text wrapping |
| `.text-nowrap` | Prevent text wrapping |
| `.text-mono` | Use monospace font |
| `.text-bold` | Font weight 600 |
| `.text-italic` | Font style italic |
| `.text-underline` | Text underline |

## 5. Spacing & Layout System

### 5.1 Spacing Scale

| Token | Value | Usage |
|---|---|---|
| `--space-1` | 4px | Tightest spacing |
| `--space-2` | 8px | Small spacing |
| `--space-3` | 16px | Default spacing |
| `--space-4` | 24px | Section spacing |
| `--space-5` | 32px | Page spacing |
| `--space-6` | 48px | Large section spacing |
| `--space-8` | 64px | Page-level spacing |

### 5.2 Layout Utilities

| Class | Effect |
|---|---|
| `.flex` | `display: flex` |
| `.flex-col` | `flex-direction: column` |
| `.flex-row` | `flex-direction: row` |
| `.flex-wrap` | `flex-wrap: wrap` |
| `.items-center` | `align-items: center` |
| `.justify-between` | `justify-content: space-between` |
| `.justify-center` | `justify-content: center` |
| `.gap-1` to `.gap-4` | `gap: var(--space-N)` |
| `.p-1` to `.p-4` | `padding: var(--space-N)` |
| `.px-1` to `.px-4` | `padding-left/right: var(--space-N)` |
| `.py-1` to `.py-4` | `padding-top/bottom: var(--space-N)` |
| `.m-1` to `.m-4` | `margin: var(--space-N)` |
| `.mx-auto` | `margin-left/right: auto` |
| `.w-full` | `width: 100%` |
| `.h-full` | `height: 100%` |
| `.min-h-0` | `min-height: 0` (for flex children) |
| `.overflow-hidden` | `overflow: hidden` |
| `.overflow-auto` | `overflow: auto` |
| `.overflow-scroll` | `overflow: scroll` |

### 5.3 Grid System

| Class | Effect |
|---|---|
| `.grid` | `display: grid` |
| `.grid-cols-1` to `.grid-cols-12` | `grid-template-columns: repeat(N, 1fr)` |
| `.col-span-1` to `.col-span-12` | `grid-column: span N` |
| `.gap-1` to `.gap-4` | `gap: var(--space-N)` |

## 6. Common Patterns

### 6.1 Pattern: Confirm Before Destructive Action

```
User clicks Delete → ConfirmDialog appears
  → "Are you sure you want to delete this?"
  → [Cancel] [Delete]
  → If Cancel → Dialog closes, no action
  → If Delete → Action executed, toast confirmation shown
```

### 6.2 Pattern: Optimistic Update

```
User edits cell → UI updates immediately (optimistic)
  → API call sent in background
  → If success → Toast "Saved" (subtle)
  → If error → Revert UI, show error tooltip
```

### 6.3 Pattern: Loading States

```
Action triggered
  → If < 200ms → No loading state needed
  → If 200ms - 2s → Show spinner in relevant location
  → If > 2s → Show progress dialog with cancel option
  → On completion → Remove loading state, show result
  → On error → Show error state with retry option
```

### 6.4 Pattern: Empty State

```
No data available
  → Show illustration (simple icon, not photo)
  → Show descriptive text: "No X yet"
  → Show call-to-action button if applicable
  → If data can be loaded, show "Refresh" button
```

### 6.5 Pattern: Error State

```
Error occurred
  → Show error message (i18n key, not raw message)
  → If recoverable → Show "Retry" button
  → If not recoverable → Show "Dismiss" button
  → Log error with context (timestamp, action, connection_id)
  → Allow user to continue working
```

### 6.6 Pattern: Search

```
User types in search
  → Debounce 300ms
  → Show results as user types
  → If no results → Show "No results for '{query}'"
  → If loading → Show skeleton/loading indicator
  → If error → Show error message with retry
  → Keyboard: Esc to clear, Enter to submit
```

### 6.7 Pattern: Filter

```
User opens filter panel
  → Show filter options (text, number, date, select)
  → Apply filter on change (no "Apply" button)
  → Show active filters as chips above grid
  → Click chip to remove filter
  → Clear all filters button available
```

### 6.8 Pattern: Pagination

```
Grid shows first page of data
  → User scrolls to bottom or clicks page number
  → Load next page (cursor-based, not offset-based)
  → Show page info: "Showing 1-25 of 1,234"
  → Page size selector: 25 / 50 / 100 / 200
  → Jump to page input available
```

### 6.9 Pattern: Toast Notification

```
Action completed or error occurred
  → Show toast in bottom-right corner
  → Auto-dismiss after timeout (success: 4s, error: 8s)
  → Click to expand (show details for errors)
  → Click close button to dismiss early
  → Stack multiple toasts (max 3)
  → Pause auto-dismiss on hover
```

### 6.10 Pattern: Dialog

```
User triggers dialog action
  → Overlay appears (darkened background)
  → Dialog slides in from center (scale + fade)
  → Focus moves to first interactive element
  → Esc key closes dialog (unless modal with confirmation)
  → Click overlay closes dialog (unless modal)
  → Dialog has header, body, footer (actions)
  → Dialog size depends on content type
```

## 7. Design System File Structure

```
frontend/src/commons/
├── design/
│   ├── tokens/
│   │   ├── primitive.ts        # Primitive token values
│   │   ├── alias.ts            # Semantic token aliases
│   │   ├── component.ts        # Component-specific tokens
│   │   └── index.ts            # Re-export all tokens
│   ├── themes/
│   │   ├── light.ts            # Light theme configuration
│   │   ├── dark.ts             # Dark theme configuration
│   │   └── index.ts            # Theme creation, switching logic
│   ├── icons/
│   │   ├── index.ts            # Icon registry
│   │   ├── db-icon.tsx         # Custom SVG icon component
│   │   └── material-icons.ts   # Material Icons wrapper
│   ├── animations/
│   │   ├── index.ts            # Animation utilities
│   │   ├── transitions.ts      # Transition definitions
│   │   └── reduced-motion.ts   # prefers-reduced-motion handling
│   ├── typography/
│   │   ├── index.ts            # Typography utilities
│   │   └── font-loading.ts     # Font loading strategy
│   ├── layout/
│   │   ├── index.ts            # Layout utilities
│   │   ├── grid.ts             # Grid system utilities
│   │   └── spacing.ts          # Spacing utilities
│   └── index.ts                # Design system entry point
```

## 8. Design System Integration

### 8.1 MUI Theme Integration

```typescript
// commons/design/themes/index.ts

import { createTheme } from '@mui/material/styles';

export function createAppTheme(mode: 'light' | 'dark') {
  return createTheme({
    palette: {
      mode,
      primary: { main: 'var(--color-primary)' },
      error: { main: 'var(--color-error)' },
      success: { main: 'var(--color-success)' },
      warning: { main: 'var(--color-warning)' },
      info: { main: 'var(--color-info)' },
      background: {
        default: 'var(--color-bg)',
        paper: 'var(--color-surface)',
      },
      text: {
        primary: 'var(--color-text)',
        secondary: 'var(--color-text-secondary)',
      },
    },
    typography: {
      fontFamily: 'var(--font-family-body)',
      h1: { fontSize: 'var(--text-xl)', fontWeight: 600 },
      h2: { fontSize: 'var(--text-lg)', fontWeight: 600 },
      body1: { fontSize: 'var(--text-sm)', lineHeight: 1.5 },
      body2: { fontSize: 'var(--text-xs)', lineHeight: 1.4 },
    },
    shape: {
      borderRadius: Number('var(--radius-default)'.replace('px', '')) || 8,
    },
    spacing: 8,
  });
}
```

### 8.2 CSS Custom Properties Injection

All design tokens are injected as CSS custom properties on `:root` at app initialization:

```typescript
// app/providers/theme.provider.tsx

import { useEffect } from 'react';
import { tokens } from '../../design/tokens';

export function ThemeProvider({ children }) {
  useEffect(() => {
    const root = document.documentElement;
    Object.entries(tokens).forEach(([key, value]) => {
      root.style.setProperty(key, value);
    });
  }, []);

  return <>{children}</>;
}
```

### 8.3 Design System Testing

| Test | Tool | Coverage |
|---|---|---|
| Visual regression | `storybook` + `chromatic` (future) | All components |
| Token validation | Custom script | All tokens defined, no unused tokens |
| Contrast ratio | `axe-core` | All text/background combinations ≥ 4.5:1 |
| Theme switching | `vitest` | Light ↔ Dark transitions work correctly |
| Reduced motion | Manual test | Animations disabled when preference set |
| Responsive layout | `playwright` | All breakpoints render correctly |

## 9. Design System Maintenance

### 9.1 Adding New Tokens

1. Add primitive token to `design/tokens/primitive.ts`
2. Add alias token to `design/tokens/alias.ts` if needed
3. Add component token to `design/tokens/component.ts` if needed
4. Export from `design/tokens/index.ts`
5. Update CSS custom properties in theme provider
6. Add to TypeScript types in `design/tokens/types.ts`

### 9.2 Adding New Components

1. Create component in `commons/components/`
2. Add component tokens if needed
3. Write unit tests for component
4. Write component tests for all states
5. Document usage in component README
6. Add to Storybook (future)

### 9.3 Design Review Checklist

- [ ] All user-facing strings use i18n keys
- [ ] All interactive elements have ARIA labels
- [ ] Color contrast meets WCAG AA (4.5:1)
- [ ] Keyboard navigation works for all features
- [ ] Focus indicators are visible
- [ ] Loading states are handled
- [ ] Error states are handled
- [ ] Empty states are handled
- [ ] Reduced motion is respected
- [ ] Dark theme works correctly
- [ ] Responsive layout works at all breakpoints
- [ ] Typography is consistent
- [ ] Spacing follows the design system scale
