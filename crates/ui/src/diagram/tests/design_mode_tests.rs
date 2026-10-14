//! Tests for `../design_mode.rs` (moved out of the source file).
use super::*;

#[test]
fn undo_redo_is_deterministic() {
    let mut state = DesignModeState::default();
    state.add_draft_table("public", "orders");
    assert_eq!(state.draft.tables.len(), 1);
    state.undo();
    assert!(state.draft.tables.is_empty());
    state.redo();
    assert_eq!(state.draft.tables[0].name, "orders");
}

#[test]
fn plan_creates_table_sql() {
    let mut draft = DesignDraft {
        schema_fingerprint: "abc".into(),
        ..DesignDraft::default()
    };
    draft.tables.push(DraftTable {
        schema: "public".into(),
        name: "draft_items".into(),
        columns: vec![
            DraftColumn {
                name: "id".into(),
                data_type: "integer".into(),
                nullable: false,
                is_pk: true,
                is_unique: false,
            },
            DraftColumn {
                name: "title".into(),
                data_type: "text".into(),
                nullable: false,
                is_pk: false,
                is_unique: false,
            },
        ],
    });
    let previews = plan_design_draft(&draft, "PostgreSQL").unwrap();
    let (sql, fp, _) = merge_preview_sql(&previews);
    assert!(sql.to_ascii_lowercase().contains("create table"));
    assert!(sql.contains("draft_items"));
    assert!(!fp.is_empty());
}

#[test]
fn design_draft_quotes_identifiers_for_the_driver() {
    let mut draft = DesignDraft::default();
    draft.tables.push(DraftTable {
        schema: "app".into(),
        name: "widgets".into(),
        columns: vec![DraftColumn {
            name: "id".into(),
            data_type: "int".into(),
            nullable: false,
            is_pk: true,
            is_unique: false,
        }],
    });
    let previews = plan_design_draft(&draft, "mysql").unwrap();
    let (sql, _, _) = merge_preview_sql(&previews);
    assert!(sql.contains("`app`.`widgets`"), "{sql}");
    assert!(sql.contains("`id`"), "{sql}");
    assert!(!sql.contains("\"widgets\""), "{sql}");
}

#[test]
fn stale_fingerprint_detected() {
    let mut state = DesignModeState::default();
    state.set_schema_fingerprint("old".into());
    assert!(state.fingerprint_stale("new"));
    assert!(!state.fingerprint_stale("old"));
}
