# #479 — Clear cloud_snippet after URI import (plaintext password leak)

## Goal
After a successful `apply_connection_snippet`, no plaintext password may
remain in `draft.cloud_snippet` — the unmasked `Input` renders it on screen
and the draft may be serialized while the dialog is open.

## Scope
- `crates/app/src/connection/mapper.rs`: `apply_connection_snippet` clears
  `draft.cloud_snippet` after the URI is consumed into fields. Clearing in
  the mapper (not the button handler) makes the guarantee hold for every
  caller; on parse error the field is left untouched so the user can fix it.
- Tests in `crates/app/src/connection/tests/mapper_tests.rs` (new file;
  `#[path]` module declared in `mapper.rs`).

## Non-goals
- No redaction scheme — clearing is the safe minimal path: `cloud_snippet`
  has no readers after import (grep confirms only the paste `Input` and the
  Import button touch it; `cloud_guidance` is the only retained output).
- No password masking on the paste input itself (out of scope; the paste
  field legitimately shows what the user typed until consumed).

## Acceptance
- `apply_connection_snippet` Ok → `draft.cloud_snippet == ""` while
  host/port/database/username/password/driver/ssl_mode are populated.
- Parse error → `cloud_snippet` unchanged.
