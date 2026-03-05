---
description: dbpro-mvp-s1 stability workflow for PostgreSQL reliability and deterministic query lifecycle
---
# Workflow: dbpro-mvp-s1
Schema: antigrav.workflow@v1
Domain: agent
MaxCpuMs: 300000
MaxWallTimeMs: 1200000
MaxNetworkCalls: 35

## Step: intent_analysis
Skill: agent.analyze_code
Input: Analyze /Users/truongdq/Documents/GitHub/db-pro/docs/mvp-sprint-backlog.md and extract Sprint 1 tickets (S1-01..S1-05) with dependencies, DoD, and validation commands. Return strict JSON with summary/actions/risks.

## Step: execution_plan
Skill: agent.llm_subagent
DependsOn: intent_analysis
Input: implementer:::Build deterministic implementation plan for dbpro-mvp-s1 from {{intent_analysis}}. Keep steps small, include rollback notes, and map each action to S1 ticket IDs.

## Step: frontend_build_check
Skill: agent.run_script
DependsOn: execution_plan
Retry: 1
OnFailure: Continue
Input: npm run -s build

## Step: backend_check
Skill: agent.run_script
DependsOn: frontend_build_check
Retry: 1
OnFailure: Continue
Input: cargo check --manifest-path src-tauri/Cargo.toml

## Step: workflow_health
Skill: agent.run_script
DependsOn: backend_check
Retry: 1
OnFailure: Continue
Input: cargo run --manifest-path /Users/truongdq/Documents/GitHub/agentic-sdlc/Cargo.toml -- workflow check

## Step: risk_review
Skill: agent.llm_subagent
DependsOn: workflow_health
Input: reviewer:::Produce risk register for dbpro-mvp-s1 from {{execution_plan}} and validation outputs. Include severity, blast radius, and mitigation mapped to S1 ticket IDs.

## Step: internet_security_check
Skill: agent.llm_subagent
DependsOn: risk_review
Input: reviewer:::Run focused security check for internet-capable execution paths and secret handling. Context: {{risk_review}}

## Step: next_actions
Skill: agent.next_steps
DependsOn: internet_security_check
Input: Derive next actions from {{execution_plan}} {{risk_review}} {{internet_security_check}}

## Step: finalize
Skill: demo.echo
DependsOn: next_actions
Input: dbpro-mvp-s1 workflow completed with planning, validation gates, and risk/security review.
