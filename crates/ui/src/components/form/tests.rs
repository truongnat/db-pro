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
