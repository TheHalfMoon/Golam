#![forbid(unsafe_code)]

//! Bounded Agent Skills-compatible package discovery and lifecycle validation for Spec 005.
//!
//! Skill package content is untrusted context. Discovery never grants authority: `allowed-tools`
//! and compatibility metadata are recorded as requested behavior only, while the reviewed Golam
//! capability mapping remains an independently supplied immutable binding reference.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};

use golam_core::digest::sha256;
use golam_core::skills_protocol::{
    CurrentSkillDispatchState, DispatchValidationError, SkillAdmissionState, SkillDescriptor,
    SkillDispatchBinding, SkillDispatchKind, SkillVersion,
};
use golam_core::tool_descriptor::ToolNetworkPosture;
use golam_core::tool_request::BindingDigest;
use golam_core::{CanonicalEncoder, CoreError};

const MAX_SKILL_MD_BYTES: u64 = 512 * 1024;
const MAX_PACKAGE_BYTES: u64 = 8 * 1024 * 1024;
const MAX_PACKAGE_FILES: usize = 256;
const MAX_DIRECTORY_DEPTH: usize = 8;
const MAX_RELATIVE_PATH_BYTES: usize = 512;
const MAX_DESCRIPTION_BYTES: usize = 1024;
const MAX_COMPATIBILITY_BYTES: usize = 500;
const MAX_OPTIONAL_SCALAR_BYTES: usize = 4096;
const MAX_METADATA_ENTRIES: usize = 64;
const MAX_METADATA_KEY_BYTES: usize = 128;
const MAX_METADATA_VALUE_BYTES: usize = 1024;
const MAX_SKILL_LINES: usize = 500;
const MAX_PROVENANCE_REFS: usize = 64;
const SKILL_CONTENT_DOMAIN: &[u8] = b"golam:skill-content:v1";
const SKILL_PACKAGE_DOMAIN: &[u8] = b"golam:skill-package:v1";
const SKILL_STATE_DOMAIN: &[u8] = b"golam:skill-state:v1";
const SKILL_NAME_DOMAIN: &[u8] = b"golam:skill-name:v1";
const SKILL_DESCRIPTION_DOMAIN: &[u8] = b"golam:skill-description:v1";
const SKILL_INSTRUCTION_DOMAIN: &[u8] = b"golam:skill-instruction:v1";
const SKILL_ALLOWED_TOOLS_DOMAIN: &[u8] = b"golam:skill-allowed-tools-request:v1";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UntrustedSkillInstructions(Vec<u8>);

impl UntrustedSkillInstructions {
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SkillManifest {
    pub name: String,
    pub description: String,
    pub license: Option<String>,
    pub compatibility: Option<String>,
    pub metadata: BTreeMap<String, String>,
    pub allowed_tools_request: Option<String>,
    pub allowed_tools_request_ref: Option<BindingDigest>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SkillPackageFileEvidence {
    pub relative_path: String,
    pub byte_len: u64,
    pub content_digest: BindingDigest,
    pub script_candidate: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReviewedInstructionSkill {
    canonical_root: PathBuf,
    descriptor: SkillDescriptor,
    instructions: UntrustedSkillInstructions,
    files: Vec<SkillPackageFileEvidence>,
    reviewed_capability_mapping_ref: BindingDigest,
    provenance_refs: Vec<BindingDigest>,
}

impl ReviewedInstructionSkill {
    pub fn canonical_root(&self) -> &Path {
        &self.canonical_root
    }

    pub fn descriptor(&self) -> &SkillDescriptor {
        &self.descriptor
    }

    pub fn instructions(&self) -> &UntrustedSkillInstructions {
        &self.instructions
    }

    pub fn files(&self) -> &[SkillPackageFileEvidence] {
        &self.files
    }

    pub const fn reviewed_capability_mapping_ref(&self) -> BindingDigest {
        self.reviewed_capability_mapping_ref
    }

    pub fn provenance_refs(&self) -> &[BindingDigest] {
        &self.provenance_refs
    }

    pub fn current_state(&self, state: SkillAdmissionState) -> Result<CurrentSkillDispatchState, SkillPackageError> {
        Ok(CurrentSkillDispatchState {
            skill_package_ref: self.descriptor.package_ref,
            skill_version: self.descriptor.version.clone(),
            content_digest: self.descriptor.content_digest,
            admission_state: state,
            admission_state_ref: skill_state_ref(
                self.descriptor.package_ref,
                &self.descriptor.version,
                self.descriptor.content_digest,
                self.reviewed_capability_mapping_ref,
                state,
            )?,
            capability_mapping_ref: self.reviewed_capability_mapping_ref,
        })
    }

    fn rediscover_live(&self) -> Result<ReviewedInstructionSkill, SkillPackageError> {
        discover_reviewed_instruction_skill(SkillDiscoveryRequest {
            package_root: &self.canonical_root,
            version: self.descriptor.version.clone(),
            provenance_refs: self.provenance_refs.clone(),
            reviewed_capability_mapping_ref: self.reviewed_capability_mapping_ref,
        })
    }

    fn same_reviewed_identity(&self, other: &Self) -> bool {
        self.descriptor.package_ref == other.descriptor.package_ref
            && self.descriptor.version == other.descriptor.version
            && self.descriptor.content_digest == other.descriptor.content_digest
            && self.descriptor.instruction_ref == other.descriptor.instruction_ref
            && self.descriptor.script_refs == other.descriptor.script_refs
            && self.files == other.files
            && self.reviewed_capability_mapping_ref == other.reviewed_capability_mapping_ref
            && self.provenance_refs == other.provenance_refs
    }
}

#[derive(Clone, Debug)]
pub struct SkillDiscoveryRequest<'a> {
    pub package_root: &'a Path,
    pub version: SkillVersion,
    pub provenance_refs: Vec<BindingDigest>,
    pub reviewed_capability_mapping_ref: BindingDigest,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SkillLifecycle {
    reviewed: ReviewedInstructionSkill,
    state: SkillAdmissionState,
}

impl SkillLifecycle {
    pub fn new(reviewed: ReviewedInstructionSkill) -> Self {
        Self {
            reviewed,
            state: SkillAdmissionState::InstructionAdmitted,
        }
    }

    pub fn reviewed(&self) -> &ReviewedInstructionSkill {
        &self.reviewed
    }

    pub const fn state(&self) -> SkillAdmissionState {
        self.state
    }

    pub fn transition(&mut self, next: SkillAdmissionState) -> Result<(), SkillPackageError> {
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
        &self,
        queued_request_ref: BindingDigest,
        capability_decision_ref: BindingDigest,
        approval_decision_ref: BindingDigest,
    ) -> Result<SkillDispatchBinding, SkillPackageError> {
        require_nonzero_ref(queued_request_ref, "queued_request_ref")?;
        require_nonzero_ref(capability_decision_ref, "capability_decision_ref")?;
        require_nonzero_ref(approval_decision_ref, "approval_decision_ref")?;
        let current = self.reviewed.current_state(self.state)?;
        if !matches!(
            current.admission_state,
            SkillAdmissionState::InstructionAdmitted
                | SkillAdmissionState::ExecutableAdmitted
                | SkillAdmissionState::LockedVersion
        ) {
            return Err(SkillPackageError::LifecycleNotDispatchable(self.state));
        }
        Ok(SkillDispatchBinding {
            skill_package_ref: current.skill_package_ref,
            skill_version: current.skill_version,
            reviewed_content_digest: current.content_digest,
            reviewed_admission_state_ref: current.admission_state_ref,
            reviewed_capability_mapping_ref: current.capability_mapping_ref,
            queued_request_ref,
            capability_decision_ref,
            approval_decision_ref,
            dispatch_kind: SkillDispatchKind::InstructionActivation,
        })
    }

    pub fn activate_instructions(
        &self,
        binding: &SkillDispatchBinding,
    ) -> Result<&UntrustedSkillInstructions, SkillPackageError> {
        if binding.dispatch_kind != SkillDispatchKind::InstructionActivation {
            return Err(SkillPackageError::WrongDispatchKind);
        }
        let live = self.reviewed.rediscover_live()?;
        if !self.reviewed.same_reviewed_identity(&live) {
            return Err(SkillPackageError::LivePackageChanged);
        }
        let current = self.reviewed.current_state(self.state)?;
        binding.revalidate(&current)?;
        Ok(self.reviewed.instructions())
    }

    pub fn bind_executable_dispatch(
        &self,
        queued_request_ref: BindingDigest,
        capability_decision_ref: BindingDigest,
        approval_decision_ref: BindingDigest,
    ) -> Result<SkillDispatchBinding, SkillPackageError> {
        if !matches!(
            self.state,
            SkillAdmissionState::ExecutableAdmitted | SkillAdmissionState::LockedVersion
        ) {
            return Err(SkillPackageError::ExecutableSkillNotAdmitted);
        }
        let mut binding = self.bind_instruction_activation(
            queued_request_ref,
            capability_decision_ref,
            approval_decision_ref,
        )?;
        binding.dispatch_kind = SkillDispatchKind::ExecutableDispatch;
        Ok(binding)
    }
}

#[derive(Debug)]
pub enum SkillPackageError {
    Io {
        operation: &'static str,
        path: PathBuf,
        source: std::io::Error,
    },
    Core(CoreError),
    Protocol(golam_core::skills_protocol::ProtocolValidationError),
    Dispatch(DispatchValidationError),
    InvalidRoot(&'static str),
    InvalidManifest(&'static str),
    InvalidName,
    DescriptionOutOfBounds,
    CompatibilityOutOfBounds,
    OptionalScalarOutOfBounds(&'static str),
    TooManyMetadataEntries,
    MetadataOutOfBounds,
    TooManyFiles,
    PackageTooLarge,
    SkillFileTooLarge,
    TooManyLines,
    InvalidRelativePath,
    SymlinkForbidden(PathBuf),
    SpecialFileForbidden(PathBuf),
    DuplicatePath,
    MissingSkillFile,
    ProvenanceRequired,
    InvalidProvenanceOrder,
    ZeroBindingRef(&'static str),
    InvalidLifecycleTransition {
        from: SkillAdmissionState,
        to: SkillAdmissionState,
    },
    LifecycleNotDispatchable(SkillAdmissionState),
    WrongDispatchKind,
    LivePackageChanged,
    ExecutableSkillNotAdmitted,
}

impl fmt::Display for SkillPackageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { operation, path, source } => {
                write!(f, "skill package {operation} failed for {}: {source}", path.display())
            }
            Self::Core(error) => write!(f, "skill package canonical encoding failed: {error}"),
            Self::Protocol(error) => write!(f, "skill protocol validation failed: {error}"),
            Self::Dispatch(error) => write!(f, "skill dispatch revalidation failed: {error}"),
            Self::InvalidRoot(reason) => write!(f, "invalid skill package root: {reason}"),
            Self::InvalidManifest(reason) => write!(f, "invalid SKILL.md manifest: {reason}"),
            Self::InvalidName => f.write_str("invalid Agent Skills package name"),
            Self::DescriptionOutOfBounds => f.write_str("skill description is empty or exceeds bound"),
            Self::CompatibilityOutOfBounds => f.write_str("skill compatibility exceeds bound"),
            Self::OptionalScalarOutOfBounds(field) => write!(f, "skill optional scalar exceeds bound: {field}"),
            Self::TooManyMetadataEntries => f.write_str("skill metadata entry bound exceeded"),
            Self::MetadataOutOfBounds => f.write_str("skill metadata key/value exceeds bound"),
            Self::TooManyFiles => f.write_str("skill package file-count bound exceeded"),
            Self::PackageTooLarge => f.write_str("skill package byte bound exceeded"),
            Self::SkillFileTooLarge => f.write_str("SKILL.md byte bound exceeded"),
            Self::TooManyLines => f.write_str("SKILL.md line bound exceeded"),
            Self::InvalidRelativePath => f.write_str("skill package contains an invalid relative path"),
            Self::SymlinkForbidden(path) => write!(f, "skill package symlink is forbidden: {}", path.display()),
            Self::SpecialFileForbidden(path) => write!(f, "skill package special file is forbidden: {}", path.display()),
            Self::DuplicatePath => f.write_str("skill package contains duplicate canonical relative paths"),
            Self::MissingSkillFile => f.write_str("skill package is missing root SKILL.md"),
            Self::ProvenanceRequired => f.write_str("skill package requires reviewed provenance evidence"),
            Self::InvalidProvenanceOrder => f.write_str("skill provenance refs must be sorted and unique"),
            Self::ZeroBindingRef(field) => write!(f, "skill binding reference must be nonzero: {field}"),
            Self::InvalidLifecycleTransition { from, to } => write!(f, "invalid skill lifecycle transition: {from:?} -> {to:?}"),
            Self::LifecycleNotDispatchable(state) => write!(f, "skill lifecycle state is not dispatchable: {state:?}"),
            Self::WrongDispatchKind => f.write_str("skill binding dispatch kind does not match activation"),
            Self::LivePackageChanged => f.write_str("live skill package identity changed after review"),
            Self::ExecutableSkillNotAdmitted => f.write_str("skill executable dispatch is not independently admitted"),
        }
    }
}

impl Error for SkillPackageError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::Core(error) => Some(error),
            Self::Protocol(error) => Some(error),
            Self::Dispatch(error) => Some(error),
            _ => None,
        }
    }
}

impl From<CoreError> for SkillPackageError {
    fn from(value: CoreError) -> Self {
        Self::Core(value)
    }
}

impl From<golam_core::skills_protocol::ProtocolValidationError> for SkillPackageError {
    fn from(value: golam_core::skills_protocol::ProtocolValidationError) -> Self {
        Self::Protocol(value)
    }
}

impl From<DispatchValidationError> for SkillPackageError {
    fn from(value: DispatchValidationError) -> Self {
        Self::Dispatch(value)
    }
}

pub fn discover_reviewed_instruction_skill(
    input: SkillDiscoveryRequest<'_>,
) -> Result<ReviewedInstructionSkill, SkillPackageError> {
    validate_provenance(&input.provenance_refs)?;
    require_nonzero_ref(
        input.reviewed_capability_mapping_ref,
        "reviewed_capability_mapping_ref",
    )?;

    let root_meta = fs::symlink_metadata(input.package_root)
        .map_err(|source| io_error("inspect root", input.package_root, source))?;
    if root_meta.file_type().is_symlink() {
        return Err(SkillPackageError::SymlinkForbidden(input.package_root.to_owned()));
    }
    if !root_meta.is_dir() {
        return Err(SkillPackageError::InvalidRoot("package root must be a directory"));
    }
    let canonical_root = fs::canonicalize(input.package_root)
        .map_err(|source| io_error("canonicalize root", input.package_root, source))?;

    let files = collect_package_files(&canonical_root)?;
    let skill_file = files
        .iter()
        .find(|file| file.relative_path == "SKILL.md")
        .ok_or(SkillPackageError::MissingSkillFile)?;
    if skill_file.byte_len > MAX_SKILL_MD_BYTES {
        return Err(SkillPackageError::SkillFileTooLarge);
    }
    let skill_bytes = read_bounded(&canonical_root.join("SKILL.md"), MAX_SKILL_MD_BYTES)?;
    if skill_bytes.iter().filter(|byte| **byte == b'\n').count() + 1 > MAX_SKILL_LINES {
        return Err(SkillPackageError::TooManyLines);
    }
    let (manifest, body) = parse_manifest(&skill_bytes, &canonical_root)?;

    let package_content_digest = package_content_digest(&files)?;
    let package_ref = package_ref(&manifest.name, &input.version, &input.provenance_refs)?;
    let instruction_ref = digest_ref(SKILL_INSTRUCTION_DOMAIN, body)?;
    let script_refs = files
        .iter()
        .filter(|file| file.script_candidate)
        .map(|file| file.content_digest)
        .collect::<Vec<_>>();

    let descriptor = SkillDescriptor {
        name_ref: digest_ref(SKILL_NAME_DOMAIN, manifest.name.as_bytes())?,
        description_ref: digest_ref(SKILL_DESCRIPTION_DOMAIN, manifest.description.as_bytes())?,
        package_ref,
        version: input.version,
        content_digest: package_content_digest,
        instruction_ref,
        script_refs,
        requested_capability_classes: Vec::new(),
        network_posture: ToolNetworkPosture::Denied,
        provenance_refs: input.provenance_refs.clone(),
        admission_state: SkillAdmissionState::InstructionAdmitted,
    };
    descriptor.validate()?;

    Ok(ReviewedInstructionSkill {
        canonical_root,
        descriptor,
        instructions: UntrustedSkillInstructions(body.to_vec()),
        files,
        reviewed_capability_mapping_ref: input.reviewed_capability_mapping_ref,
        provenance_refs: input.provenance_refs,
    })
}

fn collect_package_files(root: &Path) -> Result<Vec<SkillPackageFileEvidence>, SkillPackageError> {
    let mut pending = vec![(root.to_owned(), 0_usize)];
    let mut paths = Vec::new();
    while let Some((directory, depth)) = pending.pop() {
        if depth > MAX_DIRECTORY_DEPTH {
            return Err(SkillPackageError::InvalidRoot("directory nesting exceeds bound"));
        }
        let entries = fs::read_dir(&directory)
            .map_err(|source| io_error("read directory", &directory, source))?;
        for entry in entries {
            let entry = entry.map_err(|source| io_error("read directory entry", &directory, source))?;
            let path = entry.path();
            let metadata = fs::symlink_metadata(&path)
                .map_err(|source| io_error("inspect package entry", &path, source))?;
            if metadata.file_type().is_symlink() {
                return Err(SkillPackageError::SymlinkForbidden(path));
            }
            if metadata.is_dir() {
                pending.push((path, depth + 1));
            } else if metadata.is_file() {
                paths.push(path);
                if paths.len() > MAX_PACKAGE_FILES {
                    return Err(SkillPackageError::TooManyFiles);
                }
            } else {
                return Err(SkillPackageError::SpecialFileForbidden(path));
            }
        }
    }

    let mut evidence = Vec::with_capacity(paths.len());
    let mut total = 0_u64;
    for path in paths {
        let relative = path
            .strip_prefix(root)
            .map_err(|_| SkillPackageError::InvalidRelativePath)?;
        let relative_path = normalized_relative_path(relative)?;
        let bytes = read_bounded(&path, MAX_PACKAGE_BYTES)?;
        let byte_len = u64::try_from(bytes.len()).map_err(|_| SkillPackageError::PackageTooLarge)?;
        total = total.checked_add(byte_len).ok_or(SkillPackageError::PackageTooLarge)?;
        if total > MAX_PACKAGE_BYTES {
            return Err(SkillPackageError::PackageTooLarge);
        }
        evidence.push(SkillPackageFileEvidence {
            script_candidate: relative_path == "scripts" || relative_path.starts_with("scripts/"),
            relative_path,
            byte_len,
            content_digest: BindingDigest::new(sha256(&bytes)),
        });
    }
    evidence.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    if evidence
        .windows(2)
        .any(|pair| pair[0].relative_path == pair[1].relative_path)
    {
        return Err(SkillPackageError::DuplicatePath);
    }
    Ok(evidence)
}

fn read_bounded(path: &Path, max_bytes: u64) -> Result<Vec<u8>, SkillPackageError> {
    let mut file = File::open(path).map_err(|source| io_error("open file", path, source))?;
    let mut bytes = Vec::new();
    file.by_ref()
        .take(max_bytes.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|source| io_error("read file", path, source))?;
    if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > max_bytes {
        return Err(if path.file_name().is_some_and(|name| name == "SKILL.md") {
            SkillPackageError::SkillFileTooLarge
        } else {
            SkillPackageError::PackageTooLarge
        });
    }
    Ok(bytes)
}

fn normalized_relative_path(path: &Path) -> Result<String, SkillPackageError> {
    if path.is_absolute() {
        return Err(SkillPackageError::InvalidRelativePath);
    }
    let mut segments = Vec::new();
    for component in path.components() {
        match component {
            std::path::Component::Normal(value) => {
                let segment = value.to_str().ok_or(SkillPackageError::InvalidRelativePath)?;
                if segment.is_empty() || segment == "." || segment == ".." {
                    return Err(SkillPackageError::InvalidRelativePath);
                }
                segments.push(segment);
            }
            _ => return Err(SkillPackageError::InvalidRelativePath),
        }
    }
    let joined = segments.join("/");
    if joined.is_empty() || joined.len() > MAX_RELATIVE_PATH_BYTES {
        return Err(SkillPackageError::InvalidRelativePath);
    }
    Ok(joined)
}

fn parse_manifest<'a>(
    bytes: &'a [u8],
    root: &Path,
) -> Result<(SkillManifest, &'a [u8]), SkillPackageError> {
    let text = std::str::from_utf8(bytes)
        .map_err(|_| SkillPackageError::InvalidManifest("SKILL.md must be UTF-8"))?;
    let mut lines = text.split_inclusive('\n');
    let first = lines
        .next()
        .ok_or(SkillPackageError::InvalidManifest("SKILL.md is empty"))?;
    if first.trim_end_matches(['\r', '\n']) != "---" {
        return Err(SkillPackageError::InvalidManifest("YAML frontmatter must start with ---"));
    }

    let mut offset = first.len();
    let mut frontmatter_lines = Vec::new();
    let mut closed = false;
    for line in lines {
        let trimmed = line.trim_end_matches(['\r', '\n']);
        offset += line.len();
        if trimmed == "---" {
            closed = true;
            break;
        }
        frontmatter_lines.push(trimmed);
    }
    if !closed {
        return Err(SkillPackageError::InvalidManifest("YAML frontmatter is not closed"));
    }
    let body = bytes
        .get(offset..)
        .ok_or(SkillPackageError::InvalidManifest("invalid body offset"))?;
    if body.is_empty() {
        return Err(SkillPackageError::InvalidManifest("instruction body is empty"));
    }

    let mut name = None;
    let mut description = None;
    let mut license = None;
    let mut compatibility = None;
    let mut allowed_tools_request = None;
    let mut metadata = BTreeMap::new();
    let mut in_metadata = false;

    for line in frontmatter_lines {
        if line.trim().is_empty() || line.trim_start().starts_with('#') {
            continue;
        }
        if line.starts_with(' ') || line.starts_with('\t') {
            if !in_metadata {
                return Err(SkillPackageError::InvalidManifest("nested YAML is supported only for metadata"));
            }
            let nested = line.trim();
            let (key, value) = split_scalar(nested)?;
            validate_metadata_entry(key, value)?;
            if metadata.insert(key.to_owned(), value.to_owned()).is_some() {
                return Err(SkillPackageError::InvalidManifest("duplicate metadata key"));
            }
            if metadata.len() > MAX_METADATA_ENTRIES {
                return Err(SkillPackageError::TooManyMetadataEntries);
            }
            continue;
        }

        in_metadata = false;
        let (key, value) = split_scalar(line)?;
        match key {
            "name" => set_once(&mut name, value, "name")?,
            "description" => set_once(&mut description, value, "description")?,
            "license" => set_once(&mut license, value, "license")?,
            "compatibility" => set_once(&mut compatibility, value, "compatibility")?,
            "allowed-tools" => set_once(&mut allowed_tools_request, value, "allowed-tools")?,
            "metadata" => {
                if !value.is_empty() {
                    return Err(SkillPackageError::InvalidManifest("metadata must be a string mapping"));
                }
                in_metadata = true;
            }
            _ => return Err(SkillPackageError::InvalidManifest("unsupported frontmatter field")),
        }
    }

    let name = name.ok_or(SkillPackageError::InvalidManifest("missing name"))?;
    validate_skill_name(&name, root)?;
    let description = description.ok_or(SkillPackageError::InvalidManifest("missing description"))?;
    if description.is_empty() || description.len() > MAX_DESCRIPTION_BYTES {
        return Err(SkillPackageError::DescriptionOutOfBounds);
    }
    if compatibility.as_ref().is_some_and(|value| value.len() > MAX_COMPATIBILITY_BYTES) {
        return Err(SkillPackageError::CompatibilityOutOfBounds);
    }
    for (field, value) in [
        ("license", license.as_ref()),
        ("allowed-tools", allowed_tools_request.as_ref()),
    ] {
        if value.is_some_and(|value| value.len() > MAX_OPTIONAL_SCALAR_BYTES) {
            return Err(SkillPackageError::OptionalScalarOutOfBounds(field));
        }
    }
    let allowed_tools_request_ref = allowed_tools_request
        .as_ref()
        .map(|value| digest_ref(SKILL_ALLOWED_TOOLS_DOMAIN, value.as_bytes()))
        .transpose()?;

    Ok((
        SkillManifest {
            name,
            description,
            license,
            compatibility,
            metadata,
            allowed_tools_request,
            allowed_tools_request_ref,
        },
        body,
    ))
}

fn split_scalar(line: &str) -> Result<(&str, String), SkillPackageError> {
    let (key, raw) = line
        .split_once(':')
        .ok_or(SkillPackageError::InvalidManifest("frontmatter entry must contain ':'"))?;
    let key = key.trim();
    if key.is_empty() {
        return Err(SkillPackageError::InvalidManifest("frontmatter key is empty"));
    }
    let raw = raw.trim();
    let value = if raw.len() >= 2
        && ((raw.starts_with('"') && raw.ends_with('"'))
            || (raw.starts_with('\'') && raw.ends_with('\'')))
    {
        raw[1..raw.len() - 1].to_owned()
    } else {
        raw.to_owned()
    };
    if value.contains(['\0', '\r', '\n']) {
        return Err(SkillPackageError::InvalidManifest("frontmatter scalar contains forbidden control bytes"));
    }
    Ok((key, value))
}

fn set_once(
    slot: &mut Option<String>,
    value: String,
    field: &'static str,
) -> Result<(), SkillPackageError> {
    if slot.replace(value).is_some() {
        return Err(SkillPackageError::InvalidManifest(match field {
            "name" => "duplicate name",
            "description" => "duplicate description",
            "license" => "duplicate license",
            "compatibility" => "duplicate compatibility",
            "allowed-tools" => "duplicate allowed-tools",
            _ => "duplicate scalar",
        }));
    }
    Ok(())
}

fn validate_skill_name(name: &str, root: &Path) -> Result<(), SkillPackageError> {
    if name.is_empty()
        || name.len() > 64
        || name.starts_with('-')
        || name.ends_with('-')
        || name.contains("--")
        || !name
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
    {
        return Err(SkillPackageError::InvalidName);
    }
    let directory_name = root
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or(SkillPackageError::InvalidName)?;
    if directory_name != name {
        return Err(SkillPackageError::InvalidName);
    }
    Ok(())
}

fn validate_metadata_entry(key: &str, value: &str) -> Result<(), SkillPackageError> {
    if key.is_empty()
        || key.len() > MAX_METADATA_KEY_BYTES
        || value.len() > MAX_METADATA_VALUE_BYTES
        || key.bytes().any(|byte| byte.is_ascii_control())
    {
        return Err(SkillPackageError::MetadataOutOfBounds);
    }
    Ok(())
}

fn validate_provenance(values: &[BindingDigest]) -> Result<(), SkillPackageError> {
    if values.is_empty() {
        return Err(SkillPackageError::ProvenanceRequired);
    }
    if values.len() > MAX_PROVENANCE_REFS
        || values.windows(2).any(|pair| pair[0] >= pair[1])
        || values.iter().any(|value| value.bytes() == [0; 32])
    {
        return Err(SkillPackageError::InvalidProvenanceOrder);
    }
    Ok(())
}

fn package_content_digest(files: &[SkillPackageFileEvidence]) -> Result<BindingDigest, SkillPackageError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(SKILL_CONTENT_DOMAIN)?;
    encoder.push_u64(u64::try_from(files.len()).map_err(|_| SkillPackageError::TooManyFiles)?);
    for file in files {
        encoder.push_bytes(file.relative_path.as_bytes())?;
        encoder.push_u64(file.byte_len);
        encoder.push_bytes(&file.content_digest.bytes())?;
        encoder.push_u8(u8::from(file.script_candidate));
    }
    Ok(BindingDigest::new(sha256(&encoder.finish())))
}

fn package_ref(
    name: &str,
    version: &SkillVersion,
    provenance_refs: &[BindingDigest],
) -> Result<BindingDigest, SkillPackageError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(SKILL_PACKAGE_DOMAIN)?;
    encoder.push_bytes(name.as_bytes())?;
    encoder.push_bytes(version.as_str().as_bytes())?;
    encoder.push_u64(u64::try_from(provenance_refs.len()).map_err(|_| SkillPackageError::InvalidProvenanceOrder)?);
    for provenance in provenance_refs {
        encoder.push_bytes(&provenance.bytes())?;
    }
    Ok(BindingDigest::new(sha256(&encoder.finish())))
}

fn skill_state_ref(
    package_ref: BindingDigest,
    version: &SkillVersion,
    content_digest: BindingDigest,
    capability_mapping_ref: BindingDigest,
    state: SkillAdmissionState,
) -> Result<BindingDigest, SkillPackageError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(SKILL_STATE_DOMAIN)?;
    encoder.push_bytes(&package_ref.bytes())?;
    encoder.push_bytes(version.as_str().as_bytes())?;
    encoder.push_bytes(&content_digest.bytes())?;
    encoder.push_bytes(&capability_mapping_ref.bytes())?;
    encoder.push_u8(skill_state_code(state));
    Ok(BindingDigest::new(sha256(&encoder.finish())))
}

fn digest_ref(domain: &[u8], bytes: &[u8]) -> Result<BindingDigest, SkillPackageError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(domain)?;
    encoder.push_bytes(bytes)?;
    Ok(BindingDigest::new(sha256(&encoder.finish())))
}

const fn skill_state_code(state: SkillAdmissionState) -> u8 {
    match state {
        SkillAdmissionState::Discovered => 1,
        SkillAdmissionState::ProvenanceRecorded => 2,
        SkillAdmissionState::Reviewed => 3,
        SkillAdmissionState::InstructionAdmitted => 4,
        SkillAdmissionState::ExecutableAdmitted => 5,
        SkillAdmissionState::LockedVersion => 6,
        SkillAdmissionState::Deprecated => 7,
        SkillAdmissionState::Revoked => 8,
        SkillAdmissionState::Unknown => 9,
    }
}

const fn allowed_transition(from: SkillAdmissionState, to: SkillAdmissionState) -> bool {
    use SkillAdmissionState::*;
    matches!(
        (from, to),
        (Discovered, ProvenanceRecorded | Reviewed | Deprecated | Revoked | Unknown)
            | (ProvenanceRecorded, Reviewed | Deprecated | Revoked | Unknown)
            | (Reviewed, InstructionAdmitted | ExecutableAdmitted | Deprecated | Revoked | Unknown)
            | (InstructionAdmitted, ExecutableAdmitted | LockedVersion | Deprecated | Revoked | Unknown)
            | (ExecutableAdmitted, LockedVersion | Deprecated | Revoked | Unknown)
            | (LockedVersion, Deprecated | Revoked | Unknown)
            | (Deprecated, Revoked | Unknown)
    )
}

fn require_nonzero_ref(value: BindingDigest, field: &'static str) -> Result<(), SkillPackageError> {
    if value.bytes() == [0; 32] {
        return Err(SkillPackageError::ZeroBindingRef(field));
    }
    Ok(())
}

fn io_error(operation: &'static str, path: &Path, source: std::io::Error) -> SkillPackageError {
    SkillPackageError::Io {
        operation,
        path: path.to_owned(),
        source,
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU64, Ordering};

    use super::*;

    static NEXT_DIR: AtomicU64 = AtomicU64::new(1);

    fn digest(value: u8) -> BindingDigest {
        BindingDigest::new([value; 32])
    }

    struct TempSkill {
        root: PathBuf,
    }

    impl TempSkill {
        fn new(name: &str, body: &str, extra: &[(&str, &[u8])]) -> Self {
            let id = NEXT_DIR.fetch_add(1, Ordering::Relaxed);
            let root = std::env::temp_dir()
                .join(format!("golam-skill-test-{}-{id}", std::process::id()))
                .join(name);
            fs::create_dir_all(&root).unwrap();
            let manifest = format!(
                "---\nname: {name}\ndescription: Use this skill for bounded test work.\nlicense: MIT\ncompatibility: local only\nmetadata:\n  author: golam-test\n  version: 1.0.0\nallowed-tools: Bash(git:*) Read\n---\n{body}\n"
            );
            fs::write(root.join("SKILL.md"), manifest).unwrap();
            for (relative, bytes) in extra {
                let path = root.join(relative);
                if let Some(parent) = path.parent() {
                    fs::create_dir_all(parent).unwrap();
                }
                fs::write(path, bytes).unwrap();
            }
            Self { root }
        }
    }

    impl Drop for TempSkill {
        fn drop(&mut self) {
            let parent = self.root.parent().unwrap_or(&self.root).to_owned();
            let _ = fs::remove_dir_all(parent);
        }
    }

    fn discover(skill: &TempSkill) -> ReviewedInstructionSkill {
        discover_reviewed_instruction_skill(SkillDiscoveryRequest {
            package_root: &skill.root,
            version: SkillVersion::new("1.0.0").unwrap(),
            provenance_refs: vec![digest(1), digest(2)],
            reviewed_capability_mapping_ref: digest(3),
        })
        .unwrap()
    }

    #[test]
    fn discovers_agent_skill_without_granting_allowed_tools_authority() {
        let skill = TempSkill::new("repo-check", "# Repo Check\nRead the repository.", &[]);
        let reviewed = discover(&skill);
        assert_eq!(reviewed.descriptor().admission_state, SkillAdmissionState::InstructionAdmitted);
        assert!(reviewed.descriptor().requested_capability_classes.is_empty());
        assert_eq!(reviewed.descriptor().network_posture, ToolNetworkPosture::Denied);
        assert_eq!(reviewed.instructions().as_bytes(), b"# Repo Check\nRead the repository.\n");
        assert!(reviewed.descriptor().script_refs.is_empty());
    }

    #[test]
    fn script_presence_is_recorded_but_does_not_admit_executable_dispatch() {
        let skill = TempSkill::new(
            "scripted-skill",
            "# Scripted Skill\nInstructions remain usable.",
            &[("scripts/run.sh", b"#!/bin/sh\nexit 0\n")],
        );
        let reviewed = discover(&skill);
        assert_eq!(reviewed.descriptor().script_refs.len(), 1);
        let lifecycle = SkillLifecycle::new(reviewed);
        assert!(matches!(
            lifecycle.bind_executable_dispatch(digest(10), digest(11), digest(12)),
            Err(SkillPackageError::ExecutableSkillNotAdmitted)
        ));
    }

    #[test]
    fn activation_revalidates_live_content_and_lifecycle() {
        let skill = TempSkill::new("live-skill", "# Live Skill\nOriginal instructions.", &[]);
        let reviewed = discover(&skill);
        let mut lifecycle = SkillLifecycle::new(reviewed);
        let binding = lifecycle
            .bind_instruction_activation(digest(10), digest(11), digest(12))
            .unwrap();
        assert!(lifecycle.activate_instructions(&binding).is_ok());

        let changed = "---\nname: live-skill\ndescription: Use this skill for bounded test work.\n---\n# Live Skill\nChanged instructions.\n";
        fs::write(skill.root.join("SKILL.md"), changed).unwrap();
        assert!(matches!(
            lifecycle.activate_instructions(&binding),
            Err(SkillPackageError::LivePackageChanged)
        ));

        fs::write(
            skill.root.join("SKILL.md"),
            "---\nname: live-skill\ndescription: Use this skill for bounded test work.\nlicense: MIT\ncompatibility: local only\nmetadata:\n  author: golam-test\n  version: 1.0.0\nallowed-tools: Bash(git:*) Read\n---\n# Live Skill\nOriginal instructions.\n",
        )
        .unwrap();
        lifecycle.transition(SkillAdmissionState::Revoked).unwrap();
        assert!(matches!(
            lifecycle.activate_instructions(&binding),
            Err(SkillPackageError::Dispatch(
                DispatchValidationError::SkillAdmissionStateMismatch
                    | DispatchValidationError::SkillLifecycleNotDispatchable
            ))
        ));
    }

    #[test]
    fn rejects_symlinks_wrong_directory_name_and_unreviewed_provenance() {
        let skill = TempSkill::new("safe-skill", "# Safe Skill\nSafe.", &[]);
        assert!(matches!(
            discover_reviewed_instruction_skill(SkillDiscoveryRequest {
                package_root: &skill.root,
                version: SkillVersion::new("1.0.0").unwrap(),
                provenance_refs: vec![],
                reviewed_capability_mapping_ref: digest(3),
            }),
            Err(SkillPackageError::ProvenanceRequired)
        ));

        fs::write(
            skill.root.join("SKILL.md"),
            "---\nname: wrong-name\ndescription: mismatch\n---\n# Wrong\n",
        )
        .unwrap();
        assert!(matches!(
            discover_reviewed_instruction_skill(SkillDiscoveryRequest {
                package_root: &skill.root,
                version: SkillVersion::new("1.0.0").unwrap(),
                provenance_refs: vec![digest(1)],
                reviewed_capability_mapping_ref: digest(3),
            }),
            Err(SkillPackageError::InvalidName)
        ));
    }

    #[test]
    fn lifecycle_is_forward_only_and_old_binding_dies_on_state_change() {
        let skill = TempSkill::new("lifecycle-skill", "# Lifecycle Skill\nDo work.", &[]);
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
}
