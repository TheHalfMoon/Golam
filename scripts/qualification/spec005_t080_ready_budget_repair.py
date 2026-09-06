from pathlib import Path


def replace_once(path: str, old: str, new: str) -> None:
    file_path = Path(path)
    content = file_path.read_text()
    if old not in content:
        raise SystemExit(f"expected ready-budget pattern missing in {path}: {old[:160]!r}")
    file_path.write_text(content.replace(old, new, 1))


path = "crates/golamd/src/process_dispatch_v2.rs"

replace_once(
    path,
    """const SUPERVISOR_POLL_MS: u64 = 10;
const TERMINAL_DRAIN_MS: u64 = 1000;
""",
    """const SUPERVISOR_POLL_MS: u64 = 10;
const TERMINAL_DRAIN_MS: u64 = 1000;
const HELPER_READY_TIMEOUT_MS: u64 = 30_000;
""",
)

replace_once(
    path,
    """        let started = Instant::now();
        if let Err(reason) = await_ready(
            &mut child,
            &receiver,
            execution_binding_digest,
            input.limits.wall_time_ms,
            started,
        ) {
""",
    """        let ready_started = Instant::now();
        if let Err(reason) = await_ready(
            &mut child,
            &receiver,
            execution_binding_digest,
            HELPER_READY_TIMEOUT_MS,
            ready_started,
        ) {
""",
)

replace_once(
    path,
    """            return Err(ProcessExecutionV2Error::HelperProtocol(reason));
        }

        let binding = RootContainmentBinding {
""",
    """            return Err(ProcessExecutionV2Error::HelperProtocol(reason));
        }

        // The payload wall-time budget begins only after the trusted helper has emitted the
        // containment-ready receipt. Helper verification and containment setup remain separately
        // bounded by HELPER_READY_TIMEOUT_MS and cannot consume the payload execution budget.
        let started = Instant::now();

        let binding = RootContainmentBinding {
""",
)

replace_once(
    path,
    """    fn await_ready(
        child: &mut Child,
        receiver: &Receiver<StreamEvent>,
        expected_binding: [u8; 32],
        wall_time_ms: u64,
        started: Instant,
    ) -> Result<(), &'static str> {
""",
    """    fn await_ready(
        child: &mut Child,
        receiver: &Receiver<StreamEvent>,
        expected_binding: [u8; 32],
        ready_timeout_ms: u64,
        ready_started: Instant,
    ) -> Result<(), &'static str> {
""",
)

replace_once(
    path,
    """            if started.elapsed().as_millis() >= u128::from(wall_time_ms) {
                return Err("process_execute_helper_ready_timeout");
            }
""",
    """            if ready_started.elapsed().as_millis() >= u128::from(ready_timeout_ms) {
                return Err("process_execute_helper_ready_timeout");
            }
""",
)
