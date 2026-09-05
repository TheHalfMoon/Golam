from pathlib import Path

path = Path("crates/golamd/tests/process_v2_qualification.rs")
content = path.read_text()
replacements = {
    '&[b"success"]': '&[b"process-v2-payload", b"success"]',
    '&[b"isolation"]': '&[b"process-v2-payload", b"isolation"]',
    '&[b"spin"], 100': '&[b"process-v2-payload", b"spin"], 100',
    '&[b"output"]': '&[b"process-v2-payload", b"output"]',
    '&[b"spin"], 2_000': '&[b"process-v2-payload", b"spin"], 2_000',
}
for old, new in replacements.items():
    if old not in content:
        raise SystemExit(f"expected argv fixture pattern missing: {old!r}")
    content = content.replace(old, new, 1)
path.write_text(content)
