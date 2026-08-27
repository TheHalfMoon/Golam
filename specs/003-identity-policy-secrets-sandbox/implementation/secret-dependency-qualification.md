# T003-004 — Secret Cryptography & Key-Protection Qualification

**Decision**: `ADMITTED_EXACT_VERSIONS_WITH_PLATFORM_BOUNDARIES`

No real secret material is required or authorized for Spec 003 qualification. Tests use deterministic canaries only.

## Vault encryption

Admit:

```toml
aes-gcm = { version = "=0.11.0", default-features = false, features = ["aes", "alloc", "zeroize"] }
zeroize = { version = "=1.9.0", default-features = false, features = ["alloc"] }
```

### AES-GCM

- upstream: `https://github.com/RustCrypto/AEADs`
- tag: `aes-gcm-v0.11.0`
- tag commit: `a10b56f281e2d3770e86aec024ea735b2dfa566b`
- crate: `aes-gcm 0.11.0`
- license: Apache-2.0 OR MIT
- MSRV: Rust 1.85
- implementation: pure Rust with optional architecture hardware acceleration
- audit: upstream documents an NCC Group audit with no significant findings

Golam will generate nonces through its already-admitted `getrandom = =0.4.3` boundary instead of enabling the aes-gcm `getrandom` feature. A fresh 96-bit nonce is required for every AES-256-GCM encryption under a key; nonce reuse is a hard failure.

AEAD associated data must bind at least secret ID, immutable version, classification/security metadata version, and vault format version so ciphertext cannot be transplanted between records without authentication failure.

### Zeroize

- upstream: `https://github.com/RustCrypto/utils`
- tag: `zeroize-v1.9.0`
- tag commit: `0b715735a660a8566ccd240bf42489fe2ed98efb`
- crate: `zeroize 1.9.0`
- license: Apache-2.0 OR MIT
- MSRV: Rust 1.85
- pure Rust; no FFI or assembly

Zeroization is defense in depth for Golam-owned plaintext buffers. It does not create a claim that all copies made by operating systems, allocator internals, third-party credential stores, or crash dumps are erased.

## Master-key protection API

Golam will define its own narrow Rust `KeyProtector` boundary. Production implementations may store/retrieve only the random vault master key through explicitly selected OS credential stores. The generic keyring all-in-one crate is **not admitted** because backend selection is security-relevant and must remain explicit.

Admit common interface:

```toml
keyring-core = { version = "=1.0.0", default-features = false }
```

- upstream: `https://github.com/open-source-cooperative/keyring-core`
- tag: `v1.0.0`
- tag commit: `eb41b5cd54694c1622d3c30c59f2e87368463151`
- license: MIT OR Apache-2.0
- MSRV: Rust 1.85
- no default features
- `sample` feature is forbidden for production; upstream explicitly does not warrant sample/mock stores as secure or robust.

## macOS

Admit only the local keychain profile:

```toml
apple-native-keyring-store = { version = "=1.0.2", default-features = false, features = ["keychain"] }
```

- upstream tag `v1.0.2`
- tag commit `78cdfff31e8a6579119b75ff7cbfeae7d4fc7d0a`
- license: MIT OR Apache-2.0
- MSRV: Rust 1.85
- uses Apple's Security Framework through the `security-framework` crate/FFI boundary.

Do **not** enable the `protected` feature in Spec 003. That feature can use Apple Protected Data/iCloud synchronization and requires provisioning entitlements; cloud synchronization is not part of the local vault contract. The CLI/daemon profile uses local macOS Keychain only.

## Windows

Admit:

```toml
windows-native-keyring-store = { version = "=1.1.0", default-features = false }
```

- upstream tag `v1.1.0`
- tag commit `65bf68219cab395e7b508f57df1aa0899d20face`
- license: MIT OR Apache-2.0
- MSRV: Rust 1.88
- native boundary: Windows Credential Manager through `windows-sys`
- default `search` feature is disabled, removing the unnecessary regex/search surface.

The provider documents unreliable ordering when the same entry is mutated from different threads. Golam therefore serializes master-key entry operations inside its privileged key-protection boundary and never relies on concurrent same-entry ordering.

## Linux

Admit:

```toml
zbus-secret-service-keyring-store = { version = "=1.0.1", default-features = false, features = ["crypto-rust"] }
```

- upstream tag `v1.0.1`
- tag commit `a97612aa64dd0148ad2504b3a9d4d82ca94e070b`
- release date: 2026-08-15
- license: MIT OR Apache-2.0
- MSRV: Rust 1.88
- backend: freedesktop Secret Service over local D-Bus/ZBus
- cryptographic transport feature: Rust implementation only; OpenSSL feature is not admitted.
- v1.0.1 fixes binary-secret update MIME handling and is preferred over 1.0.0.

Secret Service availability is not assumed. Headless/minimal Linux systems may lack a usable service. Production behavior when the configured OS key protector is unavailable, locked, corrupt, or returns ambiguous data is **fail closed**. There is no plaintext file fallback, environment-variable fallback, command-line fallback, or silent replacement with an in-process test store.

## Test boundary

CI must not require access to a developer's real OS credential store. Secret-vault tests use a deterministic test-only `KeyProtector` implementation carrying canary keys inside the test process. That fake cannot be selected by production configuration and cannot create a security claim for an OS backend.

Platform-specific integration tests may exercise an available disposable OS credential store only with deterministic canary material and explicit isolation; lack of such a store must not weaken production fail-closed semantics.

## Not admitted

- `keyring` all-in-one crate;
- keyring sample/mock store as production backend;
- Argon2/passphrase-derived fallback in this slice;
- OpenSSL-backed Secret Service mode;
- Apple `protected`/iCloud-synchronizing profile;
- plaintext master-key files;
- real credentials.

Any need for a passphrase recovery/export feature is a separate future design/qualification decision.

```text
T003_004=PASS
AES_GCM=0.11.0
ZEROIZE=1.9.0
KEYRING_CORE=1.0.0
MACOS_KEYCHAIN_PROVIDER=apple-native-keyring-store@1.0.2:keychain
WINDOWS_PROVIDER=windows-native-keyring-store@1.1.0:no-default-features
LINUX_PROVIDER=zbus-secret-service-keyring-store@1.0.1:crypto-rust
PLAINTEXT_FALLBACK=FORBIDDEN
REAL_SECRETS_USED=NO
```
