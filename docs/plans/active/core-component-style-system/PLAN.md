# Core Component Style System

## State
- **State:** IMPLEMENTING
- **Branch:** `main`
- **Baseline SHA:** `dc87da1d`
- **Surface:** `crates/ui` native egui theme and core components.

## Goal
Calibrate the native core component system to a coherent dark database workstation inspired by shadcn Primitives & Controls: crisp borders, restrained surfaces, compact controls, clear focus/selection states, and a blue action accent.

## Design context
- **Users:** database developers and analysts working long sessions.
- **Jobs:** navigate schema, run queries, inspect result sets, filter data, and make controlled edits.
- **Tone:** calm, technical, precise.
- **Reference:** the supplied dense dark IDE screenshot; adapt its hierarchy rather than copying a web implementation.

## First batch scope
- `DbProTheme` dark surface, border, text, accent, focus and widget visual tokens.
- Shared spacing/radius/control-size tokens used by core controls.
- `Button`, `Input` family, `Card`, and shared focus/interaction treatment.
- Characterization tests for token relationships and existing component behavior.
- Runtime capture at 1280×800, 1440×900, and 1920×1080 for the affected component gallery/shell surface when available.

## Non-goals
- No database/runtime/provider behavior changes.
- No broad view-by-view restyle in this batch.
- No new component framework or web/Tauri restoration.
- Preserve public Rust APIs and interaction semantics unless a separate finding proves a correctness issue.

## Design principles
1. Surface hierarchy uses quiet value steps and 1px borders instead of heavy cards or glow.
2. Controls are compact but keyboard/focus states remain visible.
3. Blue is reserved for action, focus and active navigation; semantic colors communicate status only.
4. Shared tokens belong in `DbProTheme`/`tokens.rs`; views do not invent colors.
5. Dense data remains scannable through alignment, row rhythm and restrained selection fills.

## Acceptance criteria
- Core components use semantic theme/tokens with no new raw color literals in views.
- Dark mode matches the reference direction while light mode remains readable.
- Existing public component APIs and focused behavior tests remain intact.
- Rust fmt/check/clippy/tests/native release build pass.
- Runtime visual evidence is recorded separately from automated evidence.

## Provider impact
N/A — native UI styling only; PostgreSQL and SQLite behavior are unchanged.

## Tổng kết bằng tiếng Việt
Batch đầu tập trung vào design system và core primitives, không restyle toàn bộ view cùng lúc. Mục tiêu là nền dark workstation, border rõ, control compact, focus/selection dễ nhận biết và giữ nguyên API/hành vi hiện tại.
