# Findings

## P1 — Agent key bypasses the shared secret lifecycle

At baseline `941c397065a59f8e976bd212e8ee00287ca6f5ac`,
`crates/native-app/src/main.rs` constructs a raw keyring entry for
`agent/groq_api_key`, saves it best-effort, and seeds it into the process
environment. The runtime owns a configured `KeyringVault` already used by
connection and backup services, but the Agent key does not use it. “Forget key”
therefore cannot remove encrypted-file/session fallback data and its failure is
not returned to the UI.

## P2 — Configure failure can leave the UI request pending

The generic `RuntimeEvent::Failed` path does not clear
`agent_configure_request`, so a failed Agent key operation can leave the settings
panel in its busy state. The fix will route the failure through the existing
request failure handler.

## Runtime evidence

Native UI runtime verification is intentionally skipped for this slice because
the user requested moving to another coding issue. Automated evidence remains
required.
