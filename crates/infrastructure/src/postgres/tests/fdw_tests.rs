//! Tests for `../fdw.rs` (moved out of the source file).
use super::*;

#[test]
fn parse_options_redacts_password() {
    let opts = parse_options("host=db,password=secret,port=5432".into());
    assert_eq!(opts.len(), 3);
    assert!(opts
        .iter()
        .any(|o| o.key == "password" && o.sensitive && o.value.is_empty()));
    assert!(db_pro_core::domain::fdw::quote_ident("ok_name").is_ok());
}
