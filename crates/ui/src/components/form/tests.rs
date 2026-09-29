use super::rules::FieldRule;
use super::state::FormState;

#[test]
fn test_field_rules_validation() {
    let req = FieldRule::required("Field is required");
    assert!(req.validate("hello").is_ok());
    assert!(req.validate("   ").is_err());

    let min = FieldRule::min_length(4, "Min 4 characters");
    assert!(min.validate("1234").is_ok());
    assert!(min.validate("123").is_err());

    let email = FieldRule::email("Invalid email address");
    assert!(email.validate("user@example.com").is_ok());
    assert!(email.validate("invalid-email").is_err());

    let port = FieldRule::port("Must be valid port (1-65535)");
    assert!(port.validate("5432").is_ok());
    assert!(port.validate("0").is_err());
    assert!(port.validate("99999").is_err());
    assert!(port.validate("abc").is_err());
}

#[test]
fn test_legacy_field_module_reexports_remain_available() {
    let theme = crate::DbProTheme::light();
    let _label = super::field::Label::new("Host", theme);
    let mut value = String::new();
    let _field = super::field::FormField::new("Host", &mut value, "db.internal", theme);
}

#[test]
fn test_compose_access_label_includes_required_and_helper_context() {
    assert_eq!(
        super::handler::compose_access_label("Name", true, Some("Shown to users"), None),
        "Name, required. Shown to users"
    );
}

#[test]
fn test_compose_access_label_error_takes_precedence_over_helper() {
    assert_eq!(
        super::handler::compose_access_label("Name", false, Some("Hint"), Some("Invalid")),
        "Name. Error: Invalid"
    );
}

#[test]
fn test_compose_access_label_without_optional_context() {
    assert_eq!(super::handler::compose_access_label("Name", false, None, None), "Name");
}

#[test]
fn test_compose_access_label_omits_blank_context() {
    assert_eq!(
        super::handler::compose_access_label("Name", false, Some("  \t"), None),
        "Name"
    );
    assert_eq!(
        super::handler::compose_access_label("Name", false, Some("Hint"), Some("\n ")),
        "Name. Hint"
    );
}

#[test]
fn test_form_field_supports_unique_id_salt() {
    let theme = crate::DbProTheme::light();
    let mut first = String::new();
    let _field = super::ui::FormField::new("Name", &mut first, "", theme).id_salt("first");
}

#[test]
fn test_should_show_error_all_validation_modes_and_states() {
    for mode in [
        super::rules::ValidationMode::OnTouched,
        super::rules::ValidationMode::OnBlur,
        super::rules::ValidationMode::OnChange,
        super::rules::ValidationMode::OnSubmit,
    ] {
        let mut form = FormState::with_mode(mode);
        form.errors.insert("name".into(), "Invalid".into());
        assert!(!form.should_show_error("name"));
        form.touched.insert("name".into());
        assert_eq!(
            form.should_show_error("name"),
            matches!(
                mode,
                super::rules::ValidationMode::OnTouched | super::rules::ValidationMode::OnBlur
            )
        );
        form.touched.clear();
        form.dirty.insert("name".into());
        assert_eq!(
            form.should_show_error("name"),
            matches!(mode, super::rules::ValidationMode::OnChange)
        );
        form.submitted = true;
        assert!(form.should_show_error("name"));
    }
    let form = FormState::new();
    assert!(!form.should_show_error("missing"));
}

#[test]
fn test_form_state_lifecycle() {
    let mut form = FormState::new();
    form.register(
        "username",
        vec![FieldRule::required("Required"), FieldRule::min_length(3, "Min 3")],
    );
    form.register(
        "port",
        vec![FieldRule::required("Required"), FieldRule::port("Invalid port")],
    );

    assert!(form.is_required("username"));
    assert!(form.is_required("port"));

    // Validate individual field
    assert!(!form.validate_field("username", "ab"));
    assert_eq!(form.get_error("username"), Some("Min 3"));

    assert!(form.validate_field("username", "alice"));
    assert_eq!(form.get_error("username"), None);

    // Submit handling
    let mut submitted_flag = false;
    let valid = form.handle_submit(&[("username", "alice"), ("port", "invalid")], || {
        submitted_flag = true;
    });

    assert!(!valid);
    assert!(!submitted_flag);
    assert!(form.is_touched("port"));
    assert!(form.should_show_error("port"));
    assert_eq!(form.get_error("port"), Some("Invalid port"));

    // Valid submit
    let valid = form.handle_submit(&[("username", "alice"), ("port", "5432")], || {
        submitted_flag = true;
    });
    assert!(valid);
    assert!(submitted_flag);
    assert!(form.is_valid());
}
