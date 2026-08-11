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
| PostgreSQL | PENDING | PENDING | |
| SQLite | PENDING | PENDING | |

## UI lifecycle

```text
UI
→ command
→ backend
→ database
→ introspection
→ refreshed UI
```

Status: PENDING

## Remaining evidence gaps

- ...

## Completion decision

Do not mark COMPLETED until all applicable lifecycle gates are satisfied.
