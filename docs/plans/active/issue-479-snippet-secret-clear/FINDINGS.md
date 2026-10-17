# #479 Findings

- `apply_connection_snippet` (`crates/app/src/connection/mapper.rs`) copied the
  URI's password into `draft.password` but left the full URI — including the
  plaintext password — in `draft.cloud_snippet`.
- `advanced_panels.rs:108-128` binds `cloud_snippet` to an unmasked `Input`
  (`.password(false)` semantics — plain text field), so the secret stays
  visible in the Cloud Presets panel after import.
- Grep audit of `cloud_snippet` across the workspace:
  - written to `String::new()` at draft construction sites
    (`mapper.rs:98`, `workspace_actions.rs:378`, runtime `Default`)
  - read only by the paste `Input` and cloned for the Import button call in
    `advanced_panels.rs`
  - never serialized to disk by `draft_to_domain_config` (no such field on
    `ConnectionConfig`), but the live draft keeps the secret for the dialog's
    lifetime — clearing is still required by the issue.
- Conclusion: nothing depends on the snippet after a successful import;
  `draft.cloud_snippet.clear()` inside `apply_connection_snippet` is the
  minimal safe fix. Done in the mapper so all callers get the guarantee; a
  failed parse keeps the user's input for correction.
