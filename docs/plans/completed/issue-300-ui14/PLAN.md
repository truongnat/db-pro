# Issue #300 — UI14 Responsive Density and Overflow Hardening

## Goal

Harden the native egui surfaces against narrow windows, long content, dense navigation and resize
cycles without hiding primary actions or losing keyboard focus.

## Scope

This issue is delivered in focused slices, each with source tests where practical and runtime evidence:

1. Query workspace header/path/tab overflow and action reachability.
2. Sidebar/header long-name and narrow-width behavior.
3. Dialog/form minimum sizes and long validation/error content.
4. Grid, output, schema and ER surfaces: scroll ownership and dense content.
5. Resize persistence and final stress-matrix verification.

## Required stress matrix

- Windows: 1024x700 sanity floor, 1280x800, 1440x900, 1920x1080, maximized/fullscreen.
- Long content: 60+ character names, Vietnamese/Japanese/Unicode, many tabs/sidebar sections,
  30+ columns, large errors/tool output, long SQL paths/breadcrumbs, verbose definitions and dense ER labels.

## Acceptance criteria

- No critical surface is unusable at 1280x800.
- No toolbar or dialog spills outside the window.
- Long content truncates, wraps or scrolls according to an explicit contract; primary actions remain reachable.
- Keyboard focus remains reachable after overflow/scroll.
- Splitter and panel sizes remain stable through window resize.
- Runtime evidence includes deliberate worst-case fixtures at required sizes.

## Non-goals

- No redesign of the product visual language.
- No changes to database/runtime behavior unless required to expose a deterministic UI fixture.
- No broad refactor unrelated to responsive behavior.
