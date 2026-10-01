# SqlEditorToolbar API

`SqlEditorToolbar` and `SqlEditorAction` are re-exported through `components::sql_editor` and `components`.

`SqlEditorToolbar::new(is_running, has_selection, theme).show(ui) -> Option<SqlEditorAction>` takes current state and returns an action intent only when a button is clicked. `SqlEditorAction` variants are `RunQuery`, `RunSelection`, `ExplainQuery`, `FormatSql`, `CancelQuery`, and `AskAi`.

When idle, the toolbar shows Run, Explain, Format, Ask AI, and Run Selection only if `has_selection` is true. While running, Cancel replaces Run/Explain/Format/Run Selection; Ask AI remains available. The caller owns query execution, cancellation, formatting, provider capability checks, and result/error state. Button accessible descriptions include shortcut hints where defined. The component does not parse SQL or infer selection itself.
