#![forbid(unsafe_code)]

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
mod fcntl {
    pub use golam_core::unix_fs::{OFlag, openat};
}
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
mod sys {
    pub mod stat {
        pub use golam_core::unix_fs::Mode;
    }
}

pub mod local_fs;
pub mod mcp_local_process_v2;
pub mod mcp_protocol;
pub mod native_containment_v2;
pub mod native_process_supervisor_v2;
#[allow(dead_code)]
pub mod process_dispatch_v2;
#[allow(clippy::redundant_guards)]
pub mod process_execution_v2;
pub mod process_secret_evidence;
pub mod skill_packages;
pub mod skill_process_v2;
pub mod static_elf_v2;

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

    pub fn kernel(&self) -> &KernelApi<P> {
        &self.kernel
    }

    pub fn kernel_mut(&mut self) -> &mut KernelApi<P> {
        &mut self.kernel
    }

    pub fn handle(&mut self, request: RequestMessage<'_>) -> ReplyMessage {
        let request_id = request.request_id;
        let command = match decode_command(request.body) {
            Ok(command) => command,
            Err(error) => return ReplyMessage::error(request_id, error.to_string()),
        };

        let reply = match command {
            Command::CreateSession { session_id } => self.handle_create_session(session_id),
            Command::AppendUser {
                session_id,
                message_id,
                user_input,
            } => self.handle_append_user(session_id, message_id, user_input),
            Command::BeginTurn {
                session_id,
                message_id,
                turn_id,
                turn_index,
            } => self.handle_begin_turn(session_id, message_id, turn_id, turn_index),
            Command::CompleteTurn {
                session_id,
                turn_id,
                assistant_message_id,
                assistant_text,
            } => self.handle_complete_turn(
                session_id,
                turn_id,
                assistant_message_id,
                assistant_text,
            ),
            Command::PrepareSynthetic {
                session_id,
                effect_id,
                semantics,
            } => self.handle_prepare_synthetic(session_id, effect_id, semantics),
            Command::ExecuteSynthetic {
                session_id,
                effect_id,
                semantics,
            } => self.handle_execute_synthetic(session_id, effect_id, semantics),
            Command::ReconcileSynthetic {
                session_id,
                effect_id,
                semantics,
                result,
            } => self.handle_reconcile_synthetic(session_id, effect_id, semantics, result),
            Command::EnrollClient {
                client_id,
                key_id,
                client_kind,
                key_material,
            } => self.handle_enroll_client(client_id, key_id, client_kind, key_material),
            Command::RevokeClient { client_id } => self.handle_revoke_client(client_id),
            Command::RotateClientKey {
                client_id,
                old_key_id,
                new_key_id,
                new_key_material,
            } => self.handle_rotate_client_key(
                client_id,
                old_key_id,
                new_key_id,
                new_key_material,
            ),
            Command::AuthorityQualify { kind } => self.handle_authority_qualify(kind),
            Command::AdminQualify { kind } => self.handle_admin_qualify(kind),
        };

        match reply {
            Ok(body) => {
                if body.len() > MAX_REPLY_BODY_BYTES {
                    ReplyMessage::error(request_id, "reply body exceeds daemon bound")
                } else {
                    ReplyMessage::ok(request_id, body)
                }
            }
            Err(error) => ReplyMessage::error(request_id, error),
        }
    }

    fn handle_create_session(&mut self, session_id: SessionId) -> Result<Vec<u8>, String> {
        self.kernel
            .create_session(session_id)
            .map_err(|error| error.to_string())?;
        Ok(Vec::new())
    }

    fn handle_append_user(
        &mut self,
        session_id: SessionId,
        message_id: golam_core::MessageId,
        user_input: Vec<u8>,
    ) -> Result<Vec<u8>, String> {
        self.kernel
            .append_user_message(session_id, message_id, &user_input)
            .map_err(|error| error.to_string())?;
        Ok(Vec::new())
    }

    fn handle_begin_turn(
        &mut self,
        session_id: SessionId,
        message_id: golam_core::MessageId,
        turn_id: golam_core::TurnId,
        turn_index: u64,
    ) -> Result<Vec<u8>, String> {
        self.kernel
            .begin_turn(session_id, message_id, turn_id, turn_index)
            .map_err(|error| error.to_string())?;
        Ok(Vec::new())
    }

    fn handle_complete_turn(
        &mut self,
        session_id: SessionId,
        turn_id: golam_core::TurnId,
        assistant_message_id: golam_core::MessageId,
        assistant_text: Vec<u8>,
    ) -> Result<Vec<u8>, String> {
        self.kernel
            .complete_turn(
                session_id,
                turn_id,
                assistant_message_id,
                assistant_text,
            )
            .map_err(|error| error.to_string())?;
        Ok(Vec::new())
    }

    fn handle_prepare_synthetic(
        &mut self,
        session_id: SessionId,
        effect_id: EffectId,
        semantics: SyntheticSemantics,
    ) -> Result<Vec<u8>, String> {
        let intent = HandlerIntent {
            scope: "ipc.synthetic".to_owned(),
            operation: semantics.operation().to_owned(),
            parameters: Vec::new(),
        };
        let prepared = self
            .kernel
            .prepare_synthetic_effect(PrepareSyntheticEffect {
                session_id,
                effect_id,
                semantics: semantics.effect_semantics(),
                handler_id: semantics.handler_id(),
                handler_version: "1",
                intent: &intent,
            })
            .map_err(|error| error.to_string())?;
        Ok(prepared.intent_digest.to_vec())
    }

    fn handle_execute_synthetic(
        &mut self,
        session_id: SessionId,
        effect_id: EffectId,
        semantics: SyntheticSemantics,
    ) -> Result<Vec<u8>, String> {
        let handler = self
            .handlers
            .by_semantics_mut(semantics);
        let prior_attempt = self
            .kernel
            .synthetic_prior_attempt(session_id, effect_id)
            .map_err(|error| error.to_string())?;
        let attempt = handler
            .execute(HandlerIntent {
                scope: "ipc.synthetic".to_owned(),
                operation: semantics.operation().to_owned(),
                parameters: Vec::new(),
            }, prior_attempt.as_ref())
            .map_err(|error| error.to_string())?;
        let completion = match attempt.outcome {
            HandlerAttemptOutcome::Succeeded { result } => SyntheticExecutionCompletion::Succeeded {
                output: result.output,
                verification: result.verification,
            },
            HandlerAttemptOutcome::Failed { error } => SyntheticExecutionCompletion::Failed { error },
            HandlerAttemptOutcome::UnknownOutcome { evidence } => {
                SyntheticExecutionCompletion::UnknownOutcome { evidence }
            }
        };
        let outcome = self
            .kernel
            .complete_synthetic_effect(CompleteSyntheticEffect {
                session_id,
                effect_id,
                handler_id: semantics.handler_id(),
                handler_version: "1",
                completion,
            })
            .map_err(|error| error.to_string())?;
        encode_synthetic_outcome(&outcome)
    }

    fn handle_reconcile_synthetic(
        &mut self,
        session_id: SessionId,
        effect_id: EffectId,
        semantics: SyntheticSemantics,
        result: golam_effects::ReconciliationResult,
    ) -> Result<Vec<u8>, String> {
        let handler = self
            .handlers
            .by_handler_id_mut(semantics.handler_id())
            .ok_or_else(|| "synthetic handler missing".to_owned())?;
        let reconciled = handler
            .reconcile(HandlerIntent {
                scope: "ipc.synthetic".to_owned(),
                operation: semantics.operation().to_owned(),
                parameters: Vec::new(),
            }, &result)
            .map_err(|error| error.to_string())?;
        let resolution = match reconciled {
            golam_effects::ReconciliationOutcome::Succeeded { output, verification } => {
                SyntheticReconciliationResult::Succeeded { output, verification }
            }
            golam_effects::ReconciliationOutcome::Failed { error } => {
                SyntheticReconciliationResult::Failed { error }
            }
            golam_effects::ReconciliationOutcome::StillUnknown { evidence } => {
                SyntheticReconciliationResult::StillUnknown { evidence }
            }
        };
        let outcome = self
            .kernel
            .resolve_synthetic_reconciliation(ResolveSyntheticReconciliation {
                session_id,
                effect_id,
                handler_id: semantics.handler_id(),
                handler_version: "1",
                result: resolution,
            })
            .map_err(|error| error.to_string())?;
        encode_synthetic_outcome(&outcome)
    }

    fn handle_enroll_client(
        &mut self,
        client_id: golam_core::ClientId,
        key_id: golam_ipc::lifecycle::ClientKeyId,
        client_kind: ClientKind,
        key_material: [u8; 32],
    ) -> Result<Vec<u8>, String> {
        self.kernel
            .enroll_client(client_id, key_id, client_kind, key_material)
            .map_err(|error| error.to_string())?;
        Ok(Vec::new())
    }

    fn handle_revoke_client(&mut self, client_id: golam_core::ClientId) -> Result<Vec<u8>, String> {
        self.kernel
            .revoke_client(client_id)
            .map_err(|error| error.to_string())?;
        Ok(Vec::new())
    }

    fn handle_rotate_client_key(
        &mut self,
        client_id: golam_core::ClientId,
        old_key_id: golam_ipc::lifecycle::ClientKeyId,
        new_key_id: golam_ipc::lifecycle::ClientKeyId,
        new_key_material: [u8; 32],
    ) -> Result<Vec<u8>, String> {
        self.kernel
            .rotate_client_key(client_id, old_key_id, new_key_id, new_key_material)
            .map_err(|error| error.to_string())?;
        Ok(Vec::new())
    }

    fn handle_authority_qualify(&mut self, kind: AuthorityQualificationKind) -> Result<Vec<u8>, String> {
        self.kernel
            .qualify_authority(kind)
            .map_err(|error| error.to_string())
    }

    fn handle_admin_qualify(&mut self, kind: AdminQualificationKind) -> Result<Vec<u8>, String> {
        self.kernel
            .qualify_admin_surface(kind)
            .map_err(|error| error.to_string())
    }
}

fn encode_synthetic_outcome(outcome: &golam_kernel::SyntheticEffectOutcome) -> Result<Vec<u8>, String> {
    let mut encoded = Vec::new();
    encoded.extend_from_slice(&outcome.effect_id.0.to_le_bytes());
    encoded.extend_from_slice(&outcome.intent_digest);
    encoded.push(match outcome.status {
        golam_kernel::SyntheticEffectStatus::Committed => 1,
        golam_kernel::SyntheticEffectStatus::Failed => 2,
        golam_kernel::SyntheticEffectStatus::UnknownOutcome => 3,
    });
    Ok(encoded)
}
