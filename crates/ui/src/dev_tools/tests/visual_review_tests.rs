//! Tests for `../visual_review.rs` (moved out of the source file).
use serde_json::json;

use super::*;

fn report() -> serde_json::Value {
    json!({
        "schema_version": 3,
        "meta": {"scenario": "shell", "viewport": "1440x900"},
        "summary": {"widgets": 10, "issues": 0},
        "rules": [{"id": "geometry.invalid", "verdict": "pass"}],
        "issues": [],
        "screen": {"sufficient": true},
        // Must NOT leak into the bundle — not whitelisted.
        "signature": {"widget_ids": [1, 2]},
        "secret_field": "hunter2",
    })
}

#[test]
fn bundle_whitelists_report_sections() {
    let b = build_bundle(&report(), "shot.png");
    let r = &b["report"];
    assert!(r.get("meta").is_some() && r.get("screen").is_some());
    assert!(r.get("signature").is_none(), "signature stays out");
    assert!(r.get("secret_field").is_none(), "unknown keys never pass through");
    assert_eq!(b["screenshot"], "shot.png");
    assert_eq!(b["context"], "database-ide");
    // Instructions pin the objective/subjective split the schema enforces.
    assert!(b["instructions"]["output_schema"]["observations"][0]["kind"]
        .as_str()
        .unwrap()
        .contains("observation"));
}

#[test]
fn parse_valid_review_preserves_kinds() {
    let raw = r#"{
          "provider":"mock","model":"m1","verdict":"attention","summary":"dense",
          "observations":[
            {"category":"density","kind":"observation","severity":"warning",
             "confidence":"high","rect":[320,42,1108,644],
             "evidence":"46 controls within 2px in the workspace band"},
            {"category":"hierarchy","kind":"suggestion","confidence":"low",
             "evidence":"two competing accent colors","recommendation":"one accent"}
          ]}"#;
    let r = parse_review(raw).unwrap();
    assert_eq!(r.verdict, Some(ReviewVerdict::Attention));
    assert_eq!(r.observations[0].kind, ObservationKind::Observation);
    assert_eq!(r.observations[1].kind, ObservationKind::Suggestion);
    assert_eq!(r.observations[0].rect, Some([320.0, 42.0, 1108.0, 644.0]));
}

#[test]
fn parse_rejects_empty_evidence_and_bad_enums() {
    assert!(parse_review("not json").is_err());
    let no_evidence = r#"{"provider":"m","observations":[
            {"category":"spacing","kind":"observation","evidence":""}]}"#;
    assert!(parse_review(no_evidence).unwrap_err().contains("empty evidence"));
    let bad_kind = r#"{"provider":"m","observations":[
            {"category":"spacing","kind":"fact","evidence":"x"}]}"#;
    assert!(parse_review(bad_kind).is_err(), "kind enum is closed");
    let bad_conf = r#"{"provider":"m","observations":[
            {"category":"spacing","kind":"observation","confidence":"sure","evidence":"x"}]}"#;
    assert!(parse_review(bad_conf).unwrap_err().contains("confidence"));
}

#[test]
fn outcome_json_never_changes_audit_shape() {
    // Completed/Failed/BundleOnly all serialize under `visual_review`;
    // none touch issues/summary — verified structurally.
    for outcome in [
        ReviewOutcome::BundleOnly,
        ReviewOutcome::Failed("timeout".into()),
        ReviewOutcome::Completed(Box::new(VisualReview {
            provider: "m".into(),
            model: None,
            verdict: Some(ReviewVerdict::Ok),
            summary: None,
            observations: vec![],
        })),
    ] {
        let j = outcome.to_json("b.json", "s.png");
        assert!(j.get("status").is_some());
        assert!(
            j.get("issues").is_none()
                && j.get("summary")
                    .is_none_or(|s| !s.is_object() || s.get("issues").is_none())
        );
    }
}

#[test]
fn provider_unset_is_bundle_only_not_error() {
    std::env::remove_var(PROVIDER_CMD_ENV);
    match run_provider(Path::new("/tmp/nonexistent-bundle.json")) {
        ReviewOutcome::BundleOnly => {}
        other => panic!("unset provider must not be an error: {}", other.to_json("b", "s")),
    }
}

#[test]
fn privacy_gate_blocks_live_session_without_provenance() {
    // No live connection → always safe.
    assert!(screenshot_is_safe(false, false));
    assert!(screenshot_is_safe(false, true));
    // Live connection without provenance → withheld, no matter what env
    // claims (env flags are spoofable; provenance is a paint record).
    assert!(
        !screenshot_is_safe(true, false),
        "live session without provenance must withhold"
    );
    // Live connection + fixture provenance → released.
    assert!(screenshot_is_safe(true, true));
}
