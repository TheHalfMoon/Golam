#![forbid(unsafe_code)]

use rusqlite::{OptionalExtension, Transaction, TransactionBehavior, params};

use crate::authority_security_write::append_egress_permit_snapshot;
use crate::egress_destination::EffectiveDestination;
use crate::egress_permit::{
    EgressPermitError, EgressPermitStore, EgressPermitUseReceipt, UseDecisionEvidence,
    latest_global_seq, load_permit, verify_active_policy, verify_lease_chain_for_use,
};

#[allow(clippy::too_many_arguments)]
/// Authorize one connect/follow attempt against the exact effective endpoint observed by the caller.
///
/// DNS, redirect, rebinding, protocol/port and address-class changes are represented by a different
/// effective resource and/or decision context. A prior decision therefore cannot authorize a changed
/// endpoint. This boundary performs no DNS lookup or socket operation itself.
pub fn authorize_effective_use(
    store: &mut EgressPermitStore,
    permit_id: [u8; 16],
    decision_id: [u8; 16],
    principal_or_process: &str,
    action: &str,
    purpose: &str,
    effective: &EffectiveDestination,
    observed_at: &str,
) -> Result<EgressPermitUseReceipt, EgressPermitError> {
    if !valid_utc_second(observed_at) {
        return Err(EgressPermitError::InvalidTime);
    }

    let transaction = store
        .connection_mut()
        .transaction_with_behavior(TransactionBehavior::Immediate)?;
    crate::integrity::verify(&transaction)
        .map_err(|error| EgressPermitError::Integrity(error.to_string()))?;
    crate::authority_security_v2::verify(&transaction)
        .map_err(|error| EgressPermitError::AuthoritySecurity(error.to_string()))?;

    let permit = load_permit(&transaction, permit_id)?;
    if permit.status == "revoked" {
        return Err(EgressPermitError::PermitRevoked);
    }
    if permit.status == "exhausted" {
        return Err(EgressPermitError::PermitUsageExhausted);
    }
    if permit.status != "active" {
        return Err(EgressPermitError::PermitInactive);
    }
    if permit.principal_or_process != principal_or_process
        || permit.action != action
        || permit.purpose != purpose
    {
        return Err(EgressPermitError::PermitScopeMismatch);
    }
    if permit.protocol_port_scope != effective.protocol_port() {
        return Err(EgressPermitError::PermitScopeMismatch);
    }
    if observed_at < permit.issued_at.as_str() {
        return Err(EgressPermitError::PermitInactive);
    }
    if let Some(expires_at) = permit.expires_at.as_deref()
        && observed_at >= expires_at
    {
        return Err(EgressPermitError::PermitExpired);
    }
    if let Some(limit) = permit.usage_limit
        && permit.uses_consumed >= limit
    {
        return Err(EgressPermitError::PermitUsageExhausted);
    }

    let decision = load_current_effective_use_decision(
        &transaction,
        decision_id,
        principal_or_process,
        action,
        &permit,
        effective,
    )?;
    verify_active_policy(&transaction, &decision)?;
    verify_lease_chain_for_use(
        &transaction,
        permit.parent_lease_id,
        decision.lease_generation,
        principal_or_process,
        action,
        effective.resource(),
        observed_at,
    )?;

    let new_uses = permit
        .uses_consumed
        .checked_add(1)
        .ok_or(EgressPermitError::IntegerOverflow)?;
    let new_status = if permit.usage_limit == Some(new_uses) {
        "exhausted"
    } else {
        "active"
    };
    let changed = transaction.execute(
        "UPDATE egress_permits SET uses_consumed = ?1, status = ?2 WHERE permit_id = ?3 AND uses_consumed = ?4 AND status = 'active'",
        params![
            to_i64(new_uses)?,
            new_status,
            &permit_id[..],
            to_i64(permit.uses_consumed)?,
        ],
    )?;
    if changed != 1 {
        return Err(EgressPermitError::ConcurrentUseConflict);
    }
    append_egress_permit_snapshot(&transaction, &permit_id)
        .map_err(|error| EgressPermitError::AuthoritySecurity(error.to_string()))?;
    crate::authority_security_v2::verify(&transaction)
        .map_err(|error| EgressPermitError::AuthoritySecurity(error.to_string()))?;
    transaction.commit()?;

    Ok(EgressPermitUseReceipt {
        permit_id,
        decision_id,
        uses_consumed: new_uses,
        status: new_status.to_owned(),
    })
}

fn load_current_effective_use_decision(
    transaction: &Transaction<'_>,
    decision_id: [u8; 16],
    principal: &str,
    action: &str,
    permit: &crate::egress_permit::EgressPermitRecord,
    effective: &EffectiveDestination,
) -> Result<UseDecisionEvidence, EgressPermitError> {
    let row = transaction
        .query_row(
            "SELECT principal, action, resource, context_hash, hard_guard_result, lease_id, lease_generation, policy_bundle_id, policy_bundle_hash, decision, global_seq, authority_evidence_version FROM authorization_decisions WHERE decision_id = ?1",
            params![&decision_id[..]],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, Vec<u8>>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, Option<Vec<u8>>>(5)?,
                    row.get::<_, Option<i64>>(6)?,
                    row.get::<_, Option<Vec<u8>>>(7)?,
                    row.get::<_, Option<Vec<u8>>>(8)?,
                    row.get::<_, String>(9)?,
                    row.get::<_, i64>(10)?,
                    row.get::<_, i64>(11)?,
                ))
            },
        )
        .optional()?
        .ok_or(EgressPermitError::UseDecisionNotFound)?;

    let expected_context_hash =
        effective.decision_context_hash(permit.permit_id, &permit.destination_scope);
    if row.0 != principal
        || row.1 != action
        || row.2 != effective.resource()
        || row.3.as_slice() != &expected_context_hash[..]
        || row.4 != "pass"
        || row.9 != "allow"
        || row.11 < 2
    {
        return Err(EgressPermitError::UseDecisionMismatch);
    }

    let lease_id = id16(
        row.5.ok_or(EgressPermitError::UseDecisionMismatch)?,
        "effective decision lease id is invalid",
    )?;
    if lease_id != permit.parent_lease_id {
        return Err(EgressPermitError::UseDecisionMismatch);
    }
    let lease_generation = positive_u64(
        row.6.ok_or(EgressPermitError::UseDecisionMismatch)?,
        "effective decision lease generation is invalid",
    )?;
    let policy_bundle_id = id16(
        row.7.ok_or(EgressPermitError::UseDecisionMismatch)?,
        "effective decision policy bundle id is invalid",
    )?;
    let policy_bundle_hash = hash32(
        row.8.ok_or(EgressPermitError::UseDecisionMismatch)?,
        "effective decision policy bundle hash is invalid",
    )?;
    let global_seq = nonnegative_u64(row.10, "effective use decision sequence is invalid")?;
    if latest_global_seq(transaction)? != global_seq {
        return Err(EgressPermitError::UseDecisionStale);
    }

    Ok(UseDecisionEvidence {
        lease_generation,
        policy_bundle_id,
        policy_bundle_hash,
    })
}

fn valid_utc_second(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 20
        || bytes[4] != b'-'
        || bytes[7] != b'-'
        || bytes[10] != b'T'
        || bytes[13] != b':'
        || bytes[16] != b':'
        || bytes[19] != b'Z'
    {
        return false;
    }
    for index in [0, 1, 2, 3, 5, 6, 8, 9, 11, 12, 14, 15, 17, 18] {
        if !bytes[index].is_ascii_digit() {
            return false;
        }
    }
    let year = decimal(bytes, 0, 4);
    let month = decimal(bytes, 5, 7);
    let day = decimal(bytes, 8, 10);
    let hour = decimal(bytes, 11, 13);
    let minute = decimal(bytes, 14, 16);
    let second = decimal(bytes, 17, 19);
    if year == 0 || !(1..=12).contains(&month) || hour > 23 || minute > 59 || second > 59 {
        return false;
    }
    let max_day = match month {
        2 if is_leap_year(year) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    };
    (1..=max_day).contains(&day)
}

fn decimal(bytes: &[u8], start: usize, end: usize) -> u32 {
    bytes[start..end]
        .iter()
        .fold(0_u32, |value, byte| value * 10 + u32::from(*byte - b'0'))
}

fn is_leap_year(year: u32) -> bool {
    year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400))
}

fn id16(value: Vec<u8>, reason: &'static str) -> Result<[u8; 16], EgressPermitError> {
    value
        .try_into()
        .map_err(|_| EgressPermitError::InvalidStoredRecord(reason))
}

fn hash32(value: Vec<u8>, reason: &'static str) -> Result<[u8; 32], EgressPermitError> {
    value
        .try_into()
        .map_err(|_| EgressPermitError::InvalidStoredRecord(reason))
}

fn positive_u64(value: i64, reason: &'static str) -> Result<u64, EgressPermitError> {
    let value = nonnegative_u64(value, reason)?;
    if value == 0 {
        return Err(EgressPermitError::InvalidStoredRecord(reason));
    }
    Ok(value)
}

fn nonnegative_u64(value: i64, reason: &'static str) -> Result<u64, EgressPermitError> {
    u64::try_from(value).map_err(|_| EgressPermitError::InvalidStoredRecord(reason))
}

fn to_i64(value: u64) -> Result<i64, EgressPermitError> {
    i64::try_from(value).map_err(|_| EgressPermitError::IntegerOverflow)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::authority_security_write::append_authorization_decision_v2_snapshot;
    use crate::egress_destination::EffectiveNetworkClass;
    use crate::egress_permit::tests as permit_test;
    use crate::security_audit::{AuthorizationAuditInput, append_authorization_decision};

    fn install_effective_use_decision(
        store: &mut EgressPermitStore,
        global_seq: u64,
        discriminator: u8,
        permit: &crate::egress_permit::EgressPermitRecord,
        effective: &EffectiveDestination,
    ) -> [u8; 16] {
        let decision = [discriminator; 16];
        let context_hash =
            effective.decision_context_hash(permit.permit_id, &permit.destination_scope);
        let transaction = store
            .connection_mut()
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .unwrap();
        transaction
            .execute(
                "INSERT INTO authorization_decisions (decision_id, principal, action, resource, context_hash, decision, reason_code, global_seq, hard_guard_result, lease_id, lease_generation, policy_bundle_id, policy_bundle_hash, matched_rule_ids, approval_id, authority_evidence_version) VALUES (?1, ?2, ?3, ?4, ?5, 'allow', 'effective_egress_test_allow', ?6, 'pass', ?7, 1, ?8, ?9, X'', NULL, 2)",
                params![
                    &decision[..],
                    permit_test::PRINCIPAL,
                    permit_test::ACTION,
                    effective.resource(),
                    &context_hash[..],
                    i64::try_from(global_seq).unwrap(),
                    &permit_test::LEASE_ID[..],
                    &permit_test::POLICY_ID[..],
                    &permit_test::POLICY_HASH[..],
                ],
            )
            .unwrap();
        append_authorization_decision(
            &transaction,
            AuthorizationAuditInput {
                decision_id: &decision,
                principal: permit_test::PRINCIPAL,
                action: permit_test::ACTION,
                resource: effective.resource(),
                context_hash: &context_hash,
                decision: "allow",
                reason_code: "effective_egress_test_allow",
                global_seq,
            },
        )
        .unwrap();
        append_authorization_decision_v2_snapshot(&transaction, &decision).unwrap();
        crate::authority_security_v2::verify(&transaction).unwrap();
        transaction.commit().unwrap();
        decision
    }

    #[test]
    fn changed_effective_destination_requires_fresh_exact_authority_before_use() {
        let (runtime, authority) = permit_test::authority();
        let mut store = EgressPermitStore::open(&authority).unwrap();
        permit_test::install_policy_and_parent_lease(&mut store.connection);

        let issue = permit_test::prepared(Some(10));
        let work = permit_test::install_mutation_work(
            &mut store.connection,
            1,
            31,
            crate::egress_permit::EGRESS_PERMIT_ISSUE_ACTION,
            issue.resource(),
            issue.intent_digest(),
        );
        let permit = store
            .issue(issue, work.decision, work.approval, work.effect)
            .unwrap();

        let first = EffectiveDestination::new(
            "example.invalid",
            "203.0.113.10".parse().unwrap(),
            "https",
            443,
        )
        .unwrap();
        let first_decision = install_effective_use_decision(&mut store, 3, 101, &permit, &first);
        let first_receipt = authorize_effective_use(
            &mut store,
            permit.permit_id,
            first_decision,
            permit_test::PRINCIPAL,
            permit_test::ACTION,
            permit_test::PURPOSE,
            &first,
            "2026-08-28T02:00:00Z",
        )
        .unwrap();
        assert_eq!(first_receipt.uses_consumed, 1);

        let redirected = EffectiveDestination::new(
            "other.invalid",
            "203.0.113.10".parse().unwrap(),
            "https",
            443,
        )
        .unwrap();
        assert!(matches!(
            authorize_effective_use(
                &mut store,
                permit.permit_id,
                first_decision,
                permit_test::PRINCIPAL,
                permit_test::ACTION,
                permit_test::PURPOSE,
                &redirected,
                "2026-08-28T02:01:00Z",
            ),
            Err(EgressPermitError::UseDecisionMismatch)
        ));
        let redirect_decision =
            install_effective_use_decision(&mut store, 4, 102, &permit, &redirected);
        assert_eq!(
            authorize_effective_use(
                &mut store,
                permit.permit_id,
                redirect_decision,
                permit_test::PRINCIPAL,
                permit_test::ACTION,
                permit_test::PURPOSE,
                &redirected,
                "2026-08-28T02:02:00Z",
            )
            .unwrap()
            .uses_consumed,
            2
        );

        let rebound = EffectiveDestination::new(
            "other.invalid",
            "203.0.113.11".parse().unwrap(),
            "https",
            443,
        )
        .unwrap();
        assert!(matches!(
            authorize_effective_use(
                &mut store,
                permit.permit_id,
                redirect_decision,
                permit_test::PRINCIPAL,
                permit_test::ACTION,
                permit_test::PURPOSE,
                &rebound,
                "2026-08-28T02:03:00Z",
            ),
            Err(EgressPermitError::UseDecisionMismatch)
        ));
        let rebound_decision = install_effective_use_decision(&mut store, 5, 103, &permit, &rebound);
        assert_eq!(
            authorize_effective_use(
                &mut store,
                permit.permit_id,
                rebound_decision,
                permit_test::PRINCIPAL,
                permit_test::ACTION,
                permit_test::PURPOSE,
                &rebound,
                "2026-08-28T02:04:00Z",
            )
            .unwrap()
            .uses_consumed,
            3
        );

        let private = EffectiveDestination::new(
            "other.invalid",
            "10.0.0.7".parse().unwrap(),
            "https",
            443,
        )
        .unwrap();
        assert_eq!(private.class(), EffectiveNetworkClass::Private);
        assert!(matches!(
            authorize_effective_use(
                &mut store,
                permit.permit_id,
                rebound_decision,
                permit_test::PRINCIPAL,
                permit_test::ACTION,
                permit_test::PURPOSE,
                &private,
                "2026-08-28T02:05:00Z",
            ),
            Err(EgressPermitError::UseDecisionMismatch)
        ));
        let private_decision = install_effective_use_decision(&mut store, 6, 104, &permit, &private);
        assert_eq!(
            authorize_effective_use(
                &mut store,
                permit.permit_id,
                private_decision,
                permit_test::PRINCIPAL,
                permit_test::ACTION,
                permit_test::PURPOSE,
                &private,
                "2026-08-28T02:06:00Z",
            )
            .unwrap()
            .uses_consumed,
            4
        );

        let changed_protocol = EffectiveDestination::new(
            "other.invalid",
            "10.0.0.7".parse().unwrap(),
            "http",
            80,
        )
        .unwrap();
        assert!(matches!(
            authorize_effective_use(
                &mut store,
                permit.permit_id,
                private_decision,
                permit_test::PRINCIPAL,
                permit_test::ACTION,
                permit_test::PURPOSE,
                &changed_protocol,
                "2026-08-28T02:07:00Z",
            ),
            Err(EgressPermitError::PermitScopeMismatch)
        ));

        let final_uses: i64 = store
            .connection
            .query_row(
                "SELECT uses_consumed FROM egress_permits WHERE permit_id = ?1",
                params![&permit.permit_id[..]],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(final_uses, 4);
        crate::integrity::verify(&store.connection).unwrap();
        crate::authority_security_v2::verify(&store.connection).unwrap();
        drop(store);
        std::fs::remove_dir_all(runtime.root).unwrap();
    }
}
