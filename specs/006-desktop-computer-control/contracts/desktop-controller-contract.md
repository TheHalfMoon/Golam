# Desktop Controller Contract

## Purpose

Expose one versioned local desktop-control façade without erasing platform permission and capability differences.

## Required interface

Conceptual operations:
- `capabilities()`
- `observe(request)`
- `focus_work_surface(intent)`
- `perform_semantic_action(intent)`
- `perform_raw_fallback(intent)`
- `capture(intent)`
- `clipboard_read(intent)`
- `clipboard_write(intent)`
- `release_handles(scope)`

## Contract rules

1. Observation is read-only and cannot mint actuation authority.
2. Every side-effect operation requires an already-authorized immutable intent/effect binding.
3. Adapter capability discovery is descriptive only.
4. Native handles remain adapter-private.
5. Platform adapters return deterministic typed unsupported/permission/stale/unknown outcomes.
6. Semantic failure never triggers raw fallback automatically.
7. Capture, raw input and clipboard are separate capability/effect classes.
8. No remote fallback is part of this interface.
