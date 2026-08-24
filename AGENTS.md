# Golam Agent Instructions

## Current phase

Golam is in specification and architecture planning. Do not implement product code until the active Spec Kit package has passed its external architecture review gate and a later `tasks.md` explicitly authorizes implementation.

## Read order

1. `.specify/memory/constitution.md`
2. `specs/001-golam-local-agent-os-foundation/spec.md`
3. `specs/001-golam-local-agent-os-foundation/clarification-closeout.md`
4. `specs/001-golam-local-agent-os-foundation/research.md`
5. `specs/001-golam-local-agent-os-foundation/plan.md`
6. `specs/001-golam-local-agent-os-foundation/data-model.md`
7. `specs/001-golam-local-agent-os-foundation/contracts/`
8. `specs/001-golam-local-agent-os-foundation/quickstart.md`
9. `specs/001-golam-local-agent-os-foundation/checklists/implementation-readiness.md`
10. `specs/001-golam-local-agent-os-foundation/review/finalization-status.md`

## Non-negotiable boundaries

- Rust is mandatory for the trusted runtime, daemon, CLI/TUI, policy, durability, secrets, device transport, remote-control broker, model routing, memory orchestration, audit, and protocol authority.
- Golam MUST remain fully useful in strict-local mode without a cloud model or Golam-hosted cloud service.
- TypeScript is permitted only in the untrusted Tauri renderer and other explicitly untrusted UI surfaces.
- Python/Node integrations may exist only as optional sandboxed adapters; they cannot become required for strict-local operation.
- `Golam-Research` and reconstructed Grok Bot material are behavioral/reference evidence only. Do not copy reconstructed proprietary material into Golam.
- No donor source enters Golam until its exact repository, commit/tree, license, notices, dependency closure, and reuse strategy are recorded and approved.
- GPL/AGPL/SSPL and similarly reciprocal donors are reference-only unless the founder explicitly approves their license obligations for a bounded component.
- A model, skill, MCP server, channel, worker, or remote device can request authority but cannot grant or expand its own authority.
- All consequential external effects must pass the Effect Gate and produce durable evidence.
- Never claim a benchmark, CI, security, parity, or platform gate passed without evidence from the exact tested head.

## Development discipline

When implementation is later authorized, work in small Spec Kit task slices, preserve exact-head verification, use cargo fmt/clippy/test as default Rust gates, add security tests alongside security-sensitive code, and prefer reversible changes and independent PRs over broad rewrites.
