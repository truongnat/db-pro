//! Diagnostics/problems sidebar rendering and intent mapping.
use super::*;
use egui::{Align, Layout, RichText};
use lucide_icons::Icon;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum SidebarProblemsAction {
    SetSeverityFilter(ProblemsSeverityFilter),
    SetSourceFilter(ProblemsSourceFilter),
    Select(ProblemEntry),
    QuickFix {
        document_index: usize,
        diagnostic_index: usize,
    },
}

pub(super) struct SidebarProblemsContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) entries: &'a [ProblemEntry],
    pub(super) severity_filter: ProblemsSeverityFilter,
    pub(super) source_filter: ProblemsSourceFilter,
    pub(super) selected: Option<(&'a str, usize)>,
}

impl SidebarProblemsContext<'_> {
    pub(super) fn draw(&self, ui: &mut egui::Ui) -> Vec<SidebarProblemsAction> {
        let mut actions = self.draw_header(ui);
        actions.extend(self.draw_filters(ui));
        ui.add_space(8.0);

        let filtered: Vec<&ProblemEntry> = self
            .entries
            .iter()
            .filter(|entry| matches_filters(entry, self.severity_filter, self.source_filter))
            .collect();
        if filtered.is_empty() {
            self.draw_empty(ui);
            return actions;
        }

        let mut last_document: Option<usize> = None;
        for entry in filtered {
            if last_document != Some(entry.document_index) {
                last_document = Some(entry.document_index);
                ui.add_space(6.0);
                ui.label(
                    RichText::new(&entry.document_title)
                        .small()
                        .strong()
                        .color(self.theme.text_muted),
                );
            }
            actions.extend(self.draw_entry(ui, entry));
        }
        actions
    }

    fn draw_header(&self, ui: &mut egui::Ui) -> Vec<SidebarProblemsAction> {
        let error_count = self
            .entries
            .iter()
            .filter(|entry| entry.severity == crate::editor::DiagnosticSeverity::Error)
            .count();
        let warning_count = self
            .entries
            .iter()
            .filter(|entry| entry.severity == crate::editor::DiagnosticSeverity::Warning)
            .count();
        ui.horizontal(|ui| {
            section_label(ui, "PROBLEMS", self.theme);
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                badge(
                    ui,
                    &format!("{error_count}E · {warning_count}W"),
                    self.theme.surface_hover,
                    self.theme.text_muted,
                );
            });
        });
        Vec::new()
    }

    fn draw_filters(&self, ui: &mut egui::Ui) -> Vec<SidebarProblemsAction> {
        let mut actions = Vec::new();
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 6.0;
            let mut severity = self.severity_filter;
            egui::ComboBox::from_id_salt("problems_severity_filter")
                .selected_text(severity_label(severity))
                .width(96.0)
                .show_ui(ui, |ui| {
                    for (filter, label) in [
                        (ProblemsSeverityFilter::All, "All"),
                        (ProblemsSeverityFilter::Errors, "Errors"),
                        (ProblemsSeverityFilter::Warnings, "Warnings"),
                    ] {
                        ui.selectable_value(&mut severity, filter, label);
                    }
                });
            if severity != self.severity_filter {
                actions.push(SidebarProblemsAction::SetSeverityFilter(severity));
            }

            let mut source = self.source_filter;
            egui::ComboBox::from_id_salt("problems_source_filter")
                .selected_text(source_label(source))
                .width(120.0)
                .show_ui(ui, |ui| {
                    for (filter, label) in [
                        (ProblemsSourceFilter::All, "All sources"),
                        (ProblemsSourceFilter::Parser, "Parser"),
                        (ProblemsSourceFilter::Lint, "Lint"),
                        (ProblemsSourceFilter::Delimiter, "Delimiter"),
                        (ProblemsSourceFilter::Database, "Database"),
                    ] {
                        ui.selectable_value(&mut source, filter, label);
                    }
                });
            if source != self.source_filter {
                actions.push(SidebarProblemsAction::SetSourceFilter(source));
            }
        });
        actions
    }

    fn draw_empty(&self, ui: &mut egui::Ui) {
        ui.add_space(24.0);
        ui.vertical_centered(|ui| {
            ui.label(icon_text(Icon::TriangleAlert, "", self.theme.text_muted));
            ui.add_space(8.0);
            ui.label(RichText::new("No problems in open documents").color(self.theme.text_secondary));
            ui.label(
                RichText::new("Lint, parser, and execution diagnostics appear here.")
                    .small()
                    .color(self.theme.text_muted),
            );
        });
    }

    fn draw_entry(&self, ui: &mut egui::Ui, entry: &ProblemEntry) -> Vec<SidebarProblemsAction> {
        let (icon, _color) = match entry.severity {
            crate::editor::DiagnosticSeverity::Error => (Icon::AlertCircle, self.theme.danger),
            crate::editor::DiagnosticSeverity::Warning => (Icon::TriangleAlert, self.theme.warning),
            crate::editor::DiagnosticSeverity::Information | crate::editor::DiagnosticSeverity::Hint => {
                (Icon::Info, self.theme.text_muted)
            }
        };
        let source = source_label(entry.source);
        let label = format!(
            "L{}:{}  {}  · {source}",
            entry.line + 1,
            entry.column + 1,
            entry.message
        );
        let selected = self.selected == Some((entry.document_id.as_str(), entry.diagnostic_index));
        let response = sidebar_item(ui, icon, &label, selected, self.theme);
        let mut actions = Vec::new();
        if response.clicked() {
            actions.push(SidebarProblemsAction::Select(entry.clone()));
        }
        if entry.has_fix {
            ui.horizontal(|ui| {
                ui.add_space(12.0);
                if compact_button(ui, "Quick fix", self.theme).clicked() {
                    actions.push(SidebarProblemsAction::QuickFix {
                        document_index: entry.document_index,
                        diagnostic_index: entry.diagnostic_index,
                    });
                }
            });
        }
        actions
    }
}

fn matches_filters(
    entry: &ProblemEntry,
    severity_filter: ProblemsSeverityFilter,
    source_filter: ProblemsSourceFilter,
) -> bool {
    let severity_ok = match severity_filter {
        ProblemsSeverityFilter::All => true,
        ProblemsSeverityFilter::Errors => entry.severity == crate::editor::DiagnosticSeverity::Error,
        ProblemsSeverityFilter::Warnings => entry.severity == crate::editor::DiagnosticSeverity::Warning,
    };
    let source_ok = match source_filter {
        ProblemsSourceFilter::All => true,
        ProblemsSourceFilter::Parser => entry.source == crate::editor::DiagnosticSource::Parser,
        ProblemsSourceFilter::Lint => entry.source == crate::editor::DiagnosticSource::Lint,
        ProblemsSourceFilter::Delimiter => entry.source == crate::editor::DiagnosticSource::Delimiter,
        ProblemsSourceFilter::Database => entry.source == crate::editor::DiagnosticSource::Database,
    };
    severity_ok && source_ok
}

fn severity_label(filter: ProblemsSeverityFilter) -> &'static str {
    match filter {
        ProblemsSeverityFilter::All => "All",
        ProblemsSeverityFilter::Errors => "Errors",
        ProblemsSeverityFilter::Warnings => "Warnings",
    }
}

fn source_label(source: impl Into<ProblemSourceLabel>) -> &'static str {
    match source.into() {
        ProblemSourceLabel::Filter(ProblemsSourceFilter::All) => "All sources",
        ProblemSourceLabel::Filter(ProblemsSourceFilter::Parser)
        | ProblemSourceLabel::Entry(crate::editor::DiagnosticSource::Parser) => "Parser",
        ProblemSourceLabel::Filter(ProblemsSourceFilter::Lint)
        | ProblemSourceLabel::Entry(crate::editor::DiagnosticSource::Lint) => "Lint",
        ProblemSourceLabel::Filter(ProblemsSourceFilter::Delimiter)
        | ProblemSourceLabel::Entry(crate::editor::DiagnosticSource::Delimiter) => "Delimiter",
        ProblemSourceLabel::Filter(ProblemsSourceFilter::Database)
        | ProblemSourceLabel::Entry(crate::editor::DiagnosticSource::Database) => "Database",
    }
}

enum ProblemSourceLabel {
    Filter(ProblemsSourceFilter),
    Entry(crate::editor::DiagnosticSource),
}

impl From<ProblemsSourceFilter> for ProblemSourceLabel {
    fn from(filter: ProblemsSourceFilter) -> Self {
        Self::Filter(filter)
    }
}

impl From<crate::editor::DiagnosticSource> for ProblemSourceLabel {
    fn from(source: crate::editor::DiagnosticSource) -> Self {
        Self::Entry(source)
    }
}
