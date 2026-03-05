---
description: dbpro-mvp-p0 rollout from DBeaver gap report with deterministic quality gates
---
# Workflow: dbpro-mvp-p0
Schema: antigrav.workflow@v1
Domain: agent
MaxCpuMs: 240000
MaxWallTimeMs: 900000
MaxNetworkCalls: 35

## Step: intent_analysis
Skill: agent.analyze_code
Input: Analyze /Users/truongdq/Documents/GitHub/db-pro/docs/mvp-dbeaver-gap-report.md and extract only P0 scope with feature IDs, acceptance criteria, dependencies, and rollout order. Return strict JSON with summary/actions/risks.

## Step: execution_plan
Skill: agent.llm_subagent
DependsOn: intent_analysis
Input: implementer:::Build deterministic implementation plan for dbpro-mvp-p0 from {{intent_analysis}}. Plan must be incremental, code-oriented, and include rollback notes and validation checklist.

## Step: validate_frontend
Skill: agent.run_script
DependsOn: execution_plan
Retry: 1
OnFailure: FailFast
Input: npm run -s build

## Step: validate_backend
Skill: agent.run_script
DependsOn: validate_frontend
Retry: 1
OnFailure: FailFast
Input: cargo check --manifest-path src-tauri/Cargo.toml

## Step: risk_review
Skill: agent.llm_subagent
DependsOn: validate_backend
Input: reviewer:::Produce risk register for dbpro-mvp-p0 from {{execution_plan}}. Include severity, blast radius, mitigation, and follow-up tasks mapped to feature IDs.

## Step: internet_security_check
Skill: agent.llm_subagent
DependsOn: risk_review
Input: reviewer:::Run focused security check for internet-capable execution paths. Confirm no plaintext secret persistence and no unsafe shell patterns. Context: {{risk_review}}

## Step: workflow_report
Skill: agent.workflow_report
DependsOn: internet_security_check
Input: Build detailed dbpro-mvp-p0 workflow report from:
{{intent_analysis}}
{{execution_plan}}
{{validate_frontend}}
{{validate_backend}}
{{risk_review}}
{{internet_security_check}}
Return strict JSON with summary/actions/risks and objective readiness status.

## Step: report_quality_gate
Skill: agent.report_quality_gate
DependsOn: workflow_report
Input: {{workflow_report}}

## Step: next_actions
Skill: agent.next_steps
DependsOn: report_quality_gate
Input: Derive next actions from {{workflow_report}} with strict ordering by feature dependency and risk severity.

## Step: finalize
Skill: demo.echo
DependsOn: next_actions
Input: dbpro-mvp-p0 workflow completed with planning, dual-stack validation, security gates, detailed report, and next-action plan.
