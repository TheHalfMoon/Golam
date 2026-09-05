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
path.write_text(content.replace(old, new, 1))
