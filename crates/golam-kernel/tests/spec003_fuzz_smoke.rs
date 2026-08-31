use golam_kernel::policy_candidate::{
    CandidatePolicyFailureKind, MAX_CANDIDATE_DIAGNOSTIC_CHARS, MAX_CANDIDATE_DIAGNOSTICS,
    MAX_CANDIDATE_POLICY_BYTES, MAX_CANDIDATE_SCHEMA_BYTES, validate_policy_candidate,
};
use golam_kernel::{CapabilityLeaseScope, CapabilityLeaseScopeError};
use golam_ledger::sandbox_profile::{
    SandboxNetworkRule, SandboxProfileClass, SandboxProfileDefinition, SandboxSpawnRule,
    prepare_sandbox_profile,
};

const VALID_SCHEMA: &str = r#"
entity User;
entity Photo;
action view appliesTo { principal: [User], resource: [Photo] };
"#;

const VALID_POLICY: &str = r#"
permit(
    principal is User,
    action == Action::"view",
    resource is Photo
);
"#;

fn corpus_text(seed: u64, max_len: usize) -> String {
    const ALPHABET: &[u8] = b"permitforbidprincipalactionresourceUserPhoto(){}[],:;=\" abcXYZ0123456789_./\\\n\t";
    let mut state = seed ^ 0x9e37_79b9_7f4a_7c15;
    state ^= state << 13;
    state ^= state >> 7;
    state ^= state << 17;
    let len = usize::try_from(state % u64::try_from(max_len + 1).unwrap()).unwrap();
    let mut value = String::with_capacity(len);
    for _ in 0..len {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        let index = usize::try_from(state % u64::try_from(ALPHABET.len()).unwrap()).unwrap();
        value.push(char::from(ALPHABET[index]));
    }
    value
}

fn assert_bounded_policy_error(error: &golam_kernel::policy_candidate::CandidatePolicyError) {
    assert!(error.diagnostics().len() <= MAX_CANDIDATE_DIAGNOSTICS);
    assert!(
        error
            .diagnostics()
            .iter()
            .all(|diagnostic| diagnostic.chars().count() <= MAX_CANDIDATE_DIAGNOSTIC_CHARS)
    );
}

#[test]
fn policy_and_schema_mutation_corpus_is_bounded_and_fail_closed() {
    for seed in 0_u64..128 {
        let policy = corpus_text(seed, 512);
        if let Err(error) = validate_policy_candidate(&policy, VALID_SCHEMA) {
            assert_bounded_policy_error(&error);
        }

        let schema = corpus_text(seed ^ 0xa5a5_a5a5, 512);
        if let Err(error) = validate_policy_candidate(VALID_POLICY, &schema) {
            assert_bounded_policy_error(&error);
        }
    }

    let oversized_policy = " ".repeat(MAX_CANDIDATE_POLICY_BYTES + 1);
    let error = validate_policy_candidate(&oversized_policy, VALID_SCHEMA).unwrap_err();
    assert_eq!(error.kind(), CandidatePolicyFailureKind::PolicyTooLarge);
    assert_bounded_policy_error(&error);

    let oversized_schema = " ".repeat(MAX_CANDIDATE_SCHEMA_BYTES + 1);
    let error = validate_policy_candidate(VALID_POLICY, &oversized_schema).unwrap_err();
    assert_eq!(error.kind(), CandidatePolicyFailureKind::SchemaTooLarge);
    assert_bounded_policy_error(&error);
}

#[test]
fn capability_scope_mutation_corpus_never_normalizes_nondeterministically() {
    for seed in 0_u64..128 {
        let action = corpus_text(seed ^ 0x1111, 160);
        let resource = corpus_text(seed ^ 0x2222, 256);
        let context = corpus_text(seed ^ 0x3333, 96);
        let first = CapabilityLeaseScope::normalize(&[&action], &[&resource], &[&context]);
        let second = CapabilityLeaseScope::normalize(&[&action], &[&resource], &[&context]);
        assert_eq!(first, second);
        if let Ok(scope) = first {
            assert_eq!(scope.actions(), &[action]);
            assert_eq!(scope.resources(), &[resource]);
            assert_eq!(scope.context_constraints(), &[context]);
        }
    }

    let actions = (0..33).map(|index| format!("action.{index}")).collect::<Vec<_>>();
    let action_refs = actions.iter().map(String::as_str).collect::<Vec<_>>();
    assert_eq!(
        CapabilityLeaseScope::normalize(&action_refs, &["resource:bounded"], &[]),
        Err(CapabilityLeaseScopeError::TooManyActions)
    );
}

#[test]
fn sandbox_profile_validation_corpus_is_deterministic_and_bounded() {
    const ROOTS: &[&str] = &[
        "/tmp",
        "C:/tmp",
        "relative",
        "/tmp/../escape",
        "/tmp//double",
        " /tmp",
        "/tmp\\backslash",
        "",
    ];
    const ENV_NAMES: &[&str] = &["PATH", "SAFE_1", "_OK", "1BAD", " BAD", "", "A-B"];
    const TOKENS: &[&str] = &["native:unqualified", "network", "", " bad", "bad token", "x/y"];

    for seed in 1_usize..=128 {
        let root = ROOTS[seed % ROOTS.len()];
        let env = ENV_NAMES[(seed * 3) % ENV_NAMES.len()];
        let token = TOKENS[(seed * 5) % TOKENS.len()];
        let profile_id = [u8::try_from((seed % 250) + 1).unwrap(); 16];
        let read_roots = [root];
        let env_names = [env];
        let tokens = [token];
        let definition = SandboxProfileDefinition {
            profile_id,
            version: 1,
            class: SandboxProfileClass::NativeUntrustedSubprocess,
            filesystem_read_roots: &read_roots,
            filesystem_write_roots: &[],
            network_rule: SandboxNetworkRule::DenyAll,
            environment_allowlist: &env_names,
            spawn_rule: SandboxSpawnRule::Deny,
            cpu_limit: Some(1),
            memory_limit: Some(1),
            time_limit: Some(1),
            output_limit: Some(1),
            device_allowlist: &tokens,
            ipc_allowlist: &[],
            inherited_handle_rules: &[],
            platform_requirements: &[],
        };
        let first = prepare_sandbox_profile(definition, "owner:owner", [0; 32]);
        let second = prepare_sandbox_profile(definition, "owner:owner", [0; 32]);
        match (first, second) {
            (Ok(first), Ok(second)) => {
                assert_eq!(first.profile_id(), second.profile_id());
                assert_eq!(first.version(), second.version());
                assert_eq!(first.resource(), second.resource());
                assert_eq!(first.intent_digest(), second.intent_digest());
            }
            (Err(first), Err(second)) => assert_eq!(first.to_string(), second.to_string()),
            _ => panic!("identical sandbox input normalized nondeterministically"),
        }
    }
}
