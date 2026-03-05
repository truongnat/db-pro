---
description: release workflow with evidence-based go/no-go decision
---
# Workflow: release
Schema: antigrav.workflow@v1
Domain: agent
MaxCpuMs: 220000
MaxWallTimeMs: 900000
MaxNetworkCalls: 25

## Step: release_scope
Skill: agent.analyze_code
Input: Build release scope baseline and return strict JSON with:
- summary: included changes, impacted components, customer impact
- actions: ordered release activities
- risks: known release risks and mitigation direction

## Step: release_checklist
Skill: agent.llm_subagent
DependsOn: release_scope
Input: releaser:::Convert release scope into deterministic checklist:
{{release_scope}}
Include pre-release, release, and post-release actions.

## Step: validate_frontend
Skill: agent.run_script
DependsOn: release_checklist
Retry: 1
OnFailure: FailFast
Input: npm run -s build

## Step: validate_backend
Skill: agent.run_script
DependsOn: validate_frontend
Retry: 1
OnFailure: FailFast
Input: cargo check --manifest-path src-tauri/Cargo.toml

## Step: release_risk
Skill: agent.llm_subagent
DependsOn: validate_backend
Input: releaser:::Produce release risk register and mitigation status using:
{{release_checklist}}
Include go/no-go conditions.

## Step: internet_security_check
Skill: agent.llm_subagent
DependsOn: release_risk
Input: reviewer:::Run focused security check for internet-capable execution paths.
Context:
{{release_risk}}
Return pass/fail and mandatory mitigations.

## Step: go_no_go_decision
Skill: agent.llm_subagent
DependsOn: internet_security_check
Input: releaser:::Issue final go/no-go decision from:
{{release_risk}}
{{internet_security_check}}
Return strict JSON with explicit conditions and rollback note.

## Step: workflow_report
Skill: agent.workflow_report
DependsOn: go_no_go_decision
Input: Build detailed release workflow report from:
{{release_scope}}
{{release_checklist}}
{{validate_frontend}}
{{validate_backend}}
{{release_risk}}
{{internet_security_check}}
{{go_no_go_decision}}
Return strict JSON with summary/actions/risks and explicit go/no-go rationale.

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
Input: Derive next actions from {{workflow_report}} and keep pre-release/release/post-release phases explicit.

## Step: finalize
Skill: demo.echo
DependsOn: next_actions
Input: Release workflow completed with checklist, dual-stack validation, security/decision gates, detailed report, and next-action plan.
