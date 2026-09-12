use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::components::dialog::Dialog;
use crate::components::input::Input;
use crate::DbProTheme;
use egui::{Color32, FontFamily, FontId, Pos2, Rect, RichText, Rounding, Sense, Stroke, Ui, Vec2};
use lucide_icons::Icon;

// ── Transaction Bar ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionAction {
    Commit,
    Rollback,
    Begin,
    ToggleAutoCommit(bool),
}

pub struct TransactionBar<'a> {
    in_transaction: bool,
    pending_mutations: usize,
    auto_commit: bool,
    isolation_level: &'a str,
    theme: DbProTheme,
}

impl<'a> TransactionBar<'a> {
    pub fn new(in_transaction: bool, pending_mutations: usize, theme: DbProTheme) -> Self {
        Self {
            in_transaction,
            pending_mutations,
            auto_commit: false,
            isolation_level: "READ COMMITTED",
            theme,
        }
    }

    pub fn auto_commit(mut self, auto_commit: bool) -> Self {
        self.auto_commit = auto_commit;
        self
    }

    pub fn isolation_level(mut self, isolation_level: &'a str) -> Self {
        self.isolation_level = isolation_level;
        self
    }

    pub fn show(self, ui: &mut Ui) -> Option<TransactionAction> {
        let mut triggered_action = None;

        let frame = egui::Frame::none()
            .fill(self.theme.surface_panel)
            .stroke(Stroke::new(1.0, self.theme.border_default))
            .rounding(Rounding::same(8.0))
            .inner_margin(egui::Margin::symmetric(12.0, 8.0));

        frame.show(ui, |ui| {
            ui.set_width(ui.available_width());

            ui.horizontal(|ui| {
                // Status dot indicator
                let (dot_color, status_text) = if self.auto_commit {
                    (self.theme.info, "Auto-commit Active")
                } else if self.in_transaction {
                    if self.pending_mutations > 0 {
                        (self.theme.warning, "Uncommitted Transaction")
                    } else {
                        (self.theme.success, "Transaction Active")
                    }
                } else {
                    (self.theme.text_tertiary, "No Active Transaction")
                };

                let (dot_rect, _) = ui.allocate_exact_size(Vec2::new(10.0, 10.0), Sense::hover());
                ui.painter().circle_filled(dot_rect.center(), 4.0, dot_color);
                ui.add_space(4.0);

                ui.label(
                    RichText::new(status_text)
                        .size(12.5)
                        .strong()
                        .color(self.theme.text_primary),
                );

                // Pending mutations badge
                if self.pending_mutations > 0 {
                    ui.add_space(8.0);
                    let badge_text = format!(
                        "{} pending change{}",
                        self.pending_mutations,
                        if self.pending_mutations > 1 { "s" } else { "" }
                    );
                    let badge_galley =
                        ui.painter()
                            .layout_no_wrap(badge_text, FontId::proportional(11.0), self.theme.warning);
                    let badge_rect = Rect::from_min_size(
                        Pos2::new(ui.cursor().min.x, ui.cursor().min.y + 2.0),
                        Vec2::new(badge_galley.size().x + 12.0, 18.0),
                    );
                    let fill = self.theme.warning_soft();
                    ui.painter().rect_filled(badge_rect, Rounding::same(9.0), fill);
                    ui.painter().rect_stroke(
                        badge_rect,
                        Rounding::same(9.0),
                        Stroke::new(1.0, self.theme.warning.linear_multiply(0.4)),
                    );
                    ui.painter().galley(
                        Pos2::new(badge_rect.left() + 6.0, badge_rect.top() + 2.0),
                        badge_galley,
                        Color32::PLACEHOLDER,
                    );
                    ui.add_space(badge_rect.width() + 4.0);
                }

                // Isolation level tag
                ui.add_space(6.0);
                ui.label(
                    RichText::new(format!("[{}]", self.isolation_level))
                        .size(11.0)
                        .monospace()
                        .color(self.theme.text_tertiary),
                );

                // Actions on the right
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if self.in_transaction {
                        // Rollback button
                        if Button::new(self.theme)
                            .text("Rollback")
                            .icon(Icon::Undo2)
                            .variant(ButtonVariant::Destructive)
                            .size(ButtonSize::Sm)
                            .show(ui)
                            .clicked()
                        {
                            triggered_action = Some(TransactionAction::Rollback);
                        }

                        ui.add_space(4.0);

                        // Commit button
                        if Button::new(self.theme)
                            .text("Commit")
                            .icon(Icon::Check)
                            .variant(ButtonVariant::Default)
                            .size(ButtonSize::Sm)
                            .show(ui)
                            .clicked()
                        {
                            triggered_action = Some(TransactionAction::Commit);
                        }
                    } else if !self.auto_commit
                        && Button::new(self.theme)
                            .text("Begin Transaction")
                            .icon(Icon::Play)
                            .variant(ButtonVariant::Outline)
                            .size(ButtonSize::Sm)
                            .show(ui)
                            .clicked()
                    {
                        triggered_action = Some(TransactionAction::Begin);
                    }
                });
            });
        });

        triggered_action
    }
}

// ── Destructive Operation Dialog ─────────────────────────────────────────────

pub struct DestructiveOperationDialog<'a> {
    open: &'a mut bool,
    title: &'a str,
    warning_text: &'a str,
    target_object: &'a str,
    required_keyword: &'a str,
    input_keyword: &'a mut String,
    theme: DbProTheme,
}

impl<'a> DestructiveOperationDialog<'a> {
    pub fn new(
        open: &'a mut bool,
        title: &'a str,
        warning_text: &'a str,
        target_object: &'a str,
        required_keyword: &'a str,
        input_keyword: &'a mut String,
        theme: DbProTheme,
    ) -> Self {
        Self {
            open,
            title,
            warning_text,
            target_object,
            required_keyword,
            input_keyword,
            theme,
        }
    }

    pub fn show(self, ui: &mut Ui) -> bool {
        let mut confirmed = false;
        let is_keyword_valid = self.input_keyword.trim() == self.required_keyword;

        let title = self.title;
        let warning_text = self.warning_text;
        let target_object = self.target_object;
        let required_keyword = self.required_keyword;
        let theme = self.theme;

        Dialog::new(self.open, title, theme).width(480.0).show(ui, |ui| {
            ui.add_space(4.0);

            // Danger Banner Frame
            let banner_frame = egui::Frame::none()
                .fill(theme.danger_soft())
                .stroke(Stroke::new(1.0, theme.danger.linear_multiply(0.4)))
                .rounding(Rounding::same(8.0))
                .inner_margin(egui::Margin::same(12.0));

            banner_frame.show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(char::from(Icon::AlertTriangle).to_string())
                            .font(FontId::new(16.0, FontFamily::Name("lucide".into())))
                            .color(theme.danger),
                    );
                    ui.add_space(6.0);
                    ui.vertical(|ui| {
                        ui.label(
                            RichText::new("Irreversible Action Warning")
                                .size(13.0)
                                .strong()
                                .color(theme.danger),
                        );
                        ui.add_space(2.0);
                        ui.label(RichText::new(warning_text).size(12.0).color(theme.text_secondary));
                    });
                });
            });

            ui.add_space(14.0);

            // Target Object Info
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("Target Object:")
                        .size(12.5)
                        .strong()
                        .color(theme.text_secondary),
                );
                ui.add_space(4.0);
                ui.label(
                    RichText::new(target_object)
                        .monospace()
                        .size(12.5)
                        .color(theme.text_primary),
                );
            });

            ui.add_space(14.0);

            // Confirmation input instructions
            ui.label(
                RichText::new(format!("To confirm, type \"{}\" in the box below:", required_keyword))
                    .size(12.5)
                    .color(theme.text_secondary),
            );

            ui.add_space(6.0);

            Input::new(self.input_keyword, required_keyword, theme).show(ui);

            ui.add_space(18.0);

            // Action buttons
            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if Button::new(theme)
                        .text("Permanently Execute")
                        .variant(ButtonVariant::Destructive)
                        .enabled(is_keyword_valid)
                        .show(ui)
                        .clicked()
                    {
                        confirmed = true;
                    }

                    ui.add_space(8.0);

                    if Button::new(theme)
                        .text("Cancel")
                        .variant(ButtonVariant::Ghost)
                        .show(ui)
                        .clicked()
                    {
                        // handled by Dialog closing
                    }
                });
            });
        });

        if confirmed {
            *self.open = false;
        }

        confirmed
    }
}
