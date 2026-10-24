# Dispatch log — run `rqm-20261024T104824Z`

| Time (UTC) | Worker | Mode | Scope |
|---|---|---|---|
| 2026-10-24T10:48:24Z | Lead (Composer) | local | Frozen `run-manifest.json`, `expected-instances.json` (digest `ace27c14…`), desktop lease file created |
| 2026-10-24T10:48:24Z | Subagent L0/L1/GUI-1 | **blocked** | Cursor `Task` tool: *Subagents are not implemented by this client* |
| 2026-10-24T10:50Z | Lead (Composer) | local | L1 package `ad731c2b…` (rustc **1.99.0** — later disqualified) |
| 2026-10-24T10:55Z | Lead audit | local | Run **INVALIDATED**; PASS retracted → NOT_TESTED/BLOCKED; schema + §2 audit; GUI halted |
| blocked | All workers | — | New run after `cargo +1.95.0` rebuild per `runs/rqm-next-pin195/preflight.json` |

**Host boundary:** Windows/Linux matrix rows are `BLOCKED_HOST` on this Mac; do not mark them PASS from macOS evidence.

**Scale:** 3,663 candidate instances ≠ completed tests; phase gates in `run-manifest.json`.
