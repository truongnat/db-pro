# <Feature> — Verification

State: RUNTIME_VERIFY

## Commands actually executed

Record the exact command and observed result. Do not copy expected output as evidence.

```text
<command>
<result>
```

## Provider matrix

| Provider | Automated | Live/runtime | Notes |
|---|---|---|---|
| PostgreSQL | NOT VERIFIED | NOT VERIFIED | |
| SQLite | NOT VERIFIED | NOT VERIFIED | |

## UI lifecycle

```text
UI
→ command
→ backend
→ database
→ introspection
→ refreshed UI
```

Status: NOT VERIFIED

## Remaining evidence gaps

- ...

## Artifact retention

Evidence paths linked from this file must match `docs/ARTIFACT_POLICY.md`. Qualification runs: copy `docs/plans/_template/runs.gitignore`; invalidated runs need `ARTIFACT_RETENTION.md` + `RUN_INVALIDATION.md` § Retained evidence.

## Completion decision

Do not mark COMPLETED until all applicable lifecycle gates are satisfied.
