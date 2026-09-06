from pathlib import Path

path = Path("crates/golamd/src/process_dispatch_v2.rs")
content = path.read_text()

old = "const SUPERVISOR_POLL_MS: u64 = 10;\nconst TERMINAL_DRAIN_MS: u64 = 1000;\n"
new = "const SUPERVISOR_POLL_MS: u64 = 10;\nconst HELPER_READY_TIMEOUT_MS: u64 = 5_000;\nconst TERMINAL_DRAIN_MS: u64 = 1000;\n"
if old not in content:
    raise SystemExit("expected supervisor constants not found")
content = content.replace(old, new, 1)

old = '''        let started = Instant::now();
        if let Err(reason) = await_ready(
            &mut child,
            &receiver,
            execution_binding_digest,
            input.limits.wall_time_ms,
            started,
        ) {
'''
new = '''        let helper_started = Instant::now();
        if let Err(reason) = await_ready(
            &mut child,
            &receiver,
            execution_binding_digest,
            helper_started,
        ) {
'''
if old not in content:
    raise SystemExit("expected helper ready call not found")
content = content.replace(old, new, 1)

old = '''        let binding = RootContainmentBinding {
'''
new = '''        let started = Instant::now();
        let binding = RootContainmentBinding {
'''
if old not in content:
    raise SystemExit("expected containment binding not found")
content = content.replace(old, new, 1)

old = '''    fn await_ready(
        child: &mut Child,
        receiver: &Receiver<StreamEvent>,
        expected_binding: [u8; 32],
        wall_time_ms: u64,
        started: Instant,
    ) -> Result<(), &'static str> {
'''
new = '''    fn await_ready(
        child: &mut Child,
        receiver: &Receiver<StreamEvent>,
        expected_binding: [u8; 32],
        started: Instant,
    ) -> Result<(), &'static str> {
'''
if old not in content:
    raise SystemExit("expected await_ready signature not found")
content = content.replace(old, new, 1)

old = '''            if started.elapsed().as_millis() >= u128::from(wall_time_ms) {
                return Err("process_execute_helper_ready_timeout");
            }
'''
new = '''            if started.elapsed().as_millis() >= u128::from(HELPER_READY_TIMEOUT_MS) {
                return Err("process_execute_helper_ready_timeout");
            }
'''
if old not in content:
    raise SystemExit("expected helper ready timeout guard not found")
content = content.replace(old, new, 1)

path.write_text(content)
