# Form

`Form` provides native egui labels, text fields, and validation state for database forms.

## Public API

- `FormField` and `Label`: builders for rendered form controls.
- `FormState`: registration, validation, touched/dirty tracking, and submission.
- `FieldRule`, `CustomValidator`, and `ValidationMode`: validation configuration.

Validation is lifecycle-driven: callers mark fields touched or dirty, call `validate_field`
at the appropriate event, and choose when to render `get_error`. `ValidationMode` only gates
error visibility; it does not invoke validators. `OnBlur` and `OnTouched` share the touched
flag, so callers distinguish them by when they call `touch` (typically on focus loss for
`OnBlur`, or first interaction for `OnTouched`). `handle_submit` marks fields touched and
validates all values; callers remain responsible for supplying current values and rendering errors.

```rust
let mut state = FormState::with_mode(ValidationMode::OnTouched);
state.register("host", vec![FieldRule::required("Host is required")]);
let valid = state.handle_submit(&[("host", host)], || connect());
if state.should_show_error("host") {
    // Pass state.get_error("host") to FormField::error_text.
}
```

`form::field::{FormField, Label}`, `form::rules::*`, and `form::state::FormState` remain
supported compatibility paths. `FormField` includes required context and the displayed feedback
context in the input's semantic accessibility label: error text takes precedence over helper text,
with blank/whitespace-only context omitted. Labels are the default input ID salt; repeated labels
can provide a unique salt with `.id_salt("connection-host")`.
