# Skill: workflow_report
Schema: antigrav.skill@v1

```json
{
  "name": "workflow_report",
  "domain": "agent",
  "description": "Generate a detailed, evidence-backed workflow report from planning, validation, and risk outputs.",
  "risk": "safe",
  "source": "self",
  "tags": ["reporting", "evidence", "workflow", "db-pro"],
  "executor": "ollama",
  "model": "qwen3:8b",
  "temperature": 0.03,
  "input_type": "text",
  "output_type": "json",
  "estimated_cost": 10,
  "estimated_latency_ms": 3200,
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
Use this skill to convert workflow artifacts into an explicit, production-grade report.
The report must be specific, verifiable, and actionable.

## When to Use
- Before workflow finalization.
- When validation evidence exists across multiple steps.
- When a merge/release decision depends on risk and security posture.

## Required Input
Provide:
- Objective and scope context.
- Planning outputs (architecture/implementation/test strategy).
- Validation evidence (commands + outcomes).
- Risk and security review outputs.

## Execution Process
1. Restate scope and objective boundary.
2. Extract concrete evidence from validation outputs.
3. Summarize implementation/test progress by milestone.
4. Build risk register with severity and mitigation status.
5. Produce decision posture (`ready`, `needs_changes`, `blocked`) with rationale.
6. List deterministic next actions ordered by critical path.

## Examples
Input:
```text
Feature plan completed, frontend build passed, backend check failed on query layer, risk review identified high regression risk in export flow.
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
- Depends on quality and completeness of prior step outputs.
- Does not execute fixes; it only synthesizes evidence and guidance.
- Cannot replace domain-expert sign-off for high-risk releases.

## Output Contract
Return strict JSON object with:
- `summary` (string): detailed state narrative with decision posture and confidence.
- `actions` (string[]): ordered critical-path actions including validation commands.
- `risks` (string[]): unresolved risks with severity and mitigation or owner.

## Quality Checklist
- No vague statements without evidence.
- Every risk must tie to a concrete failure mode.
- Actions must be executable in sequence.

## Non-goals
- Do not repeat raw logs verbatim.
- Do not mark `ready` when blocking evidence is missing.

Task input:
{{input}}
