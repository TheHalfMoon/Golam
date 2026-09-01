#![forbid(unsafe_code)]

use golam_core::harness::{CompactionId, ExecutionProfileId, HardwareProfileId};
use golam_core::harness_state::{
    BenchmarkMetrics, BenchmarkRecord, BenchmarkResult, CalibrationObservation, CalibrationResult,
    CalibrationRun, CompactionArtifact,
};
use golam_ledger::harness_evidence::{
    HardwareProfileEvidence, HarnessEvidenceError, HarnessEvidenceStore,
};

fn assert_identity_collision(result: Result<(), HarnessEvidenceError>, expected: &'static str) {
    assert!(matches!(
        result,
        Err(HarnessEvidenceError::InvalidRecord(reason)) if reason == expected
    ));
}

fn benchmark(profile_id: u128, workload: &str) -> BenchmarkRecord {
    BenchmarkRecord {
        benchmark_id: 7,
        schema_version: 1,
        code_revision: "revision-a".into(),
        execution_profile_id: ExecutionProfileId::from_u128(profile_id),
        hardware_profile_id: HardwareProfileId::from_u128(2),
        workload_fixture_id: workload.into(),
        backend_metrics: vec![BenchmarkMetrics {
            metric_name: "backend_events".into(),
            value_milli: 1_000,
            unit: "count_milli".into(),
        }],
        harness_metrics: vec![BenchmarkMetrics {
            metric_name: "accepted_events".into(),
            value_milli: 1_000,
            unit: "count_milli".into(),
        }],
        started_at_unix_ms: 10,
        finished_at_unix_ms: 11,
        result: BenchmarkResult::Passed,
        raw_evidence_refs: vec!["evidence:benchmark".into()],
    }
}

fn calibration(workload: &str) -> CalibrationRun {
    CalibrationRun {
        calibration_id: 9,
        hardware_profile_id: HardwareProfileId::from_u128(2),
        backend_identity_ref: "backend:scripted:v1".into(),
        profile_candidate_digest: [3; 32],
        workload_fixture_id: workload.into(),
        started_at_unix_ms: 20,
        finished_at_unix_ms: Some(21),
        max_memory_bytes: 1024,
        max_runtime_ms: 100,
        observations: vec![CalibrationObservation {
            metric: "compatibility".into(),
            value_milli: 1_000,
            unit: "milli-ratio".into(),
        }],
        result: CalibrationResult::Supported,
        failure_class: None,
        evidence_refs: vec!["evidence:calibration".into()],
    }
}

#[test]
fn hardware_profile_identity_is_idempotent_only_for_exact_evidence() {
    let mut store = HarnessEvidenceStore::open_in_memory().unwrap();
    store
        .record_hardware_profile(HardwareProfileEvidence {
            profile_id: HardwareProfileId::from_u128(1),
            observed_at_unix_ms: 10,
            content_digest: [1; 32],
            record_bytes: b"hardware-a",
        })
        .unwrap();
    store
        .record_hardware_profile(HardwareProfileEvidence {
            profile_id: HardwareProfileId::from_u128(1),
            observed_at_unix_ms: 10,
            content_digest: [1; 32],
            record_bytes: b"hardware-a",
        })
        .unwrap();

    assert_identity_collision(
        store.record_hardware_profile(HardwareProfileEvidence {
            profile_id: HardwareProfileId::from_u128(1),
            observed_at_unix_ms: 11,
            content_digest: [2; 32],
            record_bytes: b"hardware-b",
        }),
        "hardware profile identity collision",
    );
}

#[test]
fn compaction_artifact_identity_rejects_conflicting_payload() {
    let mut store = HarnessEvidenceStore::open_in_memory().unwrap();
    let first = CompactionArtifact {
        compaction_id: CompactionId::from_u128(5),
        source_projection_ref: "projection:1".into(),
        source_event_refs: vec!["event:1".into()],
        goal_refs: vec!["goal:1".into()],
        deterministic: true,
        producing_request_attempt_id: None,
        accepted_output_ref: None,
        artifact_digest: [5; 32],
    };
    store
        .record_compaction_artifact(&first, b"artifact-a")
        .unwrap();
    store
        .record_compaction_artifact(&first, b"artifact-a")
        .unwrap();

    let mut conflicting = first;
    conflicting.artifact_digest = [6; 32];
    assert_identity_collision(
        store.record_compaction_artifact(&conflicting, b"artifact-b"),
        "compaction artifact identity collision",
    );
}

#[test]
fn benchmark_identity_rejects_stale_binding_reuse() {
    let mut store = HarnessEvidenceStore::open_in_memory().unwrap();
    let first = benchmark(1, "fixture:a");
    store.record_benchmark(&first, b"benchmark-a").unwrap();
    store.record_benchmark(&first, b"benchmark-a").unwrap();

    let conflicting = benchmark(99, "fixture:b");
    assert_identity_collision(
        store.record_benchmark(&conflicting, b"benchmark-b"),
        "benchmark identity collision",
    );
}

#[test]
fn calibration_identity_rejects_conflicting_binding() {
    let mut store = HarnessEvidenceStore::open_in_memory().unwrap();
    let first = calibration("fixture:a");
    store.record_calibration(&first, b"calibration-a").unwrap();
    store.record_calibration(&first, b"calibration-a").unwrap();

    let conflicting = calibration("fixture:b");
    assert_identity_collision(
        store.record_calibration(&conflicting, b"calibration-b"),
        "calibration identity collision",
    );
}
