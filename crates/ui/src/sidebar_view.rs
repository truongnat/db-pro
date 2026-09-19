//! Primary left sidebar shell: exact-width panel, padded content, and resize grip.
use super::*;
use egui::{vec2, Align2, Pos2, Rect, Rounding, Sense, Stroke};

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
        // ── 1. Header: one launcher pill + new-connection ───────────
        // Name and search share a single hit target → Command Palette
        // (connections + commands). Plus stays separate (create flow).
        ui.add_space(SPACE_SM);
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = vec2(SPACE_XS, 0.0);
            let active_name = if self.connection_lifecycle.active_connection_id.is_some() {
                self.active_connection_name().to_owned()
            } else {
                "DB Pro".to_owned()
            };

            const PLUS_SLOT_W: f32 = 28.0;
            const SELECTOR_H: f32 = 24.0;
            const SEARCH_SLOT_W: f32 = 18.0;
            let selector_w = (ui.available_width() - PLUS_SLOT_W - ui.spacing().item_spacing.x).max(72.0);
            let (sel_rect, sel_resp) = ui.allocate_exact_size(vec2(selector_w, SELECTOR_H), Sense::click());

            if sel_resp.hovered() {
                ui.painter()
                    .rect_filled(sel_rect, Rounding::same(RADIUS_SM), self.theme.surface_hover);
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }

            let search_galley = ui.painter().layout_no_wrap(
                char::from(Icon::Search).to_string(),
                font_icon(ICON_XS),
                self.theme.text_secondary,
            );
            let name_max = (sel_rect.width() - SPACE_XS * 2.0 - SEARCH_SLOT_W - SPACE_XS).max(24.0);
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
            let search_pos = Pos2::new(
                sel_rect.right() - SPACE_XS - search_galley.size().x,
                sel_rect.center().y - search_galley.size().y * 0.5,
            );
            ui.painter().galley(name_pos, name_galley, self.theme.text_primary);
            ui.painter()
                .galley(search_pos, search_galley, self.theme.text_secondary);

            if sel_resp.clicked() {
                self.open_palette(PaletteMode::Commands);
            }
            sel_resp.on_hover_text(format!(
                "{active_name}\nCommand Palette ({})",
                Self::format_shortcut(&["Shift", "P"])
            ));

            if Button::new(self.theme)
                .icon(Icon::Plus)
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::IconSm)
                .tooltip(format!("New Connection ({})", Self::format_shortcut(&["N"])))
                .show(ui)
                .clicked()
            {
                self.open_new_connection();
            }
        });
        ui.add_space(SPACE_XS);

        // ── 2. Primary action: New query ───────────────────────────
        let new_query_h = BUTTON_HEIGHT_SM;
        let btn_rect = Rect::from_min_size(ui.cursor().min, vec2(ui.available_width(), new_query_h));
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
        // Shortcut hint as separate kbd chips: Ctrl + T (matches query.new).
        paint_shortcut_chips(
            ui,
            btn_rect.right() - SPACE_MD,
            left_center.y,
            &Self::shortcut_parts(&["T"]),
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
                let scroll_h = ui.available_height();
                egui::ScrollArea::vertical()
                    .id_salt("sidebar_scroll")
                    .auto_shrink([false, false])
                    .max_height(scroll_h)
                    .show(ui, |ui| {
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
        let sidebar_width = self.sidebar_width.clamp(SIDEBAR_MIN_WIDTH, SIDEBAR_MAX_WIDTH);
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
fn paint_shortcut_chips(ui: &egui::Ui, right_x: f32, center_y: f32, parts: &[String], theme: DbProTheme) {
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
        let chip = Rect::from_min_size(Pos2::new(x, center_y - chip_h * 0.5), vec2(chip_w, chip_h));
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
