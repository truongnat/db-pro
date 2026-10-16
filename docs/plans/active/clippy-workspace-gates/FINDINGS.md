# Findings — Restore Workspace Clippy Gate

CI failed because the current Rust Clippy version reports three collapsible-match warnings in `saved_task.rs`, two needless borrows in dialog titles, and one large enum variant in the monitoring surface. Since CI promotes warnings to errors, every PR was blocked before its own code could pass the required Rust check.

The fixes are mechanical: express credential checks as guarded match arms, pass owned formatted titles directly, and box the large monitoring action payload at the internal enum boundary.

The first required CI run then exposed an additional test-compilation blocker: an unused closure argument under `RUSTFLAGS=-Dwarnings`. Renaming it `_ctx` preserves the test and lets the workspace test target compile.
