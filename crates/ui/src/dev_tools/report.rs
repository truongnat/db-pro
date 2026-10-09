//! Headless audit report: serializes `AuditReport` + environment metadata to a
//! versioned JSON schema for CI / script consumption (`DB_PRO_AUDIT_JSON`).
//!
//! Schema v1 keeps every verdict explicit — `skipped`/`not_applicable` are
//! distinct from `pass`, and the exit code never treats them as pass.

use super::audit::{AuditIssue, AuditReport, Severity, Verdict};

/// Bump when the report shape changes; baseline loaders check it first.
/// v3 adds the `screen` section (P10): measured screen metrics plus a
/// separate finding list (screen findings never affect the exit code).
/// v2 adds `scenario` to meta/signature and `layout_fingerprint` to meta.
pub const SCHEMA_VERSION: u32 = 3;

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
    let mut widget_ids: Vec<u64> = report.widget_ids.iter().copied().collect();
    widget_ids.sort_unstable();
    let signature = Signature {
        schema_version: SCHEMA_VERSION,
        scenario: rc.scenario.clone(),
        fixture: rc.fixture,
        viewport: rc.viewport.clone(),
        theme: rc.theme_label.clone(),
        widget_ids,
    };

    let rules: Vec<serde_json::Value> = report
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
        .collect();

    let issues: Vec<serde_json::Value> = report
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
        .collect();

    let errors = report.issues.iter().filter(|i| i.severity == Severity::Error).count();
    let warnings = report.issues.iter().filter(|i| i.severity == Severity::Warning).count();
    let fail_rules = report.rules.iter().filter(|r| r.verdict() == Verdict::Fail).count();

    serde_json::json!({
        "schema_version": SCHEMA_VERSION,
        "meta": {
            "tool": "db-pro-ui-audit",
            "binary": "db-pro-native",
            "viewport": rc.viewport,
            "theme": rc.theme_label,
            "scenario": rc.scenario,
            "fixture": rc.fixture,
            "frames": rc.frames,
            "layout_fingerprint": report.layout_fingerprint,
            "os": std::env::consts::OS,
            "timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
        },
        "signature": signature,
        "summary": {
            "widgets": report.widget_count,
            "rules": report.rules.len(),
            "issues": report.issues.len(),
            "errors": errors,
            "warnings": warnings,
            "verdicts": {
                "pass": report.rules.iter().filter(|r| r.verdict() == Verdict::Pass).count(),
                "fail": fail_rules,
                "skip": report.rules.iter().filter(|r| r.verdict() == Verdict::Skipped).count(),
                "not_applicable": report.rules.iter().filter(|r| r.verdict() == Verdict::NotApplicable).count(),
            },
        },
        "rules": rules,
        "issues": issues,
        "screen": report.screen.as_ref().map(render_screen),
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

/// Exit code for an audit run: FAIL verdicts or Error-severity issues mean the
/// audit failed; SKIP/N/A are reported but never decide the code.
pub fn exit_code(report: &AuditReport) -> i32 {
    let failed = report.rules.iter().any(|r| r.verdict() == Verdict::Fail)
        || report.issues.iter().any(|i| i.severity == Severity::Error);
    if failed {
        EXIT_AUDIT_FAILED
    } else {
        EXIT_OK
    }
}

#[cfg(test)]
mod tests {
    use super::super::audit::{AuditIssue, AuditReport, Confidence, RuleReport, RuleStats, Severity};
    use super::*;

    fn issue(rule: &'static str, widget_bits: u64, other_bits: Option<u64>, sev: Severity) -> AuditIssue {
        AuditIssue {
            rule_id: rule,
            severity: sev,
            confidence: Confidence::High,
            widget_id: egui::Id::new(widget_bits),
            widget_debug: egui::Id::new(widget_bits).short_debug_format(),
            other_id: other_bits.map(egui::Id::new),
            evidence: "test evidence".to_owned(),
        }
    }

    fn report(issues: Vec<AuditIssue>, rules: Vec<RuleReport>, ids: &[u64]) -> AuditReport {
        AuditReport {
            issues,
            rules,
            screen: None,
            widget_count: ids.len(),
            widget_ids: ids.iter().copied().collect(),
            layout_fingerprint: 0,
        }
    }

    #[test]
    fn schema_and_exit_constants_are_stable() {
        // The schema version and exit codes are a public contract for CI.
        assert_eq!(SCHEMA_VERSION, 3);
        assert_eq!(EXIT_OK, 0);
        assert_eq!(EXIT_AUDIT_FAILED, 1);
        assert_eq!(EXIT_TOOL_ERROR, 2);
    }

    #[test]
    fn render_report_emits_verdicts_coverage_and_signature() {
        let ctx = egui::Context::default();
        let r = report(
            vec![issue("geometry.invalid", 7, None, Severity::Error)],
            vec![
                RuleReport {
                    rule_id: "geometry.invalid",
                    issues: 1,
                    stats: RuleStats {
                        evaluated: 5,
                        ..Default::default()
                    },
                },
                RuleReport {
                    rule_id: "visual.alignment_drift",
                    issues: 0,
                    stats: RuleStats {
                        not_applicable: 2,
                        ..Default::default()
                    },
                },
                RuleReport {
                    rule_id: "focus.orphaned",
                    issues: 0,
                    stats: RuleStats {
                        skipped: 1,
                        ..Default::default()
                    },
                },
            ],
            &[7, 9],
        );
        let rc = ReportContext {
            viewport: "1440x900".into(),
            fixture: true,
            frames: 12,
            scenario: "test-scenario".into(),
            theme_label: "Dark".into(),
        };
        let j = render_report(&r, &ctx, &rc);

        assert_eq!(j["schema_version"], SCHEMA_VERSION as u64);
        assert_eq!(j["meta"]["viewport"], "1440x900");
        assert_eq!(j["meta"]["fixture"], true);
        assert_eq!(j["summary"]["issues"], 1);
        assert_eq!(j["summary"]["errors"], 1);
        assert_eq!(j["summary"]["verdicts"]["fail"], 1);
        assert_eq!(j["summary"]["verdicts"]["not_applicable"], 1);
        assert_eq!(j["summary"]["verdicts"]["skip"], 1);
        // Signatures are the equivalence gate — viewport/theme/fixture + ids.
        assert_eq!(j["signature"]["viewport"], "1440x900");
        assert_eq!(j["signature"]["widget_ids"], serde_json::json!([7, 9]));
        // Per-rule verdict + coverage surfaced, never collapsed to "pass".
        let drift = j["rules"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["id"] == "visual.alignment_drift")
            .unwrap();
        assert_eq!(drift["verdict"], "not_applicable");
        assert_eq!(drift["coverage"]["not_applicable"], 2);
        // Issues carry a stable identity key + evidence + confidence.
        let i0 = &j["issues"][0];
        assert_eq!(i0["severity"], "error");
        assert_eq!(i0["confidence"], "high");
        assert!(i0["key"].as_str().unwrap().starts_with("geometry.invalid|"));
    }

    #[test]
    fn exit_code_fails_on_findings_and_ignores_skip_na() {
        let clean = report(
            vec![],
            vec![
                RuleReport {
                    rule_id: "a",
                    issues: 0,
                    stats: RuleStats {
                        evaluated: 3,
                        ..Default::default()
                    },
                },
                RuleReport {
                    rule_id: "b",
                    issues: 0,
                    stats: RuleStats {
                        skipped: 2,
                        ..Default::default()
                    },
                },
            ],
            &[1],
        );
        // SKIP/N-A verdicts alone must not flip the run to failed.
        assert_eq!(exit_code(&clean), EXIT_OK);
        let bad = report(vec![issue("x", 1, None, Severity::Warning)], vec![], &[1]);
        // A warning alone isn't a Fail verdict here (no rule rows) — but the
        // issue exists; warnings still pass, errors fail.
        assert_eq!(exit_code(&bad), EXIT_OK);
        let err = report(vec![issue("x", 1, None, Severity::Error)], vec![], &[1]);
        assert_eq!(exit_code(&err), EXIT_AUDIT_FAILED);
    }

    fn write_tmp(json: &serde_json::Value) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("dbpro-audit-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let p = dir.join("baseline.json");
        std::fs::write(&p, serde_json::to_string(json).unwrap()).unwrap();
        p
    }

    #[test]
    fn baseline_diffs_only_on_equal_signatures() {
        let ctx = egui::Context::default();
        let r = report(
            vec![issue("geometry.invalid", 7, None, Severity::Error)],
            vec![],
            &[7, 9],
        );
        let rc = ReportContext {
            viewport: "1440x900".into(),
            fixture: false,
            frames: 1,
            scenario: "test-scenario".into(),
            theme_label: "Dark".into(),
        };
        let j = render_report(&r, &ctx, &rc);
        let sig: Signature = serde_json::from_value(j["signature"].clone()).unwrap();
        let keys = issue_keys(&r);

        // Same conditions, same snapshot → comparable; identical issues →
        // empty diff.
        let path = write_tmp(&j);
        match diff_with_baseline(&path, &sig, &keys).unwrap() {
            BaselineDiff::Comparable { new, resolved } => {
                assert!(new.is_empty() && resolved.is_empty());
            }
            _ => panic!("same signature must be comparable"),
        }

        // New issue → reported as new; missing issue → resolved.
        let mut base = j.clone();
        base["issues"] = serde_json::json!([
            {"key": "old.rule|AAAA|"},
        ]);
        let path = write_tmp(&base);
        match diff_with_baseline(&path, &sig, &keys).unwrap() {
            BaselineDiff::Comparable { new, resolved } => {
                assert_eq!(new.len(), 1);
                assert_eq!(resolved, vec!["old.rule|AAAA|".to_owned()]);
            }
            _ => panic!("expected comparable"),
        }

        // Different viewport → not comparable, reason names the field.
        let mut other_sig = sig.clone();
        other_sig.viewport = "1920x1080".into();
        match diff_with_baseline(&path, &other_sig, &keys).unwrap() {
            BaselineDiff::NotComparable { reason } => assert!(reason.contains("viewport")),
            _ => panic!("different viewport must not compare"),
        }
        // Different scenario → not comparable, reason names the field.
        let mut scenario_sig = sig.clone();
        scenario_sig.scenario = "other-scenario".into();
        match diff_with_baseline(&path, &scenario_sig, &keys).unwrap() {
            BaselineDiff::NotComparable { reason } => assert!(reason.contains("scenario")),
            _ => panic!("different scenario must not compare"),
        }
    }

    #[test]
    fn baseline_unreadable_is_a_tool_error_not_a_verdict() {
        let sig = Signature {
            schema_version: SCHEMA_VERSION,
            scenario: "test".into(),
            fixture: false,
            viewport: "x".into(),
            theme: "Dark".into(),
            widget_ids: vec![],
        };
        let missing = std::path::Path::new("/nonexistent/dbpro-baseline.json");
        assert!(diff_with_baseline(missing, &sig, &Default::default()).is_err());
    }
}
