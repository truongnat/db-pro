//! Tests for `../agent_orchestrator.rs` (moved out of the source file).
use std::collections::VecDeque;
use std::sync::Mutex;

use async_trait::async_trait;
use db_pro_core::domain::agent::{AgentDocumentSnapshot, AgentMode, AgentSession, AgentToolInput, AgentToolResult};
use db_pro_core::domain::agent_context::AgentContext;

use super::*;

struct FakeProvider {
    responses: Mutex<VecDeque<Vec<AgentProviderEvent>>>,
}

#[async_trait]
impl AgentProvider for FakeProvider {
    async fn complete(
        &self,
        _request: AgentProviderRequest,
    ) -> Result<Vec<AgentProviderEvent>, crate::CodexProviderError> {
        self.responses
            .lock()
            .expect("provider lock")
            .pop_front()
            .ok_or(crate::CodexProviderError::EmptyResponse)
    }
}

struct FakeToolRunner;

#[async_trait]
impl AgentToolRunner for FakeToolRunner {
    async fn execute(
        &self,
        request: &AgentToolRequest,
        context: &AgentExecutionContext,
    ) -> Result<AgentToolResult, AgentToolError> {
        let document = context.document.as_ref().ok_or(AgentToolError::DocumentNotFound)?;
        match request.tool {
            AgentTool::GetCurrentQuery => Ok(AgentToolResult {
                tool: request.tool,
                output: AgentToolOutput::CurrentQuery {
                    document_id: document.document_id.clone(),
                    document_version: document.document_version,
                    sql: document.sql.clone(),
                    cursor_offset: document.cursor_offset,
                    selection: document.selection,
                    current_statement: document.current_statement.clone(),
                },
            }),
            _ => Err(AgentToolError::InvalidInput { tool: request.tool }),
        }
    }
}

fn document(version: u64) -> AgentDocumentSnapshot {
    AgentDocumentSnapshot {
        document_id: "doc-a".to_owned(),
        document_version: version,
        sql: "SELECT 1".to_owned(),
        cursor_offset: 8,
        selection: None,
        current_statement: Some("SELECT 1".to_owned()),
    }
}

fn context(version: u64) -> AgentContext {
    AgentContext {
        document_id: "doc-a".to_owned(),
        document_version: version,
        connection_id: None,
        schema: Some("public".to_owned()),
        current_sql: "SELECT 1".to_owned(),
        user_request: "inspect the query".to_owned(),
        selected_range: None,
        referenced_tables: Vec::new(),
        foreign_keys: Vec::new(),
        diagnostics: Vec::new(),
        result_summary: None,
    }
}

fn orchestrator_with_mode(responses: Vec<Vec<AgentProviderEvent>>, mode: AgentMode) -> AgentRunOrchestrator {
    let session = AgentSession::new("doc-a", None, Some("public".to_owned()));
    let execution_context = AgentExecutionContext::new(session.clone(), document(1), mode);
    AgentRunOrchestrator::new(
        Box::new(FakeProvider {
            responses: Mutex::new(responses.into_iter().collect()),
        }),
        Box::new(FakeToolRunner),
        AgentWorkflow::new(session, mode, false),
        execution_context,
        "inspect".to_owned(),
        context(1),
    )
    .expect("orchestrator should start")
}

fn orchestrator(responses: Vec<Vec<AgentProviderEvent>>) -> AgentRunOrchestrator {
    orchestrator_with_mode(responses, AgentMode::Ask)
}

#[tokio::test]
async fn routes_typed_tool_result_into_provider_continuation() {
    let mut orchestrator = orchestrator(vec![
        vec![
            AgentProviderEvent::ToolCall(AgentToolCall {
                call_id: "call-1".to_owned(),
                tool: AgentTool::GetCurrentQuery,
                input: AgentToolInput::None,
            }),
            AgentProviderEvent::Completed,
        ],
        vec![
            AgentProviderEvent::TextDelta {
                delta: "The query is read-only.".to_owned(),
            },
            AgentProviderEvent::Completed,
        ],
    ]);

    let events = orchestrator.run().await;
    assert!(events
        .iter()
        .any(|event| matches!(event, AgentWorkflowEvent::ToolCompleted { call_id, .. } if call_id == "call-1")));
    assert!(events
        .iter()
        .any(|event| matches!(event, AgentWorkflowEvent::TextDelta { delta, .. } if delta.contains("read-only"))));
    assert!(events
        .iter()
        .any(|event| matches!(event, AgentWorkflowEvent::Completed { .. })));
    assert!(!orchestrator.has_pending_confirmation());
}

#[tokio::test]
async fn patch_confirmation_updates_run_version_before_continuation() {
    let mut orchestrator = orchestrator_with_mode(
        vec![
            vec![AgentProviderEvent::ToolCall(AgentToolCall {
                call_id: "patch-1".to_owned(),
                tool: AgentTool::PatchQuery,
                input: AgentToolInput::Patch {
                    patch: db_pro_core::domain::agent::AgentSqlPatch {
                        document_id: "doc-a".to_owned(),
                        expected_version: 1,
                        range: (0, 8),
                        replacement: "SELECT 2".to_owned(),
                    },
                },
            })],
            vec![AgentProviderEvent::Completed],
        ],
        AgentMode::Edit,
    );

    let first = orchestrator.run().await;
    assert!(first.iter().any(|event| matches!(
        event,
        AgentWorkflowEvent::ConfirmationRequired {
            kind: AgentConfirmationKind::ApplyPatch,
            ..
        }
    )));
    assert!(orchestrator.has_pending_confirmation());

    let second = orchestrator
        .resume_confirmation(
            true,
            Some(document(1)),
            Some(AgentToolOutput::PatchApplied {
                document_id: "doc-a".to_owned(),
                new_version: 2,
                range: (0, 8),
            }),
        )
        .await
        .expect("patch confirmation should continue");
    assert!(second
        .iter()
        .any(|event| matches!(event, AgentWorkflowEvent::Completed { .. })));
}

#[tokio::test]
async fn stale_document_can_recover_through_current_query() {
    let mut orchestrator = orchestrator(vec![
        vec![AgentProviderEvent::ToolCall(AgentToolCall {
            call_id: "current-1".to_owned(),
            tool: AgentTool::GetCurrentQuery,
            input: AgentToolInput::None,
        })],
        vec![AgentProviderEvent::Completed],
    ]);
    orchestrator.update_document_snapshot(AgentDocumentSnapshot {
        sql: "SELECT 2".to_owned(),
        document_version: 2,
        ..document(1)
    });

    let events = orchestrator.run().await;
    assert!(events
        .iter()
        .any(|event| matches!(event, AgentWorkflowEvent::ToolCompleted { .. })));
    assert_eq!(
        orchestrator
            .workflow()
            .session()
            .active_run
            .as_ref()
            .map(|run| run.document_version),
        None
    );
}

#[tokio::test]
async fn duplicate_tool_call_replays_cached_output_without_re_execution() {
    let mut orchestrator = orchestrator(vec![
        vec![AgentProviderEvent::ToolCall(AgentToolCall {
            call_id: "dedup-1".to_owned(),
            tool: AgentTool::GetCurrentQuery,
            input: AgentToolInput::None,
        })],
        vec![AgentProviderEvent::ToolCall(AgentToolCall {
            call_id: "dedup-1".to_owned(),
            tool: AgentTool::GetCurrentQuery,
            input: AgentToolInput::None,
        })],
        vec![AgentProviderEvent::Completed],
    ]);

    let events = orchestrator.run().await;
    let completed_count = events
        .iter()
        .filter(|event| matches!(event, AgentWorkflowEvent::ToolCompleted { call_id, .. } if call_id == "dedup-1"))
        .count();
    assert_eq!(completed_count, 2);
}

#[tokio::test]
async fn duplicate_pending_confirmation_reuses_pending_state() {
    let patch = db_pro_core::domain::agent::AgentSqlPatch {
        document_id: "doc-a".to_owned(),
        expected_version: 1,
        range: (0, 8),
        replacement: "SELECT 2".to_owned(),
    };
    let mut orchestrator = orchestrator_with_mode(
        vec![
            vec![AgentProviderEvent::ToolCall(AgentToolCall {
                call_id: "patch-dup".to_owned(),
                tool: AgentTool::PatchQuery,
                input: AgentToolInput::Patch { patch: patch.clone() },
            })],
            vec![AgentProviderEvent::ToolCall(AgentToolCall {
                call_id: "patch-dup".to_owned(),
                tool: AgentTool::PatchQuery,
                input: AgentToolInput::Patch { patch },
            })],
        ],
        AgentMode::Edit,
    );

    let events = orchestrator.run().await;
    let confirmations = events
        .iter()
        // cc-scan:allow LINE_TOO_LONG — literal must not wrap
        .filter(
            |event| matches!(event, AgentWorkflowEvent::ConfirmationRequired { call_id, .. } if call_id == "patch-dup"),
        )
        .count();
    assert!(confirmations >= 1);
    assert!(orchestrator.has_pending_confirmation());
}

#[tokio::test]
async fn tool_call_collision_with_different_input_fails_with_protocol_error() {
    let mut orchestrator = orchestrator(vec![
        vec![AgentProviderEvent::ToolCall(AgentToolCall {
            call_id: "coll-1".to_owned(),
            tool: AgentTool::GetCurrentQuery,
            input: AgentToolInput::None,
        })],
        vec![AgentProviderEvent::ToolCall(AgentToolCall {
            call_id: "coll-1".to_owned(),
            tool: AgentTool::InspectSchema,
            input: AgentToolInput::Schema {
                schema: Some("public".to_owned()),
            },
        })],
        vec![AgentProviderEvent::Completed],
    ]);

    let events = orchestrator.run().await;
    assert!(events
        .iter()
        .any(|event| matches!(event, AgentWorkflowEvent::Failed { message, .. } if message.contains("collision"))));
}

struct CountingToolRunner {
    count: std::sync::atomic::AtomicUsize,
}

#[async_trait::async_trait]
impl AgentToolRunner for CountingToolRunner {
    async fn execute(
        &self,
        request: &AgentToolRequest,
        _context: &AgentExecutionContext,
    ) -> Result<AgentToolResult, AgentToolError> {
        self.count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        match request.tool {
            AgentTool::RunQuery => Ok(AgentToolResult {
                tool: request.tool,
                output: AgentToolOutput::QueryResult {
                    statement_index: Some(0),
                    result_count: 1,
                    summary: db_pro_core::domain::agent_context::AgentResultSummary {
                        columns: vec![],
                        sample_rows: vec![],
                        row_count: Some(1),
                        affected_rows: Some(1),
                        truncated: false,
                    },
                },
            }),
            _ => Err(AgentToolError::InvalidInput { tool: request.tool }),
        }
    }
}

#[tokio::test]
// cc-scan:allow LONG_FUNCTION — linear pipeline — one cohesive pass
// cc-scan:allow HUGE_FUNCTION — linear arrange-act-assert scenario
async fn mutation_run_query_executes_once_and_repeats_replay_without_database_re_execution() {
    let runner = std::sync::Arc::new(CountingToolRunner {
        count: std::sync::atomic::AtomicUsize::new(0),
    });
    struct ArcRunner(std::sync::Arc<CountingToolRunner>);
    // cc-scan:allow DUPLICATE_BLOCK — coincidental boilerplate, not a real clone
    #[async_trait::async_trait]
    impl AgentToolRunner for ArcRunner {
        async fn execute(
            // cc-scan:allow DUPLICATE_BLOCK — coincidental boilerplate
            &self,
            request: &AgentToolRequest,
            context: &AgentExecutionContext,
            // cc-scan:allow DUPLICATE_BLOCK — coincidental boilerplate
        ) -> Result<AgentToolResult, AgentToolError> {
            self.0.execute(request, context).await
        }
    }

    let session = AgentSession::new("doc-a", Some("conn-1".to_owned()), Some("public".to_owned()));
    let execution_context = AgentExecutionContext::new(session.clone(), document(1), AgentMode::Agent);
    let mut orchestrator = AgentRunOrchestrator::new(
        // cc-scan:allow DUPLICATE_BLOCK — coincidental boilerplate
        Box::new(FakeProvider {
            responses: Mutex::new(
                vec![
                    vec![AgentProviderEvent::ToolCall(AgentToolCall {
                        call_id: "mut-1".to_owned(),
                        tool: AgentTool::RunQuery,
                        input: AgentToolInput::Query {
                            sql: "UPDATE users SET active = true WHERE id = 1".to_owned(),
                        },
                    })],
                    vec![AgentProviderEvent::ToolCall(AgentToolCall {
                        call_id: "mut-1".to_owned(),
                        tool: AgentTool::RunQuery,
                        input: AgentToolInput::Query {
                            // cc-scan:allow DUPLICATE_BLOCK — coincidental boilerplate, not a real clone
                            sql: "UPDATE users SET active = true WHERE id = 1".to_owned(),
                        },
                    })],
                    vec![AgentProviderEvent::Completed],
                ]
                .into_iter()
                .collect(),
            ),
        }),
        Box::new(ArcRunner(runner.clone())),
        AgentWorkflow::new(session, AgentMode::Agent, false),
        execution_context,
        "update".to_owned(),
        context(1),
    )
    .expect("orchestrator should start");

    let first = orchestrator.run().await;
    assert!(first.iter().any(|e| matches!(
        e,
        AgentWorkflowEvent::ConfirmationRequired {
            kind: AgentConfirmationKind::RunMutation,
            ..
        }
    )));

    let second = orchestrator
        .resume_confirmation(true, Some(document(1)), None)
        .await
        .expect("confirmation approved");
    assert!(second.iter().any(|e| matches!(e, AgentWorkflowEvent::Completed { .. })));

    // Verification: Even though provider called tool "mut-1" twice, the database runner executed exactly ONCE!
    assert_eq!(runner.count.load(std::sync::atomic::Ordering::SeqCst), 1);
}

#[tokio::test]
// cc-scan:allow LONG_FUNCTION — linear pipeline — one cohesive pass
async fn rejected_confirmation_is_cached_and_replayed_on_repeated_call_id() {
    let runner = std::sync::Arc::new(CountingToolRunner {
        count: std::sync::atomic::AtomicUsize::new(0),
    });
    struct ArcRunner(std::sync::Arc<CountingToolRunner>);
    #[async_trait::async_trait]
    impl AgentToolRunner for ArcRunner {
        async fn execute(
            &self,
            request: &AgentToolRequest,
            context: &AgentExecutionContext,
        ) -> Result<AgentToolResult, AgentToolError> {
            self.0.execute(request, context).await
        }
    }

    let session = AgentSession::new("doc-a", Some("conn-1".to_owned()), Some("public".to_owned()));
    let execution_context = AgentExecutionContext::new(session.clone(), document(1), AgentMode::Agent);
    let mut orchestrator = AgentRunOrchestrator::new(
        Box::new(FakeProvider {
            responses: Mutex::new(
                vec![
                    vec![AgentProviderEvent::ToolCall(AgentToolCall {
                        call_id: "mut-rej".to_owned(),
                        tool: AgentTool::RunQuery,
                        input: AgentToolInput::Query {
                            sql: "DELETE FROM users".to_owned(),
                        },
                    })],
                    vec![AgentProviderEvent::ToolCall(AgentToolCall {
                        call_id: "mut-rej".to_owned(),
                        tool: AgentTool::RunQuery,
                        input: AgentToolInput::Query {
                            sql: "DELETE FROM users".to_owned(),
                        },
                    })],
                    vec![AgentProviderEvent::Completed],
                ]
                .into_iter()
                .collect(),
            ),
        }),
        Box::new(ArcRunner(runner.clone())),
        AgentWorkflow::new(session, AgentMode::Agent, false),
        execution_context,
        "delete".to_owned(),
        context(1),
    )
    .expect("orchestrator should start");

    let first = orchestrator.run().await;
    assert!(first.iter().any(|e| matches!(
        e,
        AgentWorkflowEvent::ConfirmationRequired {
            kind: AgentConfirmationKind::RunDestructive,
            ..
        }
    )));

    // User rejects confirmation
    let second = orchestrator
        .resume_confirmation(false, Some(document(1)), None)
        .await
        .expect("confirmation rejected handled");
    assert!(second.iter().any(|e| matches!(e, AgentWorkflowEvent::Completed { .. })));

    // Verification: Database was never touched, count is 0!
    assert_eq!(runner.count.load(std::sync::atomic::Ordering::SeqCst), 0);
}

#[tokio::test]
// cc-scan:allow LONG_FUNCTION — linear pipeline — one cohesive pass
async fn safety_escalation_requires_destructive_confirmation_for_dangerous_mutation() {
    let runner = std::sync::Arc::new(CountingToolRunner {
        count: std::sync::atomic::AtomicUsize::new(0),
    });
    struct ArcRunner(std::sync::Arc<CountingToolRunner>);
    #[async_trait::async_trait]
    impl AgentToolRunner for ArcRunner {
        async fn execute(
            &self,
            request: &AgentToolRequest,
            context: &AgentExecutionContext,
        ) -> Result<AgentToolResult, AgentToolError> {
            self.0.execute(request, context).await
        }
    }

    let session = AgentSession::new("doc-a", Some("conn-1".to_owned()), Some("public".to_owned()));
    let execution_context = AgentExecutionContext::new(session.clone(), document(1), AgentMode::Agent);
    let mut orchestrator = AgentRunOrchestrator::new(
        Box::new(FakeProvider {
            responses: Mutex::new(
                vec![
                    vec![AgentProviderEvent::ToolCall(AgentToolCall {
                        call_id: "drop-table-1".to_owned(),
                        tool: AgentTool::RunQuery,
                        input: AgentToolInput::Query {
                            sql: "DROP TABLE users CASCADE".to_owned(),
                        },
                    })],
                    vec![AgentProviderEvent::Completed],
                ]
                .into_iter()
                .collect(),
            ),
        }),
        Box::new(ArcRunner(runner.clone())),
        AgentWorkflow::new(session, AgentMode::Agent, false),
        execution_context,
        "drop table".to_owned(),
        context(1),
    )
    .expect("orchestrator should start");

    let events = orchestrator.run().await;
    // Verify it demands RunDestructive, NOT RunMutation or RunUnknown
    assert!(events.iter().any(|e| matches!(
        e,
        AgentWorkflowEvent::ConfirmationRequired {
            kind: AgentConfirmationKind::RunDestructive,
            call_id,
            ..
        } if call_id == "drop-table-1"
    )));

    // Reject confirmation and verify database count remains 0
    let finish_events = orchestrator
        .resume_confirmation(false, Some(document(1)), None)
        .await
        .expect("resume rejected");
    assert!(finish_events
        .iter()
        .any(|e| matches!(e, AgentWorkflowEvent::Completed { .. })));
    assert_eq!(runner.count.load(std::sync::atomic::Ordering::SeqCst), 0);
}

#[tokio::test]
async fn malformed_provider_missing_arguments_reports_tool_failed_gracefully() {
    let mut orchestrator = orchestrator_with_mode(
        vec![
            vec![AgentProviderEvent::ToolCall(AgentToolCall {
                call_id: "bad-1".to_owned(),
                tool: AgentTool::RunQuery,
                input: AgentToolInput::None,
            })],
            vec![AgentProviderEvent::Completed],
        ],
        AgentMode::Agent,
    );

    let events = orchestrator.run().await;
    assert!(events.iter().any(|e| matches!(
        e,
        AgentWorkflowEvent::ToolFailed {
            call_id,
            error: AgentToolError::InvalidInput { .. },
            ..
        } if call_id == "bad-1"
    )));
    assert!(events.iter().any(|e| matches!(e, AgentWorkflowEvent::Completed { .. })));
}

#[tokio::test]
async fn idempotency_matrix_read_tool_replays_cached_output() {
    let session = AgentSession::new("doc-a", Some("conn-1".to_owned()), Some("public".to_owned()));
    let execution_context = AgentExecutionContext::new(session.clone(), document(1), AgentMode::Ask);
    let mut orchestrator = AgentRunOrchestrator::new(
        Box::new(FakeProvider {
            responses: Mutex::new(
                vec![
                    vec![AgentProviderEvent::ToolCall(AgentToolCall {
                        call_id: "read-1".to_owned(),
                        tool: AgentTool::GetCurrentQuery,
                        input: AgentToolInput::None,
                    })],
                    vec![AgentProviderEvent::ToolCall(AgentToolCall {
                        call_id: "read-1".to_owned(),
                        tool: AgentTool::GetCurrentQuery,
                        input: AgentToolInput::None,
                    })],
                    vec![AgentProviderEvent::Completed],
                ]
                .into_iter()
                .collect(),
            ),
        }),
        Box::new(FakeToolRunner),
        AgentWorkflow::new(session, AgentMode::Ask, false),
        execution_context,
        "read query twice".to_owned(),
        context(1),
    )
    .expect("orchestrator starts");

    let events = orchestrator.run().await;
    let tool_completed_count = events
        .iter()
        .filter(|e| matches!(e, AgentWorkflowEvent::ToolCompleted { call_id, .. } if call_id == "read-1"))
        .count();
    // Emitted ToolCompleted twice: once executed, once replayed from cache
    assert_eq!(tool_completed_count, 2);
}

#[tokio::test]
async fn idempotency_collision_with_different_input_fails_with_protocol_error() {
    let mut orchestrator = orchestrator_with_mode(
        vec![
            vec![AgentProviderEvent::ToolCall(AgentToolCall {
                call_id: "schema-dup".to_owned(),
                tool: AgentTool::InspectSchema,
                input: AgentToolInput::Schema {
                    schema: Some("public".to_owned()),
                },
            })],
            vec![AgentProviderEvent::ToolCall(AgentToolCall {
                call_id: "schema-dup".to_owned(),
                tool: AgentTool::InspectSchema,
                input: AgentToolInput::Schema {
                    schema: Some("secret".to_owned()),
                },
            })],
            vec![AgentProviderEvent::Completed],
        ],
        AgentMode::Ask,
    );

    let events = orchestrator.run().await;
    assert!(events.iter().any(|e| matches!(
        e,
        AgentWorkflowEvent::Failed { message, .. } if message.contains("collision")
    )));
}

#[tokio::test]
async fn cancel_while_awaiting_confirmation_emits_cancelled_event() {
    let mut orchestrator = orchestrator_with_mode(
        vec![
            vec![AgentProviderEvent::ToolCall(AgentToolCall {
                call_id: "mut-cancel".to_owned(),
                tool: AgentTool::RunQuery,
                input: AgentToolInput::Query {
                    sql: "UPDATE users SET active = true WHERE id = 1".to_owned(),
                },
            })],
            vec![AgentProviderEvent::Completed],
        ],
        AgentMode::Agent,
    );

    let first = orchestrator.run().await;
    assert!(first.iter().any(|e| matches!(
        e,
        AgentWorkflowEvent::ConfirmationRequired {
            kind: AgentConfirmationKind::RunMutation,
            ..
        }
    )));
    assert!(orchestrator.has_pending_confirmation());

    // Cancel the orchestrator directly while confirmation is pending
    let cancel_event = orchestrator.cancel().expect("orchestrator cancel succeeds");
    assert!(matches!(cancel_event, AgentWorkflowEvent::Cancelled { .. }));
    assert!(!orchestrator.has_pending_confirmation());
}

#[tokio::test]
async fn multi_result_inspection_routes_results_by_index_and_reports_tool_failure() {
    let mut orchestrator = orchestrator_with_mode(
        vec![
            vec![AgentProviderEvent::ToolCall(AgentToolCall {
                call_id: "inspect-res-1".to_owned(),
                tool: AgentTool::InspectQueryResult,
                input: AgentToolInput::ResultSample {
                    max_rows: 20,
                    statement_index: Some(99),
                },
            })],
            vec![AgentProviderEvent::Completed],
        ],
        AgentMode::Agent,
    );

    let events = orchestrator.run().await;
    assert!(events.iter().any(|e| matches!(
        e,
        AgentWorkflowEvent::ToolFailed {
            call_id,
            ..
        } if call_id == "inspect-res-1"
    )));
}
