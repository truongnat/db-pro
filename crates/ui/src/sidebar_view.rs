//! Primary left sidebar shell: exact-width panel, padded content, and resize grip.
use super::*;
use egui::{vec2, Align2, Color32, Margin, Pos2, Rect, Rounding, Sense, Stroke};

impl DbProApp {
    pub(super) fn draw_sidebar(&mut self, ctx: &egui::Context) {
        // egui SidePanel resize grabs on `panel_rect` but paints the separator on
        // the narrower frame response — and frame margins make that mismatch
        // worse, leaving a clear_color (white) strip beside the workspace.
        // Drive width ourselves and paint/allocate the full exact width.
        let sidebar_width = self.sidebar_width.clamp(SIDEBAR_MIN_WIDTH, SIDEBAR_MAX_WIDTH);
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
                let full = ui.max_rect();
                ui.painter()
                    .rect_filled(full, egui::Rounding::ZERO, theme.surface_panel);

                // Keep left/right padding equal so content breathes beside the
                // activity rail and the resize edge (not flush on either side).
                let pad_x = SPACE_MD;
                let pad_y = SPACE_SM;
                let content_rect = Rect::from_min_max(
                    Pos2::new(full.left() + pad_x, full.top() + pad_y),
                    Pos2::new(full.right() - pad_x, full.bottom() - pad_y),
                );
                ui.allocate_new_ui(egui::UiBuilder::new().max_rect(content_rect), |ui| {
                    // Keep the content ui as wide as the padded panel so tree rows
                    // and toolbars stretch when the sidebar is resized.
                    let content_w = content_rect.width();
                    ui.set_min_width(content_w);
                    ui.set_max_width(content_w);

                    // ── 1. Codex-style Header Row: Workspace Selector + Action Icons ──
                    ui.add_space(SPACE_SM);
                    ui.horizontal(|ui| {
                        let active_name = if self.active_connection_id.is_some() {
                            self.active_connection_name()
                        } else {
                            "DB Pro"
                        };

                        // Workspace / Connection Dropdown Selector (e.g. "Codex ⌵")
                        let selector_resp = egui::Frame {
                            fill: Color32::TRANSPARENT,
                            rounding: Rounding::same(RADIUS_SM),
                            inner_margin: Margin::symmetric(SPACE_XS, 3.0),
                            ..Default::default()
                        }
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing = vec2(SPACE_XS, 0.0);
                                ui.add(
                                    egui::Label::new(
                                        RichText::new(active_name)
                                            .font(font_ui_label())
                                            .strong()
                                            .color(self.theme.text_primary),
                                    )
                                    .truncate(),
                                )
                                .on_hover_text(active_name);
                                ui.label(
                                    RichText::new(char::from(Icon::ChevronDown).to_string())
                                        .font(font_icon(ICON_XS))
                                        .color(self.theme.text_secondary),
                                );
                            });
                        });
                        let selector_interact = ui.interact(
                            selector_resp.response.rect,
                            ui.id().with("workspace_selector"),
                            Sense::click(),
                        );
                        if selector_interact.hovered() {
                            ui.painter().rect_filled(
                                selector_resp.response.rect,
                                Rounding::same(RADIUS_SM),
                                self.theme.surface_hover,
                            );
                        }
                        if selector_interact.clicked() {
                            self.open_palette(PaletteMode::Commands);
                        }
                        selector_interact.on_hover_text("Switch connection / workspace");

                        // Right header action buttons: Search & New Connection
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if Button::new(self.theme)
                                .icon(Icon::Plus)
                                .variant(ButtonVariant::Ghost)
                                .size(ButtonSize::IconSm)
                                .tooltip("New Connection")
                                .show(ui)
                                .clicked()
                            {
                                self.open_new_connection();
                            }
                            if Button::new(self.theme)
                                .icon(Icon::Search)
                                .variant(ButtonVariant::Ghost)
                                .size(ButtonSize::IconSm)
                                .tooltip(format!(
                                    "Search / Command Palette ({}⇧P)",
                                    Self::primary_modifier_label()
                                ))
                                .show(ui)
                                .clicked()
                            {
                                self.open_palette(PaletteMode::Commands);
                            }
                        });
                    });
                    ui.add_space(SPACE_XS);

                    // ── 2. Codex-style Primary Action: "+ New query" Button ──────
                    let new_query_rect = ui.available_rect_before_wrap();
                    let new_query_h = BUTTON_HEIGHT_SM;
                    let btn_rect = Rect::from_min_size(new_query_rect.min, vec2(ui.available_width(), new_query_h));
                    let new_query_resp = ui.allocate_rect(btn_rect, Sense::click());
                    let is_hovered = new_query_resp.hovered();
                    let bg_color = if is_hovered {
                        self.theme.surface_hover
                    } else {
                        self.theme.surface_panel
                    };
                    ui.painter().rect_filled(btn_rect, Rounding::same(RADIUS_SM), bg_color);
                    ui.painter().rect_stroke(
                        btn_rect,
                        Rounding::same(RADIUS_SM),
                        Stroke::new(
                            STROKE_THIN,
                            if is_hovered {
                                self.theme.border_default
                            } else {
                                self.theme.border_subtle
                            },
                        ),
                    );

                    let left_center = Pos2::new(btn_rect.left() + SPACE_MD, btn_rect.center().y);
                    ui.painter().text(
                        left_center,
                        Align2::LEFT_CENTER,
                        char::from(Icon::SquarePen).to_string(),
                        font_icon(ICON_SM),
                        self.theme.text_primary,
                    );
                    ui.painter().text(
                        Pos2::new(left_center.x + SPACE_LG + 2.0, left_center.y),
                        Align2::LEFT_CENTER,
                        "New query",
                        font_caption(),
                        self.theme.text_primary,
                    );
                    ui.painter().text(
                        Pos2::new(btn_rect.right() - SPACE_MD, left_center.y),
                        Align2::RIGHT_CENTER,
                        format!("{}N", Self::primary_modifier_label()),
                        font_caption(),
                        self.theme.text_muted,
                    );

                    if new_query_resp.clicked() {
                        self.new_query_document();
                        self.active_tab = WorkspaceTab::Query;
                    }
                    new_query_resp.on_hover_cursor(egui::CursorIcon::PointingHand);

                    ui.add_space(SPACE_SM);
                    ui.separator();
                    ui.add_space(SPACE_XS);

                    // ── 3. Per-activity content ────────────────────────────────────
                    match self.activity {
                        Activity::Explorer => self.draw_explorer_sub_panes(ui),
                        _ => {
                            egui::ScrollArea::vertical().id_salt("sidebar_scroll").show(ui, |ui| {
                                ui.add_space(4.0);
                                match self.activity {
                                    Activity::Queries => self.draw_queries(ui),
                                    Activity::Files => self.draw_files_activity(ui),
                                    Activity::Data => self.draw_data_activity(ui),
                                    Activity::History => self.draw_history(ui),
                                    Activity::Problems => self.draw_problems(ui),
                                    Activity::Transfers => self.draw_transfers_activity(ui),
                                    Activity::Monitor => self.draw_monitor_activity(ui),
                                    Activity::Security => self.draw_security_activity(ui),
                                    Activity::Settings => self.draw_settings(ui),
                                    Activity::Diagram => self.draw_diagram_sidebar(ui),
                                    Activity::Schema => self.draw_schema_workbench_sidebar(ui),
                                    Activity::Compare => self.draw_schema_compare_sidebar(ui),
                                    Activity::Tasks => {
                                        if self.pending_destructive_task_id.is_some() {
                                            ui.checkbox(
                                                &mut self.saved_task_confirm_destructive,
                                                "Confirm destructive task run",
                                            );
                                            if self.saved_task_confirm_destructive {
                                                if let Some(id) = self.pending_destructive_task_id {
                                                    if primary_button(ui, "Run destructive task", self.theme).clicked()
                                                    {
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
                });

                // Claim the full exact_width so CentralPanel starts at the painted edge.
                ui.allocate_rect(full, Sense::hover());
            });

        // Prefer the width we asked for; fall back to measured rect if egui clamped.
        let painted = response.response.rect;
        if (painted.width() - sidebar_width).abs() > 0.5 {
            self.sidebar_width = painted.width().clamp(SIDEBAR_MIN_WIDTH, SIDEBAR_MAX_WIDTH);
        }
        self.draw_sidebar_resize_handle(ctx, painted);
    }

    /// Drag grip + separator locked to the painted sidebar edge.
    fn draw_sidebar_resize_handle(&mut self, ctx: &egui::Context, panel_rect: Rect) {
        let theme = self.theme;
        let grip = ctx.style().interaction.resize_grab_radius_side.max(5.0);
        let edge_x = panel_rect.right();
        let resize_rect = Rect::from_x_y_ranges((edge_x - grip)..=(edge_x + grip), panel_rect.y_range());

        let id = egui::Id::new("dbpro_sidebar_resize");
        let layer_id = egui::LayerId::new(egui::Order::Foreground, id);
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
                self.sidebar_width = (pointer.x - panel_rect.left()).clamp(SIDEBAR_MIN_WIDTH, SIDEBAR_MAX_WIDTH);
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
        painter.vline(line_x, panel_rect.y_range(), stroke);
    }
}
