pub mod agent_composer;
pub mod agent_primitives;
pub mod alert;
pub mod animation;
pub mod badge;
pub mod button;
pub mod card;
pub mod chrome;
pub mod code;
pub mod database;
pub mod dev_tools;
pub mod dialog;
pub mod diff;
pub mod explain;
pub mod feedback;
pub mod form;
pub mod input;
pub mod interact;
pub mod legacy;
pub mod logs;
pub mod nav;
pub mod overlay;
pub mod select;
pub mod selection;
pub mod sql_editor;
pub mod table;
pub mod tabs;
pub mod transaction;
pub mod tree;
pub mod workspace;

pub use agent_composer::{AgentComposer, AgentComposerAction, AgentMode};
pub use agent_primitives::{
    AgentPlan, AgentSqlActionKind, AgentTaskItem, AgentTaskStatus, AgentThinking, ContextChip, ContextChipKind,
    ExecutionApproval, ExecutionApprovalAction, RiskLevel, StatusBadge, StatusBadgeVariant, ToolCall, ToolCallStatus,
};
pub use alert::{Alert, AlertVariant};
pub use badge::{Badge, BadgeVariant};
pub use button::{Button, ButtonSize, ButtonVariant};
pub use card::{card_header, Card, MetricCard};
pub use chrome::{toolbar_button, Avatar, AvatarSize, EmptyState, Skeleton, Toolbar};
pub use code::{CodeBlock, InlineCode};
pub use database::{ConnectionCard, ConnectionCardAction, ConnectionStatus, DatabaseDriver, DatabaseTypeBadge};
pub use dev_tools::{ProgressRing, TerminalBlock};
pub use dialog::{dialog_actions, Dialog, Sheet};
pub use diff::{DiffLine, DiffLineType, DiffViewer};
pub use explain::{ExplainPlanTree, PlanNode};
pub use feedback::{kbd_badge, kbd_combo, separator_with_text, Progress, Spinner};
pub use form::{FieldRule, FormField, FormState, Label, ValidationMode};
pub use input::{Input, PasswordInput, SearchInput, Textarea};
pub use legacy::*;
pub use logs::{LogEntry, LogLevel, LogViewer};
pub use nav::{Breadcrumb, BreadcrumbItem, PageHeader, Pagination, SectionHeader};
pub use overlay::{
    context_action_menu, ctx_menu_item, is_context_menu_triggered, DropdownItem, DropdownMenu, Popover, Toast,
    ToastItem, ToastManager, ToastPosition, ToastResponse, ToastVariant, Tooltip, TooltipPosition,
};
pub use select::{dropdown_should_open_above, Select};
pub use selection::{Checkbox, Radio, Slider, Switch};
pub use sql_editor::{SqlEditorAction, SqlEditorToolbar};
pub use table::{Table, TableColumn, TableColumnAlign};
pub use tabs::{SegmentedTabs, UnderlineTabs};
pub use transaction::{DestructiveOperationDialog, TransactionAction, TransactionBar};
pub use tree::{reveal_children, DatabaseTreeNode, TreeNodeKind};
pub use workspace::{
    ActivityBar, ActivityBarItemKind, ConnectionHealth, ConnectionIndicator, StatusBar, StatusBarItem,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DbProTheme;

    #[test]
    fn test_button_builder_variants_and_sizes() {
        let theme = DbProTheme::light();
        let btn = Button::new(theme)
            .text("Click me")
            .variant(ButtonVariant::Destructive)
            .size(ButtonSize::Lg)
            .enabled(false)
            .loading(true)
            .full_width(true);

        assert_eq!(btn.variant, ButtonVariant::Destructive);
        assert_eq!(btn.size, ButtonSize::Lg);
        assert!(!btn.enabled);
        assert!(btn.loading);
        assert!(btn.full_width);
    }

    fn tab_press_event() -> egui::Event {
        egui::Event::Key {
            key: egui::Key::Tab,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::default(),
        }
    }

    #[test]
    fn enabled_button_is_tab_focusable_and_disabled_is_not() {
        let theme = DbProTheme::light();
        run_ui(|ui| {
            let enabled = Button::new(theme).text("Save").show(ui);
            assert!(enabled.sense.click);
            assert!(enabled.sense.focusable);
            let disabled = Button::new(theme).text("Locked").enabled(false).show(ui);
            assert!(!disabled.sense.click);
            assert!(!disabled.sense.focusable);
            let icon = Button::new(theme)
                .icon(lucide_icons::Icon::Copy)
                .access_label("Copy")
                .size(ButtonSize::Icon)
                .show(ui);
            assert!(icon.sense.focusable);
        });
    }

    #[test]
    fn tab_moves_focus_onto_the_first_button() {
        let theme = DbProTheme::light();
        let ctx = egui::Context::default();
        DbProTheme::install_fonts(&ctx);
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                Button::new(theme).text("First").show(ui);
                Button::new(theme).text("Second").show(ui);
            });
        });

        let input = egui::RawInput {
            events: vec![tab_press_event()],
            ..Default::default()
        };
        let mut first_focused = false;
        let mut second_focused = false;
        let _ = ctx.run(input, |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                first_focused = Button::new(theme).text("First").show(ui).has_focus();
                second_focused = Button::new(theme).text("Second").show(ui).has_focus();
            });
        });
        assert!(first_focused, "Tab should land on the first focusable button");
        assert!(!second_focused);
    }

    #[test]
    fn space_activates_a_focused_button() {
        let theme = DbProTheme::light();
        let ctx = egui::Context::default();
        DbProTheme::install_fonts(&ctx);
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                Button::new(theme).text("Run").show(ui);
            });
        });
        let tab = egui::RawInput {
            events: vec![tab_press_event()],
            ..Default::default()
        };
        let _ = ctx.run(tab, |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                Button::new(theme).text("Run").show(ui);
            });
        });
        let space = egui::RawInput {
            events: vec![egui::Event::Key {
                key: egui::Key::Space,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::default(),
            }],
            ..Default::default()
        };
        let mut clicked = false;
        let mut focused = false;
        let _ = ctx.run(space, |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let response = Button::new(theme).text("Run").show(ui);
                focused = response.has_focus();
                clicked = response.clicked();
            });
        });
        assert!(focused, "button should keep focus after Tab");
        assert!(clicked, "Space on a focused button should activate it");
    }

    #[test]
    fn test_badge_variants() {
        let theme = DbProTheme::light();
        let badge = Badge::new("Active", theme).variant(BadgeVariant::Success).dot(true);

        assert_eq!(badge.variant, BadgeVariant::Success);
        assert!(badge.dot);
    }

    #[test]
    fn select_with_many_options_shows_without_panic() {
        let theme = DbProTheme::light();
        let options: Vec<String> = (0..24).map(|i| format!("option-{i}")).collect();
        let mut selected = 3;
        let mut load_more = false;
        run_ui(|ui| {
            Select::new("many-options", &mut selected, &options[..12], theme)
                .has_more(true)
                .load_more(&mut load_more)
                .show(ui);
        });
        assert_eq!(selected, 3);
        assert!(!load_more);
    }

    #[test]
    fn test_alert_variants() {
        let theme = DbProTheme::light();
        let alert = Alert::new("Title", "Description", theme)
            .variant(AlertVariant::Warning)
            .dismissable(true);

        assert_eq!(alert.variant, AlertVariant::Warning);
        assert!(alert.dismissable);
    }

    #[test]
    fn test_progress_fraction_clamping() {
        let theme = DbProTheme::light();
        let p1 = Progress::new(1.5, theme);
        assert_eq!(p1.fraction, 1.0);

        let p2 = Progress::new(-0.2, theme);
        assert_eq!(p2.fraction, 0.0);
    }

    #[test]
    fn test_progress_indeterminate_is_distinct_constructor() {
        let theme = DbProTheme::light();
        let determinate = Progress::new(0.4, theme);
        let indeterminate = Progress::indeterminate(theme);
        assert!(!determinate.indeterminate);
        assert_eq!(determinate.fraction, 0.4);
        assert!(indeterminate.indeterminate);
        assert_eq!(indeterminate.fraction, 0.0);
    }

    fn run_ui(mut on_ui: impl FnMut(&mut egui::Ui)) {
        let ctx = egui::Context::default();
        DbProTheme::install_fonts(&ctx);
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| on_ui(ui));
        });
    }

    #[test]
    fn foundation_widgets_show_inside_egui_context() {
        let theme = DbProTheme::light();
        run_ui(|ui| {
            let label = Label::new("Host", theme).required(true).show(ui);
            assert!(label.rect.is_finite());

            let mut host = String::from("localhost");
            let field = FormField::new("Host", &mut host, "hostname", theme)
                .helper_text("Used for the connection")
                .show(ui);
            assert!(field.rect.is_finite());

            let avatar = Avatar::new(theme).initials("TD").size(AvatarSize::Md).show(ui);
            assert!(avatar.rect.is_finite());

            let trigger = Button::new(theme).text("Hint").show(ui);
            let tip = Tooltip::new("More detail", theme).show(&trigger);
            assert!(tip.rect.is_finite());

            let mut popover_open = true;
            let popover_trigger = Button::new(theme).text("Popover").show(ui);
            let popover = Popover::new(&mut popover_open, theme).show(ui, &popover_trigger, |ui| {
                ui.label("Pinned context");
                ui.id()
            });
            assert!(popover.is_some());

            let mut menu_open = true;
            let items = [
                DropdownItem::new("Run").icon(lucide_icons::Icon::Play).shortcut("⌘↵"),
                DropdownItem::new("Delete").danger(true).enabled(false),
            ];
            let menu_trigger = Button::new(theme).text("Menu").show(ui);
            let _clicked = DropdownMenu::new(&mut menu_open, &items, theme).show(ui, &menu_trigger);

            let mut dialog_open = true;
            let dialog = Dialog::new(&mut dialog_open, "Confirm", theme)
                .description("This will close the connection.")
                .show(ui, |ui| ui.label("Dialog body"));
            assert!(dialog.is_some());
            assert!(dialog.unwrap().rect.is_finite());

            let mut sheet_open = true;
            let sheet = Sheet::new(&mut sheet_open, "Inspector", theme).show(ui, |ui| ui.label("Sheet body"));
            assert!(sheet.is_some());
            assert!(sheet.unwrap().rect.is_finite());

            let skeleton = Skeleton::new(theme).size(120.0, 12.0).show(ui);
            assert!(skeleton.rect.is_finite());

            let toast = Toast::new("Changes saved", theme)
                .variant(ToastVariant::Success)
                .show(ui);
            assert!(!toast.action_clicked);
            assert!(!toast.dismiss_clicked);
            let toast_action = Toast::new("Failed to save", theme)
                .variant(ToastVariant::Danger)
                .action("Retry")
                .show(ui);
            assert!(!toast_action.action_clicked);

            let empty = EmptyState::new(
                lucide_icons::Icon::FolderOpen,
                "No projects yet",
                "Create your first project to get started.",
                theme,
            )
            .action("Create project")
            .show(ui);
            assert!(empty.is_some());
            assert!(empty.unwrap().rect.is_finite());

            let mut page = 2;
            let pagination = Pagination::new(&mut page, 5, theme).show(ui);
            assert!(pagination.rect.is_finite());

            let crumbs = [
                BreadcrumbItem::new("Workspace"),
                BreadcrumbItem::new("public").current(true),
            ];
            let _crumb = Breadcrumb::new(&crumbs, theme).show(ui);

            let header = PageHeader::new("Connections", theme)
                .description("Manage database endpoints.")
                .show(ui);
            assert!(header.rect.is_finite());

            let section = SectionHeader::new("SSL", theme)
                .description("Certificate and mode")
                .show(ui);
            assert!(section.rect.is_finite());

            Toolbar::new(theme).show(ui, |ui| {
                toolbar_button(ui, "Run", lucide_icons::Icon::Play, theme);
            });

            Button::new(theme).text("Loading").loading(true).show(ui);
            Progress::new(0.5, theme).show(ui);
            Progress::indeterminate(theme).show(ui);
            Spinner::new(theme).show(ui);
        });
    }

    fn run_ui_frames(frames: usize, mut on_ui: impl FnMut(&mut egui::Ui)) {
        let ctx = egui::Context::default();
        DbProTheme::install_fonts(&ctx);
        for frame in 0..frames {
            let input = egui::RawInput {
                predicted_dt: 1.0 / 60.0,
                time: Some(f64::from(frame as u32) / 60.0),
                ..Default::default()
            };
            let _ = ctx.run(input, |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| on_ui(ui));
            });
        }
    }

    #[test]
    fn sibling_progress_bars_lerp_independently() {
        let theme = DbProTheme::light();
        let mut left = 0.0;
        let mut right = 0.0;
        run_ui_frames(12, |ui| {
            left = Progress::new(0.3, theme).show(ui);
            right = Progress::new(0.8, theme).show(ui);
        });
        assert!((left - 0.3).abs() < 0.05, "left fill {left} should stay near 0.3");
        assert!((right - 0.8).abs() < 0.05, "right fill {right} should stay near 0.8");
        assert!(
            (right - left).abs() > 0.2,
            "sibling bars must keep distinct fills (left={left}, right={right})"
        );
    }

    #[test]
    fn closed_dialog_does_not_shut_open_sibling() {
        let theme = DbProTheme::light();
        let mut open = true;
        let mut closed = false;
        let mut open_shown = false;
        let mut closed_shown = false;
        run_ui_frames(8, |ui| {
            open_shown = Dialog::new(&mut open, "Keep open", theme)
                .id_salt("dialog-open")
                .show(ui, |ui| ui.label("open body"))
                .is_some();
            closed_shown = Dialog::new(&mut closed, "Stay closed", theme)
                .id_salt("dialog-closed")
                .show(ui, |ui| ui.label("closed body"))
                .is_some();
        });
        assert!(open, "closed sibling must not flip the open dialog shut");
        assert!(!closed);
        assert!(open_shown);
        assert!(!closed_shown);
    }

    #[test]
    fn unsalted_closed_dialog_does_not_shut_open_sibling() {
        let theme = DbProTheme::light();
        let mut open = true;
        let mut closed = false;
        let mut open_shown = false;
        let mut closed_shown = false;
        run_ui_frames(8, |ui| {
            open_shown = Dialog::new(&mut open, "Keep open", theme)
                .show(ui, |ui| ui.label("open body"))
                .is_some();
            closed_shown = Dialog::new(&mut closed, "Stay closed", theme)
                .show(ui, |ui| ui.label("closed body"))
                .is_some();
        });
        assert!(open, "unsalted closed sibling must not flip the open dialog shut");
        assert!(!closed);
        assert!(open_shown);
        assert!(!closed_shown);
    }

    #[test]
    fn unsalted_closed_sheet_does_not_shut_open_sibling() {
        let theme = DbProTheme::light();
        let mut open = true;
        let mut closed = false;
        let mut open_shown = false;
        let mut closed_shown = false;
        run_ui_frames(8, |ui| {
            open_shown = Sheet::new(&mut open, "Keep open", theme)
                .show(ui, |ui| ui.label("open sheet"))
                .is_some();
            closed_shown = Sheet::new(&mut closed, "Stay closed", theme)
                .show(ui, |ui| ui.label("closed sheet"))
                .is_some();
        });
        assert!(open, "unsalted closed sibling must not flip the open sheet shut");
        assert!(!closed);
        assert!(open_shown);
        assert!(!closed_shown);
    }

    #[test]
    fn closed_sheet_does_not_shut_open_sibling() {
        let theme = DbProTheme::light();
        let mut open = true;
        let mut closed = false;
        let mut open_shown = false;
        let mut closed_shown = false;
        run_ui_frames(8, |ui| {
            open_shown = Sheet::new(&mut open, "Keep open", theme)
                .id_salt("sheet-open")
                .show(ui, |ui| ui.label("open sheet"))
                .is_some();
            closed_shown = Sheet::new(&mut closed, "Stay closed", theme)
                .id_salt("sheet-closed")
                .show(ui, |ui| ui.label("closed sheet"))
                .is_some();
        });
        assert!(open, "closed sibling must not flip the open sheet shut");
        assert!(!closed);
        assert!(open_shown);
        assert!(!closed_shown);
    }

    #[test]
    fn test_table_column_builder() {
        let col = TableColumn::new("Status")
            .width(120.0)
            .align(TableColumnAlign::Center)
            .sortable(true);

        assert_eq!(col.title, "Status");
        assert_eq!(col.width, Some(120.0));
        assert_eq!(col.align, TableColumnAlign::Center);
        assert!(col.sortable);
    }

    #[test]
    fn test_checkbox_builder() {
        let theme = DbProTheme::light();
        let mut checked = false;
        let cb = Checkbox::new(&mut checked, "Enable SSL", theme)
            .description("Requires valid server certificate")
            .enabled(false);

        assert_eq!(cb.label, "Enable SSL");
        assert_eq!(cb.description, Some("Requires valid server certificate"));
        assert!(!cb.enabled);
    }

    #[test]
    fn test_slider_builder() {
        let theme = DbProTheme::light();
        let mut val = 50.0;
        let slider = Slider::new(&mut val, 0.0..=100.0, theme)
            .label("Volume")
            .show_value(true)
            .width(200.0);

        assert_eq!(slider.label, Some("Volume"));
        assert!(slider.show_value);
        assert_eq!(slider.width, Some(200.0));
    }

    #[test]
    fn test_diff_line_builders() {
        let ctx = DiffLine::context(10, 10, "SELECT 1;");
        assert_eq!(ctx.line_type, DiffLineType::Context);
        assert_eq!(ctx.old_line_num, Some(10));
        assert_eq!(ctx.new_line_num, Some(10));

        let add = DiffLine::added(11, "+ ADDED COLUMN");
        assert_eq!(add.line_type, DiffLineType::Added);
        assert_eq!(add.old_line_num, None);
        assert_eq!(add.new_line_num, Some(11));

        let rem = DiffLine::removed(12, "- REMOVED COLUMN");
        assert_eq!(rem.line_type, DiffLineType::Removed);
        assert_eq!(rem.old_line_num, Some(12));
        assert_eq!(rem.new_line_num, None);
    }

    #[test]
    fn test_context_chip_kind_icons() {
        assert_eq!(
            ContextChipKind::Connection.icon().to_string(),
            lucide_icons::Icon::Server.to_string()
        );
        assert_eq!(
            ContextChipKind::Database.icon().to_string(),
            lucide_icons::Icon::Database.to_string()
        );
        assert_eq!(
            ContextChipKind::Table.icon().to_string(),
            lucide_icons::Icon::Table.to_string()
        );
    }

    #[test]
    fn test_agent_thinking_and_plan_rendering() {
        let theme = DbProTheme::light();
        run_ui(|ui| {
            let mut expanded = true;
            let resp = AgentThinking::new("Analyzing schema...", &mut expanded, theme)
                .duration("1.2s")
                .step_count(3)
                .show(ui);
            assert!(resp.rect.width() > 0.0);

            let tasks = vec![
                AgentTaskItem::new("Inspect table", AgentTaskStatus::Completed).with_detail("10ms"),
                AgentTaskItem::new("Generate index SQL", AgentTaskStatus::Running),
                AgentTaskItem::new("Review safety", AgentTaskStatus::Pending),
            ];
            let plan_resp = AgentPlan::new("Plan", &tasks, theme).show(ui);
            assert!(plan_resp.rect.width() > 0.0);
        });
    }

    #[test]
    fn test_agent_sql_action_kind_attributes() {
        assert_eq!(AgentSqlActionKind::Explain.label(), "Explain Query");
        assert_eq!(AgentSqlActionKind::Optimize.label(), "Optimize Query");
        assert_eq!(AgentSqlActionKind::FixError.label(), "Fix Error");
        assert_eq!(AgentSqlActionKind::GenerateMigration.label(), "Generate Migration");
    }

    #[test]
    fn test_transaction_bar_rendering() {
        let theme = DbProTheme::light();
        run_ui(|ui| {
            let bar = TransactionBar::new(true, 2, theme)
                .auto_commit(false)
                .isolation_level("SERIALIZABLE");
            let _ = bar.show(ui);
        });
    }

    #[test]
    fn test_workspace_status_bar_and_activity_bar() {
        let theme = DbProTheme::light();
        run_ui(|ui| {
            let left = [StatusBarItem::new("Connected")
                .icon(lucide_icons::Icon::Database)
                .accent(true)];
            let right = [StatusBarItem::new("Ln 1, Col 1")];
            let resp = StatusBar::new(&left, &right, theme).show(ui);
            assert!(resp.rect.height() > 0.0);

            let act_resp = ActivityBar::new(ActivityBarItemKind::Explorer, theme).show(ui);
            assert_eq!(act_resp, None);

            let ind_resp = ConnectionIndicator::new("Prod DB", "PostgreSQL", ConnectionHealth::Healthy, theme)
                .latency(24)
                .show(ui);
            assert!(ind_resp.rect.width() > 0.0);
        });
    }

    #[test]
    fn test_database_cards_and_badges() {
        let theme = DbProTheme::light();
        run_ui(|ui| {
            let badge_resp = DatabaseTypeBadge::new(DatabaseDriver::PostgreSql, theme).show(ui);
            assert!(badge_resp.rect.width() > 0.0);

            let card = ConnectionCard::new(
                "Test DB",
                DatabaseDriver::Sqlite,
                "local.db",
                "main",
                ConnectionStatus::Connected,
                theme,
            )
            .ssl(false);
            let action = card.show(ui);
            assert_eq!(action, None);
        });
    }

    #[test]
    fn test_sql_editor_toolbar_and_explain_tree() {
        let theme = DbProTheme::light();
        run_ui(|ui| {
            let tb_action = SqlEditorToolbar::new(false, true, theme).show(ui);
            assert_eq!(tb_action, None);

            let root = PlanNode::new("Hash Join", 100.0, 12.0, 500)
                .relation("users")
                .bottleneck(true);
            let tree_resp = ExplainPlanTree::new(&root, 12.0, theme).show(ui);
            assert!(tree_resp.rect.width() > 0.0);
        });
    }

    #[test]
    fn test_logs_composer_and_devtools() {
        let theme = DbProTheme::light();
        run_ui(|ui| {
            let entries = [LogEntry::new("12:00:00", LogLevel::Info, "Ready")];
            let log_resp = LogViewer::new(&entries, theme).show(ui);
            assert!(log_resp.rect.width() > 0.0);

            let mut prompt = "SELECT 1;".to_owned();
            let comp_action = AgentComposer::new(&mut prompt, "GPT-4o", AgentMode::Code, theme)
                .token_usage(100)
                .show(ui);
            assert_eq!(comp_action, None);

            let term_resp = TerminalBlock::new("output log", theme).show(ui);
            assert!(term_resp.rect.width() > 0.0);

            let ring_resp = ProgressRing::new(0.5, 16.0, theme).show(ui);
            assert!(ring_resp.rect.width() > 0.0);
        });
    }
}
