# Skill: next_steps
Schema: antigrav.skill@v1

```json
{
  "name": "next_steps",
  "domain": "agent",
  "description": "Derive the next deterministic execution tasks from workflow progress and blockers.",
  "risk": "none",
  "source": "self",
  "tags": ["planning", "triage", "workflow", "db-pro"],
  "executor": "ollama",
  "model": "qwen3:8b",
  "temperature": 0.05,
  "input_type": "text",
  "output_type": "json",
  "estimated_cost": 7,
  "estimated_latency_ms": 2200,
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
Use this skill to keep workflow momentum by producing execution-ready next tasks.
The output should be an ordered queue, not a generic todo list.

## When to Use
- Planning after partial progress.
- Replanning after blockers or failed validations.
- Building short-term roadmap for a thread.

## Required Input
Provide:
- Current status summary.
- Completed vs pending items.
- Active blockers and dependencies.

## Execution Process
1. Determine current objective and progress state.
2. Rank pending work by critical path and risk.
3. Break down into smallest executable tasks.
4. Mark blockers and unblock actions.
5. Add validation checkpoints for each stage.

## Examples
Input:
```text
Frontend build passes, tauri check fails in query command module; pending CSV export review.
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
- Depends on quality of status context.
- Does not execute tasks directly.
- Cannot replace deep root-cause analysis for unknown failures.

## Output Contract
Return strict JSON object with:
- `summary` (string): current state and planning rationale.
- `actions` (string[]): ordered next actions.
- `risks` (string[]): blockers/uncertainties that may derail progress.

## Quality Checklist
- Actions must be specific and bounded.
- Include validation actions where code changes are involved.
- Keep dependency ordering explicit.

## Non-goals
- Do not repeat completed work.
- Do not output vague steps like "continue implementation".

Task input:
{{input}}
