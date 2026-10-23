//! Tools & Administration Hub drawer view.
use super::*;
use egui::RichText;
use lucide_icons::Icon;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ToolsHubAction {
    SchemaCompare,
    SchemaWorkbench,
    Diagram,
    Monitor,
    Security,
    Transfers,
    Tasks,
    History,
    Problems,
}

struct ToolCardItem<'a> {
    action: ToolsHubAction,
    icon: Icon,
    title: &'a str,
    description: &'a str,
    is_active: bool,
}

pub(super) fn monitor_card_description(driver: &str) -> &'static str {
    match driver {
        "MySQL" => "MySQL has no sessions",
        "SQL Server" => "SQL Server has no sessions",
        _ => "Active sessions, query locks, storage and server health",
    }
}

pub(super) struct ToolsHubContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) active_activity: Activity,
    pub(super) search_query: &'a mut String,
    pub(super) driver: &'a str,
}

impl ToolsHubContext<'_> {
    pub(super) fn draw(&mut self, ui: &mut egui::Ui) -> Vec<ToolsHubAction> {
        let mut actions = Vec::new();
        ui.add_space(SPACE_SM);
        ui.label(
            RichText::new("TOOLS & MANAGEMENT")
                .font(font_caption())
                .strong()
                .color(self.theme.text_secondary),
        );
        ui.add_space(SPACE_XS);
        ui.label(
            RichText::new("Central hub for database migration, server administration, monitoring, and security.")
                .font(font_caption())
                .color(self.theme.text_muted),
        );
        crate::components::SearchInput::new(self.search_query, "Search tools & administration…", self.theme).show(ui);
        ui.add_space(SPACE_SM);

        self.draw_schema_section(ui, &mut actions);
        ui.add_space(SPACE_MD);
        self.draw_admin_section(ui, &mut actions);
        ui.add_space(SPACE_MD);
        self.draw_data_section(ui, &mut actions);
        ui.add_space(SPACE_LG);
        actions
    }

    fn draw_schema_section(&self, ui: &mut egui::Ui, actions: &mut Vec<ToolsHubAction>) {
        self.draw_category_header(ui, "Schema & Migration", Icon::GitCompare);
        ui.add_space(SPACE_XXS);
        self.draw_tool_card(
            ui,
            actions,
            ToolCardItem {
                action: ToolsHubAction::SchemaCompare,
                icon: Icon::GitCompare,
                title: "Schema & Data Compare",
                description: "Diff schemas, compare live rows, generate sync DDL",
                is_active: self.active_activity == Activity::Compare,
            },
        );
        self.draw_tool_card(
            ui,
            actions,
            ToolCardItem {
                action: ToolsHubAction::SchemaWorkbench,
                icon: Icon::Boxes,
                title: "Schema Workbench",
                description: "Design tables, columns, constraints and preview migrations",
                is_active: self.active_activity == Activity::Schema,
            },
        );
        self.draw_tool_card(
            ui,
            actions,
            ToolCardItem {
                action: ToolsHubAction::Diagram,
                icon: Icon::Workflow,
                title: "ER Diagram",
                description: "Interactive schema canvas, foreign keys and neighborhood map",
                is_active: self.active_activity == Activity::Diagram,
            },
        );
    }

    fn draw_admin_section(&self, ui: &mut egui::Ui, actions: &mut Vec<ToolsHubAction>) {
        self.draw_category_header(ui, "Administration & Server", Icon::Gauge);
        ui.add_space(SPACE_XXS);
        self.draw_tool_card(
            ui,
            actions,
            ToolCardItem {
                action: ToolsHubAction::Monitor,
                icon: Icon::Gauge,
                title: "Server Monitor",
                description: monitor_card_description(self.driver),
                is_active: self.active_activity == Activity::Monitor,
            },
        );
        self.draw_tool_card(
            ui,
            actions,
            ToolCardItem {
                action: ToolsHubAction::Security,
                icon: Icon::Shield,
                title: "Security & Roles",
                description: "Database users, role attributes, privileges and RLS policies",
                is_active: self.active_activity == Activity::Security,
            },
        );
        self.draw_tool_card(
            ui,
            actions,
            ToolCardItem {
                action: ToolsHubAction::History,
                icon: Icon::History,
                title: "Execution History & Audit",
                description: "Past query runs, execution timing and server CSV logs",
                is_active: self.active_activity == Activity::History,
            },
        );
    }

    fn draw_data_section(&self, ui: &mut egui::Ui, actions: &mut Vec<ToolsHubAction>) {
        self.draw_category_header(ui, "Data & Automation", Icon::ArrowRightLeft);
        ui.add_space(SPACE_XXS);
        self.draw_tool_card(
            ui,
            actions,
            ToolCardItem {
                action: ToolsHubAction::Transfers,
                icon: Icon::Upload,
                title: "Data Transfer & Migration",
                description: "Copy chosen tables between connections as CSV or JSON",
                is_active: self.active_activity == Activity::Transfers,
            },
        );
        self.draw_tool_card(
            ui,
            actions,
            ToolCardItem {
                action: ToolsHubAction::Tasks,
                icon: Icon::ListTodo,
                title: "Saved Tasks & Backups",
                description: "Automated SQL jobs, dump pipelines and maintenance tasks",
                is_active: self.active_activity == Activity::Tasks,
            },
        );
        self.draw_tool_card(
            ui,
            actions,
            ToolCardItem {
                action: ToolsHubAction::Problems,
                icon: Icon::TriangleAlert,
                title: "Diagnostics & Problems",
                description: "Syntax linter, connection errors and schema warnings",
                is_active: self.active_activity == Activity::Problems,
            },
        );
    }

    fn draw_category_header(&self, ui: &mut egui::Ui, title: &str, icon: Icon) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = crate::tokens::ICON_TEXT_GAP;
            ui.label(
                RichText::new(char::from(icon).to_string())
                    .font(font_icon(ICON_XS))
                    .color(self.theme.text_secondary),
            );
            ui.label(
                RichText::new(title)
                    .font(font_caption())
                    .strong()
                    .color(self.theme.text_secondary),
            );
        });
    }

    fn draw_tool_card(&self, ui: &mut egui::Ui, actions: &mut Vec<ToolsHubAction>, item: ToolCardItem<'_>) {
        let ToolCardItem {
            action,
            icon,
            title,
            description,
            is_active,
        } = item;
        let filter = self.search_query.trim().to_lowercase();
        if !filter.is_empty()
            && !title.to_lowercase().contains(&filter)
            && !description.to_lowercase().contains(&filter)
        {
            return;
        }
        let theme = self.theme;
        let card_frame = selectable_frame(theme, is_active);

        let response = card_frame
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                ui.horizontal(|ui| {
                    // Card rows are tighter than the shell rhythm: the title and the
                    // description belong to one text block, not two stacked controls.
                    ui.spacing_mut().item_spacing = egui::vec2(crate::tokens::SPACE_SM, crate::tokens::SPACE_XXS);
                    ui.label(
                        RichText::new(char::from(icon).to_string())
                            .font(font_icon(ICON_DEFAULT))
                            .color(if is_active { theme.accent } else { theme.text_primary }),
                    );
                    ui.vertical(|ui| {
                        ui.label(RichText::new(title).font(font_ui_label()).strong().color(if is_active {
                            theme.accent
                        } else {
                            theme.text_primary
                        }));
                        ui.label(RichText::new(description).font(font_caption()).color(theme.text_muted));
                    });
                });
            })
            .response;

        let response = ui.interact(response.rect, response.id, egui::Sense::click());
        // The card is painted by a custom frame — egui can't infer the role, so
        // declare it as a named button for accessibility tools.
        response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, ui.is_enabled(), title));
        // The whole card is one click target.
        let response = if ui.is_enabled() {
            response.on_hover_cursor(egui::CursorIcon::PointingHand)
        } else {
            response
        };
        if response.clicked() {
            actions.push(action);
        }
        if std::env::var("DB_PRO_CAPTURE_FOCUS_TOOL_CARD").ok().as_deref() == Some(title) {
            response.request_focus();
        }
        if response.has_focus() {
            crate::components::interact::paint_focus_ring(ui, response.rect, RADIUS_SM, theme);
        }
        if response.hovered() && !is_active {
            ui.painter().rect_stroke(
                response.rect,
                egui::CornerRadius::same(RADIUS_SM as u8),
                egui::Stroke::new(STROKE_THIN, theme.border_strong),
                egui::StrokeKind::Inside,
            );
        }
        ui.add_space(SPACE_XXS);
    }
}
