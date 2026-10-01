# Form API

Import common types from `db_pro_ui::components::form`; compatibility module paths remain available under `form::field`, `form::rules`, and `form::state`.

```rust
let mut state = FormState::with_mode(ValidationMode::OnTouched);
state.register("host", vec![FieldRule::required("Host is required")]);
let valid = state.handle_submit(&[("host", host)], || connect());

FormField::new("Host", &mut host, "localhost", theme)
    .required(true)
    .error_text(state.get_error("host").unwrap_or(""))
    .show(ui);
```

## Rendering

- `Label::new(text, theme)` supports `.required(bool)`, `.enabled(bool)`, and `.show(ui) -> Response`.
- `FormField::new(label, value: &mut String, placeholder, theme)` supports `.id_salt(hashable)`, `.helper_text(text)`, `.error_text(text)`, `.required(bool)`, `.enabled(bool)`, and `.show(ui) -> Response`.
- Input accessibility name contains the field label, required context, and displayed helper/error context. Error text wins; blank context is omitted. The label is the default ID salt; use a unique salt for repeated labels.

## Validation

- `ValidationMode`: `OnTouched` (default), `OnChange`, `OnBlur`, or `OnSubmit` controls error visibility only. `OnBlur` and `OnTouched` share the touched flag; callers decide when to call `touch`.
- `FieldRule` constructors: `required(message)`, `min_length(min,message)`, `max_length(max,message)`, `email(message)`, `port(message)`, `hostname(message)`, `numeric(message)`, and `custom(description, validator)`. `validate(value) -> Result<(), String>` runs one rule.
- `CustomValidator` is a thread-safe `Arc` callback from `&str` to `Result<(), String>`.
- `FormState::new()` / `with_mode(mode)` create state. `register(field,rules)` sets rules. `validate_field(field,value)` and `validate_all(fields)` mutate errors and return validity. `check_validity(fields)` checks without mutating state.
- `touch`, `set_dirty`, `is_touched`, `is_dirty`, `is_required`, `get_error`, `has_error`, `should_show_error`, `is_valid`, and `error_count` expose lifecycle state. `reset()` clears errors/touched/dirty/submitted state but keeps registered rules.
- `handle_submit(fields,on_valid) -> bool` marks all supplied fields touched, validates them, then invokes `on_valid` only when all pass. Values must be current and are passed by the caller as `&[(&str,&str)]`.

`FormState` exposes `errors`, `touched`, `dirty`, `submitted`, and `mode` as public fields; registered rules remain private and are changed through `register`.

Validation is intentionally lightweight: Email checks for `@` and `.`, Hostname checks a small set of invalid forms, Numeric parses `f64`, Port accepts integers in `1..=65535`, and optional empty values pass non-required rules. Pair a `required` rule with other constraints when emptiness is invalid. No form value is stored in `FormState`.
