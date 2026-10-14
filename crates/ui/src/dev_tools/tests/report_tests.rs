//! Tests for `../report.rs` (moved out of the source file).
use super::{
    super::audit::{AuditIssue, AuditReport, Confidence, RuleReport, RuleStats, Severity},
    *,
};

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
    assert_eq!(SCHEMA_VERSION, 4);
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

#[test]
fn fail_policy_gates_warnings_and_objective_screen_findings() {
    use crate::dev_tools::screen::{FindingKind, ScreenAnalysis, ScreenFinding};
    let warning = report(vec![issue("x", 1, None, Severity::Warning)], vec![], &[1]);
    // Default policy: warning passes.
    assert_eq!(exit_code_with_policy(&warning, &FailPolicy::default()), EXIT_OK);
    let warn_policy = FailPolicy {
        on_warnings: true,
        ..Default::default()
    };
    assert_eq!(exit_code_with_policy(&warning, &warn_policy), EXIT_AUDIT_FAILED);

    // Objective screen finding gates only under `screen`; heuristic never.
    let mut with_screen = report(vec![], vec![], &[1]);
    with_screen.screen = Some(ScreenAnalysis {
        findings: vec![
            ScreenFinding {
                rule: "screen.off_screen",
                kind: FindingKind::Objective,
                severity: Severity::Warning,
                confidence: super::super::audit::Confidence::High,
                widget_id: None,
                widget_debug: None,
                region: None,
                evidence: "test".into(),
            },
            ScreenFinding {
                rule: "screen.spacing_outlier",
                kind: FindingKind::Heuristic,
                severity: Severity::Warning,
                confidence: super::super::audit::Confidence::Medium,
                widget_id: None,
                widget_debug: None,
                region: None,
                evidence: "test".into(),
            },
        ],
        ..Default::default()
    });
    assert_eq!(exit_code_with_policy(&with_screen, &FailPolicy::default()), EXIT_OK);
    let screen_policy = FailPolicy {
        on_objective_screen: true,
        ..Default::default()
    };
    assert_eq!(exit_code_with_policy(&with_screen, &screen_policy), EXIT_AUDIT_FAILED);

    // Heuristic-only findings never gate.
    with_screen.screen.as_mut().unwrap().findings.remove(0);
    assert_eq!(exit_code_with_policy(&with_screen, &screen_policy), EXIT_OK);
}

#[test]
fn ai_review_never_reaches_exit_code() {
    // Contract, structural: exit_code consumes AuditReport only — there
    // is no field through which visual_review could leak in. Verify the
    // rendered report keeps the two channels separate.
    let ctx = egui::Context::default();
    let r = report(vec![], vec![], &[1]);
    let rc = ReportContext {
        viewport: "1440x900".into(),
        fixture: false,
        frames: 5,
        scenario: "s".into(),
        theme_label: "Dark".into(),
    };
    let mut j = render_report(&r, &ctx, &rc);
    j["visual_review"] = serde_json::json!({"status": "completed", "verdict": "poor"});
    assert_eq!(j["summary"]["issues"], 0);
    assert_eq!(j["visual_review"]["verdict"], "poor");
    // Exit code already computed from the report; AI verdict is inert.
    assert_eq!(exit_code_with_policy(&r, &FailPolicy::default()), EXIT_OK);
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
