from pathlib import Path

path = Path("crates/golamd/tests/process_v2_qualification.rs")
content = path.read_text()
old = '''            let runtime = RuntimeLayout::initialize(root.join("runtime")).expect("runtime");
            let source_root = root.join("source");
            let staging_root = root.join("stage");
'''
new = '''            let runtime = RuntimeLayout::initialize(root.join("runtime")).expect("runtime");
            let source_root = root.join("source");
            let staging_root = runtime.runtime_dir.join("process-stage-v2");
'''
if content.count(old) != 1:
    raise SystemExit("expected one process qualification staging layout")
path.write_text(content.replace(old, new, 1))
