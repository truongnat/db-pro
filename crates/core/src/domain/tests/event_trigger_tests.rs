//! Tests for `../event_trigger.rs` (moved out of the source file).
use super::*;

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

#[test]
fn accepts_simple_zero_argument_function_references() {
    for function_ref in [
        "on_ddl",
        "on_ddl()",
        "public.on_ddl",
        "public.on_ddl()",
        "PUBLIC.On_Ddl()",
    ] {
        assert!(validate_function_ref(function_ref).is_ok(), "{function_ref}");
    }
}

#[test]
fn rejects_arguments_and_malformed_function_references() {
    for function_ref in [
        "",
        ".",
        ".on_ddl",
        "public.",
        "a.b.c",
        "a..b",
        "1func",
        "1schema.func",
        "func(,)",
        "func()()",
        "func(",
        "func)",
        "func(arg)",
        "func(a,b)",
        "func(())",
        "func ()",
        "func;DROP",
        "func--comment",
        "\"func\"",
        "public. func",
        "()",
    ] {
        assert!(validate_function_ref(function_ref).is_err(), "{function_ref}");
        assert!(
            preview_create_event_trigger("audit_ddl", "ddl_command_end", function_ref, "").is_err(),
            "{function_ref}"
        );
    }
}

#[test]
fn rejects_overlong_function_identifier_components() {
    assert!(validate_function_ref(&"a".repeat(64)).is_err());
    assert!(validate_function_ref(&format!("{}.func", "a".repeat(64))).is_err());
    assert!(validate_function_ref(&format!("public.{}()", "a".repeat(64))).is_err());
    assert!(validate_function_ref(&format!("{}.{}()", "a".repeat(63), "b".repeat(63))).is_ok());
}
