use super::*;

pub(super) struct SchemaCompareViewContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) compare: &'a mut SchemaCompareState,
    pub(super) schema: &'a UiSchemaSummary,
    pub(super) connection_name: &'a str,
    pub(super) driver: &'a str,
    pub(super) feedback: &'a mut FeedbackState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SchemaCompareAction {
    OpenWorkspace,
    RequestDataDiff,
    ApplyMigration,
}

pub(super) fn draw_schema_compare_sidebar(
    context: &mut SchemaCompareViewContext<'_>,
    ui: &mut egui::Ui,
) -> Option<SchemaCompareAction> {
    section_label(ui, "SCHEMA COMPARE", context.theme);
    ui.add_space(6.0);
    ui.label(
        RichText::new("Snapshot the loaded schema, then re-introspect and diff.")
            .small()
            .color(context.theme.text_muted),
    );
    ui.add_space(8.0);
    if compact_button_with_icon(ui, Icon::Camera, "Take snapshot", context.theme).clicked() {
        context
            .compare
            .take_snapshot(context.schema, context.connection_name, context.feedback);
    }
    let mut action = None;
    if compact_button_with_icon(ui, Icon::GitCompare, "Diff vs snapshot", context.theme).clicked() {
        context.compare.diff_against_snapshot(context.schema, context.feedback);
        action = Some(SchemaCompareAction::OpenWorkspace);
    }
    ui.add_space(8.0);
    if let Some(snap) = &context.compare.schema_snapshot {
        ui.label(
            RichText::new(format!("Snapshot: {}", snap.label))
                .small()
                .color(context.theme.text_secondary),
        );
        ui.label(
            RichText::new(format!("{} tables", snap.tables.len()))
                .small()
                .color(context.theme.text_muted),
        );
    } else {
        ui.label(
            RichText::new("No snapshot yet.")
                .small()
                .color(context.theme.text_muted),
        );
    }
    action
}

pub(super) fn draw_schema_compare(
    context: &mut SchemaCompareViewContext<'_>,
    ui: &mut egui::Ui,
) -> Option<SchemaCompareAction> {
    ui.set_min_width(ui.available_width());
    ui.add_space(SPACE_SM);
    ui.horizontal(|ui| {
        ui.label(
            RichText::new("Schema Compare")
                .font(font_subheading())
                .strong()
                .color(context.theme.text_primary),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if secondary_button_with_icon(ui, Icon::GitCompare, "Diff now", context.theme).clicked() {
                context.compare.diff_against_snapshot(context.schema, context.feedback);
            }
            if secondary_button_with_icon(ui, Icon::Camera, "Snapshot", context.theme).clicked() {
                context
                    .compare
                    .take_snapshot(context.schema, context.connection_name, context.feedback);
            }
        });
    });
    ui.add_space(SPACE_MD);
    let Some(diff) = context.compare.schema_diff.clone() else {
        card_frame(context.theme).show(ui, |ui| {
            ui.set_min_width((ui.available_width() - 8.0).max(0.0));
            empty_state(
                ui,
                Icon::GitCompare,
                "No schema diff yet",
                "Take a snapshot, change or refresh the schema, then Diff now.",
                context.theme,
            );
        });
        return None;
    };

    let mut action = None;
    egui::ScrollArea::vertical().show(ui, |ui| {
        for (title, items) in [
            ("Tables only in snapshot", &diff.tables_only_in_source),
            ("Tables only in current", &diff.tables_only_in_target),
            ("Views only in snapshot", &diff.views_only_in_source),
            ("Views only in current", &diff.views_only_in_target),
            ("Routines only in snapshot", &diff.functions_only_in_source),
            ("Routines only in current", &diff.functions_only_in_target),
            ("Column / type changes", &diff.column_changes),
        ] {
            ui.label(RichText::new(title).strong());
            if items.is_empty() {
                ui.label(RichText::new("— none —").small().color(context.theme.text_muted));
            } else {
                for item in items {
                    ui.label(RichText::new(format!("• {item}")).small());
                }
            }
            ui.add_space(8.0);
        }
        ui.separator();
        ui.add_space(8.0);
        section_label(ui, "MIGRATION PLAN", context.theme);
        if primary_button_with_icon(ui, Icon::FileCode2, "Generate migration plan", context.theme).clicked() {
            context.compare.plan_migration(context.driver, context.feedback);
        }
        if let Some(plan) = &context.compare.migration_plan {
            ui.label(
                RichText::new(format!(
                    "{} ops · fingerprint {} · destructive={}",
                    plan.operations.len(),
                    plan.fingerprint,
                    plan.has_destructive
                ))
                .small()
                .monospace()
                .color(context.theme.text_secondary),
            );
            for warning in &plan.warnings {
                ui.label(
                    RichText::new(format!("⚠ {warning}"))
                        .small()
                        .color(context.theme.warning),
                );
            }
            for op in &plan.operations {
                let risk = match op.risk {
                    db_pro_core::domain::migration::MigrationRisk::Destructive => "DESTRUCTIVE",
                    db_pro_core::domain::migration::MigrationRisk::Mutating => "mutating",
                    db_pro_core::domain::migration::MigrationRisk::Safe => "safe",
                };
                ui.label(
                    RichText::new(format!("{} · {} · {}", op.id, risk, op.sql))
                        .small()
                        .monospace()
                        .color(context.theme.text_secondary),
                );
            }
        }
        if !context.compare.migration_preview_sql.is_empty() {
            ui.label(
                RichText::new(&context.compare.migration_preview_sql)
                    .small()
                    .monospace()
                    .color(context.theme.text_primary),
            );
            if context
                .compare
                .migration_plan
                .as_ref()
                .is_some_and(|plan| plan.has_destructive)
            {
                ui.checkbox(
                    &mut context.compare.migration_confirm_destructive,
                    "Confirm destructive operations (never auto-applied)",
                );
            }
            if primary_button_with_icon(ui, Icon::Play, "Apply migration SQL", context.theme).clicked() {
                action = Some(SchemaCompareAction::ApplyMigration);
            }
        }
        ui.separator();
        ui.add_space(8.0);
        section_label(ui, "DATA COMPARE", context.theme);
        ui.label(
            RichText::new("Key-aware sample compare across two active connections. Sync SQL is preview-only.")
                .small()
                .color(context.theme.text_muted),
        );
        input_full_width(
            ui,
            &mut context.compare.data_diff_target_id,
            "target connection id",
            context.theme,
        );
        input_full_width(ui, &mut context.compare.data_diff_schema, "schema", context.theme);
        input_full_width(ui, &mut context.compare.data_diff_table, "table", context.theme);
        input_full_width(
            ui,
            &mut context.compare.data_diff_keys,
            "key columns (comma)",
            context.theme,
        );
        if primary_button_with_icon(ui, Icon::GitCompare, "Compare rows", context.theme).clicked() {
            action = Some(SchemaCompareAction::RequestDataDiff);
        }
        if let Some(diff) = &context.compare.data_diff_result {
            ui.label(
                RichText::new(format!(
                    "counts src={} tgt={} · +{} -{} ~{} ={} · truncated={}",
                    diff.source_row_count,
                    diff.target_row_count,
                    diff.added,
                    diff.removed,
                    diff.changed,
                    diff.equal,
                    diff.truncated
                ))
                .small()
                .monospace()
                .color(context.theme.text_secondary),
            );
            ui.horizontal(|ui| {
                for label in ["all", "added", "removed", "changed"] {
                    if ui
                        .selectable_label(context.compare.data_diff_filter == label, label)
                        .clicked()
                    {
                        context.compare.data_diff_filter = label.to_owned();
                    }
                }
            });
            for row in &diff.row_diffs {
                let include = match context.compare.data_diff_filter.as_str() {
                    "added" => row.state == db_pro_core::domain::cross_connection::DataRowState::Added,
                    "removed" => row.state == db_pro_core::domain::cross_connection::DataRowState::Removed,
                    "changed" => row.state == db_pro_core::domain::cross_connection::DataRowState::Changed,
                    _ => true,
                };
                if include {
                    ui.label(
                        RichText::new(format!("{:?} · {}", row.state, row.key.replace('\u{1f}', "|")))
                            .small()
                            .monospace()
                            .color(context.theme.text_secondary),
                    );
                }
            }
            for sql in &diff.sync_sql_preview {
                ui.label(RichText::new(sql).small().monospace().color(context.theme.text_muted));
            }
        }
    });
    action
}
