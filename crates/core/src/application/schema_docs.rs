//! Schema documentation export (#250).

use crate::domain::object_mutation::{ObjectDependencyEdge, ObjectKind};
use crate::domain::schema::IntrospectResult;

pub fn export_schema_markdown(schema: &IntrospectResult) -> String {
    let mut out = String::from("# Schema documentation\n\n");

    out.push_str("## Tables\n\n");
    for table in &schema.tables {
        out.push_str(&format!("### `{}`.`{}`\n\n", table.schema, table.name));
        let columns: Vec<_> = schema
            .columns
            .iter()
            .filter(|c| c.schema == table.schema && c.table_name == table.name)
            .collect();
        if columns.is_empty() {
            out.push_str("_No columns._\n\n");
        } else {
            out.push_str("| Column | Type | Nullable | Default | PK |\n");
            out.push_str("| --- | --- | --- | --- | --- |\n");
            for col in columns {
                out.push_str(&format!(
                    "| `{}` | `{}` | {} | {} | {} |\n",
                    col.name,
                    col.data_type,
                    if col.nullable { "yes" } else { "no" },
                    col.default.as_deref().unwrap_or(""),
                    if col.is_primary_key { "yes" } else { "no" }
                ));
            }
            out.push('\n');
        }

        let fks: Vec<_> = schema
            .foreign_keys
            .iter()
            .filter(|fk| fk.schema == table.schema && fk.from_table == table.name)
            .collect();
        if !fks.is_empty() {
            out.push_str("Foreign keys:\n\n");
            for fk in fks {
                out.push_str(&format!(
                    "- `{}` → `{}`.`{}` ({:?} → {:?})\n",
                    fk.name, fk.to_schema, fk.to_table, fk.from_columns, fk.to_columns
                ));
            }
            out.push('\n');
        }
    }

    if !schema.views.is_empty() {
        out.push_str("## Views\n\n");
        for view in &schema.views {
            out.push_str(&format!("- `{}`.`{}`\n", view.schema, view.name));
        }
        out.push('\n');
    }

    if !schema.indexes.is_empty() {
        out.push_str("## Indexes\n\n");
        for index in &schema.indexes {
            out.push_str(&format!(
                "- `{}` on `{}`.`{}` ({})\n",
                index.name,
                index.schema,
                index.table_name,
                index.columns.join(", ")
            ));
        }
        out.push('\n');
    }

    out
}

pub fn export_schema_html(schema: &IntrospectResult) -> String {
    let md = export_schema_markdown(schema);
    format!(
        "<!DOCTYPE html><html><head><meta charset=\"utf-8\"><title>Schema docs</title>\
<style>body{{font-family:system-ui,sans-serif;margin:2rem;line-height:1.45}}\
code{{background:#f4f4f4;padding:0.1rem 0.3rem}} table{{border-collapse:collapse}}\
td,th{{border:1px solid #ccc;padding:0.35rem 0.6rem}}</style></head><body><pre>{}</pre></body></html>",
        html_escape(&md)
    )
}

pub fn collect_dependency_edges(schema: &IntrospectResult) -> Vec<ObjectDependencyEdge> {
    let mut edges = Vec::new();
    for fk in &schema.foreign_keys {
        edges.push(ObjectDependencyEdge {
            from_kind: ObjectKind::Table,
            from_schema: Some(fk.schema.clone()),
            from_name: fk.from_table.clone(),
            to_kind: ObjectKind::Table,
            to_schema: Some(fk.to_schema.clone()),
            to_name: fk.to_table.clone(),
            relation: format!("fk:{}", fk.name),
        });
    }
    for index in &schema.indexes {
        edges.push(ObjectDependencyEdge {
            from_kind: ObjectKind::Index,
            from_schema: Some(index.schema.clone()),
            from_name: index.name.clone(),
            to_kind: ObjectKind::Table,
            to_schema: Some(index.schema.clone()),
            to_name: index.table_name.clone(),
            relation: "index_on".into(),
        });
    }
    for trigger in &schema.triggers {
        edges.push(ObjectDependencyEdge {
            from_kind: ObjectKind::Trigger,
            from_schema: Some(trigger.schema.clone()),
            from_name: trigger.name.clone(),
            to_kind: ObjectKind::Table,
            to_schema: Some(trigger.schema.clone()),
            to_name: trigger.table_name.clone(),
            relation: "trigger_on".into(),
        });
    }
    for view in &schema.views {
        edges.push(ObjectDependencyEdge {
            from_kind: ObjectKind::View,
            from_schema: Some(view.schema.clone()),
            from_name: view.name.clone(),
            to_kind: ObjectKind::Schema,
            to_schema: None,
            to_name: view.schema.clone(),
            relation: "contained_in".into(),
        });
    }
    edges
}

fn html_escape(input: &str) -> String {
    input.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}
