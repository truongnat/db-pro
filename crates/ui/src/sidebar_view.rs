//! Primary left sidebar shell: exact-width panel, padded content, and resize grip.
use super::*;
use egui::{vec2, Align2, Pos2, Rect, Rounding, Sense, Stroke};

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
                // Anchor the panel to the width we own; do not trust a wider
                // max_rect from egui panel state / frame response mismatch.
                let panel_origin = ui.max_rect().min;
                let full = Rect::from_min_size(panel_origin, vec2(sidebar_width, ui.max_rect().height()));
                ui.painter()
                    .rect_filled(full, egui::Rounding::ZERO, theme.surface_panel);

                // IDE-style: breathe on the activity-rail side, sit nearly flush
                // against the resize edge so navigator content fills the panel.
                let pad_left = SPACE_SM;
                let pad_right = SPACE_SM;
                let pad_y = SPACE_SM;
                // Derive content width from the clamped sidebar width, not from
                // egui's panel response rect — that rect can disagree with the
                // width we asked for and leave a dead gutter before the drag line.
                let content_w = (sidebar_width - pad_left - pad_right).max(0.0);
                let content_rect = Rect::from_min_size(
                    Pos2::new(full.left() + pad_left, full.top() + pad_y),
                    vec2(content_w, (full.height() - 2.0 * pad_y).max(0.0)),
                );
                ui.allocate_new_ui(egui::UiBuilder::new().max_rect(content_rect), |ui| {
                    // Keep the content ui as wide as the padded panel so tree rows
                    // and toolbars stretch when the sidebar is resized.
                    ui.set_min_width(content_w);
                    ui.set_max_width(content_w);
                    ui.set_min_height(content_rect.height());

                    // ── 1. Header: connection selector + actions ───────────────
                    // Paint order matters: allocate → hover wash → text/icons on top.
                    // The previous path filled surface_hover *after* the label, which
                    // wiped the name to a blank wash on hover.
                    ui.add_space(SPACE_SM);
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing = vec2(SPACE_XS, 0.0);
                        let active_name = if self.active_connection_id.is_some() {
                            self.active_connection_name().to_owned()
                        } else {
                            "DB Pro".to_owned()
                        };

                        const HEADER_ACTIONS_W: f32 = 52.0;
                        const SELECTOR_H: f32 = 24.0;
                        let selector_w =
                            (ui.available_width() - HEADER_ACTIONS_W - ui.spacing().item_spacing.x).max(72.0);
                        let (sel_rect, sel_resp) =
                            ui.allocate_exact_size(vec2(selector_w, SELECTOR_H), Sense::click());

                        if sel_resp.hovered() {
                            ui.painter().rect_filled(
                                sel_rect,
                                Rounding::same(RADIUS_SM),
                                self.theme.surface_hover,
                            );
                            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                        }

                        let chevron = char::from(Icon::ChevronDown).to_string();
                        let chevron_galley = ui.painter().layout_no_wrap(
                            chevron,
                            font_icon(ICON_XS),
                            self.theme.text_secondary,
                        );
                        let name_max = (sel_rect.width()
                            - SPACE_XS * 2.0
                            - chevron_galley.size().x
                            - SPACE_XS)
                            .max(24.0);
                        let name_galley = ui.painter().layout_job({
                            let mut job = egui::text::LayoutJob::single_section(
                                active_name.to_owned(),
                                egui::TextFormat {
                                    font_id: font_ui_label(),
                                    color: self.theme.text_primary,
                                    ..Default::default()
                                },
                            );
                            job.wrap = egui::text::TextWrapping {
                                max_width: name_max,
                                max_rows: 1,
                                break_anywhere: true,
                                overflow_character: Some('…'),
                            };
                            job
                        });
                        let name_pos = Pos2::new(
                            sel_rect.left() + SPACE_XS,
                            sel_rect.center().y - name_galley.size().y * 0.5,
                        );
                        let chevron_pos = Pos2::new(
                            name_pos.x + name_galley.size().x + SPACE_XS,
                            sel_rect.center().y - chevron_galley.size().y * 0.5,
                        );
                        ui.painter().galley(name_pos, name_galley, self.theme.text_primary);
                        ui.painter().galley(chevron_pos, chevron_galley, self.theme.text_secondary);

                        if sel_resp.clicked() {
                            self.open_palette(PaletteMode::Commands);
                        }
                        sel_resp.on_hover_text(format!("{active_name}\nSwitch connection / workspace"));

                        if Button::new(self.theme)
                            .icon(Icon::Search)
                            .variant(ButtonVariant::Ghost)
                            .size(ButtonSize::IconSm)
                            .tooltip(format!(
                                "Search / Command Palette ({})",
                                Self::format_shortcut(&["Shift", "P"])
                            ))
                            .show(ui)
                            .clicked()
                        {
                            self.open_palette(PaletteMode::Commands);
                        }
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
                    });
                    ui.add_space(SPACE_XS);

                    // ── 2. Primary action: New query ───────────────────────────
                    let new_query_h = BUTTON_HEIGHT_SM;
                    let btn_rect = Rect::from_min_size(
                        ui.cursor().min,
                        vec2(ui.available_width(), new_query_h),
                    );
                    let new_query_resp = ui.allocate_rect(btn_rect, Sense::click());
                    let is_hovered = new_query_resp.hovered();
                    ui.painter().rect_filled(
                        btn_rect,
                        Rounding::same(RADIUS_SM),
                        if is_hovered {
                            self.theme.surface_hover
                        } else {
                            self.theme.surface_panel
                        },
                    );
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
                    // Shortcut hint as separate kbd chips: Ctrl + N (not "CtrlN").
                    paint_shortcut_chips(
                        ui,
                        btn_rect.right() - SPACE_MD,
                        left_center.y,
                        &Self::shortcut_parts(&["N"]),
                        self.theme,
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

        // Lock the drag handle to the width we requested (`sidebar_width`), not
        // egui's frame response width — a wider response rect was painting the
        // separator far past the actual navigator content.
        let painted = response.response.rect;
        self.draw_sidebar_resize_handle(ctx, painted.left(), painted.y_range());
    }

    /// Drag grip + separator locked to `sidebar_width` from the panel's left edge.
    fn draw_sidebar_resize_handle(
        &mut self,
        ctx: &egui::Context,
        panel_left: f32,
        y_range: egui::Rangef,
    ) {
        let theme = self.theme;
        let sidebar_width = self.sidebar_width.clamp(SIDEBAR_MIN_WIDTH, SIDEBAR_MAX_WIDTH);
        let grip = ctx.style().interaction.resize_grab_radius_side.max(5.0);
        let edge_x = panel_left + sidebar_width;
        let resize_rect = Rect::from_x_y_ranges((edge_x - grip)..=(edge_x + grip), y_range);

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
                self.sidebar_width = (pointer.x - panel_left).clamp(SIDEBAR_MIN_WIDTH, SIDEBAR_MAX_WIDTH);
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

/// Paints right-aligned kbd chips (`Ctrl` + `N`) ending at `right_x`.
fn paint_shortcut_chips(
    ui: &egui::Ui,
    right_x: f32,
    center_y: f32,
    parts: &[String],
    theme: DbProTheme,
) {
    if parts.is_empty() {
        return;
    }
    let painter = ui.painter();
    let font = egui::FontId::monospace(10.5);
    let chip_pad_x = 5.0;
    let chip_pad_y = 2.0;
    let plus_gap = 2.0;

    let galleys: Vec<_> = parts
        .iter()
        .map(|part| painter.layout_no_wrap(part.clone(), font.clone(), theme.text_muted))
        .collect();
    let plus_galley = painter.layout_no_wrap("+".to_owned(), font, theme.text_muted);

    let mut total_w = galleys
        .iter()
        .map(|galley| chip_pad_x * 2.0 + galley.size().x)
        .sum::<f32>();
    if galleys.len() > 1 {
        total_w += (galleys.len() - 1) as f32 * (plus_gap * 2.0 + plus_galley.size().x);
    }

    let mut x = right_x - total_w;
    for (i, galley) in galleys.iter().enumerate() {
        if i > 0 {
            x += plus_gap;
            painter.galley(
                Pos2::new(x, center_y - plus_galley.size().y * 0.5),
                plus_galley.clone(),
                theme.text_muted,
            );
            x += plus_galley.size().x + plus_gap;
        }
        let chip_w = galley.size().x + chip_pad_x * 2.0;
        let chip_h = galley.size().y + chip_pad_y * 2.0;
        let chip = Rect::from_min_size(
            Pos2::new(x, center_y - chip_h * 0.5),
            vec2(chip_w, chip_h),
        );
        painter.rect_filled(chip, Rounding::same(4.0), theme.surface_elevated);
        painter.rect_stroke(chip, Rounding::same(4.0), Stroke::new(1.0, theme.border_subtle));
        painter.galley(
            Pos2::new(chip.left() + chip_pad_x, center_y - galley.size().y * 0.5),
            std::sync::Arc::clone(galley),
            theme.text_muted,
        );
        x += chip_w;
    }
}
