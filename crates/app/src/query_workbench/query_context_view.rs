//! Query context strip rendering and its local toggle state.
use super::*;
use crate::tokens::{component::button::BUTTON_HEIGHT_SM, RADIUS_INPUT, SPACE_SM, SPACE_XS, STROKE_THIN};
use egui::{Align, Layout, RichText};
use lucide_icons::Icon;

const INLINE_TOOLS_MIN_WIDTH: f32 = 860.0;

pub(super) struct QueryContextViewContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) editor: &'a mut QueryEditorState,
    pub(super) execution: &'a mut QueryExecutionPolicyState,
    pub(super) file_path: Option<&'a str>,
    pub(super) connected: bool,
    pub(super) connection_name: &'a str,
    pub(super) schema: &'a str,
    pub(super) environment: db_pro_core::domain::connection::ConnectionEnvironment,
    pub(super) active_request_id: Option<RequestId>,
    /// Set while a query runs so the run control can show elapsed time.
    pub(super) execution_started_at: Option<std::time::Instant>,
    pub(super) cancel_supported: bool,
    pub(super) cancel_reason: Option<&'a str>,
    pub(super) modifier: &'a str,
}

pub(super) enum QueryChromeAction {
    RunControl(query_run_control_view::QueryRunControlAction),
    Explain,
    Format,
}

pub(super) struct QueryChromeOutput {
    pub(super) anchors: QueryChromeAnchors,
    pub(super) action: Option<QueryChromeAction>,
}

#[derive(Default)]
pub(super) struct QueryChromeAnchors {
    pub(super) context_anchor: Option<egui::Rect>,
    pub(super) more_anchor: Option<egui::Rect>,
}

pub(super) fn draw_context_strip(context: &mut QueryContextViewContext<'_>, ui: &mut egui::Ui) -> QueryChromeOutput {
    if let Some(path) = context.file_path {
        draw_file_path_breadcrumb(context.theme, ui, path);
    }

    let mut anchors = QueryChromeAnchors::default();
    let mut action = None;
    let compact = ui.available_width() < INLINE_TOOLS_MIN_WIDTH;
    let context_width = if compact {
        ui.available_width()
    } else {
        ui.available_width() - 560.0
    };
    if compact {
        let chip_response = draw_context_chip(context, ui, context_width);
        if context_chip_activated(&chip_response, ui) {
            context.editor.query_context_picker_open = !context.editor.query_context_picker_open;
        }
        anchors.context_anchor = Some(chip_response.rect);
    }
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = egui::vec2(6.0, 6.0);
        if !compact {
            let chip_response = draw_context_chip(context, ui, context_width);
            if context_chip_activated(&chip_response, ui) {
                context.editor.query_context_picker_open = !context.editor.query_context_picker_open;
            }
            anchors.context_anchor = Some(chip_response.rect);
        }

        let run_context = query_run_control_view::QueryRunControlContext {
            theme: context.theme,
            connected: context.connected,
            active_request_id: context.active_request_id,
            execution_started_at: context.execution_started_at,
            cancel_supported: context.cancel_supported,
            cancel_reason: context.cancel_reason,
            modifier: context.modifier,
        };
        if let Some(run) = query_run_control_view::draw_run_control(&run_context, ui) {
            action = Some(QueryChromeAction::RunControl(run));
        }
        if !compact && context.active_request_id.is_none() {
            if let Some(tool) = draw_editor_tools(context, ui) {
                action = Some(tool);
            }
        }
        // Result settings sit with the overflow action on the trailing edge, so a
        // wide strip reads as "actions left, view settings right" instead of one
        // control cluster followed by a ~60 % empty run.
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let more_response = Button::new(context.theme)
                .icon(Icon::MoreHorizontal)
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::IconSm)
                .access_label("More query actions")
                .tooltip("More query actions")
                .show(ui);
            if more_response.clicked() {
                context.editor.query_tools_open = !context.editor.query_tools_open;
            }
            anchors.more_anchor = Some(more_response.rect);
            // Right-to-left: drawing after the overflow places the cap to its left.
            draw_row_limit_picker(context, ui);
        });
    });
    let (rule, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 1.0), egui::Sense::hover());
    ui.painter().hline(
        rule.x_range(),
        rule.center().y,
        egui::Stroke::new(1.0, context.theme.border_subtle),
    );
    QueryChromeOutput { anchors, action }
}

fn draw_editor_tools(context: &QueryContextViewContext<'_>, ui: &mut egui::Ui) -> Option<QueryChromeAction> {
    let explain = Button::new(context.theme)
        .text("Explain")
        .icon(Icon::ChartNoAxesCombined)
        .variant(ButtonVariant::Outline)
        .size(ButtonSize::Sm)
        .tooltip("Explain query")
        .show(ui);
    let format = Button::new(context.theme)
        .text("Format")
        .icon(Icon::AlignLeft)
        .variant(ButtonVariant::Ghost)
        .size(ButtonSize::Sm)
        .tooltip("Format SQL")
        .show(ui);
    if explain.clicked() {
        Some(QueryChromeAction::Explain)
    } else if format.clicked() {
        Some(QueryChromeAction::Format)
    } else {
        None
    }
}

/// Row-cap selector (`100 / 500 / 1,000 / 100,000`).
///
/// The top step is the product ceiling (`ConnectionConfig::max_rows`). A real
/// "no limit" made the server sort or scan a whole table before the client
/// stopped reading.
const QUERY_ROW_CAP_CEILING: u64 = 100_000;
fn draw_row_limit_picker(context: &mut QueryContextViewContext<'_>, ui: &mut egui::Ui) {
    let (sep, _) = ui.allocate_exact_size(egui::vec2(1.0, 16.0), egui::Sense::hover());
    ui.painter().vline(
        sep.center().x,
        sep.y_range(),
        egui::Stroke::new(1.0, context.theme.border_subtle),
    );
    if context.execution.query_row_limit.is_none() {
        context.execution.query_row_limit = Some(QUERY_ROW_CAP_CEILING);
    }
    const ROW_CAPS: [u64; 4] = [100, 500, 1_000, QUERY_ROW_CAP_CEILING];
    let labels: Vec<String> = ROW_CAPS
        .iter()
        .map(|cap| match cap {
            100 => t!("query.row_limit_100").into_owned(),
            500 => t!("query.row_limit_500").into_owned(),
            1_000 => t!("query.row_limit_1000").into_owned(),
            _ => t!("query.row_limit_100000").into_owned(),
        })
        .collect();
    let mut selected = ROW_CAPS
        .iter()
        .position(|value| Some(*value) == context.execution.query_row_limit)
        .unwrap_or(usize::MAX);
    let previous = selected;
    crate::components::Select::new("query_row_limit", &mut selected, &labels)
        .theme(context.theme)
        .width(108.0)
        .size(crate::components::SelectSize::Sm)
        .variant(crate::components::SelectVariant::Ghost)
        .show(ui)
        .on_hover_text(t!("query.row_limit_tooltip"));
    if selected != previous {
        context.execution.query_row_limit = Some(ROW_CAPS[selected]);
    }
}

fn draw_file_path_breadcrumb(theme: DbProTheme, ui: &mut egui::Ui, path: &str) {
    let segments: Vec<&str> = path.split(['/', '\\']).filter(|segment| !segment.is_empty()).collect();
    let start = segments.len().saturating_sub(3);
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = egui::vec2(2.0, 0.0);
        ui.label(
            RichText::new(char::from(Icon::FileCode2).to_string())
                .font(font_icon(ICON_XS))
                .color(theme.text_muted),
        );
        if start > 0 {
            ui.label(RichText::new("…").font(font_caption()).color(theme.text_muted));
            ui.label(RichText::new("/").font(font_caption()).color(theme.text_muted));
        }
        for (index, segment) in segments[start..].iter().enumerate() {
            if index > 0 {
                ui.label(RichText::new("/").font(font_caption()).color(theme.text_muted));
            }
            let is_leaf = start + index + 1 == segments.len();
            ui.label(RichText::new(*segment).font(font_caption()).color(if is_leaf {
                theme.text_secondary
            } else {
                theme.text_muted
            }));
        }
    })
    .response
    .on_hover_text(path);
}

fn context_chip_activated(response: &egui::Response, ui: &egui::Ui) -> bool {
    response.clicked()
        || (response.has_focus()
            && ui.input(|input| input.key_pressed(egui::Key::Enter) || input.key_pressed(egui::Key::Space)))
}

fn draw_context_chip(context: &QueryContextViewContext<'_>, ui: &mut egui::Ui, max_width: f32) -> egui::Response {
    let is_production = matches!(
        context.environment,
        db_pro_core::domain::connection::ConnectionEnvironment::Production
    );
    let chip_text = if context.connected {
        format!("{} / {}", context.connection_name, context.schema)
    } else {
        context.connection_name.to_owned()
    };
    let tooltip = if context.connected {
        format!("{} · {}", context.connection_name, context.schema)
    } else {
        t!("query.context_pick_connection").into_owned()
    };
    let color = if !context.connected {
        context.theme.warning
    } else if is_production {
        context.theme.danger
    } else {
        context.theme.text_secondary
    };

    let response = context_chip_frame(context.theme)
        .show(ui, |ui| {
            // Match the small-button (28px) row height so the chip centers with
            // the run control instead of pushing the whole strip down.
            ui.set_min_height(BUTTON_HEIGHT_SM - SPACE_XS * 2.0 - STROKE_THIN * 2.0);
            // cc-scan:allow TOO_MANY_PARAMS — context params passed through
            draw_context_chip_contents(context, ui, is_production, &chip_text, color, max_width)
        })
        .response
        .on_hover_text(tooltip)
        .interact(egui::Sense::click());
    // The chip is clickable but painted manually — egui can't derive a name.
    response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, ui.is_enabled(), chip_text.as_str()));
    response
}

/// Toolbar-height chrome for the context chip. `card_frame`'s 12px card inset
/// made the chip ~50px tall, so every 28px control in the strip centered 11px
/// lower and read as a separate row.
fn context_chip_frame(theme: DbProTheme) -> egui::Frame {
    egui::Frame {
        fill: theme.surface_elevated,
        inner_margin: egui::Margin::symmetric(SPACE_SM as i8, SPACE_XS as i8),
        corner_radius: egui::CornerRadius::same(RADIUS_INPUT as u8),
        stroke: egui::Stroke::new(STROKE_THIN, theme.border_subtle),
        ..Default::default()
    }
}

fn draw_context_chip_contents(
    context: &QueryContextViewContext<'_>,
    ui: &mut egui::Ui,
    is_production: bool,
    chip_text: &str,
    color: egui::Color32,
    max_width: f32,
) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = egui::vec2(4.0, 0.0);
        if context.connected {
            let dot = if is_production {
                context.theme.danger
            } else {
                context.theme.success
            };
            ui.label(RichText::new("●").font(font_caption()).color(dot));
        } else {
            ui.label(icon_text(Icon::AlertTriangle, "", context.theme.warning));
        }
        let mut job = egui::text::LayoutJob::simple_singleline(chip_text.to_owned(), font_caption(), color);
        // Reserve the frame margins, status glyph, disclosure and inter-item gaps.
        job.wrap.max_width = (max_width - SPACE_SM * 2.0 - ICON_DEFAULT * 2.0 - SPACE_XS * 2.0).max(1.0);
        job.wrap.max_rows = 1;
        job.wrap.break_anywhere = true;
        let galley = ui.fonts_mut(|fonts| fonts.layout_job(job));
        ui.label(galley);
        ui.label(icon_text(Icon::ChevronDown, "", context.theme.text_muted));
    });
}

#[cfg(test)]
#[path = "tests/query_context_view_tests.rs"]
mod tests;
