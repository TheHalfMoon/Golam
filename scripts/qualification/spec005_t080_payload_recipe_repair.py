from pathlib import Path

path = Path(".github/workflows/ci.yml")
content = path.read_text()
old = """          rustc +1.98.0 -C target-feature=+crt-static -C opt-level=1 scripts/qualification/process-v2-payload.rs -o target/debug/process-v2-payload
          if readelf -l target/debug/process-v2-payload | grep -q 'INTERP'; then
            echo 'qualification payload is dynamically linked' >&2
            exit 1
          fi
"""
new = """          rustc +1.98.0 -C target-feature=+crt-static -C relocation-model=static -C link-arg=-no-pie -C opt-level=1 scripts/qualification/process-v2-payload.rs -o target/debug/process-v2-payload
          if ! readelf -h target/debug/process-v2-payload | grep -Eq 'Type:[[:space:]]+EXEC'; then
            echo 'qualification payload is not an ET_EXEC ELF' >&2
            exit 1
          fi
          if readelf -l target/debug/process-v2-payload | grep -q 'INTERP'; then
            echo 'qualification payload is dynamically linked' >&2
            exit 1
          fi
"""
if content.count(old) != 1:
    raise SystemExit("expected one governed-process payload recipe")
path.write_text(content.replace(old, new, 1))
