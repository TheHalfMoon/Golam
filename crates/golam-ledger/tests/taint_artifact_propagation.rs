#![forbid(unsafe_code)]

use std::path::PathBuf;

use golam_core::taint::{Provenanced, TaintLabel, TaintSet};
use golam_ledger::artifacts::ArtifactReceipt;

fn receipt(byte: u8) -> ArtifactReceipt {
    ArtifactReceipt {
        hash: [byte; 32],
        size_bytes: 16,
        relative_path: PathBuf::from(format!("{:02x}/artifact-{byte}", byte)),
    }
}

#[test]
fn derived_artifact_keeps_content_identity_separate_from_monotonic_provenance() {
    let web = Provenanced::source(
        receipt(1),
        TaintSet::from_labels([TaintLabel::WebUntrusted]),
    );
    let local = Provenanced::source(
        receipt(2),
        TaintSet::from_labels([TaintLabel::LocalUnverified]),
    );
    let derived_receipt = receipt(3);
    let expected_hash = derived_receipt.hash;
    let expected_path = derived_receipt.relative_path.clone();

    let derived = Provenanced::derive(
        derived_receipt,
        [web.taint(), local.taint()],
        TaintSet::from_labels([TaintLabel::ModelGenerated]),
    );

    assert_eq!(derived.value().hash, expected_hash);
    assert_eq!(derived.value().relative_path, expected_path);
    assert!(derived.taint().contains(TaintLabel::WebUntrusted));
    assert!(derived.taint().contains(TaintLabel::LocalUnverified));
    assert!(derived.taint().contains(TaintLabel::ModelGenerated));
}

#[test]
fn derived_artifact_taint_is_source_order_invariant() {
    let web = TaintSet::from_labels([TaintLabel::WebUntrusted]);
    let channel = TaintSet::from_labels([TaintLabel::ChannelUntrusted]);
    let introduced = TaintSet::from_labels([TaintLabel::ModelGenerated]);

    let first = Provenanced::derive(receipt(4), [web, channel], introduced);
    let second = Provenanced::derive(receipt(4), [channel, web], introduced);

    assert_eq!(first.value(), second.value());
    assert_eq!(first.taint(), second.taint());
    assert_eq!(
        first.taint().canonical_bytes().unwrap(),
        second.taint().canonical_bytes().unwrap()
    );
}
