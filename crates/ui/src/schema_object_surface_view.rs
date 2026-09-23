use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum SchemaObjectSurfaceAction {
    OpenQuery(String),
    SelectView(SchemaObjectView),
}

pub(super) struct SchemaObjectSurfaceContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) icon: Icon,
    pub(super) kind: &'a str,
    pub(super) schema: &'a str,
    pub(super) name: &'a str,
    pub(super) metadata: Option<&'a str>,
    pub(super) query: &'a str,
    pub(super) is_view: bool,
    pub(super) active_view: SchemaObjectView,
}

pub(super) struct SchemaDefinitionContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) kind: &'a str,
    pub(super) definition: &'a str,
}

impl SchemaDefinitionContext<'_> {
    pub(super) fn draw(&self, ui: &mut egui::Ui) {
        card_frame(self.theme).show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            section_label(ui, format!("{} DEFINITION", self.kind), self.theme);
            ui.add_space(SPACE_SM);
            CodeBlock::new(self.definition, self.theme).language("sql").show(ui);
        });
    }
}

impl SchemaObjectSurfaceContext<'_> {
    pub(super) fn draw(&self, ui: &mut egui::Ui) -> Vec<SchemaObjectSurfaceAction> {
        let mut actions = Vec::new();
        toolbar_frame(self.theme).show(ui, |ui| {
            ui.set_min_width(ui.available_width().max(0.0));
            ui.horizontal_wrapped(|ui| {
                self.draw_breadcrumb(ui);
                if self.is_view {
                    actions.extend(self.draw_view_tabs(ui));
                }
                if Button::new(self.theme)
                    .icon(Icon::FileCode2)
                    .text("Open in Query")
                    .variant(ButtonVariant::Secondary)
                    .size(ButtonSize::Sm)
                    .show(ui)
                    .clicked()
                {
                    actions.push(SchemaObjectSurfaceAction::OpenQuery(self.query.to_owned()));
                }
            });
        });
        actions
    }

    fn draw_breadcrumb(&self, ui: &mut egui::Ui) {
        ui.label(icon_text(self.icon, self.kind, self.theme.text_primary));
        ui.label(icon_text(Icon::ChevronRight, "", self.theme.text_muted));
        ui.label(
            RichText::new(format!("{}.{}", self.schema, self.name))
                .strong()
                .color(self.theme.accent),
        );
        if let Some(metadata) = self.metadata {
            badge(ui, metadata, self.theme.surface_active, self.theme.text_secondary);
        }
    }

    fn draw_view_tabs(&self, ui: &mut egui::Ui) -> Vec<SchemaObjectSurfaceAction> {
        let mut actions = Vec::new();
        for (view, icon, label) in [
            (SchemaObjectView::Definition, Icon::Code2, "Definition"),
            (SchemaObjectView::Data, Icon::Table2, "Data"),
        ] {
            let selected = self.active_view == view;
            let tab = tab_frame(self.theme, selected).show(ui, |ui| {
                ui.selectable_label(selected, icon_text(icon, label, self.theme.text_primary))
            });
            if tab.inner.clicked() {
                actions.push(SchemaObjectSurfaceAction::SelectView(view));
            }
        }
        actions
    }
}
