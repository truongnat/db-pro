# Code design

`InlineCode` measures one monospace galley, allocates its padded size, and paints a themed chip. `CodeBlock` measures the source lines, computes a line-number gutter, then paints a header, clipped language label, copy control, and code body. Copy clicks write the source string to egui's clipboard output and record the frame time; handler functions derive the visible Copy/Copied state and its remaining feedback duration.

The copy control is keyboard-accessible through egui's button interaction and reports its current action label in widget metadata. `InlineCode` is presentation-only. Shared radius/stroke values use canonical tokens; code-specific typography and dimensions remain local. There is no syntax highlighting or line wrapping.

Rendering is linear in the number of source lines and text length because each line is measured/painted with its line number. Large files should use the SQL editor or another virtualized surface; CodeBlock creates layout for every line each frame. Long lines remain unwrapped and may extend horizontally within the allocated body.
