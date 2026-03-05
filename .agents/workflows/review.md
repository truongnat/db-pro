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

## Step: workflow_report
Skill: agent.workflow_report
DependsOn: internet_security_check
Input: Build detailed review report from:
{{review_context}}
{{generate_findings}}
{{merge_recommendation}}
{{internet_security_check}}
Return strict JSON with summary/actions/risks and clear merge posture.

## Step: report_quality_gate
Skill: agent.report_quality_gate
DependsOn: workflow_report
Input: {{workflow_report}}

## Step: simulation_fallback_gate
Skill: agent.simulation_fallback_gate
DependsOn: report_quality_gate
Input: {{workflow_report}}

## Step: next_actions
Skill: agent.next_steps
DependsOn: simulation_fallback_gate
Input: Derive next actions from {{workflow_report}} and preserve severity-first remediation ordering.

## Step: finalize
Skill: demo.echo
DependsOn: next_actions
Input: Review workflow completed with findings, recommendation, security gate, detailed report, and next-action plan.
