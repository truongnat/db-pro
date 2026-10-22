# Repository File → Folder Mapping

Baseline: `be2e431b07a8fcfbf39bf16fa1deefd600273633`.

Physical path changes only. Logical Rust modules/public exports and historical plan states stay unchanged.

Pre-move SHA-256 and byte counts for every file are retained in [MAPPING.json](MAPPING.json).

## Rust source and tests (40 files)

| Before | After |
|---|---|
| `crates/ui/src/components/input/tests.rs` | `crates/ui/src/components/input/tests/mod_tests.rs` |
| `crates/ui/src/components/select/tests.rs` | `crates/ui/src/components/select/tests/mod_tests.rs` |
| `crates/ui/src/components/dialog/tests.rs` | `crates/ui/src/components/dialog/tests/mod_tests.rs` |
| `crates/ui/src/components/form/tests.rs` | `crates/ui/src/components/form/tests/mod_tests.rs` |
| `crates/ui/src/components/table/tests.rs` | `crates/ui/src/components/table/tests/mod_tests.rs` |
| `crates/ui/src/components/tabs/tests.rs` | `crates/ui/src/components/tabs/tests/mod_tests.rs` |
| `crates/ui/src/diagram/tests.rs` | `crates/ui/src/diagram/tests/mod_tests.rs` |
| `crates/app/src/connection/tests.rs` | `crates/app/src/connection/tests/mod_tests.rs` |
| `crates/core/src/application/schema_service_tests.rs` | `crates/core/src/application/tests/schema_service_tests.rs` |
| `crates/core/src/application/query_service/tests.rs` | `crates/core/src/application/tests/query_service_tests.rs` |
| `crates/core/src/application/connection_service/tests.rs` | `crates/core/src/application/tests/connection_service_tests.rs` |
| `crates/core/src/application/connection_service/tests/connection_service_security_tests.rs` | `crates/core/src/application/tests/connection_service_security_tests.rs` |
| `crates/core/src/application/connection_service/tests/connection_service_provider_tests.rs` | `crates/core/src/application/tests/connection_service_provider_tests.rs` |
| `crates/core/src/application/connection_service/tests/connection_service_lifecycle_tests.rs` | `crates/core/src/application/tests/connection_service_lifecycle_tests.rs` |
| `crates/core/src/domain/safety/tests.rs` | `crates/core/src/domain/tests/safety_tests.rs` |
| `crates/core/src/domain/agent_workflow/tests.rs` | `crates/core/src/domain/tests/agent_workflow_tests.rs` |
| `crates/native-app/src/translate_tests.rs` | `crates/native-app/src/tests/translate_tests.rs` |
| `crates/core/src/domain/capability_limitations.rs` | `crates/core/src/domain/capabilities/capability_limitations.rs` |
| `crates/core/src/domain/capability_presets.rs` | `crates/core/src/domain/capabilities/capability_presets.rs` |
| `crates/core/src/domain/safety_cte.rs` | `crates/core/src/domain/safety/safety_cte.rs` |
| `crates/core/src/domain/safety_lexer.rs` | `crates/core/src/domain/safety/safety_lexer.rs` |
| `crates/core/src/domain/safety_policy.rs` | `crates/core/src/domain/safety/safety_policy.rs` |
| `crates/core/src/application/multi_query_execution.rs` | `crates/core/src/application/query_service/multi_query_execution.rs` |
| `crates/core/src/application/query_classification.rs` | `crates/core/src/application/query_service/query_classification.rs` |
| `crates/core/src/application/schema_ddl.rs` | `crates/core/src/application/schema_service/schema_ddl.rs` |
| `crates/core/src/application/schema_table_info.rs` | `crates/core/src/application/schema_service/schema_table_info.rs` |
| `crates/native-app/src/translate.rs` | `crates/native-app/src/translate/mod.rs` |
| `crates/native-app/src/translate_cmd.rs` | `crates/native-app/src/translate/translate_cmd.rs` |
| `crates/native-app/src/tests/translate_cmd_tests.rs` | `crates/native-app/src/translate/tests/translate_cmd_tests.rs` |
| `crates/native-app/src/translate_evt.rs` | `crates/native-app/src/translate/translate_evt.rs` |
| `crates/native-app/src/tests/translate_evt_tests.rs` | `crates/native-app/src/translate/tests/translate_evt_tests.rs` |
| `crates/native-app/src/translate_map.rs` | `crates/native-app/src/translate/translate_map.rs` |
| `crates/native-app/src/tests/translate_map_tests.rs` | `crates/native-app/src/translate/tests/translate_map_tests.rs` |
| `crates/runtime/src/agent.rs` | `crates/runtime/src/agent/mod.rs` |
| `crates/runtime/src/agent_executor.rs` | `crates/runtime/src/agent/executor.rs` |
| `crates/runtime/src/agent_orchestrator.rs` | `crates/runtime/src/agent/orchestrator.rs` |
| `crates/runtime/src/tests/agent_tests.rs` | `crates/runtime/src/agent/tests/agent_tests.rs` |
| `crates/runtime/src/tests/agent_executor_tests.rs` | `crates/runtime/src/agent/tests/agent_executor_tests.rs` |
| `crates/runtime/src/tests/agent_orchestrator_tests.rs` | `crates/runtime/src/agent/tests/agent_orchestrator_tests.rs` |
| `crates/app/src/tools/problems_view.rs` | `crates/app/src/shell/sidebar/problems_view.rs` |

## Documentation and retained evidence (53 files)

| Before | After |
|---|---|
| `plans/00-index.md` | `docs/plans/project/00-index.md` |
| `plans/01-overview.md` | `docs/plans/project/01-overview.md` |
| `plans/02-backend-tasks.md` | `docs/plans/project/02-backend-tasks.md` |
| `plans/03-frontend-tasks.md` | `docs/plans/project/03-frontend-tasks.md` |
| `plans/04-testing-tasks.md` | `docs/plans/project/04-testing-tasks.md` |
| `plans/05-cicd-tasks.md` | `docs/plans/project/05-cicd-tasks.md` |
| `plans/06-database-tasks.md` | `docs/plans/project/06-database-tasks.md` |
| `plans/07-current-status.md` | `docs/plans/project/07-current-status.md` |
| `tasks/plan.md` | `docs/plans/project/core-component-standardization/plan.md` |
| `tasks/todo.md` | `docs/plans/project/core-component-standardization/todo.md` |
| `QA_AUDIT_REPORT.md` | `docs/plans/active/bsn-real-data-native-audit/QA_AUDIT_REPORT.md` |
| `CLEAN_CODE_AUDIT_REPORT.md` | `docs/quality/CLEAN_CODE_AUDIT_REPORT.md` |
| `open-api-style.md` | `docs/design/reference/open-api-style.md` |
| `open-ai-refer.md` | `docs/design/reference/open-ai-refer.md` |
| `docs/07-fe-architecture.md` | `docs/architecture/07-fe-architecture.md` |
| `docs/08-technology-decisions.md` | `docs/architecture/08-technology-decisions.md` |
| `docs/09-architecture-decisions.md` | `docs/architecture/09-architecture-decisions.md` |
| `docs/rendering-analysis.md` | `docs/architecture/rendering-analysis.md` |
| `docs/rendering-history.md` | `docs/architecture/rendering-history.md` |
| `docs/dev-tools-audit-engine.md` | `docs/ui-audit/dev-tools-audit-engine.md` |
| `docs/dev-tools-audit-engine-vi.md` | `docs/ui-audit/dev-tools-audit-engine-vi.md` |
| `docs/introspection-audit-2026-08-13.md` | `docs/quality/introspection-audit-2026-08-13.md` |
| `docs/screen_1_sql_editor.html` | `docs/design/mockups/screen_1_sql_editor.html` |
| `docs/screen_component_library.html` | `docs/design/mockups/screen_component_library.html` |
| `docs/screen_explorer.html` | `docs/design/mockups/screen_explorer.html` |
| `docs/plans/dead-code-analysis-2026-08-13.md` | `docs/plans/archived/dead-code-analysis-2026-08-13.md` |
| `docs/plans/performance-baseline-audit-2026-08-13.md` | `docs/plans/archived/performance-baseline-audit-2026-08-13.md` |
| `docs/plans/performance-optimization-report-2026-08-13.md` | `docs/plans/archived/performance-optimization-report-2026-08-13.md` |
| `docs/stitch_screens/01_db_pro___sql_editor_and_results_grid_native_theme_fixed.html` | `docs/design/stitch_screens/01_db_pro___sql_editor_and_results_grid_native_theme_fixed.html` |
| `docs/stitch_screens/02_db_pro___sql_editor_and_results_grid_native_theme.html` | `docs/design/stitch_screens/02_db_pro___sql_editor_and_results_grid_native_theme.html` |
| `docs/stitch_screens/03_db_pro___er_diagram_audit_schema.html` | `docs/design/stitch_screens/03_db_pro___er_diagram_audit_schema.html` |
| `docs/stitch_screens/04_db_pro___data_activity_f1_stable_table_reference_identity.html` | `docs/design/stitch_screens/04_db_pro___data_activity_f1_stable_table_reference_identity.html` |
| `docs/stitch_screens/05_db_pro_component_library___inputs_selection_and_navigation.html` | `docs/design/stitch_screens/05_db_pro_component_library___inputs_selection_and_navigation.html` |
| `docs/stitch_screens/06_designmd.html` | `docs/design/stitch_screens/06_designmd.html` |
| `docs/stitch_screens/07_db_pro_component_library___sql_editor_and_results_grid.html` | `docs/design/stitch_screens/07_db_pro_component_library___sql_editor_and_results_grid.html` |
| `docs/stitch_screens/08_db_pro___sql_editor_and_results_grid_native_theme.html` | `docs/design/stitch_screens/08_db_pro___sql_editor_and_results_grid_native_theme.html` |
| `docs/stitch_screens/09_db_pro___schema_compare_f1_block_incomplete_migration_ddl.html` | `docs/design/stitch_screens/09_db_pro___schema_compare_f1_block_incomplete_migration_ddl.html` |
| `docs/stitch_screens/10_db_pro___er_diagram_f1_schema_scoped_graph_refresh.html` | `docs/design/stitch_screens/10_db_pro___er_diagram_f1_schema_scoped_graph_refresh.html` |
| `docs/stitch_screens/11_db_pro___explorer_f1_object_filter_workbench.html` | `docs/design/stitch_screens/11_db_pro___explorer_f1_object_filter_workbench.html` |
| `docs/stitch_screens/12_db_pro___execution_history_f1_unified_execution_history.html` | `docs/design/stitch_screens/12_db_pro___execution_history_f1_unified_execution_history.html` |
| `docs/stitch_screens/13_db_pro___execution_history_proposed_f1.html` | `docs/design/stitch_screens/13_db_pro___execution_history_proposed_f1.html` |
| `docs/stitch_screens/14_db_pro___queries_activity_f1_safe_open_and_connection_context.html` | `docs/design/stitch_screens/14_db_pro___queries_activity_f1_safe_open_and_connection_context.html` |
| `docs/stitch_screens/15_db_pro___files_workspace_f1_trust_and_file_mutation_review.html` | `docs/design/stitch_screens/15_db_pro___files_workspace_f1_trust_and_file_mutation_review.html` |
| `docs/stitch_screens/16_db_pro___rebind_a_missing_table_reference.html` | `docs/design/stitch_screens/16_db_pro___rebind_a_missing_table_reference.html` |
| `docs/stitch_screens/17_db_pro___queries_activity_f1_source_context_unavailable.html` | `docs/design/stitch_screens/17_db_pro___queries_activity_f1_source_context_unavailable.html` |
| `docs/stitch_screens/18_db_pro_component_library___feedback_dialogs_and_commands.html` | `docs/design/stitch_screens/18_db_pro_component_library___feedback_dialogs_and_commands.html` |
| `docs/stitch_screens/19_db_pro_component_library.html` | `docs/design/stitch_screens/19_db_pro_component_library.html` |
| `docs/stitch_screens/inventory.json` | `docs/design/stitch_screens/inventory.json` |
| `screenshots/query-workspace-zed-shell/A-untitled-1280x800.png` | `docs/plans/completed/query-workspace-zed-shell/screenshots/A-untitled-1280x800.png` |
| `screenshots/query-workspace-zed-shell/A-untitled-1440x900.png` | `docs/plans/completed/query-workspace-zed-shell/screenshots/A-untitled-1440x900.png` |
| `screenshots/query-workspace-zed-shell/A-untitled-dark-1280x800.png` | `docs/plans/completed/query-workspace-zed-shell/screenshots/A-untitled-dark-1280x800.png` |
| `screenshots/query-workspace-zed-shell/theme-dark-1280x800.png` | `docs/plans/completed/query-workspace-zed-shell/screenshots/theme-dark-1280x800.png` |
| `screenshots/query-workspace-zed-shell/theme-light-1280x800.png` | `docs/plans/completed/query-workspace-zed-shell/screenshots/theme-light-1280x800.png` |

## Deliberately stable paths

- Workspace crate boundaries, Cargo manifests and entrypoint files remain stable.
- `docs/plans/STATUS.md`, `FEATURE_LIFECYCLE.md`, release evidence and active/completed lifecycle classifications are not reorganized.
- `docs/10-egui-native-migration-plan.md` and `docs/DESIGN.md` remain discoverable entrypoints.
- Instructions, skills, review infrastructure, configuration, build directories and user databases are not moved.
- Revision-pinned source citations and verbatim historical command transcripts retain their historical paths. Use the table above to locate the corresponding file in this tree.
