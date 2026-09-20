use super::query_editor_support::{apply_completion_item, draw_completion_explanation};
use super::*;
use crate::editor::{CompletionItemKind, SqlDialect};
use egui::RichText;

pub(super) fn draw_floating_completion_popup(
    ctx: &egui::Context,
    theme: DbProTheme,
    dialect: SqlDialect,
    doc: &mut QueryDocument,
) {
    if !doc.completion.is_open || doc.completion.items.is_empty() {
        return;
    }

    let mut apply_item = None;
    let mut close_popup = false;

    ctx.input(|i| {
        for event in &i.events {
            if let egui::Event::Key { key, pressed: true, .. } = event {
                match key {
                    egui::Key::ArrowUp => {
                        doc.completion.select_prev();
                    }
                    egui::Key::ArrowDown => {
                        doc.completion.select_next();
                    }
                    egui::Key::PageUp => {
                        doc.completion.select_page_up(5);
                    }
                    egui::Key::PageDown => {
                        doc.completion.select_page_down(5);
                    }
                    egui::Key::Enter | egui::Key::Tab => {
                        if let Some(item) = doc.completion.current_item() {
                            apply_item = Some(item.clone());
                        }
                    }
                    egui::Key::Escape => {
                        close_popup = true;
                    }
                    _ => {}
                }
            }
        }
    });

    if let Some(item) = apply_item {
        apply_completion_item(doc, &item, dialect);
        return;
    }

    if close_popup {
        doc.completion.close();
        return;
    }

    let popup_height = 300.0;
    let popup_width = 420.0;
    let popup_pos = crate::components::clamp_popup_to_screen(
        doc.completion.popup_position,
        egui::vec2(popup_width, popup_height),
        ctx.screen_rect(),
        10.0,
    );

    let mut clicked_item = None;

    let area_resp = egui::Area::new(egui::Id::new("floating_completion_popup"))
        .order(egui::Order::Foreground)
        .fixed_pos(popup_pos)
        .show(ctx, |ui| {
            egui::Frame {
                // Panel tone contrasts against the flush editor buffer in both themes.
                fill: theme.surface_panel,
                rounding: egui::Rounding::same(8.0),
                stroke: egui::Stroke::new(1.0, theme.border_default),
                shadow: theme.floating_shadow(),
                inner_margin: egui::Margin::same(4.0),
                ..Default::default()
            }
            .show(ui, |ui| {
                ui.set_max_width(popup_width);
                ui.set_max_height(popup_height);
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Suggestions").small().strong().color(theme.text_primary));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            RichText::new("↑↓ navigate · Enter accept · Esc close")
                                .small()
                                .color(theme.text_muted),
                        );
                    });
                });
                ui.separator();

                egui::ScrollArea::vertical().max_height(170.0).show(ui, |ui| {
                    let sel_idx = doc.completion.selected_index;
                    let mut hover_idx = None;
                    let mut click_idx = None;

                    for idx in 0..doc.completion.items.len() {
                        let item = &doc.completion.items[idx];
                        let is_selected = idx == sel_idx;
                        let bg = if is_selected {
                            theme.accent_soft
                        } else {
                            egui::Color32::TRANSPARENT
                        };

                        let item_frame = egui::Frame::none()
                            .fill(bg)
                            .rounding(egui::Rounding::same(5.0))
                            .inner_margin(egui::Margin::symmetric(8.0, 4.0))
                            .show(ui, |ui| {
                                // Fixed row width so label/detail never paint on top of each other.
                                let row_w = (popup_width - 20.0).max(200.0);
                                ui.set_width(row_w);
                                ui.horizontal(|ui| {
                                    let (badge_text, badge_color) = match item.kind {
                                        CompletionItemKind::Keyword => ("KEY", theme.code_keyword),
                                        CompletionItemKind::Table => ("TBL", theme.accent),
                                        CompletionItemKind::View => ("VIEW", theme.info),
                                        CompletionItemKind::Column => ("COL", theme.code_variable),
                                        CompletionItemKind::Function => ("FN", theme.code_function),
                                        CompletionItemKind::Schema => ("SCH", theme.warning),
                                        CompletionItemKind::Snippet => ("SNP", theme.success),
                                        CompletionItemKind::Cte => ("CTE", theme.code_type),
                                    };

                                    egui::Frame::none()
                                        .fill(theme.soft_tint(badge_color))
                                        .rounding(egui::Rounding::same(3.0))
                                        .inner_margin(egui::Margin::symmetric(4.0, 1.0))
                                        .show(ui, |ui| {
                                            ui.label(
                                                RichText::new(badge_text)
                                                    .font(FontId::monospace(9.0))
                                                    .color(badge_color),
                                            );
                                        });
                                    ui.add_space(6.0);

                                    let detail = item.detail.as_deref().unwrap_or("");
                                    let detail_budget = if detail.is_empty() {
                                        0.0
                                    } else {
                                        (ui.available_width() * 0.42).clamp(72.0, 150.0)
                                    };
                                    let label_budget = (ui.available_width() - detail_budget - 4.0).max(48.0);
                                    let row_h = ui.spacing().interact_size.y.max(16.0);

                                    ui.add_sized(
                                        [label_budget, row_h],
                                        egui::Label::new(
                                            RichText::new(&item.label)
                                                .font(FontId::monospace(12.5))
                                                .strong()
                                                .color(theme.text_primary),
                                        )
                                        .truncate(),
                                    )
                                    .on_hover_text(&item.label);

                                    if !detail.is_empty() {
                                        ui.add_sized(
                                            [detail_budget, row_h],
                                            egui::Label::new(
                                                RichText::new(detail).font(font_caption()).color(theme.text_muted),
                                            )
                                            .truncate(),
                                        )
                                        .on_hover_text(detail);
                                    }
                                });
                            });

                        let item_response = item_frame.response.interact(egui::Sense::click());
                        if is_selected {
                            item_response.scroll_to_me(Some(egui::Align::Center));
                        }
                        if item_response.hovered() {
                            hover_idx = Some(idx);
                            item_response.clone().on_hover_ui(|ui| {
                                ui.set_max_width(320.0);
                                draw_completion_explanation(ui, item, &theme);
                            });
                        }
                        if item_response.clicked() {
                            click_idx = Some(idx);
                        }
                    }

                    if let Some(idx) = hover_idx {
                        doc.completion.selected_index = idx;
                    }
                    if let Some(idx) = click_idx {
                        clicked_item = doc.completion.items.get(idx).cloned();
                    }
                });
                if let Some(item) = doc.completion.current_item() {
                    ui.separator();
                    ui.allocate_ui_with_layout(
                        egui::vec2(ui.available_width(), 58.0),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| draw_completion_explanation(ui, item, &theme),
                    );
                }
                ui.separator();
                ui.label(
                    RichText::new("Ctrl/Cmd+Space open · ↑↓ navigate · Enter accept · Esc close")
                        .small()
                        .color(theme.text_muted),
                );
            });
        });

    if let Some(item) = clicked_item {
        apply_completion_item(doc, &item, dialect);
    }

    // Close on click outside
    let clicked_outside = ctx.input(|i| {
        i.pointer.any_click()
            && i.pointer
                .interact_pos()
                .is_some_and(|pos| !area_resp.response.rect.contains(pos))
    });
    if clicked_outside {
        doc.completion.close();
    }
}
