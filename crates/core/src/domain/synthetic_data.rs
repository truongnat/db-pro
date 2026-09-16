//! Deterministic synthetic row generation for dev/test seeding (#233).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ColumnGeneratorKind {
    Auto,
    Integer,
    Decimal,
    Boolean,
    Text,
    Email,
    Name,
    Uuid,
    Date,
    Timestamp,
    Null,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ColumnSpec {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
    pub is_primary_key: bool,
    pub generator: ColumnGeneratorKind,
    /// When set, values cycle through this FK key pool (deterministic).
    pub fk_values: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyntheticPlan {
    pub schema: String,
    pub table: String,
    pub columns: Vec<ColumnSpec>,
    pub row_count: u64,
    pub seed: u64,
    pub null_rate_pct: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyntheticPreview {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub seed: u64,
    pub message: String,
}

#[derive(Debug, Clone)]
struct Rng {
    state: u64,
}

impl Rng {
    fn new(seed: u64) -> Self {
        Self {
            state: seed ^ 0xA5A5_5A5A_C3C3_3C3C,
        }
    }

    fn next_u64(&mut self) -> u64 {
        // xorshift64*
        let mut x = self.state;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.state = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    fn next_usize(&mut self, max: usize) -> usize {
        if max == 0 {
            return 0;
        }
        (self.next_u64() as usize) % max
    }

    fn next_bool(&mut self, true_pct: u8) -> bool {
        (self.next_u64() % 100) < u64::from(true_pct)
    }
}

pub fn infer_generator(data_type: &str) -> ColumnGeneratorKind {
    let t = data_type.to_ascii_lowercase();
    if t.contains("uuid") || t.contains("guid") {
        ColumnGeneratorKind::Uuid
    } else if t.contains("bool") || t == "bit" {
        ColumnGeneratorKind::Boolean
    } else if t.contains("timestamp") || t.contains("datetime") {
        ColumnGeneratorKind::Timestamp
    } else if t.contains("date") {
        ColumnGeneratorKind::Date
    } else if t.contains("int") || t.contains("serial") || (t.contains("numeric") && !t.contains("(")) {
        ColumnGeneratorKind::Integer
    } else if t.contains("decimal")
        || t.contains("numeric")
        || t.contains("real")
        || t.contains("double")
        || t.contains("float")
    {
        ColumnGeneratorKind::Decimal
    } else if t.contains("email") {
        ColumnGeneratorKind::Email
    } else {
        ColumnGeneratorKind::Text
    }
}

pub fn generate_preview(plan: &SyntheticPlan, preview_limit: usize) -> Result<SyntheticPreview, String> {
    if plan.row_count == 0 {
        return Err("row_count must be > 0".into());
    }
    if plan.columns.is_empty() {
        return Err("no columns selected".into());
    }
    if plan.null_rate_pct > 100 {
        return Err("null_rate_pct must be 0..=100".into());
    }
    let limit = preview_limit.min(plan.row_count as usize).max(1);
    let rows = generate_rows(plan, limit)?;
    Ok(SyntheticPreview {
        columns: plan.columns.iter().map(|c| c.name.clone()).collect(),
        rows,
        seed: plan.seed,
        message: format!(
            "preview {}/{} rows · seed={} · null_rate={}%",
            limit, plan.row_count, plan.seed, plan.null_rate_pct
        ),
    })
}

pub fn generate_rows(plan: &SyntheticPlan, count: usize) -> Result<Vec<Vec<String>>, String> {
    let mut rng = Rng::new(plan.seed);
    let mut rows = Vec::with_capacity(count);
    for i in 0..count {
        let mut row = Vec::with_capacity(plan.columns.len());
        for col in &plan.columns {
            row.push(generate_cell(col, i as u64, plan.null_rate_pct, &mut rng)?);
        }
        rows.push(row);
    }
    Ok(rows)
}

fn generate_cell(col: &ColumnSpec, row_index: u64, null_rate: u8, rng: &mut Rng) -> Result<String, String> {
    if !col.fk_values.is_empty() {
        let idx = (row_index as usize) % col.fk_values.len();
        return Ok(col.fk_values[idx].clone());
    }
    if col.nullable && null_rate > 0 && rng.next_bool(null_rate) {
        return Ok("NULL".into());
    }
    let kind = if col.generator == ColumnGeneratorKind::Auto {
        infer_generator(&col.data_type)
    } else {
        col.generator
    };
    Ok(match kind {
        ColumnGeneratorKind::Auto => unreachable!(),
        ColumnGeneratorKind::Null => "NULL".into(),
        ColumnGeneratorKind::Integer => {
            if col.is_primary_key {
                (row_index + 1).to_string()
            } else {
                (rng.next_u64() % 10_000).to_string()
            }
        }
        ColumnGeneratorKind::Decimal => format!("{}.{:02}", rng.next_u64() % 1000, rng.next_u64() % 100),
        ColumnGeneratorKind::Boolean => if rng.next_bool(50) { "true" } else { "false" }.into(),
        ColumnGeneratorKind::Text => format!("sample_{}_{}", sanitize(&col.name), row_index + 1),
        ColumnGeneratorKind::Email => format!("user{}@example.test", row_index + 1),
        ColumnGeneratorKind::Name => {
            const NAMES: &[&str] = &["Ada", "Grace", "Linus", "Alice", "Bob", "Carol"];
            NAMES[rng.next_usize(NAMES.len())].to_owned()
        }
        ColumnGeneratorKind::Uuid => format!(
            "{:08x}-{:04x}-4{:03x}-a{:03x}-{:012x}",
            (rng.next_u64() & 0xffff_ffff) as u32,
            (rng.next_u64() & 0xffff) as u16,
            (rng.next_u64() & 0x0fff) as u16,
            (rng.next_u64() & 0x0fff) as u16,
            rng.next_u64() & 0xffff_ffff_ffff
        ),
        ColumnGeneratorKind::Date => {
            let day = 1 + (row_index % 28);
            let month = 1 + (row_index % 12);
            format!("2024-{month:02}-{day:02}")
        }
        ColumnGeneratorKind::Timestamp => {
            let day = 1 + (row_index % 28);
            let hour = row_index % 24;
            format!("2024-01-{day:02} {hour:02}:00:00")
        }
    })
}

fn sanitize(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect()
}

pub fn quote_ident(name: &str) -> String {
    format!("\"{}\"", name.replace('"', "\"\""))
}

pub fn qualify_table(schema: &str, table: &str) -> String {
    if schema.is_empty() {
        quote_ident(table)
    } else {
        format!("{}.{}", quote_ident(schema), quote_ident(table))
    }
}

pub fn render_insert_sql(plan: &SyntheticPlan, rows: &[Vec<String>]) -> Result<String, String> {
    if rows.is_empty() {
        return Err("no rows to insert".into());
    }
    let cols = plan
        .columns
        .iter()
        .map(|c| quote_ident(&c.name))
        .collect::<Vec<_>>()
        .join(", ");
    let table = qualify_table(&plan.schema, &plan.table);
    let mut out = String::new();
    out.push_str(&format!("-- synthetic seed seed={} rows={}\n", plan.seed, rows.len()));
    out.push_str("BEGIN;\n");
    for row in rows {
        if row.len() != plan.columns.len() {
            return Err("row width mismatch".into());
        }
        let values = row
            .iter()
            .enumerate()
            .map(|(i, v)| sql_literal(v, &plan.columns[i]))
            .collect::<Vec<_>>()
            .join(", ");
        out.push_str(&format!("INSERT INTO {table} ({cols}) VALUES ({values});\n"));
    }
    out.push_str("COMMIT;\n");
    Ok(out)
}

fn sql_literal(value: &str, col: &ColumnSpec) -> String {
    if value == "NULL" {
        return "NULL".into();
    }
    let kind = if col.generator == ColumnGeneratorKind::Auto {
        infer_generator(&col.data_type)
    } else {
        col.generator
    };
    match kind {
        ColumnGeneratorKind::Integer | ColumnGeneratorKind::Decimal | ColumnGeneratorKind::Boolean => value.to_owned(),
        _ => format!("'{}'", value.replace('\'', "''")),
    }
}

/// Attach FK value pools from a map of `to_table` → sample key strings.
pub fn attach_fk_pools(plan: &mut SyntheticPlan, pools: &HashMap<String, Vec<String>>) {
    for col in &mut plan.columns {
        if let Some(pool) = pools.get(&col.name) {
            col.fk_values = pool.clone();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_plan(seed: u64) -> SyntheticPlan {
        SyntheticPlan {
            schema: "public".into(),
            table: "users".into(),
            columns: vec![
                ColumnSpec {
                    name: "id".into(),
                    data_type: "integer".into(),
                    nullable: false,
                    is_primary_key: true,
                    generator: ColumnGeneratorKind::Auto,
                    fk_values: Vec::new(),
                },
                ColumnSpec {
                    name: "email".into(),
                    data_type: "text".into(),
                    nullable: false,
                    is_primary_key: false,
                    generator: ColumnGeneratorKind::Email,
                    fk_values: Vec::new(),
                },
                ColumnSpec {
                    name: "active".into(),
                    data_type: "boolean".into(),
                    nullable: false,
                    is_primary_key: false,
                    generator: ColumnGeneratorKind::Auto,
                    fk_values: Vec::new(),
                },
            ],
            row_count: 5,
            seed,
            null_rate_pct: 0,
        }
    }

    #[test]
    fn seeded_output_is_deterministic() {
        let a = generate_preview(&sample_plan(42), 5).unwrap();
        let b = generate_preview(&sample_plan(42), 5).unwrap();
        assert_eq!(a.rows, b.rows);
        let c = generate_preview(&sample_plan(99), 5).unwrap();
        assert_ne!(a.rows, c.rows);
    }

    #[test]
    fn render_insert_wraps_transaction() {
        let plan = sample_plan(1);
        let rows = generate_rows(&plan, 2).unwrap();
        let sql = render_insert_sql(&plan, &rows).unwrap();
        assert!(sql.contains("BEGIN;"));
        assert!(sql.contains("COMMIT;"));
        assert!(sql.contains("INSERT INTO \"public\".\"users\""));
        assert!(sql.contains("user1@example.test"));
    }

    #[test]
    fn fk_pool_cycles_deterministically() {
        let mut plan = sample_plan(7);
        plan.columns.push(ColumnSpec {
            name: "org_id".into(),
            data_type: "integer".into(),
            nullable: false,
            is_primary_key: false,
            generator: ColumnGeneratorKind::Integer,
            fk_values: vec!["10".into(), "20".into()],
        });
        let rows = generate_rows(&plan, 4).unwrap();
        assert_eq!(rows[0].last().unwrap(), "10");
        assert_eq!(rows[1].last().unwrap(), "20");
        assert_eq!(rows[2].last().unwrap(), "10");
    }
}
