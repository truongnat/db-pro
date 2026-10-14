//! Tests for `../synthetic_data.rs` (moved out of the source file).
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
