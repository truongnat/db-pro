# Skill: generate_tests
Schema: antigrav.skill@v1

```json
{
  "name": "generate_tests",
  "domain": "agent",
  "description": "Generate high-value deterministic test plans and cases for changed behavior.",
  "risk": "safe",
  "source": "self",
  "tags": ["testing", "qa", "regression", "db-pro"],
  "executor": "ollama",
  "model": "qwen3:8b",
  "temperature": 0.05,
  "input_type": "text",
  "output_type": "text",
  "estimated_cost": 9,
  "estimated_latency_ms": 3000,
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
Use this skill to convert requirements or diffs into concrete, prioritized test suites.
Focus on regression protection and critical path confidence.

## When to Use
- New feature verification planning.
- Regression suite updates after bug fixes.
- Coverage gap analysis before merge/release.

## Required Input
Provide:
- Changed behavior and acceptance criteria.
- Relevant modules/files/endpoints.
- Existing test approach (if known).

## Execution Process
1. Identify behavior contracts that must hold.
2. Enumerate happy-path and failure-path scenarios.
3. Prioritize tests by production risk.
4. Define expected outcomes and failure signals.
5. Recommend deterministic test commands.

## Examples
Input:
```text
Connection sidebar now supports quick filtering by environment and status.
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
- Produces test strategy/cases, not executable test code.
- Cannot validate environment-specific integrations directly.
- Requires accurate change context for best prioritization.

## Output Contract
Return strict JSON object with:
- `summary` (string): strategy and coverage intent.
- `actions` (string[]): ordered test case checklist.
- `risks` (string[]): uncovered or hard-to-test areas.

## Quality Checklist
- Include edge/negative paths, not only happy path.
- Include data integrity and error handling checks when relevant.
- Keep actions specific and runnable.

## Non-goals
- Do not suggest flaky/non-deterministic tests.
- Do not rely on hidden assumptions.

Task input:
{{input}}
