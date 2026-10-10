//! AI visual review (P11) — provider-agnostic second opinion on a captured
//! screen.
//!
//! The headless audit emits a schema-v3 report; when `DB_PRO_REVIEW_DIR` is
//! set the capture driver additionally writes a *review bundle* (screenshot
//! PNG + bundle JSON containing the report's meta, screen metrics, and
//! issues) and hands it to an external provider command
//! (`DB_PRO_AI_REVIEW_CMD`). The command receives the bundle path on argv
//! and must print a review JSON on stdout. Whatever it returns lands in the
//! report's `visual_review` section — never in `issues`, never in the exit
//! code. AI output informs; it cannot gate, and it can never modify code,
//! rules, or verdicts.
//!
//! Privacy contract:
//!
//! - Review is opt-in only: unset `DB_PRO_REVIEW_DIR` → no bundle, no
//!   provider call, no data leaves the process.
//! - The bundle carries audit data only: scenario/viewport/theme metadata,
//!   measured screen metrics, and rule issues. Widget labels, text values,
//!   connection names, credentials, and query results are never serialized —
//!   [`build_bundle`] whitelists keys rather than copying the report.
//! - The screenshot is a local file reference inside the bundle dir; whether
//!   bytes leave the machine is the provider's decision, not ours.
//! - Audit scenarios run on seeded fixtures, not live connections.

use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Review bundle directory (`DB_PRO_REVIEW_DIR`) — opt-in switch. Without it
/// nothing is written and no provider runs.
pub const REVIEW_DIR_ENV: &str = "DB_PRO_REVIEW_DIR";
/// External provider command (`DB_PRO_AI_REVIEW_CMD`). Receives the bundle
/// JSON path as argv[1]; must write review JSON to stdout.
pub const PROVIDER_CMD_ENV: &str = "DB_PRO_AI_REVIEW_CMD";
/// Optional model selector, forwarded to the provider as `DB_PRO_REVIEW_MODEL`.
pub const MODEL_ENV: &str = "DB_PRO_AI_REVIEW_MODEL";
/// Provider timeout in seconds (`DB_PRO_AI_REVIEW_TIMEOUT`, default 60).
pub const TIMEOUT_ENV: &str = "DB_PRO_AI_REVIEW_TIMEOUT";

/// A provider's verdict vocabulary — kept narrow so reports stay comparable
/// across providers.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewVerdict {
    Ok,
    Attention,
    Poor,
}

/// What the observation is about — the database-IDE review dimensions.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewCategory {
    Hierarchy,
    Typography,
    Spacing,
    Balance,
    Density,
    Consistency,
    Clarity,
    Other,
}

/// The contract split: `observation` claims something visible in the
/// evidence (objective); `suggestion` is advice (subjective). Reports keep
/// them separate so a suggestion never masquerades as a measurement.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObservationKind {
    Observation,
    Suggestion,
}

/// Provider output contract. Unknown fields are ignored; missing or
/// mistyped required fields fail the whole review — partial AI output is
/// worse than none.
#[derive(Debug, Deserialize)]
pub struct VisualReview {
    pub provider: String,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub verdict: Option<ReviewVerdict>,
    #[serde(default)]
    pub summary: Option<String>,
    #[serde(default)]
    pub observations: Vec<Observation>,
}

#[derive(Debug, Deserialize)]
pub struct Observation {
    pub category: ReviewCategory,
    pub kind: ObservationKind,
    #[serde(default = "default_severity")]
    pub severity: String,
    #[serde(default = "default_confidence")]
    pub confidence: String,
    /// Optional region the observation refers to: `[x, y, w, h]`.
    #[serde(default)]
    pub rect: Option<[f32; 4]>,
    /// What the provider claims to see — required so an observation is
    /// checkable, not a bare opinion.
    pub evidence: String,
    /// Concrete change the provider recommends; may be empty for pure
    /// observations that need no action.
    #[serde(default)]
    pub recommendation: Option<String>,
}

fn default_severity() -> String {
    "info".into()
}
fn default_confidence() -> String {
    "medium".into()
}

/// Parse and validate provider stdout. Errors are strings, never panics —
/// a broken provider must degrade the review section, not the audit.
pub fn parse_review(raw: &str) -> Result<VisualReview, String> {
    let review: VisualReview =
        serde_json::from_str(raw).map_err(|e| format!("provider output is not valid review JSON: {e}"))?;
    for (i, o) in review.observations.iter().enumerate() {
        if o.evidence.trim().is_empty() {
            return Err(format!(
                "observation {i} has empty evidence — unverifiable claim rejected"
            ));
        }
        for (field, value, allowed) in [
            ("severity", o.severity.as_str(), ["info", "warning"].as_slice()),
            (
                "confidence",
                o.confidence.as_str(),
                ["high", "medium", "low"].as_slice(),
            ),
        ] {
            if !allowed.contains(&value) {
                return Err(format!("observation {i}: {field} '{value}' not in {allowed:?}"));
            }
        }
    }
    Ok(review)
}

/// Whitelist of report keys copied into the bundle. Issues carry rule ids,
/// severities, and rect-derived evidence — never labels, text values, or
/// connection data (the report format itself omits them).
const BUNDLE_KEYS: &[&str] = &["schema_version", "meta", "summary", "rules", "issues", "screen"];

/// Build the review bundle JSON: report excerpt + screenshot reference.
/// Whitelisting is the privacy boundary — new report fields are NOT passed
/// through automatically.
pub fn build_bundle(report: &serde_json::Value, screenshot_file: &str) -> serde_json::Value {
    let mut excerpt = serde_json::Map::new();
    for key in BUNDLE_KEYS {
        if let Some(v) = report.get(*key) {
            excerpt.insert((*key).to_owned(), v.clone());
        }
    }
    serde_json::json!({
        "bundle_version": 1,
        "tool": "db-pro-ui-visual-review",
        "context": "database-ide",
        "screenshot": screenshot_file,
        "report": excerpt,
        "instructions": {
            "task": "Review visual hierarchy, typography, spacing, balance, density, consistency and UX clarity of this database IDE screen.",
            "ground_rules": [
                "Distinguish observations (visible in evidence) from suggestions (advice).",
                "Every observation needs evidence and a confidence level.",
                "Dense data grids, narrow side rails and stacked toolbars are intentional IDE patterns, not defects.",
                "Recommend concrete changes only; never restate audit issues as new findings."
            ],
            "output_schema": {
                "provider": "string", "model": "string?",
                "verdict": "ok|attention|poor?",
                "summary": "string?",
                "observations": [{
                    "category": "hierarchy|typography|spacing|balance|density|consistency|clarity|other",
                    "kind": "observation|suggestion",
                    "severity": "info|warning",
                    "confidence": "high|medium|low",
                    "rect": "[x,y,w,h]?",
                    "evidence": "string (required)",
                    "recommendation": "string?"
                }]
            }
        }
    })
}

/// How a review attempt ended — recorded in the report either way so a
/// failed provider is visible evidence, not silence.
pub enum ReviewOutcome {
    /// Provider returned a valid review.
    Completed(Box<VisualReview>),
    /// Provider configured but failed (spawn/timeout/malformed output).
    Failed(String),
    /// Bundle written but no provider configured — bundle exists for a
    /// later/manual review pass.
    BundleOnly,
    /// Refused: the screenshot may contain live database data. The bundle
    /// JSON is still written — it is whitelisted text — but nothing is
    /// offered to a provider.
    Blocked(String),
}

impl ReviewOutcome {
    /// Serialize into the report's `visual_review` section.
    pub fn to_json(&self, bundle_file: &str, screenshot_file: &str) -> serde_json::Value {
        match self {
            Self::Completed(r) => serde_json::json!({
                "status": "completed",
                "bundle": bundle_file,
                "screenshot": screenshot_file,
                "provider": r.provider,
                "model": r.model,
                "verdict": r.verdict.map(|v| format!("{v:?}").to_lowercase()),
                "summary": r.summary,
                "observations": r.observations.iter().map(|o| serde_json::json!({
                    "category": format!("{:?}", o.category).to_lowercase(),
                    "kind": format!("{:?}", o.kind).to_lowercase(),
                    "severity": o.severity,
                    "confidence": o.confidence,
                    "rect": o.rect,
                    "evidence": o.evidence,
                    "recommendation": o.recommendation,
                })).collect::<Vec<_>>(),
            }),
            Self::Failed(e) => serde_json::json!({
                "status": "error",
                "bundle": bundle_file,
                "screenshot": screenshot_file,
                "error": e,
            }),
            Self::BundleOnly => serde_json::json!({
                "status": "bundle_only",
                "bundle": bundle_file,
                "screenshot": screenshot_file,
                "note": "no provider configured (DB_PRO_AI_REVIEW_CMD unset) — bundle written for later review",
            }),
            Self::Blocked(reason) => serde_json::json!({
                "status": "blocked",
                "bundle": bundle_file,
                "screenshot": screenshot_file,
                "reason": reason,
            }),
        }
    }
}
/// Screenshot privacy gate: a review may only ship a screenshot that cannot
/// contain live data. Safe when there is no live connection, or when the
/// app proves the surface is fixture-seeded (`provenance` — set by code that
/// ran the fixture, never by an env string an operator could spoof).
/// Default posture is withhold: no provenance, no provider call.
pub fn screenshot_is_safe(live_connection: bool, provenance: bool) -> bool {
    !live_connection || provenance
}

/// Run the configured provider against a written bundle. `None` provider
/// command → [`ReviewOutcome::BundleOnly`]. Provider failures return
/// `Failed(reason)` — callers must not let this change the audit verdict.
pub fn run_provider(bundle_path: &Path) -> ReviewOutcome {
    let Ok(cmd) = std::env::var(PROVIDER_CMD_ENV) else {
        return ReviewOutcome::BundleOnly;
    };
    if cmd.trim().is_empty() {
        return ReviewOutcome::BundleOnly;
    }
    let timeout = std::env::var(TIMEOUT_ENV)
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .map(Duration::from_secs)
        .unwrap_or(Duration::from_secs(60));
    match run_command(&cmd, bundle_path, timeout) {
        Ok(raw) => match parse_review(&raw) {
            Ok(review) => ReviewOutcome::Completed(Box::new(review)),
            Err(e) => ReviewOutcome::Failed(e),
        },
        Err(e) => ReviewOutcome::Failed(e),
    }
}

/// Spawn `sh -c "<cmd> <bundle-path>"`, collect stdout with a timeout.
/// stderr is inherited for provider diagnostics; stdout is the contract.
fn run_command(cmd: &str, bundle_path: &Path, timeout: Duration) -> Result<String, String> {
    use std::io::Read;
    let mut command = std::process::Command::new("sh");
    command
        .arg("-c")
        .arg(format!("{} \"$1\"", cmd))
        .arg("db-pro-ai-review")
        .arg(bundle_path)
        .stdout(std::process::Stdio::piped());
    // Model selection is opaque to us — the provider decides what it means.
    if let Ok(model) = std::env::var(MODEL_ENV) {
        command.env("DB_PRO_REVIEW_MODEL", model);
    }
    let mut child = command.spawn().map_err(|e| format!("provider spawn failed: {e}"))?;
    // stdout on a side thread so a chatty provider can't deadlock on a full
    // pipe while the main thread only waits.
    let mut out_pipe = child.stdout.take().ok_or("provider stdout unavailable")?;
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let mut buf = String::new();
        let _ = out_pipe.read_to_string(&mut buf);
        let _ = tx.send(buf);
    });
    let deadline = std::time::Instant::now() + timeout;
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let out = rx.recv().unwrap_or_default();
                return if status.success() {
                    Ok(out)
                } else {
                    Err(format!("provider exited {status}: {out}"))
                };
            }
            Ok(None) => {
                if std::time::Instant::now() > deadline {
                    let _ = child.kill();
                    return Err(format!("provider timed out after {}s", timeout.as_secs()));
                }
                std::thread::sleep(Duration::from_millis(20));
            }
            Err(e) => {
                let _ = child.kill();
                return Err(format!("provider wait failed: {e}"));
            }
        }
    }
}

/// Where the bundle lands: `<review_dir>/<scenario>.bundle.json`, screenshot
/// `<review_dir>/<scenario>.png`.
pub fn bundle_path(review_dir: &Path, scenario: &str) -> PathBuf {
    review_dir.join(format!("{scenario}.bundle.json"))
}
pub fn screenshot_path(review_dir: &Path, scenario: &str) -> PathBuf {
    review_dir.join(format!("{scenario}.png"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

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
}
