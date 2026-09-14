//! Contract test for the v0.1.0 candidate evidence manifest (issue #88).
//!
//! `docs/release/evidence-manifest.json` ties every qualification result and
//! packaged artifact to one release-candidate SHA. The release process depends on
//! that file being true, so the rules it relies on are pinned here rather than
//! left to review: one SHA for every result and artifact, no gate claimed as
//! passing without repo evidence, every evidence pointer resolving to a file that
//! exists, and no publication claim while a distribution blocker is open.

use serde_json::{Map, Value};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

/// Manifest location, relative to the repository root.
const MANIFEST_RELATIVE_PATH: &str = "docs/release/evidence-manifest.json";
/// This crate's `CARGO_MANIFEST_DIR` is `crates/native-app`, two levels below the root.
const REPO_ROOT_FROM_CRATE_MANIFEST_DIR: &str = "../..";
const SHA_HEX_LENGTH: usize = 40;
const SHORT_SHA_HEX_LENGTH: usize = 7;
const SHA256_HEX_LENGTH: usize = 64;
const MIN_RECORDED_BYTES: u64 = 1;

const PASSING_GATE_STATUS: &str = "pass";
const GATE_STATUSES: [&str; 6] = ["pass", "partial", "failed", "not_run", "not_applicable", "blocked"];
/// Gate statuses that a release candidate may carry without invalidating
/// `claims.internal_rc_ready`; the release policy allows recorded non-pass states
/// (an unrun GUI smoke is not a failed gate) but never a failed one.
const GATE_STATUSES_COMPATIBLE_WITH_INTERNAL_RC: [&str; 4] = ["pass", "partial", "not_run", "not_applicable"];

/// Keys whose string values are repository paths and must resolve to a non-empty file.
const REPOSITORY_PATH_KEYS: [&str; 6] = [
    "evidence",
    "reference",
    "registry",
    "artifact_contract",
    "smoke_preparation",
    "version_source",
];

const REQUIRED_TOP_LEVEL_KEYS: [&str; 16] = [
    "manifest_version",
    "kind",
    "app",
    "candidate",
    "collection",
    "claims",
    "pipeline_run",
    "gates",
    "artifacts",
    "checksum_manifest",
    "build_artifacts",
    "provenance",
    "signing",
    "p2_dispositions",
    "known_limitations",
    "handoff",
];

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(REPO_ROOT_FROM_CRATE_MANIFEST_DIR)
}

fn manifest() -> Value {
    let path = repo_root().join(MANIFEST_RELATIVE_PATH);
    let raw = fs::read_to_string(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    serde_json::from_str(&raw).unwrap_or_else(|error| panic!("parse {} as JSON: {error}", path.display()))
}

fn object<'a>(value: &'a Value, context: &str) -> &'a Map<String, Value> {
    value
        .as_object()
        .unwrap_or_else(|| panic!("{context} must be a JSON object"))
}

fn array<'a>(value: &'a Value, context: &str) -> &'a Vec<Value> {
    value
        .as_array()
        .unwrap_or_else(|| panic!("{context} must be a JSON array"))
}

fn text<'a>(value: &'a Value, context: &str) -> &'a str {
    value.as_str().unwrap_or_else(|| panic!("{context} must be a string"))
}

fn field<'a>(value: &'a Value, key: &str, context: &str) -> &'a Value {
    object(value, context)
        .get(key)
        .unwrap_or_else(|| panic!("{context} must carry `{key}`"))
}

fn is_lowercase_hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn assert_repository_path_exists(path: &str, context: &str) {
    let resolved = repo_root().join(path);
    let metadata = fs::metadata(&resolved)
        .unwrap_or_else(|error| panic!("{context} points at `{path}`, which cannot be read: {error}"));
    assert!(
        metadata.is_file(),
        "{context} points at `{path}`, which is not a regular file"
    );
    assert!(
        metadata.len() >= MIN_RECORDED_BYTES,
        "{context} points at `{path}`, which is empty"
    );
}

/// Collects every repository path the manifest cites, so a stale pointer fails the
/// build instead of surviving as documentation debt.
fn collect_repository_paths(value: &Value, context: &str, collected: &mut BTreeSet<String>) {
    match value {
        Value::Object(entries) => {
            for (key, child) in entries {
                let child_context = format!("{context}.{key}");
                if REPOSITORY_PATH_KEYS.contains(&key.as_str()) {
                    match child {
                        Value::String(path) => {
                            collected.insert(path.clone());
                        }
                        Value::Array(paths) => {
                            for item in paths {
                                collected.insert(text(item, &child_context).to_string());
                            }
                        }
                        other => panic!("{child_context} must be a path string or an array of paths, got {other}"),
                    }
                }
                collect_repository_paths(child, &child_context, collected);
            }
        }
        Value::Array(items) => {
            for (index, item) in items.iter().enumerate() {
                collect_repository_paths(item, &format!("{context}[{index}]"), collected);
            }
        }
        _ => {}
    }
}

fn candidate_shas(manifest: &Value) -> BTreeSet<String> {
    let candidate = field(manifest, "candidate", "manifest");
    let mut allowed = BTreeSet::from([text(field(candidate, "sha", "candidate"), "candidate.sha").to_string()]);
    for descendant in array(
        field(candidate, "descendant_evidence_shas", "candidate"),
        "candidate.descendant_evidence_shas",
    ) {
        allowed.insert(text(field(descendant, "sha", "descendant"), "descendant.sha").to_string());
    }
    allowed
}

#[test]
fn manifest_carries_every_required_section() {
    let manifest = manifest();
    let entries = object(&manifest, "manifest");
    for key in REQUIRED_TOP_LEVEL_KEYS {
        assert!(
            entries.contains_key(key),
            "manifest must carry the required top-level key `{key}`"
        );
    }
}

#[test]
fn manifest_version_matches_the_release_source_of_truth() {
    let manifest = manifest();
    let version = text(
        field(field(&manifest, "app", "manifest"), "version", "app"),
        "app.version",
    );
    assert_eq!(
        version,
        env!("CARGO_PKG_VERSION"),
        "the manifest must describe the version this workspace builds (db-pro-native is the release source of truth)"
    );
}

#[test]
fn candidate_sha_is_one_well_formed_commit() {
    let manifest = manifest();
    let candidate = field(&manifest, "candidate", "manifest");
    let sha = text(field(candidate, "sha", "candidate"), "candidate.sha");
    let short_sha = text(field(candidate, "short_sha", "candidate"), "candidate.short_sha");
    assert!(
        is_lowercase_hex(sha, SHA_HEX_LENGTH),
        "candidate.sha `{sha}` must be 40 lowercase hex characters"
    );
    assert!(
        is_lowercase_hex(short_sha, SHORT_SHA_HEX_LENGTH),
        "candidate.short_sha `{short_sha}` must be 7 lowercase hex characters"
    );
    assert!(
        sha.starts_with(short_sha),
        "candidate.short_sha `{short_sha}` must be a prefix of `{sha}`"
    );

    for descendant in array(
        field(candidate, "descendant_evidence_shas", "candidate"),
        "candidate.descendant_evidence_shas",
    ) {
        let descendant_sha = text(field(descendant, "sha", "descendant"), "descendant.sha");
        assert!(
            is_lowercase_hex(descendant_sha, SHA_HEX_LENGTH),
            "descendant `{descendant_sha}` must be a full commit sha"
        );
        for key in ["relationship", "ancestry_check"] {
            let value = text(field(descendant, key, "descendant"), &format!("descendant.{key}"));
            assert!(
                !value.trim().is_empty(),
                "descendant `{descendant_sha}` must explain its `{key}`"
            );
        }
    }
}

#[test]
fn every_gate_status_is_truthful_and_backed_by_evidence() {
    let manifest = manifest();
    let allowed_shas = candidate_shas(&manifest);
    let mut seen_ids = BTreeSet::new();

    let gates = array(field(&manifest, "gates", "manifest"), "gates");
    assert!(!gates.is_empty(), "the manifest must list the gates it claims");

    for gate in gates {
        let id = text(field(gate, "id", "gate"), "gate.id");
        assert!(seen_ids.insert(id.to_string()), "gate id `{id}` appears twice");

        let status = text(field(gate, "status", "gate"), &format!("gate `{id}` status"));
        assert!(
            GATE_STATUSES.contains(&status),
            "gate `{id}` has unknown status `{status}`"
        );

        let observed_at = text(
            field(gate, "observed_at_sha", "gate"),
            &format!("gate `{id}` observed_at_sha"),
        );
        assert!(
            allowed_shas.contains(observed_at),
            "gate `{id}` was observed at `{observed_at}`, which is neither the candidate nor a recorded descendant — \
             a result pointing at another SHA cannot ship as this candidate's evidence"
        );

        let evidence = array(field(gate, "evidence", "gate"), &format!("gate `{id}` evidence"));
        match status {
            PASSING_GATE_STATUS => assert!(
                !evidence.is_empty(),
                "gate `{id}` is claimed as `pass`, so it must cite the evidence that shows it"
            ),
            // A gate that did not pass must say why, so a skipped or unrun gate can
            // never read as an unexplained pass.
            _ => {
                let reason = text(field(gate, "reason", "gate"), &format!("gate `{id}` reason"));
                assert!(
                    !reason.trim().is_empty(),
                    "gate `{id}` is `{status}` and must carry a non-empty reason"
                );
            }
        }

        for item in evidence {
            assert_repository_path_exists(text(item, &format!("gate `{id}` evidence")), &format!("gate `{id}`"));
        }
    }
}

#[test]
fn no_result_or_artifact_points_at_a_sha_other_than_the_candidate() {
    let manifest = manifest();
    let candidate_sha = text(
        field(field(&manifest, "candidate", "manifest"), "sha", "candidate"),
        "candidate.sha",
    );

    for artifact in array(field(&manifest, "artifacts", "manifest"), "artifacts") {
        let name = text(field(artifact, "archive", "artifact"), "artifact.archive");
        let artifact_sha = text(
            field(artifact, "candidate_sha", "artifact"),
            &format!("artifact `{name}`"),
        );
        assert_eq!(
            artifact_sha, candidate_sha,
            "artifact `{name}` must be built from the candidate commit"
        );
    }

    for artifact in array(field(&manifest, "build_artifacts", "manifest"), "build_artifacts") {
        let name = text(field(artifact, "name", "build artifact"), "build_artifact.name");
        let artifact_sha = text(
            field(artifact, "candidate_sha", "build artifact"),
            &format!("build artifact `{name}`"),
        );
        assert_eq!(
            artifact_sha, candidate_sha,
            "build artifact `{name}` must be built from the candidate commit"
        );
    }

    for section in ["checksum_manifest", "provenance"] {
        let section_sha = text(
            field(field(&manifest, section, "manifest"), "candidate_sha", section),
            &format!("{section}.candidate_sha"),
        );
        assert_eq!(
            section_sha, candidate_sha,
            "`{section}` must describe the candidate commit"
        );
    }

    let provenance_sha = field(&manifest, "provenance", "manifest");
    let recorded = field(provenance_sha, "values", "provenance");
    assert_eq!(
        text(
            field(recorded, "candidate_sha", "provenance.values"),
            "provenance.values.candidate_sha"
        ),
        candidate_sha,
        "the CI-recorded provenance must name the same commit as the manifest"
    );
}

#[test]
fn every_artifact_is_checksummed_or_explicitly_unavailable() {
    let manifest = manifest();
    let mut checksummed = Vec::new();
    checksummed.extend(array(field(&manifest, "artifacts", "manifest"), "artifacts").iter());
    checksummed.extend(array(field(&manifest, "build_artifacts", "manifest"), "build_artifacts").iter());
    checksummed.push(field(&manifest, "checksum_manifest", "manifest"));

    for entry in checksummed {
        let label = ["archive", "name"]
            .iter()
            .find_map(|key| object(entry, "artifact").get(*key).and_then(Value::as_str))
            .unwrap_or("checksum_manifest");

        let recorded_bytes = ["size_bytes", "uploaded_bytes"]
            .iter()
            .filter_map(|key| object(entry, "artifact").get(*key).and_then(Value::as_u64))
            .max();
        let measured = recorded_bytes.unwrap_or_else(|| panic!("`{label}` must record its byte size"));
        assert!(measured >= MIN_RECORDED_BYTES, "`{label}` records a zero-byte size");

        match object(entry, "artifact").get("sha256") {
            Some(Value::String(sha256)) => assert!(
                is_lowercase_hex(sha256, SHA256_HEX_LENGTH),
                "`{label}` sha256 `{sha256}` must be 64 lowercase hex characters"
            ),
            _ => {
                let reason = text(
                    field(entry, "sha256_unavailable_reason", &format!("`{label}`")),
                    &format!("`{label}` sha256_unavailable_reason"),
                );
                assert!(
                    !reason.trim().is_empty(),
                    "`{label}` has no sha256, so it must record why the checksum is unavailable"
                );
            }
        }
    }
}

#[test]
fn a_superseded_artifact_never_poses_as_the_candidate() {
    let manifest = manifest();
    // Same archive name is expected across runs (the version does not change); what
    // must never be reused is the artifact *content*, identified by its digest.
    let candidate_digests: BTreeSet<(String, String)> = array(field(&manifest, "artifacts", "manifest"), "artifacts")
        .iter()
        .map(|artifact| {
            (
                text(field(artifact, "archive", "artifact"), "artifact.archive").to_string(),
                text(field(artifact, "sha256", "artifact"), "artifact.sha256").to_string(),
            )
        })
        .collect();
    let candidate_run_id = field(field(&manifest, "pipeline_run", "manifest"), "run_id", "pipeline_run");

    for entry in array(field(&manifest, "superseded_history", "manifest"), "superseded_history") {
        let run_id = field(entry, "run_id", "superseded entry");
        assert_ne!(
            run_id, candidate_run_id,
            "the candidate's own run cannot be listed as superseded history"
        );
        for artifact in array(
            field(entry, "artifacts", "superseded entry"),
            "superseded entry artifacts",
        ) {
            let archive = text(
                field(artifact, "archive", "superseded artifact"),
                "superseded artifact.archive",
            );
            let digest = text(
                field(artifact, "sha256", "superseded artifact"),
                "superseded artifact.sha256",
            );
            assert!(
                !candidate_digests.contains(&(archive.to_string(), digest.to_string())),
                "`{archive}` from superseded run {run_id} carries the candidate's digest `{digest}`"
            );
        }
    }
}

#[test]
fn publication_claims_match_the_recorded_blockers() {
    let manifest = manifest();
    let claims = field(&manifest, "claims", "manifest");
    let internal_ready = field(claims, "internal_rc_ready", "claims")
        .as_bool()
        .expect("claims.internal_rc_ready must be a boolean");
    let public_ready = field(claims, "public_release_ready", "claims")
        .as_bool()
        .expect("claims.public_release_ready must be a boolean");
    let blockers = array(
        field(claims, "public_release_blockers", "claims"),
        "claims.public_release_blockers",
    );

    let gate_statuses: Vec<&str> = array(field(&manifest, "gates", "manifest"), "gates")
        .iter()
        .map(|gate| text(field(gate, "status", "gate"), "gate.status"))
        .collect();

    if internal_ready {
        for status in &gate_statuses {
            assert!(
                GATE_STATUSES_COMPATIBLE_WITH_INTERNAL_RC.contains(status),
                "claims.internal_rc_ready is true while a gate is `{status}`"
            );
        }
    }

    if public_ready {
        assert!(
            blockers.is_empty(),
            "claims.public_release_ready is true while {} distribution blocker(s) are recorded",
            blockers.len()
        );
        assert!(
            gate_statuses.iter().all(|status| *status == PASSING_GATE_STATUS),
            "claims.public_release_ready is true while a gate is not `pass`"
        );
        let signing_status = text(
            field(field(&manifest, "signing", "manifest"), "status", "signing"),
            "signing.status",
        );
        assert_eq!(
            signing_status, "configured",
            "claims.public_release_ready needs configured signing"
        );
    } else {
        assert!(
            !blockers.is_empty(),
            "claims.public_release_ready is false, so the blocking reason must be recorded"
        );
    }
}

#[test]
fn every_evidence_pointer_resolves_to_a_file_in_this_tree() {
    let manifest = manifest();
    let mut cited = BTreeSet::new();
    collect_repository_paths(&manifest, "manifest", &mut cited);

    for path in &cited {
        assert_repository_path_exists(path, "manifest pointer");
    }
    assert!(
        !cited.is_empty(),
        "the manifest must cite the evidence it was assembled from"
    );
}
