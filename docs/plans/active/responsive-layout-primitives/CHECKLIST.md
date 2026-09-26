# Responsive Layout Primitives — Checklist

## Planning

- [x] Record baseline evidence, failure scenario, severity, support scope and non-goals.
- [x] Identify existing Gallery breakpoint/container behavior and active-plan overlap.
- [x] Approve initial direction: local-Ui bounded Container + min-width-driven equal-column Grid; defer CSS-style 12-column breakpoints until a real consumer requires them.
- [ ] Review public builder API and prove heterogeneous child placement before broad consumer migration.

## Implementation

- [ ] Add pure responsive measurement helpers and boundary tests.
- [ ] Add Container primitive with bounded fluid/max-width behavior and gutter.
- [ ] Add Row/Col or equivalent responsive Grid API; document wrapping/span semantics.
- [ ] Export primitives from `crates/ui/src/components/mod.rs`.
- [ ] Migrate Component Gallery container and connection form to primitives.
- [ ] Add regression tests for narrow parent/sidebar-open layout and column transitions; compare actual child response bounds with parent clip/available bounds.
- [ ] Review whether any internal min-width in Input/PasswordInput can defeat allocated column width.

## Review and verification

- [ ] Independent architecture/correctness review; resolve all P0/P1 findings.
- [ ] `cargo fmt --all -- --check`
- [ ] `cargo check --workspace`
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] `cargo test --workspace`
- [ ] `cargo build --release --locked -p db-pro-native`
- [ ] Native captures: sidebar open/closed; 1280×800, 1440×900, 1920×1080; test just below/at/above grid column transitions and long validation/helper text; verify no horizontal clipping and normal/error/empty states where applicable.
- [ ] Record exact command results and actual runtime evidence in `VERIFICATION.md`.
- [ ] Update `STATUS.md` only when state actually changes; do not mark COMPLETED before gates pass.
