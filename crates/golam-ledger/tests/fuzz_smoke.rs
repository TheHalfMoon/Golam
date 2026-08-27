#![forbid(unsafe_code)]

use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use golam_ledger::EventKind;
use golam_ledger::storage::{AUTHORITY_SCHEMA_VERSION, AuthorityStore, StorageError};
use rusqlite::Connection;

static N: AtomicU64 = AtomicU64::new(0);

fn temp_db(label: &str) -> std::path::PathBuf {
    let n = N.fetch_add(1, Ordering::Relaxed);
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "golam-fuzz-smoke-{label}-{}-{t}-{n}.db",
        std::process::id()
    ))
}

#[test]
fn event_kind_decoder_exhaustive_byte_corpus_is_bounded() {
    let expected = [
        EventKind::SessionCreated,
        EventKind::GoalVersioned,
        EventKind::EffectProposed,
        EventKind::EffectTransitioned,
        EventKind::CheckpointCreated,
        EventKind::SessionForked,
    ];

    for code in u8::MIN..=u8::MAX {
        let decoded = EventKind::from_code(code);
        if (1..=6).contains(&code) {
            assert_eq!(decoded, Some(expected[usize::from(code - 1)]));
        } else {
            assert_eq!(decoded, None);
        }
    }
}

#[test]
fn migration_version_corpus_migrates_zero_and_rejects_future_versions() {
    let zero_path = temp_db("schema-zero");
    let store = AuthorityStore::open(&zero_path).unwrap();
    assert_eq!(store.schema_version().unwrap(), AUTHORITY_SCHEMA_VERSION);
    drop(store);
    fs::remove_file(&zero_path).unwrap();

    for future in (AUTHORITY_SCHEMA_VERSION + 1)..=(AUTHORITY_SCHEMA_VERSION + 16) {
        let path = temp_db("future-schema");
        let connection = Connection::open(&path).unwrap();
        connection
            .execute_batch(&format!("PRAGMA user_version = {future};"))
            .unwrap();
        drop(connection);

        assert!(matches!(
            AuthorityStore::open(&path),
            Err(StorageError::FutureSchema { found, supported })
                if found == future && supported == AUTHORITY_SCHEMA_VERSION
        ));
        fs::remove_file(path).unwrap();
    }
}

#[test]
fn malformed_sqlite_header_fails_closed_without_reset() {
    for seed in [0_u8, 1, 0x55, 0xaa, 0xff] {
        let path = temp_db("malformed-header");
        let mut bytes = vec![seed; 512];
        bytes[..16].copy_from_slice(b"not-sqlite-db!!!");
        fs::write(&path, &bytes).unwrap();

        assert!(AuthorityStore::open(&path).is_err());
        assert_eq!(fs::read(&path).unwrap(), bytes);
        fs::remove_file(path).unwrap();
    }
}
