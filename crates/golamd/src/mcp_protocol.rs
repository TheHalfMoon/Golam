#![forbid(unsafe_code)]

//! Bounded MCP advertisement normalization and reviewed binding lifecycle for Spec 005.
//!
//! JSON and server-advertised metadata are untrusted input. This module does not perform network
//! I/O, launch a process, mint a capability, approve an action, or expose privileged Kernel state.

use std::error::Error;
use std::fmt;

use golam_core::digest::sha256;
use golam_core::skills_protocol::{
    CurrentMcpDispatchState, DispatchValidationError, ExternalToolDescriptor, McpDispatchBinding,
    McpLifecycleState, McpServerBinding, McpTransport, McpVersionLock, ProtocolFeatureId,
    ProtocolValidationError,
};
use golam_core::taint::{TaintLabel, TaintSet};
use golam_core::tool_request::BindingDigest;
use golam_core::{CanonicalEncoder, CoreError};
use serde_json::Value;

const MAX_MCP_JSON_BYTES: usize = 256 * 1024;
const MAX_MCP_JSON_DEPTH: usize = 16;
const MAX_MCP_JSON_NODES: usize = 2048;
const MAX_MCP_CONTAINER_ITEMS: usize = 128;
const MAX_MCP_STRING_BYTES: usize = 16 * 1024;
const MAX_MCP_NAME_BYTES: usize = 128;
const MAX_MCP_DESCRIPTION_BYTES: usize = 4096;
const MAX_MCP_URI_BYTES: usize = 4096;
const MCP_BINDING_DOMAIN: &[u8] = b"golam:mcp-server-binding:v1";
const MCP_LIFECYCLE_DOMAIN: &[u8] = b"golam:mcp-lifecycle:v1";
const MCP_JSON_DOMAIN: &[u8] = b"golam:mcp-json:v1";
const MCP_TOOL_IDENTITY_DOMAIN: &[u8] = b"golam:mcp-tool-identity:v1";
const MCP_MISSING_SCHEMA_DOMAIN: &[u8] = b"golam:mcp-missing-schema:v1";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum McpAdvertisementKind {
    Tool,
    Resource,
    Prompt,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct McpReviewRequest {
    pub binding_id: BindingDigest,
    pub server_identity: BindingDigest,
    pub transport: McpTransport,
    pub process_profile_ref_or_remote_endpoint: BindingDigest,
    pub allowed_protocol_features: Vec<ProtocolFeatureId>,
    pub golam_local_mapping_ref: BindingDigest,
    pub golam_local_mapping_digest: BindingDigest,
    pub network_policy_ref: BindingDigest,
    pub secret_policy_ref: BindingDigest,
    pub taint_class: TaintSet,
    pub version_lock: McpVersionLock,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NormalizedMcpAdvertisement {
    pub kind: McpAdvertisementKind,
    pub name: String,
    pub description: Option<String>,
    pub resource_uri: Option<String>,
    pub input_schema_ref: Option<BindingDigest>,
    pub output_schema_ref: Option<BindingDigest>,
    pub protocol_payload_ref: BindingDigest,
    pub binding_state: CurrentMcpDispatchState,
    pub taint_class: TaintSet,
}

impl NormalizedMcpAdvertisement {
    pub fn external_tool_descriptor(&self) -> Result<ExternalToolDescriptor, McpProtocolError> {
        if self.kind != McpAdvertisementKind::Tool {
            return Err(McpProtocolError::WrongAdvertisementKind);
        }
        let input_schema_digest = self
            .input_schema_ref
            .ok_or(McpProtocolError::MissingField("inputSchema"))?;
        let output_schema_digest = self
            .output_schema_ref
            .unwrap_or(BindingDigest::new(sha256(MCP_MISSING_SCHEMA_DOMAIN)));
        let mut encoder = CanonicalEncoder::new();
        encoder.push_bytes(MCP_TOOL_IDENTITY_DOMAIN)?;
        encoder.push_bytes(&self.binding_state.binding_id.bytes())?;
        encoder.push_bytes(&self.binding_state.binding_digest.bytes())?;
        encoder.push_bytes(self.name.as_bytes())?;
        encoder.push_bytes(&self.protocol_payload_ref.bytes())?;
        Ok(ExternalToolDescriptor {
            server_tool_identity: BindingDigest::new(sha256(&encoder.finish())),
            input_schema_digest,
            output_schema_digest,
            golam_local_mapping_ref: self.binding_state.golam_local_mapping_ref,
            golam_local_mapping_digest: self.binding_state.golam_local_mapping_digest,
            taint_class: self.taint_class,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct McpLifecycle {
    binding: McpServerBinding,
    lifecycle_state_ref: BindingDigest,
}

impl McpLifecycle {
    pub fn review(request: McpReviewRequest) -> Result<Self, McpProtocolError> {
        require_nonzero(request.binding_id, "binding_id")?;
        require_nonzero(request.server_identity, "server_identity")?;
        require_nonzero(
            request.process_profile_ref_or_remote_endpoint,
            "process_profile_ref_or_remote_endpoint",
        )?;
        require_nonzero(request.golam_local_mapping_ref, "golam_local_mapping_ref")?;
        require_nonzero(
            request.golam_local_mapping_digest,
            "golam_local_mapping_digest",
        )?;
        require_nonzero(request.secret_policy_ref, "secret_policy_ref")?;
        if request.transport == McpTransport::RemoteHttp {
            require_nonzero(request.network_policy_ref, "network_policy_ref")?;
        }

        let taint_class = request
            .taint_class
            .union(TaintSet::from_labels([TaintLabel::McpUntrusted]));
        let binding_digest = mcp_binding_digest(&request, taint_class)?;
        let mut binding = McpServerBinding {
            binding_id: request.binding_id,
            binding_digest,
            server_identity: request.server_identity,
            transport: request.transport,
            process_profile_ref_or_remote_endpoint: request.process_profile_ref_or_remote_endpoint,
            allowed_protocol_features: request.allowed_protocol_features,
            golam_local_mapping_ref: request.golam_local_mapping_ref,
            golam_local_mapping_digest: request.golam_local_mapping_digest,
            network_policy_ref: request.network_policy_ref,
            secret_policy_ref: request.secret_policy_ref,
            taint_class,
            version_lock: request.version_lock,
            lifecycle_state: McpLifecycleState::Reviewed,
        };
        binding.validate()?;
        let lifecycle_state_ref = mcp_lifecycle_state_ref(&binding, binding.lifecycle_state)?;
        // Freeze exactly the validated reviewed state.
        binding.lifecycle_state = McpLifecycleState::Reviewed;
        Ok(Self {
            binding,
            lifecycle_state_ref,
        })
    }

    pub fn binding(&self) -> &McpServerBinding {
        &self.binding
    }

    pub const fn state(&self) -> McpLifecycleState {
        self.binding.lifecycle_state
    }

    pub const fn lifecycle_state_ref(&self) -> BindingDigest {
        self.lifecycle_state_ref
    }

    pub fn transition(&mut self, next: McpLifecycleState) -> Result<(), McpProtocolError> {
        if !allowed_transition(self.binding.lifecycle_state, next) {
            return Err(McpProtocolError::InvalidLifecycleTransition {
                from: self.binding.lifecycle_state,
                to: next,
            });
        }
        self.binding.lifecycle_state = next;
        self.lifecycle_state_ref = mcp_lifecycle_state_ref(&self.binding, next)?;
        Ok(())
    }

    pub fn current_state(&self) -> CurrentMcpDispatchState {
        CurrentMcpDispatchState {
            binding_id: self.binding.binding_id,
            binding_digest: self.binding.binding_digest,
            version_lock: self.binding.version_lock.clone(),
            golam_local_mapping_ref: self.binding.golam_local_mapping_ref,
            golam_local_mapping_digest: self.binding.golam_local_mapping_digest,
            lifecycle_state: self.binding.lifecycle_state,
            lifecycle_state_ref: self.lifecycle_state_ref,
        }
    }

    pub fn bind_dispatch(
        &self,
        queued_request_ref: BindingDigest,
        capability_decision_ref: BindingDigest,
        approval_decision_ref: BindingDigest,
    ) -> Result<McpDispatchBinding, McpProtocolError> {
        if self.binding.lifecycle_state != McpLifecycleState::Active {
            return Err(McpProtocolError::LifecycleNotDispatchable(
                self.binding.lifecycle_state,
            ));
        }
        require_nonzero(queued_request_ref, "queued_request_ref")?;
        require_nonzero(capability_decision_ref, "capability_decision_ref")?;
        require_nonzero(approval_decision_ref, "approval_decision_ref")?;
        Ok(McpDispatchBinding {
            binding_id: self.binding.binding_id,
            binding_digest: self.binding.binding_digest,
            version_lock: self.binding.version_lock.clone(),
            golam_local_mapping_ref: self.binding.golam_local_mapping_ref,
            golam_local_mapping_digest: self.binding.golam_local_mapping_digest,
            lifecycle_state_ref: self.lifecycle_state_ref,
            queued_request_ref,
            capability_decision_ref,
            approval_decision_ref,
        })
    }

    pub fn revalidate_dispatch(
        &self,
        dispatch: &McpDispatchBinding,
    ) -> Result<(), McpProtocolError> {
        dispatch.revalidate(&self.current_state())?;
        Ok(())
    }

    pub fn normalize_advertisement(
        &self,
        kind: McpAdvertisementKind,
        bytes: &[u8],
    ) -> Result<NormalizedMcpAdvertisement, McpProtocolError> {
        if !matches!(
            self.binding.lifecycle_state,
            McpLifecycleState::Reviewed | McpLifecycleState::Active
        ) {
            return Err(McpProtocolError::LifecycleNotDispatchable(
                self.binding.lifecycle_state,
            ));
        }
        normalize_advertisement(self, kind, bytes)
    }
}

#[derive(Debug)]
pub enum McpProtocolError {
    Core(CoreError),
    Protocol(ProtocolValidationError),
    Dispatch(DispatchValidationError),
    Json(serde_json::Error),
    InputTooLarge,
    PayloadTooDeep,
    TooManyNodes,
    ContainerTooLarge,
    StringTooLarge,
    ExpectedObject,
    MissingField(&'static str),
    InvalidFieldType(&'static str),
    UnsupportedField(String),
    AuthorityMetadataForbidden(String),
    InvalidName,
    InvalidDescription,
    InvalidUri,
    InvalidBinding(&'static str),
    WrongAdvertisementKind,
    LifecycleNotDispatchable(McpLifecycleState),
    InvalidLifecycleTransition {
        from: McpLifecycleState,
        to: McpLifecycleState,
    },
}

impl fmt::Display for McpProtocolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Core(error) => write!(f, "MCP canonical encoding failed: {error}"),
            Self::Protocol(error) => write!(f, "MCP binding validation failed: {error}"),
            Self::Dispatch(error) => write!(f, "MCP dispatch revalidation failed: {error}"),
            Self::Json(error) => write!(f, "MCP JSON parse failed: {error}"),
            Self::InputTooLarge => f.write_str("MCP JSON input exceeds the byte bound"),
            Self::PayloadTooDeep => f.write_str("MCP JSON nesting exceeds the depth bound"),
            Self::TooManyNodes => f.write_str("MCP JSON node count exceeds the bound"),
            Self::ContainerTooLarge => f.write_str("MCP JSON container exceeds the item bound"),
            Self::StringTooLarge => f.write_str("MCP JSON string exceeds the byte bound"),
            Self::ExpectedObject => f.write_str("MCP advertisement must be a JSON object"),
            Self::MissingField(field) => write!(f, "MCP advertisement is missing field: {field}"),
            Self::InvalidFieldType(field) => {
                write!(f, "MCP advertisement field has invalid type: {field}")
            }
            Self::UnsupportedField(field) => {
                write!(f, "unsupported MCP advertisement field: {field}")
            }
            Self::AuthorityMetadataForbidden(field) => {
                write!(f, "MCP authority-like metadata is forbidden: {field}")
            }
            Self::InvalidName => f.write_str("MCP advertisement name is invalid"),
            Self::InvalidDescription => f.write_str("MCP description exceeds the bound"),
            Self::InvalidUri => f.write_str("MCP resource URI is invalid or oversized"),
            Self::InvalidBinding(field) => write!(f, "MCP binding reference is invalid: {field}"),
            Self::WrongAdvertisementKind => {
                f.write_str("MCP advertisement is not a tool descriptor")
            }
            Self::LifecycleNotDispatchable(state) => write!(
                f,
                "MCP lifecycle is not usable for this operation: {state:?}"
            ),
            Self::InvalidLifecycleTransition { from, to } => {
                write!(f, "invalid MCP lifecycle transition: {from:?} -> {to:?}")
            }
        }
    }
}

impl Error for McpProtocolError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Core(error) => Some(error),
            Self::Protocol(error) => Some(error),
            Self::Dispatch(error) => Some(error),
            Self::Json(error) => Some(error),
            _ => None,
        }
    }
}

impl From<CoreError> for McpProtocolError {
    fn from(value: CoreError) -> Self {
        Self::Core(value)
    }
}
impl From<ProtocolValidationError> for McpProtocolError {
    fn from(value: ProtocolValidationError) -> Self {
        Self::Protocol(value)
    }
}
impl From<DispatchValidationError> for McpProtocolError {
    fn from(value: DispatchValidationError) -> Self {
        Self::Dispatch(value)
    }
}
impl From<serde_json::Error> for McpProtocolError {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value)
    }
}

fn normalize_advertisement(
    lifecycle: &McpLifecycle,
    kind: McpAdvertisementKind,
    bytes: &[u8],
) -> Result<NormalizedMcpAdvertisement, McpProtocolError> {
    if bytes.len() > MAX_MCP_JSON_BYTES {
        return Err(McpProtocolError::InputTooLarge);
    }
    let value: Value = serde_json::from_slice(bytes)?;
    let mut nodes = 0_usize;
    validate_json_shape(&value, 0, &mut nodes)?;
    let object = value.as_object().ok_or(McpProtocolError::ExpectedObject)?;
    reject_authority_metadata(object.keys().map(String::as_str))?;
    let allowed = match kind {
        McpAdvertisementKind::Tool => &[
            "name",
            "description",
            "inputSchema",
            "outputSchema",
            "annotations",
            "title",
            "_meta",
        ][..],
        McpAdvertisementKind::Resource => &[
            "uri",
            "name",
            "description",
            "mimeType",
            "annotations",
            "title",
            "_meta",
        ][..],
        McpAdvertisementKind::Prompt => &["name", "description", "arguments", "title", "_meta"][..],
    };
    for key in object.keys() {
        if !allowed.contains(&key.as_str()) {
            return Err(McpProtocolError::UnsupportedField(key.clone()));
        }
    }

    let name = required_string(object.get("name"), "name")?.to_owned();
    validate_name(&name)?;
    let description = optional_string(object.get("description"), "description")?.map(str::to_owned);
    if description
        .as_ref()
        .is_some_and(|value| value.len() > MAX_MCP_DESCRIPTION_BYTES)
    {
        return Err(McpProtocolError::InvalidDescription);
    }
    let resource_uri = if kind == McpAdvertisementKind::Resource {
        let uri = required_string(object.get("uri"), "uri")?.to_owned();
        if uri.is_empty() || uri.len() > MAX_MCP_URI_BYTES || uri.chars().any(char::is_control) {
            return Err(McpProtocolError::InvalidUri);
        }
        Some(uri)
    } else {
        None
    };

    let input_schema_ref = match kind {
        McpAdvertisementKind::Tool => {
            let schema = object
                .get("inputSchema")
                .ok_or(McpProtocolError::MissingField("inputSchema"))?;
            if !schema.is_object() {
                return Err(McpProtocolError::InvalidFieldType("inputSchema"));
            }
            Some(BindingDigest::new(canonical_json_digest(schema)?))
        }
        _ => None,
    };
    let output_schema_ref = match object.get("outputSchema") {
        Some(schema) if kind == McpAdvertisementKind::Tool && schema.is_object() => {
            Some(BindingDigest::new(canonical_json_digest(schema)?))
        }
        Some(_) if kind == McpAdvertisementKind::Tool => {
            return Err(McpProtocolError::InvalidFieldType("outputSchema"));
        }
        Some(_) => {
            return Err(McpProtocolError::UnsupportedField(
                "outputSchema".to_owned(),
            ));
        }
        None => None,
    };

    if kind == McpAdvertisementKind::Prompt
        && let Some(arguments) = object.get("arguments")
    {
        let arguments = arguments
            .as_array()
            .ok_or(McpProtocolError::InvalidFieldType("arguments"))?;
        if arguments.len() > MAX_MCP_CONTAINER_ITEMS
            || arguments.iter().any(|value| !value.is_object())
        {
            return Err(McpProtocolError::ContainerTooLarge);
        }
    }

    let protocol_payload_ref = BindingDigest::new(canonical_json_digest(&value)?);
    Ok(NormalizedMcpAdvertisement {
        kind,
        name,
        description,
        resource_uri,
        input_schema_ref,
        output_schema_ref,
        protocol_payload_ref,
        binding_state: lifecycle.current_state(),
        taint_class: lifecycle.binding.taint_class,
    })
}

fn validate_json_shape(
    value: &Value,
    depth: usize,
    nodes: &mut usize,
) -> Result<(), McpProtocolError> {
    if depth > MAX_MCP_JSON_DEPTH {
        return Err(McpProtocolError::PayloadTooDeep);
    }
    *nodes = nodes.checked_add(1).ok_or(McpProtocolError::TooManyNodes)?;
    if *nodes > MAX_MCP_JSON_NODES {
        return Err(McpProtocolError::TooManyNodes);
    }
    match value {
        Value::String(value) => {
            if value.len() > MAX_MCP_STRING_BYTES {
                return Err(McpProtocolError::StringTooLarge);
            }
        }
        Value::Array(values) => {
            if values.len() > MAX_MCP_CONTAINER_ITEMS {
                return Err(McpProtocolError::ContainerTooLarge);
            }
            for value in values {
                validate_json_shape(value, depth + 1, nodes)?;
            }
        }
        Value::Object(values) => {
            if values.len() > MAX_MCP_CONTAINER_ITEMS {
                return Err(McpProtocolError::ContainerTooLarge);
            }
            for (key, value) in values {
                if key.len() > MAX_MCP_STRING_BYTES {
                    return Err(McpProtocolError::StringTooLarge);
                }
                validate_json_shape(value, depth + 1, nodes)?;
            }
        }
        Value::Null | Value::Bool(_) | Value::Number(_) => {}
    }
    Ok(())
}

fn canonical_json_digest(value: &Value) -> Result<[u8; 32], McpProtocolError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(MCP_JSON_DOMAIN)?;
    encode_json(&mut encoder, value)?;
    Ok(sha256(&encoder.finish()))
}

fn encode_json(encoder: &mut CanonicalEncoder, value: &Value) -> Result<(), McpProtocolError> {
    match value {
        Value::Null => encoder.push_u8(0),
        Value::Bool(false) => encoder.push_u8(1),
        Value::Bool(true) => encoder.push_u8(2),
        Value::Number(number) => {
            encoder.push_u8(3);
            encoder.push_bytes(number.to_string().as_bytes())?;
        }
        Value::String(value) => {
            encoder.push_u8(4);
            encoder.push_bytes(value.as_bytes())?;
        }
        Value::Array(values) => {
            encoder.push_u8(5);
            encoder.push_u64(
                u64::try_from(values.len()).map_err(|_| McpProtocolError::ContainerTooLarge)?,
            );
            for value in values {
                encode_json(encoder, value)?;
            }
        }
        Value::Object(values) => {
            encoder.push_u8(6);
            let mut entries = values.iter().collect::<Vec<_>>();
            entries.sort_by(|left, right| left.0.cmp(right.0));
            encoder.push_u64(
                u64::try_from(entries.len()).map_err(|_| McpProtocolError::ContainerTooLarge)?,
            );
            for (key, value) in entries {
                encoder.push_bytes(key.as_bytes())?;
                encode_json(encoder, value)?;
            }
        }
    }
    Ok(())
}

fn mcp_binding_digest(
    request: &McpReviewRequest,
    taint_class: TaintSet,
) -> Result<BindingDigest, McpProtocolError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(MCP_BINDING_DOMAIN)?;
    encoder.push_bytes(&request.binding_id.bytes())?;
    encoder.push_bytes(&request.server_identity.bytes())?;
    encoder.push_u8(match request.transport {
        McpTransport::LocalStdio => 1,
        McpTransport::RemoteHttp => 2,
    });
    encoder.push_bytes(&request.process_profile_ref_or_remote_endpoint.bytes())?;
    encoder.push_u64(
        u64::try_from(request.allowed_protocol_features.len())
            .map_err(|_| McpProtocolError::InvalidBinding("allowed_protocol_features"))?,
    );
    for feature in &request.allowed_protocol_features {
        encoder.push_bytes(&(feature.0).bytes())?;
    }
    encoder.push_bytes(&request.golam_local_mapping_ref.bytes())?;
    encoder.push_bytes(&request.golam_local_mapping_digest.bytes())?;
    encoder.push_bytes(&request.network_policy_ref.bytes())?;
    encoder.push_bytes(&request.secret_policy_ref.bytes())?;
    encoder.push_bytes(&taint_class.canonical_bytes()?)?;
    encoder.push_bytes(request.version_lock.as_str().as_bytes())?;
    Ok(BindingDigest::new(sha256(&encoder.finish())))
}

fn mcp_lifecycle_state_ref(
    binding: &McpServerBinding,
    state: McpLifecycleState,
) -> Result<BindingDigest, McpProtocolError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(MCP_LIFECYCLE_DOMAIN)?;
    encoder.push_bytes(&binding.binding_id.bytes())?;
    encoder.push_bytes(&binding.binding_digest.bytes())?;
    encoder.push_bytes(binding.version_lock.as_str().as_bytes())?;
    encoder.push_bytes(&binding.golam_local_mapping_ref.bytes())?;
    encoder.push_bytes(&binding.golam_local_mapping_digest.bytes())?;
    encoder.push_u8(match state {
        McpLifecycleState::Reviewed => 1,
        McpLifecycleState::Active => 2,
        McpLifecycleState::Deprecated => 3,
        McpLifecycleState::Revoked => 4,
        McpLifecycleState::Replaced => 5,
        McpLifecycleState::Unknown => 6,
    });
    Ok(BindingDigest::new(sha256(&encoder.finish())))
}

fn reject_authority_metadata<'a>(
    keys: impl Iterator<Item = &'a str>,
) -> Result<(), McpProtocolError> {
    const FORBIDDEN: &[&str] = &[
        "approval",
        "approved",
        "authority",
        "capability",
        "capabilities",
        "effect",
        "network_policy_ref",
        "secret_policy_ref",
        "skipApproval",
        "taint",
        "trust",
    ];
    for key in keys {
        if FORBIDDEN.contains(&key) {
            return Err(McpProtocolError::AuthorityMetadataForbidden(key.to_owned()));
        }
    }
    Ok(())
}

fn required_string<'a>(
    value: Option<&'a Value>,
    field: &'static str,
) -> Result<&'a str, McpProtocolError> {
    value
        .ok_or(McpProtocolError::MissingField(field))?
        .as_str()
        .ok_or(McpProtocolError::InvalidFieldType(field))
}

fn optional_string<'a>(
    value: Option<&'a Value>,
    field: &'static str,
) -> Result<Option<&'a str>, McpProtocolError> {
    value
        .map(|value| {
            value
                .as_str()
                .ok_or(McpProtocolError::InvalidFieldType(field))
        })
        .transpose()
}

fn validate_name(name: &str) -> Result<(), McpProtocolError> {
    if name.is_empty()
        || name.len() > MAX_MCP_NAME_BYTES
        || name.chars().any(char::is_control)
        || name.bytes().any(|byte| byte.is_ascii_whitespace())
    {
        return Err(McpProtocolError::InvalidName);
    }
    Ok(())
}

fn require_nonzero(value: BindingDigest, field: &'static str) -> Result<(), McpProtocolError> {
    if value.bytes() == [0; 32] {
        return Err(McpProtocolError::InvalidBinding(field));
    }
    Ok(())
}

const fn allowed_transition(from: McpLifecycleState, to: McpLifecycleState) -> bool {
    use McpLifecycleState::*;
    matches!(
        (from, to),
        (Reviewed, Active | Deprecated | Revoked | Replaced | Unknown)
            | (Active, Deprecated | Revoked | Replaced | Unknown)
            | (Deprecated, Revoked | Replaced | Unknown)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn digest(value: u8) -> BindingDigest {
        BindingDigest::new([value; 32])
    }

    fn lifecycle(transport: McpTransport) -> McpLifecycle {
        McpLifecycle::review(McpReviewRequest {
            binding_id: digest(1),
            server_identity: digest(2),
            transport,
            process_profile_ref_or_remote_endpoint: digest(3),
            allowed_protocol_features: vec![ProtocolFeatureId(digest(4))],
            golam_local_mapping_ref: digest(5),
            golam_local_mapping_digest: digest(6),
            network_policy_ref: digest(7),
            secret_policy_ref: digest(8),
            taint_class: TaintSet::empty(),
            version_lock: McpVersionLock::new("2025-06-18").unwrap(),
        })
        .unwrap()
    }

    #[test]
    fn reviewed_binding_forces_mcp_taint_and_active_dispatch_revalidates() {
        let mut lifecycle = lifecycle(McpTransport::LocalStdio);
        assert!(
            lifecycle
                .binding()
                .taint_class
                .contains(TaintLabel::McpUntrusted)
        );
        assert!(matches!(
            lifecycle.bind_dispatch(digest(10), digest(11), digest(12)),
            Err(McpProtocolError::LifecycleNotDispatchable(
                McpLifecycleState::Reviewed
            ))
        ));
        lifecycle.transition(McpLifecycleState::Active).unwrap();
        let dispatch = lifecycle
            .bind_dispatch(digest(10), digest(11), digest(12))
            .unwrap();
        lifecycle.revalidate_dispatch(&dispatch).unwrap();
        lifecycle.transition(McpLifecycleState::Revoked).unwrap();
        assert!(matches!(
            lifecycle.revalidate_dispatch(&dispatch),
            Err(McpProtocolError::Dispatch(
                DispatchValidationError::McpLifecycleStateMismatch
            )) | Err(McpProtocolError::Dispatch(
                DispatchValidationError::McpLifecycleNotDispatchable
            ))
        ));
    }

    #[test]
    fn tool_normalization_is_mapping_bound_and_authority_neutral() {
        let mut lifecycle = lifecycle(McpTransport::LocalStdio);
        lifecycle.transition(McpLifecycleState::Active).unwrap();
        let normalized = lifecycle
            .normalize_advertisement(
                McpAdvertisementKind::Tool,
                br#"{"name":"repo.read","description":"Read repository state","inputSchema":{"type":"object","properties":{"path":{"type":"string"}}},"annotations":{"readOnlyHint":true}}"#,
            )
            .unwrap();
        let descriptor = normalized.external_tool_descriptor().unwrap();
        assert_eq!(descriptor.golam_local_mapping_ref, digest(5));
        assert_eq!(descriptor.golam_local_mapping_digest, digest(6));
        assert!(descriptor.taint_class.contains(TaintLabel::McpUntrusted));
    }

    #[test]
    fn authority_like_metadata_and_unknown_fields_fail_closed() {
        let lifecycle = lifecycle(McpTransport::LocalStdio);
        assert!(matches!(
            lifecycle.normalize_advertisement(
                McpAdvertisementKind::Tool,
                br#"{"name":"evil","inputSchema":{},"skipApproval":true}"#,
            ),
            Err(McpProtocolError::AuthorityMetadataForbidden(_))
        ));
        assert!(matches!(
            lifecycle.normalize_advertisement(
                McpAdvertisementKind::Tool,
                br#"{"name":"evil","inputSchema":{},"shell":"/bin/sh"}"#,
            ),
            Err(McpProtocolError::UnsupportedField(_))
        ));
    }

    #[test]
    fn deep_and_wide_json_fail_before_normalization() {
        let lifecycle = lifecycle(McpTransport::LocalStdio);
        let mut deep = String::from("{\"name\":\"deep\",\"inputSchema\":");
        for _ in 0..18 {
            deep.push_str("{\"x\":");
        }
        deep.push('0');
        for _ in 0..18 {
            deep.push('}');
        }
        deep.push('}');
        assert!(matches!(
            lifecycle.normalize_advertisement(McpAdvertisementKind::Tool, deep.as_bytes()),
            Err(McpProtocolError::PayloadTooDeep)
        ));

        let items = (0..129).map(|_| "0").collect::<Vec<_>>().join(",");
        let wide = format!("{{\"name\":\"wide\",\"inputSchema\":{{\"enum\":[{items}]}}}}");
        assert!(matches!(
            lifecycle.normalize_advertisement(McpAdvertisementKind::Tool, wide.as_bytes()),
            Err(McpProtocolError::ContainerTooLarge)
        ));
    }

    #[test]
    fn canonical_schema_digest_does_not_depend_on_object_key_order() {
        let lifecycle = lifecycle(McpTransport::LocalStdio);
        let left = lifecycle
            .normalize_advertisement(
                McpAdvertisementKind::Tool,
                br#"{"name":"same","inputSchema":{"type":"object","properties":{"a":{"type":"string"},"b":{"type":"integer"}}}}"#,
            )
            .unwrap();
        let right = lifecycle
            .normalize_advertisement(
                McpAdvertisementKind::Tool,
                br#"{"inputSchema":{"properties":{"b":{"type":"integer"},"a":{"type":"string"}},"type":"object"},"name":"same"}"#,
            )
            .unwrap();
        assert_eq!(left.input_schema_ref, right.input_schema_ref);
        assert_eq!(left.protocol_payload_ref, right.protocol_payload_ref);
    }

    #[test]
    fn mapping_or_version_drift_invalidates_dispatch() {
        let mut lifecycle = lifecycle(McpTransport::LocalStdio);
        lifecycle.transition(McpLifecycleState::Active).unwrap();
        let dispatch = lifecycle
            .bind_dispatch(digest(10), digest(11), digest(12))
            .unwrap();
        let mut current = lifecycle.current_state();
        current.golam_local_mapping_digest = digest(99);
        assert_eq!(
            dispatch.revalidate(&current),
            Err(DispatchValidationError::McpMappingMismatch)
        );
        current = lifecycle.current_state();
        current.version_lock = McpVersionLock::new("2026-07-28").unwrap();
        assert_eq!(
            dispatch.revalidate(&current),
            Err(DispatchValidationError::McpVersionMismatch)
        );
    }

    #[test]
    fn remote_review_requires_explicit_network_policy() {
        let error = McpLifecycle::review(McpReviewRequest {
            binding_id: digest(1),
            server_identity: digest(2),
            transport: McpTransport::RemoteHttp,
            process_profile_ref_or_remote_endpoint: digest(3),
            allowed_protocol_features: vec![],
            golam_local_mapping_ref: digest(5),
            golam_local_mapping_digest: digest(6),
            network_policy_ref: BindingDigest::new([0; 32]),
            secret_policy_ref: digest(8),
            taint_class: TaintSet::empty(),
            version_lock: McpVersionLock::new("2025-06-18").unwrap(),
        })
        .unwrap_err();
        assert!(matches!(
            error,
            McpProtocolError::InvalidBinding("network_policy_ref")
        ));
    }
}
