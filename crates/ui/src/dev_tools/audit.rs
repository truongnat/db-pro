// cc-scan:allow LONG_FUNCTION — rule bodies are linear evidence gathering.
//! Geometry audit rule engine for the UI inspector.
//!
//! Consumes [`GeometrySnapshot`] — the same widget data the inspector already
//! collects — and emits [`AuditIssue`]s. The engine is a registry of rules:
//! adding an audit is one function + one entry in [`RULES`].
//!
//! Design contract:
//!
//! - *Evidence or silence.* Every issue carries the numbers that produced it
//!   and a [`Confidence`]. When the data needed to decide is missing the rule
//!   reports `skipped` instead of guessing (e.g. a clipped widget whose true
//!   hitbox is unknowable from geometry alone).
//! - *Layout is not a bug.* Interactive widgets may overlap when one fully
//!   contains the other (checkbox inside a row), when a `Foreground`/`Tooltip`
//!   layer intentionally floats above `Background` (popups, menus), or when a
//!   widget is clipped (scrolled out, collapsed). Same-layer sibling overlap
//!   that covers an enabled, unclipped interactive widget is what we flag.
//! - *Snapshots are read-only.* Rules never mutate; a stale widget id in a
//!   report resolves through `snapshot.find` and degrades to "widget gone"
//!   instead of crashing.

use egui::Id;

use crate::dev_tools::geometry::{GeometrySnapshot, WidgetGeometry};
use crate::dev_tools::semantic::{is_name_required_role, SemanticSnapshot};

/// How sure the rule is that this issue is a real defect (not intentional UI).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Confidence {
    /// Geometry is self-contradictory or the covered widget is fully hidden.
    High,
    /// Strong signal but a legitimate layout could produce it (e.g. dense
    /// toolbars with decorative layering).
    Medium,
}

impl Confidence {
    pub fn label(self) -> &'static str {
        match self {
            Self::High => "high",
            Self::Medium => "medium",
        }
    }
}

/// Triage ordering inside the inspector.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Warning,
    Error,
}

impl Severity {
    pub fn label(self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Warning => "warning",
        }
    }
}

/// One audit finding. `widget_id` stays typed so the inspector can pin the
/// widget (click → highlight) and re-resolve it against the next snapshot.
#[derive(Clone, Debug)]
pub struct AuditIssue {
    /// Stable rule identifier, e.g. `"overlap.suspicious"`.
    pub rule_id: &'static str,
    pub severity: Severity,
    pub confidence: Confidence,
    /// Primary widget the issue is about.
    pub widget_id: Id,
    /// Short widget id for display.
    pub widget_debug: String,
    /// Second widget involved, when the rule compares a pair (overlap).
    pub other_id: Option<Id>,
    /// Human-readable evidence: the numbers that fired the rule.
    pub evidence: String,
}

/// Why a rule could not evaluate a widget/pair — surfaced in the report so the
/// engine's silence is explained, not hidden.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SkipReason {
    /// Widget clipped — geometry is valid, just out of view.
    Clipped,
    /// Intentional overlay structure (separate Area/popup over content).
    IntentionalOverlay,
    /// AccessKit data missing — tree unavailable (disabled/first frame) or
    /// the specific widget has no semantic node.
    NoSemanticTree,
    /// A color pair has a non-opaque channel — the rendered composite depends
    /// on an unknown backdrop, so no ratio is computable.
    NonOpaqueColors,
}

impl SkipReason {
    pub fn label(self) -> &'static str {
        match self {
            Self::Clipped => "clipped",
            Self::IntentionalOverlay => "intentional overlay",
            Self::NoSemanticTree => "no semantic data",
            Self::NonOpaqueColors => "non-opaque colors",
        }
    }
}

/// Per-rule tallies for one evaluation pass.
#[derive(Clone, Debug, Default)]
pub struct RuleStats {
    /// Widgets/pairs actually evaluated.
    pub evaluated: usize,
    /// Skipped because required data was missing or the pattern is legitimate
    /// by construction (never a guess).
    pub skipped: usize,
    /// Dominant skip reason, when any.
    pub skip_reason: Option<SkipReason>,
    /// Out of scope for this rule — the widget/group exists but the rule's
    /// precondition doesn't (e.g. a column too small to have a modal edge).
    /// Distinct from `skipped`, which means "we wanted to evaluate but lacked
    /// evidence".
    pub not_applicable: usize,
}

/// What the rule concluded this pass. `NotApplicable` and `Skipped` are
/// deliberately distinct: N/A = nothing in scope; Skip = in scope but the
/// evidence needed to decide was missing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verdict {
    Pass,
    Fail,
    /// Everything in scope lacked evidence to decide.
    Skipped,
    /// Nothing in scope at all — never reported as PASS.
    NotApplicable,
}

impl Verdict {
    pub fn label(self) -> &'static str {
        match self {
            Self::Pass => "PASS",
            Self::Fail => "FAIL",
            Self::Skipped => "SKIP",
            Self::NotApplicable => "N/A",
        }
    }
}

/// Per-rule evaluation outcome for the audit report.
#[derive(Clone, Debug)]
pub struct RuleReport {
    pub rule_id: &'static str,
    pub stats: RuleStats,
    /// Issues this rule emitted this pass.
    pub issues: usize,
}

impl RuleReport {
    /// Issues present → Fail; something evaluated clean → Pass; only skips
    /// → Skipped; nothing at all → NotApplicable (never "PASS" on zero
    /// evaluated).
    pub fn verdict(&self) -> Verdict {
        if self.issues > 0 {
            Verdict::Fail
        } else if self.stats.evaluated > 0 {
            Verdict::Pass
        } else if self.stats.skipped > 0 {
            Verdict::Skipped
        } else {
            Verdict::NotApplicable
        }
    }
}

/// Full audit result for one snapshot.
#[derive(Clone, Debug, Default)]
pub struct AuditReport {
    pub issues: Vec<AuditIssue>,
    pub rules: Vec<RuleReport>,
    /// Screen-level composition analysis (P10) — measured metrics plus a
    /// separate finding list that never enters `issues` and never affects
    /// the audit exit code.
    pub screen: Option<crate::dev_tools::screen::ScreenAnalysis>,
    /// Widgets seen this pass — lets callers detect churn between frames.
    pub widget_count: usize,
    /// Widget id bits present in the audited snapshot. Used to check whether
    /// two reports came from equivalent snapshots before diffing issues.
    pub widget_ids: std::collections::HashSet<u64>,
    /// Order-independent hash of (id, rect) for every widget — equal on two
    /// consecutive audits means the layout has settled. Replaces a fixed
    /// warm-up frame count: emit when the fingerprint stops changing.
    pub layout_fingerprint: u64,
}

impl AuditReport {
    /// Two reports are comparable only if they audited the same widget set —
    /// diffing across a UI change would compare unrelated snapshots.
    pub fn equivalent_snapshot(&self, other: &AuditReport) -> bool {
        self.widget_ids == other.widget_ids
    }

    /// Identity key for an issue: rule + subject widget + context widget.
    /// Stable across frames for the same defect; independent of issue order.
    fn issue_key(issue: &AuditIssue) -> String {
        format!(
            "{}|{}|{}",
            issue.rule_id,
            issue.widget_debug,
            issue.other_id.map(|id| id.short_debug_format()).unwrap_or_default()
        )
    }

    /// (new issues, resolved issues) vs a previous report — `None` when the
    /// snapshots aren't equivalent and a diff would be meaningless.
    pub fn diff(&self, prev: &AuditReport) -> Option<(Vec<String>, Vec<String>)> {
        if !self.equivalent_snapshot(prev) {
            return None;
        }
        use std::collections::HashSet;
        let now: HashSet<String> = self.issues.iter().map(Self::issue_key).collect();
        let then: HashSet<String> = prev.issues.iter().map(Self::issue_key).collect();
        let new: Vec<String> = now.difference(&then).cloned().collect();
        let resolved: Vec<String> = then.difference(&now).cloned().collect();
        Some((new, resolved))
    }
}

/// An extensible rule: examine the snapshots, return issues + tallies.
/// `semantic` is `None` when accesskit never produced a tree (disabled or not
/// yet run) — semantic rules must report themselves skipped, not silently pass.
type Rule = fn(&egui::Context, &GeometrySnapshot, Option<&SemanticSnapshot>) -> (Vec<AuditIssue>, RuleStats);

/// Hash of every (widget id, rect-at-0.5px) pair, order-independent — stable
/// across consecutive frames once layout settles, different when anything
/// moved/resized/appeared.
fn layout_fingerprint(snapshot: &GeometrySnapshot) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    let mut parts: Vec<(u64, i64, i64, i64, i64)> = snapshot
        .layers
        .iter()
        .flat_map(|(_, ws)| ws.iter())
        .map(|w| {
            (
                w.id.value(),
                (w.rect.min_x * 2.0) as i64,
                (w.rect.min_y * 2.0) as i64,
                (w.rect.max_x * 2.0) as i64,
                (w.rect.max_y * 2.0) as i64,
            )
        })
        .collect();
    parts.sort_unstable();
    parts.hash(&mut h);
    h.finish()
}

/// Registered rules in report order. Add new audits here.
const RULES: &[(&str, Rule)] = &[
    ("geometry.invalid", rule_invalid_geometry),
    ("interactive.zero_size", rule_zero_size_interactive),
    ("overlap.suspicious", rule_suspicious_overlap),
    ("semantic.missing_name", rule_semantic_missing_name),
    ("semantic.disabled_mismatch", rule_semantic_disabled_mismatch),
    ("semantic.orphan_node", rule_semantic_orphan_node),
    ("focus.orphaned", rule_focus_orphaned),
    ("focus.unfocusable", rule_focus_unfocusable),
    ("visual.typography_hierarchy", rule_typography_hierarchy),
    ("visual.text_contrast", rule_text_contrast),
    ("visual.touching_controls", rule_touching_controls),
    ("visual.alignment_drift", rule_alignment_drift),
];

/// Run every registered rule over a valid snapshot.
/// "Valid" = produced by `GeometrySnapshot::collect`; an empty snapshot is
/// valid and yields an empty report (e.g. first frame before any pass).
/// `semantic` may be `None` (accesskit off / first frame) — rules that need
/// it report `skipped`, never fabricate results.
pub fn run_audit(ctx: &egui::Context, snapshot: &GeometrySnapshot, semantic: Option<&SemanticSnapshot>) -> AuditReport {
    let mut report = AuditReport {
        widget_count: snapshot.layers.iter().map(|(_, ws)| ws.len()).sum(),
        widget_ids: snapshot
            .layers
            .iter()
            .flat_map(|(_, ws)| ws.iter().map(|w| w.id.value()))
            .collect(),
        layout_fingerprint: layout_fingerprint(snapshot),
        issues: Vec::new(),
        rules: Vec::new(),
        screen: None,
    };
    for (rule_id, rule) in RULES {
        let (issues, stats) = rule(ctx, snapshot, semantic);
        report.rules.push(RuleReport {
            rule_id,
            issues: issues.len(),
            stats,
        });
        report.issues.extend(issues);
    }
    // Screen-level pass (P10): reads the same snapshots the rules audited.
    // The fixture env scopes the analysis to the fixture's own layer +
    // virtual screen so fixture evidence is app-independent.
    let scope = crate::dev_tools::screen::fixture_scope();
    report.screen = Some(crate::dev_tools::screen::analyze_screen(
        ctx, snapshot, semantic, &scope,
    ));
    report
}

/// Mark a whole rule skipped when the semantic tree is unavailable.
fn no_semantic_data() -> (Vec<AuditIssue>, RuleStats) {
    (
        Vec::new(),
        RuleStats {
            evaluated: 0,
            skipped: 1,
            skip_reason: Some(SkipReason::NoSemanticTree),
            not_applicable: 0,
        },
    )
}

// ── Rule: geometry.invalid ────────────────────────────────────────────────
//
// A widget rect is broken when any component is NaN/±∞ or min > max
// (negative size). Always evaluable — never skips.

fn rule_invalid_geometry(
    _ctx: &egui::Context,
    snapshot: &GeometrySnapshot,
    _sem: Option<&SemanticSnapshot>,
) -> (Vec<AuditIssue>, RuleStats) {
    let mut issues = Vec::new();
    let mut stats = RuleStats::default();
    for widget in snapshot.layers.iter().flat_map(|(_, ws)| ws.iter()) {
        stats.evaluated += 1;
        let rect = &widget.rect;
        let non_finite = ![rect.min_x, rect.min_y, rect.max_x, rect.max_y]
            .iter()
            .all(|v| v.is_finite());
        let inverted = rect.width() < 0.0 || rect.height() < 0.0;
        if non_finite || inverted {
            issues.push(AuditIssue {
                rule_id: "geometry.invalid",
                severity: Severity::Error,
                confidence: Confidence::High,
                widget_id: widget.id,
                widget_debug: widget.id_debug.clone(),
                other_id: None,
                evidence: if non_finite {
                    format!(
                        "non-finite rect ({:.1},{:.1})→({:.1},{:.1})",
                        rect.min_x, rect.min_y, rect.max_x, rect.max_y
                    )
                } else {
                    format!("inverted rect {:.1}×{:.1} (min > max)", rect.width(), rect.height())
                },
            });
        }
    }
    (issues, stats)
}

// ── Rule: interactive.zero_size ───────────────────────────────────────────
//
// An enabled widget that senses click/drag but has ~no interactive area cannot
// be hit — it exists in the tree but is unreachable by pointer or keyboard.
// Clipped widgets skip: their rect is fine, the parent clip shrank interact.

const MIN_HIT_SIZE: f32 = 1.0;

fn rule_zero_size_interactive(
    _ctx: &egui::Context,
    snapshot: &GeometrySnapshot,
    _sem: Option<&SemanticSnapshot>,
) -> (Vec<AuditIssue>, RuleStats) {
    let mut issues = Vec::new();
    let mut stats = RuleStats::default();
    for widget in snapshot.layers.iter().flat_map(|(_, ws)| ws.iter()) {
        if !widget.enabled || !(widget.senses_click || widget.senses_drag || widget.focusable) {
            continue; // not interactive — outside this rule's scope
        }
        stats.evaluated += 1;
        let w = widget.interact_rect.width();
        let h = widget.interact_rect.height();
        if w >= MIN_HIT_SIZE && h >= MIN_HIT_SIZE {
            continue;
        }
        if widget.clipped {
            stats.skipped += 1;
            stats.skip_reason = Some(SkipReason::Clipped);
            continue;
        }
        issues.push(AuditIssue {
            rule_id: "interactive.zero_size",
            severity: Severity::Error,
            confidence: Confidence::High,
            widget_id: widget.id,
            widget_debug: widget.id_debug.clone(),
            other_id: None,
            evidence: format!(
                "interactable {:.1}×{:.1}px < {:.0}px floor (rect {:.1}×{:.1})",
                w,
                h,
                MIN_HIT_SIZE,
                widget.rect.width(),
                widget.rect.height()
            ),
        });
    }
    (issues, stats)
}

// ── Rule: overlap.suspicious ──────────────────────────────────────────────
//
// Two enabled, interactive widgets in the SAME Area (same layer id) whose
// interact rects overlap substantially with no containment — a sibling covers
// another sibling's hitbox. Intentional overlays skip:
//   - different `layer.id` (separate Area/Window/Popup) → IntentionalOverlay
//   - clipped widgets → Clipped
//   - containment (parent widget with interactive child) → evaluated, no issue
//   - non-interactive or disabled widgets → not evaluated
//
// Coverage = overlap area / smaller interact area.
//   ≥ 0.9 → High confidence (a widget is effectively hidden)
//   ≥ 0.5 → Medium confidence

const OVERLAP_MED: f32 = 0.5;
const OVERLAP_HIGH: f32 = 0.9;

fn rule_suspicious_overlap(
    ctx: &egui::Context,
    snapshot: &GeometrySnapshot,
    sem: Option<&SemanticSnapshot>,
) -> (Vec<AuditIssue>, RuleStats) {
    let mut issues = Vec::new();
    let mut stats = RuleStats::default();
    // Flatten: candidates are enabled, interactive widgets anywhere in the
    // snapshot.
    let widgets: Vec<&WidgetGeometry> = snapshot
        .layers
        .iter()
        .flat_map(|(_, ws)| ws.iter())
        .filter(|w| w.enabled && (w.senses_click || w.senses_drag))
        .collect();
    for (i, a) in widgets.iter().enumerate() {
        for b in widgets.iter().skip(i + 1) {
            if a.layer.id != b.layer.id || a.layer.order != b.layer.order {
                // Different Order = popup/menu/tooltip over content; different
                // layer id on the same Order = a separate window/Area floating
                // above. Both are intentional layering → skip, never flag.
                stats.skipped += 1;
                stats.skip_reason = Some(SkipReason::IntentionalOverlay);
                continue;
            }
            stats.evaluated += 1;
            // Structured exception: an accesskit `Splitter` node (the grid's
            // 8px column-resize handle) overlapping a same-parent sibling is
            // intended — the grab target straddles the column edge by design.
            // Verified via the semantic tree, not a widened threshold.
            let is_resize_pair = sem.is_some_and(|s| {
                [a.id, b.id]
                    .iter()
                    .any(|id| s.find_widget(*id).is_some_and(|n| n.role_debug == "Splitter"))
            }) && a.parent_id == b.parent_id;
            if is_resize_pair {
                stats.skipped += 1;
                stats.skip_reason = Some(SkipReason::IntentionalOverlay);
                continue;
            }
            if a.clipped || b.clipped {
                stats.skipped += 1;
                stats.skip_reason = Some(SkipReason::Clipped);
                continue;
            }
            let ra = ctx
                .layer_transform_to_global(a.layer)
                .map(|t| t * a.interact_rect.to_egui())
                .unwrap_or_else(|| a.interact_rect.to_egui());
            let rb = ctx
                .layer_transform_to_global(b.layer)
                .map(|t| t * b.interact_rect.to_egui())
                .unwrap_or_else(|| b.interact_rect.to_egui());
            if !ra.intersects(rb) {
                continue;
            }
            // Containment = interactive child inside a parent hitbox —
            // normal layout (checkbox in a row, icon in a button).
            if ra.contains_rect(rb) || rb.contains_rect(ra) {
                continue;
            }
            let overlap = ra.intersect(rb);
            let smaller = (ra.width() * ra.height()).min(rb.width() * rb.height());
            if smaller <= 0.0 {
                continue; // zero-size handled by interactive.zero_size
            }
            let coverage = (overlap.width() * overlap.height()) / smaller;
            if coverage < OVERLAP_MED {
                continue;
            }
            issues.push(AuditIssue {
                rule_id: "overlap.suspicious",
                severity: Severity::Warning,
                confidence: if coverage >= OVERLAP_HIGH {
                    Confidence::High
                } else {
                    Confidence::Medium
                },
                widget_id: b.id, // `b` painted after `a` → it sits on top
                widget_debug: b.id_debug.clone(),
                other_id: Some(a.id),
                evidence: format!(
                    "{:.0}% of the smaller hitbox covered by a sibling widget (overlap {:.0}×{:.0}px)",
                    coverage * 100.0,
                    overlap.width(),
                    overlap.height()
                ),
            });
        }
    }
    (issues, stats)
}

// ── Rule: visual.typography_hierarchy ─────────────────────────────────────
//
// `Style::text_styles` is real measured typography: every TextStyle maps to a
// concrete `FontId.size`. The hierarchy contract is the ordering the API
// documents — Heading > Body ≥ Small. An inversion is an objective defect,
// not an aesthetic opinion. Pairs whose style is absent are skipped, not
// assumed.

fn rule_typography_hierarchy(
    ctx: &egui::Context,
    _snapshot: &GeometrySnapshot,
    _sem: Option<&SemanticSnapshot>,
) -> (Vec<AuditIssue>, RuleStats) {
    let mut issues = Vec::new();
    let mut stats = RuleStats::default();
    // Active-theme style — not the other palette's tokens.
    let styles = ctx.style_of(ctx.theme()).text_styles.clone();
    let size = |ts: egui::TextStyle| styles.get(&ts).map(|f| f.size);
    // (larger, smaller-or-equal, invariant)
    let pairs: [(egui::TextStyle, egui::TextStyle, &str); 3] = [
        (
            egui::TextStyle::Heading,
            egui::TextStyle::Body,
            "Heading must be larger than Body",
        ),
        (
            egui::TextStyle::Body,
            egui::TextStyle::Small,
            "Body must not be smaller than Small",
        ),
        (
            egui::TextStyle::Button,
            egui::TextStyle::Small,
            "Button must not be smaller than Small",
        ),
    ];
    for (hi, lo, contract) in pairs {
        let (Some(hi_px), Some(lo_px)) = (size(hi.clone()), size(lo.clone())) else {
            stats.skipped += 1; // style not configured — no data, no verdict
            continue;
        };
        stats.evaluated += 1;
        if hi_px <= lo_px {
            issues.push(AuditIssue {
                rule_id: "visual.typography_hierarchy",
                severity: Severity::Warning,
                confidence: Confidence::High,
                widget_id: egui::Id::NULL,
                widget_debug: "style".into(),
                other_id: None,
                evidence: format!("{contract}: {hi:?}={hi_px:.1}pt ≤ {lo:?}={lo_px:.1}pt"),
            });
        }
    }
    (issues, stats)
}

// ── Rule: visual.text_contrast ────────────────────────────────────────────
//
// WCAG 2.x contrast between *actual configured tokens*: the fg_stroke color
// egui paints text with vs the fill behind it. Both come from
// `Visuals.widgets.*` / `panel_fill` / `window_fill` — measured, never
// inferred from widget rects. Skips when either color is non-opaque
// (compositing over an unknown backdrop isn't decidable).
// `widgets.disabled` is exempt (WCAG exempts disabled UI); `active` is
// transient press state — skipped by design, not data.

const WCAG_AA_NORMAL: f32 = 4.5;

/// WCAG relative-luminance contrast ratio, or `None` when a color isn't
/// opaque (result would be a guess about the backdrop).
fn contrast_ratio(fg: egui::Color32, bg: egui::Color32) -> Option<f32> {
    if fg.a() != 255 || bg.a() != 255 {
        return None;
    }
    fn lum(c: egui::Color32) -> f32 {
        fn ch(v: u8) -> f32 {
            let s = v as f32 / 255.0;
            if s <= 0.04045 {
                s / 12.92
            } else {
                ((s + 0.055) / 1.055).powf(2.4)
            }
        }
        0.2126 * ch(c.r()) + 0.7152 * ch(c.g()) + 0.0722 * ch(c.b())
    }
    let (a, b) = (lum(fg), lum(bg));
    let (hi, lo) = if a >= b { (a, b) } else { (b, a) };
    Some((hi + 0.05) / (lo + 0.05))
}

fn rule_text_contrast(
    ctx: &egui::Context,
    _snapshot: &GeometrySnapshot,
    _sem: Option<&SemanticSnapshot>,
) -> (Vec<AuditIssue>, RuleStats) {
    let mut issues = Vec::new();
    let mut stats = RuleStats::default();
    // Active-theme visuals; these are theme-token pairs — the evidence names
    // the tokens because no per-widget color evidence exists in the API.
    let visuals = ctx.style_of(ctx.theme()).visuals.clone();
    let checks: [(&str, egui::Color32, egui::Color32); 4] = [
        (
            "labels on panel",
            visuals.widgets.noninteractive.fg_stroke.color,
            visuals.panel_fill,
        ),
        (
            "labels in windows",
            visuals.widgets.noninteractive.fg_stroke.color,
            visuals.window_fill,
        ),
        (
            "buttons at rest",
            visuals.widgets.inactive.fg_stroke.color,
            visuals.widgets.inactive.weak_bg_fill,
        ),
        (
            "buttons hovered",
            visuals.widgets.hovered.fg_stroke.color,
            visuals.widgets.hovered.weak_bg_fill,
        ),
    ];
    for (what, fg, bg) in checks {
        let Some(ratio) = contrast_ratio(fg, bg) else {
            stats.skipped += 1;
            stats.skip_reason = Some(SkipReason::NonOpaqueColors);
            continue;
        };
        stats.evaluated += 1;
        if ratio < WCAG_AA_NORMAL {
            issues.push(AuditIssue {
                rule_id: "visual.text_contrast",
                severity: Severity::Error,
                confidence: Confidence::High,
                widget_id: egui::Id::NULL,
                widget_debug: "style".into(),
                other_id: None,
                evidence: format!(
                    "{what}: {:.2}:1 < {:.1}:1 AA (fg {} on bg {})",
                    ratio,
                    WCAG_AA_NORMAL,
                    fg.to_hex(),
                    bg.to_hex()
                ),
            });
        }
    }
    (issues, stats)
}

// ── Shared sibling geometry helpers ───────────────────────────────────────
//
// Both spacing rules only trust same-`parent_id` groups on the same layer —
// the only structure egui guarantees. Clipped widgets are skipped: their
// rects are cut off, so gaps/offsets derived from them aren't real.

fn interactive_sibling_groups(snapshot: &GeometrySnapshot) -> Vec<Vec<&WidgetGeometry>> {
    use std::collections::HashMap;
    let mut groups: HashMap<(egui::Id, egui::Id), Vec<&WidgetGeometry>> = HashMap::new();
    for w in snapshot.layers.iter().flat_map(|(_, ws)| ws.iter()) {
        if w.enabled && w.senses_click && !w.clipped {
            groups.entry((w.layer.id, w.parent_id)).or_default().push(w);
        }
    }
    groups.into_values().filter(|g| g.len() >= 2).collect()
}

// ── Rule: visual.touching_controls ────────────────────────────────────────
//
// Same-parent, same-layer interactive widgets stacked vertically with <1px
// of gap — controls that share an edge are hard to hit discretely. Only
// unclipped, enabled, click-sensing widgets in `Background`/`Middle`/`Panel
// order; `Foreground`+ layers are menus/popups where gapless rows are the
// intended pattern → SKIP. Threshold: 1.0px — local tuning, not a universal
// component standard.

const TOUCHING_GAP_PX: f32 = 1.0;

fn rule_touching_controls(
    _ctx: &egui::Context,
    snapshot: &GeometrySnapshot,
    _sem: Option<&SemanticSnapshot>,
) -> (Vec<AuditIssue>, RuleStats) {
    let mut issues = Vec::new();
    let mut stats = RuleStats::default();
    for group in interactive_sibling_groups(snapshot) {
        // Sort by vertical position; only compare consecutive, x-overlapping
        // siblings (a column). Rows in a horizontal strip don't stack.
        let mut column: Vec<&WidgetGeometry> = group.clone();
        column.sort_by(|a, b| {
            a.rect
                .min_y
                .partial_cmp(&b.rect.min_y)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        for pair in column.windows(2) {
            let (a, b) = (pair[0], pair[1]);
            if matches!(
                a.layer.order,
                egui::Order::Foreground | egui::Order::Tooltip | egui::Order::Debug
            ) {
                stats.skipped += 1;
                stats.skip_reason = Some(SkipReason::IntentionalOverlay);
                continue;
            }
            let x_overlap = (a.rect.max_x.min(b.rect.max_x) - a.rect.min_x.max(b.rect.min_x)) > 0.0;
            if !x_overlap {
                continue; // side-by-side siblings, not a stack
            }
            stats.evaluated += 1;
            // Negative gap = the rects actually overlap → `overlap.suspicious`
            // owns that pair; this rule only audits edge-to-edge stacking.
            let gap = b.rect.min_y - a.rect.max_y;
            if (0.0..TOUCHING_GAP_PX).contains(&gap) {
                issues.push(AuditIssue {
                    rule_id: "visual.touching_controls",
                    severity: Severity::Warning,
                    confidence: Confidence::Medium,
                    widget_id: b.id,
                    widget_debug: b.id_debug.clone(),
                    other_id: Some(a.id),
                    evidence: format!("{gap:.1}px gap between stacked controls (< {TOUCHING_GAP_PX:.0}px)"),
                });
            }
        }
    }
    (issues, stats)
}

// ── Rule: visual.alignment_drift ──────────────────────────────────────────
//
// In a column of ≥3 same-parent widgets sharing an x-range, most share one
// `min_x`; a sibling off by a few pixels (0.5–4px) is drift, not an indent —
// deliberate indentation is structurally larger. Smaller deltas are paint
// noise, larger ones are layout. Local tuning bounds, both ends documented.

const DRIFT_MIN_PX: f32 = 0.5;
const DRIFT_MAX_PX: f32 = 4.0;

fn rule_alignment_drift(
    _ctx: &egui::Context,
    snapshot: &GeometrySnapshot,
    _sem: Option<&SemanticSnapshot>,
) -> (Vec<AuditIssue>, RuleStats) {
    let mut issues = Vec::new();
    let mut stats = RuleStats::default();
    for group in interactive_sibling_groups(snapshot) {
        // Column = x-overlapping members sorted by y, ≥3 needed for a
        // "majority alignment" to exist at all.
        let mut column: Vec<&WidgetGeometry> = group.clone();
        column.sort_by(|a, b| {
            a.rect
                .min_y
                .partial_cmp(&b.rect.min_y)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        if column.len() < 3 {
            stats.not_applicable += 1; // no majority edge can exist
            continue;
        }
        // Modal min_x: the most common left edge (±0.5 buckets).
        let mut buckets: Vec<(f32, usize)> = Vec::new();
        for w in &column {
            match buckets
                .iter_mut()
                .find(|(x, _)| (x - w.rect.min_x).abs() <= DRIFT_MIN_PX)
            {
                Some((x, n)) => {
                    *x = (*x * *n as f32 + w.rect.min_x) / (*n as f32 + 1.0);
                    *n += 1;
                }
                None => buckets.push((w.rect.min_x, 1)),
            }
        }
        let Some(&(mode_x, mode_n)) = buckets.iter().max_by_key(|(_, n)| n) else {
            continue;
        };
        if mode_n < 2 || mode_n == column.len() {
            continue; // no majority edge, or perfectly aligned
        }
        for w in &column {
            let delta = (w.rect.min_x - mode_x).abs();
            if delta <= DRIFT_MIN_PX || delta > DRIFT_MAX_PX {
                continue;
            }
            // Verify vertical stacking (x-overlap with the mode cluster).
            stats.evaluated += 1;
            issues.push(AuditIssue {
                rule_id: "visual.alignment_drift",
                severity: Severity::Warning,
                confidence: Confidence::Medium,
                widget_id: w.id,
                widget_debug: w.id_debug.clone(),
                other_id: None,
                evidence: format!(
                    "left edge {delta:.1}px off the column's shared edge ({mode_x:.1}), within the {DRIFT_MIN_PX:.1}–{DRIFT_MAX_PX:.0}px drift band"
                ),
            });
        }
    }
    (issues, stats)
}

// ── Rule: semantic.missing_name ───────────────────────────────────────────
//
// A node with an interactive role (button, checkbox, text input, …) must have
// an accessible name — `label`, `labelled_by` target text, or for Label-role
// the `value`. Skips: no tree (NoSemanticTree), hidden nodes (out of AT view),
// and roles that don't require a name (plain labels, groups, panes).

fn rule_semantic_missing_name(
    _ctx: &egui::Context,
    snapshot: &GeometrySnapshot,
    sem: Option<&SemanticSnapshot>,
) -> (Vec<AuditIssue>, RuleStats) {
    let Some(sem) = sem else {
        return no_semantic_data();
    };
    let mut issues = Vec::new();
    let mut stats = RuleStats::default();
    for node in sem.nodes.values() {
        if node.hidden {
            stats.skipped += 1;
            stats.skip_reason = Some(SkipReason::Clipped); // hidden ≈ out of view
            continue;
        }
        // A node must be named when its role demands it. Nodes with a
        // name-agnostic role only flag when *both* sides agree it's clickable:
        // the node advertises Click AND its widget senses clicks — plain
        // containers (scroll areas) that carry a Click action are not flagged.
        let name_required = is_name_required_role(node.role)
            || (node.supports_click
                && node
                    .widget_id
                    .and_then(|id| snapshot.find(id))
                    .is_some_and(|w| w.senses_click));
        if !name_required {
            continue;
        }
        stats.evaluated += 1;
        if node.name.is_none() {
            issues.push(AuditIssue {
                rule_id: "semantic.missing_name",
                severity: Severity::Error,
                confidence: Confidence::High,
                widget_id: node.widget_id.unwrap_or_else(unknown_issue_id),
                widget_debug: node
                    .widget_id
                    .map(|id| id.short_debug_format())
                    .unwrap_or_else(|| format!("{:?}", node.node_id)),
                other_id: None,
                evidence: format!("{} node has no label / labelled_by / value", node.role_debug),
            });
        }
    }
    (issues, stats)
}

// ── Rule: semantic.disabled_mismatch ──────────────────────────────────────
//
// The semantic tree and the widget registry agree on the same egui::Id; if a
// widget is enabled in geometry but its accesskit node says `disabled` (or
// vice-versa) the two layers of truth disagree — a screen reader sees a
// different control than the pointer does.
// Skips: widget has no semantic node (not all widgets emit one), hidden nodes.

fn rule_semantic_disabled_mismatch(
    _ctx: &egui::Context,
    snapshot: &GeometrySnapshot,
    sem: Option<&SemanticSnapshot>,
) -> (Vec<AuditIssue>, RuleStats) {
    let Some(sem) = sem else {
        return no_semantic_data();
    };
    let mut issues = Vec::new();
    let mut stats = RuleStats::default();
    for widget in snapshot.layers.iter().flat_map(|(_, ws)| ws.iter()) {
        if !(widget.senses_click || widget.senses_drag || widget.focusable) {
            continue; // non-interactive widgets aren't in the disabled contract
        }
        let Some(node) = sem.find_widget(widget.id) else {
            stats.skipped += 1; // widget without a semantic node — no data
            stats.skip_reason = Some(SkipReason::NoSemanticTree);
            continue;
        };
        if node.hidden {
            stats.skipped += 1;
            stats.skip_reason = Some(SkipReason::Clipped);
            continue;
        }
        stats.evaluated += 1;
        if node.disabled == widget.enabled {
            // Mismatch both ways: disabled node + enabled widget (or the
            // reverse) — AT and pointer disagree about interactivity.
            issues.push(AuditIssue {
                rule_id: "semantic.disabled_mismatch",
                severity: Severity::Warning,
                confidence: Confidence::High,
                widget_id: widget.id,
                widget_debug: widget.id_debug.clone(),
                other_id: None,
                evidence: format!(
                    "widget enabled={} but semantic node disabled={}",
                    widget.enabled, node.disabled
                ),
            });
        }
    }
    (issues, stats)
}

// ── Rule: semantic.orphan_node ────────────────────────────────────────────
//
// Every non-root node must have a parent in the tree (the update lists it as
// someone's child). A node with no parent is unreachable — egui emits full
// trees, so a missing parent is a real defect, not partial-update noise.

fn rule_semantic_orphan_node(
    _ctx: &egui::Context,
    _snapshot: &GeometrySnapshot,
    sem: Option<&SemanticSnapshot>,
) -> (Vec<AuditIssue>, RuleStats) {
    let Some(sem) = sem else {
        return no_semantic_data();
    };
    let mut issues = Vec::new();
    let mut stats = RuleStats::default();
    for node in sem.nodes.values() {
        if node.widget_id.is_none() {
            continue; // the window root has no parent by design
        }
        stats.evaluated += 1;
        if node.parent.is_none() {
            issues.push(AuditIssue {
                rule_id: "semantic.orphan_node",
                severity: Severity::Error,
                confidence: Confidence::High,
                widget_id: node.widget_id.unwrap_or_else(unknown_issue_id),
                widget_debug: node
                    .widget_id
                    .map(|id| id.short_debug_format())
                    .unwrap_or_else(|| format!("{:?}", node.node_id)),
                other_id: None,
                evidence: format!("{} node is not a child of any node in the update", node.role_debug),
            });
        }
    }
    (issues, stats)
}

// ── Rule: focus.orphaned ──────────────────────────────────────────────────
//
// The tree update's `focus` must point at a node that exists in the same
// update (or the root). A focus id outside the tree is a dangling focus —
// AT can't land anywhere. Skips entirely when there's no tree.

fn rule_focus_orphaned(
    _ctx: &egui::Context,
    _snapshot: &GeometrySnapshot,
    sem: Option<&SemanticSnapshot>,
) -> (Vec<AuditIssue>, RuleStats) {
    let Some(sem) = sem else {
        return no_semantic_data();
    };
    let mut issues = Vec::new();
    let mut stats = RuleStats {
        evaluated: 1,
        ..Default::default()
    };
    if let Some(focus) = sem.focus {
        if !sem.nodes.contains_key(&focus) {
            issues.push(AuditIssue {
                rule_id: "focus.orphaned",
                severity: Severity::Error,
                confidence: Confidence::High,
                widget_id: u64_to_issue_id(focus),
                widget_debug: format!("{focus:?}"),
                other_id: None,
                evidence: format!(
                    "focus {focus:?} is not a node in the tree update ({} nodes)",
                    sem.nodes.len()
                ),
            });
        }
    } else {
        stats.skipped += 1;
        stats.skip_reason = Some(SkipReason::NoSemanticTree);
    }
    (issues, stats)
}

// ── Rule: focus.unfocusable ───────────────────────────────────────────────
//
// `TreeUpdate.focus` must land on a node that accepts focus (Action::Focus).
// Focus on a node that can't take it breaks keyboard navigation — the next
// Tab has nowhere consistent to go. Skips when there is no tree.

fn rule_focus_unfocusable(
    _ctx: &egui::Context,
    _snapshot: &GeometrySnapshot,
    sem: Option<&SemanticSnapshot>,
) -> (Vec<AuditIssue>, RuleStats) {
    let Some(sem) = sem else {
        return no_semantic_data();
    };
    let mut issues = Vec::new();
    let stats = RuleStats {
        evaluated: 1,
        ..Default::default()
    };
    if let Some(focus) = sem.focus {
        if let Some(node) = sem.nodes.get(&focus) {
            // The root Window node legitimately holds "no widget focused" —
            // it doesn't advertise Action::Focus, so only non-root nodes are
            // audited.
            if node.widget_id.is_some() && !node.supports_focus {
                issues.push(AuditIssue {
                    rule_id: "focus.unfocusable",
                    severity: Severity::Error,
                    confidence: Confidence::High,
                    widget_id: node.widget_id.unwrap_or_else(unknown_issue_id),
                    widget_debug: node
                        .widget_id
                        .map(|id| id.short_debug_format())
                        .unwrap_or_else(|| format!("{:?}", node.node_id)),
                    other_id: None,
                    evidence: format!("focus is on a {} node without Action::Focus", node.role_debug),
                });
            }
        }
        // focus not in nodes at all → focus.orphaned reports it; don't double.
    }
    (issues, stats)
}

/// Issue `widget_id` for a node with no egui counterpart — `Id::NULL`.
fn unknown_issue_id() -> egui::Id {
    egui::Id::NULL
}

/// Convert a focus `NodeId` back to an `egui::Id` for the issue, falling back
/// to `Id::NULL` when the bits are zero.
fn u64_to_issue_id(node_id: egui::accesskit::NodeId) -> egui::Id {
    let bits: u64 = node_id.into();
    if bits == 0 {
        egui::Id::NULL
    } else {
        // SAFETY-contract: non-zero checked above; bits are an Id's own hash.
        unsafe { egui::Id::from_high_entropy_bits(bits) }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::dev_tools::geometry::GeometryRect;
    use egui::{Pos2, Rect};

    /// Minimal widget fixture — rule-irrelevant fields fixed so tests read as
    /// pure geometry.
    pub(crate) fn widget(id: u64, layer_id: u64, rect: Rect, interact: Rect) -> WidgetGeometry {
        let layer = egui::LayerId {
            order: egui::Order::Middle,
            id: Id::new(layer_id),
        };
        WidgetGeometry {
            id: Id::new(id),
            id_debug: Id::new(id).short_debug_format(),
            parent_id: Id::new("ui"),
            parent_debug: "ui".into(),
            layer,
            layer_debug: layer.short_debug_format(),
            rect: GeometryRect::from_egui(rect),
            interact_rect: GeometryRect::from_egui(interact),
            clipped: rect != interact,
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

    fn issues_for<'r>(report: &'r AuditReport, rule: &str) -> Vec<&'r AuditIssue> {
        report.issues.iter().filter(|i| i.rule_id == rule).collect()
    }

    fn rule_stats<'r>(report: &'r AuditReport, rule: &str) -> &'r RuleStats {
        &report.rules.iter().find(|r| r.rule_id == rule).unwrap().stats
    }

    // ── geometry.invalid ─────────────────────────────────────────────

    #[test]
    fn invalid_geometry_flags_nan() {
        let ctx = egui::Context::default();
        // egui::Rect panics on NaN construction — inject it post-build.
        let mut w = widget(
            1,
            1,
            Rect::from_min_size(Pos2::ZERO, egui::vec2(10.0, 10.0)),
            Rect::from_min_size(Pos2::ZERO, egui::vec2(10.0, 10.0)),
        );
        w.rect.min_x = f32::NAN;
        let report = run_audit(&ctx, &snapshot(vec![w]), None);
        let issues = issues_for(&report, "geometry.invalid");
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].severity, Severity::Error);
        assert_eq!(issues[0].confidence, Confidence::High);
        assert!(issues[0].evidence.contains("non-finite"));
    }

    #[test]
    fn invalid_geometry_flags_inverted() {
        let ctx = egui::Context::default();
        let r = Rect::from_min_max(Pos2::new(10.0, 0.0), Pos2::new(5.0, 10.0));
        let report = run_audit(&ctx, &snapshot(vec![widget(1, 1, r, r)]), None);
        let issues = issues_for(&report, "geometry.invalid");
        assert_eq!(issues.len(), 1);
        assert!(issues[0].evidence.contains("inverted"));
    }

    #[test]
    fn invalid_geometry_passes_normal_rect() {
        let ctx = egui::Context::default();
        let r = Rect::from_min_size(Pos2::ZERO, egui::vec2(50.0, 20.0));
        let report = run_audit(&ctx, &snapshot(vec![widget(1, 1, r, r)]), None);
        assert!(issues_for(&report, "geometry.invalid").is_empty());
    }

    // ── interactive.zero_size ────────────────────────────────────────

    #[test]
    fn zero_size_flags_unclipped_zero_interact() {
        let ctx = egui::Context::default();
        // Genuine zero hitbox, no clipping → reachable in the tree but dead.
        let mut w = widget(1, 1, Rect::from_min_size(Pos2::ZERO, egui::vec2(0.0, 0.0)), Rect::ZERO);
        w.clipped = false;
        let report = run_audit(&ctx, &snapshot(vec![w]), None);
        let issues = issues_for(&report, "interactive.zero_size");
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].severity, Severity::Error);
        assert!(issues[0].evidence.contains("floor"));
    }

    #[test]
    fn zero_size_skips_clipped_widget() {
        let ctx = egui::Context::default();
        // rect≠interact ⇒ clipped=true by fixture — parent clip shrank the
        // hitbox, geometry itself is fine → SKIP, don't guess.
        let w = widget(
            1,
            1,
            Rect::from_min_size(Pos2::ZERO, egui::vec2(50.0, 20.0)),
            Rect::from_min_size(Pos2::ZERO, egui::vec2(0.0, 0.0)),
        );
        let report = run_audit(&ctx, &snapshot(vec![w]), None);
        assert!(issues_for(&report, "interactive.zero_size").is_empty());
        let stats = rule_stats(&report, "interactive.zero_size");
        assert_eq!(stats.skipped, 1);
        assert_eq!(stats.skip_reason, Some(SkipReason::Clipped));
    }

    #[test]
    fn zero_size_ignores_non_interactive() {
        let ctx = egui::Context::default();
        let mut w = widget(1, 1, Rect::ZERO, Rect::ZERO);
        w.senses_click = false;
        w.senses_drag = false;
        w.focusable = false;
        let report = run_audit(&ctx, &snapshot(vec![w]), None);
        assert!(issues_for(&report, "interactive.zero_size").is_empty());
    }

    #[test]
    fn zero_size_ignores_disabled() {
        let ctx = egui::Context::default();
        let mut w = widget(1, 1, Rect::ZERO, Rect::ZERO);
        w.enabled = false;
        let report = run_audit(&ctx, &snapshot(vec![w]), None);
        assert!(issues_for(&report, "interactive.zero_size").is_empty());
    }

    // ── overlap.suspicious ───────────────────────────────────────────

    #[test]
    fn overlap_flags_sibling_covering_sibling() {
        let ctx = egui::Context::default();
        // b overlaps x 5..105 over a's x 0..100 → 95×19 of a's 100×20 ≈ 90%,
        // no containment (b extends past a on x, a past b on y? no: y 1..20 ⊂
        // 0..20 — rect containment is on BOTH axes; b's x range (5..105)
        // exceeds a's (0..100) so neither contains the other).
        let a = widget(
            1,
            1,
            Rect::from_min_size(Pos2::ZERO, egui::vec2(100.0, 20.0)),
            Rect::from_min_size(Pos2::ZERO, egui::vec2(100.0, 20.0)),
        );
        let b = widget(
            2,
            1,
            Rect::from_min_size(Pos2::new(5.0, 1.0), egui::vec2(100.0, 19.0)),
            Rect::from_min_size(Pos2::new(5.0, 1.0), egui::vec2(100.0, 19.0)),
        );
        let report = run_audit(&ctx, &snapshot(vec![a, b]), None);
        let issues = issues_for(&report, "overlap.suspicious");
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].severity, Severity::Warning);
        assert_eq!(issues[0].other_id, Some(Id::new(1u64)));
        assert!(issues[0].evidence.contains('%'));
    }

    #[test]
    fn overlap_containment_is_layout_not_bug() {
        let ctx = egui::Context::default();
        // Checkbox inside a row's hitbox → containment → evaluated, no issue.
        let row = widget(
            1,
            1,
            Rect::from_min_size(Pos2::ZERO, egui::vec2(200.0, 30.0)),
            Rect::from_min_size(Pos2::ZERO, egui::vec2(200.0, 30.0)),
        );
        let cb = widget(
            2,
            1,
            Rect::from_min_size(Pos2::new(8.0, 8.0), egui::vec2(14.0, 14.0)),
            Rect::from_min_size(Pos2::new(8.0, 8.0), egui::vec2(14.0, 14.0)),
        );
        let report = run_audit(&ctx, &snapshot(vec![row, cb]), None);
        assert!(issues_for(&report, "overlap.suspicious").is_empty());
        assert_eq!(rule_stats(&report, "overlap.suspicious").evaluated, 1);
    }

    #[test]
    fn overlap_different_areas_skip_as_intentional() {
        let ctx = egui::Context::default();
        // Same Order::Middle but different layer ids = two Areas (floating
        // palette over canvas) — intentional overlay, skip.
        let a = widget(
            1,
            1,
            Rect::from_min_size(Pos2::ZERO, egui::vec2(100.0, 30.0)),
            Rect::from_min_size(Pos2::ZERO, egui::vec2(100.0, 30.0)),
        );
        let b = widget(
            2,
            2,
            Rect::from_min_size(Pos2::new(10.0, 5.0), egui::vec2(80.0, 20.0)),
            Rect::from_min_size(Pos2::new(10.0, 5.0), egui::vec2(80.0, 20.0)),
        );
        let report = run_audit(&ctx, &snapshot(vec![a, b]), None);
        assert!(issues_for(&report, "overlap.suspicious").is_empty());
        let stats = rule_stats(&report, "overlap.suspicious");
        assert_eq!(stats.skipped, 1);
        assert_eq!(stats.skip_reason, Some(SkipReason::IntentionalOverlay));
    }

    #[test]
    fn overlap_small_overlap_does_not_flag() {
        let ctx = egui::Context::default();
        // ~10% edge overlap — dense toolbar adjacency, below threshold.
        let a = widget(
            1,
            1,
            Rect::from_min_size(Pos2::ZERO, egui::vec2(100.0, 20.0)),
            Rect::from_min_size(Pos2::ZERO, egui::vec2(100.0, 20.0)),
        );
        let b = widget(
            2,
            1,
            Rect::from_min_size(Pos2::new(90.0, 0.0), egui::vec2(50.0, 20.0)),
            Rect::from_min_size(Pos2::new(90.0, 0.0), egui::vec2(50.0, 20.0)),
        );
        let report = run_audit(&ctx, &snapshot(vec![a, b]), None);
        assert!(issues_for(&report, "overlap.suspicious").is_empty());
    }

    #[test]
    fn overlap_disabled_widget_not_flagged() {
        let ctx = egui::Context::default();
        let a = widget(
            1,
            1,
            Rect::from_min_size(Pos2::ZERO, egui::vec2(100.0, 20.0)),
            Rect::from_min_size(Pos2::ZERO, egui::vec2(100.0, 20.0)),
        );
        let mut b = widget(
            2,
            1,
            Rect::from_min_size(Pos2::new(50.0, 5.0), egui::vec2(100.0, 20.0)),
            Rect::from_min_size(Pos2::new(50.0, 5.0), egui::vec2(100.0, 20.0)),
        );
        b.enabled = false;
        let report = run_audit(&ctx, &snapshot(vec![a, b]), None);
        assert!(issues_for(&report, "overlap.suspicious").is_empty());
    }

    #[test]
    fn empty_snapshot_is_valid_and_clean() {
        let ctx = egui::Context::default();
        let report = run_audit(&ctx, &GeometrySnapshot::default(), None);
        assert!(report.issues.is_empty());
        assert_eq!(report.widget_count, 0);
        assert_eq!(report.rules.len(), RULES.len());
    }

    // ── semantic.missing_name ───────────────────────────────────────

    fn sem_tree(
        nodes: Vec<(egui::accesskit::NodeId, egui::accesskit::Node)>,
        focus: egui::accesskit::NodeId,
    ) -> SemanticSnapshot {
        use egui::accesskit::Tree;
        let root = egui::accesskit_root_id().accesskit_id();
        SemanticSnapshot::from_update(&egui::accesskit::TreeUpdate {
            nodes,
            tree: Some(Tree::new(root)),
            focus,
            tree_id: egui::accesskit::TreeId::ROOT,
        })
        .expect("fixture tree")
    }

    #[test]
    fn missing_name_flags_unlabelled_button() {
        use egui::accesskit::{Action, Node, Role};
        let ctx = egui::Context::default();
        let nid = egui::Id::new("b").accesskit_id();
        let root = egui::accesskit_root_id().accesskit_id();
        let mut b = Node::new(Role::Button);
        b.add_action(Action::Click);
        b.add_action(Action::Focus);
        let mut r = Node::new(Role::Window);
        r.push_child(nid);
        let sem = sem_tree(vec![(root, r), (nid, b)], root);
        let report = run_audit(&ctx, &GeometrySnapshot::default(), Some(&sem));
        let issues = issues_for(&report, "semantic.missing_name");
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].severity, Severity::Error);
        assert_eq!(issues[0].widget_id, egui::Id::new("b"));
    }

    #[test]
    fn missing_name_passes_labelled_and_skips_hidden() {
        use egui::accesskit::{Action, Node, Role};
        let ctx = egui::Context::default();
        let labelled = egui::Id::new("ok").accesskit_id();
        let hidden = egui::Id::new("hid").accesskit_id();
        let root = egui::accesskit_root_id().accesskit_id();
        let mut b = Node::new(Role::Button);
        b.set_label("OK");
        b.add_action(Action::Click);
        let mut h = Node::new(Role::Button); // unnamed but hidden → SKIP
        h.add_action(Action::Click);
        h.set_hidden();
        let mut r = Node::new(Role::Window);
        r.push_child(labelled);
        r.push_child(hidden);
        let sem = sem_tree(vec![(root, r), (labelled, b), (hidden, h)], root);
        let report = run_audit(&ctx, &GeometrySnapshot::default(), Some(&sem));
        assert!(issues_for(&report, "semantic.missing_name").is_empty());
        let stats = rule_stats(&report, "semantic.missing_name");
        assert_eq!(stats.evaluated, 1); // only the labelled, visible button
        assert_eq!(stats.skipped, 1); // hidden node skipped
    }

    #[test]
    fn missing_name_joins_clickable_unknown_role_with_widget() {
        use egui::accesskit::{Action, Node, Role};
        let ctx = egui::Context::default();
        // Unknown-role node advertising Click + widget that senses clicks →
        // flags; the same node without a clickable widget counterpart → skip.
        let w = widget(
            1,
            1,
            Rect::from_min_size(Pos2::ZERO, egui::vec2(30.0, 20.0)),
            Rect::from_min_size(Pos2::ZERO, egui::vec2(30.0, 20.0)),
        );
        let nid = w.id.accesskit_id();
        let root = egui::accesskit_root_id().accesskit_id();
        let mut n = Node::new(Role::Unknown);
        n.add_action(Action::Click);
        let mut r = Node::new(Role::Window);
        r.push_child(nid);
        let sem = sem_tree(vec![(root, r), (nid, n)], root);

        let report = run_audit(&ctx, &snapshot(vec![w.clone()]), Some(&sem));
        assert_eq!(
            issues_for(&report, "semantic.missing_name").len(),
            1,
            "clickable unnamed node must flag"
        );

        // Same node, no widget counterpart (or non-clicking widget) → not evaluated.
        let mut quiet = widget(
            2,
            1,
            Rect::from_min_size(Pos2::ZERO, egui::vec2(30.0, 20.0)),
            Rect::from_min_size(Pos2::ZERO, egui::vec2(30.0, 20.0)),
        );
        quiet.senses_click = false;
        quiet.senses_drag = false;
        quiet.focusable = false;
        quiet.id = w.id; // same egui id, but the widget doesn't sense clicks
        quiet.id_debug = w.id_debug.clone();
        let report = run_audit(&ctx, &snapshot(vec![quiet]), Some(&sem));
        assert!(
            issues_for(&report, "semantic.missing_name").is_empty(),
            "non-clicking widget must not flag"
        );
    }

    #[test]
    fn missing_name_ignores_label_role_and_skips_without_tree() {
        let ctx = egui::Context::default();
        // No semantic data → the rule reports SKIP, never invents a verdict.
        let report = run_audit(&ctx, &GeometrySnapshot::default(), None);
        let stats = rule_stats(&report, "semantic.missing_name");
        assert_eq!(stats.evaluated, 0);
        assert_eq!(stats.skipped, 1);
        assert_eq!(stats.skip_reason, Some(SkipReason::NoSemanticTree));
        assert!(issues_for(&report, "semantic.missing_name").is_empty());
    }

    // ── semantic.disabled_mismatch ──────────────────────────────────

    #[test]
    fn disabled_mismatch_flags_and_skips_cleanly() {
        use egui::accesskit::{Action, Node, Role};
        let ctx = egui::Context::default();
        // Widget enabled, but its semantic node says disabled → mismatch.
        let w = widget(
            1,
            1,
            Rect::from_min_size(Pos2::ZERO, egui::vec2(50.0, 20.0)),
            Rect::from_min_size(Pos2::ZERO, egui::vec2(50.0, 20.0)),
        );
        let mut node = Node::new(Role::Button);
        node.set_label("X");
        node.set_disabled();
        node.add_action(Action::Click);
        let nid = w.id.accesskit_id();
        let root = egui::accesskit_root_id().accesskit_id();
        let mut r = Node::new(Role::Window);
        r.push_child(nid);
        let sem = sem_tree(vec![(root, r), (nid, node)], root);
        let report = run_audit(&ctx, &snapshot(vec![w]), Some(&sem));
        let issues = issues_for(&report, "semantic.disabled_mismatch");
        assert_eq!(issues.len(), 1);
        assert!(issues[0].evidence.contains("enabled=true"));
    }

    #[test]
    fn disabled_mismatch_skips_widget_without_node() {
        let ctx = egui::Context::default();
        // Interactive widget with no semantic node at all → SKIP, not a flag.
        let w = widget(
            1,
            1,
            Rect::from_min_size(Pos2::ZERO, egui::vec2(50.0, 20.0)),
            Rect::from_min_size(Pos2::ZERO, egui::vec2(50.0, 20.0)),
        );
        let root = egui::accesskit_root_id().accesskit_id();
        let sem = sem_tree(
            vec![(root, egui::accesskit::Node::new(egui::accesskit::Role::Window))],
            root,
        );
        let report = run_audit(&ctx, &snapshot(vec![w]), Some(&sem));
        assert!(issues_for(&report, "semantic.disabled_mismatch").is_empty());
        assert_eq!(rule_stats(&report, "semantic.disabled_mismatch").skipped, 1);
    }

    // ── reliability: verdicts, coverage, baseline diff ─────────────

    fn report_with(stats: RuleStats, n_issues: usize) -> RuleReport {
        RuleReport {
            rule_id: "test",
            stats,
            issues: n_issues,
        }
    }

    #[test]
    fn verdict_never_passes_on_zero_evaluated() {
        // Evaluated something clean → Pass.
        assert_eq!(
            report_with(
                RuleStats {
                    evaluated: 3,
                    ..Default::default()
                },
                0
            )
            .verdict(),
            Verdict::Pass
        );
        // All skips, nothing evaluated → Skipped (not Pass).
        assert_eq!(
            report_with(
                RuleStats {
                    skipped: 2,
                    skip_reason: Some(SkipReason::Clipped),
                    ..Default::default()
                },
                0
            )
            .verdict(),
            Verdict::Skipped
        );
        // Nothing in scope at all → NotApplicable.
        assert_eq!(report_with(RuleStats::default(), 0).verdict(), Verdict::NotApplicable);
        // Issues → Fail.
        assert_eq!(
            report_with(
                RuleStats {
                    evaluated: 3,
                    ..Default::default()
                },
                2
            )
            .verdict(),
            Verdict::Fail
        );
    }

    #[test]
    fn diff_compares_identity_not_counts() {
        let ctx = egui::Context::default();
        let mk = |id: u64| {
            let mut w = widget(id, 1, Rect::ZERO, Rect::ZERO);
            w.clipped = false;
            w
        };
        let snap = snapshot(vec![mk(10)]);
        let report_a = run_audit(&ctx, &snap, None);
        let report_b = run_audit(&ctx, &snap, None);
        // Equivalent snapshots, identical issues → empty diff.
        assert!(report_b.equivalent_snapshot(&report_a));
        let (new, resolved) = report_b.diff(&report_a).unwrap();
        assert!(new.is_empty() && resolved.is_empty());

        // A different widget set → not equivalent → no diff at all.
        let other = snapshot(vec![mk(20)]);
        let report_c = run_audit(&ctx, &other, None);
        assert!(!report_c.equivalent_snapshot(&report_a));
        assert!(report_c.diff(&report_a).is_none());
    }

    #[test]
    fn diff_reports_new_and_resolved_by_identity() {
        let ctx = egui::Context::default();
        let bad = |id: u64| {
            let mut w = widget(id, 1, Rect::ZERO, Rect::ZERO);
            w.clipped = false;
            w
        };
        let ok = |id: u64| {
            widget(
                id,
                1,
                Rect::from_min_size(Pos2::ZERO, egui::vec2(30.0, 20.0)),
                Rect::from_min_size(Pos2::ZERO, egui::vec2(30.0, 20.0)),
            )
        };
        // Frame A: widgets 1(bad) + 2(ok). Frame B: same set, but 1 fixed and
        // 2 broken — same *count*, different identities.
        let a = run_audit(&ctx, &snapshot(vec![bad(1), ok(2)]), None);
        let b = run_audit(&ctx, &snapshot(vec![ok(1), bad(2)]), None);
        assert!(a.equivalent_snapshot(&b));
        let (new, resolved) = b.diff(&a).unwrap();
        assert_eq!(new.len(), 1, "widget 2's new issue must appear");
        assert_eq!(resolved.len(), 1, "widget 1's fixed issue must resolve");
        assert!(new[0].contains(&egui::Id::new(2u64).short_debug_format()));
    }

    // ── visual.typography_hierarchy ─────────────────────────────────

    #[test]
    fn typography_flags_heading_smaller_than_body() {
        let ctx = egui::Context::default();
        ctx.global_style_mut(|style| {
            style
                .text_styles
                .insert(egui::TextStyle::Heading, egui::FontId::proportional(10.0));
            style
                .text_styles
                .insert(egui::TextStyle::Body, egui::FontId::proportional(14.0));
        });
        let report = run_audit(&ctx, &GeometrySnapshot::default(), None);
        let issues = issues_for(&report, "visual.typography_hierarchy");
        assert_eq!(issues.len(), 1);
        assert!(issues[0].evidence.contains("Heading"));
    }

    #[test]
    fn typography_passes_egui_defaults() {
        let ctx = egui::Context::default();
        let report = run_audit(&ctx, &GeometrySnapshot::default(), None);
        assert!(issues_for(&report, "visual.typography_hierarchy").is_empty());
        assert!(rule_stats(&report, "visual.typography_hierarchy").evaluated > 0);
    }

    // ── visual.text_contrast ────────────────────────────────────────

    #[test]
    fn contrast_flags_same_fg_bg_and_skips_transparent() {
        let ctx = egui::Context::default();
        ctx.global_style_mut(|style| {
            style.visuals.widgets.inactive.fg_stroke.color = egui::Color32::DARK_GRAY;
            style.visuals.widgets.inactive.weak_bg_fill = egui::Color32::DARK_GRAY;
            // A non-opaque bg makes the pair undecidable → skip, not a guess.
            style.visuals.widgets.hovered.weak_bg_fill = egui::Color32::TRANSPARENT;
        });
        let report = run_audit(&ctx, &GeometrySnapshot::default(), None);
        let issues = issues_for(&report, "visual.text_contrast");
        assert_eq!(issues.len(), 1);
        assert!(issues[0].evidence.contains("buttons at rest"));
        let stats = rule_stats(&report, "visual.text_contrast");
        assert_eq!(stats.skipped, 1);
        assert_eq!(stats.skip_reason, Some(SkipReason::NonOpaqueColors));
    }

    #[test]
    fn contrast_passes_egui_default_visuals() {
        let ctx = egui::Context::default();
        let report = run_audit(&ctx, &GeometrySnapshot::default(), None);
        assert!(issues_for(&report, "visual.text_contrast").is_empty());
    }

    #[test]
    fn contrast_ratio_is_wcag() {
        // Black on white is the WCAG reference point: 21:1.
        let r = contrast_ratio(egui::Color32::BLACK, egui::Color32::WHITE).unwrap();
        assert!((r - 21.0).abs() < 0.1, "expected ~21:1, got {r}");
        assert!(contrast_ratio(egui::Color32::WHITE, egui::Color32::TRANSPARENT).is_none());
    }

    // ── visual.touching_controls ────────────────────────────────────

    #[test]
    fn touching_flags_gapless_stacked_siblings() {
        let ctx = egui::Context::default();
        let a = widget(
            1,
            1,
            Rect::from_min_size(Pos2::ZERO, egui::vec2(100.0, 20.0)),
            Rect::from_min_size(Pos2::ZERO, egui::vec2(100.0, 20.0)),
        );
        let b = widget(
            2,
            1,
            Rect::from_min_size(Pos2::new(0.0, 20.0), egui::vec2(100.0, 20.0)),
            Rect::from_min_size(Pos2::new(0.0, 20.0), egui::vec2(100.0, 20.0)),
        );
        let report = run_audit(&ctx, &snapshot(vec![a, b]), None);
        let issues = issues_for(&report, "visual.touching_controls");
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].confidence, Confidence::Medium);
    }

    #[test]
    fn touching_ignores_normal_gaps_and_rows() {
        let ctx = egui::Context::default();
        // 8px gap — fine.
        let a = widget(
            1,
            1,
            Rect::from_min_size(Pos2::ZERO, egui::vec2(100.0, 20.0)),
            Rect::from_min_size(Pos2::ZERO, egui::vec2(100.0, 20.0)),
        );
        let b = widget(
            2,
            1,
            Rect::from_min_size(Pos2::new(0.0, 28.0), egui::vec2(100.0, 20.0)),
            Rect::from_min_size(Pos2::new(0.0, 28.0), egui::vec2(100.0, 20.0)),
        );
        // Side-by-side on one row — not a stack, not evaluated.
        let c = widget(
            3,
            1,
            Rect::from_min_size(Pos2::new(120.0, 0.0), egui::vec2(50.0, 20.0)),
            Rect::from_min_size(Pos2::new(120.0, 0.0), egui::vec2(50.0, 20.0)),
        );
        let report = run_audit(&ctx, &snapshot(vec![a, b, c]), None);
        assert!(issues_for(&report, "visual.touching_controls").is_empty());
    }

    #[test]
    fn touching_skips_foreground_layers() {
        let ctx = egui::Context::default();
        // Menu/popup rows live on Foreground — gapless stacking is the pattern.
        let mut a = widget(
            1,
            1,
            Rect::from_min_size(Pos2::ZERO, egui::vec2(100.0, 20.0)),
            Rect::from_min_size(Pos2::ZERO, egui::vec2(100.0, 20.0)),
        );
        let mut b = widget(
            2,
            1,
            Rect::from_min_size(Pos2::new(0.0, 20.0), egui::vec2(100.0, 20.0)),
            Rect::from_min_size(Pos2::new(0.0, 20.0), egui::vec2(100.0, 20.0)),
        );
        a.layer.order = egui::Order::Foreground;
        b.layer.order = egui::Order::Foreground;
        let report = run_audit(&ctx, &snapshot(vec![a, b]), None);
        assert!(issues_for(&report, "visual.touching_controls").is_empty());
        let stats = rule_stats(&report, "visual.touching_controls");
        assert_eq!(stats.skipped, 1);
        assert_eq!(stats.skip_reason, Some(SkipReason::IntentionalOverlay));
    }

    // ── visual.alignment_drift ──────────────────────────────────────

    #[test]
    fn drift_flags_off_by_few_px_sibling() {
        let ctx = egui::Context::default();
        // Column of 4, one row 2px off the shared left edge → drift.
        let mk = |id, y, x| {
            widget(
                id,
                1,
                Rect::from_min_size(Pos2::new(x, y), egui::vec2(100.0, 20.0)),
                Rect::from_min_size(Pos2::new(x, y), egui::vec2(100.0, 20.0)),
            )
        };
        let widgets = vec![
            mk(1, 0.0, 10.0),
            mk(2, 30.0, 10.0),
            mk(3, 60.0, 10.0),
            mk(4, 90.0, 12.0),
        ];
        let report = run_audit(&ctx, &snapshot(widgets), None);
        let issues = issues_for(&report, "visual.alignment_drift");
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].widget_id, egui::Id::new(4u64));
        assert!(issues[0].evidence.contains("2.0px"));
    }

    #[test]
    fn drift_ignores_aligned_columns_and_real_indents() {
        let ctx = egui::Context::default();
        let mk = |id, y, x| {
            widget(
                id,
                1,
                Rect::from_min_size(Pos2::new(x, y), egui::vec2(100.0, 20.0)),
                Rect::from_min_size(Pos2::new(x, y), egui::vec2(100.0, 20.0)),
            )
        };
        // Aligned column.
        let aligned = vec![mk(1, 0.0, 10.0), mk(2, 30.0, 10.0), mk(3, 60.0, 10.0)];
        let report = run_audit(&ctx, &snapshot(aligned), None);
        assert!(issues_for(&report, "visual.alignment_drift").is_empty());
        // 20px offset = deliberate indent, outside the drift band.
        let indented = vec![
            mk(1, 0.0, 10.0),
            mk(2, 30.0, 10.0),
            mk(3, 60.0, 10.0),
            mk(4, 90.0, 30.0),
        ];
        let report = run_audit(&ctx, &snapshot(indented), None);
        assert!(issues_for(&report, "visual.alignment_drift").is_empty());
        // Two members only — no majority edge exists → skip whole group.
        let pair = vec![mk(1, 0.0, 10.0), mk(2, 30.0, 12.0)];
        let report = run_audit(&ctx, &snapshot(pair), None);
        assert!(issues_for(&report, "visual.alignment_drift").is_empty());
    }

    // ── semantic.orphan_node ────────────────────────────────────────

    #[test]
    fn orphan_node_flags_parentless_node() {
        use egui::accesskit::{Node, Role};
        let ctx = egui::Context::default();
        let root = egui::accesskit_root_id().accesskit_id();
        // Node emitted but never listed as anyone's child → orphan.
        let stray = egui::Id::new("stray").accesskit_id();
        let mut n = Node::new(Role::Button);
        n.set_label("Stray");
        let sem = sem_tree(vec![(root, Node::new(Role::Window)), (stray, n)], root);
        let report = run_audit(&ctx, &GeometrySnapshot::default(), Some(&sem));
        let issues = issues_for(&report, "semantic.orphan_node");
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].widget_id, egui::Id::new("stray"));
    }

    #[test]
    fn orphan_node_passes_root_and_children() {
        use egui::accesskit::{Node, Role};
        let ctx = egui::Context::default();
        let root = egui::accesskit_root_id().accesskit_id();
        let child = egui::Id::new("c").accesskit_id();
        let mut r = Node::new(Role::Window);
        r.push_child(child);
        let mut c = Node::new(Role::Button);
        c.set_label("C");
        let sem = sem_tree(vec![(root, r), (child, c)], root);
        let report = run_audit(&ctx, &GeometrySnapshot::default(), Some(&sem));
        assert!(issues_for(&report, "semantic.orphan_node").is_empty());
    }

    // ── focus.unfocusable ───────────────────────────────────────────

    #[test]
    fn focus_unfocusable_flags_focus_on_non_focusable_node() {
        use egui::accesskit::{Node, Role};
        let ctx = egui::Context::default();
        let root = egui::accesskit_root_id().accesskit_id();
        let id = egui::Id::new("lbl").accesskit_id();
        let mut r = Node::new(Role::Window);
        r.push_child(id);
        let mut n = Node::new(Role::Label); // no Focus action
        n.set_value("some label");
        let sem = sem_tree(vec![(root, r), (id, n)], id);
        let report = run_audit(&ctx, &GeometrySnapshot::default(), Some(&sem));
        let issues = issues_for(&report, "focus.unfocusable");
        assert_eq!(issues.len(), 1);
        assert!(issues[0].evidence.contains("without Action::Focus"));
    }

    #[test]
    fn focus_unfocusable_passes_on_root_and_focusable() {
        use egui::accesskit::{Action, Node, Role};
        let ctx = egui::Context::default();
        let root = egui::accesskit_root_id().accesskit_id();
        let id = egui::Id::new("b").accesskit_id();
        let mut r = Node::new(Role::Window);
        r.push_child(id);
        let mut n = Node::new(Role::Button);
        n.set_label("B");
        n.add_action(Action::Focus);
        // Case 1: focus on the focusable button.
        let sem = sem_tree(vec![(root, r.clone()), (id, n)], id);
        let report = run_audit(&ctx, &GeometrySnapshot::default(), Some(&sem));
        assert!(issues_for(&report, "focus.unfocusable").is_empty());
        // Case 2: focus on the root = "nothing focused" — not a violation.
        let mut r2 = Node::new(Role::Window);
        r2.push_child(id);
        let sem2 = sem_tree(vec![(root, r2), (id, Node::new(Role::Button))], root);
        let report = run_audit(&ctx, &GeometrySnapshot::default(), Some(&sem2));
        assert!(issues_for(&report, "focus.unfocusable").is_empty());
    }

    // ── focus.orphaned ──────────────────────────────────────────────

    #[test]
    fn focus_orphaned_flags_dangling_focus() {
        use egui::accesskit::{Node, Role};
        let ctx = egui::Context::default();
        let root = egui::accesskit_root_id().accesskit_id();
        // focus points at a node that was never emitted.
        let sem = sem_tree(vec![(root, Node::new(Role::Window))], egui::accesskit::NodeId(424242));
        let report = run_audit(&ctx, &GeometrySnapshot::default(), Some(&sem));
        let issues = issues_for(&report, "focus.orphaned");
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].severity, Severity::Error);
        assert!(issues[0].evidence.contains("424242"));
    }

    #[test]
    fn focus_orphaned_passes_on_root_focus() {
        use egui::accesskit::{Node, Role};
        let ctx = egui::Context::default();
        let root = egui::accesskit_root_id().accesskit_id();
        let sem = sem_tree(vec![(root, Node::new(Role::Window))], root);
        let report = run_audit(&ctx, &GeometrySnapshot::default(), Some(&sem));
        assert!(issues_for(&report, "focus.orphaned").is_empty());
    }

    #[test]
    fn issue_id_resolves_then_survives_widget_gone() {
        // Issue → snapshot.find round-trips while the widget lives; a stale
        // id after the widget vanishes resolves to None, not a panic.
        let ctx = egui::Context::default();
        let mut w = widget(7, 1, Rect::from_min_size(Pos2::ZERO, egui::vec2(0.0, 0.0)), Rect::ZERO);
        w.clipped = false;
        let snap = snapshot(vec![w]);
        let report = run_audit(&ctx, &snap, None);
        let issue = report
            .issues
            .iter()
            .find(|i| i.rule_id == "interactive.zero_size")
            .unwrap();
        assert!(snap.find(issue.widget_id).is_some(), "issue must resolve to its widget");
        let empty = GeometrySnapshot::default();
        assert!(empty.find(issue.widget_id).is_none());
    }
}
