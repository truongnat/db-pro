//! Unified execution-history activity — spec 12 parity.
//!
//! Renders `query_history_entries` (real executed-statement records, not draft
//! history) grouped by local day with an outcome filter, per-entry details and
//! an "open as new query" handoff.
use super::*;
use super::sidebar_constants::{DETAILS_MAX_HEIGHT, ENTRY_TITLE_MAX_CHARS, FILTER_SELECT_WIDTH, META_LINE_MAX_CHARS, STATUS_DOT_INSET, STATUS_DOT_RADIUS};
use egui::{Align, Layout};
use lucide_icons::Icon;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum SidebarHistoryAction {
    SetOutcomeFilter(HistoryOutcomeFilter),
    Select(Option<String>),
    OpenAsNewQuery(String),
    CopySql(String),
}

pub(super) struct SidebarHistoryContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) entries: &'a [UiQueryHistoryEntry],
    pub(super) catalog: &'a ConnectionCatalogState,
    pub(super) outcome_filter: HistoryOutcomeFilter,
    pub(super) selected_id: Option<&'a str>,
}

impl SidebarHistoryContext<'_> {
    pub(super) fn draw(&self, ui: &mut egui::Ui) -> Vec<SidebarHistoryAction> {
        let mut actions = self.draw_outcome_filter(ui);
        ui.add_space(SPACE_XS);
        if self.entries.is_empty() {
            self.draw_empty(ui);
            return actions;
        }
        let today = chrono::Local::now().date_naive();
        let mut last_day: Option<chrono::NaiveDate> = None;
        let mut shown = 0usize;
        // entries are stored oldest-first; rev() puts newest on top
        for entry in self.entries.iter().rev() {
            if !self.matches_outcome(entry) {
                continue;
            }
            let local = parse_started_at(entry);
            let day = local.map(|stamp| stamp.date_naive());
            if day != last_day {
                last_day = day;
                ui.add_space(SPACE_XS);
                section_label(ui, day_label(day, today), self.theme);
            }
            actions.extend(self.draw_entry(ui, entry, local));
            shown += 1;
        }
        if shown == 0 {
            ui.label(
                RichText::new(t!("sidebar.no_matching_filters"))
                    .font(font_caption())
                    .color(self.theme.text_muted),
            );
        }
        actions
    }

    fn draw_outcome_filter(&self, ui: &mut egui::Ui) -> Vec<SidebarHistoryAction> {
        let mut actions = Vec::new();
        let mut filter = self.outcome_filter;
        ui.horizontal(|ui| {
            section_label(ui, t!("sidebar.executions").as_ref(), self.theme);
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let labels: Vec<String> = HistoryOutcomeFilter::ALL
                    .iter()
                    .map(|candidate| t!(candidate.locale_key()).into_owned())
                    .collect();
                let mut selected = HistoryOutcomeFilter::ALL
                    .iter()
                    .position(|candidate| *candidate == self.outcome_filter)
                    .unwrap_or(0);
                let previous = selected;
                crate::components::Select::new("history_outcome_filter", &mut selected, &labels)
                    .theme(self.theme)
                    .width(FILTER_SELECT_WIDTH)
                    .size(crate::components::SelectSize::Sm)
                    .variant(crate::components::SelectVariant::Ghost)
                    .show(ui);
                if selected != previous {
                    filter = HistoryOutcomeFilter::ALL[selected];
                }
            });
        });
        if filter != self.outcome_filter {
            actions.push(SidebarHistoryAction::SetOutcomeFilter(filter));
        }
        actions
    }

    fn draw_empty(&self, ui: &mut egui::Ui) {
        ui.add_space(SPACE_SM);
        ui.label(
            RichText::new(t!("sidebar.no_executions"))
                .font(font_caption())
                .strong()
                .color(self.theme.text_secondary),
        );
        ui.label(
            RichText::new(t!("sidebar.run_to_see"))
                .font(font_caption())
                .color(self.theme.text_muted),
        );
    }

    fn matches_outcome(&self, entry: &UiQueryHistoryEntry) -> bool {
        match self.outcome_filter {
            HistoryOutcomeFilter::All => true,
            HistoryOutcomeFilter::Success => entry.status == UiQueryHistoryStatus::Success,
            HistoryOutcomeFilter::Failed => entry.status == UiQueryHistoryStatus::Failed,
            HistoryOutcomeFilter::Cancelled => entry.status == UiQueryHistoryStatus::Cancelled,
        }
    }

    fn draw_entry(
        &self,
        ui: &mut egui::Ui,
        entry: &UiQueryHistoryEntry,
        local: Option<chrono::DateTime<chrono::Local>>,
    ) -> Vec<SidebarHistoryAction> {
        let mut actions = Vec::new();
        let selected = self.selected_id == Some(entry.id.as_str());
        let (icon, icon_color) = outcome_style(entry.status, self.theme);
        let title = truncate(
            entry.sql.lines().find(|line| !line.trim().is_empty()).unwrap_or(t!("sidebar.empty_sql").as_ref()),
            ENTRY_TITLE_MAX_CHARS,
        );
        let response = sidebar_item(ui, icon, &title, selected, self.theme);
        ui.painter().circle_filled(
            egui::pos2(response.rect.right() - STATUS_DOT_INSET, response.rect.center().y),
            STATUS_DOT_RADIUS,
            icon_color,
        );
        let meta = self.meta_line(entry, local);
        ui.label(
            RichText::new(meta)
                .font(font_caption())
                .color(self.theme.text_muted),
        );
        let is_context_menu = is_context_menu_triggered(&response, ui);
        if response.clicked() && !is_context_menu {
            let next = if selected { None } else { Some(entry.id.clone()) };
            actions.push(SidebarHistoryAction::Select(next));
        }
        if selected {
            self.draw_details(ui, entry, local, &mut actions);
        }
        actions
    }

    fn meta_line(&self, entry: &UiQueryHistoryEntry, local: Option<chrono::DateTime<chrono::Local>>) -> String {
        let connection = entry
            .connection_id
            .as_deref()
            .and_then(|id| self.catalog.find(id))
            .map(|conn| conn.name.as_str()).unwrap_or("local");
        let schema = entry.schema.as_deref().unwrap_or("—");
        let outcome = match entry.status {
            UiQueryHistoryStatus::Success => entry
                .row_count
                .map(|count| t!("sidebar.rows_count", n = count).into_owned())
                .or_else(|| entry.affected_rows.map(|count| t!("sidebar.affected_count", n = count).into_owned()))
                .unwrap_or_else(|| t!("sidebar.outcome_ok").into_owned()),
            UiQueryHistoryStatus::Failed => entry
                .error_summary
                .clone()
                .or_else(|| entry.error_code.clone())
                .unwrap_or_else(|| t!("sidebar.outcome_error").into_owned()),
            UiQueryHistoryStatus::Cancelled => t!("sidebar.outcome_cancelled").into_owned(),
        };
        let time = local
            .map(|stamp| stamp.format("%H:%M").to_string())
            .unwrap_or_else(|| "—".to_owned());
        truncate(&format!("{connection} · {schema} · {outcome} · {time} · {} ms", entry.duration_ms), META_LINE_MAX_CHARS)
    }

    fn draw_details(
        &self,
        ui: &mut egui::Ui,
        entry: &UiQueryHistoryEntry,
        local: Option<chrono::DateTime<chrono::Local>>,
        actions: &mut Vec<SidebarHistoryAction>,
    ) {
        Card::new(self.theme).show(ui, |ui| {
            ui.label(
                RichText::new(&entry.id)
                    .font(font_mono_sm())
                    .color(self.theme.text_muted),
            );
            if let Some(stamp) = local {
                ui.label(
                    RichText::new(t!("sidebar.started_at", at = stamp.format("%Y-%m-%d %H:%M:%S").to_string().as_str()).into_owned())
                        .font(font_caption())
                        .color(self.theme.text_muted),
                );
            }
            if let Some(summary) = &entry.error_summary {
                ui.label(
                    RichText::new(summary)
                        .font(font_caption())
                        .color(self.theme.danger),
                );
            }
            ui.add_space(SPACE_XXS);
            ScrollArea::new(self.theme).max_height(DETAILS_MAX_HEIGHT).show(ui, |ui| {
                ui.add(
                    egui::Label::new(
                        RichText::new(&entry.sql)
                            .font(font_mono_sm())
                            .color(self.theme.text_secondary),
                    )
                    .selectable(true),
                );
            });
            ui.horizontal(|ui| {
                if Button::new(self.theme)
                    .icon(Icon::ExternalLink)
                    .text(t!("sidebar.open_as_new").as_ref())
                    .variant(ButtonVariant::Secondary)
                    .size(ButtonSize::Sm)
                    .show(ui)
                    .clicked()
                {
                    actions.push(SidebarHistoryAction::OpenAsNewQuery(entry.sql.clone()));
                }
                if Button::new(self.theme)
                    .icon(Icon::Copy)
                    .text(t!("compare.copy_sql"))
                    .variant(ButtonVariant::Ghost)
                    .size(ButtonSize::Sm)
                    .show(ui)
                    .clicked()
                {
                    actions.push(SidebarHistoryAction::CopySql(entry.sql.clone()));
                }
            });
        });
    }
}

fn parse_started_at(entry: &UiQueryHistoryEntry) -> Option<chrono::DateTime<chrono::Local>> {
    chrono::DateTime::parse_from_rfc3339(&entry.started_at)
        .ok()
        .map(|stamp| stamp.with_timezone(&chrono::Local))
}

fn day_label(day: Option<chrono::NaiveDate>, today: chrono::NaiveDate) -> String {
    match day {
        Some(day) if day == today => t!("sidebar.day_today", date = day.to_string().as_str()).into_owned(),
        Some(day) if day == today.pred_opt().unwrap_or(day) => {
            t!("sidebar.day_yesterday", date = day.to_string().as_str()).into_owned()
        }
        Some(day) => day.to_string(),
        None => t!("sidebar.day_undated").into_owned(),
    }
}

fn outcome_style(status: UiQueryHistoryStatus, theme: DbProTheme) -> (Icon, egui::Color32) {
    match status {
        UiQueryHistoryStatus::Success => (Icon::CheckCircle2, theme.success),
        UiQueryHistoryStatus::Failed => (Icon::CircleX, theme.danger),
        UiQueryHistoryStatus::Cancelled => (Icon::Ban, theme.text_muted),
    }
}

fn truncate(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_owned();
    }
    let mut truncated: String = text.chars().take(max_chars.saturating_sub(1)).collect();
    truncated.push('…');
    truncated
}

#[cfg(test)]
#[path = "tests/sidebar_history_view_tests.rs"]
mod tests;

