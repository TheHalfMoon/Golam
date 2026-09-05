import sys
from pathlib import Path

path = Path("crates/golamd/tests/process_v2_qualification.rs")
content = path.read_text()
needle = """            execute_staged_process_v2(
                &mut self.kernel,
"""
diag = """            eprintln!(\"SPEC005_T080_CASE_REQUEST_ID={request_id}\");
            execute_staged_process_v2(
                &mut self.kernel,
"""
if len(sys.argv) > 1 and sys.argv[1] == "--revert":
    if diag not in content:
        raise SystemExit("diagnostic marker not present for revert")
    path.write_text(content.replace(diag, needle, 1))
else:
    if needle not in content:
        raise SystemExit("execute_staged_process_v2 call not found for diagnostic marker")
    path.write_text(content.replace(needle, diag, 1))
