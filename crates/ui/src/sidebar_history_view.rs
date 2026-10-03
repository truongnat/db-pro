//! Unified execution-history activity — spec 12 parity.
//!
//! Renders `query_history_entries` (real executed-statement records, not draft
//! history) grouped by local day with an outcome filter, per-entry details and
//! an "open as new query" handoff.
use super::*;
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
                RichText::new("0 matching filters")
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
            section_label(ui, "EXECUTIONS", self.theme);
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                egui::ComboBox::from_id_salt("history_outcome_filter")
                    .selected_text(filter.label())
                    .width(72.0)
                    .show_ui(ui, |ui| {
                        for candidate in HistoryOutcomeFilter::ALL {
                            ui.selectable_value(&mut filter, candidate, candidate.label());
                        }
                    });
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
            RichText::new("No executions yet")
                .font(font_caption())
                .strong()
                .color(self.theme.text_secondary),
        );
        ui.label(
            RichText::new("Run a query to see it here")
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
            entry.sql.lines().find(|line| !line.trim().is_empty()).unwrap_or("(empty)"),
            22,
        );
        let response = sidebar_item(ui, icon, &title, selected, self.theme);
        ui.painter().circle_filled(
            egui::pos2(response.rect.right() - 12.0, response.rect.center().y),
            3.0,
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
            .map(|conn| conn.name.as_str())
            .unwrap_or("local");
        let schema = entry.schema.as_deref().unwrap_or("—");
        let outcome = match entry.status {
            UiQueryHistoryStatus::Success => entry
                .row_count
                .map(|count| format!("{count} rows"))
                .or_else(|| entry.affected_rows.map(|count| format!("{count} affected")))
                .unwrap_or_else(|| "ok".to_owned()),
            UiQueryHistoryStatus::Failed => entry
                .error_summary
                .clone()
                .or_else(|| entry.error_code.clone())
                .unwrap_or_else(|| "error".to_owned()),
            UiQueryHistoryStatus::Cancelled => "cancelled".to_owned(),
        };
        let time = local
            .map(|stamp| stamp.format("%H:%M").to_string())
            .unwrap_or_else(|| "—".to_owned());
        truncate(&format!("{connection} · {schema} · {outcome} · {time} · {} ms", entry.duration_ms), 44)
    }

    fn draw_details(
        &self,
        ui: &mut egui::Ui,
        entry: &UiQueryHistoryEntry,
        local: Option<chrono::DateTime<chrono::Local>>,
        actions: &mut Vec<SidebarHistoryAction>,
    ) {
        card_frame(self.theme).show(ui, |ui| {
            ui.label(
                RichText::new(&entry.id)
                    .font(font_mono_sm())
                    .color(self.theme.text_muted),
            );
            if let Some(stamp) = local {
                ui.label(
                    RichText::new(stamp.format("Started: %Y-%m-%d %H:%M:%S").to_string())
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
            egui::ScrollArea::vertical().max_height(96.0).show(ui, |ui| {
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
                if compact_button_with_icon(ui, Icon::ExternalLink, "Open as new query", self.theme).clicked() {
                    actions.push(SidebarHistoryAction::OpenAsNewQuery(entry.sql.clone()));
                }
                if compact_button_with_icon(ui, Icon::Copy, "Copy SQL", self.theme).clicked() {
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
        Some(day) if day == today => format!("TODAY — {day}"),
        Some(day) if day == today.pred_opt().unwrap_or(day) => format!("YESTERDAY — {day}"),
        Some(day) => day.to_string(),
        None => "UNDATED".to_owned(),
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
mod tests {
    use super::*;

    #[test]
    fn day_groups_use_local_calendar() {
        let today = chrono::Local::now().date_naive();
        assert!(day_label(Some(today), today).starts_with("TODAY"));
        let yesterday = today.pred_opt().unwrap();
        assert!(day_label(Some(yesterday), today).starts_with("YESTERDAY"));
        assert_eq!(day_label(None, today), "UNDATED");
    }

    #[test]
    fn truncate_marks_long_text() {
        assert_eq!(truncate("short", 10), "short");
        assert!(truncate("a much longer line of text", 10).ends_with('…'));
    }
}
