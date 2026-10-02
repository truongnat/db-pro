# Checklist

- [x] Shared Button whole-shape transform; geometry regression test covers centered/left alignment, icon, underline and untouched source galley.
- [x] Alert reserves full copy-column width; regression test covers short/wrapped copy at 280 and 800 points.
- [x] Theme owns blue/white pair; egui adapter recolors only selected glyphs after native selection finalizes.
- [x] Native Label drag and TextEdit partial-selection tests; both themes for TextEdit.
- [x] fmt, check, clippy, workspace tests, release build.
- [x] Native form-error screenshots at three requested widths; Alert dismiss clicked successfully.
- [x] Native selected Alert description and selected Database input screenshots.
- [ ] Full requested 900/1080 heights (host display clamps capture height).
- [ ] Native held-button animation recording; current coverage is geometry tests.
- [ ] Independent review (self-review is recorded separately).
