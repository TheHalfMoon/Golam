# T003-061 Egress Permit Qualification

**Status**: PASS  
**Qualified implementation head**: `94d1482f8963ea4d5630a1ba4d2bdaba0e12e7ef`  
**Qualified tree**: `68dc5b73bee5d8beafd3e5a34d316ac17c371f53`  
**Official qualification**: CI #514 / run `33198112325` — SUCCESS on Windows, macOS, and Ubuntu.

## Qualified boundary

T003-061 implements the bounded non-strict `EgressPermit` authority lifecycle while preserving the T003-060 strict-local hard guard above every downstream permit decision.

The qualified implementation proves:

- permit preparation binds principal/process, network action, purpose, destination, protocol/port, taint digest, optional secret handle, parent lease binding, lifetime, and optional usage limit;
- protected permit issuance requires an exact current allow decision, an authorized `AT_MOST_ONCE` elevated effect, a matching `ONCE` approval, and an active in-scope parent capability lease;
- permit issuance and approval consumption commit atomically with `authority-security-v2` coverage;
- protected revocation requires the same typed current-authority/effect/approval discipline and atomically changes the protected permit status;
- use-time authorization requires the exact active permit scope, current allow decision, active policy bundle/hash, exact parent lease generation, active/revocation-safe lease chain, and valid permit/lease time bounds;
- bounded use accounting increments atomically and transitions the permit to `exhausted` exactly at its usage limit;
- revoked, exhausted, expired, out-of-scope, stale-decision, stale-policy, or stale-lease uses fail closed;
- authority schema v4 adds only `uses_consumed INTEGER NOT NULL DEFAULT 0` to the existing protected `egress_permits` table through a forward migration;
- every permit state change is included in the existing `authority-security-v2` source coverage and write path;
- no actual network socket, DNS resolver, redirect follower, or hidden external integration is introduced by this task.

## Focused implementation evidence

The focused T003-061 recovery qualification executed the three egress lifecycle tests, the authority-schema migration test, formatting, and crate-wide clippy with `-D warnings`. Three security-boundary functions retain explicit authority dimensions and therefore use narrowly scoped `clippy::too_many_arguments` allowances with repository comments; no global lint weakening was introduced.

Temporary implementation workflows were staging/recovery mechanisms only and self-deleted before the clean qualification tree. The task PASS authority is solely official `ci.yml` run `33198112325` on the exact clean user-authored head above.

## Deferred ordered boundaries

- Effective destination transformation and mandatory fresh reauthorization for DNS resolution, redirects, rebinding, protocol/port changes, and private/link-local/loopback transitions remain T003-062.
- Taint/provenance and optional secret-handle equality binding into egress-use evidence remain T003-063.
- Descendant-aware strict-local external sinkhole/no-egress qualification remains T003-064.

```text
T003_061=PASS
T003_061_QUALIFIED_HEAD=94d1482f8963ea4d5630a1ba4d2bdaba0e12e7ef
T003_061_CI_RUN=33198112325
PHASE_G_ACTIVE=YES
NEXT_TASK=T003-062
REAL_SECRETS_USED=NO
```
