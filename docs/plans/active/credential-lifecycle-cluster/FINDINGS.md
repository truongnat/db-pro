# Credential Lifecycle Cluster — FINDINGS

All file:line anchors are against baseline SHA
`beaa1882a6253d902748c27fba53b99d9969b635`.

## Cluster coherence verdict

**Not coherent to group as one change.** The two issues share only the word
"credential"; they live in different crate layers, have different failure
mechanisms, and — critically — different evidence/verification lanes. Fixing
#483 (infrastructure, unit-provable) together with #479 (UI-visible behaviour,
requires the UI self-verification gate) would mix evidence levels and expand
scope. Per the analyst-first / smallest-coherent-fix policy, only **#483** is
implemented here; #479 is deferred to its own change.

---

## #483 — pgpass partial-write file is left on disk — CONFIRMED, P2

### Evidence

`crates/infrastructure/src/backup/pg_dump.rs:201-207`:

```rust
let mut file = options
    .open(&path)
    .map_err(|error| DbError::Internal(format!("failed to create pgpass file {}: {error}", path.display())))?;
use std::io::Write as _;
file.write_all(contents.as_bytes())
    .map_err(|error| DbError::Internal(format!("failed to write pgpass file: {error}")))?;
Ok(Self { path })   // guard only exists after a *successful* write
```

The `?` on `write_all` returns before `Ok(Self { path })` constructs the guard,
so `Drop` (lines 215-221) — the only cleanup path — never runs. The file created
at line 191 (`temp_dir()/db-pro-pgpass-<uuid>`) survives with truncated
credential material.

### Failure scenario

`write_all` fails partway through (ENOSPC / EDQUOT / EIO). The function returns
`DbError::Internal`; the half-written `host:port:db:user:pass…` file remains in
the temp dir. The struct's own doc contract (lines 160-169) promises removal "as
soon as the spawned tool exits or the run errors" — that contract is broken for
the pre-`Ok` path.

### Severity justification

P2. Exposure is bounded: on Unix the file is created 0600 (line 195-199) and has
an unpredictable UUID name, and Windows temp is per-user. This is not a
catastrophic or remotely-triggerable leak; it is an edge-case I/O failure that
violates a documented cleanup invariant and can strand credential bytes.

### Provider impact

Provider-agnostic: `PgPassFile` is PostgreSQL-only client-tool plumbing. SQLite
has no equivalent. No provider behaviour is changed.

### Existing coverage

`pg_dump_tests.rs::pgpass_file_is_owner_only_and_removed_on_drop` covers the
**happy-path** Drop cleanup only; it never exercises a failed write. No open PR
addresses this.

### Smallest coherent fix

Create the guard immediately after `open` succeeds (before `write_all`), so any
later error drops it and removes the path. New private helper `write_and_guard`.
Regression test forces the write to fail and asserts the file is gone.

---

## #479 — URI password retained in `cloud_snippet` — code fact CONFIRMED, risk model OVERSTATED, P2 (hygiene)

### Confirmed code facts

- `crates/app/src/connection/mapper.rs:203-217`: `apply_connection_snippet`
  assigns `draft.password = pwd` (209-211) and never clears or rewrites
  `draft.cloud_snippet`.
- `crates/app/src/connection/advanced_panels.rs:108-131`: the raw `cloud_snippet`
  is edited through a **plain** `Input::new` (108-115; not a masked field); on
  Import it is cloned (125) and applied (126). On success (127) only the error
  state is cleared — the URI, including any plaintext password, stays in the
  field.

### Claims in the issue that are NOT supported at this SHA

The issue states "Draft persist/session làm việc giữ cả hai" (draft
persistence/session keeps both). This is **false**:

- `UiConnectionDraft` (`crates/ui/src/runtime/runtime_connection_types.rs:27-57`)
  derives only `Debug, Clone, PartialEq, Eq` — **no `Serialize`/`Deserialize`**.
  It is not persisted to disk.
- `crates/native-app/src/translate_cmd.rs:14-130` (`draft_to_domain`) reads
  `host/port/database/username/password/auth_kind/cloud_preset/…` but **never
  reads `cloud_snippet`**; the field is dropped and never reaches the backend.
- A repo-wide grep shows `cloud_snippet` used only in the panel, the draft
  initializers, and the type definition — no `tracing`, export, or logging path.

### Corrected risk

Residual risk is **on-screen only**: after a successful import the user's own
pasted URI (with password) remains visible in a plaintext field, which is a
screenshot / shoulder-surf hygiene issue, not a persistence or log leak. The
password is in `draft.password` in memory regardless, so masking is visual.

### Why this is a separate change

1. Different layer: `crates/app` UI-mapper behaviour vs `crates/infrastructure`
   file lifecycle; no shared module or test harness.
2. Different verification lane: a user-visible form-behaviour change falls under
   the AGENTS UI self-verification rule (runtime light/dark captures), which the
   lifecycle forbids mixing with an infrastructure unit-test fix.
3. Corrected evidence: its severity case must be re-scoped against the
   (overstated) persistence claim documented above.

### Recommended follow-up (for the separate change)

On `Ok(())` in `advanced_panels.rs:126-129`, clear `draft.cloud_snippet` (and/or
redact the URI) so the consumed secret does not linger in a plaintext field;
cover with an app-level test plus the UI runtime evidence the gate requires.

---

## Residual P0/P1/P2 for this change

- P0: 0
- P1: 0
- P2: #483 fixed here; #479 carried over as a separate, re-scoped P2.
