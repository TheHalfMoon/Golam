from pathlib import Path

path = Path("crates/golamd/src/skill_process_v2.rs")
text = path.read_text()
old = "use crate::local_fs::LocalFsResolver;\n"
if text.count(old) != 1:
    raise SystemExit("expected exactly one stale LocalFsResolver import")
path.write_text(text.replace(old, "", 1))
