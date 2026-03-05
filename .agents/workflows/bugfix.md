---
description: bugfix workflow with root-cause discipline and regression containment
---
# Workflow: bugfix
Schema: antigrav.workflow@v1
Domain: agent
MaxCpuMs: 220000
MaxWallTimeMs: 900000
MaxNetworkCalls: 30

## Step: issue_triage
Skill: agent.analyze_code
Input: Analyze the reported bug and return strict JSON with:
- summary: symptom, impact, reproducibility, affected scope
- actions: investigation steps ordered by confidence
- risks: uncertainty and potential regression vectors

## Step: root_cause_hypothesis
Skill: agent.llm_subagent
DependsOn: issue_triage
Input: resolver:::From this triage output, produce root-cause hypothesis with confidence and disproof checks:
{{issue_triage}}

## Step: patch_plan
Skill: agent.llm_subagent
DependsOn: root_cause_hypothesis
Input: implementer:::Create minimal patch plan from:
{{root_cause_hypothesis}}
Actions must map to concrete files and include rollback strategy.

## Step: regression_plan
Skill: agent.generate_tests
DependsOn: patch_plan
Input: Generate regression test plan from:
{{patch_plan}}
Prioritize high-risk and edge-case scenarios.

## Step: validate_frontend
Skill: agent.run_script
DependsOn: regression_plan
Retry: 2
OnFailure: Continue
Input: npm run -s build

## Step: validate_backend
Skill: agent.run_script
DependsOn: validate_frontend
Retry: 2
OnFailure: Continue
Input: cargo check --manifest-path src-tauri/Cargo.toml

## Step: post_fix_review
Skill: agent.llm_subagent
DependsOn: validate_backend
Input: reviewer:::Evaluate patch quality and unresolved risk using:
{{patch_plan}}
{{regression_plan}}
Return strict JSON with remediation actions if gaps remain.

## Step: internet_security_check
Skill: agent.llm_subagent
DependsOn: post_fix_review
Input: reviewer:::Run focused security check for internet-capable paths.
Context:
{{post_fix_review}}
Return pass/fail and mandatory mitigations.

## Step: next_actions
Skill: agent.next_steps
DependsOn: internet_security_check
Input: Derive next actions from {{patch_plan}} {{post_fix_review}} {{internet_security_check}}

## Step: finalize
Skill: demo.echo
DependsOn: next_actions
Input: Bugfix workflow completed with root-cause, patch, regression, security validation, and next-action plan.
