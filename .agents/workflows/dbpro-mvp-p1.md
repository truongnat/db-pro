---
description: dbpro-mvp-p1 polish workflow from DBeaver gap report with deterministic gates
---
# Workflow: dbpro-mvp-p1
Schema: antigrav.workflow@v1
Domain: agent
MaxCpuMs: 240000
MaxWallTimeMs: 900000
MaxNetworkCalls: 35

## Step: intent_analysis
Skill: agent.analyze_code
Input: Analyze /Users/truongdq/Documents/GitHub/db-pro/docs/mvp-dbeaver-gap-report.md and extract P1 items for navigator actions, SQL templates/snippets, and result-grid filter/sort. Return strict JSON with summary/actions/risks.

## Step: execution_plan
Skill: agent.llm_subagent
DependsOn: intent_analysis
Input: implementer:::Build deterministic implementation plan for dbpro-mvp-p1 from {{intent_analysis}}. Use small delivery slices with rollback notes and quality gates.

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
Input: reviewer:::Produce risk register for dbpro-mvp-p1 from {{execution_plan}}. Include severity, blast radius, regression vectors, and mitigation checklist.

## Step: internet_security_check
Skill: agent.llm_subagent
DependsOn: risk_review
Input: reviewer:::Run focused security check for internet-capable execution paths. Confirm no secret leakage through export/copy tooling. Context: {{risk_review}}

## Step: workflow_report
Skill: agent.workflow_report
DependsOn: internet_security_check
Input: Build detailed dbpro-mvp-p1 workflow report from:
{{intent_analysis}}
{{execution_plan}}
{{validate_frontend}}
{{validate_backend}}
{{risk_review}}
{{internet_security_check}}
Return strict JSON with summary/actions/risks and delivery readiness status.

## Step: report_quality_gate
Skill: agent.report_quality_gate
DependsOn: workflow_report
Input: {{workflow_report}}

## Step: next_actions
Skill: agent.next_steps
DependsOn: report_quality_gate
Input: Derive next actions from {{workflow_report}} with priority on unresolved regression and usability risks.

## Step: finalize
Skill: demo.echo
DependsOn: next_actions
Input: dbpro-mvp-p1 workflow completed with planning, dual-stack validation, security gates, detailed report, and next-action plan.
