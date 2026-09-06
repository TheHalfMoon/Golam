from pathlib import Path

path = Path("crates/golamd/src/process_dispatch_v2.rs")
content = path.read_text()
old = '''                Ok(StreamEvent::Ready(_)) => {
                    return Err("process_execute_helper_ready_binding_mismatch");
                }
'''
new = '''                Ok(StreamEvent::Ready(line)) => {
                    eprintln!(
                        "SPEC005_T080_READY_DIAG={}",
                        String::from_utf8_lossy(&line)
                    );
                    return Err("process_execute_helper_ready_binding_mismatch");
                }
'''
if old not in content:
    raise SystemExit("expected READY mismatch branch not found")
content = content.replace(old, new, 1)

old = '''            if started.elapsed().as_millis() >= u128::from(wall_time_ms) {
                return Err("process_execute_helper_ready_timeout");
            }
'''
new = '''            if started.elapsed().as_millis() >= u128::from(wall_time_ms) {
                let pid = child.id();
                eprintln!(
                    "SPEC005_T080_HELPER_WCHAN={:?}",
                    fs::read_to_string(format!("/proc/{pid}/wchan"))
                );
                eprintln!(
                    "SPEC005_T080_HELPER_SYSCALL={:?}",
                    fs::read_to_string(format!("/proc/{pid}/syscall"))
                );
                eprintln!(
                    "SPEC005_T080_HELPER_STATUS={:?}",
                    fs::read_to_string(format!("/proc/{pid}/status"))
                );
                eprintln!(
                    "SPEC005_T080_HELPER_MILESTONE={:?}",
                    fs::read_to_string(format!("/tmp/golam-spec005-t080-{pid}.diag"))
                );
                return Err("process_execute_helper_ready_timeout");
            }
'''
if old not in content:
    raise SystemExit("expected READY timeout branch not found")
path.write_text(content.replace(old, new, 1))

helper_path = Path("crates/golamd/src/bin/golam-native-exec-helper-v2.rs")
helper = helper_path.read_text()
old = '''        close_inherited_descriptors()?;
        let config = parse_args()?;
        bind_parent_death(config.expected_parent_pid)?;

        let initial_file = File::open(&config.executable_path)
'''
new = '''        write_ready_diag("before-close");
        close_inherited_descriptors()?;
        write_ready_diag("after-close");
        let config = parse_args()?;
        write_ready_diag("after-parse");
        bind_parent_death(config.expected_parent_pid)?;
        write_ready_diag("after-parent-bind");

        let initial_file = File::open(&config.executable_path)
'''
if old not in helper:
    raise SystemExit("expected helper pre-containment milestone insertion point missing")
helper = helper.replace(old, new, 1)
old = '''        verify_staged_file(&initial_file, &config)?;
        drop(initial_file);

        let cwd = observe_expected_object(&config.cwd, "cwd")?;
'''
new = '''        verify_staged_file(&initial_file, &config)?;
        drop(initial_file);
        write_ready_diag("after-staged-verify");

        let cwd = observe_expected_object(&config.cwd, "cwd")?;
        write_ready_diag("after-cwd-observe");
'''
if old not in helper:
    raise SystemExit("expected helper staged verification milestone insertion point missing")
helper = helper.replace(old, new, 1)
old = '''        let filesystem_write_roots = config
            .write_roots
            .iter()
            .map(|expected| observe_expected_object(expected, "write root"))
            .collect::<Result<Vec<_>, _>>()?;

        std::env::set_current_dir(&config.cwd.path)
'''
new = '''        let filesystem_write_roots = config
            .write_roots
            .iter()
            .map(|expected| observe_expected_object(expected, "write root"))
            .collect::<Result<Vec<_>, _>>()?;
        write_ready_diag("after-root-observe");

        std::env::set_current_dir(&config.cwd.path)
'''
if old not in helper:
    raise SystemExit("expected helper root observation milestone insertion point missing")
helper = helper.replace(old, new, 1)
old = '''        let receipt = apply_child_side(&plan)
            .map_err(|error| format!("apply admitted containment profile: {error}"))?;
'''
new = '''        write_ready_diag("before-apply-child-side");
        let receipt = apply_child_side(&plan)
            .map_err(|error| format!("apply admitted containment profile: {error}"))?;
'''
if old not in helper:
    raise SystemExit("expected helper containment milestone insertion point missing")
helper = helper.replace(old, new, 1)
old = '''    fn close_inherited_descriptors() -> Result<(), String> {
'''
new = '''    fn write_ready_diag(label: &str) {
        let _ = fs::write(
            format!("/tmp/golam-spec005-t080-{}.diag", std::process::id()),
            label,
        );
    }

    fn close_inherited_descriptors() -> Result<(), String> {
'''
if old not in helper:
    raise SystemExit("expected helper diagnostic function insertion point missing")
helper_path.write_text(helper.replace(old, new, 1))
