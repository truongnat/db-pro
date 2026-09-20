use super::*;

impl DbProApp {
    pub(super) fn er_design_add_fk(&mut self) {
        let Some((from_schema, from_table, from_col)) = split_three(&self.schema.diagram.foreign_key_from) else {
            self.schema.diagram.design.error = Some("FK from must be schema.table.column".into());
            return;
        };
        let Some((to_schema, to_table, to_col)) = split_three(&self.schema.diagram.foreign_key_to) else {
            self.schema.diagram.design.error = Some("FK to must be schema.table.column".into());
            return;
        };
        let name = if self.schema.diagram.foreign_key_name.trim().is_empty() {
            format!("fk_{from_table}_{to_table}")
        } else {
            self.schema.diagram.foreign_key_name.trim().to_owned()
        };
        self.schema
            .diagram
            .design
            .add_fk(crate::diagram::design_mode::DraftForeignKey {
                name,
                from_schema,
                from_table,
                from_columns: vec![from_col],
                to_schema,
                to_table,
                to_columns: vec![to_col],
            });
    }

    pub(super) fn er_design_preview_plan(&mut self) {
        struct QuoteDialect;
        impl db_pro_core::ports::SqlDialect for QuoteDialect {
            fn placeholder(&self, index: usize) -> String {
                format!("${index}")
            }
            fn quote_identifier(&self, name: &str) -> String {
                format!("\"{}\"", name.replace('"', "\"\""))
            }
        }

        let driver = self.active_driver().to_owned();
        match crate::diagram::design_mode::plan_design_draft(&self.schema.diagram.design.draft, &driver, &QuoteDialect)
        {
            Ok(previews) => {
                let (sql, fingerprint, effects) = crate::diagram::design_mode::merge_preview_sql(&previews);
                self.schema.diagram.design.preview_sql = sql;
                self.schema.diagram.design.preview_fingerprint = fingerprint;
                self.schema.diagram.design.preview_effects = effects;
                self.schema.diagram.design.error = None;
                self.schema.diagram.design.apply_confirm = true;
            }
            Err(err) => {
                self.schema.diagram.design.error = Some(err);
                self.schema.diagram.design.clear_preview();
            }
        }
    }

    pub(super) fn er_design_apply_plan(&mut self) {
        let live_fingerprint = {
            let names: Vec<String> = self
                .schema
                .explorer
                .schema
                .table_details
                .iter()
                .map(|table| format!("{}.{}", table.schema, table.name))
                .collect();
            crate::diagram::design_mode::schema_fingerprint_from_names(&names)
        };
        if self.schema.diagram.design.fingerprint_stale(&live_fingerprint) {
            self.schema.diagram.design.error = Some("schema fingerprint stale — refresh Design Mode".into());
            return;
        }
        if self.schema.diagram.design.preview_sql.is_empty() || !self.schema.diagram.design.apply_confirm {
            self.er_design_preview_plan();
            if self.schema.diagram.design.preview_sql.is_empty() {
                return;
            }
        }
        if self.connection.lifecycle.active_connection_id().is_none() || !self.connection.lifecycle.is_connected() {
            self.schema.diagram.design.error = Some("connect before applying design plan".into());
            return;
        }
        self.set_active_query_text(self.schema.diagram.design.preview_sql.clone());
        self.workspace.active_tab = WorkspaceTab::Query;
        self.dispatch_query();
        self.feedback.runtime_message = "Design Mode mutation plan applied via query runtime".into();
        self.schema.diagram.design.discard();
    }
}

fn split_three(raw: &str) -> Option<(String, String, String)> {
    let parts: Vec<&str> = raw.split('.').collect();
    (parts.len() == 3).then(|| (parts[0].to_owned(), parts[1].to_owned(), parts[2].to_owned()))
}
