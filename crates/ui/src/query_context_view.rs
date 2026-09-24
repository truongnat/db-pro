//! Query context strip rendering and its local toggle state.
use super::*;
use egui::{Align, Layout, RichText};
use lucide_icons::Icon;

const CONTEXT_CHIP_MAX_CHARS: usize = 28;

pub(super) struct QueryContextViewContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) editor: &'a mut QueryEditorState,
    pub(super) file_path: Option<&'a str>,
    pub(super) connected: bool,
    pub(super) connection_name: &'a str,
    pub(super) schema: &'a str,
    pub(super) environment: &'a str,
    pub(super) active_request_id: Option<RequestId>,
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
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = egui::vec2(6.0, 0.0);
        let chip_response = draw_context_chip(context, ui);
        if chip_response.clicked() {
            context.editor.query_context_picker_open = !context.editor.query_context_picker_open;
        }
        anchors.context_anchor = Some(chip_response.rect);

        let run_context = query_run_control_view::QueryRunControlContext {
            theme: context.theme,
            connected: context.connected,
            active_request_id: context.active_request_id,
            cancel_supported: context.cancel_supported,
            cancel_reason: context.cancel_reason,
            modifier: context.modifier,
        };
        if let Some(run) = query_run_control_view::draw_run_control(&run_context, ui) {
            action = Some(QueryChromeAction::RunControl(run));
        }
        if context.active_request_id.is_none() {
            if let Some(tool) = draw_editor_tools(context, ui) {
                action = Some(tool);
            }
        }

        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let more_response = Button::new(context.theme)
                .icon(Icon::MoreHorizontal)
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::IconSm)
                .tooltip("More query actions")
                .show(ui);
            if more_response.clicked() {
                context.editor.query_tools_open = !context.editor.query_tools_open;
            }
            anchors.more_anchor = Some(more_response.rect);
        });
    });
    let (rule, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 1.0), egui::Sense::hover());
    ui.painter()
        .hline(rule.x_range(), rule.center().y, egui::Stroke::new(1.0, context.theme.border_subtle));
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

fn draw_file_path_breadcrumb(theme: DbProTheme, ui: &mut egui::Ui, path: &str) {
    let segments: Vec<&str> = path.split(['/', '\\']).filter(|segment| !segment.is_empty()).collect();
    let start = segments.len().saturating_sub(3);
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = egui::vec2(2.0, 0.0);
        ui.label(icon_text(Icon::FileCode2, "", theme.text_muted));
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

fn draw_context_chip(context: &QueryContextViewContext<'_>, ui: &mut egui::Ui) -> egui::Response {
    let is_production =
        context.environment.eq_ignore_ascii_case("production") || context.environment.eq_ignore_ascii_case("prod");
    let chip_text = if context.connected {
        format!(
            "{} / {}",
            crate::components::truncate_ellipsis(context.connection_name, CONTEXT_CHIP_MAX_CHARS),
            crate::components::truncate_ellipsis(context.schema, CONTEXT_CHIP_MAX_CHARS)
        )
    } else {
        context.connection_name.to_owned()
    };
    let tooltip = if context.connected {
        format!("{} · {}", context.connection_name, context.schema)
    } else {
        "Choose a connection to run SQL".to_owned()
    };
    let color = if !context.connected {
        context.theme.warning
    } else if is_production {
        context.theme.danger
    } else {
        context.theme.text_secondary
    };

    let chip_fill = if context.connected {
        egui::Color32::TRANSPARENT
    } else {
        context.theme.warning_soft()
    };
    let chip_stroke = if context.connected {
        context.theme.border_subtle
    } else {
        context.theme.warning
    };
    egui::Frame::none()
        .fill(chip_fill)
        .rounding(egui::Rounding::same(RADIUS_SM))
        .inner_margin(egui::Margin::symmetric(SPACE_XS + 2.0, 2.0))
        .stroke(egui::Stroke::new(1.0, chip_stroke))
        .show(ui, |ui| {
            draw_context_chip_contents(context, ui, is_production, chip_text, color)
        })
        .response
        .on_hover_text(tooltip)
        .interact(egui::Sense::click())
}

fn draw_context_chip_contents(
    context: &QueryContextViewContext<'_>,
    ui: &mut egui::Ui,
    is_production: bool,
    chip_text: String,
    color: egui::Color32,
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
        ui.label(RichText::new(chip_text).font(font_caption()).color(color));
        ui.label(icon_text(Icon::ChevronDown, "", context.theme.text_muted));
    });
}
