use super::diagram_design_actions::{er_design_add_fk, er_design_apply_plan, er_design_preview_plan};
use super::diagram_view::{DiagramAction, DiagramViewContext};
use super::*;

pub(super) fn draw_er_design_panel(ctx: &mut DiagramViewContext<'_>, ui: &mut egui::Ui) -> Option<DiagramAction> {
    if !ctx.diagram.design.enabled {
        return None;
    }
    let mut action = None;
    ui.add_space(SPACE_SM);
    card_frame(ctx.theme).show(ui, |ui| {
        section_label(ui, "DESIGN DRAFT", ctx.theme);
        ui.label(
            RichText::new("Draft only · ObjectMutationService plan · fingerprint-gated apply")
                .small()
                .color(ctx.theme.text_muted),
        );
        let live_fp = {
            let names: Vec<String> = ctx
                .explorer
                .schema
                .table_details
                .iter()
                .map(|t| format!("{}.{}", t.schema, t.name))
                .collect();
            crate::diagram::design_mode::schema_fingerprint_from_names(&names)
        };
        if ctx.diagram.design.fingerprint_stale(&live_fp) {
            ui.colored_label(
                ctx.theme.warning,
                "Live schema changed — re-open Design Mode or discard before apply",
            );
        }
        ui.horizontal(|ui| {
            ui.add(egui::TextEdit::singleline(&mut ctx.diagram.new_schema).hint_text("schema"));
            ui.add(egui::TextEdit::singleline(&mut ctx.diagram.new_table).hint_text("table"));
            if secondary_button(ui, "Add draft table", ctx.theme).clicked() {
                ctx.diagram
                    .design
                    .add_draft_table(&ctx.diagram.new_schema, &ctx.diagram.new_table);
                ctx.diagram.new_table.clear();
            }
            if ghost_button(ui, "Undo", ctx.theme).clicked() {
                ctx.diagram.design.undo();
            }
            if ghost_button(ui, "Redo", ctx.theme).clicked() {
                ctx.diagram.design.redo();
            }
            if danger_button(ui, "Discard", ctx.theme).clicked() {
                ctx.diagram.design.discard();
            }
        });
        for (idx, table) in ctx.diagram.design.draft.tables.clone().into_iter().enumerate() {
            ui.label(
                RichText::new(format!(
                    "draft {}.{} · {} cols",
                    table.schema,
                    table.name,
                    table.columns.len()
                ))
                .strong()
                .monospace(),
            );
            for col in &table.columns {
                ui.label(
                    RichText::new(format!(
                        "  {} {}{}{}",
                        col.name,
                        col.data_type,
                        if col.is_pk { " PK" } else { "" },
                        if col.is_unique { " UNIQUE" } else { "" }
                    ))
                    .small()
                    .monospace(),
                );
            }
            ui.horizontal(|ui| {
                ui.add(egui::TextEdit::singleline(&mut ctx.diagram.column_name).hint_text("col"));
                ui.add(egui::TextEdit::singleline(&mut ctx.diagram.column_type).hint_text("type"));
                if ghost_button(ui, "Add col", ctx.theme).clicked() {
                    ctx.diagram.design.add_column(
                        idx,
                        crate::diagram::design_mode::DraftColumn {
                            name: ctx.diagram.column_name.clone(),
                            data_type: ctx.diagram.column_type.clone(),
                            nullable: true,
                            is_pk: false,
                            is_unique: false,
                        },
                    );
                    ctx.diagram.column_name.clear();
                }
            });
        }
        ui.add_space(SPACE_XS);
        ui.horizontal(|ui| {
            ui.add(egui::TextEdit::singleline(&mut ctx.diagram.foreign_key_name).hint_text("fk name"));
            ui.add(egui::TextEdit::singleline(&mut ctx.diagram.foreign_key_from).hint_text("from schema.table.col"));
            ui.add(egui::TextEdit::singleline(&mut ctx.diagram.foreign_key_to).hint_text("to schema.table.col"));
            if secondary_button(ui, "Add FK", ctx.theme).clicked() {
                er_design_add_fk(ctx);
            }
        });
        for fk in &ctx.diagram.design.draft.foreign_keys {
            ui.label(
                RichText::new(format!(
                    "FK {} · {}.{}({}) → {}.{}({})",
                    fk.name,
                    fk.from_schema,
                    fk.from_table,
                    fk.from_columns.join(","),
                    fk.to_schema,
                    fk.to_table,
                    fk.to_columns.join(",")
                ))
                .small()
                .monospace(),
            );
        }
        ui.horizontal(|ui| {
            if secondary_button(ui, "Preview mutation plan", ctx.theme).clicked() {
                er_design_preview_plan(ctx);
            }
            if primary_button(ui, "Apply (confirm)", ctx.theme).clicked() {
                action = er_design_apply_plan(ctx);
            }
        });
        if let Some(error) = &ctx.diagram.design.error {
            ui.colored_label(ctx.theme.danger, error);
        }
        if !ctx.diagram.design.preview_sql.is_empty() {
            ui.label(
                RichText::new(format!("fingerprint {}", ctx.diagram.design.preview_fingerprint))
                    .small()
                    .color(ctx.theme.text_muted),
            );
            for effect in &ctx.diagram.design.preview_effects {
                ui.label(RichText::new(effect).small().color(ctx.theme.text_secondary));
            }
            egui::ScrollArea::vertical().max_height(140.0).show(ui, |ui| {
                ui.label(RichText::new(&ctx.diagram.design.preview_sql).monospace());
            });
        }
    });
    action
}
