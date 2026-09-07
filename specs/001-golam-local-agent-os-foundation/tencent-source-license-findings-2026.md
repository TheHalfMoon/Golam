# Tencent Source License Findings — 2026-09

**Status**: `SOURCE_RIGHTS_GUARDRAIL`

This artifact supplements `tencent-source-adoption-2026.md`. It grants no code, dependency, model, dataset or binary admission.

## Reviewed exact pins

```text
Tencent/WeKnora@647848f3954dae34473b8a8d0e0eef5e0fb3a58e
Tencent/RoMem@39ac1417b4db41ea729e5c3be71ac20de54da993
Tencent/SkillHone@7d565839fb4dc74f9c77f09ace660e1c0484e048
Tencent/LoopForge@09c765286f549624dd95434e1e6ef2249657cbeb
Tencent/AI-Infra-Guard@e4e622af3ad2b8228ce82dd62b01415dd8ce2b9c
```

## Dispositions

### WeKnora

- Root project license: MIT, with listed third-party components under their original licenses.
- Any selective code reuse requires file-level provenance and transitive/embedded third-party review; the root MIT statement is not sufficient evidence for blindly copying arbitrary bundled/vendor/generated content.

```text
WEKNORA_CODE_COPY=BLOCKED_UNTIL_FILE_LEVEL_SOURCE_FOUNDRY
```

### RoMem

- GitHub repository metadata reports `license=null`.
- Root `LICENSE` is absent at the reviewed revision.
- Bundled baselines contain their own notices/licenses, but those do not establish a license for RoMem's original project code, checkpoints or datasets.

```text
ROMEM_ORIGINAL_CODE_COPY=DENIED_PENDING_CLEAR_LICENSE
ROMEM_CHECKPOINT_REUSE=DENIED_PENDING_RIGHTS_REVIEW
ROMEM_DATASET_REUSE=DENIED_PENDING_PER_DATASET_RIGHTS_REVIEW
ROMEM_RESEARCH_IDEAS=REFERENCE_ONLY
```

### SkillHone

- Root project license: MIT.
- Runtime behavior may invoke bypass/high-permission execution modes; license compatibility does not imply security or authority compatibility.

```text
SKILLHONE_SELECTIVE_CODE_REUSE=SOURCE_FOUNDRY_CANDIDATE
SKILLHONE_BYPASS_EXECUTION_POLICY=NOT_ADOPTED
```

### LoopForge

- Root project license: MIT with third-party notices.
- The reviewed license records modified `obra/superpowers` material under MIT.
- Any copied file requires provenance/notice reconciliation rather than assuming every file is original Tencent material.

```text
LOOPFORGE_SELECTIVE_CODE_REUSE=SOURCE_FOUNDRY_CANDIDATE_WITH_PROVENANCE_CHECK
```

### AI-Infra-Guard

- Root repository license: Apache-2.0.
- `skill-scan/README.md` states Apache-2.0 and its `skill-scan/NOTICE` requires downstream attribution including the statement `Based on Tencent Zhuque Lab AI-Infra-Guard` plus a link to the original repository when incorporating/deriving from that project.
- `mcp-scan/README.md` states `MIT License`, but no `mcp-scan/LICENSE` exists at the reviewed revision while the repository root remains Apache-2.0. This is a material subtree-license ambiguity for code-copy purposes.

Therefore:

```text
AI_INFRA_GUARD_SECURITY_IDEAS=HIGH_VALUE_REFERENCE
SKILL_SCAN_CODE_REUSE=SOURCE_FOUNDRY_CANDIDATE_WITH_EXACT_NOTICE_ATTRIBUTION
MCP_SCAN_CODE_REUSE=BLOCKED_PENDING_FILE_LEVEL_LICENSE_RECONCILIATION
MCP_SCAN_BEHAVIOR_AND_RISK_TAXONOMY=REFERENCE_ONLY_UNTIL_THEN
FULL_PLATFORM_REUSE=NOT_PROPOSED
```

## Global rule

```text
PUBLIC_REPOSITORY != LICENSE_PERMISSION
ROOT_LICENSE != AUTOMATIC_VENDOR_SUBTREE_PERMISSION
LICENSE_PERMISSION != SECURITY_ADMISSION
REFERENCE_IDEA != CODE_COPY
CODE_COPY != DEPENDENCY_ADMISSION
DEPENDENCY_ADMISSION != RUNTIME_AUTHORITY
```

Any later owning spec must re-fetch the exact donor revision and repeat the license/provenance check before copying, vendoring, packaging or redistributing donor code/assets.