use super::*;
use egui::{Pos2, Rect, Rounding, Vec2};

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
            RichText::new("Schema Compare & Migration")
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
        // Summary header chips
        let total_diffs = diff.tables_only_in_source.len()
            + diff.tables_only_in_target.len()
            + diff.views_only_in_source.len()
            + diff.views_only_in_target.len()
            + diff.functions_only_in_source.len()
            + diff.functions_only_in_target.len()
            + diff.column_changes.len();

        ui.horizontal(|ui| {
            ui.label(
                RichText::new(format!("{total_diffs} total difference(s) detected"))
                    .small()
                    .strong()
                    .color(if total_diffs > 0 { context.theme.accent } else { context.theme.success }),
            );
        });
        ui.add_space(8.0);

        for (title, items, is_added, is_removed) in [
            ("Tables only in snapshot (Removed in current)", &diff.tables_only_in_source, false, true),
            ("Tables only in current (Added in current)", &diff.tables_only_in_target, true, false),
            ("Views only in snapshot", &diff.views_only_in_source, false, true),
            ("Views only in current", &diff.views_only_in_target, true, false),
            ("Routines only in snapshot", &diff.functions_only_in_source, false, true),
            ("Routines only in current", &diff.functions_only_in_target, true, false),
            ("Column / type changes", &diff.column_changes, false, false),
        ] {
            if items.is_empty() {
                continue;
            }
            card_frame(context.theme).show(ui, |ui| {
                ui.horizontal(|ui| {
                    let badge_text = if is_added {
                        format!("+{}", items.len())
                    } else if is_removed {
                        format!("-{}", items.len())
                    } else {
                        format!("~{}", items.len())
                    };
                    let badge_color = if is_added {
                        context.theme.success
                    } else if is_removed {
                        context.theme.danger
                    } else {
                        context.theme.warning
                    };
                    ui.label(RichText::new(badge_text).small().strong().color(badge_color));
                    ui.label(RichText::new(title).strong().color(context.theme.text_primary));
                });
                ui.add_space(4.0);
                for item in items {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("•").color(context.theme.text_muted));
                        ui.label(RichText::new(item).small().monospace().color(context.theme.text_secondary));
                    });
                }
            });
            ui.add_space(6.0);
        }

        if total_diffs == 0 {
            ui.label(RichText::new("Schemas are identical. No drift detected.").color(context.theme.success));
            ui.add_space(8.0);
        }

        ui.separator();
        ui.add_space(8.0);
        section_label(ui, "MIGRATION PLAN", context.theme);
        if primary_button_with_icon(ui, Icon::FileCode2, "Generate migration plan", context.theme).clicked() {
            context.compare.plan_migration(context.driver, context.feedback);
        }
        if let Some(plan) = &context.compare.migration_plan {
            ui.add_space(6.0);
            ui.label(
                RichText::new(format!(
                    "{} operation(s) · fingerprint {}",
                    plan.operations.len(),
                    plan.fingerprint,
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
            ui.add_space(4.0);
            for op in &plan.operations {
                let (risk_label, risk_color, risk_bg) = match op.risk {
                    db_pro_core::domain::migration::MigrationRisk::Destructive => {
                        ("DESTRUCTIVE", context.theme.danger, context.theme.danger_soft())
                    }
                    db_pro_core::domain::migration::MigrationRisk::Mutating => {
                        ("MUTATING", context.theme.warning, context.theme.warning_soft())
                    }
                    db_pro_core::domain::migration::MigrationRisk::Safe => {
                        ("SAFE", context.theme.success, context.theme.success_soft())
                    }
                };
                ui.horizontal(|ui| {
                    let galley = ui.painter().layout_no_wrap(risk_label.to_owned(), font_caption(), risk_color);
                    let r = Rect::from_min_size(Pos2::new(ui.cursor().min.x, ui.cursor().min.y + 1.0), Vec2::new(galley.size().x + 6.0, 15.0));
                    ui.painter().rect_filled(r, Rounding::same(RADIUS_XS), risk_bg);
                    ui.painter().galley(Pos2::new(r.left() + 3.0, r.top() + 1.0), galley, egui::Color32::PLACEHOLDER);
                    ui.add_space(r.width() + 4.0);
                    ui.label(
                        RichText::new(format!("{} · {}", op.id, op.sql))
                            .small()
                            .monospace()
                            .color(context.theme.text_secondary),
                    );
                });
            }
        }
        if !context.compare.migration_preview_sql.is_empty() {
            ui.add_space(8.0);
            card_frame(context.theme).show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Generated Migration DDL").strong().color(context.theme.text_primary));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if compact_button_with_icon(ui, Icon::Copy, "Copy SQL", context.theme).clicked() {
                            ui.output_mut(|o| o.copied_text = context.compare.migration_preview_sql.clone());
                            context.feedback.set_runtime_message("Copied migration SQL to clipboard");
                        }
                    });
                });
                ui.add_space(4.0);
                ui.label(
                    RichText::new(&context.compare.migration_preview_sql)
                        .small()
                        .monospace()
                        .color(context.theme.text_primary),
                );
            });
            ui.add_space(6.0);
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
            ui.add_space(6.0);
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
                    let (state_text, state_color) = match row.state {
                        db_pro_core::domain::cross_connection::DataRowState::Added => ("+ ADDED", context.theme.success),
                        db_pro_core::domain::cross_connection::DataRowState::Removed => ("- REMOVED", context.theme.danger),
                        db_pro_core::domain::cross_connection::DataRowState::Changed => ("~ CHANGED", context.theme.warning),
                        db_pro_core::domain::cross_connection::DataRowState::Equal => ("= EQUAL", context.theme.text_muted),
                    };
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(state_text).small().strong().color(state_color));
                        ui.label(
                            RichText::new(format!("key: {}", row.key.replace('\u{1f}', " | ")))
                                .small()
                                .monospace()
                                .color(context.theme.text_secondary),
                        );
                    });
                }
            }
            if !diff.sync_sql_preview.is_empty() {
                ui.add_space(4.0);
                section_label(ui, "DATA SYNC SQL PREVIEW", context.theme);
                for sql in &diff.sync_sql_preview {
                    ui.label(RichText::new(sql).small().monospace().color(context.theme.text_muted));
                }
            }
        }
    });
    action
}
