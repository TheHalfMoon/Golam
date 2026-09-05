from pathlib import Path


def replace_once(path: str, old: str, new: str) -> None:
    file_path = Path(path)
    content = file_path.read_text()
    if old not in content:
        raise SystemExit(f"expected fd-hygiene pattern missing in {path}: {old[:160]!r}")
    file_path.write_text(content.replace(old, new, 1))


replace_once(
    "crates/golamd/src/bin/golam-native-exec-helper-v2.rs",
    "use std::fs::File;\n",
    "use std::fs::{self, File};\n",
)
replace_once(
    "crates/golamd/src/bin/golam-native-exec-helper-v2.rs",
    "use nix::unistd::{fexecve, geteuid, getppid};\n",
    "use nix::errno::Errno;\n    use nix::unistd::{close, fexecve, geteuid, getppid};\n",
)
replace_once(
    "crates/golamd/src/bin/golam-native-exec-helper-v2.rs",
    """        if std::env::vars_os().next().is_some() {
            return Err(
                \"trusted native exec helper requires an empty ambient environment\".to_owned(),
            );
        }
        let config = parse_args()?;
""",
    """        if std::env::vars_os().next().is_some() {
            return Err(
                \"trusted native exec helper requires an empty ambient environment\".to_owned(),
            );
        }
        close_inherited_descriptors()?;
        let config = parse_args()?;
""",
)
replace_once(
    "crates/golamd/src/bin/golam-native-exec-helper-v2.rs",
    """    fn bind_parent_death(expected_parent_pid: u32) -> Result<(), String> {
""",
    """    fn close_inherited_descriptors() -> Result<(), String> {
        let entries = fs::read_dir(\"/proc/self/fd\")
            .map_err(|error| format!(\"inspect inherited descriptors before containment: {error}\"))?;
        let mut descriptors = entries
            .map(|entry| {
                entry
                    .map_err(|error| format!(\"read inherited descriptor entry: {error}\"))
                    .and_then(|entry| {
                        entry
                            .file_name()
                            .to_string_lossy()
                            .parse::<i32>()
                            .map_err(|_| \"non-numeric /proc/self/fd entry\".to_owned())
                    })
            })
            .collect::<Result<Vec<_>, _>>()?;
        descriptors.sort_unstable();
        descriptors.dedup();
        for fd in descriptors.into_iter().filter(|fd| *fd > 2) {
            match close(fd) {
                Ok(()) | Err(Errno::EBADF) => {}
                Err(error) => {
                    return Err(format!(\"close inherited descriptor {fd}: {error}\"));
                }
            }
        }
        Ok(())
    }

    fn bind_parent_death(expected_parent_pid: u32) -> Result<(), String> {
""",
)

replace_once(
    "crates/golamd/tests/process_v2_qualification.rs",
    """        let success = fixture.execute(100, (0x7000, 0x7100), &[b\"success\"], 2_000, 4096, false);
        assert_eq!(success.status, ProcessExecutionStatusV2::Succeeded);
""",
    """        let secret_canary = std::env::var(\"GOLAM_PROCESS_SECRET_CANARY\")
            .expect(\"qualification secret canary\");
        let assert_secret_absent = |receipt: &golamd::process_dispatch_v2::ProcessExecutionReceiptV2| {
            let secret = secret_canary.as_bytes();
            assert!(!receipt.stdout.windows(secret.len()).any(|window| window == secret));
            assert!(!receipt.stderr.windows(secret.len()).any(|window| window == secret));
        };

        let success = fixture.execute(100, (0x7000, 0x7100), &[b\"success\"], 2_000, 4096, false);
        assert_secret_absent(&success);
        assert_eq!(success.status, ProcessExecutionStatusV2::Succeeded);
""",
)
replace_once(
    "crates/golamd/tests/process_v2_qualification.rs",
    """        let isolation = fixture.execute(101, (0x7200, 0x7300), &[b\"isolation\"], 2_000, 4096, false);
        assert_eq!(isolation.status, ProcessExecutionStatusV2::Succeeded);
""",
    """        let isolation = fixture.execute(101, (0x7200, 0x7300), &[b\"isolation\"], 2_000, 4096, false);
        assert_secret_absent(&isolation);
        assert_eq!(isolation.status, ProcessExecutionStatusV2::Succeeded);
""",
)
replace_once(
    "crates/golamd/tests/process_v2_qualification.rs",
    """        let timeout = fixture.execute(102, (0x7400, 0x7500), &[b\"spin\"], 100, 4096, false);
        assert_eq!(timeout.status, ProcessExecutionStatusV2::TimedOut);
""",
    """        let timeout = fixture.execute(102, (0x7400, 0x7500), &[b\"spin\"], 100, 4096, false);
        assert_secret_absent(&timeout);
        assert_eq!(timeout.status, ProcessExecutionStatusV2::TimedOut);
""",
)
replace_once(
    "crates/golamd/tests/process_v2_qualification.rs",
    """        let output = fixture.execute(103, (0x7600, 0x7700), &[b\"output\"], 2_000, 128, false);
        assert_eq!(output.status, ProcessExecutionStatusV2::OutputLimitExceeded);
""",
    """        let output = fixture.execute(103, (0x7600, 0x7700), &[b\"output\"], 2_000, 128, false);
        assert_secret_absent(&output);
        assert_eq!(output.status, ProcessExecutionStatusV2::OutputLimitExceeded);
""",
)
replace_once(
    "crates/golamd/tests/process_v2_qualification.rs",
    """        let cancelled = fixture.execute(104, (0x7800, 0x7900), &[b\"spin\"], 2_000, 4096, true);
        assert_eq!(cancelled.status, ProcessExecutionStatusV2::Cancelled);
""",
    """        let cancelled = fixture.execute(104, (0x7800, 0x7900), &[b\"spin\"], 2_000, 4096, true);
        assert_secret_absent(&cancelled);
        assert_eq!(cancelled.status, ProcessExecutionStatusV2::Cancelled);
""",
)
