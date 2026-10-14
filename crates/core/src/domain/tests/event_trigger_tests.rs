//! Tests for `../event_trigger.rs` (moved out of the source file).
use super::*;

#[test]
fn create_with_tags() {
    let sql = preview_create_event_trigger(
        "audit_ddl",
        "ddl_command_end",
        "public.note_ddl()",
        "CREATE TABLE, ALTER TABLE",
    )
    .unwrap();
    assert!(sql.contains("CREATE EVENT TRIGGER \"audit_ddl\" ON ddl_command_end"));
    assert!(sql.contains("WHEN TAG IN ('CREATE TABLE', 'ALTER TABLE')"));
    assert!(sql.contains("EXECUTE FUNCTION public.note_ddl()"));
}

#[test]
fn alter_and_drop() {
    assert_eq!(
        preview_alter_event_trigger("t1", "disable").unwrap(),
        "ALTER EVENT TRIGGER \"t1\" DISABLE;"
    );
    assert_eq!(
        preview_drop_event_trigger("t1").unwrap(),
        "DROP EVENT TRIGGER IF EXISTS \"t1\";"
    );
}

#[test]
fn rejects_bad_event() {
    assert!(validate_event("after_insert").is_err());
}
