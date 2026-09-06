# Focus and Work-Surface Contract

Focus is mutable external state, not authority.

A focus intent binds an exact work-surface identity and authorized effect. Before focus/action dispatch, the adapter must revalidate that the surface incarnation still matches the prepared identity. After a focus request, the runtime must observe whether the intended surface actually became focused before allowing any operation whose target depends on focus.

Required denial/race cases:
- target window closed/recreated;
- process/application restarted;
- focus stolen between prepare and execute;
- active session changed;
- permission revoked;
- surface identifier reused with different incarnation;
- user switches workspace/desktop such that the target is no longer eligible.

A coordinate is never a stable work-surface identity.
