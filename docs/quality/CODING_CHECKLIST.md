# DB Pro — Coding Review Checklist

A durable, actionable review checklist for self-review and PR code review across all DB Pro crates. Grounded in `AGENTS.md`, clean code standards, component layer structure guidelines, and database safety policies.

Mark severity levels when reporting issues:
- **P0**: Catastrophic data loss, security-critical vulnerability, application unusable (blocking).
- **P1**: Data corruption, wrong database mutation, unsafe SQL, broken transaction semantics, stale state causing incorrect behavior, swallowed errors / unchecked `unwrap()` on data paths, missing required component layers / architecture violation (blocking).
- **P2**: Non-blocking UX inconsistency, maintainability issue, missing edge-case coverage, naming/comment drift, debt requiring follow-up (fix in PR or record in backlog).

---

## Áp dụng lần đầu: Accordion

Review source-level cho `crates/ui/src/components/accordion/`, đối chiếu với kiến trúc `Select`:

- [x] Có `mod.rs`, `ui.rs`, `handler.rs`, `config.rs` và README.
- [x] README mô tả public API, hành vi và ví dụ Rust.
- [x] `mod.rs` là public seam; các quyết định toggle/disabled được tách vào handler.
- [x] Cấu hình component được tách riêng; UI tiếp tục dùng theme/token dùng chung.
- [ ] Test cho handler/state transitions và trường hợp biên.
- [ ] Xác nhận API exports/callers và hành vi hiện hữu bằng kiểm tra build/test.
- [ ] Chạy các quality gate áp dụng và ghi kết quả thực tế trong `VERIFICATION.md`.

Các dấu [x] ở trên chỉ phản ánh source đã thấy trong worktree; chưa đại diện cho pass build, test hay runtime. Không có runtime evidence được thu thập trong review này.

## Component batch source review: Badge

Review source-level cho `crates/ui/src/components/badge/`, đối chiếu với kiến trúc `Select` và plan component-layer:

- [x] Có `mod.rs`, `ui.rs`, `handler.rs`, `config.rs` và README.
- [x] README mô tả mục đích, public API, hành vi dot/icon priority và ví dụ Rust.
- [x] `components/mod.rs` vẫn re-export `Badge` và `BadgeVariant` qua `badge` module; public `BadgePalette`/`BadgeMetrics` vẫn được re-export từ `badge/mod.rs` để giữ seam module cũ.
- [x] Cấu hình kích thước component được tách vào `config.rs`; radius/stroke vẫn dùng token chung.
- [x] Handler chứa quyết định/pure calculations có test: variant palette, metrics density, leading-space priority, gap và text centering.
- [x] UI layer giữ egui allocation/painting và gọi handler cho palette, metrics, gap và vị trí text.
- [ ] Xác nhận build/test bằng quality gates áp dụng.
- [ ] Runtime evidence không áp dụng cho batch refactor source-only này.

Các dấu [x] ở trên chỉ phản ánh source đã thấy trong worktree; chưa đại diện cho pass build, test hay runtime. Không có runtime evidence được thu thập trong review này.

## Collapsible product review follow-up

- [x] Animation IDs use the allocated response ID, with an explicit stable-ID escape hatch.
- [x] Keyboard activation and accessible button metadata are present; disabled state remains inert.
- [x] Focus ring uses the existing theme accent token.
- [x] No animation-height rewrite or badge truncation scope was introduced.

## 0. Automated Quality Gates

Before declaring any batch or PR ready, ensure the following commands are **actually executed** and pass:

- [ ] `cargo fmt --all -- --check` — Clean formatting without mixing style and logic changes.
- [ ] `cargo check --workspace` (or focused `cargo check -p <crate>`) — Clean compilation without errors.
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` — No lint warnings or unapproved `#[allow]`.
- [ ] `cargo test --workspace` (or targeted crate/module unit tests) — All tests pass with zero failures.
- [ ] `bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci` — Clean code scan passes ratchet budget.
- [ ] `cargo build --release --locked -p db-pro-native` — Native UI release binary compiles successfully when touching UI.

---

## 1. Native UI & Component Layer Architecture

Every public UI component in `crates/ui/src/components/` must adhere to the standardized five-element structure established by `Select`:

```text
components/<name>/
  mod.rs       # Public entry point; module declarations and re-exports of stable public API
  ui.rs        # egui rendering, layout, geometry allocation, and painter calls (may have ui/ submodules)
  handler.rs   # Pure typed behavior, state transitions, validation, and UI-independent calculations
  config.rs    # Component-local constants/defaults with English doc comments
  README.md    # Component purpose, public API, key behavior/constraints, and Rust usage example
```

- [ ] **Public Seam (`mod.rs`)**:
  - `mod.rs` is the single public entry point for the component.
  - All existing caller-facing types, structs, enums, and builder functions remain re-exported without breaking changes.
  - `crates/ui/src/components/mod.rs` continues to re-export the component's public surface.
- [ ] **Presentation Separation (`ui.rs`)**:
  - `ui.rs` strictly contains egui widgets, layout, painter instructions, frame margins, and event dispatching.
  - No business logic, persistence calls, or complex non-UI mathematical calculations in `ui.rs`.
  - UI queries egui geometry/inputs, passes them into pure handler functions, and applies the returned outcome/actions.
- [ ] **Behavior & Calculation Layer (`handler.rs`)**:
  - Handler functions are pure, typed, and independent of `egui::Ui` context wherever feasible.
  - Calculations accept measured values (e.g. bounding rects, center coordinates) rather than querying UI state internally.
  - State transitions and decision logic (e.g. toggles, keyboard navigation, selection syncing) are isolated in handler functions.
  - Comprehensive unit tests exist covering handler branches, state transitions, and edge cases (e.g. disabled items, out-of-bounds selection).
- [ ] **Component Configuration (`config.rs`)**:
  - Contains only genuinely component-owned constants and defaults (e.g. row heights, animation durations, thresholds).
  - Reuses shared canonical tokens from `crate::tokens::*` and semantic colors from `DbProTheme` instead of duplicating local tokens.
  - Does not alias, reassign, or wrap values already defined in shared infrastructure (tokens, theme, spacing, typography); components use the common value directly at the call site.
  - Every `const`, `static`, and associated constant has an English doc comment explaining its semantics, unit, and rationale.
- [ ] **Documentation (`README.md`)**:
  - Contains clear overview of component purpose, public types/enums, key behavioral constraints, and a complete copy-pasteable Rust usage example.

---

## 2. Code Cleanliness & Readability

- [ ] **Naming (`naming.md`)**:
  - Intention-revealing, pronounceable, and free of cryptic abbreviations.
  - Constants follow `SCREAMING_SNAKE_CASE`; structs/enums follow `PascalCase`; functions/methods follow `snake_case`.
  - Boolean variables and accessors use clear prefixes (`is_`, `has_`, `can_`, `should_`).
  - No magic numbers or magic strings in logic; meaningful constants are defined with descriptive names.
- [ ] **Functions & Methods (`functions.md`)**:
  - Small and focused (prefer ≤ 30 lines; flag if > 50 lines without clear justification).
  - Single Responsibility Principle: do one thing at a single level of abstraction.
  - Parameter count ≤ 3; prefer typed options or builder structs over boolean flag parameter soup.
  - Command-Query Separation (CQS): queries do not produce hidden side effects.
  - Shallow nesting (≤ 3 levels); use early returns and guard clauses.
- [ ] **Comments & Documentation (`comments.md`)**:
  - Comments explain **why** (intent, invariant, architectural constraint, workaround), not **what** (do not merely restate code).
  - Detailed English comments for non-obvious UI flow, tracing inputs/events → handlers/state → rendering outcome.
  - No commented-out code, decorative banners, or narration of development history.
  - `TODO(#issue|owner)` format used for tracked follow-ups.
- [ ] **Error Handling & Robustness (`error-handling.md`)**:
  - **P1**: No swallowed errors (`let _ = fallible`, empty catch blocks, silent `.ok()`, or unjustified `unwrap_or_default()`).
  - **P1**: No `unwrap()` or `expect()` on user-facing data paths or network/IPC boundaries.
  - Errors provide actionable context with appropriate error types/enums rather than fragile string matching.

---

## 3. Database Safety & Provider Integrity

- [ ] **SQL Construction & Safety**:
  - **P0/P1**: Never concatenate untrusted SQL identifiers or literals directly into query strings; use parameterized queries or validated identifier quoting.
  - Never bypass destructive operation confirmation policies or safety confirmation prompts.
- [ ] **Transactions & Mutations**:
  - **P1**: Do not claim atomicity without an explicit database transaction.
  - **P1**: Do not treat `affected_rows == 0` as mutation success when row modification was required.
- [ ] **Provider Capability Gating**:
  - Respect explicit provider capabilities (`PostgreSQL`, `SQLite`).
  - Unsupported operations must be gracefully capability-gated with clear error messages rather than emitting invalid dialect SQL.

---

## 4. Review Process & Finding Template

When conducting self-reviews or PR reviews, structure findings using this standard format:

```text
[<Severity: P0|P1|P2>] <Category> — <file_path>:<line_range>
Smell / Issue: <Concise description of the invariant violation or code smell>
Impact: <Why this matters, failure scenario, risk of bug/regression>
Proposed Fix: <Specific refactoring or architectural correction>
```

### Example Finding

```text
[P1] component-layer — crates/ui/src/components/alert.rs:65-171
Smell: Alert mixing egui drawing, layout calculation, and dismissal event branching in a single 100+ line show() method without handler/config separation.
Impact: Inability to unit test variant visual styling rules or dismissal logic without full egui context; violates component layering architecture.
Proposed Fix: Extract Alert into `components/alert/{mod.rs, ui.rs, handler.rs, config.rs, README.md}`; move style mapping and dismissal actions into handler functions with unit tests.
```
