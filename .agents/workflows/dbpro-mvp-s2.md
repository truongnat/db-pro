---
description: dbpro-mvp-s2 architecture refactor workflow for controller extraction and regression safety
---
# Workflow: dbpro-mvp-s2
Schema: antigrav.workflow@v1
Domain: agent
MaxCpuMs: 300000
MaxWallTimeMs: 1200000
MaxNetworkCalls: 35

## Step: intent_analysis
Skill: agent.analyze_code
Input: Analyze /Users/truongdq/Documents/GitHub/db-pro/docs/mvp-sprint-backlog.md and extract Sprint 2 tickets (S2-01..S2-06) with dependency graph, DoD, and validation strategy. Return strict JSON with summary/actions/risks.

## Step: execution_plan
Skill: agent.llm_subagent
DependsOn: intent_analysis
Input: implementer:::Build deterministic implementation plan for dbpro-mvp-s2 from {{intent_analysis}}. Keep refactor slices small and include rollback notes for each controller extraction.

## Step: test_plan
Skill: agent.generate_tests
DependsOn: execution_plan
Input: Generate integration test plan and concrete test cases for S2-05 critical flows using current architecture and target controller boundaries.

## Step: frontend_build_check
Skill: agent.run_script
DependsOn: test_plan
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
Input: reviewer:::Produce risk register for dbpro-mvp-s2 from {{execution_plan}} and {{test_plan}}. Include regression vectors and mitigation mapped to S2 ticket IDs.

## Step: internet_security_check
Skill: agent.llm_subagent
DependsOn: risk_review
Input: reviewer:::Run focused security check for refactor and integration-test execution paths. Confirm no secret leakage and no unsafe shell patterns. Context: {{risk_review}}

## Step: workflow_report
Skill: agent.workflow_report
DependsOn: internet_security_check
Input: Build detailed dbpro-mvp-s2 workflow report from:
{{intent_analysis}}
{{execution_plan}}
{{test_plan}}
{{frontend_build_check}}
{{backend_check}}
{{workflow_health}}
{{risk_review}}
{{internet_security_check}}
Return strict JSON with summary/actions/risks and refactor-readiness posture.

## Step: report_quality_gate
Skill: agent.report_quality_gate
DependsOn: workflow_report
Input: {{workflow_report}}

## Step: next_actions
Skill: agent.next_steps
DependsOn: report_quality_gate
Input: Derive next actions from {{workflow_report}} with emphasis on regression containment and controller-boundary stability.

## Step: finalize
Skill: demo.echo
DependsOn: next_actions
Input: dbpro-mvp-s2 workflow completed with architecture slicing, test planning, validation gates, detailed report, and next-action plan.
