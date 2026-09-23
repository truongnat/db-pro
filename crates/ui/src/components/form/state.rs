use super::rules::{FieldRule, ValidationMode};
use std::collections::{HashMap, HashSet};

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
