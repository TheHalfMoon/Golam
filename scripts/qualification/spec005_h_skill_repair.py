from pathlib import Path


def replace_once(path: str, old: str, new: str) -> None:
    p = Path(path)
    content = p.read_text()
    if old not in content:
        raise SystemExit(f"expected pattern missing in {path}: {old[:180]!r}")
    p.write_text(content.replace(old, new, 1))


replace_once(
    "crates/golam-core/src/skills_protocol.rs",
    """    Deprecated,
    Revoked,
    Unknown,
""",
    """    Deprecated,
    Revoked,
    Replaced,
    Unknown,
""",
)

replace_once(
    "crates/golam-core/src/skills_protocol.rs",
    """        if !self.script_refs.is_empty()
            && !matches!(
                self.admission_state,
                SkillAdmissionState::ExecutableAdmitted | SkillAdmissionState::LockedVersion
            )
        {
            return Err(ProtocolValidationError::ExecutableSkillNotAdmitted);
        }
        Ok(())
""",
    """        // Script references are discovery evidence, not executable authority. A package may
        // expose scripts while remaining instruction-only; executable dispatch is gated separately
        // by `SkillDispatchBinding::revalidate` and current lifecycle state.
        Ok(())
""",
)

replace_once(
    "crates/golamd/src/lib.rs",
    """pub mod process_secret_evidence;
pub mod static_elf_v2;
""",
    """pub mod process_secret_evidence;
pub mod skill_packages;
pub mod static_elf_v2;
""",
)

replace_once(
    "crates/golamd/src/skill_packages.rs",
    """        lifecycle.transition(SkillAdmissionState::Deprecated).unwrap();
        assert!(lifecycle.activate_instructions(&binding).is_err());
""",
    """        lifecycle.transition(SkillAdmissionState::Deprecated).unwrap();
        assert!(lifecycle.activate_instructions(&binding).is_err());
""",
)

replace_once(
    "crates/golamd/src/skill_packages.rs",
    """    pub fn transition(&mut self, next: SkillAdmissionState) -> Result<(), SkillPackageError> {
        if !allowed_transition(self.state, next) {
            return Err(SkillPackageError::InvalidLifecycleTransition {
                from: self.state,
                to: next,
            });
        }
        self.state = next;
        Ok(())
    }

    pub fn bind_instruction_activation(
""",
    """    pub fn transition(&mut self, next: SkillAdmissionState) -> Result<(), SkillPackageError> {
        if !allowed_transition(self.state, next) {
            return Err(SkillPackageError::InvalidLifecycleTransition {
                from: self.state,
                to: next,
            });
        }
        self.state = next;
        Ok(())
    }

    pub fn replace_with(
        &mut self,
        replacement: ReviewedInstructionSkill,
    ) -> Result<SkillLifecycle, SkillPackageError> {
        self.transition(SkillAdmissionState::Replaced)?;
        Ok(SkillLifecycle::new(replacement))
    }

    pub fn bind_instruction_activation(
""",
)

replace_once(
    "crates/golamd/src/skill_packages.rs",
    """    let skill_bytes = read_bounded(&canonical_root.join(\"SKILL.md\"), MAX_SKILL_MD_BYTES)?;
    if skill_bytes.iter().filter(|byte| **byte == b'\\n').count() + 1 > MAX_SKILL_LINES {
""",
    """    let skill_bytes = read_bounded(&canonical_root.join(\"SKILL.md\"), MAX_SKILL_MD_BYTES)?;
    if skill_file.byte_len != u64::try_from(skill_bytes.len()).unwrap_or(u64::MAX)
        || skill_file.content_digest != BindingDigest::new(sha256(&skill_bytes))
    {
        return Err(SkillPackageError::PackageChangedDuringDiscovery);
    }
    if skill_bytes.iter().filter(|byte| **byte == b'\\n').count() + 1 > MAX_SKILL_LINES {
""",
)

replace_once(
    "crates/golamd/src/skill_packages.rs",
    """    let script_refs = files
        .iter()
        .filter(|file| file.script_candidate)
        .map(|file| file.content_digest)
        .collect::<Vec<_>>();
""",
    """    let mut script_refs = files
        .iter()
        .filter(|file| file.script_candidate)
        .map(|file| file.content_digest)
        .collect::<Vec<_>>();
    script_refs.sort_unstable();
    script_refs.dedup();
""",
)

replace_once(
    "crates/golamd/src/skill_packages.rs",
    """    MissingSkillFile,
    ProvenanceRequired,
""",
    """    MissingSkillFile,
    PackageChangedDuringDiscovery,
    ProvenanceRequired,
""",
)

replace_once(
    "crates/golamd/src/skill_packages.rs",
    """            Self::MissingSkillFile => f.write_str(\"skill package is missing root SKILL.md\"),
            Self::ProvenanceRequired => f.write_str(\"skill package requires reviewed provenance evidence\"),
""",
    """            Self::MissingSkillFile => f.write_str(\"skill package is missing root SKILL.md\"),
            Self::PackageChangedDuringDiscovery => {
                f.write_str(\"skill package changed while discovery evidence was being frozen\")
            }
            Self::ProvenanceRequired => f.write_str(\"skill package requires reviewed provenance evidence\"),
""",
)

replace_once(
    "crates/golamd/src/skill_packages.rs",
    """    if value.contains(['\\0', '\\r', '\\n']) {
        return Err(SkillPackageError::InvalidManifest(\"frontmatter scalar contains forbidden control bytes\"));
    }
""",
    """    if value.chars().any(|character| matches!(character, '\\0' | '\\r' | '\\n')) {
        return Err(SkillPackageError::InvalidManifest(\"frontmatter scalar contains forbidden control bytes\"));
    }
""",
)

replace_once(
    "crates/golamd/src/skill_packages.rs",
    """        SkillAdmissionState::Deprecated => 7,
        SkillAdmissionState::Revoked => 8,
        SkillAdmissionState::Unknown => 9,
""",
    """        SkillAdmissionState::Deprecated => 7,
        SkillAdmissionState::Revoked => 8,
        SkillAdmissionState::Replaced => 9,
        SkillAdmissionState::Unknown => 10,
""",
)

replace_once(
    "crates/golamd/src/skill_packages.rs",
    """        (Discovered, ProvenanceRecorded | Reviewed | Deprecated | Revoked | Unknown)
            | (ProvenanceRecorded, Reviewed | Deprecated | Revoked | Unknown)
            | (Reviewed, InstructionAdmitted | ExecutableAdmitted | Deprecated | Revoked | Unknown)
            | (InstructionAdmitted, ExecutableAdmitted | LockedVersion | Deprecated | Revoked | Unknown)
            | (ExecutableAdmitted, LockedVersion | Deprecated | Revoked | Unknown)
            | (LockedVersion, Deprecated | Revoked | Unknown)
            | (Deprecated, Revoked | Unknown)
""",
    """        (Discovered, ProvenanceRecorded | Reviewed | Deprecated | Revoked | Replaced | Unknown)
            | (ProvenanceRecorded, Reviewed | Deprecated | Revoked | Replaced | Unknown)
            | (Reviewed, InstructionAdmitted | ExecutableAdmitted | Deprecated | Revoked | Replaced | Unknown)
            | (InstructionAdmitted, ExecutableAdmitted | LockedVersion | Deprecated | Revoked | Replaced | Unknown)
            | (ExecutableAdmitted, LockedVersion | Deprecated | Revoked | Replaced | Unknown)
            | (LockedVersion, Deprecated | Revoked | Replaced | Unknown)
            | (Deprecated, Revoked | Replaced | Unknown)
""",
)

replace_once(
    "crates/golamd/src/skill_packages.rs",
    """    fn lifecycle_is_forward_only_and_old_binding_dies_on_state_change() {
        let skill = TempSkill::new(\"lifecycle-skill\", \"# Lifecycle Skill\\nDo work.\", &[]);
        let reviewed = discover(&skill);
        let mut lifecycle = SkillLifecycle::new(reviewed);
        let binding = lifecycle
            .bind_instruction_activation(digest(10), digest(11), digest(12))
            .unwrap();
        lifecycle.transition(SkillAdmissionState::Deprecated).unwrap();
        assert!(lifecycle.activate_instructions(&binding).is_err());
        assert!(matches!(
            lifecycle.transition(SkillAdmissionState::InstructionAdmitted),
            Err(SkillPackageError::InvalidLifecycleTransition { .. })
        ));
    }
""",
    """    fn lifecycle_is_forward_only_and_old_binding_dies_on_state_change() {
        let skill = TempSkill::new(\"lifecycle-skill\", \"# Lifecycle Skill\\nDo work.\", &[]);
        let reviewed = discover(&skill);
        let mut lifecycle = SkillLifecycle::new(reviewed);
        let binding = lifecycle
            .bind_instruction_activation(digest(10), digest(11), digest(12))
            .unwrap();
        lifecycle.transition(SkillAdmissionState::Deprecated).unwrap();
        assert!(lifecycle.activate_instructions(&binding).is_err());
        assert!(matches!(
            lifecycle.transition(SkillAdmissionState::InstructionAdmitted),
            Err(SkillPackageError::InvalidLifecycleTransition { .. })
        ));
    }

    #[test]
    fn replacement_invalidates_old_binding_and_creates_new_reviewed_identity() {
        let old_skill = TempSkill::new(\"replace-old\", \"# Old\\nOld instructions.\", &[]);
        let new_skill = TempSkill::new(\"replace-new\", \"# New\\nNew instructions.\", &[]);
        let mut old = SkillLifecycle::new(discover(&old_skill));
        let old_binding = old
            .bind_instruction_activation(digest(20), digest(21), digest(22))
            .unwrap();
        let replacement = old.replace_with(discover(&new_skill)).unwrap();
        assert_eq!(old.state(), SkillAdmissionState::Replaced);
        assert!(old.activate_instructions(&old_binding).is_err());
        assert_ne!(
            old.reviewed().descriptor().package_ref,
            replacement.reviewed().descriptor().package_ref
        );
    }
""",
)
