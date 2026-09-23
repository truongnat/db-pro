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
