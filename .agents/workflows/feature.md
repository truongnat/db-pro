---
description: feature delivery workflow with deterministic planning and dual-stack validation
---
# Workflow: feature
Schema: antigrav.workflow@v1
Domain: agent
MaxCpuMs: 240000
MaxWallTimeMs: 900000
MaxNetworkCalls: 30

## Step: intent_analysis
Skill: agent.analyze_code
Input: Analyze the incoming feature request and return strict JSON with:
- summary: scope boundary, assumptions, acceptance criteria, impacted components
- actions: ordered implementation milestones with file-level intent
- risks: top execution risks and mitigation direction

## Step: architecture_plan
Skill: agent.llm_subagent
DependsOn: intent_analysis
Input: architect:::From this context, produce a deterministic architecture plan with minimal blast radius:
{{intent_analysis}}
Focus on contracts, state transitions, and compatibility constraints.

## Step: implementation_plan
Skill: agent.llm_subagent
DependsOn: architecture_plan
Input: implementer:::Generate an execution-ready implementation plan from:
{{architecture_plan}}
Return strict JSON where actions are concrete and testable.

## Step: validation_strategy
Skill: agent.generate_tests
DependsOn: implementation_plan
Input: Create a validation strategy from:
{{implementation_plan}}
Include expected pass criteria for frontend and Rust backend.

## Step: validate_frontend
Skill: agent.run_script
DependsOn: validation_strategy
Retry: 1
OnFailure: Continue
Input: npm run -s build

## Step: validate_backend
Skill: agent.run_script
DependsOn: validate_frontend
Retry: 1
OnFailure: Continue
Input: cargo check --manifest-path src-tauri/Cargo.toml

## Step: risk_review
Skill: agent.llm_subagent
DependsOn: validate_backend
Input: reviewer:::Build final risk register from planning + validation evidence:
{{implementation_plan}}
{{validation_strategy}}
Include severity, blast radius, mitigation, and rollback trigger.

## Step: internet_security_check
Skill: agent.llm_subagent
DependsOn: risk_review
Input: reviewer:::Run focused security review for internet-capable execution paths.
Use this risk register:
{{risk_review}}
Return pass/fail plus required mitigations.

## Step: next_actions
Skill: agent.next_steps
DependsOn: internet_security_check
Input: Derive next actions from current workflow state using {{implementation_plan}} {{risk_review}} {{internet_security_check}}

## Step: finalize
Skill: demo.echo
DependsOn: next_actions
Input: Feature workflow completed with architecture, implementation, validation, security gates, and next-action plan.
