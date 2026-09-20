use super::diagram_view::{DiagramAction, DiagramViewContext};

pub(super) fn er_design_add_fk(ctx: &mut DiagramViewContext<'_>) {
    let Some((from_schema, from_table, from_col)) = split_three(&ctx.diagram.foreign_key_from) else {
        ctx.diagram.design.error = Some("FK from must be schema.table.column".into());
        return;
    };
    let Some((to_schema, to_table, to_col)) = split_three(&ctx.diagram.foreign_key_to) else {
        ctx.diagram.design.error = Some("FK to must be schema.table.column".into());
        return;
    };
    let name = if ctx.diagram.foreign_key_name.trim().is_empty() {
        format!("fk_{from_table}_{to_table}")
    } else {
        ctx.diagram.foreign_key_name.trim().to_owned()
    };
    ctx.diagram.design.add_fk(crate::diagram::design_mode::DraftForeignKey {
        name,
        from_schema,
        from_table,
        from_columns: vec![from_col],
        to_schema,
        to_table,
        to_columns: vec![to_col],
    });
}

pub(super) fn er_design_preview_plan(ctx: &mut DiagramViewContext<'_>) {
    struct QuoteDialect;
    impl db_pro_core::ports::SqlDialect for QuoteDialect {
        fn placeholder(&self, index: usize) -> String {
            format!("${index}")
        }
        fn quote_identifier(&self, name: &str) -> String {
            format!("\"{}\"", name.replace('"', "\"\""))
        }
    }

    match crate::diagram::design_mode::plan_design_draft(&ctx.diagram.design.draft, ctx.active_driver, &QuoteDialect) {
        Ok(previews) => {
            let (sql, fingerprint, effects) = crate::diagram::design_mode::merge_preview_sql(&previews);
            ctx.diagram.design.preview_sql = sql;
            ctx.diagram.design.preview_fingerprint = fingerprint;
            ctx.diagram.design.preview_effects = effects;
            ctx.diagram.design.error = None;
            ctx.diagram.design.apply_confirm = true;
        }
        Err(err) => {
            ctx.diagram.design.error = Some(err);
            ctx.diagram.design.clear_preview();
        }
    }
}

pub(super) fn er_design_apply_plan(ctx: &mut DiagramViewContext<'_>) -> Option<DiagramAction> {
    let live_fingerprint = {
        let names: Vec<String> = ctx
            .explorer
            .schema
            .table_details
            .iter()
            .map(|table| format!("{}.{}", table.schema, table.name))
            .collect();
        crate::diagram::design_mode::schema_fingerprint_from_names(&names)
    };
    if ctx.diagram.design.fingerprint_stale(&live_fingerprint) {
        ctx.diagram.design.error = Some("schema fingerprint stale — refresh Design Mode".into());
        return None;
    }
    if ctx.diagram.design.preview_sql.is_empty() || !ctx.diagram.design.apply_confirm {
        er_design_preview_plan(ctx);
        if ctx.diagram.design.preview_sql.is_empty() {
            return None;
        }
    }
    if !ctx.connected {
        ctx.diagram.design.error = Some("connect before applying design plan".into());
        return None;
    }

    let sql = ctx.diagram.design.preview_sql.clone();
    ctx.diagram.design.discard();
    Some(DiagramAction::ExecuteQuery(sql))
}

fn split_three(raw: &str) -> Option<(String, String, String)> {
    let parts: Vec<&str> = raw.split('.').collect();
    (parts.len() == 3).then(|| (parts[0].to_owned(), parts[1].to_owned(), parts[2].to_owned()))
}
