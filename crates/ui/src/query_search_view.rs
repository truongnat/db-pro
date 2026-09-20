use super::*;

pub(super) struct QuerySearchContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) editor: &'a mut QueryEditorState,
    pub(super) session: &'a mut QuerySessionState,
}

pub(super) fn draw_editor_search_overlay(context: &mut QuerySearchContext<'_>, ctx: &egui::Context) {
    if !context.editor.editor_search_open {
        return;
    }
    let editor_rect = context.editor.query_editor_rect;
    if !editor_rect.is_positive() {
        return;
    }
    let width = 320.0;
    let pos = egui::pos2(
        (editor_rect.right() - width - 8.0).max(editor_rect.left() + 8.0),
        editor_rect.top() + 6.0,
    );
    let mut goto_range = None;
    let mut close_search = false;

    egui::Area::new(egui::Id::new("query_find_overlay"))
        .order(egui::Order::Foreground)
        .fixed_pos(pos)
        .show(ctx, |ui| {
            egui::Frame {
                fill: context.theme.surface_elevated,
                inner_margin: egui::Margin::symmetric(8.0, 6.0),
                rounding: egui::Rounding::same(RADIUS_SM),
                stroke: egui::Stroke::new(1.0, context.theme.border_subtle),
                shadow: context.theme.floating_shadow(),
                ..Default::default()
            }
            .show(ui, |ui| {
                ui.set_width(width);
                if let Some(doc) = context.session.documents.get_mut(context.session.active_document_index) {
                    ui.horizontal(|ui| {
                        let previous_search = context.editor.editor_search.clone();
                        input(ui, &mut context.editor.editor_search, "Search…", 160.0, context.theme);
                        if context.editor.editor_search != previous_search {
                            doc.search.query = context.editor.editor_search.clone();
                            doc.search.update_matches(doc.buffer.text());
                            if let Some(first_match) = doc.search.matches.first().copied() {
                                doc.search.active_match_index = 0;
                                goto_range = Some(first_match);
                            }
                        }
                        if !context.editor.editor_search.is_empty() {
                            let total = doc.search.matches.len();
                            let current = if total == 0 {
                                0
                            } else {
                                doc.search.active_match_index + 1
                            };
                            ui.label(
                                RichText::new(if total == 0 {
                                    "0/0".to_owned()
                                } else {
                                    format!("{current}/{total}")
                                })
                                .small()
                                .color(if total == 0 {
                                    context.theme.danger
                                } else {
                                    context.theme.text_muted
                                }),
                            );
                            if Button::new(context.theme)
                                .icon(Icon::ChevronUp)
                                .variant(ButtonVariant::Ghost)
                                .size(ButtonSize::IconSm)
                                .tooltip("Previous match (Shift+Enter)")
                                .show(ui)
                                .clicked()
                            {
                                if let Some(matched_range) = doc.search.prev_match() {
                                    goto_range = Some(matched_range);
                                }
                            }
                            if Button::new(context.theme)
                                .icon(Icon::ChevronDown)
                                .variant(ButtonVariant::Ghost)
                                .size(ButtonSize::IconSm)
                                .tooltip("Next match (Enter)")
                                .show(ui)
                                .clicked()
                            {
                                if let Some(matched_range) = doc.search.next_match() {
                                    goto_range = Some(matched_range);
                                }
                            }
                        }
                        if Button::new(context.theme)
                            .icon(Icon::X)
                            .variant(ButtonVariant::Ghost)
                            .size(ButtonSize::IconSm)
                            .tooltip("Close find (Esc)")
                            .show(ui)
                            .clicked()
                        {
                            close_search = true;
                        }
                    });
                    if let Some((start, end)) = goto_range {
                        doc.cursor.set_offset(&doc.buffer, end);
                        doc.selection = crate::editor::SelectionRange::new(start, end);
                    }
                }
            });
        });

    if close_search {
        context.editor.editor_search_open = false;
        context.editor.editor_search.clear();
        if let Some(doc) = context.session.documents.get_mut(context.session.active_document_index) {
            doc.search.query.clear();
            doc.search.matches.clear();
            doc.search.active_match_index = 0;
        }
    }
}

/// Legacy full-width find bar retained for compatibility with the editor path.
#[allow(dead_code)]
pub(super) fn draw_editor_search_bar(context: &mut QuerySearchContext<'_>, ui: &mut egui::Ui) {
    ui.add_space(SPACE_SM);
    let mut goto_range = None;
    let mut close_search = false;
    if let Some(doc) = context.session.documents.get_mut(context.session.active_document_index) {
        ui.horizontal(|ui| {
            let previous_search = context.editor.editor_search.clone();
            input(
                ui,
                &mut context.editor.editor_search,
                "Find in SQL…",
                240.0,
                context.theme,
            );
            if context.editor.editor_search != previous_search {
                doc.search.query = context.editor.editor_search.clone();
                doc.search.update_matches(doc.buffer.text());
                if let Some(first_match) = doc.search.matches.first().copied() {
                    doc.search.active_match_index = 0;
                    goto_range = Some(first_match);
                }
            }
            if !context.editor.editor_search.is_empty() {
                let total = doc.search.matches.len();
                let current = if total == 0 {
                    0
                } else {
                    doc.search.active_match_index + 1
                };
                ui.label(
                    RichText::new(if total == 0 {
                        "No matches".to_owned()
                    } else {
                        format!("{current} of {total}")
                    })
                    .small()
                    .color(if total == 0 {
                        context.theme.danger
                    } else {
                        context.theme.text_muted
                    }),
                );
                if Button::new(context.theme)
                    .icon(Icon::ChevronUp)
                    .variant(ButtonVariant::Ghost)
                    .size(ButtonSize::IconSm)
                    .tooltip("Previous match (Shift+Enter)")
                    .show(ui)
                    .clicked()
                {
                    if let Some(matched_range) = doc.search.prev_match() {
                        goto_range = Some(matched_range);
                    }
                }
                if Button::new(context.theme)
                    .icon(Icon::ChevronDown)
                    .variant(ButtonVariant::Ghost)
                    .size(ButtonSize::IconSm)
                    .tooltip("Next match (Enter)")
                    .show(ui)
                    .clicked()
                {
                    if let Some(matched_range) = doc.search.next_match() {
                        goto_range = Some(matched_range);
                    }
                }
            }
            if Button::new(context.theme)
                .icon(Icon::X)
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::IconSm)
                .tooltip("Close find bar (Esc)")
                .show(ui)
                .clicked()
            {
                close_search = true;
            }
        });
        if let Some((start, end)) = goto_range {
            doc.cursor.set_offset(&doc.buffer, end);
            doc.selection = crate::editor::SelectionRange::new(start, end);
        }
    }
    if close_search {
        context.editor.editor_search_open = false;
    }
}
