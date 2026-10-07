# Findings

Baseline SHA: 3dd988e2ff06ac71942c7ec331acf66389d3c5cb. Final uncommitted source snapshot/hash will be stored under evidence/. Prior Data Grid/footer edits remain in the shared checkout and are not rewritten.

P2: Query row limit used egui::ComboBox plus selectable_value, bypassing shared Select. Screenshot shows selected item's dark text on strong blue. Shared Select already paints selected option with theme.accent_soft, theme.accent and a checkmark; reuse it rather than overriding global theme or duplicating popup logic. Adapter preserves 100/500/1000/None and existing unknown-cap behavior.

P2: Welcome duplicated composer, SQL fixtures, connection catalog, shell start actions and draft preview. Owner explicitly asked for just an attractive introduction. Static centered icon/name/copy/provider line fulfills that scope; removed the obsolete private Welcome action and prompt state. Query documents and connection catalog remain owned by their existing state modules.

Reusable lessons: caller migration matters after shared component styling changes; Welcome should not duplicate workspace controls when a simple introduction is requested. Recorded locally; global memory not modified because no memory-write authorization.

## Owner follow-up: toolbar variants

The owner rejected the form-sized row-limit trigger in the Query toolbar. Shared Select previously had fixed 38px input height, 14px trigger text, and outlined chrome. Added public SelectSize (Default / Sm) and SelectVariant (Outline / Ghost), with defaults retaining existing form callers. Sm reuses Button's 28px height, 11.5px type, icon and padding tokens; Ghost has transparent idle chrome, semantic hover fill and open/focus feedback. Query uses Sm + Ghost at 108px width. Popup navigation and option selection logic remain unchanged. Geometry tests compare compact Select to adjacent Button and preserve default form height.

Lesson: using the shared component alone does not establish visual consistency; its context-appropriate size/chrome must be supported centrally and verified in the real parent toolbar. Prior captures are historical; current variant screenshots and hashes will supersede them in the manifest.
