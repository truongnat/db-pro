// cc-scan:allow-file HUGE_FUNCTION,LONG_FUNCTION,HARD_COMPLEXITY,COMPLEXITY,DEEP_NESTING
// cc-scan:allow-file HARD_PARAMS,TOO_MANY_PARAMS,DUPLICATE_BLOCK
// egui painter/view file: fns are linear layout code; branches are per-state paint variants.
use super::config::*;
use super::handler::{
    apply_destructive_dialog_action, confirmation_is_valid, destructive_dialog_action, pending_mutation_suffix,
    status_text, transaction_action, transaction_status, TransactionStatus,
};
use super::TransactionAction;
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::components::dialog::Dialog;
use crate::components::input::Input;
use crate::DbProTheme;
use egui::{CornerRadius, FontFamily, FontId, Pos2, Rect, RichText, Sense, Stroke, Ui, Vec2};
use lucide_icons::Icon;

// ── Transaction Bar ──────────────────────────────────────────────────────────

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
        let status = transaction_status(self.auto_commit, self.in_transaction, self.pending_mutations);
        let status_text = status_text(status);
        let status_color = match status {
            TransactionStatus::AutoCommit => self.theme.info,
            TransactionStatus::Uncommitted => self.theme.warning,
            TransactionStatus::Active => self.theme.success,
            TransactionStatus::Inactive => self.theme.text_tertiary,
        };
        let mut rollback_clicked = false;
        let mut commit_clicked = false;
        let mut begin_clicked = false;

        let frame = egui::Frame::NONE
            .fill(self.theme.surface_panel)
            .stroke(Stroke::new(BAR_STROKE_WIDTH, self.theme.border_default))
            .corner_radius(CornerRadius::same(BAR_RADIUS as u8))
            .inner_margin(egui::Margin::symmetric(BAR_MARGIN_X as i8, BAR_MARGIN_Y as i8));
        frame.show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                let (dot_rect, _) = ui.allocate_exact_size(Vec2::splat(STATUS_DOT_SIZE), Sense::hover());
                ui.painter()
                    .circle_filled(dot_rect.center(), STATUS_DOT_RADIUS, status_color);
                ui.add_space(STATUS_DOT_GAP);
                ui.label(
                    RichText::new(status_text)
                        .size(STATUS_TEXT_SIZE)
                        .strong()
                        .color(self.theme.text_primary),
                );

                if self.pending_mutations > 0 {
                    ui.add_space(PENDING_BEFORE_GAP);
                    let badge_text = format!(
                        "{} pending change{}",
                        self.pending_mutations,
                        pending_mutation_suffix(self.pending_mutations),
                    );
                    let badge_galley = ui.painter().layout_no_wrap(
                        badge_text,
                        FontId::proportional(PENDING_BADGE_TEXT_SIZE),
                        self.theme.warning,
                    );
                    let badge_rect = Rect::from_min_size(
                        Pos2::new(ui.cursor().min.x, ui.cursor().min.y + PENDING_BADGE_TOP_OFFSET),
                        Vec2::new(badge_galley.size().x + PENDING_BADGE_PADDING_X, PENDING_BADGE_HEIGHT),
                    );
                    ui.painter().rect_filled(
                        badge_rect,
                        CornerRadius::same(PENDING_BADGE_RADIUS as u8),
                        self.theme.warning_soft(),
                    );
                    ui.painter().rect_stroke(
                        badge_rect,
                        CornerRadius::same(PENDING_BADGE_RADIUS as u8),
                        Stroke::new(BAR_STROKE_WIDTH, self.theme.warning.linear_multiply(0.4)),
                        egui::StrokeKind::Inside,
                    );
                    ui.painter().galley(
                        Pos2::new(
                            badge_rect.left() + PENDING_BADGE_TEXT_OFFSET_X,
                            badge_rect.top() + PENDING_BADGE_TEXT_OFFSET_Y,
                        ),
                        badge_galley,
                        self.theme.warning,
                    );
                    ui.add_space(badge_rect.width() + PENDING_AFTER_GAP);
                }

                ui.add_space(ISOLATION_GAP);
                ui.label(
                    RichText::new(format!("[{}]", self.isolation_level))
                        .size(ISOLATION_TEXT_SIZE)
                        .monospace()
                        .color(self.theme.text_tertiary),
                );

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if self.in_transaction {
                        rollback_clicked = Button::new(self.theme)
                            .text("Rollback")
                            .icon(Icon::Undo2)
                            .variant(ButtonVariant::Destructive)
                            .size(ButtonSize::Sm)
                            .show(ui)
                            .clicked();
                        ui.add_space(ACTION_GAP);
                        commit_clicked = Button::new(self.theme)
                            .text("Commit")
                            .icon(Icon::Check)
                            .variant(ButtonVariant::Default)
                            .size(ButtonSize::Sm)
                            .show(ui)
                            .clicked();
                    } else if !self.auto_commit {
                        begin_clicked = Button::new(self.theme)
                            .text("Begin Transaction")
                            .icon(Icon::Play)
                            .variant(ButtonVariant::Outline)
                            .size(ButtonSize::Sm)
                            .show(ui)
                            .clicked();
                    }
                });
            });
        });

        transaction_action(
            self.in_transaction,
            self.auto_commit,
            rollback_clicked,
            commit_clicked,
            begin_clicked,
        )
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
        let mut confirm_clicked = false;
        let mut cancel_clicked = false;
        let is_keyword_valid = confirmation_is_valid(self.input_keyword, self.required_keyword);

        let title = self.title;
        let warning_text = self.warning_text;
        let target_object = self.target_object;
        let required_keyword = self.required_keyword;
        let theme = self.theme;

        Dialog::new(self.open, title, theme).width(DIALOG_WIDTH).show(ui, |ui| {
            ui.add_space(DIALOG_TOP_SPACE);

            // Keep the destructive warning visually separate so users see the irreversible-action risk first.
            let banner_frame = egui::Frame::NONE
                .fill(theme.danger_soft())
                .stroke(Stroke::new(
                    DIALOG_BANNER_STROKE_WIDTH,
                    theme.danger.linear_multiply(0.4),
                ))
                .corner_radius(CornerRadius::same(DIALOG_BANNER_RADIUS as u8))
                .inner_margin(egui::Margin::same(DIALOG_BANNER_MARGIN as i8));

            banner_frame.show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(char::from(Icon::AlertTriangle).to_string())
                            .font(FontId::new(DIALOG_ICON_SIZE, FontFamily::Name("lucide".into())))
                            .color(theme.danger),
                    );
                    ui.add_space(DIALOG_ICON_GAP);
                    ui.vertical(|ui| {
                        ui.label(
                            RichText::new("Irreversible Action Warning")
                                .size(DIALOG_TITLE_SIZE)
                                .strong()
                                .color(theme.danger),
                        );
                        ui.add_space(DIALOG_TITLE_GAP);
                        ui.label(
                            RichText::new(warning_text)
                                .size(DIALOG_WARNING_SIZE)
                                .color(theme.text_secondary),
                        );
                    });
                });
            });

            ui.add_space(DIALOG_SECTION_GAP);

            // Target Object Info
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("Target Object:")
                        .size(DIALOG_LABEL_SIZE)
                        .strong()
                        .color(theme.text_secondary),
                );
                ui.add_space(DIALOG_TARGET_GAP);
                ui.label(
                    RichText::new(target_object)
                        .monospace()
                        .size(DIALOG_LABEL_SIZE)
                        .color(theme.text_primary),
                );
            });

            ui.add_space(DIALOG_SECTION_GAP);

            // Confirmation input instructions
            ui.label(
                RichText::new(format!("To confirm, type \"{}\" in the box below:", required_keyword))
                    .size(DIALOG_LABEL_SIZE)
                    .color(theme.text_secondary),
            );

            ui.add_space(DIALOG_INPUT_GAP);

            Input::new(self.input_keyword, required_keyword, theme).show(ui);

            ui.add_space(DIALOG_ACTION_GAP);

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
                        confirm_clicked = true;
                    }

                    ui.add_space(DIALOG_BUTTON_GAP);

                    if Button::new(theme)
                        .text("Cancel")
                        .variant(ButtonVariant::Ghost)
                        .show(ui)
                        .clicked()
                    {
                        cancel_clicked = true;
                    }
                });
            });
        });

        let action = destructive_dialog_action(cancel_clicked, confirm_clicked, is_keyword_valid);
        apply_destructive_dialog_action(self.open, action)
    }
}
