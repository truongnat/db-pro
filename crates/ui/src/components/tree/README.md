# Tree

Native database tree component for the egui UI.

## Layers

- `ui.rs`: public node builder, interaction, and painting.
- `traversal.rs`: animated nested-content traversal and clipping.
- `geometry.rs`: pure row and position calculations.
- `config.rs`: component-owned layout constants.
- `mod.rs`: stable public API re-exports.

`DatabaseTreeNode` and `reveal_children` retain the API of the former
`components/tree.rs`. Rows remain clickable, optional expansion state is toggled
on click, and nested rows use animated clipping.
