---
description: dbpro-mvp-s3 UX parity workflow for completion, grid interaction, and connection panel polish
---
# Workflow: dbpro-mvp-s3
Schema: antigrav.workflow@v1
Domain: agent
MaxCpuMs: 300000
MaxWallTimeMs: 1200000
MaxNetworkCalls: 35

## Step: intent_analysis
Skill: agent.analyze_code
Input: Analyze /Users/truongdq/Documents/GitHub/db-pro/docs/mvp-sprint-backlog.md and extract Sprint 3 tickets (S3-01..S3-06) with keyboard-first UX acceptance criteria and interaction quality bars. Return strict JSON with summary/actions/risks.

## Step: execution_plan
Skill: agent.llm_subagent
DependsOn: intent_analysis
Input: implementer:::Build deterministic implementation plan for dbpro-mvp-s3 from {{intent_analysis}}. Include UX validation checklist and rollback notes for interaction changes.

## Step: frontend_build_check
Skill: agent.run_script
DependsOn: execution_plan
Retry: 1
OnFailure: FailFast
Input: npm run -s build

## Step: backend_check
Skill: agent.run_script
DependsOn: frontend_build_check
Retry: 1
OnFailure: FailFast
Input: cargo check --manifest-path src-tauri/Cargo.toml

## Step: workflow_health
Skill: agent.run_script
DependsOn: backend_check
Retry: 1
OnFailure: FailFast
Input: cargo run --manifest-path /Users/truongdq/Documents/GitHub/agentic-sdlc/Cargo.toml -- workflow check

## Step: risk_review
Skill: agent.llm_subagent
DependsOn: workflow_health
Input: reviewer:::Produce UX/regression risk register for dbpro-mvp-s3 from {{execution_plan}} with severity, blast radius, and mitigation mapped to S3 ticket IDs.

## Step: internet_security_check
Skill: agent.llm_subagent
DependsOn: risk_review
Input: reviewer:::Run focused security check for frontend interaction changes and copy/export paths. Context: {{risk_review}}

## Step: workflow_report
Skill: agent.workflow_report
DependsOn: internet_security_check
Input: Build detailed dbpro-mvp-s3 workflow report from:
{{intent_analysis}}
{{execution_plan}}
{{frontend_build_check}}
{{backend_check}}
{{workflow_health}}
{{risk_review}}
{{internet_security_check}}
Return strict JSON with summary/actions/risks and UX-release readiness posture.

## Step: report_quality_gate
Skill: agent.report_quality_gate
DependsOn: workflow_report
Input: {{workflow_report}}

## Step: next_actions
Skill: agent.next_steps
DependsOn: report_quality_gate
Input: Derive next actions from {{workflow_report}} and prioritize keyboard/interaction regressions by user impact.

## Step: finalize
Skill: demo.echo
DependsOn: next_actions
Input: dbpro-mvp-s3 workflow completed with UX-focused planning, validation gates, risk/security review, detailed report, and next-action plan.
