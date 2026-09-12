use crate::components::input::Input;
use crate::DbProTheme;
use egui::{Response, RichText, Ui};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

/// Validation execution triggers, inspired by React Hook Form.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ValidationMode {
    /// Validate on change once field has been touched, or on blur (Default & recommended UX).
    #[default]
    OnTouched,
    /// Validate on every keystroke/change.
    OnChange,
    /// Validate only when a field loses focus.
    OnBlur,
    /// Validate only when the form submission is triggered.
    OnSubmit,
}

/// Type alias for custom field validator functions.
pub type CustomValidator = Arc<dyn Fn(&str) -> Result<(), String> + Send + Sync>;

/// Validation rule for a form field.
#[derive(Clone)]
pub enum FieldRule {
    Required(String),
    MinLength(usize, String),
    MaxLength(usize, String),
    Email(String),
    Port(String),
    Hostname(String),
    Numeric(String),
    Custom(String, CustomValidator),
}

impl std::fmt::Debug for FieldRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Required(msg) => write!(f, "Required({msg})"),
            Self::MinLength(min, msg) => write!(f, "MinLength({min}, {msg})"),
            Self::MaxLength(max, msg) => write!(f, "MaxLength({max}, {msg})"),
            Self::Email(msg) => write!(f, "Email({msg})"),
            Self::Port(msg) => write!(f, "Port({msg})"),
            Self::Hostname(msg) => write!(f, "Hostname({msg})"),
            Self::Numeric(msg) => write!(f, "Numeric({msg})"),
            Self::Custom(desc, _) => write!(f, "Custom({desc})"),
        }
    }
}

impl FieldRule {
    pub fn required(message: impl Into<String>) -> Self {
        Self::Required(message.into())
    }

    pub fn min_length(min: usize, message: impl Into<String>) -> Self {
        Self::MinLength(min, message.into())
    }

    pub fn max_length(max: usize, message: impl Into<String>) -> Self {
        Self::MaxLength(max, message.into())
    }

    pub fn email(message: impl Into<String>) -> Self {
        Self::Email(message.into())
    }

    pub fn port(message: impl Into<String>) -> Self {
        Self::Port(message.into())
    }

    pub fn hostname(message: impl Into<String>) -> Self {
        Self::Hostname(message.into())
    }

    pub fn numeric(message: impl Into<String>) -> Self {
        Self::Numeric(message.into())
    }

    pub fn custom<F>(desc: impl Into<String>, validator: F) -> Self
    where
        F: Fn(&str) -> Result<(), String> + Send + Sync + 'static,
    {
        Self::Custom(desc.into(), Arc::new(validator))
    }

    pub fn validate(&self, value: &str) -> Result<(), String> {
        match self {
            FieldRule::Required(msg) => {
                if value.trim().is_empty() {
                    Err(msg.clone())
                } else {
                    Ok(())
                }
            }
            FieldRule::MinLength(min, msg) => {
                if !value.is_empty() && value.chars().count() < *min {
                    Err(msg.clone())
                } else {
                    Ok(())
                }
            }
            FieldRule::MaxLength(max, msg) => {
                if value.chars().count() > *max {
                    Err(msg.clone())
                } else {
                    Ok(())
                }
            }
            FieldRule::Email(msg) => {
                if !value.is_empty() && (!value.contains('@') || !value.contains('.')) {
                    Err(msg.clone())
                } else {
                    Ok(())
                }
            }
            FieldRule::Port(msg) => {
                if !value.is_empty() {
                    match value.trim().parse::<u16>() {
                        Ok(p) if p > 0 => Ok(()),
                        _ => Err(msg.clone()),
                    }
                } else {
                    Ok(())
                }
            }
            FieldRule::Hostname(msg) => {
                if !value.is_empty() {
                    let trimmed = value.trim();
                    if trimmed.is_empty() || trimmed.contains(' ') || trimmed.starts_with('.') || trimmed.ends_with('.')
                    {
                        Err(msg.clone())
                    } else {
                        Ok(())
                    }
                } else {
                    Ok(())
                }
            }
            FieldRule::Numeric(msg) => {
                if !value.is_empty() && value.trim().parse::<f64>().is_err() {
                    Err(msg.clone())
                } else {
                    Ok(())
                }
            }
            FieldRule::Custom(_, validator) => validator(value),
        }
    }
}

/// Central form state controller managing field rules, errors, touched, and dirty tracking.
#[derive(Debug, Clone, Default)]
pub struct FormState {
    pub errors: HashMap<String, String>,
    pub touched: HashSet<String>,
    pub dirty: HashSet<String>,
    pub submitted: bool,
    pub mode: ValidationMode,
    rules: HashMap<String, Vec<FieldRule>>,
}

impl FormState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_mode(mode: ValidationMode) -> Self {
        Self {
            mode,
            ..Default::default()
        }
    }

    /// Registers a field with its validation rules.
    pub fn register(&mut self, field: &str, rules: Vec<FieldRule>) -> &mut Self {
        self.rules.insert(field.to_string(), rules);
        self
    }

    pub fn is_required(&self, field: &str) -> bool {
        self.rules
            .get(field)
            .map(|r| r.iter().any(|rule| matches!(rule, FieldRule::Required(_))))
            .unwrap_or(false)
    }

    pub fn validate_field(&mut self, field: &str, value: &str) -> bool {
        if let Some(rules) = self.rules.get(field).cloned() {
            for rule in &rules {
                if let Err(err) = rule.validate(value) {
                    self.errors.insert(field.to_string(), err);
                    return false;
                }
            }
        }
        self.errors.remove(field);
        true
    }

    pub fn validate_all(&mut self, fields: &[(&str, &str)]) -> bool {
        let mut all_valid = true;
        for (name, val) in fields {
            if !self.validate_field(name, val) {
                all_valid = false;
            }
        }
        all_valid
    }

    pub fn touch(&mut self, field: &str) {
        self.touched.insert(field.to_string());
    }

    pub fn set_dirty(&mut self, field: &str) {
        self.dirty.insert(field.to_string());
    }

    pub fn is_touched(&self, field: &str) -> bool {
        self.touched.contains(field)
    }

    pub fn is_dirty(&self, field: &str) -> bool {
        self.dirty.contains(field)
    }

    pub fn get_error(&self, field: &str) -> Option<&str> {
        self.errors.get(field).map(|s| s.as_str())
    }

    pub fn has_error(&self, field: &str) -> bool {
        self.errors.contains_key(field)
    }

    pub fn should_show_error(&self, field: &str) -> bool {
        if self.errors.contains_key(field) {
            match self.mode {
                ValidationMode::OnSubmit => self.submitted,
                ValidationMode::OnBlur => self.is_touched(field) || self.submitted,
                ValidationMode::OnChange => self.is_dirty(field) || self.submitted,
                ValidationMode::OnTouched => self.is_touched(field) || self.submitted,
            }
        } else {
            false
        }
    }

    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    /// Checks if all fields pass their registered validation rules without mutating state.
    pub fn check_validity(&self, fields: &[(&str, &str)]) -> bool {
        for (name, val) in fields {
            if let Some(rules) = self.rules.get(*name) {
                for rule in rules {
                    if rule.validate(val).is_err() {
                        return false;
                    }
                }
            }
        }
        true
    }

    pub fn error_count(&self) -> usize {
        self.errors.len()
    }

    pub fn reset(&mut self) {
        self.errors.clear();
        self.touched.clear();
        self.dirty.clear();
        self.submitted = false;
    }

    /// Handles form submission: marks all fields as touched, triggers validation,
    /// and invokes `on_valid` only if all fields pass validation.
    pub fn handle_submit<F: FnOnce()>(&mut self, fields: &[(&str, &str)], on_valid: F) -> bool {
        self.submitted = true;
        for (name, _) in fields {
            self.touched.insert(name.to_string());
        }
        let valid = self.validate_all(fields);
        if valid {
            on_valid();
        }
        valid
    }
}

pub struct Label<'a> {
    text: &'a str,
    required: bool,
    enabled: bool,
    theme: DbProTheme,
}

impl<'a> Label<'a> {
    pub fn new(text: &'a str, theme: DbProTheme) -> Self {
        Self {
            text,
            required: false,
            enabled: true,
            theme,
        }
    }

    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let color = if self.enabled {
            self.theme.text_secondary
        } else {
            self.theme.text_disabled
        };
        ui.horizontal(|ui| {
            let response = ui.label(RichText::new(self.text).size(12.0).strong().color(color));
            if self.required {
                ui.label(RichText::new("*").size(12.0).strong().color(self.theme.danger));
            }
            response
        })
        .inner
    }
}

pub struct FormField<'a> {
    label: &'a str,
    value: &'a mut String,
    placeholder: &'a str,
    helper_text: Option<&'a str>,
    error_text: Option<&'a str>,
    required: bool,
    enabled: bool,
    theme: DbProTheme,
}

impl<'a> FormField<'a> {
    pub fn new(label: &'a str, value: &'a mut String, placeholder: &'a str, theme: DbProTheme) -> Self {
        Self {
            label,
            value,
            placeholder,
            helper_text: None,
            error_text: None,
            required: false,
            enabled: true,
            theme,
        }
    }

    pub fn helper_text(mut self, text: &'a str) -> Self {
        self.helper_text = Some(text);
        self
    }

    pub fn error_text(mut self, text: &'a str) -> Self {
        self.error_text = Some(text);
        self
    }

    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {
            Label::new(self.label, self.theme)
                .required(self.required)
                .enabled(self.enabled)
                .show(ui);
            ui.add_space(4.0);
            let mut field = Input::new(self.value, self.placeholder, self.theme).enabled(self.enabled);
            if let Some(error) = self.error_text {
                field = field.error_text(error);
            } else if let Some(helper) = self.helper_text {
                field = field.helper_text(helper);
            }
            field.show(ui)
        })
        .inner
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
