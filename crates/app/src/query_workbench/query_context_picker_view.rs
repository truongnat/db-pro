//! Query connection/schema picker rendering and intent collection.
use super::*;
use egui::RichText;

pub(super) struct QueryContextPickerContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) current_connection_id: Option<&'a str>,
    pub(super) current_schema: &'a str,
    pub(super) available_schemas: &'a [String],
    pub(super) connections: &'a [(String, String, String)],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum QueryContextPickerAction {
    SelectConnection(String),
    SelectSchema(String),
    Close,
}

pub(super) fn draw_picker(
    context: &QueryContextPickerContext<'_>,
    ctx: &egui::Context,
    anchor: egui::Rect,
) -> Option<QueryContextPickerAction> {
    let mut action = None;
    let menu_position = egui::pos2(anchor.left(), anchor.bottom() + 4.0);
    let menu = FloatingPopup::new(egui::Id::new("query_context_picker"), context.theme, menu_position)
        .order(egui::Order::Foreground)
        .constrain_to(ctx.content_rect().shrink(4.0))
        .surface_elevated()
        .radius(8.0)
        .margin(egui::Margin::same(8.0 as i8))
        .shadow(false)
        .show(ctx, |ui| {
            ui.set_min_width(280.0);
            ui.label(
                RichText::new(t!("query.context_connection"))
                    .small()
                    .strong()
                    .color(context.theme.text_muted),
            );
            ui.add_space(4.0);
            ScrollArea::new(context.theme).max_height(180.0).show(ui, |ui| {
                for (id, name, environment) in context.connections {
                    let selected = context.current_connection_id == Some(id.as_str());
                    let label = if environment.is_empty() {
                        name.clone()
                    } else {
                        format!("{name} · {environment}")
                    };
                    if crate::components::select::paint_option(
                        ui,
                        crate::components::select::SelectOption {
                            label: &label,
                            selected,
                            theme: context.theme,
                            icon: None,
                            disabled: false,
                        },
                    )
                    .clicked()
                    {
                        action = Some(QueryContextPickerAction::SelectConnection(id.clone()));
                    }
                }
            });
            Separator::horizontal(context.theme).show(ui);
            ui.label(
                RichText::new(t!("query.context_schema"))
                    .small()
                    .strong()
                    .color(context.theme.text_muted),
            );
            ui.add_space(4.0);
            // Schema lists grow with tenants (PostgreSQL search_paths, one schema per
            // tenant). Without a bounded scroll the popup clipped every schema past
            // the viewport, making them unreachable — UI-03.
            ScrollArea::new(context.theme).max_height(180.0).show(ui, |ui| {
                for schema in context.available_schemas {
                    if crate::components::select::paint_option(
                        ui,
                        crate::components::select::SelectOption {
                            label: schema,
                            selected: context.current_schema == schema,
                            theme: context.theme,
                            icon: None,
                            disabled: false,
                        },
                    )
                    .clicked()
                    {
                        action = Some(QueryContextPickerAction::SelectSchema(schema.clone()));
                    }
                }
            });
        });

    let clicked_outside = ctx.input(|input| {
        input.pointer.any_click()
            && input
                .pointer
                .interact_pos()
                .is_some_and(|position| !menu.response.rect.contains(position) && !anchor.contains(position))
    });
    let escape = ctx.input(|input| input.key_pressed(egui::Key::Escape));
    if clicked_outside || escape {
        Some(QueryContextPickerAction::Close)
    } else {
        action
    }
}
