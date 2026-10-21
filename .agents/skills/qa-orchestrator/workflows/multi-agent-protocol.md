# Real Subagent Delegation Protocol

## Modes
- If actual subagent spawning is exposed by the coding host, dispatch specialist prompts from `../agents/` through that native tool. Parallelize read-only independent scopes.
- Without native spawning, carry out the same subtasks sequentially and explicitly say that no agents were spawned.
- Do not imply npx skills registers native agent definitions: it only installs skill directories.

## Lifecycle
Intake → Risk triage → Select workers → Dispatch scoped tasks → Execute safely → Review evidence → Challenge critical bugs → Deduplicate → Trace coverage → Release decision.

## Routing
Functional and requirements → qa-functional; UI, UX and accessibility → qa-ui-ux; code quality → qa-code-quality; security → qa-security; data/contracts/concurrency → qa-data; performance/resilience → qa-performance; tests and release → qa-regression; unexpected cross-feature behavior → qa-bug-hunter.

## Agent packet
Task ID, worker role, exact objective, paths or flows, requirements and risk IDs, artifact version, permitted tools, forbidden actions, fixture, time budget, evidence format and blockers.

## Merge
Only the QA Lead issues final overall risk assessment. Deduplicate by root cause and reproduction; disputed issues require independent confirmation. Never confuse an unexecuted case with a passed test. Each worker returns PASS/FAIL/BLOCKED/NOT TESTED/N/A records with evidence.
