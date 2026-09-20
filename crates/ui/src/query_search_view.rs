use super::*;

impl DbProApp {
    pub(super) fn draw_editor_search_overlay(&mut self, ctx: &egui::Context) {
        if !self.query.editor.editor_search_open {
            return;
        }
        let editor_rect = self.query.editor.query_editor_rect;
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
                    fill: self.theme.surface_elevated,
                    inner_margin: egui::Margin::symmetric(8.0, 6.0),
                    rounding: egui::Rounding::same(RADIUS_SM),
                    stroke: egui::Stroke::new(1.0, self.theme.border_subtle),
                    shadow: self.theme.floating_shadow(),
                    ..Default::default()
                }
                .show(ui, |ui| {
                    ui.set_width(width);
                    if let Some(doc) = self
                        .query
                        .session
                        .documents
                        .get_mut(self.query.session.active_document_index)
                    {
                        ui.horizontal(|ui| {
                            let prev_search = self.query.editor.editor_search.clone();
                            input(ui, &mut self.query.editor.editor_search, "Search…", 160.0, self.theme);
                            if self.query.editor.editor_search != prev_search {
                                doc.search.query = self.query.editor.editor_search.clone();
                                doc.search.update_matches(doc.buffer.text());
                                if let Some(first_match) = doc.search.matches.first().copied() {
                                    doc.search.active_match_index = 0;
                                    goto_range = Some(first_match);
                                }
                            }
                            if !self.query.editor.editor_search.is_empty() {
                                let total = doc.search.matches.len();
                                let current = if total == 0 {
                                    0
                                } else {
                                    doc.search.active_match_index + 1
                                };
                                let label_text = if total == 0 {
                                    "0/0".to_string()
                                } else {
                                    format!("{current}/{total}")
                                };
                                ui.label(RichText::new(label_text).small().color(if total == 0 {
                                    self.theme.danger
                                } else {
                                    self.theme.text_muted
                                }));
                                if Button::new(self.theme)
                                    .icon(Icon::ChevronUp)
                                    .variant(ButtonVariant::Ghost)
                                    .size(ButtonSize::IconSm)
                                    .tooltip("Previous match (Shift+Enter)")
                                    .show(ui)
                                    .clicked()
                                {
                                    if let Some(m) = doc.search.prev_match() {
                                        goto_range = Some(m);
                                    }
                                }
                                if Button::new(self.theme)
                                    .icon(Icon::ChevronDown)
                                    .variant(ButtonVariant::Ghost)
                                    .size(ButtonSize::IconSm)
                                    .tooltip("Next match (Enter)")
                                    .show(ui)
                                    .clicked()
                                {
                                    if let Some(m) = doc.search.next_match() {
                                        goto_range = Some(m);
                                    }
                                }
                            }
                            if Button::new(self.theme)
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
            self.query.editor.editor_search_open = false;
            self.query.editor.editor_search.clear();
            if let Some(doc) = self
                .query
                .session
                .documents
                .get_mut(self.query.session.active_document_index)
            {
                doc.search.query.clear();
                doc.search.matches.clear();
                doc.search.active_match_index = 0;
            }
        }
    }

    /// Legacy full-width find bar retained for compatibility with the editor path.
    #[allow(dead_code)]
    fn draw_editor_search_bar(&mut self, ui: &mut egui::Ui) {
        ui.add_space(SPACE_SM);
        let mut goto_range = None;
        let mut close_search = false;
        if let Some(doc) = self
            .query
            .session
            .documents
            .get_mut(self.query.session.active_document_index)
        {
            ui.horizontal(|ui| {
                let prev_search = self.query.editor.editor_search.clone();
                input(
                    ui,
                    &mut self.query.editor.editor_search,
                    "Find in SQL…",
                    240.0,
                    self.theme,
                );
                if self.query.editor.editor_search != prev_search {
                    doc.search.query = self.query.editor.editor_search.clone();
                    doc.search.update_matches(doc.buffer.text());
                    if let Some(first_match) = doc.search.matches.first().copied() {
                        doc.search.active_match_index = 0;
                        goto_range = Some(first_match);
                    }
                }
                if !self.query.editor.editor_search.is_empty() {
                    let total = doc.search.matches.len();
                    let current = if total == 0 {
                        0
                    } else {
                        doc.search.active_match_index + 1
                    };
                    let label_text = if total == 0 {
                        "No matches".to_string()
                    } else {
                        format!("{current} of {total}")
                    };
                    ui.label(RichText::new(label_text).small().color(if total == 0 {
                        self.theme.danger
                    } else {
                        self.theme.text_muted
                    }));
                    if Button::new(self.theme)
                        .icon(Icon::ChevronUp)
                        .variant(ButtonVariant::Ghost)
                        .size(ButtonSize::IconSm)
                        .tooltip("Previous match (Shift+Enter)")
                        .show(ui)
                        .clicked()
                    {
                        if let Some(m) = doc.search.prev_match() {
                            goto_range = Some(m);
                        }
                    }
                    if Button::new(self.theme)
                        .icon(Icon::ChevronDown)
                        .variant(ButtonVariant::Ghost)
                        .size(ButtonSize::IconSm)
                        .tooltip("Next match (Enter)")
                        .show(ui)
                        .clicked()
                    {
                        if let Some(m) = doc.search.next_match() {
                            goto_range = Some(m);
                        }
                    }
                }
                if Button::new(self.theme)
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
            self.query.editor.editor_search_open = false;
        }
    }
}
