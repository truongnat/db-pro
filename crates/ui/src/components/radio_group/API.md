# RadioGroup API

```rust
let mut isolation = 0;
let changed = RadioGroup::new(theme)
    .label("Transaction isolation")
    .option(RadioGroupOption::new(0, "Read committed"))
    .option(RadioGroupOption::new(1, "Repeatable read").description("Stable snapshot"))
    .option(RadioGroupOption::new(2, "Serializable").disabled(false))
    .show(ui, &mut isolation);
```

- `RadioGroupOption::new(value, label)` accepts a cloneable, partially comparable value; `.description(text)` adds supporting text and `.disabled(bool)` disables the option.
- `RadioGroup::new(theme)` creates a vertical group. `.horizontal(bool)` switches to wrapping horizontal layout; `.label(text)` names the group; `.option(option)` appends a choice.
- `.show(ui, &mut selected) -> Option<T>` mutates `selected` when an enabled choice changes and returns `Some(new_value)` for that change, otherwise `None`.

Pointer click chooses an option. Arrow keys move focus and selection with wraparound, skipping disabled entries; the axis follows layout orientation. Space/Enter activates the focused option. Empty groups return `None`. If all options are disabled, keyboard movement does not select anything. Values must implement `Clone + PartialEq` and uniquely identify choices for reliable selection.

## Layers

`ui.rs` owns egui layout and response routing; `handler.rs` owns selection eligibility, keyboard movement, and spacing; `config.rs` owns group spacing.
