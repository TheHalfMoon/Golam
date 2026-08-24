# Contract: ExecutionProfile

Golam routes execution profiles, not model names alone.

An `ExecutionProfile` binds:
- model identity/revision;
- inference backend;
- local/cloud locality class;
- quantization/precision;
- hardware/device mapping;
- harness profile;
- reasoning mode;
- tool-call/schema mode;
- sampling parameters;
- context compiler policy;
- prompt/prefix cache policy;
- resource/time/token budgets;
- privacy and network constraints.

## Router rules

- strict-local mode may only select local profiles;
- cloud profile selection must be explicit under user policy;
- profile switches are recorded in canonical events;
- user may pin/override a profile;
- benchmark results attach to exact profile definitions;
- unsupported hardware fails clearly rather than silently changing privacy mode.

## Hardware calibration

Calibration may measure available CPU/RAM/GPU/VRAM/accelerators, supported backends, model load success, prompt/decode throughput and memory pressure. Recommendations are evidence-based and reversible.
