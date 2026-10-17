//! Tests for `../cross_connection.rs` (moved out of the source file).
use super::*;

#[test]
fn quote_ident_wraps_and_doubles_inner_quotes() {
    assert_eq!(quote_ident("orders"), "\"orders\"");
    assert_eq!(quote_ident("Mixed Case"), "\"Mixed Case\"");
    assert_eq!(
        quote_ident("a.b"),
        "\"a.b\"",
        "dots inside a name must not split the qualifier"
    );
    assert_eq!(quote_ident("we\"ird"), "\"we\"\"ird\"");
}

#[test]
fn dependency_lookup_filters_by_oid_not_like_pattern() {
    // Regression for #508: the query must compare dependency OIDs against the resolved
    // object — `LIKE $1 || '.%'` matched regclass text that drops the schema under
    // search_path and treated `%`/`_` in user names as wildcards.
    let source = include_str!("../cross_connection.rs");
    let start = source
        .find("pub async fn get_object_dependencies")
        .expect("dependency lookup");
    let rest = &source[start..];
    let end = rest.find("\npub async fn ").expect("next fn follows");
    let body = &rest[..end];

    assert!(
        body.contains("d.objid = to_regclass($1)"),
        "filter must be OID equality on a resolved regclass"
    );
    assert!(!body.contains("LIKE"), "no LIKE wildcards on user names");
    assert!(!body.contains("$1 ||"), "no text pattern concatenation");
    assert!(
        body.contains("quote_ident(schema)") && body.contains("quote_ident(object_name)"),
        "qualified name is quoted before binding so it stays schema.object"
    );
}

#[test]
fn qualified_name_binds_schema_and_object_verbatim() {
    // `%` and `_` must reach the server as literal characters inside one quoted part.
    let qualified = format!("{}.{}", quote_ident("my%schema"), quote_ident("tab_le"));
    assert_eq!(qualified, "\"my%schema\".\"tab_le\"");
}
