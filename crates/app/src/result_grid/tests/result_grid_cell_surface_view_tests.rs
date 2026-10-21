//! Tests for `../result_grid_cell_surface_view.rs` (#542).
//!
//! `ellipsize_to_width` used to lay out the *entire* cell text before cutting it to the
//! visible prefix: a 65 KiB cell cost ~1.5 ms per painted cell (issue table: 14.56 ms for
//! 10 cells), ~60 ms/frame at 40 visible cells. The fix bounds the layout to a preview
//! window that only grows while the preview still fits, keeping the *observable* output
//! identical: cells that fit are returned whole, cut cells end with `…` at the same glyph
//! boundary the full layout produced.
use super::*;
use egui::{FontId, Pos2, RawInput, Vec2};

/// Column width from the issue's measurement environment (cell rect 180 px at
/// monospace 12 px ⇒ ~25 visible chars).
const CELL_WIDTH: f32 = 160.0;
const GRID_FONT: f32 = 12.0;

/// The pre-fix implementation, kept here as the semantics oracle: the bounded-window
/// rewrite must agree with this on every corpus case.
fn legacy_ellipsize_to_width(painter: &egui::Painter, text: &str, font: FontId, max_width: f32) -> String {
    let galley = painter.layout_no_wrap(text.to_owned(), font.clone(), egui::Color32::TRANSPARENT);
    if galley.size().x <= max_width {
        return text.to_owned();
    }
    let ellipsis_galley = painter.layout_no_wrap("…".to_owned(), font, egui::Color32::TRANSPARENT);
    let budget = max_width - ellipsis_galley.size().x;
    let row = &galley.rows[0].row;
    let cut = row
        .glyphs
        .iter()
        .take_while(|g| g.pos.x + g.advance_width <= budget)
        .count()
        .max(1);
    let mut out: String = text.chars().take(cut).collect();
    out.push('…');
    out
}

/// Runs `body` over one fresh batch of cells per egui frame and returns the best wall
/// time. `make` gets the batch id so every iteration lays out *new* text: egui caches
/// identical `layout_no_wrap` calls, and re-measuring a warm cache would hide the
/// O(cell-size) cost this issue is about (a scrolled-to row is always cold).
fn measure_batches(
    iterations: usize,
    make: impl Fn(usize) -> Vec<String>,
    body: impl Fn(&egui::Painter, &[String]) -> usize,
) -> std::time::Duration {
    let ctx = egui::Context::default();
    DbProTheme::install_fonts(&ctx);
    let mut best = std::time::Duration::MAX;
    // Batch 0 warms the font atlas and egui internals and is never timed.
    for batch in 0..=iterations {
        let texts = make(batch);
        let timed = batch > 0;
        let _ = crate::test_frame::frame(
            &ctx,
            RawInput {
                screen_rect: Some(egui::Rect::from_min_size(Pos2::ZERO, Vec2::new(1280.0, 800.0))),
                ..Default::default()
            },
            |ui| {
                // Only the cell-layout batch is timed, not the surrounding frame.
                let start = std::time::Instant::now();
                let _ = body(ui.painter(), &texts);
                if timed {
                    best = best.min(start.elapsed());
                }
            },
        );
    }
    best
}

#[test]
fn ellipsize_to_width_bounds_layout_cost_at_65kib_cells() {
    // Every batch gets fresh content (`#{batch}` salt) so egui's galley cache cannot
    // serve the measured layouts — the cost under test is the cold, per-cell layout.
    let small = |batch: usize| (0..10).map(|i| format!("{}{batch}-{i}", "payload".repeat(32))).collect::<Vec<_>>();
    let huge = |batch: usize| {
        (0..10)
            .map(|i| format!("{{\"payload\":\"{}\",\"id\":\"{batch}-{i}\"}}", "a".repeat(65_000)))
            .collect::<Vec<_>>()
    };
    let vi_huge = |batch: usize| {
        (0..10)
            .map(|i| format!("\"{}\"", "Nguyễn Thị Hồng Nhung ".repeat(2_600) + &format!("#{batch}{i}")))
            .collect::<Vec<_>>()
    };

    let body = |painter: &egui::Painter, texts: &[String]| {
        texts
            .iter()
            .map(|t| ellipsize_to_width(painter, t, FontId::monospace(GRID_FONT), CELL_WIDTH).chars().count())
            .sum::<usize>()
    };
    let legacy_body = |painter: &egui::Painter, texts: &[String]| {
        texts
            .iter()
            .map(|t| legacy_ellipsize_to_width(painter, t, FontId::monospace(GRID_FONT), CELL_WIDTH).chars().count())
            .sum::<usize>()
    };

    let small_best = measure_batches(8, small, body);
    let huge_best = measure_batches(8, huge, body);
    let vi_best = measure_batches(8, vi_huge, body);
    // The legacy oracle is the exact pre-fix implementation, so these are the same-session
    // "before" numbers on the same cold fixtures the "after" numbers were taken from.
    let huge_before = measure_batches(8, huge, legacy_body);
    let vi_before = measure_batches(8, vi_huge, legacy_body);
    println!(
        "#542 ellipsize cold batch of 10: small(200B)={small_best:?} | 65KiB-ascii before={huge_before:?} after={huge_best:?} | ~61KiB-vietnamese before={vi_before:?} after={vi_best:?} (issue table before: 36µs / 14.56ms on Linux)"
    );

    // Release builds carry the budget: 10 cells of 65 KiB must stay ≤ 1 ms total,
    // i.e. ≤ 0.1 ms per cell (was ~1.5 ms). Debug overhead is machine-dominated.
    if !cfg!(debug_assertions) {
        assert!(
            huge_best.as_nanos() <= 1_000_000,
            "#542: 10 × 65 KiB cells cost {huge_best:?}, budget 1 ms (0.1 ms/cell)"
        );
        assert!(
            vi_best.as_nanos() <= 1_000_000,
            "#542: 10 × 61 KiB Vietnamese cells cost {vi_best:?}, budget 1 ms"
        );
    }
}

#[test]
fn ellipsize_to_width_output_matches_the_legacy_full_layout_oracle() {
    let corpus: Vec<String> = vec![
        String::new(),
        "a".to_owned(),
        "Hello world".to_owned(),
        "short-fits-exactly".to_owned(),
        "Customer Alpha 12345".to_owned(),
        "This cell is definitely far too wide for the column".to_owned(),
        "Nguyễn Văn An • Trần Thị Bông Sen • Hà Nội".to_owned(),
        "ăâêôơưđ HÀ NỘI thành phố mang tên Bác".to_owned(),
        "emoji 🎉 mixed с текстом и digits 0123".to_owned(),
        "line-one\nline-two that is long enough to be cut somewhere past the newline".to_owned(),
        "trailing spaces                                   end".to_owned(),
        "tab\tseparated\tvalues\twith\tembedded\twhitespace\tdelimiters\tinside".to_owned(),
        "a".repeat(65_000),
        "Nguyễn Thị ".repeat(6_400),
        format!("{{\"data\":\"{}\"}}", "x".repeat(60_000)),
        "́".repeat(300) + &"abc".repeat(30), // zero-width run, still overflows later
    ];
    let ctx = egui::Context::default();
    DbProTheme::install_fonts(&ctx);
    let mut comparisons: Vec<(usize, String, String, String)> = Vec::new();
    let _ = crate::test_frame::frame(
        &ctx,
        RawInput {
            screen_rect: Some(egui::Rect::from_min_size(Pos2::ZERO, Vec2::new(1280.0, 800.0))),
            ..Default::default()
        },
        |ui| {
            let painter = ui.painter();
            for (index, text) in corpus.iter().enumerate() {
                let font = FontId::monospace(GRID_FONT);
                comparisons.push((
                    index,
                    text.clone(),
                    legacy_ellipsize_to_width(painter, text, font.clone(), CELL_WIDTH),
                    ellipsize_to_width(painter, text, font, CELL_WIDTH),
                ));
            }
        },
    );
    for (index, input, legacy, new) in &comparisons {
        // Exact equality for every corpus entry that fits (no `…` appended).
        if !legacy.ends_with('…') {
            assert_eq!(legacy, new, "corpus[{index}] fits the cell; output must be identical: {input:?}");
        } else {
            assert!(new.ends_with('…'), "corpus[{index}] must still ellipsize: {new:?}");
            assert_eq!(legacy, new, "corpus[{index}] cut point differs from the full-layout oracle");
        }
        // Invariant: the new path never returns more than the legacy prefix+ellipsis.
        assert!(
            new.chars().count() <= legacy.chars().count(),
            "corpus[{index}] new output longer than legacy"
        );
    }
}

#[test]
fn ellipsize_to_width_result_always_fits_the_cell_width() {
    let texts: Vec<String> = vec![
        "x".repeat(65_000),
        "Nguyễn Văn ".repeat(6_600),
        "a".repeat(30),
        "Hello".to_owned(),
    ];
    let ctx = egui::Context::default();
    DbProTheme::install_fonts(&ctx);
    let mut widths = Vec::new();
    let _ = crate::test_frame::frame(
        &ctx,
        RawInput {
            screen_rect: Some(egui::Rect::from_min_size(Pos2::ZERO, Vec2::new(1280.0, 800.0))),
            ..Default::default()
        },
        |ui| {
            let painter = ui.painter();
            for text in &texts {
                let out = ellipsize_to_width(painter, text, FontId::monospace(GRID_FONT), CELL_WIDTH);
                let w = painter
                    .layout_no_wrap(out.clone(), FontId::monospace(GRID_FONT), egui::Color32::TRANSPARENT)
                    .size()
                    .x;
                widths.push((out.chars().count(), w));
            }
        },
    );
    for (chars, width) in widths {
        // Small tolerance for a trailing combining mark the glyph walk may include.
        assert!(
            width <= CELL_WIDTH + 8.0,
            "ellipsized output of {chars} chars renders {width}px > {CELL_WIDTH}px cell"
        );
    }
}
