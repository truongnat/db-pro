# Plan — Restore Workspace Clippy Gate

## Objective

Fix pre-existing warnings that made the required `cargo clippy --all -- -D warnings` CI job fail on unrelated PRs.

## Scope

- Collapse match arms in saved-task SQL credential detection without changing classification behavior.
- Remove needless string-reference borrows in foreign-server/publication dialogs.
- Box the oversized monitoring-session action in its internal surface enum.
- Keep this maintenance change isolated from product-ticket PRs.
