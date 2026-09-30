/// Padding added around the inline code text (horizontal, vertical), in egui points.
pub const INLINE_CODE_PAD: egui::Vec2 = egui::Vec2::new(6.0, 2.0);
/// Font size for inline code snippet text, in egui points.
pub const INLINE_CODE_FONT_SIZE: f32 = 12.0;

/// Header bar height for the code block container.
pub const CODE_BLOCK_HEADER_HEIGHT: f32 = 32.0;
/// Corner radius for code block header top corners and outer frame, in egui points.
pub const CODE_BLOCK_CORNER_RADIUS: f32 = 8.0;
/// Language badge font size in the header bar.
pub const CODE_BLOCK_LANG_FONT_SIZE: f32 = 11.0;
/// Copy button size (width, height) in the header bar.
pub const CODE_BLOCK_COPY_BTN_SIZE: egui::Vec2 = egui::Vec2::new(68.0, 22.0);
/// Font size for copy button icon.
pub const CODE_BLOCK_COPY_ICON_SIZE: f32 = 11.0;
/// Font size for copy button label.
pub const CODE_BLOCK_COPY_LABEL_SIZE: f32 = 11.5;
/// Duration in seconds for which the "Copied" feedback state is displayed.
pub const CODE_BLOCK_COPIED_FEEDBACK_SECS: f64 = 2.0;

/// Top and bottom vertical padding space in code lines area.
pub const CODE_BLOCK_BODY_PADDING_Y: f32 = 8.0;
/// Row height for each individual line in a code block.
pub const CODE_BLOCK_LINE_HEIGHT: f32 = 20.0;
/// Monospace font size for line numbers.
pub const CODE_BLOCK_LINE_NUM_FONT_SIZE: f32 = 12.0;
/// Monospace font size for code content text.
pub const CODE_BLOCK_CODE_FONT_SIZE: f32 = 12.5;
/// Base gutter width when line numbers are hidden.
pub const CODE_BLOCK_NO_GUTTER_WIDTH: f32 = 12.0;
/// Width per digit character in line number calculation.
pub const CODE_BLOCK_DIGIT_CHAR_WIDTH: f32 = 8.0;
/// Padding added to the line numbers gutter.
pub const CODE_BLOCK_GUTTER_EXTRA_PAD: f32 = 20.0;
/// Horizontal inset for the copy button from the header edge.
pub const CODE_BLOCK_COPY_BTN_INSET: f32 = 12.0;
/// Minimum gap between the language label and the copy button.
pub const CODE_BLOCK_HEADER_LABEL_GAP: f32 = 8.0;
