#![forbid(unsafe_code)]

use golam_core::paths::RuntimeLayout;
use golam_ledger::recovery::{RecoveryMode, RecoveryScanner};
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static N: AtomicU64 = AtomicU64::new(0);

fn runtime() -> RuntimeLayout {
    let n = N.fetch_add(1, Ordering::Relaxed);
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    RuntimeLayout::initialize(std::env::temp_dir().join(format!(
        "golam-recovery-reserve-policy-{}-{t}-{n}",
        std::process::id()
    )))
    .unwrap()
}

#[test]
fn unproven_recovery_reserve_is_not_created_or_relied_on() {
    let runtime = runtime();
    let report = RecoveryScanner::scan(&runtime).unwrap();
    assert_eq!(report.mode, RecoveryMode::Normal);

    let reserve = runtime
        .data_dir
        .join("authority")
        .join("recovery-reserve.bin");
    assert!(
        !reserve.exists(),
        "Spec 002 must not create or rely on an unproven disk recovery reserve"
    );

    fs::remove_dir_all(runtime.root).unwrap();
}
