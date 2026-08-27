# T003-006 — Source Foundry Disposition

**Decision**: `NO_DONOR_CODE_ADMITTED`

## Golam-Research

The canonical planning package recorded `TheHalfMoon/Golam-research` at:

- commit `a9f633e09d49a85829b8236331b9e21f7e612634`
- tree `b68f24972427952c4934e4364736fec62661044f`

Founder-attested permission makes candidate source eligible for Source Foundry review, but permission does not force reuse and is not itself code admission.

Targeted Spec 003 donor review did not identify a bounded donor component that improves the Rust privileged authority implementation enough to justify importing its provenance, dependency, or security surface. The immediate implementation can and should be built from the frozen Golam contracts plus independently qualified Rust dependencies.

Therefore for this implementation slice:

```text
Golam-Research=REFERENCE_ONLY
DONOR_CODE_ADMITTED=NO
DONOR_FILES_COPIED=0
DONOR_DEPENDENCIES_ADDED=0
```

If a later task proposes donor reuse, it must stop before copying code and create a new per-source admission record containing exact selected files, permission evidence scope, license/notices, dependency closure, unsafe/process/network/secrets behavior, adaptation strategy, and independent Golam tests.

## Other external projects

Cedar, RustCrypto, and platform key-store crates are ordinary explicit dependencies qualified in separate dependency records; they are not admitted through the founder donor attestation mechanism.

```text
T003_006=PASS
SOURCE_FOUNDRY_REUSE_REQUIRED_NOW=NO
```
