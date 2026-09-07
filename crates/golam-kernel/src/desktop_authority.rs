#![forbid(unsafe_code)]

use core::fmt;

use golam_core::desktop_control::{
    DESKTOP_CONTROL_SCHEMA_VERSION, DesktopControlError, DesktopControlLeaseId,
    DesktopControlLeaseState, DesktopControlMode, HumanInterruptEvidence, HumanInterruptOperation,
    VisibleControlChannelId, VisibleControlChannelKind, VisibleControlChannelState,
};
use golam_core::digest::sha256;
use golam_core::tool_request::BindingDigest;
use golam_ledger::desktop_control_evidence::{
    DesktopControlEvidenceError, DesktopControlEvidenceStore,
};

use crate::{AuthorizationPolicy, KernelApi, Principal, PrincipalKind};

const MAX_VISIBLE_CHANNELS: usize = 8;
const DESKTOP_VISIBLE_CHANNEL_TTL_MS: u64 = 5_000;
const DESKTOP_HOST_BINDING_DOMAIN: &[u8] = b"golam:desktop-visible-host:v1";
const DESKTOP_CHANNEL_ID_DOMAIN: &[u8] = b"golam:desktop-visible-channel-id:v1";
const DESKTOP_INTERRUPT_ID_DOMAIN: &[u8] = b"golam:desktop-native-interrupt-id:v1";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HumanInterruptRequest {
    pub interrupt_id: u128,
    pub attributed_local_source_ref: BindingDigest,
    pub operation: HumanInterruptOperation,
    pub accepted_at_unix_ms: u64,
    pub authority_revoked_at_unix_ms: u64,
    pub affected_operation_refs: Vec<BindingDigest>,
    pub cancellation_reconciliation_refs: Vec<BindingDigest>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DesktopControlSurfaceSnapshot {
    pub mode: Option<DesktopControlMode>,
    pub visible_channel_qualified: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProtectedDesktopControlState {
    lease: DesktopControlLeaseState,
    visible_channels: Vec<VisibleControlChannelState>,
    last_interrupt: Option<HumanInterruptEvidence>,
}

impl ProtectedDesktopControlState {
    pub fn new(
        lease: DesktopControlLeaseState,
        mut visible_channels: Vec<VisibleControlChannelState>,
    ) -> Result<Self, DesktopAuthorityError> {
        lease.validate()?;
        if visible_channels.len() > MAX_VISIBLE_CHANNELS {
            return Err(DesktopAuthorityError::TooManyVisibleChannels);
        }
        for channel in &visible_channels {
            channel.validate()?;
        }
        visible_channels.sort_by_key(|channel| channel.channel_id);
        if visible_channels
            .windows(2)
            .any(|pair| pair[0].channel_id == pair[1].channel_id)
        {
            return Err(DesktopAuthorityError::DuplicateVisibleChannel);
        }
        Ok(Self {
            lease,
            visible_channels,
            last_interrupt: None,
        })
    }

    pub const fn current_lease(&self) -> DesktopControlLeaseState {
        self.lease
    }

    pub fn last_interrupt(&self) -> Option<&HumanInterruptEvidence> {
        self.last_interrupt.as_ref()
    }

    pub fn channel(
        &self,
        channel_id: VisibleControlChannelId,
    ) -> Option<VisibleControlChannelState> {
        self.visible_channels
            .iter()
            .copied()
            .find(|channel| channel.channel_id == channel_id)
    }

    pub fn qualified_visible_channel(
        &self,
        now_unix_ms: u64,
    ) -> Option<VisibleControlChannelState> {
        self.visible_channels
            .iter()
            .copied()
            .find(|channel| channel.qualifies_for_autonomous_actuation(now_unix_ms))
    }

    pub fn autonomous_actuation_allowed(&self, now_unix_ms: u64) -> bool {
        self.lease.allows_agent_input(now_unix_ms)
            && self.qualified_visible_channel(now_unix_ms).is_some()
    }

    pub fn upsert_visible_channel(
        &mut self,
        next: VisibleControlChannelState,
    ) -> Result<(), DesktopAuthorityError> {
        next.validate()?;
        match self
            .visible_channels
            .binary_search_by_key(&next.channel_id, |channel| channel.channel_id)
        {
            Ok(index) => {
                let current = self.visible_channels[index];
                if next.generation <= current.generation
                    || next.kind != current.kind
                    || next.trusted_host_ref != current.trusted_host_ref
                    || next.observed_at_unix_ms < current.observed_at_unix_ms
                {
                    return Err(DesktopAuthorityError::StaleOrSubstitutedVisibleChannel);
                }
                self.visible_channels[index] = next;
            }
            Err(index) => {
                if self.visible_channels.len() >= MAX_VISIBLE_CHANNELS {
                    return Err(DesktopAuthorityError::TooManyVisibleChannels);
                }
                self.visible_channels.insert(index, next);
            }
        }
        Ok(())
    }

    pub fn apply_human_interrupt(
        &mut self,
        request: HumanInterruptRequest,
    ) -> Result<HumanInterruptEvidence, DesktopAuthorityError> {
        validate_interrupt_request(&request)?;
        self.lease.validate()?;
        if request.authority_revoked_at_unix_ms >= self.lease.expires_at_unix_ms {
            return Err(DesktopAuthorityError::LeaseExpiredBeforeInterrupt);
        }
        let next_mode = transition_mode(self.lease.mode, request.operation)?;
        if request.operation == HumanInterruptOperation::ReleaseHumanExclusive
            && self
                .qualified_visible_channel(request.authority_revoked_at_unix_ms)
                .is_none()
        {
            return Err(DesktopAuthorityError::NoQualifiedVisibleChannel);
        }
        let resulting_generation = self
            .lease
            .generation
            .checked_add(1)
            .ok_or(DesktopAuthorityError::GenerationOverflow)?;
        let evidence = HumanInterruptEvidence {
            schema_version: DESKTOP_CONTROL_SCHEMA_VERSION,
            interrupt_id: request.interrupt_id,
            attributed_local_source_ref: request.attributed_local_source_ref,
            operation: request.operation,
            prior_lease_id: self.lease.lease_id,
            prior_generation: self.lease.generation,
            resulting_lease_id: self.lease.lease_id,
            resulting_generation,
            accepted_at_unix_ms: request.accepted_at_unix_ms,
            authority_revoked_at_unix_ms: request.authority_revoked_at_unix_ms,
            affected_operation_refs: request.affected_operation_refs,
            cancellation_reconciliation_refs: request.cancellation_reconciliation_refs,
        };
        evidence.validate()?;
        let cause_ref = evidence.binding_digest()?;
        let next_lease = DesktopControlLeaseState {
            schema_version: self.lease.schema_version,
            lease_id: self.lease.lease_id,
            generation: resulting_generation,
            controlling_principal_ref: self.lease.controlling_principal_ref,
            mode: next_mode,
            issued_at_unix_ms: self.lease.issued_at_unix_ms,
            updated_at_unix_ms: request.authority_revoked_at_unix_ms,
            expires_at_unix_ms: self.lease.expires_at_unix_ms,
            capability_ref: self.lease.capability_ref,
            policy_ref: self.lease.policy_ref,
            interrupt_cause_ref: Some(cause_ref),
        };
        next_lease.validate()?;
        self.lease = next_lease;
        self.last_interrupt = Some(evidence.clone());
        Ok(evidence)
    }
}

impl<P: AuthorizationPolicy> KernelApi<P> {
    pub fn persist_desktop_control_state(
        &self,
        state: &ProtectedDesktopControlState,
    ) -> Result<(), DesktopAuthorityError> {
        let mut store = DesktopControlEvidenceStore::open(&self.authority)?;
        store.persist_lease_state(state.lease)?;
        for channel in &state.visible_channels {
            store.persist_visible_channel(*channel)?;
        }
        Ok(())
    }

    pub fn persist_desktop_visible_channel_update(
        &self,
        state: &mut ProtectedDesktopControlState,
        next: VisibleControlChannelState,
    ) -> Result<(), DesktopAuthorityError> {
        let mut candidate = state.clone();
        candidate.upsert_visible_channel(next)?;
        let mut store = DesktopControlEvidenceStore::open(&self.authority)?;
        store.persist_visible_channel(next)?;
        *state = candidate;
        Ok(())
    }

    pub fn observe_desktop_native_visible_channel(
        &self,
        principal: Principal<'_>,
        visible: bool,
        observed_at_unix_ms: u64,
    ) -> Result<VisibleControlChannelState, DesktopAuthorityError> {
        let client_id = desktop_client_id(principal)?;
        if observed_at_unix_ms == 0 {
            return Err(DesktopAuthorityError::InvalidVisibleChannelObservation);
        }
        let channel_id = desktop_channel_id(client_id.0)?;
        let trusted_host_ref = desktop_host_ref(client_id.0);
        let mut store = DesktopControlEvidenceStore::open(&self.authority)?;
        let current = store
            .load_visible_channels()?
            .into_iter()
            .find(|channel| channel.channel_id == channel_id);
        let generation = match current {
            Some(channel) => {
                if channel.kind != VisibleControlChannelKind::TauriNativeWindow
                    || channel.trusted_host_ref != trusted_host_ref
                    || observed_at_unix_ms < channel.observed_at_unix_ms
                {
                    return Err(DesktopAuthorityError::StaleOrSubstitutedVisibleChannel);
                }
                channel
                    .generation
                    .checked_add(1)
                    .ok_or(DesktopAuthorityError::GenerationOverflow)?
            }
            None => 1,
        };
        let heartbeat_deadline_unix_ms = observed_at_unix_ms
            .checked_add(DESKTOP_VISIBLE_CHANNEL_TTL_MS)
            .ok_or(DesktopAuthorityError::VisibleChannelDeadlineOverflow)?;
        let next = VisibleControlChannelState {
            schema_version: DESKTOP_CONTROL_SCHEMA_VERSION,
            channel_id,
            generation,
            kind: VisibleControlChannelKind::TauriNativeWindow,
            trusted_host_ref,
            visible,
            live: true,
            supports_pause: true,
            supports_stop: true,
            supports_takeover: true,
            observed_at_unix_ms,
            heartbeat_deadline_unix_ms,
        };
        store.persist_visible_channel(next)?;
        Ok(next)
    }

    pub fn desktop_control_surface_snapshot(
        &self,
        now_unix_ms: u64,
    ) -> Result<DesktopControlSurfaceSnapshot, DesktopAuthorityError> {
        if now_unix_ms == 0 {
            return Err(DesktopAuthorityError::InvalidVisibleChannelObservation);
        }
        let store = DesktopControlEvidenceStore::open(&self.authority)?;
        let visible_channel_qualified = store
            .load_visible_channels()?
            .iter()
            .any(|channel| channel.qualifies_for_autonomous_actuation(now_unix_ms));
        let lease_ids = store.load_lease_ids()?;
        let mode = match lease_ids.as_slice() {
            [] => None,
            [lease_id] => Some(
                store
                    .load_lease_state(*lease_id)?
                    .ok_or(DesktopAuthorityError::MissingDesktopControlLease)?
                    .mode,
            ),
            _ => return Err(DesktopAuthorityError::AmbiguousDesktopControlLease),
        };
        Ok(DesktopControlSurfaceSnapshot {
            mode,
            visible_channel_qualified,
        })
    }

    pub fn apply_native_desktop_human_interrupt(
        &self,
        principal: Principal<'_>,
        operation: HumanInterruptOperation,
        accepted_at_unix_ms: u64,
        authority_revoked_at_unix_ms: u64,
    ) -> Result<DesktopControlSurfaceSnapshot, DesktopAuthorityError> {
        let client_id = desktop_client_id(principal)?;
        if accepted_at_unix_ms == 0 || authority_revoked_at_unix_ms < accepted_at_unix_ms {
            return Err(DesktopAuthorityError::InvalidInterruptRequest);
        }
        let channel_id = desktop_channel_id(client_id.0)?;
        let trusted_host_ref = desktop_host_ref(client_id.0);
        let store = DesktopControlEvidenceStore::open(&self.authority)?;
        let lease_ids = store.load_lease_ids()?;
        let lease_id = match lease_ids.as_slice() {
            [] => return Err(DesktopAuthorityError::MissingDesktopControlLease),
            [lease_id] => *lease_id,
            _ => return Err(DesktopAuthorityError::AmbiguousDesktopControlLease),
        };
        drop(store);
        let mut state = self
            .restore_desktop_control_state(lease_id)?
            .ok_or(DesktopAuthorityError::MissingDesktopControlLease)?;
        let source_channel = state
            .channel(channel_id)
            .ok_or(DesktopAuthorityError::NoQualifiedVisibleChannel)?;
        if source_channel.trusted_host_ref != trusted_host_ref
            || !source_channel.qualifies_for_autonomous_actuation(accepted_at_unix_ms)
        {
            return Err(DesktopAuthorityError::NoQualifiedVisibleChannel);
        }
        let interrupt_id = desktop_interrupt_id(
            client_id.0,
            state.current_lease(),
            operation,
            accepted_at_unix_ms,
        )?;
        self.apply_persisted_desktop_human_interrupt(
            &mut state,
            HumanInterruptRequest {
                interrupt_id,
                attributed_local_source_ref: trusted_host_ref,
                operation,
                accepted_at_unix_ms,
                authority_revoked_at_unix_ms,
                affected_operation_refs: Vec::new(),
                cancellation_reconciliation_refs: Vec::new(),
            },
        )?;
        Ok(DesktopControlSurfaceSnapshot {
            mode: Some(state.current_lease().mode),
            visible_channel_qualified: state
                .qualified_visible_channel(authority_revoked_at_unix_ms)
                .is_some(),
        })
    }

    pub fn apply_persisted_desktop_human_interrupt(
        &self,
        state: &mut ProtectedDesktopControlState,
        request: HumanInterruptRequest,
    ) -> Result<HumanInterruptEvidence, DesktopAuthorityError> {
        let mut candidate = state.clone();
        let operation = request.operation;
        let evidence = candidate.apply_human_interrupt(request)?;
        let resulting_lease = candidate.current_lease();
        let mut store = DesktopControlEvidenceStore::open(&self.authority)?;

        if operation == HumanInterruptOperation::ReleaseHumanExclusive {
            // A failed persistence sequence must never widen agent authority. Record
            // release evidence first; if lease persistence fails, durable state remains
            // human-exclusive and dispatch stays fail closed.
            store.persist_interrupt(&evidence)?;
            store.persist_lease_state(resulting_lease)?;
        } else {
            // Pause/stop/takeover are restrictive. Persist the newer restrictive lease
            // first so a crash cannot resurrect stale agent authority.
            store.persist_lease_state(resulting_lease)?;
            store.persist_interrupt(&evidence)?;
        }
        *state = candidate;
        Ok(evidence)
    }

    pub fn restore_desktop_control_state(
        &self,
        lease_id: DesktopControlLeaseId,
    ) -> Result<Option<ProtectedDesktopControlState>, DesktopAuthorityError> {
        let store = DesktopControlEvidenceStore::open(&self.authority)?;
        let Some(lease) = store.load_lease_state(lease_id)? else {
            return Ok(None);
        };
        let channels = store.load_visible_channels()?;
        Ok(Some(ProtectedDesktopControlState::new(lease, channels)?))
    }
}

fn desktop_client_id(principal: Principal<'_>) -> Result<golam_core::ClientId, DesktopAuthorityError> {
    if principal.kind != PrincipalKind::EnrolledClient || principal.subject != "local-desktop" {
        return Err(DesktopAuthorityError::UnauthorizedDesktopHost);
    }
    let client_id = principal
        .client_id
        .ok_or(DesktopAuthorityError::UnauthorizedDesktopHost)?;
    if client_id.0 == 0 {
        return Err(DesktopAuthorityError::UnauthorizedDesktopHost);
    }
    Ok(client_id)
}

fn desktop_host_ref(client_id: u128) -> BindingDigest {
    let mut bytes = Vec::with_capacity(DESKTOP_HOST_BINDING_DOMAIN.len() + 16);
    bytes.extend_from_slice(DESKTOP_HOST_BINDING_DOMAIN);
    bytes.extend_from_slice(&client_id.to_be_bytes());
    BindingDigest::new(sha256(&bytes))
}

fn desktop_channel_id(client_id: u128) -> Result<VisibleControlChannelId, DesktopAuthorityError> {
    let mut bytes = Vec::with_capacity(DESKTOP_CHANNEL_ID_DOMAIN.len() + 16);
    bytes.extend_from_slice(DESKTOP_CHANNEL_ID_DOMAIN);
    bytes.extend_from_slice(&client_id.to_be_bytes());
    let digest = sha256(&bytes);
    let value = u128::from_be_bytes(
        digest[..16]
            .try_into()
            .expect("desktop channel digest prefix is exactly 16 bytes"),
    );
    if value == 0 {
        return Err(DesktopAuthorityError::InvalidDesktopHostIdentity);
    }
    Ok(VisibleControlChannelId::from_u128(value))
}

fn desktop_interrupt_id(
    client_id: u128,
    lease: DesktopControlLeaseState,
    operation: HumanInterruptOperation,
    accepted_at_unix_ms: u64,
) -> Result<u128, DesktopAuthorityError> {
    let mut bytes = Vec::with_capacity(DESKTOP_INTERRUPT_ID_DOMAIN.len() + 57);
    bytes.extend_from_slice(DESKTOP_INTERRUPT_ID_DOMAIN);
    bytes.extend_from_slice(&client_id.to_be_bytes());
    bytes.extend_from_slice(&lease.lease_id.as_u128().to_be_bytes());
    bytes.extend_from_slice(&lease.generation.to_be_bytes());
    bytes.push(interrupt_operation_code(operation));
    bytes.extend_from_slice(&accepted_at_unix_ms.to_be_bytes());
    let digest = sha256(&bytes);
    let value = u128::from_be_bytes(
        digest[..16]
            .try_into()
            .expect("desktop interrupt digest prefix is exactly 16 bytes"),
    );
    if value == 0 {
        return Err(DesktopAuthorityError::InvalidInterruptRequest);
    }
    Ok(value)
}

const fn interrupt_operation_code(operation: HumanInterruptOperation) -> u8 {
    match operation {
        HumanInterruptOperation::Pause => 1,
        HumanInterruptOperation::Stop => 2,
        HumanInterruptOperation::Takeover => 3,
        HumanInterruptOperation::ReleaseHumanExclusive => 4,
    }
}

fn validate_interrupt_request(
    request: &HumanInterruptRequest,
) -> Result<(), DesktopAuthorityError> {
    if request.interrupt_id == 0
        || request.attributed_local_source_ref.bytes() == [0; 32]
        || request.accepted_at_unix_ms == 0
        || request.authority_revoked_at_unix_ms < request.accepted_at_unix_ms
    {
        return Err(DesktopAuthorityError::InvalidInterruptRequest);
    }
    validate_sorted_digest_refs(&request.affected_operation_refs)?;
    validate_sorted_digest_refs(&request.cancellation_reconciliation_refs)?;
    Ok(())
}

fn validate_sorted_digest_refs(values: &[BindingDigest]) -> Result<(), DesktopAuthorityError> {
    if values.len() > 128
        || values.iter().any(|value| value.bytes() == [0; 32])
        || values.windows(2).any(|pair| pair[0] >= pair[1])
    {
        return Err(DesktopAuthorityError::InvalidInterruptRequest);
    }
    Ok(())
}

fn transition_mode(
    current: DesktopControlMode,
    operation: HumanInterruptOperation,
) -> Result<DesktopControlMode, DesktopAuthorityError> {
    match (current, operation) {
        (DesktopControlMode::AgentAllowed, HumanInterruptOperation::Pause) => {
            Ok(DesktopControlMode::Paused)
        }
        (
            DesktopControlMode::AgentAllowed | DesktopControlMode::Paused,
            HumanInterruptOperation::Takeover,
        ) => Ok(DesktopControlMode::HumanExclusive),
        (DesktopControlMode::HumanExclusive, HumanInterruptOperation::ReleaseHumanExclusive) => {
            Ok(DesktopControlMode::AgentAllowed)
        }
        (
            DesktopControlMode::AgentAllowed
            | DesktopControlMode::Paused
            | DesktopControlMode::HumanExclusive,
            HumanInterruptOperation::Stop,
        ) => Ok(DesktopControlMode::Revoked),
        _ => Err(DesktopAuthorityError::InvalidInterruptTransition),
    }
}

#[derive(Debug)]
pub enum DesktopAuthorityError {
    InvalidInterruptRequest,
    InvalidInterruptTransition,
    GenerationOverflow,
    LeaseExpiredBeforeInterrupt,
    TooManyVisibleChannels,
    DuplicateVisibleChannel,
    StaleOrSubstitutedVisibleChannel,
    NoQualifiedVisibleChannel,
    UnauthorizedDesktopHost,
    InvalidDesktopHostIdentity,
    InvalidVisibleChannelObservation,
    VisibleChannelDeadlineOverflow,
    MissingDesktopControlLease,
    AmbiguousDesktopControlLease,
    Evidence(DesktopControlEvidenceError),
    Control(DesktopControlError),
}

impl fmt::Display for DesktopAuthorityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidInterruptRequest => f.write_str("invalid human interrupt request"),
            Self::InvalidInterruptTransition => {
                f.write_str("human interrupt transition is not permitted")
            }
            Self::GenerationOverflow => f.write_str("desktop control lease generation overflow"),
            Self::LeaseExpiredBeforeInterrupt => {
                f.write_str("desktop control lease expired before interrupt")
            }
            Self::TooManyVisibleChannels => f.write_str("too many visible control channels"),
            Self::DuplicateVisibleChannel => f.write_str("duplicate visible control channel"),
            Self::StaleOrSubstitutedVisibleChannel => {
                f.write_str("visible control channel is stale or substituted")
            }
            Self::NoQualifiedVisibleChannel => {
                f.write_str("no qualified visible control channel permits agent release")
            }
            Self::UnauthorizedDesktopHost => {
                f.write_str("desktop control request requires an authenticated DesktopFuture host")
            }
            Self::InvalidDesktopHostIdentity => f.write_str("desktop host identity is invalid"),
            Self::InvalidVisibleChannelObservation => {
                f.write_str("desktop visible-channel observation is invalid")
            }
            Self::VisibleChannelDeadlineOverflow => {
                f.write_str("desktop visible-channel heartbeat deadline overflow")
            }
            Self::MissingDesktopControlLease => f.write_str("no protected desktop control lease exists"),
            Self::AmbiguousDesktopControlLease => {
                f.write_str("multiple protected desktop control leases exist; refusing ambiguity")
            }
            Self::Evidence(error) => write!(f, "desktop authority durable evidence error: {error}"),
            Self::Control(error) => write!(f, "desktop control authority error: {error}"),
        }
    }
}

impl std::error::Error for DesktopAuthorityError {}

impl From<DesktopControlEvidenceError> for DesktopAuthorityError {
    fn from(value: DesktopControlEvidenceError) -> Self {
        Self::Evidence(value)
    }
}

impl From<DesktopControlError> for DesktopAuthorityError {
    fn from(value: DesktopControlError) -> Self {
        Self::Control(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use golam_core::desktop_control::{DesktopControlLeaseId, VisibleControlChannelKind};

    fn digest(value: u8) -> BindingDigest {
        BindingDigest::new([value; 32])
    }

    fn lease() -> DesktopControlLeaseState {
        DesktopControlLeaseState {
            schema_version: DESKTOP_CONTROL_SCHEMA_VERSION,
            lease_id: DesktopControlLeaseId::from_u128(1),
            generation: 1,
            controlling_principal_ref: digest(1),
            mode: DesktopControlMode::AgentAllowed,
            issued_at_unix_ms: 10,
            updated_at_unix_ms: 10,
            expires_at_unix_ms: 10_000,
            capability_ref: digest(2),
            policy_ref: digest(3),
            interrupt_cause_ref: None,
        }
    }

    fn channel() -> VisibleControlChannelState {
        VisibleControlChannelState {
            schema_version: DESKTOP_CONTROL_SCHEMA_VERSION,
            channel_id: VisibleControlChannelId::from_u128(1),
            generation: 1,
            kind: VisibleControlChannelKind::TauriNativeWindow,
            trusted_host_ref: digest(4),
            visible: true,
            live: true,
            supports_pause: true,
            supports_stop: true,
            supports_takeover: true,
            observed_at_unix_ms: 10,
            heartbeat_deadline_unix_ms: 1_000,
        }
    }

    fn interrupt(operation: HumanInterruptOperation, id: u128) -> HumanInterruptRequest {
        HumanInterruptRequest {
            interrupt_id: id,
            attributed_local_source_ref: digest(5),
            operation,
            accepted_at_unix_ms: 20 + id as u64,
            authority_revoked_at_unix_ms: 21 + id as u64,
            affected_operation_refs: vec![digest(6)],
            cancellation_reconciliation_refs: vec![digest(7)],
        }
    }

    #[test]
    fn takeover_advances_generation_and_stale_agent_authority_stays_invalid() {
        let mut state = ProtectedDesktopControlState::new(lease(), vec![channel()]).unwrap();
        assert!(state.autonomous_actuation_allowed(100));
        let evidence = state
            .apply_human_interrupt(interrupt(HumanInterruptOperation::Takeover, 1))
            .unwrap();
        assert_eq!(evidence.prior_generation, 1);
        assert_eq!(evidence.resulting_generation, 2);
        assert_eq!(
            state.current_lease().mode,
            DesktopControlMode::HumanExclusive
        );
        assert!(!state.autonomous_actuation_allowed(100));
        assert_eq!(evidence.takeover_latency_ms().unwrap(), 1);
    }

    #[test]
    fn release_requires_human_exclusive_state_and_live_visible_channel() {
        let mut state = ProtectedDesktopControlState::new(lease(), vec![channel()]).unwrap();
        assert!(matches!(
            state
                .apply_human_interrupt(
                    interrupt(HumanInterruptOperation::ReleaseHumanExclusive, 1,)
                )
                .unwrap_err(),
            DesktopAuthorityError::InvalidInterruptTransition
        ));
        state
            .apply_human_interrupt(interrupt(HumanInterruptOperation::Takeover, 2))
            .unwrap();
        let mut hidden = channel();
        hidden.generation = 2;
        hidden.visible = false;
        hidden.observed_at_unix_ms = 30;
        hidden.heartbeat_deadline_unix_ms = 2_000;
        state.upsert_visible_channel(hidden).unwrap();
        assert!(matches!(
            state
                .apply_human_interrupt(
                    interrupt(HumanInterruptOperation::ReleaseHumanExclusive, 3,)
                )
                .unwrap_err(),
            DesktopAuthorityError::NoQualifiedVisibleChannel
        ));
    }

    #[test]
    fn desktop_visible_identity_is_stable_and_principal_bound() {
        let first = desktop_channel_id(41).unwrap();
        assert_eq!(first, desktop_channel_id(41).unwrap());
        assert_ne!(first, desktop_channel_id(42).unwrap());
        assert_ne!(desktop_host_ref(41), desktop_host_ref(42));
    }

    #[test]
    fn only_exact_authenticated_desktop_principal_can_drive_native_control() {
        assert!(desktop_client_id(Principal::enrolled_client("local-desktop", golam_core::ClientId(9))).is_ok());
        assert!(matches!(
            desktop_client_id(Principal::enrolled_client("local-cli", golam_core::ClientId(9))),
            Err(DesktopAuthorityError::UnauthorizedDesktopHost)
        ));
        assert!(matches!(
            desktop_client_id(Principal::local_owner("local-owner")),
            Err(DesktopAuthorityError::UnauthorizedDesktopHost)
        ));
    }
}
