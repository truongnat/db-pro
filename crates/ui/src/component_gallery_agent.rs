use super::*;
use egui::{RichText, Ui};
use lucide_icons::Icon;

impl DbProApp {
    pub(super) fn draw_gallery_agent_ui_section(&mut self, ui: &mut Ui) {
        let theme = self.theme;
        self.draw_section_heading(
            ui,
            "AI Agent Workspace & Execution Components",
            "Context awareness, agent reasoning trace, plan checklist, tool approvals, transaction bar, and safe execution boundaries.",
        );

        Card::new(theme).show(ui, |ui| {
            self.draw_gallery_agent_composer(ui);
            self.draw_gallery_agent_context(ui);
            self.draw_gallery_agent_status_and_actions(ui);
            self.draw_gallery_agent_reasoning_and_plan(ui);
            self.draw_gallery_agent_tools_and_approval(ui);
            self.draw_gallery_agent_transaction_controls(ui);
        });

        self.draw_gallery_destructive_dialog(ui);
    }

    fn draw_gallery_agent_composer(&mut self, ui: &mut Ui) {
        let theme = self.theme;
        self.draw_gallery_label(ui, "Agent Prompt Composer & Model Selector");

        let composer_action = AgentComposer::new(
            &mut self.gallery_state.composer_prompt,
            "Claude 3.5 Sonnet",
            AgentMode::Code,
            theme,
        )
        .is_generating(self.gallery_state.composer_is_generating)
        .token_usage(142)
        .show(ui);

        match composer_action {
            Some(AgentComposerAction::Submit) => {
                let prompt_text = self.gallery_state.composer_prompt.clone();
                self.gallery_state.composer_prompt.clear();
                self.gallery_state.toasts.show(
                    format!("Agent request submitted: {}", prompt_text),
                    ToastVariant::Default,
                    ToastPosition::BottomRight,
                );
            }
            Some(AgentComposerAction::Stop) => {
                self.gallery_state.composer_is_generating = false;
            }
            _ => {}
        }
    }

    fn draw_gallery_agent_context(&self, ui: &mut Ui) {
        let theme = self.theme;
        self.draw_gallery_label(ui, "Agent Workspace Context Bar");
        ui.horizontal_wrapped(|ui| {
            ui.label(
                RichText::new("Active Context:")
                    .size(12.0)
                    .strong()
                    .color(theme.text_muted),
            );
            ui.add_space(4.0);
            for (kind, label, removable) in [
                (ContextChipKind::Connection, "Analytics (PostgreSQL)", false),
                (ContextChipKind::Database, "production_db", false),
                (ContextChipKind::Table, "public.users", true),
                (ContextChipKind::Editor, "query.sql:1-18", true),
                (ContextChipKind::File, "schema.sql", true),
            ] {
                ContextChip::new(kind, label, theme).removable(removable).show(ui);
                ui.add_space(4.0);
            }
        });
    }

    fn draw_gallery_agent_status_and_actions(&mut self, ui: &mut Ui) {
        let theme = self.theme;
        self.draw_gallery_label(ui, "Status Badges");
        ui.horizontal_wrapped(|ui| {
            for (label, variant) in [
                ("Connected", StatusBadgeVariant::Active),
                ("Query Running...", StatusBadgeVariant::Running),
                ("Migration Passed", StatusBadgeVariant::Success),
                ("High Latency Warning", StatusBadgeVariant::Warning),
                ("Connection Dropped", StatusBadgeVariant::Destructive),
                ("Archived Partition", StatusBadgeVariant::Archived),
                ("Draft SQL", StatusBadgeVariant::Draft),
            ] {
                StatusBadge::new(label, variant, theme).show(ui);
                ui.add_space(6.0);
            }
        });

        self.draw_gallery_label(ui, "Quick Agent Action Chips");
        ui.horizontal_wrapped(|ui| {
            for action_kind in [
                AgentSqlActionKind::Explain,
                AgentSqlActionKind::Optimize,
                AgentSqlActionKind::FixError,
                AgentSqlActionKind::GenerateMigration,
                AgentSqlActionKind::DescribeSchema,
                AgentSqlActionKind::ConvertDialect,
            ] {
                if Button::new(theme)
                    .text(action_kind.label())
                    .icon(action_kind.icon())
                    .variant(ButtonVariant::Outline)
                    .size(ButtonSize::Sm)
                    .show(ui)
                    .clicked()
                {
                    self.gallery_state.agent_action_output =
                        Some(format!("Triggered agent action: {}", action_kind.label()));
                }
                ui.add_space(6.0);
            }
        });

        if let Some(ref action_text) = self.gallery_state.agent_action_output {
            Alert::new("Agent Prompt Initiated", action_text, theme)
                .variant(AlertVariant::Default)
                .show(ui);
        }
    }

    fn draw_gallery_agent_reasoning_and_plan(&mut self, ui: &mut Ui) {
        ui.columns(2, |cols| {
            self.draw_gallery_agent_reasoning(&mut cols[0]);
            self.draw_gallery_agent_plan(&mut cols[1]);
        });
    }

    fn draw_gallery_agent_reasoning(&mut self, ui: &mut Ui) {
        let theme = self.theme;
        self.draw_gallery_label(ui, "Agent Reasoning & Thought Stream");
        let mut thinking_expanded = self.gallery_state.thinking_expanded;
        AgentThinking::new(
            "1. Parsing user intent: 'Find slow queries in the past hour'.\n2. Querying pg_stat_statements for top total_exec_time.\n3. Found query #142 (avg_time = 420ms, calls = 14,200).\n4. Analyzing EXPLAIN plan: Sequential scan on table `users` filtering by `email`.\n5. Recommendation: Add B-tree index on `users(email)'.",
            &mut thinking_expanded,
            theme,
        )
        .duration("3.8s")
        .step_count(5)
        .show(ui);
        self.gallery_state.thinking_expanded = thinking_expanded;

        let mut live_active = false;
        AgentThinking::new("Analyzing database indexes...", &mut live_active, theme)
            .is_active(true)
            .show(ui);
    }

    fn draw_gallery_agent_plan(&self, ui: &mut Ui) {
        let theme = self.theme;
        self.draw_gallery_label(ui, "Agent Execution Plan (Checklist)");
        let plan_tasks = vec![
            AgentTaskItem::new(
                "Introspect table schema & column statistics",
                AgentTaskStatus::Completed,
            )
            .with_detail("42ms"),
            AgentTaskItem::new("Analyze sequential scans & explain plans", AgentTaskStatus::Completed)
                .with_detail("120ms"),
            AgentTaskItem::new("Generate concurrent index migration SQL", AgentTaskStatus::Running),
            AgentTaskItem::new("Validate safety & prepare approval card", AgentTaskStatus::Pending),
        ];
        AgentPlan::new("Index Optimization Workflow", &plan_tasks, theme).show(ui);
    }

    fn draw_gallery_agent_tools_and_approval(&mut self, ui: &mut Ui) {
        ui.columns(2, |cols| {
            self.draw_gallery_agent_tool_calls(&mut cols[0]);
            self.draw_gallery_agent_approval(&mut cols[1]);
        });
    }

    fn draw_gallery_agent_tool_calls(&mut self, ui: &mut Ui) {
        let theme = self.theme;
        self.draw_gallery_label(ui, "Tool Call Execution Step");
        let mut tool_expanded = self.gallery_state.tool_call_expanded;
        ToolCall::new(
            "introspect_schema_indexes",
            ToolCallStatus::Success,
            "{\n  \"schema\": \"public\",\n  \"table\": \"users\",\n  \"include_stats\": true\n}",
            &mut tool_expanded,
            theme,
        )
        .duration("42ms")
        .output_preview(
            "{\n  \"indexes_found\": 3,\n  \"missing_foreign_keys\": 0,\n  \"estimated_scan_cost\": 14820.5\n}",
        )
        .show(ui);
        self.gallery_state.tool_call_expanded = tool_expanded;

        let mut running_expanded = false;
        ToolCall::new(
            "analyze_query_bottlenecks",
            ToolCallStatus::Running,
            "{\n  \"query_hash\": \"0x9b4a18f\",\n  \"sample_rate\": 0.1\n}",
            &mut running_expanded,
            theme,
        )
        .duration("120ms")
        .show(ui);
    }

    fn draw_gallery_agent_approval(&mut self, ui: &mut Ui) {
        let theme = self.theme;
        self.draw_gallery_label(ui, "Dangerous / Mutating Action Approval");
        let action = ExecutionApproval::new(
            "Apply Database Migration: Add Concurrent Index",
            "public.users (1,842,109 rows affected) - zero lock impact",
            "CREATE INDEX CONCURRENTLY idx_users_email ON users(email);",
            RiskLevel::Medium,
            theme,
        )
        .show(ui);

        self.gallery_state.approval_status = match action {
            Some(ExecutionApprovalAction::Run) => {
                Some("Migration executed successfully via background worker.".to_owned())
            }
            Some(ExecutionApprovalAction::Preview) => Some("Opening interactive SQL preview buffer...".to_owned()),
            Some(ExecutionApprovalAction::Cancel) => Some("Proposal rejected by user.".to_owned()),
            None => self.gallery_state.approval_status.clone(),
        };

        if let Some(ref message) = self.gallery_state.approval_status {
            Alert::new("Execution Action Triggered", message, theme)
                .variant(AlertVariant::Default)
                .show(ui);
        }
    }

    fn draw_gallery_agent_transaction_controls(&mut self, ui: &mut Ui) {
        let theme = self.theme;
        self.draw_gallery_label(ui, "Transaction Control & Destructive Operation Safety");
        let action = TransactionBar::new(
            self.gallery_state.tx_in_transaction,
            self.gallery_state.tx_pending_mutations,
            theme,
        )
        .auto_commit(self.gallery_state.tx_auto_commit)
        .isolation_level("READ COMMITTED")
        .show(ui);
        self.apply_gallery_transaction_action(action);

        if let Some(ref message) = self.gallery_state.tx_status_message {
            Alert::new("Transaction Event", message, theme)
                .variant(AlertVariant::Default)
                .show(ui);
        }

        ui.horizontal(|ui| {
            ui.label(
                RichText::new("Destructive Action Protection:")
                    .size(12.0)
                    .color(theme.text_secondary),
            );
            ui.add_space(6.0);
            if Button::new(theme)
                .text("Trigger DROP TABLE Safety Modal")
                .icon(Icon::Trash2)
                .variant(ButtonVariant::Destructive)
                .size(ButtonSize::Sm)
                .show(ui)
                .clicked()
            {
                self.gallery_state.destructive_dialog_open = true;
                self.gallery_state.destructive_keyword.clear();
            }
        });
    }

    fn apply_gallery_transaction_action(&mut self, action: Option<TransactionAction>) {
        match action {
            Some(TransactionAction::Commit) => {
                self.gallery_state.tx_pending_mutations = 0;
                self.gallery_state.tx_in_transaction = false;
                self.gallery_state.tx_status_message =
                    Some("Transaction committed: 3 mutations written to disk.".to_owned());
            }
            Some(TransactionAction::Rollback) => {
                self.gallery_state.tx_pending_mutations = 0;
                self.gallery_state.tx_in_transaction = false;
                self.gallery_state.tx_status_message =
                    Some("Transaction rolled back: all mutations reverted.".to_owned());
            }
            Some(TransactionAction::Begin) => {
                self.gallery_state.tx_in_transaction = true;
                self.gallery_state.tx_pending_mutations = 1;
                self.gallery_state.tx_status_message =
                    Some("New transaction started with READ COMMITTED isolation.".to_owned());
            }
            Some(TransactionAction::ToggleAutoCommit(value)) => {
                self.gallery_state.tx_auto_commit = value;
            }
            None => {}
        }
    }

    fn draw_gallery_destructive_dialog(&mut self, ui: &mut Ui) {
        if !self.gallery_state.destructive_dialog_open {
            return;
        }

        let theme = self.theme;
        let mut is_open = self.gallery_state.destructive_dialog_open;
        let mut keyword = self.gallery_state.destructive_keyword.clone();
        let confirmed = DestructiveOperationDialog::new(
            &mut is_open,
            "Drop Production Table",
            "This action will permanently remove public.audit_logs (14,209,102 rows) and cascade delete all associated foreign key records. This cannot be undone.",
            "public.audit_logs",
            "DROP",
            &mut keyword,
            theme,
        )
        .show(ui);

        self.gallery_state.destructive_dialog_open = is_open;
        self.gallery_state.destructive_keyword = keyword;
        if confirmed {
            self.gallery_state.toasts.show(
                "Table public.audit_logs dropped permanently.",
                ToastVariant::Danger,
                ToastPosition::BottomRight,
            );
        }
    }

    fn draw_gallery_label(&self, ui: &mut Ui, text: &str) {
        ui.add_space(16.0);
        ui.label(RichText::new(text).size(13.0).strong().color(self.theme.text_secondary));
        ui.add_space(6.0);
    }
}
