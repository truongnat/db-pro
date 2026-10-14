//! Headless audit report: serializes `AuditReport` + environment metadata to a
//! versioned JSON schema for CI / script consumption (`DB_PRO_AUDIT_JSON`).
//!
//! Schema v1 keeps every verdict explicit — `skipped`/`not_applicable` are
//! distinct from `pass`, and the exit code never treats them as pass.

use super::audit::{AuditIssue, AuditReport, Severity, Verdict};

/// Bump when the report shape changes; baseline loaders check it first.
/// v4 adds `visual_review` (P11): provider-run AI review of the bundle —
/// absent or `status: bundle_only|error` offline; never gates the exit code.
/// v3 adds the `screen` section (P10): measured screen metrics plus a
/// separate finding list (screen findings never affect the exit code).
/// v2 adds `scenario` to meta/signature and `layout_fingerprint` to meta.
pub const SCHEMA_VERSION: u32 = 4;

/// Output path for the headless report (`db-pro-native` exits after writing).
pub const OUTPUT_ENV: &str = "DB_PRO_AUDIT_JSON";
/// Optional baseline report for the signature-gated diff section.
pub const BASELINE_ENV: &str = "DB_PRO_AUDIT_BASELINE";
/// Names the audited UI scenario (e.g. "welcome", "table-indexes") — part of
/// the signature so baselines only diff identical scenarios.
pub const SCENARIO_ENV: &str = "DB_PRO_AUDIT_SCENARIO";
/// Optional readiness gate: `DB_PRO_AUDIT_READY=<n>` requires the audited
/// snapshot to hold ≥ n widgets before emitting — the deterministic condition
/// per scenario instead of fingerprint-stability alone.
pub const READY_ENV: &str = "DB_PRO_AUDIT_READY";

/// Process exit codes for the headless audit mode.
pub const EXIT_OK: i32 = 0;
/// Any rule verdict Fail, or any Error-severity issue.
pub const EXIT_AUDIT_FAILED: i32 = 1;
/// Tool error: unwritable output, unreadable baseline, timeout, etc.
pub const EXIT_TOOL_ERROR: i32 = 2;

/// Everything a baseline comparison needs to decide "same snapshot, same
/// conditions". Two reports only diff when their signatures agree.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Signature {
    pub schema_version: u32,
    /// Scenario identifier — audits are only comparable within a scenario.
    pub scenario: String,
    pub fixture: bool,
    pub viewport: String,
    pub theme: String,
    /// Sorted widget-id values — the snapshot's identity.
    pub widget_ids: Vec<u64>,
}

/// Stable issue identity: rule + subject widget + context widget. Stable
/// across runs of the same UI, independent of issue order.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct IssueKey {
    pub rule: String,
    pub widget: String,
    pub other: String,
}

impl IssueKey {
    pub fn of(issue: &AuditIssue) -> Self {
        Self {
            rule: issue.rule_id.to_owned(),
            widget: issue.widget_debug.clone(),
            other: issue.other_id.map(|id| id.short_debug_format()).unwrap_or_default(),
        }
    }

    fn line(&self) -> String {
        format!("{}|{}|{}", self.rule, self.widget, self.other)
    }
}

/// Inputs the caller knows but the report itself doesn't (viewport comes from
/// the capture env; fixture/theme flags from process env).
#[derive(Clone, Debug)]
pub struct ReportContext {
    pub viewport: String,
    pub fixture: bool,
    pub frames: u32,
    /// Scenario name for meta + signature (`DB_PRO_AUDIT_SCENARIO`).
    pub scenario: String,
    /// Product theme label — `egui::Theme` only tracks egui's light/dark
    /// preference, not `DbProTheme::light()/dark()`.
    pub theme_label: String,
}

fn severity_str(s: Severity) -> &'static str {
    match s {
        Severity::Error => "error",
        Severity::Warning => "warning",
    }
}

fn confidence_str(c: super::audit::Confidence) -> &'static str {
    match c {
        super::audit::Confidence::High => "high",
        super::audit::Confidence::Medium => "medium",
    }
}

fn verdict_str(v: Verdict) -> &'static str {
    match v {
        Verdict::Pass => "pass",
        Verdict::Fail => "fail",
        Verdict::Skipped => "skip",
        Verdict::NotApplicable => "not_applicable",
    }
}

/// Render an `AuditReport` to schema-v1 JSON. Pure function — no IO, so unit
/// tests exercise the whole shape.
pub fn render_report(report: &AuditReport, _ctx: &egui::Context, rc: &ReportContext) -> serde_json::Value {
    serde_json::json!({
        "schema_version": SCHEMA_VERSION,
        "meta": report_meta(report, rc),
        "signature": report_signature(report, rc),
        "summary": report_summary(report),
        "rules": report_rules_json(report),
        "issues": report_issues_json(report),
        "screen": report.screen.as_ref().map(render_screen),
    })
}

fn report_signature(report: &AuditReport, rc: &ReportContext) -> Signature {
    let mut widget_ids: Vec<u64> = report.widget_ids.iter().copied().collect();
    widget_ids.sort_unstable();
    Signature {
        schema_version: SCHEMA_VERSION,
        scenario: rc.scenario.clone(),
        fixture: rc.fixture,
        viewport: rc.viewport.clone(),
        theme: rc.theme_label.clone(),
        widget_ids,
    }
}

fn report_rules_json(report: &AuditReport) -> Vec<serde_json::Value> {
    report
        .rules
        .iter()
        .map(|r| {
            serde_json::json!({
                "id": r.rule_id,
                "verdict": verdict_str(r.verdict()),
                "issues": r.issues,
                "coverage": {
                    "evaluated": r.stats.evaluated,
                    "skipped": r.stats.skipped,
                    "skip_reason": r.stats.skip_reason.map(|s| s.label()),
                    "not_applicable": r.stats.not_applicable,
                },
            })
        })
        .collect()
}

fn report_issues_json(report: &AuditReport) -> Vec<serde_json::Value> {
    report
        .issues
        .iter()
        .map(|i| {
            serde_json::json!({
                "key": IssueKey::of(i).line(),
                "rule": i.rule_id,
                "severity": severity_str(i.severity),
                "confidence": confidence_str(i.confidence),
                "widget": i.widget_debug,
                "other": i.other_id.map(|o| o.short_debug_format()),
                "evidence": i.evidence,
            })
        })
        .collect()
}

fn report_meta(report: &AuditReport, rc: &ReportContext) -> serde_json::Value {
    serde_json::json!({
        "tool": "db-pro-ui-audit",
        "binary": "db-pro-native",
        "viewport": rc.viewport,
        "theme": rc.theme_label,
        "scenario": rc.scenario,
        "fixture": rc.fixture,
        "frames": rc.frames,
        "layout_fingerprint": report.layout_fingerprint,
        "fail_policy": FailPolicy::from_env().label(),
        "os": std::env::consts::OS,
        "timestamp": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0),
    })
}

fn report_summary(report: &AuditReport) -> serde_json::Value {
    let verdicts = |v: Verdict| report.rules.iter().filter(|r| r.verdict() == v).count();
    serde_json::json!({
        "widgets": report.widget_count,
        "rules": report.rules.len(),
        "issues": report.issues.len(),
        "errors": report.issues.iter().filter(|i| i.severity == Severity::Error).count(),
        "warnings": report.issues.iter().filter(|i| i.severity == Severity::Warning).count(),
        "verdicts": {
            "pass": verdicts(Verdict::Pass),
            "fail": verdicts(Verdict::Fail),
            "skip": verdicts(Verdict::Skipped),
            "not_applicable": verdicts(Verdict::NotApplicable),
        },
    })
}

/// Serialize the screen-level analysis: every measured metric plus the
/// finding list. `null` fields mean UNKNOWN — the analyzer had no evidence,
/// it did not measure zero. Kept out of `issues`/`summary` by design:
/// screen findings inform, they don't gate the exit code.
fn render_screen(analysis: &crate::dev_tools::screen::ScreenAnalysis) -> serde_json::Value {
    use crate::dev_tools::screen::ScreenAnalysis as A;
    let A {
        sufficient,
        regions,
        rhythm,
        density,
        balance,
        hierarchy,
        findings,
        gaps,
    } = analysis;
    serde_json::json!({
        "sufficient": sufficient,
        "coverage": {
            "gaps": gaps.iter().map(|g| g.label()).collect::<Vec<_>>(),
            "semantic_available": hierarchy.nodes.is_some(),
        },
        "metrics": {
            "regions": regions.iter().map(|r| serde_json::json!({
                "kind": r.kind.label(),
                "rect": {
                    "x": r.rect.min_x, "y": r.rect.min_y,
                    "w": r.rect.width(), "h": r.rect.height(),
                },
                "widgets": r.widgets,
                "interactive": r.interactive,
                "grid_like": r.grid_like,
                "floating": r.floating,
                "screen_share": r.screen_share,
                "fill": r.fill,
            })).collect::<Vec<_>>(),
            "rhythm": {
                "gaps_measured": rhythm.gaps_measured,
                "mode_gap": rhythm.mode_gap,
                "mode_share": rhythm.mode_share,
                "outliers": rhythm.outliers,
            },
            "density": {
                "widgets": density.widgets,
                "interactive": density.interactive,
                "widget_coverage": density.widget_coverage,
                "interactive_coverage": density.interactive_coverage,
            },
            "balance": {
                "interactive_area_share": {
                    "left_top": balance.left_top,
                    "right_top": balance.right_top,
                    "left_bottom": balance.left_bottom,
                    "right_bottom": balance.right_bottom,
                },
            },
            "hierarchy": {
                "max_depth": hierarchy.max_depth,
                "container_nodes": hierarchy.containers,
                "nodes": hierarchy.nodes,
            },
        },
        "findings": findings.iter().map(|f| serde_json::json!({
            "rule": f.rule,
            "kind": f.kind.label(),
            "severity": severity_str(f.severity),
            "confidence": confidence_str(f.confidence),
            "widget": f.widget_debug,
            "widget_id": f.widget_id.map(|id| id.value()),
            "region": f.region,
            "evidence": f.evidence,
        })).collect::<Vec<_>>(),
    })
}

/// What the baseline comparison decided — included in the report's
/// `baseline` section so consumers see *why* no diff exists.
#[derive(Debug)]
pub enum BaselineDiff {
    /// Same schema, fixture, viewport, theme AND widget set → keys diffed.
    Comparable { new: Vec<String>, resolved: Vec<String> },
    /// Any signature component disagreed → the reports describe different UIs.
    NotComparable { reason: String },
}

impl BaselineDiff {
    pub fn to_json(&self) -> serde_json::Value {
        match self {
            Self::Comparable { new, resolved } => serde_json::json!({
                "comparable": true,
                "new": new,
                "resolved": resolved,
            }),
            Self::NotComparable { reason } => serde_json::json!({
                "comparable": false,
                "reason": reason,
            }),
        }
    }
}

/// Load a baseline report file and diff it against the current signature +
/// issue keys. IO errors are tool errors, surfaced to the caller.
pub fn diff_with_baseline(
    baseline_path: &std::path::Path,
    signature: &Signature,
    current_keys: &std::collections::HashSet<String>,
) -> Result<BaselineDiff, String> {
    let text = std::fs::read_to_string(baseline_path)
        .map_err(|e| format!("baseline unreadable at {}: {e}", baseline_path.display()))?;
    let base: serde_json::Value =
        serde_json::from_str(&text).map_err(|e| format!("baseline is not report JSON: {e}"))?;
    let base_version = base.get("schema_version").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
    if base_version != SCHEMA_VERSION {
        return Ok(BaselineDiff::NotComparable {
            reason: format!("schema_version {base_version} ≠ {SCHEMA_VERSION}"),
        });
    }
    let base_sig: Signature = serde_json::from_value(
        base.get("signature")
            .cloned()
            .ok_or_else(|| "baseline has no signature".to_owned())?,
    )
    .map_err(|e| format!("baseline signature invalid: {e}"))?;
    if base_sig != *signature {
        // Name the first disagreement so the user sees which condition broke.
        let reason = if base_sig.scenario != signature.scenario {
            format!("scenario {} ≠ {}", base_sig.scenario, signature.scenario)
        } else if base_sig.fixture != signature.fixture {
            format!("fixture {} ≠ {}", base_sig.fixture, signature.fixture)
        } else if base_sig.viewport != signature.viewport {
            format!("viewport {} ≠ {}", base_sig.viewport, signature.viewport)
        } else if base_sig.theme != signature.theme {
            format!("theme {} ≠ {}", base_sig.theme, signature.theme)
        } else {
            "widget set differs — different UI snapshot".to_owned()
        };
        return Ok(BaselineDiff::NotComparable { reason });
    }
    let base_keys: std::collections::HashSet<String> = base
        .get("issues")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|i| i.get("key").and_then(|k| k.as_str()))
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default();
    let mut new: Vec<String> = current_keys.difference(&base_keys).cloned().collect();
    let mut resolved: Vec<String> = base_keys.difference(current_keys).cloned().collect();
    new.sort();
    resolved.sort();
    Ok(BaselineDiff::Comparable { new, resolved })
}

/// Issues' identity keys — fed to `diff_with_baseline`.
pub fn issue_keys(report: &AuditReport) -> std::collections::HashSet<String> {
    report.issues.iter().map(|i| IssueKey::of(i).line()).collect()
}

/// What CI fails on (`DB_PRO_AUDIT_FAIL_ON`, comma-separated tokens).
///
/// - `error` (default): FAIL verdicts or Error-severity issues.
/// - `warning`: additionally fails on Warning issues.
/// - `screen`: additionally fails on *objective* screen findings (geometry
///   facts like off-screen rects); heuristic screen findings never gate.
///
/// AI review observations can never influence the exit code — they are
/// advisory output appended after the verdict is computed.
#[derive(Clone, Debug, Default)]
pub struct FailPolicy {
    pub on_warnings: bool,
    pub on_objective_screen: bool,
}

/// Env var selecting the CI fail policy.
pub const FAIL_ON_ENV: &str = "DB_PRO_AUDIT_FAIL_ON";

impl FailPolicy {
    pub fn from_env() -> Self {
        let mut p = Self::default();
        if let Ok(v) = std::env::var(FAIL_ON_ENV) {
            for token in v.split(',').map(|t| t.trim()) {
                match token {
                    "warning" => p.on_warnings = true,
                    "screen" => p.on_objective_screen = true,
                    "error" | "" => {}
                    other => tracing::warn!(token = other, "audit: unknown fail-on token"),
                }
            }
        }
        p
    }

    /// Reported in meta so CI consumers see which policy produced the code.
    pub fn label(&self) -> &'static str {
        match (self.on_warnings, self.on_objective_screen) {
            (false, false) => "error",
            (true, false) => "error,warning",
            (false, true) => "error,screen",
            (true, true) => "error,warning,screen",
        }
    }
}

/// Exit code under an explicit policy — pure, testable.
pub fn exit_code_with_policy(report: &AuditReport, policy: &FailPolicy) -> i32 {
    let failed = report.rules.iter().any(|r| r.verdict() == Verdict::Fail)
        || report.issues.iter().any(|i| i.severity == Severity::Error)
        || (policy.on_warnings && report.issues.iter().any(|i| i.severity == Severity::Warning))
        || (policy.on_objective_screen
            && report.screen.as_ref().is_some_and(|s| {
                s.findings
                    .iter()
                    .any(|f| f.kind == crate::dev_tools::screen::FindingKind::Objective)
            }));
    if failed {
        EXIT_AUDIT_FAILED
    } else {
        EXIT_OK
    }
}

/// Exit code for an audit run under the env-configured policy. FAIL verdicts
/// or Error-severity issues always fail; warnings and objective screen
/// findings only when `DB_PRO_AUDIT_FAIL_ON` asks. SKIP/N/A and AI output
/// never decide the code.
pub fn exit_code(report: &AuditReport) -> i32 {
    exit_code_with_policy(report, &FailPolicy::from_env())
}

#[cfg(test)]
#[path = "tests/report_tests.rs"]
mod tests;
