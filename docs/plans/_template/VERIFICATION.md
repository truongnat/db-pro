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

## Completion decision

Do not mark COMPLETED until all applicable lifecycle gates are satisfied.
