---
description: Runtime execution policy for db-pro agent workflows
trigger: always_on
---
# Runtime Rules
Schema: antigrav.rule@v1
```json
{
  "allowed_domains": ["agent", "demo", "utils"],
  "preferred_domains": ["agent"],
  "cross_domain_penalty": 40,
  "disable_network": false,
  "read_only": false,
  "strict_mode": true,
  "external_mutation_penalty": 120,
  "step_timeout_ms": 180000,
  "max_trust_tier": "Constrained",
  "max_total_cost": 2500,
  "max_total_latency_ms": 900000,
  "max_steps": 30,
  "max_cpu_ms": 240000,
  "max_wall_time_ms": 1200000,
  "max_fs_reads": 3000,
  "max_fs_writes": 450,
  "max_network_calls": 25,
  "max_memory_mb": 1024,
  "run_script_timeout_ms": 180000,
  "run_script_allowed_commands": [
    "npm",
    "npx",
    "cargo",
    "rustc",
    "rustfmt",
    "clippy-driver",
    "git",
    "pnpm",
    "yarn",
    "node",
    "bun"
  ],
  "run_script_denied_commands": [
    "sudo",
    "rm",
    "dd",
    "mkfs",
    "shutdown",
    "reboot",
    "poweroff",
    "launchctl"
  ],
  "run_script_allow_shell_operators": false
}
```

## Policy Intent
- Keep db-pro workflow execution deterministic, bounded, and auditable.
- Allow code mutation workflows but cap blast radius with strict runtime and trust-tier controls.
- Enforce explicit command allowlist/denylist for every shell execution path.

## Operational Notes
- If a task requires commands outside `run_script_allowed_commands`, update this file first.
- Network remains enabled to support LLM routing, but total calls are capped.
- Time and step ceilings are tuned for mixed frontend (`npm`) + backend (`cargo`) workflows.
