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
                return Err("process_execute_helper_ready_timeout");
            }
'''
if old not in content:
    raise SystemExit("expected READY timeout branch not found")
path.write_text(content.replace(old, new, 1))
