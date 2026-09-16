# Plan — UI10 Agent Workspace Polish

## Objective
Standardize and modernize the Agent workspace surfaces to feel like a first-class IDE surface with clear conversation hierarchy, distinct tool activity, safe action visibility, and compliant design tokens.

## Scope
- `crates/ui/src/agent_view.rs`: Standardize header actions, mode dropdown, settings panel, context chips & quick actions, empty state suggestions, message bubble frames, tool-call activity items, execution approval/confirmation cards, failure/retry states, and composer toolbar.
- Standardize all legacy button helpers into canonical `Button` primitives.
- Add `include_str!("../agent_view.rs")` to the non-regression raw-button test in `crates/ui/src/components/mod.rs`.
- Verify full test suite and native release build.
