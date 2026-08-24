# Contract: Skills and Extensions

## Compatibility

Golam supports Agent Skills-style `SKILL.md` packages and may expose/import compatible skill ecosystems. Golam may also load governed MCP tools and native/WASM extensions.

## Authority rule

Skill declarations, prompts, scripts, `allowed-tools`, or MCP metadata express requested behavior/capabilities only. They cannot grant authority. Kernel policy is authoritative.

## Admission lifecycle

`discover -> pin source -> license/provenance -> normalize -> inspect scripts/dependencies -> infer capabilities -> security scan -> sandbox test -> functional test -> benchmark -> sign/hash -> lock -> install`

## Lock record

The lock records:
- skill ID/version;
- source URL/revision;
- content hash;
- license/notices;
- requested capabilities;
- dependency hashes;
- scan/test results;
- install time;
- signer/provenance.

## Built-in parity skills

Golam will provide independently implemented built-ins equivalent in user-facing purpose to publicly documented Grok Bot categories:
- Documents;
- Presentations;
- Spreadsheets;
- PDFs;
- Skill Creator.

These are Golam implementations and must not copy proprietary prompts/templates/assets unless separately licensed.
