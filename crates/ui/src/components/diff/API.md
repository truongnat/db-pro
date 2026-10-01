# Diff API

The diff API is re-exported from `components::diff` and `components`.

- `DiffLine::context(old_num, new_num, content)`, `added(new_num, content)`, and `removed(old_num, content)` construct public rows. Fields `line_type`, `old_line_num`, `new_line_num`, and `content` are public for direct construction.
- `DiffLineType` is `Context`, `Added`, or `Removed`.
- `DiffViewer::new(title, &lines, theme).show(ui) -> Response` renders rows and summary counts. Empty rows render “No changes to display.”; long content is accessible by horizontal scrolling.
- `count_diff_changes(&lines) -> (added, removed)`, `format_diff_stats(added, removed) -> String`, and `diff_line_visual(type, &theme) -> DiffLineVisual` expose the corresponding decisions.
- `DiffLineVisual` returns background, marker, marker color and text color.
- All `DIFF_*` layout/typography constants in `config.rs` are publicly re-exported; they are implementation measurements for callers that need coordinated custom rendering.

The component displays supplied changes; it does not compare two texts or apply patches. It renders every supplied line each frame. Very large diffs can therefore cost time and memory proportional to their row and text count.
