use super::*;
use egui::{RichText, Ui};
use lucide_icons::Icon;

impl DbProApp {
    pub(super) fn draw_gallery_buttons_section(&mut self, ui: &mut Ui) {
        let theme = self.theme;
        self.draw_section_heading(
            ui,
            "Buttons",
            "Displays a button or a component that looks like a button with various variants and sizes.",
        );

        Card::new(theme).show(ui, |ui| {
            ui.label(
                RichText::new("Variants")
                    .size(13.0)
                    .strong()
                    .color(theme.text_secondary),
            );
            ui.add_space(8.0);
            ui.horizontal_wrapped(|ui| {
                Button::new(theme)
                    .text("Primary (Default)")
                    .variant(ButtonVariant::Default)
                    .show(ui);
                ui.add_space(6.0);
                Button::new(theme)
                    .text("Secondary")
                    .variant(ButtonVariant::Secondary)
                    .show(ui);
                ui.add_space(6.0);
                Button::new(theme)
                    .text("Outline")
                    .variant(ButtonVariant::Outline)
                    .show(ui);
                ui.add_space(6.0);
                Button::new(theme).text("Ghost").variant(ButtonVariant::Ghost).show(ui);
                ui.add_space(6.0);
                Button::new(theme)
                    .text("Destructive")
                    .variant(ButtonVariant::Destructive)
                    .icon(Icon::Trash2)
                    .show(ui);
                ui.add_space(6.0);
                Button::new(theme)
                    .text("Link Action")
                    .variant(ButtonVariant::Link)
                    .show(ui);
            });

            ui.add_space(16.0);
            ui.label(RichText::new("Sizes").size(13.0).strong().color(theme.text_secondary));
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                Button::new(theme).text("Small (Sm)").size(ButtonSize::Sm).show(ui);
                ui.add_space(6.0);
                Button::new(theme)
                    .text("Default (Md)")
                    .size(ButtonSize::Default)
                    .show(ui);
                ui.add_space(6.0);
                Button::new(theme).text("Large (Lg)").size(ButtonSize::Lg).show(ui);
                ui.add_space(10.0);
                Button::new(theme)
                    .icon(Icon::Plus)
                    .size(ButtonSize::Icon)
                    .variant(ButtonVariant::Outline)
                    .access_label("Add")
                    .show(ui);
                ui.add_space(6.0);
                Button::new(theme)
                    .icon(Icon::Copy)
                    .size(ButtonSize::IconSm)
                    .variant(ButtonVariant::Secondary)
                    .access_label("Copy")
                    .show(ui);
            });

            ui.add_space(16.0);
            ui.label(
                RichText::new("Interactive States & Loading")
                    .size(13.0)
                    .strong()
                    .color(theme.text_secondary),
            );
            ui.add_space(8.0);
            ui.horizontal_wrapped(|ui| {
                let loading = self.gallery_state.btn_loading;
                if Button::new(theme)
                    .text(if loading { "Processing..." } else { "Click to Load" })
                    .icon(Icon::Play)
                    .loading(loading)
                    .show(ui)
                    .clicked()
                {
                    self.gallery_state.btn_loading = true;
                }

                if loading {
                    ui.add_space(8.0);
                    if Button::new(theme)
                        .text("Stop")
                        .variant(ButtonVariant::Outline)
                        .size(ButtonSize::Sm)
                        .show(ui)
                        .clicked()
                    {
                        self.gallery_state.btn_loading = false;
                    }
                }

                ui.add_space(12.0);
                Button::new(theme)
                    .text("Always Loading (Primary)")
                    .loading(true)
                    .show(ui);

                ui.add_space(12.0);
                Button::new(theme)
                    .text("Always Loading (Outline)")
                    .variant(ButtonVariant::Outline)
                    .loading(true)
                    .show(ui);

                ui.add_space(12.0);
                Button::new(theme).text("Disabled Button").enabled(false).show(ui);
            });
        });
    }

    pub(super) fn draw_gallery_badges_section(&mut self, ui: &mut Ui) {
        let theme = self.theme;
        self.draw_section_heading(
            ui,
            "Badges & Status Pills",
            "Operational telemetry for query execution and connection state.",
        );

        Card::new(theme).show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = egui::vec2(SPACE_MD, SPACE_SM);
                ui.label(
                    RichText::new("EXECUTION BADGES")
                        .font(font_caption())
                        .color(theme.text_tertiary),
                );
                Badge::new("Ready", theme)
                    .variant(BadgeVariant::Secondary)
                    .dot(true)
                    .show(ui);
                Badge::new("Running (42 ms)", theme)
                    .variant(BadgeVariant::Default)
                    .dot(true)
                    .show(ui);
                Badge::new("Succeeded", theme)
                    .variant(BadgeVariant::Success)
                    .icon(Icon::CircleCheck)
                    .show(ui);
                Badge::new("Failed", theme)
                    .variant(BadgeVariant::Destructive)
                    .icon(Icon::CircleX)
                    .show(ui);
            });
        });
    }

    pub(super) fn draw_gallery_cards_section(&mut self, ui: &mut Ui) {
        let theme = self.theme;
        self.draw_section_heading(
            ui,
            "Cards & Metric Displays",
            "Composite containers and statistical overview cards.",
        );

        ui.columns(2, |columns| {
            MetricCard::new("Active connections", "8 / 10", theme)
                .change("+2 this session", true)
                .icon(Icon::Database)
                .show(&mut columns[0]);
            MetricCard::new("Queries today", "14,289", theme)
                .change("+18.4% vs yesterday", true)
                .icon(Icon::Terminal)
                .show(&mut columns[1]);
        });
        ui.add_space(12.0);
        ui.columns(2, |columns| {
            MetricCard::new("Slow query rate", "0.42%", theme)
                .change("−0.15% vs last week", true)
                .icon(Icon::Gauge)
                .show(&mut columns[0]);
            MetricCard::new("Staged mutations", "3", theme)
                .change("Needs review", false)
                .icon(Icon::FileEdit)
                .show(&mut columns[1]);
        });
    }

    pub(super) fn draw_gallery_devtools_section(&mut self, ui: &mut Ui) {
        let theme = self.theme;
        self.draw_section_heading(
            ui,
            "Developer Tools & Code Primitives",
            "Specialized primitives for developer environments: InlineCode, CodeBlock with copy, DiffViewer, and Schema Tree.",
        );

        Card::new(theme).show(ui, |ui| {
            // Row 1: InlineCode & CodeBlock
            ui.label(RichText::new("Inline Code & CodeBlock").size(13.0).strong().color(theme.text_secondary));
            ui.add_space(6.0);
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new("Execute query with safe limit:").size(13.0).color(theme.text_primary));
                InlineCode::new("SELECT * FROM users WHERE status = 'active' LIMIT 50;", theme).show(ui);
                ui.label(RichText::new("or connect via").size(13.0).color(theme.text_primary));
                InlineCode::new("postgresql://localhost:5432/main", theme).show(ui);
            });

            ui.add_space(14.0);

            let sample_sql = "-- Optimize query: create composite index for fast join\nCREATE INDEX CONCURRENTLY idx_users_organization_created\nON users (organization_id, created_at DESC)\nWHERE deleted_at IS NULL;\n\nSELECT u.id, u.email, o.name AS organization\nFROM users u\nJOIN organizations o ON o.id = u.organization_id\nWHERE u.status = 'active'\nORDER BY u.created_at DESC\nLIMIT 25;";

            CodeBlock::new(sample_sql, theme)
                .language("sql")
                .show_line_numbers(true)
                .show(ui);

            ui.add_space(SPACE_XL);

            // Row 2: DiffViewer & Schema Tree
            ui.columns(2, |cols| {
                // Col 1: DiffViewer
                let ui = &mut cols[0];
                ui.label(RichText::new("Schema & SQL Diff Proposal").size(13.0).strong().color(theme.text_secondary));
                ui.add_space(SPACE_SM);

                let diff_lines = [
                    DiffLine::context(10, 10, "-- Schema migration for public.users"),
                    DiffLine::context(11, 11, "ALTER TABLE users ADD COLUMN is_verified BOOLEAN DEFAULT false;"),
                    DiffLine::removed(12, "CREATE INDEX idx_users_temp ON users(email);"),
                    DiffLine::added(12, "CREATE INDEX CONCURRENTLY idx_users_email_verified"),
                    DiffLine::added(13, "ON users (email, is_verified) WHERE is_verified = true;"),
                    DiffLine::context(13, 14, "COMMENT ON COLUMN users.is_verified IS 'Email verified status';"),
                ];

                DiffViewer::new("migration_0042_user_verification.sql", &diff_lines, theme).show(ui);

                // Col 2: Schema Tree
                let ui = &mut cols[1];
                ui.label(RichText::new("Hierarchical Database Tree").size(13.0).strong().color(theme.text_secondary));
                ui.add_space(SPACE_SM);

                let tree_frame = egui::Frame::none()
                    .fill(theme.surface_editor)
                    .stroke(egui::Stroke::new(STROKE_THIN, theme.border_default))
                    .rounding(egui::Rounding::same(RADIUS_CARD))
                    .inner_margin(egui::Margin::same(SPACE_SM));

                tree_frame.show(ui, |ui| {
                    DatabaseTreeNode::new("localhost:5432", TreeNodeKind::Server, 0, theme)
                        .detail("PostgreSQL 16.2")
                        .expanded(&mut self.gallery_state.tree_server_expanded)
                        .show(ui);

                    reveal_children(ui, ui.id().with("tree_server"), self.gallery_state.tree_server_expanded, |ui| {
                        DatabaseTreeNode::new("production_db", TreeNodeKind::Database, 1, theme)
                            .detail("UTF-8")
                            .expanded(&mut self.gallery_state.tree_db_expanded)
                            .show(ui);

                        reveal_children(ui, ui.id().with("tree_db"), self.gallery_state.tree_db_expanded, |ui| {
                            DatabaseTreeNode::new("public", TreeNodeKind::Schema, 2, theme)
                                .detail("12 tables")
                                .expanded(&mut self.gallery_state.tree_schema_expanded)
                                .show(ui);

                            reveal_children(
                                ui,
                                ui.id().with("tree_schema"),
                                self.gallery_state.tree_schema_expanded,
                                |ui| {
                                    DatabaseTreeNode::new("users", TreeNodeKind::Table, 3, theme)
                                        .detail("1,842,109 rows")
                                        .selected(true)
                                        .expanded(&mut self.gallery_state.tree_table_expanded)
                                        .show(ui);

                                    reveal_children(
                                        ui,
                                        ui.id().with("tree_table"),
                                        self.gallery_state.tree_table_expanded,
                                        |ui| {
                                            DatabaseTreeNode::new("id", TreeNodeKind::PrimaryKey, 4, theme)
                                                .detail("bigint PK")
                                                .show(ui);
                                            DatabaseTreeNode::new("email", TreeNodeKind::Column, 4, theme)
                                                .detail("varchar(255)")
                                                .show(ui);
                                            DatabaseTreeNode::new("organization_id", TreeNodeKind::ForeignKey, 4, theme)
                                                .detail("bigint -> orgs.id")
                                                .show(ui);
                                            DatabaseTreeNode::new("idx_users_email", TreeNodeKind::Index, 4, theme)
                                                .detail("btree (email)")
                                                .show(ui);
                                        },
                                    );

                                    DatabaseTreeNode::new("orders", TreeNodeKind::Table, 3, theme)
                                        .detail("4,291,012 rows")
                                        .show(ui);

                                    DatabaseTreeNode::new("v_active_bookings", TreeNodeKind::View, 3, theme)
                                        .detail("view")
                                        .show(ui);
                                },
                            );
                        });
                    });
                });
            });

            ui.add_space(20.0);

            // Row 3: SQL Editor Toolbar & Action Bar
            ui.label(RichText::new("SQL Editor Toolbar & Diagnostics").size(13.0).strong().color(theme.text_secondary));
            ui.add_space(6.0);

            let sql_action = SqlEditorToolbar::new(false, true, theme).show(ui);
            match sql_action {
                Some(SqlEditorAction::RunQuery) => {
                    self.gallery_state.sql_editor_status = Some("Executing full SQL buffer...".to_owned());
                }
                Some(SqlEditorAction::RunSelection) => {
                    self.gallery_state.sql_editor_status = Some("Executing selected SQL range (Ln 5-8)...".to_owned());
                }
                Some(SqlEditorAction::ExplainQuery) => {
                    self.gallery_state.sql_editor_status = Some("Running EXPLAIN (ANALYZE, BUFFERS)...".to_owned());
                }
                Some(SqlEditorAction::FormatSql) => {
                    self.gallery_state.sql_editor_status = Some("SQL formatted with canonical keyword casing.".to_owned());
                }
                Some(SqlEditorAction::AskAi) => {
                    self.gallery_state.sql_editor_status = Some("Opening Agent Copilot with active SQL context...".to_owned());
                }
                _ => {}
            }

            if let Some(ref status) = self.gallery_state.sql_editor_status {
                ui.add_space(6.0);
                Alert::new("Editor Event", status, theme)
                    .variant(AlertVariant::Default)
                    .show(ui);
            }

            ui.add_space(20.0);

            // Row 4: Explain Plan Tree & Performance Bottlenecks
            ui.columns(2, |cols| {
                // Col 1: ExplainPlanTree
                let ui = &mut cols[0];
                let plan_root = PlanNode::new("Hash Join", 1420.5, 38.4, 18420)
                    .with_child(
                        PlanNode::new("Seq Scan", 940.0, 31.2, 142000)
                            .relation("users")
                            .bottleneck(true),
                    )
                    .with_child(
                        PlanNode::new("Index Scan", 48.0, 1.8, 18420)
                            .relation("organizations")
                            .bottleneck(false),
                    );

                ExplainPlanTree::new(&plan_root, 38.4, theme).show(ui);

                // Col 2: Execution Log Viewer
                let ui = &mut cols[1];
                ui.label(RichText::new("Query Execution Log").size(13.0).strong().color(theme.text_secondary));
                ui.add_space(6.0);

                let log_entries = [
                    LogEntry::new("12:04:01.102", LogLevel::Info, "Connected to PostgreSQL 16.2 on port 5432"),
                    LogEntry::new("12:04:01.350", LogLevel::Notice, "Temporary table pg_temp_04 created"),
                    LogEntry::new("12:04:02.810", LogLevel::Warning, "Query scan cost exceeds budget (1420 > 1000)"),
                    LogEntry::new("12:04:03.119", LogLevel::Info, "Query returned 25 rows in 38.4ms"),
                ];

                LogViewer::new(&log_entries, theme).show(ui);
            });

            ui.add_space(20.0);

            // Row 5: Terminal Output & Progress Ring
            ui.columns(2, |cols| {
                // Col 1: TerminalBlock
                let ui = &mut cols[0];
                ui.label(RichText::new("Terminal / CLI Output").size(13.0).strong().color(theme.text_secondary));
                ui.add_space(6.0);
                TerminalBlock::new("psql -h localhost -U postgres -d production_db\npsql (16.2)\nType \"help\" for help.\n\nproduction_db=# VACUUM ANALYZE users;\nVACUUM", theme)
                    .title("psql session: production_db")
                    .show(ui);

                // Col 2: ProgressRing
                let ui = &mut cols[1];
                ui.label(RichText::new("Progress Ring Indicator").size(13.0).strong().color(theme.text_secondary));
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    ProgressRing::new(0.75, 20.0, theme).show(ui);
                    ui.add_space(12.0);
                    ui.vertical(|ui| {
                        ui.label(RichText::new("75% Schema Indexing").size(12.5).strong().color(theme.text_primary));
                        ui.label(RichText::new("Building idx_users_email concurrently...").size(11.5).color(theme.text_secondary));
                    });
                });
            });
        });
    }

    pub(super) fn draw_gallery_database_shell_section(&mut self, ui: &mut Ui) {
        let theme = self.theme;
        self.draw_section_heading(
            ui,
            "Database Connection & Application Shell",
            "Connection cards, health monitors, activity bar, status bar, and driver badges.",
        );

        Card::new(theme).show(ui, |ui| {
            // Row 1: Connection Cards & Badges
            ui.label(
                RichText::new("Connection Cards & Database Driver Badges")
                    .size(13.0)
                    .strong()
                    .color(theme.text_secondary),
            );
            ui.add_space(6.0);

            ui.horizontal_wrapped(|ui| {
                DatabaseTypeBadge::new(DatabaseDriver::PostgreSql, theme).show(ui);
                ui.add_space(6.0);
                DatabaseTypeBadge::new(DatabaseDriver::Sqlite, theme).show(ui);
                ui.add_space(6.0);
                DatabaseTypeBadge::new(DatabaseDriver::MySql, theme).show(ui);
                ui.add_space(6.0);
                DatabaseTypeBadge::new(DatabaseDriver::SqlServer, theme).show(ui);
            });

            ui.add_space(14.0);

            ui.columns(2, |cols| {
                // Col 1: ConnectionCard
                let ui = &mut cols[0];
                let card_action = ConnectionCard::new(
                    "Production PostgreSQL Cluster",
                    DatabaseDriver::PostgreSql,
                    "db.internal.company.com:5432",
                    "production_app",
                    self.gallery_state.db_card_status,
                    theme,
                )
                .ssl(true)
                .show(ui);

                match card_action {
                    Some(ConnectionCardAction::Connect) => {
                        self.gallery_state.db_card_status = ConnectionStatus::Connected;
                        self.gallery_state.toasts.show(
                            "Connected to Production Cluster.",
                            ToastVariant::Success,
                            ToastPosition::BottomRight,
                        );
                    }
                    Some(ConnectionCardAction::Disconnect) => {
                        self.gallery_state.db_card_status = ConnectionStatus::Disconnected;
                        self.gallery_state.toasts.show(
                            "Disconnected from database.",
                            ToastVariant::Default,
                            ToastPosition::BottomRight,
                        );
                    }
                    Some(ConnectionCardAction::Edit) => {
                        self.gallery_state.dialog_open = true;
                    }
                    _ => {}
                }

                // Col 2: Connection Health Indicators
                let ui = &mut cols[1];
                ui.label(
                    RichText::new("Connection Health & Latency Monitors")
                        .size(12.5)
                        .strong()
                        .color(theme.text_secondary),
                );
                ui.add_space(6.0);

                ConnectionIndicator::new("Analytics (PostgreSQL)", "pg 16.2", ConnectionHealth::Healthy, theme)
                    .latency(42)
                    .show(ui);

                ui.add_space(8.0);

                ConnectionIndicator::new("Analytics SQLite", "sqlite 3.45", ConnectionHealth::Degraded, theme)
                    .latency(280)
                    .show(ui);

                ui.add_space(8.0);

                ConnectionIndicator::new(
                    "Legacy Reporting DB",
                    "mysql 8.0",
                    ConnectionHealth::Disconnected,
                    theme,
                )
                .show(ui);
            });

            ui.add_space(20.0);

            // Row 2: Status Bar Demo
            ui.label(
                RichText::new("Application Status Bar (Editor & Database Meta)")
                    .size(13.0)
                    .strong()
                    .color(theme.text_secondary),
            );
            ui.add_space(6.0);

            let left_status = [
                StatusBarItem::new("Analytics (PostgreSQL)")
                    .icon(Icon::Database)
                    .accent(true),
                StatusBarItem::new("public.users").icon(Icon::Table),
                StatusBarItem::new("UTF-8 · READ COMMITTED"),
            ];
            let right_status = [
                StatusBarItem::new("Ln 42, Col 18"),
                StatusBarItem::new("1,842,109 rows"),
                StatusBarItem::new("42ms latency").icon(Icon::Zap),
            ];

            StatusBar::new(&left_status, &right_status, theme).show(ui);
        });
    }
}
