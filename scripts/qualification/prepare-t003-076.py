from pathlib import Path

path = Path("crates/golam-ledger/src/sandbox_executor.rs")
text = path.read_text()

old = '''    if !descriptor.filesystem_read_roots.is_empty() {
        controls.push(CONTROL_FS_READ_ROOTS.to_owned());
    }
    if !descriptor.filesystem_write_roots.is_empty() {
        controls.push(CONTROL_FS_WRITE_ROOTS.to_owned());
    }
'''
new = '''    // Empty filesystem allowlists mean deny-all, not "no filesystem control needed".
    // The executor must therefore prove that it can enforce both read and write roots for
    // every descriptor, including the empty-set case.
    controls.push(CONTROL_FS_READ_ROOTS.to_owned());
    controls.push(CONTROL_FS_WRITE_ROOTS.to_owned());
'''
if text.count(old) != 1:
    raise SystemExit(f"filesystem control anchor count {text.count(old)}")
text = text.replace(old, new, 1)

old = '''    if !descriptor.devices.is_empty() {
        controls.push(CONTROL_DEVICE_ALLOWLIST.to_owned());
    }
    if !descriptor.ipc_endpoints.is_empty() {
        controls.push(CONTROL_IPC_ALLOWLIST.to_owned());
    }
    if !descriptor.inherited_handle_rules.is_empty() {
        controls.push(CONTROL_HANDLE_ALLOWLIST.to_owned());
    }
'''
new = '''    // Empty allowlists are explicit deny-all policies. Requiring the allowlist primitives
    // unconditionally prevents an executor from treating an empty descriptor as ambient
    // device, IPC, or inherited-handle authority.
    controls.push(CONTROL_DEVICE_ALLOWLIST.to_owned());
    controls.push(CONTROL_IPC_ALLOWLIST.to_owned());
    controls.push(CONTROL_HANDLE_ALLOWLIST.to_owned());
'''
if text.count(old) != 1:
    raise SystemExit(f"device/ipc/handle control anchor count {text.count(old)}")
text = text.replace(old, new, 1)

old_test = '''    #[test]
    fn deny_network_and_spawn_are_still_required_controls() {
        let mut plan = plan();
        plan.network_rule = SandboxNetworkRule::DenyAll;
        plan.egress = None;
        plan.spawn_rule = SandboxSpawnRule::Deny;
        let descriptor = resolve_sandbox_enforcement(&plan, SandboxRequestedRights::deny_all())
            .expect("deny-all descriptor");
        let controls = required_controls(&plan, &descriptor);
        assert!(
            controls
                .iter()
                .any(|control| control == CONTROL_NETWORK_DENY)
        );
        assert!(controls.iter().any(|control| control == CONTROL_SPAWN_DENY));
        assert!(
            controls
                .iter()
                .any(|control| control == CONTROL_ENVIRONMENT_CLEAR)
        );
    }
'''
new_test = '''    #[test]
    fn deny_all_rights_still_require_explicit_executor_controls() {
        let mut plan = plan();
        plan.filesystem_read_roots.clear();
        plan.filesystem_write_roots.clear();
        plan.network_rule = SandboxNetworkRule::DenyAll;
        plan.environment_allowlist.clear();
        plan.spawn_rule = SandboxSpawnRule::Deny;
        plan.cpu_limit = None;
        plan.memory_limit = None;
        plan.time_limit = None;
        plan.output_limit = None;
        plan.device_allowlist.clear();
        plan.ipc_allowlist.clear();
        plan.inherited_handle_rules.clear();
        plan.platform_requirements.clear();
        plan.egress = None;
        plan.locality = SandboxLocality::StrictLocal;

        let descriptor = resolve_sandbox_enforcement(&plan, SandboxRequestedRights::deny_all())
            .expect("deny-all descriptor");
        let controls = required_controls(&plan, &descriptor);
        for required in [
            CONTROL_ENVIRONMENT_CLEAR,
            CONTROL_FS_READ_ROOTS,
            CONTROL_FS_WRITE_ROOTS,
            CONTROL_NETWORK_DENY,
            CONTROL_SPAWN_DENY,
            CONTROL_DEVICE_ALLOWLIST,
            CONTROL_IPC_ALLOWLIST,
            CONTROL_HANDLE_ALLOWLIST,
        ] {
            assert!(
                controls.iter().any(|control| control == required),
                "deny-all descriptor omitted required executor control {required}"
            );
        }
    }

    #[test]
    fn empty_allowlist_enforcement_cannot_be_omitted_from_manifest() {
        let mut plan = plan();
        plan.filesystem_read_roots.clear();
        plan.filesystem_write_roots.clear();
        plan.network_rule = SandboxNetworkRule::DenyAll;
        plan.environment_allowlist.clear();
        plan.spawn_rule = SandboxSpawnRule::Deny;
        plan.cpu_limit = None;
        plan.memory_limit = None;
        plan.time_limit = None;
        plan.output_limit = None;
        plan.device_allowlist.clear();
        plan.ipc_allowlist.clear();
        plan.inherited_handle_rules.clear();
        plan.platform_requirements.clear();
        plan.egress = None;
        plan.locality = SandboxLocality::StrictLocal;

        let descriptor = resolve_sandbox_enforcement(&plan, SandboxRequestedRights::deny_all())
            .expect("deny-all descriptor");
        let complete = [
            CONTROL_ENVIRONMENT_CLEAR,
            CONTROL_FS_READ_ROOTS,
            CONTROL_FS_WRITE_ROOTS,
            CONTROL_NETWORK_DENY,
            CONTROL_SPAWN_DENY,
            CONTROL_DEVICE_ALLOWLIST,
            CONTROL_IPC_ALLOWLIST,
            CONTROL_HANDLE_ALLOWLIST,
        ];

        for omitted in [
            CONTROL_FS_READ_ROOTS,
            CONTROL_FS_WRITE_ROOTS,
            CONTROL_DEVICE_ALLOWLIST,
            CONTROL_IPC_ALLOWLIST,
            CONTROL_HANDLE_ALLOWLIST,
        ] {
            let controls: Vec<_> = complete
                .iter()
                .copied()
                .filter(|control| *control != omitted)
                .collect();
            let capabilities = PlatformExecutorCapabilities::from_trusted_manifest(
                "native:deny-all-test",
                &controls,
            )
            .expect("manifest");
            assert!(matches!(
                resolve_platform_executor_capabilities(&plan, &descriptor, &capabilities),
                Err(SandboxExecutorError::UnsupportedRequiredControl(control)) if control == omitted
            ));
        }
    }

    #[test]
    fn every_declared_resource_bound_requires_executor_support() {
        let plan = plan();
        let descriptor = descriptor(&plan);

        for omitted in [
            CONTROL_RESOURCE_CPU,
            CONTROL_RESOURCE_MEMORY,
            CONTROL_RESOURCE_TIME,
            CONTROL_RESOURCE_OUTPUT,
        ] {
            let controls: Vec<_> = all_controls()
                .into_iter()
                .filter(|control| *control != omitted)
                .collect();
            let capabilities = PlatformExecutorCapabilities::from_trusted_manifest(
                "native:resource-test",
                &controls,
            )
            .expect("manifest");
            assert!(matches!(
                resolve_platform_executor_capabilities(&plan, &descriptor, &capabilities),
                Err(SandboxExecutorError::UnsupportedRequiredControl(control)) if control == omitted
            ));
        }
    }
'''
if text.count(old_test) != 1:
    raise SystemExit(f"existing deny-all test anchor count {text.count(old_test)}")
text = text.replace(old_test, new_test, 1)
path.write_text(text)

# Add a consolidated adversarial contract test without exposing trusted capability constructors.
enforcement = Path("crates/golam-ledger/src/sandbox_enforcement.rs")
etext = enforcement.read_text()
anchor = '''    #[test]
    fn resource_requests_can_only_tighten_profile_bounds() {'''
if etext.count(anchor) != 1:
    raise SystemExit(f"resource test anchor count {etext.count(anchor)}")
insert = '''    #[test]
    fn adversarial_rights_widening_is_denied_across_authority_axes() {
        let plan = plan();

        let mut request = allowed_request();
        request.filesystem_read_roots = &["/host/private"];
        assert!(matches!(
            resolve_sandbox_enforcement(&plan, request),
            Err(SandboxEnforcementError::UndeclaredFilesystemRead)
        ));

        let mut request = allowed_request();
        request.filesystem_write_roots = &["/host/private"];
        assert!(matches!(
            resolve_sandbox_enforcement(&plan, request),
            Err(SandboxEnforcementError::UndeclaredFilesystemWrite)
        ));

        let mut request = allowed_request();
        request.environment_names = &["GOLAM_AMBIENT_SECRET"];
        assert!(matches!(
            resolve_sandbox_enforcement(&plan, request),
            Err(SandboxEnforcementError::UndeclaredEnvironmentName)
        ));

        let mut request = allowed_request();
        request.devices = &["device:disk0"];
        assert!(matches!(
            resolve_sandbox_enforcement(&plan, request),
            Err(SandboxEnforcementError::UndeclaredDevice)
        ));

        let mut request = allowed_request();
        request.ipc_endpoints = &["ipc:host-daemon"];
        assert!(matches!(
            resolve_sandbox_enforcement(&plan, request),
            Err(SandboxEnforcementError::UndeclaredIpcEndpoint)
        ));

        let mut request = allowed_request();
        request.inherited_handle_rules = &["handle:ambient"];
        assert!(matches!(
            resolve_sandbox_enforcement(&plan, request),
            Err(SandboxEnforcementError::UndeclaredInheritedHandleRule)
        ));

        let mut strict = plan.clone();
        strict.locality = SandboxLocality::StrictLocal;
        let mut request = allowed_request();
        request.network = SandboxNetworkRequest::PermitBoundExternal;
        assert!(matches!(
            resolve_sandbox_enforcement(&strict, request),
            Err(SandboxEnforcementError::NetworkWidening)
        ));

        let mut deny_spawn = plan.clone();
        deny_spawn.spawn_rule = SandboxSpawnRule::Deny;
        let mut request = allowed_request();
        request.spawn_rule = SandboxSpawnRule::DirectChildOnly;
        assert!(matches!(
            resolve_sandbox_enforcement(&deny_spawn, request),
            Err(SandboxEnforcementError::SpawnWidening)
        ));
    }

'''
etext = etext.replace(anchor, insert + anchor, 1)
enforcement.write_text(etext)
