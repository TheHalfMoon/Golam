#![forbid(unsafe_code)]

use golam_core::{EffectId, SessionId};
use golam_effects::simulators::{
    AtMostOnceWriteHandler, CompensatableWriteHandler, IdempotentWriteHandler,
    IrreversibleWriteHandler, PureReadHandler,
};
use golam_effects::{
    EffectHandler, EffectSemantics, HandlerAttemptOutcome, HandlerIntent, HandlerOutcome,
    PriorAttempt,
};
use golam_ipc::command::{AuthorityQualificationKind, Command, SyntheticSemantics, decode_command};
use golam_ipc::request::{ReplyMessage, ReplyStatus, RequestMessage};
use golam_kernel::{
    AdminQualificationKind, AdminSurfaceError, AuthorizationPolicy, ClientEnrollmentError,
    ClientKind, CompleteSyntheticEffect, KernelApi, KernelError, KernelOperationError,
    PrepareSyntheticEffect, Principal, ResolveSyntheticReconciliation, SyntheticEffectError,
    SyntheticExecutionCompletion, SyntheticReconciliationResult,
};

const MAX_REPLY_BODY_BYTES: usize = 256 * 1024;

#[derive(Default)]
struct SyntheticHandlers {
    read: PureReadHandler,
    idempotent: IdempotentWriteHandler,
    at_most_once: AtMostOnceWriteHandler,
    compensatable: CompensatableWriteHandler,
    irreversible: IrreversibleWriteHandler,
}

impl SyntheticHandlers {
    fn by_semantics_mut(&mut self, semantics: SyntheticSemantics) -> &mut dyn EffectHandler {
        match semantics {
            SyntheticSemantics::ReadOnly => &mut self.read,
            SyntheticSemantics::IdempotentAtLeastOnce => &mut self.idempotent,
            SyntheticSemantics::AtMostOnce => &mut self.at_most_once,
            SyntheticSemantics::Compensatable => &mut self.compensatable,
            SyntheticSemantics::Irreversible => &mut self.irreversible,
        }
    }

    fn by_handler_id_mut(&mut self, handler_id: &str) -> Option<&mut dyn EffectHandler> {
        match handler_id {
            "sim-pure-read" => Some(&mut self.read),
            "sim-idempotent-write" => Some(&mut self.idempotent),
            "sim-at-most-once-write" => Some(&mut self.at_most_once),
            "sim-compensatable-write" => Some(&mut self.compensatable),
            "sim-irreversible-write" => Some(&mut self.irreversible),
            _ => None,
        }
    }
}

pub struct CommandRouter<P> {
    kernel: KernelApi<P>,
    handlers: SyntheticHandlers,
}

impl<P: AuthorizationPolicy> CommandRouter<P> {
    pub fn new(kernel: KernelApi<P>) -> Self {
        Self {
            kernel,
            handlers: SyntheticHandlers::default(),
        }
    }

    pub fn route(
        &mut self,
        principal: Principal<'_>,
        request: &RequestMessage,
        now: &str,
        scope: &str,
    ) -> ReplyMessage {
        let command = match decode_command(request) {
            Ok(command) => command,
            Err(error) => return reply(ReplyStatus::InvalidRequest, format!("error={error}\n")),
        };
        self.route_command(principal, command, now, scope)
    }

    fn route_command(
        &mut self,
        principal: Principal<'_>,
        command: Command,
        now: &str,
        scope: &str,
    ) -> ReplyMessage {
        match command {
            Command::ClientEnroll { client_id } => {
                match self.kernel.generate_and_enroll_client(
                    principal,
                    client_id,
                    ClientKind::Cli,
                    now,
                    scope,
                ) {
                    Ok(enrolled) => reply(
                        ReplyStatus::Ok,
                        format!(
                            "client_id={} key_id={} assurance={} credential_path={}\n",
                            enrolled.record.client_id.0,
                            hex_prefix(&enrolled.record.key_id),
                            enrolled.record.assurance_class.as_str(),
                            enrolled.credential.path.display()
                        ),
                    ),
                    Err(error) => enrollment_error(error),
                }
            }
            Command::SessionsList => match self.kernel.list_sessions(principal, scope) {
                Ok(sessions) => {
                    let mut body = String::new();
                    for session in sessions {
                        body.push_str(&format!(
                            "session_id={} owner={} status={} latest_session_seq={} parent_session_id={} latest_checkpoint_id={}\n",
                            session.session_id.0,
                            session.owner_principal,
                            session.status,
                            session.latest_session_seq,
                            optional_u128(session.parent_session_id.map(|value| value.0)),
                            optional_u128(session.latest_checkpoint_id.map(|value| value.0)),
                        ));
                    }
                    reply(ReplyStatus::Ok, body)
                }
                Err(error) => operation_error(error),
            },
            Command::SessionOpen { session_id } => {
                match self.kernel.open_session(principal, session_id, scope) {
                    Ok(Some(session)) => reply(
                        ReplyStatus::Ok,
                        format!(
                            "session_id={} owner={} status={} latest_session_seq={} parent_session_id={} latest_checkpoint_id={}\n",
                            session.session_id.0,
                            session.owner_principal,
                            session.status,
                            session.latest_session_seq,
                            optional_u128(session.parent_session_id.map(|value| value.0)),
                            optional_u128(session.latest_checkpoint_id.map(|value| value.0)),
                        ),
                    ),
                    Ok(None) => reply(ReplyStatus::Failed, "error=session_not_found\n".to_owned()),
                    Err(error) => operation_error(error),
                }
            }
            Command::SessionCreate {
                session_id,
                event_id,
                recorded_at,
                payload,
            } => match self.kernel.create_session(
                principal,
                golam_kernel::KernelCreateSession {
                    session_id,
                    event_id,
                    recorded_at: &recorded_at,
                    payload: &payload,
                },
                scope,
            ) {
                Ok(stored) => reply(
                    ReplyStatus::Ok,
                    format!(
                        "session_id={} session_seq={} global_seq={}\n",
                        stored.record.session_id.0,
                        stored.record.session_seq,
                        stored.record.global_seq
                    ),
                ),
                Err(error) => operation_error(error),
            },
            Command::SessionFork {
                child_session_id,
                event_id,
                parent_session_id,
                through_session_seq,
                recorded_at,
            } => match self.kernel.fork_session(
                principal,
                golam_kernel::KernelCreateFork {
                    child_session_id,
                    event_id,
                    parent_session_id,
                    through_session_seq,
                    recorded_at: &recorded_at,
                },
                scope,
            ) {
                Ok(record) => reply(
                    ReplyStatus::Ok,
                    format!(
                        "session_id={} parent_session_id={} parent_session_seq={} global_seq={}\n",
                        record.child_session_id.0,
                        record.anchor.parent_session_id.0,
                        record.anchor.parent_session_seq,
                        record.child_global_seq
                    ),
                ),
                Err(error) => operation_error(error),
            },
            Command::GoalAppend {
                goal_version_id,
                goal_id,
                event_id,
                session_id,
                expected_session_seq,
                expected_goal_version,
                recorded_at,
                goal,
            } => match self.kernel.append_goal_version(
                principal,
                golam_kernel::KernelAppendGoal {
                    goal_version_id,
                    goal_id,
                    event_id,
                    session_id,
                    expected_session_seq,
                    expected_goal_version,
                    recorded_at: &recorded_at,
                    document: golam_kernel::GoalDocument {
                        goal: &goal,
                        acceptance_criteria: &[],
                        constraints: &[],
                        scope: "cli",
                        proven_facts: &[],
                        blockers: &[],
                        next_safe_action: None,
                    },
                },
                scope,
            ) {
                Ok(version) => reply(
                    ReplyStatus::Ok,
                    format!(
                        "goal_id={} version={} session_id={} global_seq={}\n",
                        version.goal_id.0,
                        version.version,
                        version.session_id.0,
                        version.created_global_seq
                    ),
                ),
                Err(error) => operation_error(error),
            },
            Command::CheckpointCreate {
                checkpoint_id,
                created_event_id,
                session_id,
                through_session_seq,
                recorded_at,
            } => match self.kernel.create_checkpoint(
                principal,
                golam_kernel::KernelCreateCheckpoint {
                    checkpoint_id,
                    created_event_id,
                    session_id,
                    through_session_seq,
                    recorded_at: &recorded_at,
                },
                scope,
            ) {
                Ok(record) => reply(
                    ReplyStatus::Ok,
                    format!(
                        "checkpoint_id={} session_id={} through_session_seq={} through_global_seq={}\n",
                        record.checkpoint_id.0,
                        record.session_id.0,
                        record.through_session_seq,
                        record.through_global_seq
                    ),
                ),
                Err(error) => operation_error(error),
            },
            Command::CheckpointVerify {
                checkpoint_id,
                session_id,
                through_session_seq,
            } => match self.kernel.verify_checkpoint(
                principal,
                checkpoint_id,
                session_id,
                through_session_seq,
                scope,
            ) {
                Ok(projection) => reply(
                    ReplyStatus::Ok,
                    format!(
                        "source={:?} bytes={}\n",
                        projection.source,
                        projection.bytes.len()
                    ),
                ),
                Err(error) => operation_error(error),
            },
            Command::Replay {
                session_id,
                through_session_seq,
            } => match self
                .kernel
                .replay_session(principal, session_id, through_session_seq, scope)
            {
                Ok(bytes) => reply(
                    ReplyStatus::Ok,
                    format!("bytes={} digest={}\n", bytes.len(), hex_prefix(&bytes)),
                ),
                Err(error) => operation_error(error),
            },
            Command::EffectSimulate {
                effect_id,
                session_id,
                semantics,
            } => self.simulate_effect(principal, effect_id, session_id, semantics, now, scope),
            Command::EffectReconcile { effect_id } => {
                self.reconcile_effect(principal, effect_id, now, scope)
            }
            Command::PolicyValidate {
                policy_source,
                schema_source,
            } => match self.kernel.qualify_policy_candidate(
                principal,
                &policy_source,
                &schema_source,
                scope,
            ) {
                Ok(receipt) => reply(
                    ReplyStatus::Ok,
                    format!(
                        "kind=policy decision_id={} policy_bytes={} schema_bytes={} evidence_digest={}\n",
                        hex_prefix(&receipt.authorization_decision_id),
                        receipt.policy_bytes,
                        receipt.schema_bytes,
                        hex_prefix(&receipt.evidence_digest),
                    ),
                ),
                Err(error) => admin_error(error),
            },
            Command::AuthorityQualify { kind } => {
                let kind = match kind {
                    AuthorityQualificationKind::Lease => AdminQualificationKind::Lease,
                    AuthorityQualificationKind::Approval => AdminQualificationKind::Approval,
                    AuthorityQualificationKind::SecretCanary => {
                        AdminQualificationKind::SecretCanary
                    }
                    AuthorityQualificationKind::SandboxProfile => {
                        AdminQualificationKind::SandboxProfile
                    }
                };
                match self.kernel.qualify_admin_surface(principal, kind, scope) {
                    Ok(receipt) => reply(
                        ReplyStatus::Ok,
                        format!(
                            "kind={} decision_id={} resource={} evidence_digest={}\n",
                            receipt.kind,
                            hex_prefix(&receipt.authorization_decision_id),
                            receipt.resource,
                            hex_prefix(&receipt.evidence_digest),
                        ),
                    ),
                    Err(error) => admin_error(error),
                }
            }
            Command::AuthorityExplain { decision_id } => {
                match self
                    .kernel
                    .explain_authorization_decision(principal, decision_id, scope)
                {
                    Ok(explained) => {
                        let mut body = format!(
                            "decision_id={} decision={} principal={} action={} resource={} reason={} hard_guard={} global_seq={} evidence_version={}\ncontext_hash={} lease_id={} lease_generation={} policy_bundle_id={} policy_bundle_hash={} approval_id={}\n",
                            hex_prefix(&explained.decision_id),
                            explained.decision,
                            explained.principal,
                            explained.action,
                            explained.resource,
                            explained.reason_code,
                            explained.hard_guard_result,
                            explained.global_seq,
                            explained.authority_evidence_version,
                            hex_prefix(&explained.context_hash),
                            optional_hex(explained.lease_id.as_ref().map(|value| value.as_slice())),
                            explained
                                .lease_generation
                                .map(|value| value.to_string())
                                .unwrap_or_else(|| "-".to_owned()),
                            optional_hex(
                                explained
                                    .policy_bundle_id
                                    .as_ref()
                                    .map(|value| value.as_slice())
                            ),
                            optional_hex(
                                explained
                                    .policy_bundle_hash
                                    .as_ref()
                                    .map(|value| value.as_slice())
                            ),
                            optional_hex(
                                explained.approval_id.as_ref().map(|value| value.as_slice())
                            ),
                        );
                        for rule_id in explained.matched_rule_ids {
                            body.push_str(&format!("matched_rule_id={rule_id}\n"));
                        }
                        reply(ReplyStatus::Ok, body)
                    }
                    Err(error) => admin_error(error),
                }
            }
            Command::Doctor => match self.kernel.read_recovery_status(principal, scope) {
                Ok(report) => {
                    let mut body = format!(
                        "mode={:?} attention={} issues={}\n",
                        report.mode,
                        report.requires_attention(),
                        report.issues.len()
                    );
                    for issue in report.issues {
                        body.push_str(&format!(
                            "issue={:?} reference={} blocking={} detail={}\n",
                            issue.kind, issue.reference, issue.blocking, issue.detail
                        ));
                    }
                    reply(ReplyStatus::Ok, body)
                }
                Err(error) => operation_error(error),
            },
        }
    }

    fn simulate_effect(
        &mut self,
        principal: Principal<'_>,
        effect_id: EffectId,
        session_id: SessionId,
        semantics: SyntheticSemantics,
        now: &str,
        scope: &str,
    ) -> ReplyMessage {
        let (kernel, handlers) = (&mut self.kernel, &mut self.handlers);
        let handler = handlers.by_semantics_mut(semantics);
        let metadata = handler.metadata();
        let resource = format!("sim:effect:{}", effect_id.0);
        let action = if matches!(semantics, SyntheticSemantics::ReadOnly) {
            "sim.read"
        } else {
            "sim.write"
        };
        let payload_hash = synthetic_payload_hash(effect_id, session_id, semantics);
        let base_intent = HandlerIntent {
            action,
            resource: &resource,
            execution_semantics: metadata.execution_semantics,
            idempotency_key: None,
            payload_hash,
        };
        let idempotency_key = handler.derive_idempotency_key(&base_intent);
        let intent = HandlerIntent {
            idempotency_key: idempotency_key.as_deref(),
            ..base_intent
        };
        let prepared = match kernel.prepare_synthetic_effect(
            principal,
            PrepareSyntheticEffect {
                effect_id,
                session_id,
                execution_semantics: metadata.execution_semantics.as_str(),
                handler_id: metadata.handler_id,
                handler_version: metadata.handler_version,
                idempotency_key: idempotency_key.as_deref(),
                payload_hash,
                started_at: now,
            },
            scope,
        ) {
            Ok(prepared) => prepared,
            Err(error) => return synthetic_error(error),
        };

        let completed = match handler.execute(&intent) {
            HandlerOutcome::Succeeded { receipt } => kernel.complete_synthetic_effect(
                principal,
                CompleteSyntheticEffect {
                    effect_id,
                    attempt_id: prepared.attempt_id(),
                    finished_at: now,
                    completion: SyntheticExecutionCompletion::Succeeded,
                    reason_code: Some("synthetic_handler_succeeded"),
                    evidence_ref: None,
                    receipt: Some(&receipt),
                },
                scope,
            ),
            HandlerOutcome::Failed {
                reason_code,
                receipt,
            } => kernel.complete_synthetic_effect(
                principal,
                CompleteSyntheticEffect {
                    effect_id,
                    attempt_id: prepared.attempt_id(),
                    finished_at: now,
                    completion: SyntheticExecutionCompletion::Failed,
                    reason_code: Some(&reason_code),
                    evidence_ref: None,
                    receipt: receipt.as_deref(),
                },
                scope,
            ),
            HandlerOutcome::Unknown { evidence } => kernel.complete_synthetic_effect(
                principal,
                CompleteSyntheticEffect {
                    effect_id,
                    attempt_id: prepared.attempt_id(),
                    finished_at: now,
                    completion: SyntheticExecutionCompletion::UnknownOutcome,
                    reason_code: Some("synthetic_handler_unknown"),
                    evidence_ref: evidence.as_deref(),
                    receipt: None,
                },
                scope,
            ),
        };

        match completed {
            Ok(outcome) => reply(
                ReplyStatus::Ok,
                format!(
                    "effect_id={} attempt_id={} state={} receipt={}\n",
                    outcome.effect_id.0,
                    outcome.attempt_id.0,
                    outcome.state,
                    outcome
                        .receipt
                        .as_deref()
                        .map(hex_prefix)
                        .unwrap_or_else(|| "-".to_owned())
                ),
            ),
            Err(error) => synthetic_error(error),
        }
    }

    fn reconcile_effect(
        &mut self,
        principal: Principal<'_>,
        effect_id: EffectId,
        now: &str,
        scope: &str,
    ) -> ReplyMessage {
        let (kernel, handlers) = (&mut self.kernel, &mut self.handlers);
        let context = match kernel.begin_synthetic_reconciliation(principal, effect_id, now, scope)
        {
            Ok(context) => context,
            Err(error) => return synthetic_error(error),
        };
        let handler = match handlers.by_handler_id_mut(&context.handler_id) {
            Some(handler) => handler,
            None => {
                return reply(
                    ReplyStatus::Failed,
                    format!(
                        "error=unknown_synthetic_handler handler_id={}\n",
                        context.handler_id
                    ),
                );
            }
        };
        let semantics = match parse_execution_semantics(&context.execution_semantics) {
            Some(value) => value,
            None => {
                return reply(
                    ReplyStatus::Failed,
                    format!(
                        "error=invalid_execution_semantics value={}\n",
                        context.execution_semantics
                    ),
                );
            }
        };
        let attempt_outcome = match parse_attempt_outcome(&context.attempt_outcome) {
            Some(value) => value,
            None => {
                return reply(
                    ReplyStatus::Failed,
                    format!(
                        "error=invalid_attempt_outcome value={}\n",
                        context.attempt_outcome
                    ),
                );
            }
        };
        let intent = HandlerIntent {
            action: &context.action,
            resource: &context.resource,
            execution_semantics: semantics,
            idempotency_key: context.idempotency_key.as_deref(),
            payload_hash: context.payload_hash,
        };
        let prior = PriorAttempt {
            started_global_seq: context.started_global_seq,
            handler_id: &context.handler_id,
            handler_version: &context.handler_version,
            dispatch_token: &context.dispatch_token,
            outcome: attempt_outcome,
            receipt: context.receipt.as_deref(),
        };

        let resolved = match handler.reconcile(&intent, &prior) {
            HandlerOutcome::Succeeded { receipt } => kernel.resolve_synthetic_reconciliation(
                principal,
                ResolveSyntheticReconciliation {
                    effect_id,
                    resolution: SyntheticExecutionCompletion::Succeeded,
                    reason_code: Some("synthetic_reconciled_succeeded"),
                    evidence_ref: Some(&receipt),
                    detected_at: now,
                },
                scope,
            ),
            HandlerOutcome::Failed {
                reason_code,
                receipt,
            } => kernel.resolve_synthetic_reconciliation(
                principal,
                ResolveSyntheticReconciliation {
                    effect_id,
                    resolution: SyntheticExecutionCompletion::Failed,
                    reason_code: Some(&reason_code),
                    evidence_ref: receipt.as_deref(),
                    detected_at: now,
                },
                scope,
            ),
            HandlerOutcome::Unknown { evidence } => kernel.resolve_synthetic_reconciliation(
                principal,
                ResolveSyntheticReconciliation {
                    effect_id,
                    resolution: SyntheticExecutionCompletion::UnknownOutcome,
                    reason_code: Some("synthetic_reconciliation_ambiguous"),
                    evidence_ref: evidence.as_deref(),
                    detected_at: now,
                },
                scope,
            ),
        };

        match resolved {
            Ok(SyntheticReconciliationResult::Resolved { effect_id, state }) => reply(
                ReplyStatus::Ok,
                format!("effect_id={} state={}\n", effect_id.0, state),
            ),
            Ok(SyntheticReconciliationResult::ManualReview {
                effect_id,
                incident_id,
            }) => reply(
                ReplyStatus::Ok,
                format!(
                    "effect_id={} state=manual_review incident_id={}\n",
                    effect_id.0,
                    hex_prefix(&incident_id)
                ),
            ),
            Err(error) => synthetic_error(error),
        }
    }
}

fn parse_attempt_outcome(value: &str) -> Option<HandlerAttemptOutcome> {
    match value {
        "success" => Some(HandlerAttemptOutcome::Success),
        "failure" => Some(HandlerAttemptOutcome::Failure),
        "unknown" => Some(HandlerAttemptOutcome::Unknown),
        _ => None,
    }
}

fn parse_execution_semantics(value: &str) -> Option<EffectSemantics> {
    match value {
        "read_only" => Some(EffectSemantics::ReadOnly),
        "idempotent_at_least_once" => Some(EffectSemantics::IdempotentAtLeastOnce),
        "at_most_once" => Some(EffectSemantics::AtMostOnce),
        "compensatable" => Some(EffectSemantics::Compensatable),
        "irreversible" => Some(EffectSemantics::Irreversible),
        _ => None,
    }
}

fn synthetic_payload_hash(
    effect_id: EffectId,
    session_id: SessionId,
    semantics: SyntheticSemantics,
) -> [u8; 32] {
    let mut hash = [0_u8; 32];
    hash[..16].copy_from_slice(&effect_id.0.to_be_bytes());
    hash[16..].copy_from_slice(&session_id.0.to_be_bytes());
    hash[31] ^= match semantics {
        SyntheticSemantics::ReadOnly => 1,
        SyntheticSemantics::IdempotentAtLeastOnce => 2,
        SyntheticSemantics::AtMostOnce => 3,
        SyntheticSemantics::Compensatable => 4,
        SyntheticSemantics::Irreversible => 5,
    };
    hash
}

fn admin_error(error: AdminSurfaceError) -> ReplyMessage {
    let status = match &error {
        AdminSurfaceError::AuthorizationDenied(_) => ReplyStatus::Denied,
        _ => ReplyStatus::Failed,
    };
    reply(status, format!("error={error}\n"))
}

fn operation_error(error: KernelOperationError) -> ReplyMessage {
    let status = match &error {
        KernelOperationError::Kernel(KernelError::AuthorizationDenied(_)) => ReplyStatus::Denied,
        _ => ReplyStatus::Failed,
    };
    reply(status, format!("error={error}\n"))
}

fn enrollment_error(error: ClientEnrollmentError) -> ReplyMessage {
    let status = match &error {
        ClientEnrollmentError::Kernel(KernelError::AuthorizationDenied(_)) => ReplyStatus::Denied,
        _ => ReplyStatus::Failed,
    };
    reply(status, format!("error={error}\n"))
}

fn synthetic_error(error: SyntheticEffectError) -> ReplyMessage {
    let status = match &error {
        SyntheticEffectError::Kernel(KernelError::AuthorizationDenied(_)) => ReplyStatus::Denied,
        _ => ReplyStatus::Failed,
    };
    reply(status, format!("error={error}\n"))
}

fn reply(status: ReplyStatus, body: String) -> ReplyMessage {
    if body.len() > MAX_REPLY_BODY_BYTES {
        ReplyMessage {
            status: ReplyStatus::Failed,
            body: b"error=reply_body_too_large\n".to_vec(),
        }
    } else {
        ReplyMessage {
            status,
            body: body.into_bytes(),
        }
    }
}

fn optional_hex(value: Option<&[u8]>) -> String {
    value.map(hex_prefix).unwrap_or_else(|| "-".to_owned())
}

fn optional_u128(value: Option<u128>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "-".to_owned())
}

fn hex_prefix(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len().min(32) * 2);
    for byte in bytes.iter().take(32) {
        out.push(char::from(HEX[(byte >> 4) as usize]));
        out.push(char::from(HEX[(byte & 0x0f) as usize]));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use golam_core::paths::RuntimeLayout;
    use golam_core::{ClientId, EventId, ResourceLimits};
    use golam_ipc::command::{Command, encode_command};
    use golam_ipc::request::MethodId;
    use golam_kernel::BootstrapPolicy;
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
        RuntimeLayout::initialize(
            std::env::temp_dir().join(format!("golamd-router-{}-{t}-{n}", std::process::id())),
        )
        .unwrap()
    }

    fn request(command: Command) -> RequestMessage {
        encode_command(&command).unwrap()
    }

    #[test]
    fn client_enrollment_routes_through_kernel_owned_authority() {
        let runtime = runtime();
        let kernel = KernelApi::open(&runtime, BootstrapPolicy::default()).unwrap();
        let mut router = CommandRouter::new(kernel);
        let response = router.route(
            Principal::local_owner("owner"),
            &request(Command::ClientEnroll {
                client_id: ClientId(7000),
            }),
            "2026-08-26T01:30:00Z",
            "local-owner",
        );
        assert_eq!(response.status, ReplyStatus::Ok);
        let text = String::from_utf8(response.body).unwrap();
        assert!(text.contains("client_id=7000"));
        assert!(text.contains("assurance="));
        drop(router);
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn session_commands_route_only_through_kernel_api() {
        let runtime = runtime();
        let kernel = KernelApi::open(&runtime, BootstrapPolicy::default()).unwrap();
        let mut router = CommandRouter::new(kernel);
        let principal = Principal::local_owner("owner");
        let created = router.route(
            principal,
            &request(Command::SessionCreate {
                session_id: SessionId(1),
                event_id: EventId(2),
                recorded_at: "2026-08-26T01:31:00Z".to_owned(),
                payload: b"root".to_vec(),
            }),
            "2026-08-26T01:31:00Z",
            "local-owner",
        );
        assert_eq!(created.status, ReplyStatus::Ok);
        let listed = router.route(
            principal,
            &request(Command::SessionsList),
            "2026-08-26T01:31:01Z",
            "local-owner",
        );
        assert_eq!(listed.status, ReplyStatus::Ok);
        assert!(
            String::from_utf8(listed.body)
                .unwrap()
                .contains("session_id=1")
        );
        drop(router);
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn irreversible_simulator_reconciles_without_redispatch() {
        let runtime = runtime();
        let kernel = KernelApi::open(&runtime, BootstrapPolicy::default()).unwrap();
        let mut router = CommandRouter::new(kernel);
        let principal = Principal::local_owner("owner");
        assert_eq!(
            router
                .route(
                    principal,
                    &request(Command::SessionCreate {
                        session_id: SessionId(10),
                        event_id: EventId(11),
                        recorded_at: "2026-08-26T01:32:00Z".to_owned(),
                        payload: b"root".to_vec(),
                    }),
                    "2026-08-26T01:32:00Z",
                    "local-owner",
                )
                .status,
            ReplyStatus::Ok
        );
        let simulated = router.route(
            principal,
            &request(Command::EffectSimulate {
                effect_id: EffectId(12),
                session_id: SessionId(10),
                semantics: SyntheticSemantics::Irreversible,
            }),
            "2026-08-26T01:32:01Z",
            "local-owner",
        );
        assert_eq!(simulated.status, ReplyStatus::Ok);
        assert!(
            String::from_utf8(simulated.body)
                .unwrap()
                .contains("state=unknown_outcome")
        );
        let reconciled = router.route(
            principal,
            &request(Command::EffectReconcile {
                effect_id: EffectId(12),
            }),
            "2026-08-26T01:32:02Z",
            "local-owner",
        );
        assert_eq!(reconciled.status, ReplyStatus::Ok);
        assert!(
            String::from_utf8(reconciled.body)
                .unwrap()
                .contains("state=succeeded")
        );
        drop(router);
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn authenticated_admin_qualification_routes_through_kernel_api() {
        let runtime = runtime();
        let kernel = KernelApi::open(&runtime, BootstrapPolicy::default()).unwrap();
        let mut router = CommandRouter::new(kernel);
        let principal = Principal::enrolled_client("local-cli", ClientId(9001));
        let qualified = router.route(
            principal,
            &request(Command::AuthorityQualify {
                kind: golam_ipc::command::AuthorityQualificationKind::Lease,
            }),
            "2026-08-29T01:00:00Z",
            "local-ipc",
        );
        assert_eq!(qualified.status, ReplyStatus::Ok);
        let qualified_text = String::from_utf8(qualified.body).unwrap();
        let decision_id = qualified_text
            .split_whitespace()
            .find_map(|field| field.strip_prefix("decision_id="))
            .unwrap();
        let mut parsed = [0_u8; 16];
        for (index, chunk) in decision_id.as_bytes().as_chunks::<2>().0.iter().enumerate() {
            parsed[index] = u8::from_str_radix(std::str::from_utf8(chunk).unwrap(), 16).unwrap();
        }
        let explained = router.route(
            principal,
            &request(Command::AuthorityExplain {
                decision_id: parsed,
            }),
            "2026-08-29T01:00:01Z",
            "local-ipc",
        );
        assert_eq!(explained.status, ReplyStatus::Ok);
        assert!(
            String::from_utf8(explained.body)
                .unwrap()
                .contains("action=authority.qualify")
        );
        drop(router);
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn invalid_command_is_rejected_before_kernel_dispatch() {
        let runtime = runtime();
        let kernel = KernelApi::open(&runtime, BootstrapPolicy::default()).unwrap();
        let mut router = CommandRouter::new(kernel);
        let response = router.route(
            Principal::local_owner("owner"),
            &RequestMessage {
                method: MethodId(65535),
                body: vec![],
            },
            "2026-08-26T01:33:00Z",
            "local-owner",
        );
        assert_eq!(response.status, ReplyStatus::InvalidRequest);
        assert!(response.body.len() < ResourceLimits::default().max_frame_bytes as usize);
        drop(router);
        fs::remove_dir_all(runtime.root).unwrap();
    }
}
