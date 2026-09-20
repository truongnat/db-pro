//! Primary left sidebar shell: exact-width panel, padded content, and resize grip.
use super::sidebar_chrome_view::{SidebarChromeAction, SidebarChromeContext};
use super::*;
use egui::{vec2, Pos2, Rect, Sense, Stroke};

/// Extra clip width granted around the sidebar content column, in pixels.
///
/// Layout still stays inside the column; this is only the room a border needs. epaint
/// strokes a rect *entirely outside* its path (`StrokeKind::Outside`), so a 1px border on a
/// widget that fills the column — the filter field, the primary action button, the
/// connection selector — is otherwise cut off flush with the column: it loses both vertical
/// edges while its horizontal ones survive, because those sit well inside the clip. One
/// pixel is the least that lets such a border render, and far too little to hide a real
/// layout overflow.
const SIDEBAR_CLIP_BLEED: f32 = 1.0;

impl DbProApp {
    pub(super) fn draw_sidebar(&mut self, ctx: &egui::Context) {
        // egui SidePanel advances CentralPanel from the *frame response* rect, not
        // `panel_rect`. Any child that expands `min_rect` past `exact_width` leaves
        // a dead gutter between our painted splitter and the workspace. Keep the
        // SidePanel allocation empty/exact, and draw interactive content in a
        // separate layer clipped to that width.
        let sidebar_width = self.workspace.sidebar_width.clamp(SIDEBAR_MIN_WIDTH, SIDEBAR_MAX_WIDTH);
        let theme = self.theme;
        let response = egui::SidePanel::left("sidebar")
            .resizable(false)
            .exact_width(sidebar_width)
            .show_separator_line(false)
            .frame(egui::Frame {
                fill: theme.surface_panel,
                inner_margin: egui::Margin::ZERO,
                outer_margin: egui::Margin::ZERO,
                stroke: egui::Stroke::NONE,
                rounding: egui::Rounding::ZERO,
                shadow: egui::Shadow::NONE,
            })
            .show(ctx, |ui| {
                let panel_origin = ui.max_rect().min;
                let full = Rect::from_min_size(panel_origin, vec2(sidebar_width, ui.max_rect().height()));
                ui.painter()
                    .rect_filled(full, egui::Rounding::ZERO, theme.surface_panel);
                // Claim exactly `sidebar_width` — nothing else may allocate here.
                ui.allocate_rect(full, Sense::hover());
            });

        let panel_left = response.response.rect.left();
        let y_range = response.response.rect.y_range();
        let full = Rect::from_min_size(
            Pos2::new(panel_left, response.response.rect.top()),
            vec2(sidebar_width, response.response.rect.height()),
        );

        // IDE-style: breathe on the activity-rail side; keep a clear gap before
        // the resize splitter so header actions / tree rows are not flush to it.
        let pad_left = SPACE_SM;
        let pad_right = SPACE_MD;
        let pad_y = SPACE_SM;
        let content_w = (sidebar_width - pad_left - pad_right).max(0.0);
        let content_rect = Rect::from_min_size(
            Pos2::new(full.left() + pad_left, full.top() + pad_y),
            vec2(content_w, (full.height() - 2.0 * pad_y).max(0.0)),
        );

        // Keep content on the Background-adjacent paint path (raw Ui), not an
        // `Area(Order::Middle)` window — Middle windows compete with dialog
        // hit-testing and can leave the palette card under its own dim overlay.
        // Wheel scroll is fixed by bounding ScrollArea max_height below.
        {
            let id = egui::Id::new("dbpro_sidebar_content");
            let layer_id = egui::LayerId::new(egui::Order::Middle, id);
            let mut ui = egui::Ui::new(ctx.clone(), layer_id, id, egui::UiBuilder::new().max_rect(content_rect));
            ui.set_clip_rect(content_rect.expand(SIDEBAR_CLIP_BLEED));
            ui.set_min_size(content_rect.size());
            ui.set_max_size(content_rect.size());
            // Claim the column so wheel hover hit-tests succeed on empty padding.
            let _ = ui.interact(content_rect, id.with("bg"), Sense::hover());
            self.draw_sidebar_contents(&mut ui);
        }

        self.draw_sidebar_resize_handle(ctx, panel_left, y_range);
    }

    fn draw_sidebar_contents(&mut self, ui: &mut egui::Ui) {
        let active_name = if self.connection.lifecycle.active_connection_id().is_some() {
            self.active_connection_name().to_owned()
        } else {
            "DB Pro".to_owned()
        };
        let context = SidebarChromeContext {
            theme: self.theme,
            active_name: &active_name,
            command_palette_shortcut: &Self::format_shortcut(&["Shift", "P"]),
            new_connection_shortcut: &Self::format_shortcut(&["N"]),
            new_query_shortcuts: &Self::shortcut_parts(&["T"]),
        };
        for action in context.draw(ui) {
            match action {
                SidebarChromeAction::OpenCommandPalette => self.palette.open(PaletteMode::Commands),
                SidebarChromeAction::NewConnection => self.connection.open_new(),
                SidebarChromeAction::NewQuery => {
                    self.new_query_document();
                    self.workspace.active_tab = WorkspaceTab::Query;
                }
            }
        }

        ui.add_space(SPACE_SM);
        ui.separator();
        ui.add_space(SPACE_XS);

        // ── 3. Per-activity content ────────────────────────────────────
        match self.workspace.activity {
            Activity::Explorer => self.draw_explorer_sub_panes(ui),
            _ => {
                let scroll_h = ui.available_height();
                egui::ScrollArea::vertical()
                    .id_salt("sidebar_scroll")
                    .auto_shrink([false, false])
                    .max_height(scroll_h)
                    .show(ui, |ui| {
                        ui.add_space(4.0);
                        match self.workspace.activity {
                            Activity::Queries => self.draw_queries(ui),
                            Activity::Files => self.draw_files_activity(ui),
                            Activity::Data => self.draw_data_activity(ui),
                            Activity::History => self.draw_history(ui),
                            Activity::Problems => self.draw_problems(ui),
                            Activity::Transfers => self.draw_transfers_activity(ui),
                            Activity::Monitor => self.draw_monitor_activity(ui),
                            Activity::Security => self.draw_security_activity(ui),
                            Activity::Settings => self.draw_settings(ui),
                            Activity::Diagram => {
                                if navigation_view::draw_diagram_sidebar(ui, self.theme, &self.schema.explorer.schema) {
                                    self.workspace.activity = Activity::Explorer;
                                    self.workspace.sidebar_open = true;
                                }
                            }
                            Activity::Schema => self.draw_schema_workbench_sidebar(ui),
                            Activity::Compare => {
                                let action = {
                                    let connection_name = self.active_connection_name().to_owned();
                                    let driver = self.active_driver().to_owned();
                                    let mut context = schema_compare_view::SchemaCompareViewContext {
                                        theme: self.theme,
                                        compare: &mut self.schema.compare,
                                        schema: &self.schema.explorer.schema,
                                        connection_name: &connection_name,
                                        driver: &driver,
                                        feedback: &mut self.feedback,
                                    };
                                    schema_compare_view::draw_schema_compare_sidebar(&mut context, ui)
                                };
                                if let Some(action) = action {
                                    self.apply_schema_compare_action(action);
                                }
                            }
                            Activity::Tasks => {
                                if self.saved_tasks.pending_destructive_task_id.is_some() {
                                    ui.checkbox(
                                        &mut self.saved_tasks.confirm_destructive,
                                        "Confirm destructive task run",
                                    );
                                    if self.saved_tasks.confirm_destructive {
                                        if let Some(id) = self.saved_tasks.pending_destructive_task_id {
                                            if primary_button(ui, "Run destructive task", self.theme).clicked() {
                                                self.run_saved_task(
                                                    id,
                                                    db_pro_core::domain::saved_task::SavedTaskRunTrigger::Manual,
                                                );
                                            }
                                        }
                                    }
                                    ui.add_space(8.0);
                                }
                                self.draw_tasks_activity(ui);
                            }
                            Activity::Explorer => unreachable!(),
                        }
                    });
            }
        }
    }

    /// Drag grip + separator locked to `sidebar_width` from the panel's left edge.
    fn draw_sidebar_resize_handle(&mut self, ctx: &egui::Context, panel_left: f32, y_range: egui::Rangef) {
        let theme = self.theme;
        let sidebar_width = self.workspace.sidebar_width.clamp(SIDEBAR_MIN_WIDTH, SIDEBAR_MAX_WIDTH);
        let grip = ctx.style().interaction.resize_grab_radius_side.max(5.0);
        let edge_x = panel_left + sidebar_width;
        let resize_rect = Rect::from_x_y_ranges((edge_x - grip)..=(edge_x + grip), y_range);

        let id = egui::Id::new("dbpro_sidebar_resize");
        // PanelResizeLine sits between panels and Middle windows — never above
        // Foreground dialogs/overlays.
        let layer_id = egui::LayerId::new(egui::Order::PanelResizeLine, id);
        let mut grip_ui = egui::Ui::new(ctx.clone(), layer_id, id, egui::UiBuilder::new().max_rect(resize_rect));
        grip_ui.set_clip_rect(ctx.screen_rect());
        let drag_response = grip_ui.allocate_rect(resize_rect, Sense::drag());

        let hovering = drag_response.hovered();
        let dragging = drag_response.dragged();
        if hovering || dragging {
            ctx.set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
        }
        if dragging {
            if let Some(pointer) = ctx
                .pointer_interact_pos()
                .or_else(|| drag_response.interact_pointer_pos())
            {
                self.workspace.set_sidebar_width(pointer.x - panel_left);
                ctx.request_repaint();
            }
        }

        let stroke = if dragging {
            Stroke::new(1.5, theme.accent)
        } else if hovering {
            Stroke::new(1.0, theme.accent)
        } else {
            Stroke::new(1.0, theme.border_subtle)
        };
        let painter = ctx.layer_painter(egui::LayerId::background());
        let line_x = painter.round_to_pixel_center(edge_x - 1.0);
        painter.vline(line_x, y_range, stroke);
    }
}
