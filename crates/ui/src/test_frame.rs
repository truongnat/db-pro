//! Runs one egui pass for a unit test and clears the produced texture
//! deltas so dropping the `egui::FullOutput` stays safe: epaint ≥0.36
//! `debug_assert`s when a non-empty `TexturesDelta` is dropped unapplied.
pub(crate) fn frame(
    ctx: &impl core::borrow::Borrow<egui::Context>,
    input: egui::RawInput,
    ui: impl FnMut(&mut egui::Ui),
) -> egui::FullOutput {
    let mut output = ctx.borrow().run_ui(input, ui);
    output.textures_delta.clear();
    output
}

/// Same as [`frame`] for `Context::end_pass` results.
#[allow(dead_code)]
pub(crate) fn finish_pass(ctx: &egui::Context) -> egui::FullOutput {
    let mut output = ctx.end_pass();
    output.textures_delta.clear();
    output
}
