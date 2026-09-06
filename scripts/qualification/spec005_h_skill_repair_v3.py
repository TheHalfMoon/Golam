from pathlib import Path

path = Path("crates/golamd/src/skill_packages.rs")
content = path.read_text()
old = "            validate_metadata_entry(key, value)?;\n"
new = "            validate_metadata_entry(key, &value)?;\n"
if old not in content:
    raise SystemExit("expected metadata validation call not found")
path.write_text(content.replace(old, new, 1))
