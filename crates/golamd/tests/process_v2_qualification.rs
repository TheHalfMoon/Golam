#![forbid(unsafe_code)]

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
mod linux_x86_64 {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::AtomicBool;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    use golam_core::harness::ToolCallCandidateId;
    use golam_core::paths::RuntimeLayout;
    use golam_core::taint::TaintSet;
    use golam_core::tool_descriptor::{ToolId, ToolVersion};
    use golam_core::tool_request::{
        BindingDigest, PrincipalId, RequestedOperationId, RequestedTarget, ResourceClassId,
        ToolRequest, ToolRequestId,
    };
    use golam_core::{EffectId, EffectTransitionId, EventId, SessionId};
    use golam_kernel::policy_lifecycle::IssueApproval;
    use golam_kernel::{
        AuthorizationContext, AuthorizationDecision, AuthorizationPolicy, AuthorizationRequest,
        CapabilityLease, CapabilityLeaseScope, KernelApi, KernelCreateSession, PolicyDecision,
        Principal,
    };
    use golam_ledger::approvals::ApprovalScope;
    use golam_ledger::capability_leases::{
        CAPABILITY_LEASE_ISSUE_ACTION, CAPABILITY_LEASE_MUTATION_RISK_CLASS,
    };
    use golam_ledger::dispatch::encode_effect_dependencies;
    use golam_ledger::effects::{CompareAndSwapEffect, EffectStore, ProposeEffect};
    use golamd::local_fs::LocalFsResolver;
    use golamd::process_dispatch_v2::{
        ExecuteStagedProcessV2, ProcessExecutionLimitsV2, ProcessExecutionStatusV2,
        capability_context_ref_v2, execute_staged_process_v2,
    };
    use golamd::process_execution_v2::{
        PROCESS_EXECUTE_ACTION, StageProcessExecutable, stage_process_executable_v2,
    };

    static N: AtomicU64 = AtomicU64::new(0);
    const SCOPE: &str = "local-owner";
    const LEASE_START: &str = "2026-09-05T00:00:00Z";
    const LEASE_END: &str = "2026-09-06T00:00:00Z";
    const OBSERVED_MS: u64 = 1_787_000_000_000;

    struct QualificationPolicy;

    impl AuthorizationPolicy for QualificationPolicy {
        fn authorize(&self, request: &AuthorizationRequest<'_>) -> PolicyDecision {
            if request.context.scope == SCOPE {
                PolicyDecision::allow("spec005_process_v2_qualification_allow")
            } else {
                PolicyDecision::deny("spec005_process_v2_qualification_scope_denied")
            }
        }
    }

    struct Fixture {
        runtime: RuntimeLayout,
        source_root: PathBuf,
        staging_root: PathBuf,
        helper: PathBuf,
        kernel: KernelApi<QualificationPolicy>,
        lease: CapabilityLease,
        resolver: LocalFsResolver,
        session_id: SessionId,
    }

    impl Fixture {
        fn new(request_ids: &[u128]) -> Self {
            let n = N.fetch_add(1, Ordering::Relaxed);
            let t = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock after epoch")
                .as_nanos();
            let root = std::env::temp_dir().join(format!(
                "golam-process-v2-qualification-{}-{t}-{n}",
                std::process::id()
            ));
            let runtime = RuntimeLayout::initialize(root.join("runtime")).expect("runtime");
            let source_root = root.join("source");
            let staging_root = root.join("stage");
            fs::create_dir_all(&source_root).expect("source root");
            fs::create_dir_all(&staging_root).expect("staging root");
            fs::set_permissions(&source_root, fs::Permissions::from_mode(0o700))
                .expect("source mode");
            fs::set_permissions(&staging_root, fs::Permissions::from_mode(0o700))
                .expect("stage mode");

            let payload_source = PathBuf::from(
                std::env::var_os("GOLAM_PROCESS_V2_PAYLOAD")
                    .expect("GOLAM_PROCESS_V2_PAYLOAD must name the static qualification payload"),
            );
            let payload = source_root.join("payload");
            fs::copy(&payload_source, &payload).expect("copy static payload");
            fs::set_permissions(&payload, fs::Permissions::from_mode(0o500)).expect("payload mode");
            let helper = PathBuf::from(
                std::env::var_os("GOLAM_PROCESS_V2_HELPER")
                    .expect("GOLAM_PROCESS_V2_HELPER must name the trusted sibling helper"),
            );

            let mut kernel = KernelApi::open(&runtime, QualificationPolicy).expect("kernel");
            let session_id = SessionId(0x5000);
            kernel
                .create_session(
                    Principal::local_owner("executor"),
                    KernelCreateSession {
                        session_id,
                        event_id: EventId(0x5001),
                        recorded_at: "2026-09-05T19:15:00Z",
                        payload: b"spec005-process-v2-qualification",
                    },
                    SCOPE,
                )
                .expect("qualification session");

            let resources = request_ids
                .iter()
                .map(|id| format!("process-request:{id}"))
                .collect::<Vec<_>>();
            let resource_refs = resources.iter().map(String::as_str).collect::<Vec<_>>();
            let lease_scope =
                CapabilityLeaseScope::normalize(&[PROCESS_EXECUTE_ACTION], &resource_refs, &[])
                    .expect("lease scope");
            let lease = issue_executor_lease(&runtime, &mut kernel, lease_scope);

            let resolver = LocalFsResolver::new(
                &source_root,
                ResourceClassId::new("qualification.process-source").expect("resource class"),
                vec![RequestedOperationId::new(PROCESS_EXECUTE_ACTION).expect("operation")],
                [runtime.root.clone()],
            )
            .expect("source resolver");

            Self {
                runtime,
                source_root,
                staging_root,
                helper,
                kernel,
                lease,
                resolver,
                session_id,
            }
        }

        fn prepared_request(
            &self,
            request_id: u128,
        ) -> golam_core::tool_request::PreparedToolRequest {
            let requested = RequestedTarget::new("payload").expect("requested target");
            let operation = RequestedOperationId::new(PROCESS_EXECUTE_ACTION).expect("operation");
            let resolved = self
                .resolver
                .resolve_read_target(&requested, &operation, OBSERVED_MS)
                .expect("resolve payload");
            let target_identity = resolved
                .resolved_target_identity
                .expect("existing payload identity");
            let lease_evidence = self
                .kernel
                .validate_capability_lease_use(
                    &self.lease,
                    "executor",
                    PROCESS_EXECUTE_ACTION,
                    &format!("process-request:{request_id}"),
                    &[],
                    "2026-09-05T19:16:00Z",
                )
                .expect("live lease");
            ToolRequest {
                request_id: ToolRequestId::from_u128(request_id),
                initiating_principal: PrincipalId::new("executor").expect("principal"),
                tool_id: ToolId::new("process.exec").expect("tool id"),
                tool_version: ToolVersion::new("2").expect("tool version"),
                candidate_ref: ToolCallCandidateId::from_u128(request_id),
                requested_operation: operation,
                requested_target: Some(requested),
                authorized_resource_class: self
                    .resolver
                    .authorized_root()
                    .policy_resource_class
                    .clone(),
                target_identity_ref: Some(target_identity),
                target_resolution_plan_ref: None,
                capability_context_ref: capability_context_ref_v2(lease_evidence)
                    .expect("capability context"),
                taint_set: TaintSet::empty(),
                provenance_refs: vec![],
                idempotency_material: BindingDigest::new([request_id as u8; 32]),
                current_preconditions: vec![resolved.observed_metadata_digest],
                created_at_unix_ms: OBSERVED_MS,
            }
            .prepare()
            .expect("prepared request")
        }

        fn execute(
            &mut self,
            request_id: u128,
            effect_ids: (u128, u128),
            argv: &[&[u8]],
            wall_time_ms: u64,
            max_output: u64,
            cancelled: bool,
        ) -> golamd::process_dispatch_v2::ProcessExecutionReceiptV2 {
            let request = self.prepared_request(request_id);
            let staged = stage_process_executable_v2(
                &mut self.kernel,
                Principal::local_owner("executor"),
                StageProcessExecutable {
                    request: &request,
                    source_resolver: &self.resolver,
                    staging_parent: &self.staging_root,
                    stage_effect_id: EffectId(effect_ids.0),
                    session_id: self.session_id,
                    started_at: "2026-09-05T19:17:00Z",
                    observed_at_unix_ms: OBSERVED_MS,
                },
                SCOPE,
            )
            .expect("stage executable");
            let cancellation = AtomicBool::new(cancelled);
            let argv = argv.iter().map(|value| value.to_vec()).collect::<Vec<_>>();
            execute_staged_process_v2(
                &mut self.kernel,
                Principal::local_owner("executor"),
                ExecuteStagedProcessV2 {
                    request: &request,
                    lease: &self.lease,
                    staged: &staged,
                    helper_path: &self.helper,
                    cwd: &self.source_root,
                    filesystem_read_paths: &[],
                    filesystem_write_paths: &[],
                    argv: &argv,
                    limits: ProcessExecutionLimitsV2 {
                        cpu_seconds: 5,
                        address_space_bytes: 128 * 1024 * 1024,
                        max_created_file_bytes: 1024 * 1024,
                        max_open_files: 32,
                        wall_time_ms,
                        max_stdout_stderr_bytes: max_output,
                    },
                    execute_effect_id: EffectId(effect_ids.1),
                    session_id: self.session_id,
                    started_at: "2026-09-05T19:18:00Z",
                    dispatch_at: "2026-09-05T19:18:01Z",
                    finished_at: "2026-09-05T19:18:02Z",
                    cancellation: &cancellation,
                },
                SCOPE,
            )
            .expect("execute staged process")
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let root = self
                .runtime
                .root
                .parent()
                .map(Path::to_path_buf)
                .unwrap_or_else(|| self.runtime.root.clone());
            let _ = fs::remove_dir_all(root);
        }
    }

    fn issue_executor_lease(
        runtime: &RuntimeLayout,
        kernel: &mut KernelApi<QualificationPolicy>,
        scope: CapabilityLeaseScope,
    ) -> CapabilityLease {
        let (resource, payload_hash) = kernel
            .capability_lease_issue_effect_binding(
                "executor",
                None,
                &scope,
                Some(LEASE_START),
                Some(LEASE_END),
            )
            .expect("lease binding");
        let effect_id = EffectId(0x6000);
        let dependencies = encode_effect_dependencies(&[]).expect("dependencies");
        let authority =
            golam_core::authority::AuthorityLayout::initialize(runtime).expect("authority");
        let mut effects = EffectStore::open(&authority).expect("effect store");
        effects
            .propose(ProposeEffect {
                effect_id,
                session_id: SessionId(0x5000),
                requested_by: "issuer",
                action: CAPABILITY_LEASE_ISSUE_ACTION,
                resource: &resource,
                risk_class: CAPABILITY_LEASE_MUTATION_RISK_CLASS,
                execution_semantics: "at_most_once",
                idempotency_key: None,
                preconditions: b"[]",
                dependencies: &dependencies,
                payload_hash,
                proposed_event_id: EventId(0x6001),
                transition_id: EffectTransitionId(0x6002),
            })
            .expect("propose lease effect");
        effects
            .compare_and_swap(CompareAndSwapEffect {
                transition_id: EffectTransitionId(0x6003),
                effect_id,
                expected_state: "proposed",
                next_state: "authorized",
                attempt_id: None,
                reason_code: Some("spec005_process_qualification_lease_effect"),
                evidence_ref: None,
                event_id: EventId(0x6004),
            })
            .expect("authorize lease effect");
        drop(effects);

        let approval = kernel
            .issue_approval(IssueApproval {
                principal: Principal::local_owner("issuer"),
                approval_scope: ApprovalScope::once(
                    effect_id,
                    CAPABILITY_LEASE_ISSUE_ACTION,
                    &resource,
                )
                .expect("approval scope"),
                risk_class: CAPABILITY_LEASE_MUTATION_RISK_CLASS,
                taint_digest: [0; 32],
                issued_at: "2026-09-05T19:15:30Z",
                expires_at: None,
                max_uses: 1,
                issue_effect_id: EffectId(0x6010),
                authorization_scope: SCOPE,
            })
            .expect("lease approval");
        let decision = kernel
            .authorize(&AuthorizationRequest {
                principal: Principal::local_owner("issuer"),
                action: CAPABILITY_LEASE_ISSUE_ACTION,
                resource: &resource,
                context: AuthorizationContext::local(SCOPE),
            })
            .expect("lease authorization");
        assert_eq!(decision.decision, AuthorizationDecision::Allow);
        kernel
            .issue_capability_lease(
                "executor",
                None,
                scope,
                Some(LEASE_START),
                Some(LEASE_END),
                (decision.decision_id, approval.approval_id(), effect_id),
            )
            .expect("sealed executor lease")
    }

    #[test]
    #[ignore = "dedicated Linux x86_64 production process qualification; invoked explicitly by CI"]
    fn process_v2_requalifies_strict_local_secrets_limits_cancel_and_terminal_reconciliation() {
        let mut fixture = Fixture::new(&[100, 101, 102, 103, 104]);

        let success = fixture.execute(100, (0x7000, 0x7100), &[b"success"], 2_000, 4096, false);
        assert_eq!(success.status, ProcessExecutionStatusV2::Succeeded);
        assert_eq!(success.exit_code, Some(0));
        assert_eq!(success.observed_descendant_count, 0);
        assert_eq!(success.stdout, b"SUCCESS\n");

        let isolation = fixture.execute(101, (0x7200, 0x7300), &[b"isolation"], 2_000, 4096, false);
        assert_eq!(isolation.status, ProcessExecutionStatusV2::Succeeded);
        assert_eq!(
            isolation.stdout,
            b"ENV_EMPTY\nNETWORK_DENIED\nSPAWN_DENIED\n"
        );
        assert_eq!(isolation.observed_descendant_count, 0);

        let timeout = fixture.execute(102, (0x7400, 0x7500), &[b"spin"], 100, 4096, false);
        assert_eq!(timeout.status, ProcessExecutionStatusV2::TimedOut);
        assert_eq!(timeout.observed_descendant_count, 0);

        let output = fixture.execute(103, (0x7600, 0x7700), &[b"output"], 2_000, 128, false);
        assert_eq!(output.status, ProcessExecutionStatusV2::OutputLimitExceeded);
        assert!(output.stdout.len() <= 128);
        assert_eq!(output.observed_descendant_count, 0);

        let cancelled = fixture.execute(104, (0x7800, 0x7900), &[b"spin"], 2_000, 4096, true);
        assert_eq!(cancelled.status, ProcessExecutionStatusV2::Cancelled);
        assert_eq!(cancelled.observed_descendant_count, 0);
    }

    #[test]
    #[ignore = "dedicated Linux x86_64 restart qualification; invoked explicitly by CI"]
    fn interrupted_process_effect_restarts_as_unknown_before_reconciliation() {
        use golam_core::ToolReconciliationResolution;
        use golam_kernel::PrepareToolEffect;

        let mut fixture = Fixture::new(&[200]);
        let resource = "process-request:200";
        let effect_id = EffectId(0x7a00);
        fixture
            .kernel
            .prepare_tool_effect(
                Principal::local_owner("executor"),
                PrepareToolEffect {
                    effect_id,
                    session_id: fixture.session_id,
                    action: PROCESS_EXECUTE_ACTION,
                    resource,
                    execution_semantics: "at_most_once",
                    handler_id: "golam-native-exec-linux-x86_64",
                    handler_version: "2",
                    idempotency_key: Some(resource),
                    preconditions_hash: [0x41; 32],
                    payload_hash: [0x42; 32],
                    started_at: "2026-09-05T19:20:00Z",
                },
                SCOPE,
            )
            .expect("prepare interrupted process effect");
        let mut restarted =
            KernelApi::open(&fixture.runtime, QualificationPolicy).expect("restart kernel");
        let context = restarted
            .begin_tool_reconciliation(
                Principal::local_owner("executor"),
                effect_id,
                "2026-09-05T19:20:01Z",
                SCOPE,
            )
            .expect("restart reconciliation");
        assert_eq!(context.effect_id, effect_id);
        let resolution = restarted
            .resolve_tool_reconciliation(
                Principal::local_owner("executor"),
                effect_id,
                ToolReconciliationResolution::UnknownOutcome,
                Some("process_restart_terminal_state_unproven"),
                None,
                "2026-09-05T19:20:02Z",
                SCOPE,
            )
            .expect("manual-review unresolved process effect");
        assert!(matches!(
            resolution,
            golam_core::ToolReconciliationResult::ManualReview { .. }
        ));
        fixture.kernel = restarted;
    }
}
