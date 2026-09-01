# Source Foundry Note — Munder Difflin

**Disposition**: `REFERENCE_ONLY`  
**Upstream**: `https://github.com/chaitanyagiri/munder-difflin`  
**Observed branch**: `main`  
**License**: MIT for source code; bundled art has separate attribution/license requirements.

## Why it is relevant

Munder Difflin is a local multi-agent harness that wraps terminal-agent CLI processes and coordinates them through a local mechanism layer. Several of its architectural patterns are useful references for Golam, but its product scope and authority model are broader than Spec 004.

## Patterns worth preserving as references

1. **Mechanism vs intelligence separation**
   - The harness/main process owns routing, durable files, process lifecycle and transport.
   - Agent/model processes provide intelligence but need not own durable mechanism.
   - This is consistent with Golam's invariant that model backends are not authority roots.

2. **Single-writer / single-committer coordination**
   - Agents write isolated per-agent files.
   - A single trusted process performs cross-agent delivery and repository commits.
   - This is a useful later-spec reference for avoiding concurrent durable-state corruption.

3. **Atomic mailbox records and append-only audit**
   - One message per file with atomic rename.
   - Processed messages are retained rather than deleted.
   - An append-only event feed is used for observable coordination.
   - These patterns align with Golam's durable-evidence and immutable-attempt posture.

4. **Anti-livelock controls**
   - Bounded hop counts.
   - Idempotent message processing through cursors.
   - Only selected message classes obligate a reply.
   - Circuit-breaker concepts use steer/constrain/stop rather than unbounded autonomous loops.

5. **Process boundary as containment**
   - Real terminal-agent CLIs execute in separate OS processes rather than being merged into the UI/controller authority path.
   - This is a useful reference when evaluating sidecar isolation for future model or agent runtimes.

## Explicit non-adoptions for Spec 004

Do not import or implement the following from this reference in Spec 004:

- a privileged "GOD" model/agent as an authority root;
- prompt-defined approval or escalation policy as protected authorization;
- broad filesystem, shell, git, browser, MCP or agent-tool authority;
- long-term memory, semantic hive memory or shared agent-memory product scope;
- multi-agent worker scheduling, mailboxes, blackboards or task-ledger product scope;
- Electron/PTY/tmux/UI architecture;
- automatic installation of external CLIs or update behavior.

These are either outside Spec 004 or conflict with Golam's protected authority boundaries.

## Source-reuse decision

No source code is copied, ported, vendored or depended on by Spec 004. This record preserves architecture lessons only. Any future code reuse must reopen Source Foundry qualification against an exact upstream revision and must separately satisfy license, transitive dependency, unsafe/native, filesystem/network, process, secret and authority-boundary review.

```text
MUNDER_DIFFLIN_DISPOSITION=REFERENCE_ONLY
SOURCE_CODE_REUSED=NO
DEPENDENCY_ADDED=NO
SPEC_004_SCOPE_EXPANDED=NO
MODEL_OR_AGENT_AUTHORITY_ADOPTED=NO
```
