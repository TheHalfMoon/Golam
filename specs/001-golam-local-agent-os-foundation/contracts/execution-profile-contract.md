# Contract: ExecutionProfile

Golam routes execution profiles, not model names alone. The abstraction is frozen; individual backend adapters remain replaceable.

An `ExecutionProfile` binds:
- model identity and exact revision;
- tokenizer identity and chat-template identity;
- inference backend;
- locality class: local | explicit_cloud;
- quantization/precision;
- hardware/device mapping;
- harness profile;
- reasoning mode;
- tool-call conformance: native_tools | grammar_constrained | text_protocol_fallback;
- tool schema mode;
- sampling parameters;
- context compiler policy;
- prompt/prefix cache policy;
- KV-cache policy;
- warm-residency policy: load/keep/evict behavior;
- workload class: interactive | batch | background;
- multimodal capability flags;
- resource/time/token budgets;
- latency/quality budget;
- privacy and network constraints;
- explicit load/failure/fallback policy;
- benchmark record references.

## Router rules

- strict-local mode may only select local profiles;
- cloud profile selection must be explicit under user policy;
- profile switches are canonical events;
- user may pin/override a profile within policy;
- benchmark results attach to exact profile definitions and hardware records;
- unsupported hardware/load failure fails clearly rather than silently changing privacy/locality mode;
- fallback may change model/backend only within the profile's allowed privacy/network class.

## Backend boundary

Golam owns harness and authority semantics. `mistral.rs` is a primary Rust-native inference candidate behind an adapter. `llama.cpp` is a compatibility backend and SHOULD default to an out-of-process sidecar rather than in-process unsafe C FFI inside `golamd`.

## Hardware calibration

Calibration may measure CPU/RAM/GPU/VRAM/accelerators, backend support, model load success/time, prompt/decode throughput, TTFT, memory pressure, cache behavior, tool-call conformance, repair rate, energy where available, and task success. Recommendations are evidence-based, inspectable, and reversible.
