# Credential Lifecycle Cluster — PLAN

Baseline SHA: `beaa1882a6253d902748c27fba53b99d9969b635` (origin/main at claim time).

## Goal

Independently validate the two credential-lifecycle claims in issues
truongnat/db-pro#479 and #483, and land the **smallest verified fix** that
removes a real secret-retention defect without mixing unrelated layers or
evidence levels.

## Issues under triage

| Issue | Claim | Verdict at baseline |
|---|---|---|
| #479 | `apply_connection_snippet` copies the URI password into `draft.password` but never clears `draft.cloud_snippet`, so the raw URI (plaintext password) stays in the draft/UI. | **CONFIRMED (code fact), overstated (risk model)** — see FINDINGS. |
| #483 | `PgPassFile::create` returns `Err` on `write_all` failure before `Ok(Self { path })` is constructed, so the `Drop` guard never runs and a partial credential file is left in the temp dir. | **CONFIRMED** — fixed here. |

## Scope

- Fix #483: construct the `PgPassFile` guard **before** writing the credential
  contents, so any failure after the temp file exists still triggers the `Drop`
  cleanup. Add a deterministic regression test that forces the write to fail and
  asserts no credential file survives.
- Record why #479 must stay a **separate** change (different crate/layer and a
  UI-visible behaviour change with its own verification lane).

## Non-goals

- Do **not** change UI behaviour or clear `cloud_snippet` in this change (#479 is
  deferred to its own branch; see FINDINGS for the rationale).
- Do not touch PostgreSQL/SQLite connection logic, backup publish flow, or any
  unrelated baseline failure.
- Do not add persistence or masking to `UiConnectionDraft`.

## Architecture / affected surface

Single file plus its test seam:

- `crates/infrastructure/src/backup/pg_dump.rs` — `PgPassFile::create` and a new
  private `write_and_guard` helper.
- `crates/infrastructure/src/backup/tests/pg_dump_tests.rs` — regression test.

Layer: `infrastructure` only. No `ui` / `app` / `runtime` change.

## Provider matrix

The implemented change (#483) is PostgreSQL-client-only: `.pgpass` / `PGPASSFILE`
is PostgreSQL `libpq` plumbing with no SQLite equivalent. No provider behaviour is
altered. Automated/live columns are filled from `VERIFICATION.md` once executed.

| Provider | Supported | Automated evidence | Live/runtime evidence | Capability gate |
|---|---|---|---|---|
| PostgreSQL | yes (pgpass temp-file lifecycle only) | NOT VERIFIED (pending focused run) | N/A (no provider round-trip touched) | n/a |
| SQLite | N/A — no `.pgpass` / `PGPASSFILE` concept; not touched | N/A | N/A | n/a |

(#479 is a UI-layer credential-hygiene issue and is not part of the implemented
provider surface here.)

## Acceptance criteria

1. A failed `write_all` after temp-file creation leaves **no** `db-pro-pgpass-*`
   file on disk; the guard removes it.
2. The happy path is unchanged: file is 0600 on Unix, contents match the pgpass
   format, and it is removed on `Drop`.
3. Focused tests pass; `cargo check`/`clippy`/`fmt` gates for the touched crate
   are green.
4. #479 is documented as separately scoped with corrected risk framing.
