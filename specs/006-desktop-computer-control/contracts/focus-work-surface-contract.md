# Focus and Work-Surface Contract

Focus is mutable external state, not authority.

A focus intent binds an immutable ToolRequest/request digest, Effect/effect binding digest, canonical intent digest, exact work-surface identity, capability/policy/approval refs as applicable, and prepared session/observation evidence. Before focus/action dispatch, the runtime revalidates every binding and verifies that the surface incarnation still matches the prepared identity. Missing, mismatched, stale or substituted state fails closed before platform actuation.

After a focus request, the runtime must observe whether the intended surface actually became focused before allowing any operation whose target depends on focus. If the focus effect may have occurred but terminal truth is uncertain, record `UNKNOWN_OUTCOME` and block conflicting focus-dependent action until reconciliation.

Required denial/race cases:
- target window closed/recreated;
- process/application restarted;
- focus stolen between prepare and execute;
- active session changed;
- permission revoked;
- surface identifier reused with different incarnation;
- user switches workspace/desktop such that the target is no longer eligible.

A coordinate is never a stable work-surface identity.
