# Tabs

Two tab-track styles that update a caller-owned selected index.

```rust
let labels = ["Overview", "Schema", "Data"];
SegmentedTabs::new(&mut selected, &labels, theme).show(ui);
UnderlineTabs::new(&mut selected, &labels, theme).show(ui);
```

`SegmentedTabs` draws a selected pill; `UnderlineTabs` draws an active underline. Both support pointer selection and focused arrow-key navigation. See [API.md](API.md) for state and limits.
