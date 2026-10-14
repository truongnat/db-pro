//! Pure synthetic-data plan construction for the database management surface.

use crate::UiTableSummary;

pub(super) fn build_plan(
    table_selection: &str,
    row_count_input: &str,
    seed_input: &str,
    null_pct_input: &str,
    table_details: &[UiTableSummary],
) -> Result<db_pro_core::domain::synthetic_data::SyntheticPlan, String> {
    use db_pro_core::domain::synthetic_data::{infer_generator, ColumnSpec, SyntheticPlan};

    if table_selection.is_empty() {
        return Err("select a table".into());
    }
    let (schema, name) = if let Some((schema, table)) = table_selection.split_once('.') {
        (schema.to_owned(), table.to_owned())
    } else {
        (String::new(), table_selection.to_owned())
    };
    let detail = table_details
        .iter()
        .find(|table| table.name == name && (schema.is_empty() || table.schema == schema))
        .ok_or_else(|| "table metadata not loaded".to_owned())?;
    let row_count: u64 = row_count_input.parse().map_err(|_| "invalid row count".to_owned())?;
    let seed: u64 = seed_input.parse().map_err(|_| "invalid seed".to_owned())?;
    let null_rate_pct: u8 = null_pct_input.parse().map_err(|_| "invalid null %".to_owned())?;
    let mut columns: Vec<ColumnSpec> = detail
        .columns
        .iter()
        .map(|column| ColumnSpec {
            name: column.name.clone(),
            data_type: column.data_type.clone(),
            nullable: column.nullable,
            is_primary_key: column.is_primary_key,
            generator: infer_generator(&column.data_type),
            fk_values: Vec::new(),
        })
        .collect();

    // Keep FK seed values deterministic and bounded for preview/export.
    for foreign_key in &detail.foreign_keys {
        for from_column in &foreign_key.from_columns {
            if let Some(column) = columns.iter_mut().find(|column| column.name == *from_column) {
                let pool_len = usize::try_from(row_count.clamp(1, 50)).map_err(|_| "invalid row count".to_owned())?;
                column.fk_values = (1..=pool_len).map(|index| index.to_string()).collect();
            }
        }
    }

    Ok(SyntheticPlan {
        schema: detail.schema.clone(),
        table: detail.name.clone(),
        columns,
        row_count,
        seed,
        null_rate_pct,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{UiSchemaColumn, UiSchemaForeignKey};

    fn summary() -> UiTableSummary {
        UiTableSummary {
            schema: "public".to_owned(),
            name: "orders".to_owned(),
            row_count: None,
            columns: vec![UiSchemaColumn {
                name: "customer_id".to_owned(),
                data_type: "integer".to_owned(),
                nullable: false,
                is_primary_key: false,
            }],
            foreign_keys: vec![UiSchemaForeignKey {
                name: "orders_customer_fk".to_owned(),
                from_columns: vec!["customer_id".to_owned()],
                to_schema: "public".to_owned(),
                to_table: "customers".to_owned(),
                to_columns: vec!["id".to_owned()],
            }],
        }
    }

    #[test]
    fn plan_builder_resolves_table_and_bounded_fk_seed_values() {
        let plan = build_plan("public.orders", "75", "42", "10", &[summary()]).expect("plan");

        assert_eq!(plan.schema, "public");
        assert_eq!(plan.table, "orders");
        assert_eq!(plan.row_count, 75);
        assert_eq!(plan.columns[0].fk_values.len(), 50);
        assert_eq!(plan.columns[0].fk_values[0], "1");
    }

    #[test]
    fn plan_builder_reports_invalid_inputs_at_the_boundary() {
        let error = build_plan("public.orders", "not-a-number", "42", "0", &[summary()]).expect_err("error");

        assert_eq!(error, "invalid row count");
    }
}
