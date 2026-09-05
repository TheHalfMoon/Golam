from pathlib import Path

path = Path("crates/golamd/tests/process_v2_qualification.rs")
content = path.read_text()
old = '                    issued_at: "2026-09-05T19:15:29Z",\n'
new = '                    issued_at: "2026-09-05T19:15:30Z",\n'
if content.count(old) != 1:
    raise SystemExit("expected one approval effect issued_at binding")
path.write_text(content.replace(old, new, 1))
