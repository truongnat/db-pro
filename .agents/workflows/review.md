---
description: review workflow with severity-based findings and merge recommendation
---
# Workflow: review
Schema: antigrav.workflow@v1
Domain: agent
MaxCpuMs: 180000
MaxWallTimeMs: 600000
MaxNetworkCalls: 20

## Step: review_context
Skill: agent.analyze_code
Input: Establish review context and return strict JSON with:
- summary: requirement intent, touched areas, risk hotspots
- actions: review checklist order
- risks: likely regression or correctness concerns

## Step: generate_findings
Skill: agent.llm_subagent
DependsOn: review_context
Input: reviewer:::Generate detailed review findings from:
{{review_context}}
Use severity model (critical/high/medium/low) and include remediation guidance.

## Step: merge_recommendation
Skill: agent.llm_subagent
DependsOn: generate_findings
Input: reviewer:::Produce merge recommendation with blockers/non-blockers from:
{{generate_findings}}
Return strict JSON with actionable follow-ups.

## Step: internet_security_check
Skill: agent.llm_subagent
DependsOn: merge_recommendation
Input: reviewer:::Run focused security check for internet-capable execution paths.
Context:
{{merge_recommendation}}
Return pass/fail and required mitigations.

## Step: next_actions
Skill: agent.next_steps
DependsOn: internet_security_check
Input: Derive next actions from {{generate_findings}} {{merge_recommendation}} {{internet_security_check}}

## Step: finalize
Skill: demo.echo
DependsOn: next_actions
Input: Review workflow completed with findings, recommendation, security gate, and next-action plan.
