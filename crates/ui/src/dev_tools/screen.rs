// cc-scan:allow LONG_FUNCTION — analyzer passes are linear evidence gathering.
//! Screen-level UI composition analysis.
//!
//! Consumes the same [`GeometrySnapshot`] + [`SemanticSnapshot`] the rule
//! engine audits, plus the viewport rect, and produces a [`ScreenAnalysis`]:
//! measured screen metrics plus a *separate* finding list. Screen findings do
//! not join `AuditReport.issues` — they describe the whole composition, not a
//! widget pair, and they never flip the audit exit code.
//!
//! Design contract (same as `audit.rs`, extended):
//!
//! - *Internal consistency only.* No generic UI standards (no fixed padding
//!   scales, no universal density budgets). Findings fire on deviations from
//!   the screen's own rhythm/distribution, or on geometry that is wrong by
//!   definition (unclipped interactive widget outside the viewport).
//! - *Objective vs heuristic.* Every finding carries a [`FindingKind`] so the
//!   report separates geometric facts from consistency signals that could be
//!   legitimate design. Missing data means UNKNOWN — metrics carry `None` +
//!   `skipped` notes, never invented values.
//! - *IDE-aware.* Regions are classified by measured position/shape/content
//!   signatures (edge bars, side panels, floating dialogs, dense data grids),
//!   and several findings exempt dense data tables where IDE density is the
//!   intended pattern.
//! - *Screenshots.* Not consumed in this phase: geometry + semantics carry no
//!   pixel data, so pixel-level balance/contrast claims stay out of scope.

use egui::{Id, Rect};

use crate::dev_tools::audit::{Confidence, Severity};
use crate::dev_tools::geometry::{GeometryRect, GeometrySnapshot, WidgetGeometry};
use crate::dev_tools::semantic::SemanticSnapshot;

/// Environment selector for the screen fixture (`DB_PRO_AUDIT_SCREEN_FIXTURE`).
/// `ugly` paints intentional defects; any other non-empty value paints a
/// compositionally clean layout. Both live in one dedicated Area/layer so the
/// analyzer can scope itself to just the fixture (see [`ScreenScope::Fixture`]).
pub const SCREEN_FIXTURE_ENV: &str = "DB_PRO_AUDIT_SCREEN_FIXTURE";

/// Layer id of the screen fixture Area — the scope filter key.
pub fn screen_fixture_layer_id() -> egui::LayerId {
    egui::LayerId::new(egui::Order::Middle, Id::new("dbpro_screen_audit_fixture"))
}

/// The fixture's own "screen" — its virtual window. Chosen to fit inside the
/// smallest audited viewport (800×600) with margin.
pub const SCREEN_FIXTURE_RECT: Rect = Rect {
    min: egui::Pos2 { x: 60.0, y: 60.0 },
    max: egui::Pos2 { x: 920.0, y: 620.0 },
};

/// What `analyze_screen` should consider the screen.
pub enum ScreenScope {
    /// The real viewport: every widget, screen = `ctx.content_rect()`.
    Full,
    /// The fixture window: only widgets on `layer`, screen = `rect`.
    Fixture { layer: egui::LayerId, rect: Rect },
}

/// Whether a finding states a geometric fact or a consistency signal.
/// Distinguishing the two keeps "the audit measured X" separate from "the
/// audit suspects X is ugly" — the contract for this whole analyzer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FindingKind {
    /// True by construction (rect outside the viewport, no dominant region).
    Objective,
    /// Deviation from the screen's own pattern; could be deliberate design.
    Heuristic,
}

impl FindingKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Objective => "objective",
            Self::Heuristic => "heuristic",
        }
    }
}

/// One screen-level finding. `widget_id` pins a widget when the finding has a
/// single culprit; region-level findings carry `region` instead.
#[derive(Clone, Debug)]
pub struct ScreenFinding {
    /// Stable identifier, e.g. `"screen.spacing_outlier"`.
    pub rule: &'static str,
    pub kind: FindingKind,
    pub severity: Severity,
    pub confidence: Confidence,
    /// Culprit widget, when the finding is widget-scoped.
    pub widget_id: Option<Id>,
    pub widget_debug: Option<String>,
    /// Region index (into `ScreenAnalysis.regions`), when region-scoped.
    pub region: Option<usize>,
    /// The numbers that fired the finding — never a prose guess.
    pub evidence: String,
}

/// What a region is in IDE terms — inferred from measured position, shape,
/// and content signature, never from widget types alone.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RegionKind {
    /// Full-width strip along the top edge.
    TopBar,
    /// Full-width strip along the bottom edge.
    StatusBar,
    /// Edge-anchored tall narrow strip (activity rail, explorer sidebar).
    LeftRail,
    /// Edge-anchored tall narrow strip on the right.
    RightPanel,
    /// Interior tall narrow pane (explorer, secondary sidebar) — not
    /// edge-anchored but vertically dominant and horizontally slim.
    SidePanel,
    /// Floating layer (non-dominant layer) — dialog, palette, fixture window.
    Dialog,
    /// Everything else — the working surface.
    Content,
}

impl RegionKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::TopBar => "top_bar",
            Self::StatusBar => "status_bar",
            Self::LeftRail => "left_rail",
            Self::RightPanel => "right_panel",
            Self::SidePanel => "side_panel",
            Self::Dialog => "dialog",
            Self::Content => "content",
        }
    }
}

/// A measured screen region: a merged whitespace-delimited cluster of widgets.
#[derive(Clone, Debug)]
pub struct ScreenRegion {
    pub kind: RegionKind,
    /// Bounding box in global screen space.
    pub rect: GeometryRect,
    pub widgets: usize,
    pub interactive: usize,
    /// Dense uniform-stride data surface (result grid, tree view, table) —
    /// the signature some findings use to exempt intentional IDE density.
    pub grid_like: bool,
    /// Share of the screen's area this region covers.
    pub screen_share: f32,
    /// Union of widget rects / region rect — how tightly the region is packed.
    pub fill: f32,
    /// Majority of widgets live off the scope's dominant layer — floating
    /// layer (dialog/palette/fixture), not painted pane chrome.
    pub floating: bool,
}

/// Spacing rhythm over every measured sibling gap on the screen.
#[derive(Clone, Debug)]
pub struct RhythmStats {
    /// Sibling gaps measured (unclipped, same-layer+parent, axis-aligned).
    pub gaps_measured: usize,
    /// Most common positive gap — the screen's self-declared rhythm unit.
    /// `None` when no positive gap exists (gap-free grids only).
    pub mode_gap: Option<f32>,
    /// 0.0–1.0: share of measured gaps equal to the mode (±0.5px).
    /// `None` when fewer than two gaps were measured.
    pub mode_share: Option<f32>,
    /// Gaps beyond the screen's tolerance band (`mode + 8px` or 4×mode).
    pub outliers: usize,
}

/// Widget density, measured not prescribed.
#[derive(Clone, Debug)]
pub struct DensityStats {
    pub widgets: usize,
    pub interactive: usize,
    /// Interactive rects / viewport area.
    pub interactive_coverage: f32,
    /// Union of all widget rects / viewport area.
    pub widget_coverage: f32,
}

/// Interactive-mass distribution across screen quadrants (area share of all
/// interactive widget rect area — 4 numbers summing to ~1.0).
#[derive(Clone, Debug)]
pub struct BalanceStats {
    pub left_top: f32,
    pub right_top: f32,
    pub left_bottom: f32,
    pub right_bottom: f32,
}

/// Semantic hierarchy measurements. `Option` fields stay `None` when the
/// accesskit tree is unavailable — consumers must read UNKNOWN, not 0.
#[derive(Clone, Debug)]
pub struct HierarchyStats {
    /// Deepest parent→root chain in the tree. `None` = no tree.
    pub max_depth: Option<u32>,
    /// Nodes carrying a grouping role. `None` = no tree.
    pub containers: Option<usize>,
    /// Total nodes. `None` = no tree.
    pub nodes: Option<usize>,
}

/// Why part of the analysis is UNKNOWN — evidence, not silence.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnalysisGap {
    /// AccessKit produced no tree → semantic metrics unknown.
    NoSemanticTree,
    /// Zero usable widgets in scope → region/metric pass had nothing to read.
    NoGeometry,
    /// Every widget in scope was clipped → geometry exists but isn't usable.
    AllClipped,
}

impl AnalysisGap {
    pub fn label(self) -> &'static str {
        match self {
            Self::NoSemanticTree => "no semantic tree",
            Self::NoGeometry => "no geometry",
            Self::AllClipped => "all widgets clipped",
        }
    }
}

/// Everything measured about one screen.
#[derive(Clone, Debug)]
pub struct ScreenAnalysis {
    /// Whether the screen had enough widget geometry to analyze at all.
    /// `false` → metrics are the struct's `None` fields + gaps explains why.
    pub sufficient: bool,
    pub regions: Vec<ScreenRegion>,
    pub rhythm: RhythmStats,
    pub density: DensityStats,
    pub balance: BalanceStats,
    pub hierarchy: HierarchyStats,
    pub findings: Vec<ScreenFinding>,
    /// Data the analyzer needed but did not have.
    pub gaps: Vec<AnalysisGap>,
}

// ── Tuning ────────────────────────────────────────────────────────────────
// Every constant is local tuning, not a borrowed standard. Each documents
// which measured distribution it guards.

/// Minimum whitespace corridor that splits regions (px). Larger values merge
/// distinct panes across thin separators; this sits well below the app's
/// smallest real pane gap while absorbing sub-pixel rect noise.
const SEAM_MIN_PX: f32 = 0.5;

/// A gap inside a merged region up to this size does not prevent merging —
/// item padding lives inside regions, pane gaps between them. Only same-
/// parent groups merge, so this never crosses a structural boundary.
const REGION_MERGE_GAP: f32 = 24.0;

/// Distance (px) a region's bounding box may sit inside the screen edge and
/// still count as edge-anchored — pane padding lives inside the edge.
const EDGE_TOL_PX: f32 = 16.0;

/// An interior pane counts as a side panel when it's this tall as a screen
/// share and no wider than the panel bound below.
const PANEL_MIN_HEIGHT_SHARE: f32 = 0.3;
const PANEL_MAX_WIDTH_SHARE: f32 = 0.45;

/// Edge bars can be at most this tall/wide as a share of screen height;
/// anything larger is content, not chrome.
const BAR_MAX_THICKNESS_SHARE: f32 = 0.2;

/// Grid-like detection: a leaf region needs at least this many same-size
/// siblings before a stride pattern can even be claimed.
const GRID_MIN_ITEMS: usize = 8;

/// Within a same-parent column, siblings are "uniform" when their heights
/// differ by less than this (px) — paint noise vs a real size mix.
const UNIFORM_TOL_PX: f32 = 1.0;

/// Share of a group's gaps that must equal the mode before the group has a
/// dominant rhythm to deviate from.
const RHYTHM_MAJORITY: f32 = 0.6;

/// A gap must exceed `mode + OUTLIER_FLAT_PX` *and* `mode × OUTLIER_MULT` to
/// be an outlier — both ends protected so a 0-mode (gapless grid) can't flag
/// every padded neighbor, and a large mode can't flag modest variation.
const OUTLIER_FLAT_PX: f32 = 8.0;
const OUTLIER_MULT: f32 = 4.0;

/// `screen.layout_inversion`: edge-anchored secondary regions (rails/panels)
/// may occupy at most this fraction of screen width before they outweigh the
/// working surface. Measured as a screen-fact comparison, not a ratio ideal.
const SECONDARY_MAX_SHARE: f32 = 0.5;

/// A non-primary region this much larger than the screen's largest region
/// still isn't "primary" — requires ≥2 regions to claim a missing center.
const PRIMARY_MIN_REGIONS: usize = 6;

/// Primary working surface must cover at least this share of the screen
/// before `screen.no_primary_region` can fire at all.
const PRIMARY_MIN_SHARE: f32 = 0.15;

/// `screen.control_congestion`: interactive hitboxes packed under this gap
/// inside a non-data surface reads as an unstructured control wall. Dense
/// data surfaces (grid_like) and chrome bars are exempt by design.
const CONGESTION_GAP_PX: f32 = 2.0;
const CONGESTION_MIN_WIDGETS: usize = 10;

/// `screen.flat_semantics`: below this node count the screen is too small to
/// demand landmark structure (fixture-sized UIs must not flag).
const SEMANTIC_MIN_NODES: usize = 24;
/// A screen this large should offer more grouping than a flat list.
const SEMANTIC_CONTAINER_MIN_RATIO: f32 = 0.05;

/// How far outside the viewport an unclipped interactive rect may stick
/// before it counts as off-screen (absorbs sub-pixel edges).
const OFFSCREEN_MARGIN_PX: f32 = 1.0;

// ── Entry point ───────────────────────────────────────────────────────────

/// Analyze the whole screen. Cheap enough to run every audited frame: all
/// passes are O(widgets · log widgets) on a few hundred widgets.
pub fn analyze_screen(
    ctx: &egui::Context,
    snapshot: &GeometrySnapshot,
    semantic: Option<&SemanticSnapshot>,
    scope: &ScreenScope,
) -> ScreenAnalysis {
    let (screen_rect, scoped): (Rect, Vec<&WidgetGeometry>) = match scope {
        ScreenScope::Full => (
            ctx.content_rect(),
            snapshot.layers.iter().flat_map(|(_, ws)| ws.iter()).collect(),
        ),
        ScreenScope::Fixture { layer, rect } => (
            *rect,
            snapshot
                .layers
                .iter()
                .flat_map(|(_, ws)| ws.iter())
                .filter(|w| w.layer == *layer)
                .collect(),
        ),
    };

    // Global rects once — every pass below works in screen space.
    let widgets: Vec<(&WidgetGeometry, Rect)> = scoped
        .iter()
        .map(|w| (*w, w.to_global_rect(ctx)))
        .filter(|(_, r)| r.is_finite() && r.width() >= 0.0 && r.height() >= 0.0)
        .collect();

    let mut analysis = ScreenAnalysis {
        sufficient: !widgets.is_empty(),
        regions: Vec::new(),
        rhythm: RhythmStats {
            gaps_measured: 0,
            mode_gap: None,
            mode_share: None,
            outliers: 0,
        },
        density: DensityStats {
            widgets: widgets.len(),
            interactive: widgets
                .iter()
                .filter(|(w, _)| w.enabled && (w.senses_click || w.senses_drag || w.focusable))
                .count(),
            interactive_coverage: 0.0,
            widget_coverage: 0.0,
        },
        balance: BalanceStats {
            left_top: 0.0,
            right_top: 0.0,
            left_bottom: 0.0,
            right_bottom: 0.0,
        },
        hierarchy: HierarchyStats {
            max_depth: None,
            containers: None,
            nodes: None,
        },
        findings: Vec::new(),
        gaps: Vec::new(),
    };

    // Region/rhythm inputs need *content* widgets: interactive controls and
    // semantic widgets (labels, inputs). Pure container frames — a Ui's own
    // hover rect with no info — tile the screen edge-to-edge and would weld
    // every pane into one blob, so they're excluded from segmentation (they
    // still count in density widgets/coverage).
    let usable: Vec<(&WidgetGeometry, Rect)> = widgets
        .iter()
        .copied()
        .filter(|(w, _)| {
            if w.clipped {
                return false;
            }
            // Overlay orders (menus, popups, tooltips, frameless-window
            // resize strips) float above the pane layout — including them
            // welds every whitespace seam, so they never join segmentation.
            // They still count in density/balance and off-screen checks.
            if matches!(
                w.layer.order,
                egui::Order::Foreground | egui::Order::Tooltip | egui::Order::Debug
            ) {
                return false;
            }
            (w.enabled && (w.senses_click || w.senses_drag || w.focusable))
                || w.widget_type.is_some()
                || w.label.is_some()
        })
        .collect();
    if widgets.is_empty() {
        analysis.gaps.push(AnalysisGap::NoGeometry);
    } else if usable.is_empty() {
        analysis.gaps.push(AnalysisGap::AllClipped);
        analysis.sufficient = false;
    }
    if semantic.is_none() {
        analysis.gaps.push(AnalysisGap::NoSemanticTree);
    }

    // Coverage: union area via 1px occupancy on the screen bounds. Widget
    // count is small and the screen is bounded — a grid is exact enough and
    // cheaper than interval math for an arbitrary set of rects.
    let screen_area = screen_rect.width() * screen_rect.height();
    if screen_area > 0.0 {
        let cell_w = screen_rect.width().ceil() as usize;
        let cell_h = screen_rect.height().ceil() as usize;
        if cell_w * cell_h <= 8_000_000 {
            let mut all_cells = vec![false; cell_w * cell_h];
            let mut int_cells = vec![false; cell_w * cell_h];
            for (w, r) in &widgets {
                mark_cells(screen_rect, *r, cell_w, cell_h, &mut all_cells);
                if w.enabled && (w.senses_click || w.senses_drag || w.focusable) {
                    mark_cells(screen_rect, *r, cell_w, cell_h, &mut int_cells);
                }
            }
            analysis.density.widget_coverage = all_cells.iter().filter(|c| **c).count() as f32 / screen_area;
            analysis.density.interactive_coverage = int_cells.iter().filter(|c| **c).count() as f32 / screen_area;
        }
        // Screen too large to grid → coverage stays 0.0; a real viewport is
        // never near this, so no skip note is worth a fake number either way.
    }

    // Quadrant balance of interactive mass (widget rect area per quadrant).
    let mut int_total = 0.0_f32;
    for (w, r) in &widgets {
        if !(w.enabled && (w.senses_click || w.senses_drag || w.focusable)) {
            continue;
        }
        let area = (r.width() * r.height()).max(0.0);
        int_total += area;
        let left = r.center().x <= screen_rect.center().x;
        let top = r.center().y <= screen_rect.center().y;
        match (left, top) {
            (true, true) => analysis.balance.left_top += area,
            (false, true) => analysis.balance.right_top += area,
            (true, false) => analysis.balance.left_bottom += area,
            (false, false) => analysis.balance.right_bottom += area,
        }
    }
    if int_total > 0.0 {
        analysis.balance.left_top /= int_total;
        analysis.balance.right_top /= int_total;
        analysis.balance.left_bottom /= int_total;
        analysis.balance.right_bottom /= int_total;
    }

    analysis.regions = detect_regions(ctx, &usable, screen_rect, screen_area);
    measure_hierarchy(semantic, &mut analysis);
    measure_rhythm(&usable, &mut analysis);

    if analysis.sufficient {
        finding_off_screen(&widgets, screen_rect, &mut analysis.findings);
        finding_layout_inversion(&analysis.regions, screen_rect, &mut analysis.findings);
        let interactive_coverage = analysis.density.interactive_coverage;
        finding_no_primary(
            &analysis.regions,
            screen_area,
            interactive_coverage,
            &mut analysis.findings,
        );
        finding_dialog_offscreen(&analysis.regions, screen_rect, &mut analysis.findings);
        finding_spacing_outliers(&usable, &mut analysis.findings);
        finding_congestion(&usable, &analysis.regions, &mut analysis.findings);
    }
    finding_flat_semantics(semantic, &mut analysis.findings);

    analysis
}

/// Mark every pixel cell a rect covers (clamped to the screen bounds).
fn mark_cells(screen: Rect, r: Rect, cell_w: usize, cell_h: usize, cells: &mut [bool]) {
    let x0 = (r.min.x - screen.min.x).floor().clamp(0.0, cell_w as f32 - 1.0) as usize;
    let y0 = (r.min.y - screen.min.y).floor().clamp(0.0, cell_h as f32 - 1.0) as usize;
    let x1 = (r.max.x - screen.min.x).ceil().clamp(0.0, cell_w as f32) as usize;
    let y1 = (r.max.y - screen.min.y).ceil().clamp(0.0, cell_h as f32) as usize;
    for y in y0..y1.min(cell_h) {
        for x in x0..x1.min(cell_w) {
            cells[y * cell_w + x] = true;
        }
    }
}

// ── Region detection: whitespace XY-cut + structural merge ────────────────
//
// Whitespace segmentation is the only layout-agnostic signal geometry offers:
// painted separators, pane borders, and inter-region gaps all read as
// corridors with zero widgets. The XY-cut alternates axis splits, then the
// structural merge pass rejoins leaves that are one semantic container
// (shared `parent_id`) — which is what a toolbar or grid actually is —
// provided no pane-scale gap separates them. Merge can never cross a
// structural boundary, so it cannot smear the sidebar into the workspace.

fn detect_regions(
    ctx: &egui::Context,
    widgets: &[(&WidgetGeometry, Rect)],
    screen: Rect,
    screen_area: f32,
) -> Vec<ScreenRegion> {
    let mut leaves: Vec<Vec<usize>> = Vec::new();
    xy_cut(widgets, &(0..widgets.len()).collect::<Vec<_>>(), true, &mut leaves);

    // Merge leaves that belong to the same parent Ui and sit within item-
    // spacing distance of each other, with ≥50% cross-axis span overlap —
    // the shape of "same container, padded items". XY-cut splits every
    // internal padding gap down to singleton leaves, so the merge is where
    // "one container" is reassembled; filtering before it would drop every
    // widget. Two same-parent groups that never approach (a toolbar's
    // justified split) stay separate regions.
    let mut members: Vec<Vec<usize>> = leaves;
    loop {
        let mut merged_any = false;
        'outer: for i in 0..members.len() {
            for j in (i + 1)..members.len() {
                if can_merge(widgets, &members[i], &members[j]) {
                    let rhs = members.remove(j);
                    members[i].extend(rhs);
                    merged_any = true;
                    break 'outer;
                }
            }
        }
        if !merged_any {
            break;
        }
    }

    // Dominant layer = where most widgets live — the app's painted pane
    // layer, or the fixture's own layer inside its virtual screen. Regions
    // whose majority sits on another layer float above that surface.
    let dominant_layer = {
        use std::collections::HashMap;
        let mut counts: HashMap<egui::LayerId, usize> = HashMap::new();
        for (w, _) in widgets {
            *counts.entry(w.layer).or_default() += 1;
        }
        counts
            .into_iter()
            .max_by_key(|(_, n)| *n)
            .map(|(l, _)| l)
            .unwrap_or(egui::LayerId::new(egui::Order::Middle, egui::Id::NULL))
    };

    let mut regions: Vec<ScreenRegion> = members
        .iter()
        .filter(|idx| !idx.is_empty())
        .map(|idx| {
            let mut rect = Rect::NOTHING;
            let mut union_area = 0.0;
            let mut interactive = 0;
            let mut off_layer = 0;
            for &i in idx {
                let (w, r) = widgets[i];
                rect = rect.union(r);
                union_area += (r.width() * r.height()).max(0.0);
                if w.enabled && (w.senses_click || w.senses_drag) {
                    interactive += 1;
                }
                if w.layer != dominant_layer {
                    off_layer += 1;
                }
            }
            let floating = off_layer * 2 > idx.len();
            let kind = classify(ctx, widgets, idx, rect, screen, floating);
            ScreenRegion {
                kind,
                rect: GeometryRect::from_egui(rect),
                widgets: idx.len(),
                interactive,
                grid_like: is_grid_like(ctx, widgets, idx),
                screen_share: if screen_area > 0.0 {
                    (rect.width() * rect.height()) / screen_area
                } else {
                    0.0
                },
                fill: {
                    let area = rect.width() * rect.height();
                    if area > 0.0 {
                        union_area / area
                    } else {
                        0.0
                    }
                },
                floating,
            }
        })
        .collect();
    // Largest first — the working surface leads the report.
    regions.sort_by(|a, b| {
        b.screen_share
            .partial_cmp(&a.screen_share)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    regions
}

/// Recursive whitespace cut: try splitting `group` along a seam on `x_axis`,
/// falling back to the other axis, recursing until no seam exists.
fn xy_cut(widgets: &[(&WidgetGeometry, Rect)], group: &[usize], x_axis: bool, out: &mut Vec<Vec<usize>>) {
    if group.len() < 2 {
        out.push(group.to_vec());
        return;
    }
    if let Some((left, right)) = split_on_seam(widgets, group, x_axis) {
        xy_cut(widgets, &left, !x_axis, out);
        xy_cut(widgets, &right, !x_axis, out);
        return;
    }
    if let Some((left, right)) = split_on_seam(widgets, group, !x_axis) {
        xy_cut(widgets, &left, x_axis, out);
        xy_cut(widgets, &right, x_axis, out);
        return;
    }
    out.push(group.to_vec());
}

/// Largest whitespace corridor along one axis. `None` when widgets cover the
/// axis continuously or all corridors are thinner than [`SEAM_MIN_PX`].
fn split_on_seam(
    widgets: &[(&WidgetGeometry, Rect)],
    group: &[usize],
    x_axis: bool,
) -> Option<(Vec<usize>, Vec<usize>)> {
    let axis = |r: &Rect| if x_axis { (r.min.x, r.max.x) } else { (r.min.y, r.max.y) };
    let mut ivals: Vec<(f32, f32, usize)> = group
        .iter()
        .map(|&i| {
            let (lo, hi) = axis(&widgets[i].1);
            (lo, hi, i)
        })
        .collect();
    ivals.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

    // Walk covered intervals; a corridor is the space between the running
    // union edge and the next interval start. Take the widest one — splitting
    // on the first thin corridor would fragment before the real pane gap.
    let mut best: Option<(f32, f32)> = None;
    let mut edge = ivals[0].1;
    for &(lo, hi, _) in &ivals[1..] {
        let gap = lo - edge;
        if gap >= SEAM_MIN_PX && best.is_none_or(|(_, w)| gap > w) {
            best = Some((edge, gap));
        }
        edge = edge.max(hi);
    }
    let (cut, _) = best?;
    let left: Vec<usize> = ivals
        .iter()
        .filter(|(_, hi, _)| *hi <= cut + f32::EPSILON)
        .map(|(_, _, i)| *i)
        .collect();
    if left.is_empty() || left.len() == group.len() {
        return None;
    }
    let right: Vec<usize> = group.iter().copied().filter(|i| !left.contains(i)).collect();
    Some((left, right))
}

/// Two leaf groups merge when they share a `parent_id` (one semantic
/// container split by internal padding) and every cross pair is within
/// item-spacing distance on one axis with ≥50% overlap on the other.
fn can_merge(widgets: &[(&WidgetGeometry, Rect)], a: &[usize], b: &[usize]) -> bool {
    let shared_parent = a
        .iter()
        .any(|&i| b.iter().any(|&j| widgets[i].0.parent_id == widgets[j].0.parent_id));
    if !shared_parent {
        return false;
    }
    let close_pair = |r1: &Rect, r2: &Rect| -> bool {
        let x_gap = (r2.min.x - r1.max.x).max(r1.min.x - r2.max.x).max(0.0);
        let y_gap = (r2.min.y - r1.max.y).max(r1.min.y - r2.max.y).max(0.0);
        let x_ov = overlap(r1.min.x, r1.max.x, r2.min.x, r2.max.x);
        let y_ov = overlap(r1.min.y, r1.max.y, r2.min.y, r2.max.y);
        let x_span = (r1.max.x - r1.min.x).min(r2.max.x - r2.min.x).max(1.0);
        let y_span = (r1.max.y - r1.min.y).min(r2.max.y - r2.min.y).max(1.0);
        (x_gap <= REGION_MERGE_GAP && y_ov / y_span >= 0.5) || (y_gap <= REGION_MERGE_GAP && x_ov / x_span >= 0.5)
    };
    a.iter()
        .any(|&i| b.iter().any(|&j| close_pair(&widgets[i].1, &widgets[j].1)))
}

fn overlap(a0: f32, a1: f32, b0: f32, b1: f32) -> f32 {
    (a1.min(b1) - a0.max(b0)).max(0.0)
}

/// Positional/shape/content classification — first matching measured
/// signature wins, so the order below is the precedence.
fn classify(
    _ctx: &egui::Context,
    _widgets: &[(&WidgetGeometry, Rect)],
    _idx: &[usize],
    rect: Rect,
    screen: Rect,
    floating: bool,
) -> RegionKind {
    let sw = screen.width().max(1.0);
    let sh = screen.height().max(1.0);
    let w_share = rect.width() / sw;
    let h_share = rect.height() / sh;
    let touching_left = rect.min.x <= screen.min.x + EDGE_TOL_PX;
    let touching_right = rect.max.x >= screen.max.x - EDGE_TOL_PX;
    let touching_top = rect.min.y <= screen.min.y + EDGE_TOL_PX;
    let touching_bottom = rect.max.y >= screen.max.y - EDGE_TOL_PX;
    let edge_anchored = touching_left || touching_right || touching_top || touching_bottom;

    // Floating layers are only "dialogs" when they don't wear an edge-chrome
    // shape — a palette pinned along the top is a bar, not a dialog.
    if floating && !edge_anchored {
        return RegionKind::Dialog;
    }
    // Edge chrome: a thin strip touching an edge is a bar regardless of
    // width — a toolbar fragment is still chrome, not content.
    if touching_top && h_share <= BAR_MAX_THICKNESS_SHARE {
        return RegionKind::TopBar;
    }
    if touching_bottom && h_share <= BAR_MAX_THICKNESS_SHARE {
        return RegionKind::StatusBar;
    }
    if touching_left && h_share >= 0.5 && w_share <= PANEL_MAX_WIDTH_SHARE {
        return RegionKind::LeftRail;
    }
    if touching_right && h_share >= 0.5 && w_share <= PANEL_MAX_WIDTH_SHARE {
        return RegionKind::RightPanel;
    }
    // Interior pane: tall and slim but not edge-anchored — explorer /
    // secondary sidebar sitting between the rail and the workspace.
    if h_share >= PANEL_MIN_HEIGHT_SHARE && w_share <= PANEL_MAX_WIDTH_SHARE && !floating {
        return RegionKind::SidePanel;
    }
    if floating {
        return RegionKind::Dialog;
    }
    // grid_like is computed separately — it feeds content checks elsewhere.
    RegionKind::Content
}

/// Dense uniform-stride data surface: any single parent column or row with
/// ≥8 same-size siblings at a repeated pitch. Result grids, explorer trees,
/// and history lists all produce this signature; a toolbar of varied-size
/// buttons does not.
fn is_grid_like(_ctx: &egui::Context, widgets: &[(&WidgetGeometry, Rect)], idx: &[usize]) -> bool {
    use std::collections::HashMap;
    let mut by_parent: HashMap<Id, Vec<usize>> = HashMap::new();
    for &i in idx {
        by_parent.entry(widgets[i].0.parent_id).or_default().push(i);
    }
    by_parent.values().any(|members| {
        if members.len() < GRID_MIN_ITEMS {
            return false;
        }
        // Uniform size: same height (row grid) or same width (column strip).
        let (w0, h0) = (widgets[members[0]].1.width(), widgets[members[0]].1.height());
        let uniform_h = members
            .iter()
            .all(|&i| (widgets[i].1.height() - h0).abs() <= UNIFORM_TOL_PX);
        let uniform_w = members
            .iter()
            .all(|&i| (widgets[i].1.width() - w0).abs() <= UNIFORM_TOL_PX);
        if !(uniform_h || uniform_w) {
            return false;
        }
        // Repeated pitch along the uniform axis: sort, take consecutive
        // center deltas, require ≥75% equal to the modal pitch (±1px).
        let mut axis: Vec<f32> = members
            .iter()
            .map(|&i| {
                let r = widgets[i].1;
                if uniform_h {
                    r.min.y
                } else {
                    r.min.x
                }
            })
            .collect();
        axis.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let mut pitches: Vec<f32> = axis.windows(2).map(|w| w[1] - w[0]).collect();
        pitches.retain(|p| *p > 0.0);
        if pitches.len() < GRID_MIN_ITEMS / 2 {
            return false;
        }
        let mut best = 0usize;
        for &p in &pitches {
            let n = pitches.iter().filter(|q| (*q - p).abs() <= UNIFORM_TOL_PX).count();
            if n > best {
                best = n;
            }
        }
        best as f32 / pitches.len() as f32 >= 0.75
    })
}

// ── Metrics ───────────────────────────────────────────────────────────────

fn measure_hierarchy(semantic: Option<&SemanticSnapshot>, analysis: &mut ScreenAnalysis) {
    let Some(sem) = semantic else {
        return; // gaps already notes NoSemanticTree
    };
    use egui::accesskit::Role;
    let mut max_depth = 0u32;
    for node in sem.nodes.values() {
        let mut depth = 1u32;
        let mut cursor = node.parent;
        // Trees are shallow; a cycle guard costs one bound.
        while let Some(p) = cursor {
            depth += 1;
            cursor = sem.nodes.get(&p).and_then(|n| n.parent);
            if depth > 64 {
                break;
            }
        }
        max_depth = max_depth.max(depth);
    }
    let containers = sem
        .nodes
        .values()
        .filter(|n| {
            matches!(
                n.role,
                Role::Pane
                    | Role::GenericContainer
                    | Role::ScrollView
                    | Role::Group
                    | Role::List
                    | Role::Table
                    | Role::Tree
                    | Role::Toolbar
                    | Role::TabPanel
                    | Role::MenuBar
            )
        })
        .count();
    analysis.hierarchy = HierarchyStats {
        max_depth: Some(max_depth),
        containers: Some(containers),
        nodes: Some(sem.nodes.len()),
    };
}

/// Every unclipped sibling gap on the screen, axis-aligned per parent group.
/// Gaps feed both the rhythm metrics and the spacing-outlier finding.
fn sibling_gaps(widgets: &[(&WidgetGeometry, Rect)]) -> Vec<(f32, Id, Id, bool)> {
    use std::collections::HashMap;
    let mut groups: HashMap<(egui::LayerId, Id), Vec<(&WidgetGeometry, Rect)>> = HashMap::new();
    for (w, r) in widgets {
        groups.entry((w.layer, w.parent_id)).or_default().push((*w, *r));
    }
    let mut gaps = Vec::new();
    for members in groups.values() {
        // Vertical stacking: x-overlapping pairs consecutive in y.
        let mut col: Vec<_> = members.clone();
        col.sort_by(|a, b| a.1.min.y.partial_cmp(&b.1.min.y).unwrap_or(std::cmp::Ordering::Equal));
        for pair in col.windows(2) {
            if overlap(pair[0].1.min.x, pair[0].1.max.x, pair[1].1.min.x, pair[1].1.max.x) <= 0.0 {
                continue;
            }
            gaps.push((pair[1].1.min.y - pair[0].1.max.y, pair[0].0.id, pair[1].0.id, false));
        }
        // Horizontal sequence: y-overlapping pairs consecutive in x.
        let mut row: Vec<_> = members.clone();
        row.sort_by(|a, b| a.1.min.x.partial_cmp(&b.1.min.x).unwrap_or(std::cmp::Ordering::Equal));
        for pair in row.windows(2) {
            if overlap(pair[0].1.min.y, pair[0].1.max.y, pair[1].1.min.y, pair[1].1.max.y) <= 0.0 {
                continue;
            }
            gaps.push((pair[1].1.min.x - pair[0].1.max.x, pair[0].0.id, pair[1].0.id, true));
        }
    }
    gaps
}

fn measure_rhythm(widgets: &[(&WidgetGeometry, Rect)], analysis: &mut ScreenAnalysis) {
    let gaps = sibling_gaps(widgets);
    analysis.rhythm.gaps_measured = gaps.len();
    let positive: Vec<f32> = gaps.iter().map(|(g, _, _, _)| *g).filter(|g| *g > 0.0).collect();
    if positive.is_empty() {
        return;
    }
    // Mode = most common gap in 0.5px buckets.
    let mut buckets: Vec<(f32, usize)> = Vec::new();
    for g in &positive {
        match buckets.iter_mut().find(|(x, _)| (x - g).abs() <= 0.5) {
            Some((x, n)) => {
                *x = (*x * *n as f32 + g) / (*n as f32 + 1.0);
                *n += 1;
            }
            None => buckets.push((*g, 1)),
        }
    }
    let (mode, mode_n) = buckets
        .iter()
        .max_by_key(|(_, n)| n)
        .map(|(x, n)| (*x, *n))
        .unwrap_or((0.0, 0));
    analysis.rhythm.mode_gap = Some(mode);
    analysis.rhythm.mode_share = Some(mode_n as f32 / positive.len() as f32);
    let outlier_floor = (mode * OUTLIER_MULT).max(mode + OUTLIER_FLAT_PX);
    analysis.rhythm.outliers = positive.iter().filter(|g| **g > outlier_floor).count();
}

// ── Findings ──────────────────────────────────────────────────────────────

/// Unclipped interactive rect reaching outside the viewport — unreachable
/// hit area, objective defect. A clipped widget's true extent is unknowable
/// from geometry, so it can't flag (that's the existing Clipped contract).
fn finding_off_screen(widgets: &[(&WidgetGeometry, Rect)], screen: Rect, findings: &mut Vec<ScreenFinding>) {
    for (w, r) in widgets {
        if !(w.enabled && (w.senses_click || w.senses_drag)) || w.clipped {
            continue;
        }
        let outside_x = r.min.x < screen.min.x - OFFSCREEN_MARGIN_PX || r.max.x > screen.max.x + OFFSCREEN_MARGIN_PX;
        let outside_y = r.min.y < screen.min.y - OFFSCREEN_MARGIN_PX || r.max.y > screen.max.y + OFFSCREEN_MARGIN_PX;
        if !(outside_x || outside_y) {
            continue;
        }
        findings.push(ScreenFinding {
            rule: "screen.off_screen",
            kind: FindingKind::Objective,
            severity: Severity::Warning,
            confidence: Confidence::High,
            widget_id: Some(w.id),
            widget_debug: Some(w.id_debug.clone()),
            region: None,
            evidence: format!(
                "interactive rect ({:.0},{:.0})→({:.0},{:.0}) exceeds viewport ({:.0}×{:.0})",
                r.min.x,
                r.min.y,
                r.max.x,
                r.max.y,
                screen.width(),
                screen.height()
            ),
        });
    }
}

/// Edge-anchored secondary regions outweighing the working surface — an
/// inversion of the IDE contract (navigation serves content), measured as
/// width-share comparison, not an aesthetic ratio.
fn finding_layout_inversion(regions: &[ScreenRegion], screen: Rect, findings: &mut Vec<ScreenFinding>) {
    let sw = screen.width().max(1.0);
    let sh = screen.height().max(1.0);
    // Measured side-anchored navigation: touches a left/right edge, runs
    // vertically, and isn't itself a data grid (a dense data pane anchored
    // to the edge is working surface, not chrome). Compare its width against
    // the widest interior region — if navigation outweighs the work surface
    // the layout is inverted, by measurement not by ideal ratio.
    let mut secondary = 0.0_f32;
    let mut interior = 0.0_f32;
    for r in regions {
        let rect = r.rect.to_egui();
        let side_anchored = (rect.min.x <= screen.min.x + EDGE_TOL_PX || rect.max.x >= screen.max.x - EDGE_TOL_PX)
            && rect.min.y > screen.min.y + EDGE_TOL_PX
            && rect.max.y < screen.max.y - EDGE_TOL_PX
            && rect.height() / sh >= PANEL_MIN_HEIGHT_SHARE;
        if side_anchored {
            secondary = secondary.max(rect.width());
        } else {
            interior = interior.max(rect.width());
        }
    }
    if secondary / sw > SECONDARY_MAX_SHARE && secondary > interior {
        findings.push(ScreenFinding {
            rule: "screen.layout_inversion",
            kind: FindingKind::Objective,
            severity: Severity::Warning,
            confidence: Confidence::High,
            widget_id: None,
            widget_debug: None,
            region: None,
            evidence: format!(
                "edge-anchored panel is {:.0}px wide ({:.0}% of screen) vs the widest interior region at {:.0}px",
                secondary,
                secondary / sw * 100.0,
                interior
            ),
        });
    }
}

/// No dominant working surface on a busy, fragmented screen: many regions,
/// the largest under [`PRIMARY_MIN_SHARE`], and enough interactive coverage
/// that a work surface should exist. Sparse screens (welcome, empty states)
/// legitimately have no dominant region — they don't reach the busy gate.
fn finding_no_primary(
    regions: &[ScreenRegion],
    screen_area: f32,
    interactive_coverage: f32,
    findings: &mut Vec<ScreenFinding>,
) {
    if regions.len() < PRIMARY_MIN_REGIONS || screen_area <= 0.0 {
        return;
    }
    if interactive_coverage < 0.15 {
        return;
    }
    let Some(largest) = regions.first() else { return };
    if largest.screen_share < PRIMARY_MIN_SHARE {
        findings.push(ScreenFinding {
            rule: "screen.no_primary_region",
            kind: FindingKind::Objective,
            severity: Severity::Warning,
            confidence: Confidence::Medium,
            widget_id: None,
            widget_debug: None,
            region: Some(0),
            evidence: format!(
                "largest region covers {:.0}% of the screen across {} regions — no dominant work surface",
                largest.screen_share * 100.0,
                regions.len()
            ),
        });
    }
}

/// Floating region (dialog/palette) sticking outside the viewport —
/// unreachable content, not a styling choice.
fn finding_dialog_offscreen(regions: &[ScreenRegion], screen: Rect, findings: &mut Vec<ScreenFinding>) {
    for (i, r) in regions.iter().enumerate() {
        if r.kind != RegionKind::Dialog {
            continue;
        }
        let rect = r.rect.to_egui();
        let overflow = [
            (screen.min.x - rect.min.x).max(0.0),
            (rect.max.x - screen.max.x).max(0.0),
            (screen.min.y - rect.min.y).max(0.0),
            (rect.max.y - screen.max.y).max(0.0),
        ]
        .into_iter()
        .fold(0.0_f32, f32::max);
        if overflow > OFFSCREEN_MARGIN_PX {
            findings.push(ScreenFinding {
                rule: "screen.dialog_offscreen",
                kind: FindingKind::Objective,
                severity: Severity::Warning,
                confidence: Confidence::Medium,
                widget_id: None,
                widget_debug: None,
                region: Some(i),
                evidence: format!(
                    "dialog region ({:.0},{:.0})→({:.0},{:.0}) overflows the viewport by {:.0}px",
                    rect.min.x, rect.min.y, rect.max.x, rect.max.y, overflow
                ),
            });
        }
    }
}

/// One sibling gap in a group deviating from that group's dominant rhythm —
/// the screen's own spacing unit establishes what's irregular. Groups with no
/// dominant rhythm (or <3 gaps) skip: a rhythm must exist to deviate from.
fn finding_spacing_outliers(widgets: &[(&WidgetGeometry, Rect)], findings: &mut Vec<ScreenFinding>) {
    use std::collections::HashMap;
    // A spacing defect only exists inside a *uniform run*: same-size siblings
    // in a repeated pattern (list rows, form fields, grid columns). Justified
    // layouts (left logo vs right-aligned toolbar cluster, dialog footer
    // pinned to the bottom) produce the same large-gap shape without a
    // uniform run — the uniformity precondition is what keeps those legal.
    let mut groups: HashMap<(egui::LayerId, Id), Vec<(&WidgetGeometry, Rect)>> = HashMap::new();
    for (w, r) in widgets {
        groups.entry((w.layer, w.parent_id)).or_default().push((*w, *r));
    }
    for members in groups.values() {
        if members.len() < 4 {
            continue;
        }
        // Audit each axis: consecutive x-overlapping pairs in y, and
        // y-overlapping pairs in x.
        for horizontal in [false, true] {
            let mut run: Vec<(&WidgetGeometry, Rect)> = members.clone();
            run.sort_by(|a, b| {
                let key = |r: &Rect| if horizontal { r.min.x } else { r.min.y };
                key(&a.1).partial_cmp(&key(&b.1)).unwrap_or(std::cmp::Ordering::Equal)
            });
            // Cross-axis uniformity: stacked rows share a height; a mixed-size
            // group is not a repeated pattern, so there is no rhythm to break.
            let cross = |r: &Rect| if horizontal { r.width() } else { r.height() };
            let c0 = cross(&run[0].1);
            if !run.iter().all(|(_, r)| (cross(r) - c0).abs() <= UNIFORM_TOL_PX) {
                continue;
            }
            let mut gaps: Vec<(f32, &WidgetGeometry)> = Vec::new();
            for pair in run.windows(2) {
                let cross_overlap = if horizontal {
                    overlap(pair[0].1.min.y, pair[0].1.max.y, pair[1].1.min.y, pair[1].1.max.y)
                } else {
                    overlap(pair[0].1.min.x, pair[0].1.max.x, pair[1].1.min.x, pair[1].1.max.x)
                };
                if cross_overlap <= 0.0 {
                    continue;
                }
                let gap = if horizontal {
                    pair[1].1.min.x - pair[0].1.max.x
                } else {
                    pair[1].1.min.y - pair[0].1.max.y
                };
                gaps.push((gap, pair[1].0));
            }
            let positive: Vec<f32> = gaps.iter().map(|(g, _)| *g).filter(|g| *g >= 0.0).collect();
            if positive.len() < 3 {
                continue;
            }
            let mut buckets: Vec<(f32, usize)> = Vec::new();
            for g in &positive {
                match buckets.iter_mut().find(|(x, _)| (x - g).abs() <= 0.5) {
                    Some((x, n)) => {
                        *x = (*x * *n as f32 + g) / (*n as f32 + 1.0);
                        *n += 1;
                    }
                    None => buckets.push((*g, 1)),
                }
            }
            let Some(&(mode, mode_n)) = buckets.iter().max_by_key(|(_, n)| n) else {
                continue;
            };
            if (mode_n as f32 / positive.len() as f32) < RHYTHM_MAJORITY {
                continue; // no dominant rhythm → nothing to deviate from
            }
            let floor = (mode * OUTLIER_MULT).max(mode + OUTLIER_FLAT_PX);
            for (idx, (gap, b)) in gaps.iter().enumerate() {
                // Edge gaps are anchoring, not deviation: a bottom-pinned
                // settings icon or right-aligned toolbar cluster ends a run
                // with an intentional jump. Only interior gaps — siblings on
                // both sides — can break the rhythm.
                if idx == 0 || idx + 1 == gaps.len() {
                    continue;
                }
                if *gap > floor {
                    findings.push(ScreenFinding {
                        rule: "screen.spacing_outlier",
                        kind: FindingKind::Heuristic,
                        severity: Severity::Warning,
                        confidence: Confidence::Medium,
                        widget_id: Some(b.id),
                        widget_debug: Some(b.id_debug.clone()),
                        region: None,
                        evidence: format!(
                            "{gap:.0}px gap vs the run's dominant {mode:.0}px rhythm ({:.0}% of {} uniform siblings)",
                            mode_n as f32 / positive.len() as f32 * 100.0,
                            positive.len()
                        ),
                    });
                }
            }
        }
    }
}

/// Interactive controls packed under [`CONGESTION_GAP_PX`] inside a non-data
/// surface — a wall of undifferentiated controls. Dense data grids and edge
/// chrome are exempt: their density is the IDE pattern, not a defect.
fn finding_congestion(
    widgets: &[(&WidgetGeometry, Rect)],
    regions: &[ScreenRegion],
    findings: &mut Vec<ScreenFinding>,
) {
    for (i, region) in regions.iter().enumerate() {
        if !matches!(region.kind, RegionKind::Content) || region.grid_like {
            continue;
        }
        let inside: Vec<(&WidgetGeometry, Rect)> = widgets
            .iter()
            .copied()
            .filter(|(w, r)| w.enabled && (w.senses_click || w.senses_drag) && region.rect.to_egui().contains_rect(*r))
            .collect();
        if inside.len() < CONGESTION_MIN_WIDGETS {
            continue;
        }
        // Close-packing score: share of nearest-neighbor distances below the
        // congestion gap, on whichever axis the region flows.
        let mut packed = 0usize;
        for (a_idx, (wa, ra)) in inside.iter().enumerate() {
            let nearest = inside
                .iter()
                .enumerate()
                .filter(|(b_idx, _)| *b_idx != a_idx)
                .map(|(_, (_, rb))| {
                    let x = (rb.min.x - ra.max.x).max(ra.min.x - rb.max.x).max(0.0);
                    let y = (rb.min.y - ra.max.y).max(ra.min.y - rb.max.y).max(0.0);
                    x + y
                })
                .fold(f32::MAX, f32::min);
            let _ = wa;
            if nearest <= CONGESTION_GAP_PX {
                packed += 1;
            }
        }
        let share = packed as f32 / inside.len() as f32;
        if packed >= CONGESTION_MIN_WIDGETS && share >= 0.8 {
            findings.push(ScreenFinding {
                rule: "screen.control_congestion",
                kind: FindingKind::Heuristic,
                severity: Severity::Warning,
                confidence: Confidence::Medium,
                widget_id: None,
                widget_debug: None,
                region: Some(i),
                evidence: format!(
                    "{packed} of {} interactive widgets packed within {CONGESTION_GAP_PX:.0}px in a non-data region",
                    inside.len()
                ),
            });
        }
    }
}

/// Flat accessibility structure on a large screen: enough nodes to demand
/// landmark/grouping roles but almost none emitted — keyboard/AT navigation
/// sees one undifferentiated list. Below [`SEMANTIC_MIN_NODES`] there is no
/// structure to demand → the rule doesn't apply.
fn finding_flat_semantics(semantic: Option<&SemanticSnapshot>, findings: &mut Vec<ScreenFinding>) {
    let Some(sem) = semantic else {
        return; // coverage.gaps already explains the missing data
    };
    use egui::accesskit::Role;
    let nodes = sem.nodes.len();
    if nodes < SEMANTIC_MIN_NODES {
        return;
    }
    let containers = sem
        .nodes
        .values()
        .filter(|n| {
            matches!(
                n.role,
                Role::Pane
                    | Role::GenericContainer
                    | Role::ScrollView
                    | Role::Group
                    | Role::List
                    | Role::Table
                    | Role::Tree
                    | Role::Toolbar
                    | Role::TabPanel
                    | Role::MenuBar
            )
        })
        .count();
    if containers as f32 / nodes as f32 >= SEMANTIC_CONTAINER_MIN_RATIO {
        return;
    }
    findings.push(ScreenFinding {
        rule: "screen.flat_semantics",
        kind: FindingKind::Heuristic,
        severity: Severity::Warning,
        confidence: Confidence::Medium,
        widget_id: None,
        widget_debug: None,
        region: None,
        evidence: format!(
            "{nodes} semantic nodes but only {containers} container-role nodes (<{:.0}%) — flat AT structure",
            SEMANTIC_CONTAINER_MIN_RATIO * 100.0
        ),
    });
}

/// Scope from env: the fixture window when `DB_PRO_AUDIT_SCREEN_FIXTURE` is
/// set, otherwise the real viewport. Kept on the analyzer side so unit tests
/// can pass an explicit [`ScreenScope`] without touching env.
pub fn fixture_scope() -> ScreenScope {
    if std::env::var_os(SCREEN_FIXTURE_ENV).is_some_and(|v| !v.is_empty() && v != "0") {
        ScreenScope::Fixture {
            layer: screen_fixture_layer_id(),
            rect: SCREEN_FIXTURE_RECT,
        }
    } else {
        ScreenScope::Full
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dev_tools::geometry::GeometryRect;
    use egui::{Pos2, Rect};

    /// One interactive widget on `layer`, parented to `parent` — the fixture
    /// builder mirrors `audit::tests::widget` with a parent parameter.
    fn widget_at(id: u64, _layer: u64, parent: u64, rect: Rect) -> WidgetGeometry {
        let l = scope_layer(1);
        WidgetGeometry {
            id: Id::new(id),
            id_debug: Id::new(id).short_debug_format(),
            parent_id: Id::new(parent),
            parent_debug: Id::new(parent).short_debug_format(),
            layer: l,
            layer_debug: l.short_debug_format(),
            rect: GeometryRect::from_egui(rect),
            interact_rect: GeometryRect::from_egui(rect),
            clipped: false,
            senses_click: true,
            senses_drag: false,
            focusable: false,
            enabled: true,
            widget_type: Some("Button".into()),
            label: None,
            hint_text: None,
            current_text_value: None,
            selected: None,
            value: None,
        }
    }

    fn snapshot(widgets: Vec<WidgetGeometry>) -> GeometrySnapshot {
        let mut groups: Vec<(egui::LayerId, Vec<WidgetGeometry>)> = Vec::new();
        for w in widgets {
            match groups.iter_mut().find(|(l, _)| *l == w.layer) {
                Some((_, ws)) => ws.push(w),
                None => groups.push((w.layer, vec![w])),
            }
        }
        GeometrySnapshot { layers: groups }
    }

    /// The scope layer all fixture widgets live on — one layer id, so the
    /// fixture filter keeps them in scope.
    fn scope_layer(salt: u64) -> egui::LayerId {
        egui::LayerId::new(egui::Order::Middle, Id::new(("scr", salt)))
    }

    /// A virtual 800×600 screen — the fixture scope used by every test.
    /// Widgets land on `scope_layer(1)`; the scope filter matches that
    /// family by order so sibling layers (a dialog) also stay visible.
    fn scope() -> (ScreenScope, egui::LayerId) {
        let layer = scope_layer(1);
        (
            ScreenScope::Fixture {
                layer,
                rect: Rect::from_min_size(Pos2::new(0.0, 0.0), egui::vec2(800.0, 600.0)),
            },
            layer,
        )
    }

    fn analyze(widgets: Vec<WidgetGeometry>, sem: Option<&SemanticSnapshot>) -> ScreenAnalysis {
        let ctx = egui::Context::default();
        let (scope, _) = scope();
        analyze_screen(&ctx, &snapshot(widgets), sem, &scope)
    }

    fn findings_for<'a>(a: &'a ScreenAnalysis, rule: &str) -> Vec<&'a ScreenFinding> {
        a.findings.iter().filter(|f| f.rule == rule).collect()
    }

    /// A uniform n-item column, all in one parent — the rhythm-fixture base.
    fn column(parent: u64, x: f32, y0: f32, gap: f32, w: f32, h: f32, n: u64) -> Vec<WidgetGeometry> {
        (0..n)
            .map(|i| {
                widget_at(
                    1000 + i,
                    1,
                    parent,
                    Rect::from_min_size(Pos2::new(x, y0 + i as f32 * (h + gap)), egui::vec2(w, h)),
                )
            })
            .collect()
    }

    // ── region detection + classification ────────────────────────────

    #[test]
    fn regions_split_on_whitespace_and_classify_chrome() {
        let ctx = egui::Context::default();
        let (scope, layer) = scope();
        let mut ws = Vec::new();
        // Top bar: 4 buttons in a row.
        for i in 0..4 {
            ws.push(widget_at(
                10 + i,
                1,
                100,
                Rect::from_min_size(Pos2::new(8.0 + i as f32 * 88.0, 4.0), egui::vec2(80.0, 28.0)),
            ));
        }
        // Rail: 7 items running down the left edge (55% of screen height —
        // tall enough to count as edge chrome, not a floating nav block).
        ws.extend(column(200, 8.0, 48.0, 8.0, 120.0, 40.0, 7));
        // Content rows on the right.
        ws.extend(column(300, 160.0, 48.0, 8.0, 200.0, 32.0, 5));
        let snap = snapshot(ws);
        let a = analyze_screen(&ctx, &snap, None, &scope);
        let kinds: Vec<_> = a.regions.iter().map(|r| r.kind.label()).collect();
        assert!(kinds.contains(&"top_bar"), "top strip must classify: {kinds:?}");
        assert!(kinds.contains(&"left_rail"), "left column must classify: {kinds:?}");
        assert!(
            kinds.contains(&"side_panel") || kinds.contains(&"content"),
            "interior column must classify: {kinds:?}"
        );
        let _ = layer;
    }

    #[test]
    fn empty_scope_is_insufficient_not_clean() {
        let a = analyze(Vec::new(), None);
        assert!(!a.sufficient);
        assert!(a.gaps.contains(&AnalysisGap::NoGeometry));
        assert!(a.gaps.contains(&AnalysisGap::NoSemanticTree));
        assert!(a.hierarchy.nodes.is_none(), "missing data must read UNKNOWN");
    }

    #[test]
    fn semantic_tree_fills_hierarchy() {
        use egui::accesskit::{Node, Role, Tree};
        let root = egui::accesskit_root_id().accesskit_id();
        let child = Id::new("c").accesskit_id();
        let mut r = Node::new(Role::Window);
        r.push_child(child);
        let sem = SemanticSnapshot::from_update(&egui::accesskit::TreeUpdate {
            nodes: vec![(root, r), (child, Node::new(Role::Button))],
            tree: Some(Tree::new(root)),
            focus: root,
            tree_id: egui::accesskit::TreeId::ROOT,
        })
        .unwrap();
        let a = analyze(Vec::new(), Some(&sem));
        assert_eq!(a.hierarchy.nodes, Some(2));
        assert_eq!(a.hierarchy.max_depth, Some(2));
    }

    // ── findings: fixtures prove fire and don't-fire ─────────────────

    #[test]
    fn off_screen_flags_widget_outside_viewport() {
        let mut ws = column(1, 8.0, 8.0, 8.0, 100.0, 32.0, 4);
        // Interactive rect fully outside the 800×600 fixture screen.
        ws.push(widget_at(
            9,
            1,
            2,
            Rect::from_min_size(Pos2::new(820.0, 10.0), egui::vec2(80.0, 28.0)),
        ));
        let a = analyze(ws, None);
        let hits = findings_for(&a, "screen.off_screen");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].kind, FindingKind::Objective);
        assert_eq!(hits[0].severity, Severity::Warning);
    }

    #[test]
    fn off_screen_ignores_widget_inside() {
        let ws = column(1, 8.0, 8.0, 8.0, 100.0, 32.0, 4);
        let a = analyze(ws, None);
        assert!(findings_for(&a, "screen.off_screen").is_empty());
    }

    #[test]
    fn spacing_outlier_flags_interior_break_not_edge() {
        // Uniform run: rows at 8px pitch, but row 4 is 100px late — interior
        // break with siblings on both sides.
        let mut ws = Vec::new();
        let pitches = [8.0, 8.0, 8.0, 100.0, 8.0];
        let mut y = 40.0;
        for (i, p) in pitches.iter().enumerate() {
            if i > 0 {
                y += p;
            }
            ws.push(widget_at(
                50 + i as u64,
                1,
                1,
                Rect::from_min_size(Pos2::new(200.0, y), egui::vec2(150.0, 32.0)),
            ));
            y += 32.0;
        }
        let a = analyze(ws, None);
        let hits = findings_for(&a, "screen.spacing_outlier");
        assert_eq!(hits.len(), 1);
        assert!(hits[0].evidence.contains("100px"));
        assert_eq!(hits[0].kind, FindingKind::Heuristic);
    }

    #[test]
    fn spacing_outlier_skips_edge_anchored_gap() {
        // Same rhythm but the big gap ends the run — a bottom-pinned item,
        // which is anchoring, not a rhythm defect.
        let mut ws = Vec::new();
        let pitches = [8.0, 8.0, 8.0, 8.0, 100.0];
        let mut y = 40.0;
        for (i, p) in pitches.iter().enumerate() {
            if i > 0 {
                y += p;
            }
            ws.push(widget_at(
                60 + i as u64,
                1,
                1,
                Rect::from_min_size(Pos2::new(200.0, y), egui::vec2(150.0, 32.0)),
            ));
            y += 32.0;
        }
        let a = analyze(ws, None);
        assert!(
            findings_for(&a, "screen.spacing_outlier").is_empty(),
            "edge gap is anchoring, not a defect"
        );
    }

    #[test]
    fn spacing_outlier_skips_mixed_size_group() {
        // Justified toolbar: same large gap but items vary in height → not a
        // uniform run, so no rhythm exists to break.
        let mut ws = Vec::new();
        let mut y = 40.0;
        let heights = [32.0, 24.0, 40.0, 28.0, 36.0, 30.0];
        for (i, h) in heights.iter().enumerate() {
            if i > 0 {
                y += if i == 3 { 100.0 } else { 8.0 };
            }
            ws.push(widget_at(
                70 + i as u64,
                1,
                1,
                Rect::from_min_size(Pos2::new(200.0, y), egui::vec2(150.0, *h)),
            ));
            y += h;
        }
        let a = analyze(ws, None);
        assert!(
            findings_for(&a, "screen.spacing_outlier").is_empty(),
            "mixed-size groups have no dominant rhythm to break"
        );
    }

    #[test]
    fn layout_inversion_flags_dominant_side_panel() {
        let ctx = egui::Context::default();
        let (scope, layer) = scope();
        let mut ws = Vec::new();
        // Side-anchored, non-grid panel occupying 62% of the screen width.
        ws.extend(column(1, 8.0, 48.0, 8.0, 480.0, 30.0, 8));
        // Narrow content column on the right.
        ws.extend(column(2, 540.0, 48.0, 8.0, 200.0, 30.0, 8));
        let snap = snapshot(ws);
        let a = analyze_screen(&ctx, &snap, None, &scope);
        let hits = findings_for(&a, "screen.layout_inversion");
        assert_eq!(hits.len(), 1, "side panel outweighing content must flag");
        assert!(hits[0].evidence.contains('%'));
        let _ = layer;
    }

    #[test]
    fn layout_inversion_passes_normal_layout() {
        let ctx = egui::Context::default();
        let (scope, _l) = scope();
        let mut ws = Vec::new();
        ws.extend(column(1, 8.0, 48.0, 8.0, 120.0, 30.0, 8));
        ws.extend(column(2, 200.0, 48.0, 8.0, 500.0, 30.0, 8));
        let snap = snapshot(ws);
        let a = analyze_screen(&ctx, &snap, None, &scope);
        assert!(findings_for(&a, "screen.layout_inversion").is_empty());
    }

    #[test]
    fn no_primary_needs_busy_fragmented_screen() {
        // Sparse screen: two small regions, little interactive coverage —
        // a welcome screen legitimately has no dominant region.
        let ws = column(1, 100.0, 100.0, 8.0, 100.0, 28.0, 4);
        let a = analyze(ws, None);
        assert!(findings_for(&a, "screen.no_primary_region").is_empty());
    }

    #[test]
    fn findings_report_kind_not_opinion() {
        // Every finding carries an explicit objective/heuristic split — the
        // contract that keeps measured defects separate from taste.
        let mut ws = column(1, 8.0, 8.0, 8.0, 100.0, 32.0, 4);
        ws.push(widget_at(
            9,
            1,
            2,
            Rect::from_min_size(Pos2::new(820.0, 10.0), egui::vec2(80.0, 28.0)),
        ));
        let a = analyze(ws, None);
        for f in &a.findings {
            assert!(matches!(f.kind, FindingKind::Objective | FindingKind::Heuristic));
            assert!(!f.evidence.is_empty(), "finding without evidence violates the contract");
        }
    }
}
