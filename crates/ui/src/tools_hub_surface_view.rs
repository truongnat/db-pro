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

pub(super) struct ToolsHubContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) active_activity: Activity,
    pub(super) search_query: &'a mut String,
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

        // Section 1: Schema & Architecture
        self.draw_category_header(ui, "Schema & Migration", Icon::GitCompare);
        ui.add_space(SPACE_XXS);
        self.draw_tool_card(
            ui,
            &mut actions,
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
            &mut actions,
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
            &mut actions,
            ToolCardItem {
                action: ToolsHubAction::Diagram,
                icon: Icon::Workflow,
                title: "ER Diagram",
                description: "Interactive schema canvas, foreign keys and neighborhood map",
                is_active: self.active_activity == Activity::Diagram,
            },
        );

        ui.add_space(SPACE_MD);

        // Section 2: Administration & Monitoring
        self.draw_category_header(ui, "Administration & Server", Icon::Gauge);
        ui.add_space(SPACE_XXS);
        self.draw_tool_card(
            ui,
            &mut actions,
            ToolCardItem {
                action: ToolsHubAction::Monitor,
                icon: Icon::Gauge,
                title: "Server Monitor",
                description: "Active sessions, query locks, storage and server health",
                is_active: self.active_activity == Activity::Monitor,
            },
        );
        self.draw_tool_card(
            ui,
            &mut actions,
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
            &mut actions,
            ToolCardItem {
                action: ToolsHubAction::History,
                icon: Icon::History,
                title: "Execution History & Audit",
                description: "Past query runs, execution timing and server CSV logs",
                is_active: self.active_activity == Activity::History,
            },
        );

        ui.add_space(SPACE_MD);

        // Section 3: Data & Automation
        self.draw_category_header(ui, "Data & Automation", Icon::ArrowRightLeft);
        ui.add_space(SPACE_XXS);
        self.draw_tool_card(
            ui,
            &mut actions,
            ToolCardItem {
                action: ToolsHubAction::Transfers,
                icon: Icon::Upload,
                title: "Data Transfer & Migration",
                description: "Cross-connection streaming, batch insert and export",
                is_active: self.active_activity == Activity::Transfers,
            },
        );
        self.draw_tool_card(
            ui,
            &mut actions,
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
            &mut actions,
            ToolCardItem {
                action: ToolsHubAction::Problems,
                icon: Icon::TriangleAlert,
                title: "Diagnostics & Problems",
                description: "Syntax linter, connection errors and schema warnings",
                is_active: self.active_activity == Activity::Problems,
            },
        );

        ui.add_space(SPACE_LG);
        actions
    }

    fn draw_category_header(&self, ui: &mut egui::Ui, title: &str, icon: Icon) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = SPACE_XXS;
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

    fn draw_tool_card(
        &self,
        ui: &mut egui::Ui,
        actions: &mut Vec<ToolsHubAction>,
        item: ToolCardItem<'_>,
    ) {
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
        let card_frame = egui::Frame {
            fill: if is_active { theme.surface_active } else { theme.surface_panel },
            inner_margin: egui::Margin::symmetric(SPACE_SM as i8, SPACE_XS as i8),
            stroke: egui::Stroke::new(
                STROKE_THIN,
                if is_active { theme.accent } else { theme.border_subtle },
            ),
            corner_radius: egui::CornerRadius::same(RADIUS_SM as u8),
            ..Default::default()
        };

        let response = card_frame.show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(char::from(icon).to_string())
                        .font(font_icon(ICON_DEFAULT))
                        .color(if is_active { theme.accent } else { theme.text_primary }),
                );
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 1.0;
                    ui.label(
                        RichText::new(title)
                            .font(font_ui_label())
                            .strong()
                            .color(if is_active { theme.accent } else { theme.text_primary }),
                    );
                    ui.label(
                        RichText::new(description)
                            .font(font_caption())
                            .color(theme.text_muted),
                    );
                });
            });
        }).response;

        let response = ui.interact(response.rect, response.id, egui::Sense::click());
        if response.clicked() {
            actions.push(action);
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
