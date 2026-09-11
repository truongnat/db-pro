# Core UI Modernization — Checklist

## Planning

- [x] Evidence and quality gaps recorded
- [x] Scope/non-goals explicit
- [x] Token contract (P3.1) preservation rule stated
- [x] No provider behavior changes (UI primitives are provider-agnostic)

## Implementation — Wave 1: Polish existing primitives

- [ ] `button.tsx` — shadow lift, gradient accent, smoother active state, refined `loading`
- [ ] `input.tsx` — inset surface, refined focus ring, invalid state
- [ ] `textarea.tsx` — same polish as input
- [ ] `badge.tsx` — soft tinted variants with foreground contrast
- [ ] `switch.tsx` — spring-like thumb animation
- [ ] `tabs.tsx` — smoother indicator animation
- [ ] `tooltip.tsx` — refined appearance with subtle blur

## Implementation — Wave 2: Overlay polish

- [ ] `dialog.tsx` — backdrop blur, refined scale-in, elevation upgrade
- [ ] `alert-dialog.tsx` — same polish as dialog
- [ ] `popover.tsx` — elevation upgrade, refined slide
- [ ] `dropdown-menu.tsx` — menu item micro-shift, refined submenu indicator
- [ ] `context-menu.tsx` — same polish as dropdown-menu

## Implementation — Wave 3: Missing primitives

- [ ] `skeleton.tsx`
- [ ] `spinner.tsx`
- [ ] `kbd.tsx`
- [ ] `separator.tsx`
- [ ] `toggle.tsx`
- [ ] `slider.tsx`
- [ ] `progress.tsx`
- [ ] `sheet.tsx`
- [ ] `command.tsx`
- [ ] `card.tsx`

## Implementation — Wave 4: Token refinement

- [ ] `--surface-gradient` (additive)
- [ ] `--shadow-xs/sm/md/lg/xl` (compose existing elevation)
- [ ] `--motion-spring`, `--motion-emphasized` cubic-bezier tokens
- [ ] Both themes updated, snapshot updated in `check-token-drift.mjs`

## Implementation — Wave 5: Visual primitives

- [ ] `empty-state.tsx`
- [ ] `status-dot.tsx`
- [ ] `kbd-shortcut.tsx`

## Quality gates

- [ ] `pnpm run typecheck`
- [ ] `pnpm run lint`
- [ ] `pnpm run format:check`
- [ ] `pnpm run check:tokens`
- [ ] `pnpm run test`
- [ ] `pnpm run build`

## Verification

- [ ] Manual visual sweep of every updated primitive in both themes
- [ ] Reduced-motion check (`prefers-reduced-motion: reduce` kills animations)
- [ ] `VERIFICATION.md` written with concrete before/after evidence
