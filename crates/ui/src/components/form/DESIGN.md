# Form design

`FormField` composes a visual `Label` with the shared `Input` widget. It sets a stable field ID from the label by default, with `.id_salt(...)` for duplicate labels, and combines label, required state, and visible feedback into the editable control's semantic name. Error text takes precedence over helper text. `Label` remains a display element; the nested Input is the focus target.

`FormState` is a caller-owned lifecycle controller for registered rules, errors, touched/dirty flags, submit state, and error visibility. `FieldRule` validates a supplied string and returns either success or its configured message. Validation modes only decide when existing errors should be shown; callers choose validation events and provide current field values. `handle_submit` marks supplied fields touched, validates them, and invokes the callback only on success.

Form rendering is proportional to its child widgets. Validation cost is proportional to supplied fields times their registered rules. The component has no database or persistence behavior; ownership of form values and submission actions stays with the caller. Public `form::field`, `form::rules`, and `form::state` modules are compatibility facades.
