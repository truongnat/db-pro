# Skill: analyze_code
Schema: antigrav.skill@v1

```json
{
  "name": "analyze_code",
  "domain": "agent",
  "description": "Perform deep implementation analysis and return deterministic execution guidance.",
  "risk": "safe",
  "source": "self",
  "tags": ["analysis", "architecture", "planning", "db-pro"],
  "executor": "ollama",
  "model": "qwen3:8b",
  "temperature": 0.05,
  "input_type": "text",
  "output_type": "text",
  "estimated_cost": 8,
  "estimated_latency_ms": 2800,
  "allow_fs_read": false,
  "allow_fs_write": false,
  "allow_network": true,
  "allow_env": false,
  "allow_process_spawn": false,
  "side_effect_class": "Idempotent",
  "trust_tier": "Constrained"
}
```

## Overview
Use this skill when architecture-quality analysis is required before code changes.
The output must be concrete enough for implementers to execute without guesswork.

## When to Use
- Planning feature implementation slices.
- Assessing bug-fix blast radius.
- Evaluating technical risks before merge/release.

## Required Input
Provide:
- Objective and acceptance criteria.
- Relevant file/module context.
- Constraints (compatibility, performance, timeline).

## Execution Process
1. Restate objective and constraints.
2. Infer impacted components and dependencies.
3. Build deterministic plan with ordered actions.
4. Identify risks and mitigation options.
5. Define validation checkpoints.

## Examples
Input:
```text
Implement SQL result pagination without breaking current query execution flow.
```

Expected output shape:
```json
{
  "summary": "...",
  "actions": ["...", "..."],
  "risks": ["...", "..."]
}
```

## Limitations
- Produces analysis only; does not apply code edits.
- Quality depends on clarity of provided context.
- May require follow-up for ambiguous requirements.

## Output Contract
Return strict JSON object with:
- `summary` (string): concise architecture and execution narrative.
- `actions` (string[]): ordered executable implementation actions.
- `risks` (string[]): explicit failure modes with mitigation direction.

## Quality Checklist
- No generic advice.
- Every action must be executable and scoped.
- Risks must map to concrete failure modes.

## Non-goals
- Do not output code patches.
- Do not recommend broad redesign unless strictly required.

Task input:
{{input}}
