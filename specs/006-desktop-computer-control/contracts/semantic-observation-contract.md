# Semantic Observation Contract

## Inputs

A bounded observation request must specify work-surface scope plus limits for windows/surfaces, semantic nodes, depth, strings/bytes and wall time.

## Output

A `DesktopObservation` containing opaque identities, sanitized semantic summaries, focus state where observable, applied bounds and a canonical binding digest.

## Invariants

- Titles, labels and semantic text are untrusted strings.
- Identity cannot be based solely on title, label, coordinate or screenshot.
- Observation must not expose raw platform handles to the model/frontend.
- Truncation is explicit and attested; exceeding hard resource limits fails closed.
- Permission/session loss during observation produces a typed partial/denied result, not fabricated completeness.
- Any later action must revalidate the selected identity against fresh platform state.
