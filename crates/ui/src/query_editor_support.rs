//! Query SQL editor surface and floating completion popup.
use super::*;
use crate::editor::{CompletionItem, SqlDialect};
use crate::query::{HoverColumn, RichHoverHelp, SqlSignatureHelp, SqlSymbolHelp};
use egui::RichText;

fn draw_symbol_help(ui: &mut egui::Ui, help: &SqlSymbolHelp, theme: &DbProTheme) {
    ui.label(
        RichText::new(&help.title)
            .font(FontId::monospace(12.5))
            .strong()
            .color(theme.text_primary),
    );
    ui.horizontal(|ui| {
        ui.label(RichText::new(&help.kind).small().strong().color(theme.accent));
        ui.label(RichText::new(&help.detail).small().color(theme.text_secondary));
    });
    ui.add(egui::Label::new(RichText::new(&help.documentation).small().color(theme.text_muted)).wrap());
}

/// Render a column summary row inside a table hover card.
fn draw_hover_column_row(ui: &mut egui::Ui, col: &HoverColumn, theme: &DbProTheme) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = egui::vec2(4.0, 0.0);
        // PK badge
        if col.is_primary_key {
            egui::Frame::none()
                .fill(theme.soft_tint(theme.warning))
                .rounding(egui::Rounding::same(3.0))
                .inner_margin(egui::Margin::symmetric(3.0, 1.0))
                .show(ui, |ui| {
                    ui.label(RichText::new("PK").font(FontId::monospace(8.5)).color(theme.warning));
                });
        } else if col.is_foreign_key {
            egui::Frame::none()
                .fill(theme.soft_tint(theme.info))
                .rounding(egui::Rounding::same(3.0))
                .inner_margin(egui::Margin::symmetric(3.0, 1.0))
                .show(ui, |ui| {
                    ui.label(RichText::new("FK").font(FontId::monospace(8.5)).color(theme.info));
                });
        } else {
            ui.add_space(24.0);
        }
        // Column name
        ui.label(
            RichText::new(&col.name)
                .font(FontId::monospace(12.0))
                .color(theme.text_primary),
        );
        // Type
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let null_text = if col.nullable { "null" } else { "not null" };
            ui.label(
                RichText::new(null_text)
                    .font(FontId::monospace(10.0))
                    .color(theme.text_muted),
            );
            ui.add_space(4.0);
            ui.label(
                RichText::new(&col.data_type)
                    .font(FontId::monospace(10.5))
                    .color(theme.code_type),
            );
        });
    });
}

/// Draw the rich hover popup for any `RichHoverHelp` variant.
pub(super) fn draw_rich_hover_popup(
    ctx: &egui::Context,
    anchor: egui::Rect,
    token_range: (usize, usize),
    help: &RichHoverHelp,
    theme: &DbProTheme,
) {
    let popup_width: f32 = match help {
        RichHoverHelp::Table { columns, .. } => {
            if columns.len() > 8 {
                520.0
            } else {
                420.0
            }
        }
        RichHoverHelp::Keyword { example: Some(_), .. } => 500.0,
        _ => 380.0,
    };
    let max_height: f32 = 400.0;

    let position = crate::components::clamp_popup_to_screen(
        anchor.left_bottom() + egui::vec2(0.0, 6.0),
        egui::vec2(popup_width, max_height),
        ctx.screen_rect(),
        10.0,
    );

    egui::Area::new(egui::Id::new(("sql_rich_hover", token_range)))
        .order(egui::Order::Tooltip)
        .fixed_pos(position)
        .interactable(true) // Allow text selection / copy
        .show(ctx, |ui| {
            egui::Frame::none()
                .fill(theme.surface_panel)
                .stroke(egui::Stroke::new(1.0, theme.border_default))
                .rounding(egui::Rounding::same(8.0))
                .shadow(theme.floating_shadow())
                .inner_margin(egui::Margin::same(10.0))
                .show(ui, |ui| {
                    ui.set_max_width(popup_width);
                    egui::ScrollArea::vertical()
                        .max_height(max_height - 20.0)
                        .show(ui, |ui| {
                            draw_rich_hover_content(ui, help, theme);
                        });
                });
        });
}

/// Inner content renderer (called inside the scrollable area).
fn draw_rich_hover_content(ui: &mut egui::Ui, help: &RichHoverHelp, theme: &DbProTheme) {
    match help {
        RichHoverHelp::Table {
            qualified_name,
            kind,
            row_count,
            columns,
            foreign_keys,
        } => {
            // Header
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing = egui::vec2(4.0, 0.0);
                let (badge_text, badge_color) = if kind == "View" {
                    ("VIEW", theme.info)
                } else {
                    ("TABLE", theme.accent)
                };
                egui::Frame::none()
                    .fill(theme.soft_tint(badge_color))
                    .rounding(egui::Rounding::same(4.0))
                    .inner_margin(egui::Margin::symmetric(5.0, 2.0))
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new(badge_text)
                                .font(FontId::monospace(9.0))
                                .color(badge_color),
                        );
                    });
                ui.label(
                    RichText::new(qualified_name)
                        .font(FontId::monospace(13.0))
                        .strong()
                        .color(theme.text_primary),
                );
            });

            if let Some(rows) = row_count {
                ui.label(
                    RichText::new(format!("~{rows} rows"))
                        .font(FontId::proportional(11.0))
                        .color(theme.text_muted),
                );
            }

            if !columns.is_empty() {
                ui.add_space(6.0);
                ui.label(
                    RichText::new(format!("{} columns", columns.len()))
                        .small()
                        .strong()
                        .color(theme.text_muted),
                );
                ui.add_space(2.0);
                ui.separator();
                for col in columns.iter().take(20) {
                    draw_hover_column_row(ui, col, theme);
                }
                if columns.len() > 20 {
                    ui.label(
                        RichText::new(format!("… and {} more columns", columns.len() - 20))
                            .small()
                            .color(theme.text_muted),
                    );
                }
            }

            if !foreign_keys.is_empty() {
                ui.add_space(6.0);
                ui.label(RichText::new("Foreign keys").small().strong().color(theme.text_muted));
                ui.add_space(2.0);
                for fk in foreign_keys {
                    ui.label(
                        RichText::new(format!(
                            "({}) → {}({})",
                            fk.from_columns.join(", "),
                            fk.to_table,
                            fk.to_columns.join(", "),
                        ))
                        .font(FontId::monospace(11.0))
                        .color(theme.text_secondary),
                    );
                }
            }
        }

        RichHoverHelp::Column {
            qualified_name,
            data_type,
            nullable,
            is_primary_key,
            parent_table,
            foreign_key_target,
        } => {
            // Type badge + name
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing = egui::vec2(4.0, 0.0);
                egui::Frame::none()
                    .fill(theme.soft_tint(theme.code_type))
                    .rounding(egui::Rounding::same(4.0))
                    .inner_margin(egui::Margin::symmetric(5.0, 2.0))
                    .show(ui, |ui| {
                        ui.label(RichText::new("COL").font(FontId::monospace(9.0)).color(theme.code_type));
                    });
                ui.label(
                    RichText::new(qualified_name)
                        .font(FontId::monospace(13.0))
                        .strong()
                        .color(theme.text_primary),
                );
            });

            ui.add_space(4.0);
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = egui::vec2(4.0, 4.0);
                // Type chip
                egui::Frame::none()
                    .fill(theme.soft_tint(theme.code_type))
                    .rounding(egui::Rounding::same(4.0))
                    .inner_margin(egui::Margin::symmetric(5.0, 2.0))
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new(data_type)
                                .font(FontId::monospace(10.5))
                                .color(theme.code_type),
                        );
                    });
                // Null chip
                let null_color = if *nullable { theme.text_muted } else { theme.warning };
                let null_text = if *nullable { "nullable" } else { "not null" };
                egui::Frame::none()
                    .fill(theme.soft_tint(null_color))
                    .rounding(egui::Rounding::same(4.0))
                    .inner_margin(egui::Margin::symmetric(5.0, 2.0))
                    .show(ui, |ui| {
                        ui.label(RichText::new(null_text).font(FontId::monospace(10.0)).color(null_color));
                    });
                // PK chip
                if *is_primary_key {
                    egui::Frame::none()
                        .fill(theme.soft_tint(theme.warning))
                        .rounding(egui::Rounding::same(4.0))
                        .inner_margin(egui::Margin::symmetric(5.0, 2.0))
                        .show(ui, |ui| {
                            ui.label(
                                RichText::new("primary key")
                                    .font(FontId::monospace(10.0))
                                    .color(theme.warning),
                            );
                        });
                }
            });

            ui.add_space(4.0);
            ui.label(
                RichText::new(format!("Column of {parent_table}"))
                    .small()
                    .color(theme.text_muted),
            );

            if let Some(target) = foreign_key_target {
                ui.label(
                    RichText::new(format!("References {target}"))
                        .font(FontId::monospace(11.0))
                        .color(theme.info),
                );
            }
        }

        RichHoverHelp::Function {
            label,
            parameters,
            active_parameter,
            return_type,
            documentation,
        } => {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing = egui::vec2(4.0, 0.0);
                egui::Frame::none()
                    .fill(theme.soft_tint(theme.code_function))
                    .rounding(egui::Rounding::same(4.0))
                    .inner_margin(egui::Margin::symmetric(5.0, 2.0))
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new("FN")
                                .font(FontId::monospace(9.0))
                                .color(theme.code_function),
                        );
                    });
                ui.label(
                    RichText::new(label)
                        .font(FontId::monospace(12.5))
                        .strong()
                        .color(theme.text_primary),
                );
            });
            if !parameters.is_empty() {
                ui.add_space(4.0);
                ui.horizontal_wrapped(|ui| {
                    for (idx, param) in parameters.iter().enumerate() {
                        let color = if idx == *active_parameter {
                            theme.accent
                        } else {
                            theme.text_muted
                        };
                        ui.label(RichText::new(param).font(FontId::monospace(11.0)).color(color));
                        if idx + 1 < parameters.len() {
                            ui.label(RichText::new(",").font(FontId::monospace(11.0)).color(theme.text_muted));
                        }
                    }
                });
            }
            if !return_type.is_empty() {
                ui.label(
                    RichText::new(format!("→ {return_type}"))
                        .font(FontId::monospace(11.0))
                        .color(theme.code_type),
                );
            }
            if !documentation.is_empty() {
                ui.add_space(4.0);
                ui.add(egui::Label::new(RichText::new(documentation).small().color(theme.text_muted)).wrap());
            }
        }

        RichHoverHelp::Keyword {
            keyword,
            dialect_note,
            documentation,
            example,
        } => {
            // Header row
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing = egui::vec2(4.0, 0.0);
                egui::Frame::none()
                    .fill(theme.soft_tint(theme.code_keyword))
                    .rounding(egui::Rounding::same(4.0))
                    .inner_margin(egui::Margin::symmetric(5.0, 2.0))
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new("SQL")
                                .font(FontId::monospace(9.0))
                                .color(theme.code_keyword),
                        );
                    });
                ui.label(
                    RichText::new(keyword.as_str())
                        .font(FontId::monospace(13.0))
                        .strong()
                        .color(theme.code_keyword),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(RichText::new(dialect_note.as_str()).small().color(theme.text_muted));
                });
            });

            ui.add_space(4.0);
            ui.add(egui::Label::new(RichText::new(documentation.as_str()).small().color(theme.text_primary)).wrap());

            if let Some(ex) = example {
                ui.add_space(6.0);
                ui.label(RichText::new("Example").small().strong().color(theme.text_muted));
                ui.add_space(2.0);
                egui::Frame::none()
                    .fill(theme.editor_gutter_fill())
                    .rounding(egui::Rounding::same(5.0))
                    .inner_margin(egui::Margin::same(6.0))
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.add(
                            egui::Label::new(
                                RichText::new(ex.as_str())
                                    .font(FontId::monospace(11.0))
                                    .color(theme.text_secondary),
                            )
                            .wrap(),
                        );
                    });
            }
        }

        RichHoverHelp::Symbol(help) => {
            draw_symbol_help(ui, help, theme);
        }
    }
}

pub(super) fn draw_signature_help(
    ctx: &egui::Context,
    anchor: egui::Pos2,
    signature: &SqlSignatureHelp,
    theme: &DbProTheme,
) {
    const POPUP_WIDTH: f32 = 440.0;
    const POPUP_HEIGHT: f32 = 112.0;
    let position = crate::components::clamp_popup_to_screen(
        anchor + egui::vec2(8.0, 22.0),
        egui::vec2(POPUP_WIDTH, POPUP_HEIGHT),
        ctx.screen_rect(),
        10.0,
    );
    egui::Area::new(egui::Id::new("sql_signature_help"))
        .order(egui::Order::Foreground)
        .fixed_pos(position)
        .interactable(false)
        .show(ctx, |ui| {
            egui::Frame::none()
                .fill(theme.surface_panel)
                .stroke(egui::Stroke::new(1.0, theme.border_default))
                .rounding(egui::Rounding::same(7.0))
                .shadow(theme.floating_shadow())
                .inner_margin(egui::Margin::same(8.0))
                .show(ui, |ui| {
                    ui.set_width(POPUP_WIDTH);
                    ui.label(
                        RichText::new(&signature.label)
                            .font(FontId::monospace(12.0))
                            .color(theme.text_primary),
                    );
                    if !signature.parameters.is_empty() {
                        ui.horizontal_wrapped(|ui| {
                            for (index, parameter) in signature.parameters.iter().enumerate() {
                                let color = if index == signature.active_parameter {
                                    theme.accent
                                } else {
                                    theme.text_muted
                                };
                                ui.label(RichText::new(parameter).font(FontId::monospace(11.0)).color(color));
                            }
                        });
                    }
                    ui.label(
                        RichText::new(&signature.documentation)
                            .small()
                            .color(theme.text_secondary),
                    );
                });
        });
}

pub(super) fn draw_completion_explanation(ui: &mut egui::Ui, item: &CompletionItem, theme: &DbProTheme) {
    ui.label(
        RichText::new(&item.label)
            .font(FontId::monospace(12.5))
            .strong()
            .color(theme.text_primary),
    );
    if let Some(detail) = item.detail.as_deref() {
        ui.label(RichText::new(detail).small().color(theme.text_secondary));
    }
    if let Some(documentation) = item.documentation.as_deref() {
        ui.add(egui::Label::new(RichText::new(documentation).small().color(theme.text_muted)).wrap());
    }
}

pub(super) fn apply_completion_item(doc: &mut QueryDocument, item: &CompletionItem, dialect: SqlDialect) -> bool {
    if !doc.completion.can_apply_to_version(doc.buffer.version()) {
        doc.completion.close();
        return false;
    }

    let (start, end) = item.replacement_range;
    let source = doc.buffer.text();
    if start > end || end > source.len() || !source.is_char_boundary(start) || !source.is_char_boundary(end) {
        doc.completion.close();
        return false;
    }

    doc.buffer.replace(start, end, &item.insert_text);
    doc.cursor.set_offset(&doc.buffer, start + item.insert_text.len());
    doc.selection.collapse_to_active();
    doc.reanalyze(dialect);
    doc.dirty = true;
    doc.prediction = None;
    doc.completion.close();
    true
}
