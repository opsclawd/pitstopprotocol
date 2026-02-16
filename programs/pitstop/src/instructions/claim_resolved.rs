/// claim_resolved Rust parity model for LOCKED spec semantics.
///
/// Spec: SPEC_INSTRUCTIONS/claim_resolved.md (LOCKED v1.0.3)
///
/// Deterministic model used by Rust unit tests and spec-gate parity checks.

use crate::{
    error::PitStopError,
    events::Claimed,
    state::{Market, MarketStatus, OutcomePool, Position},
};

#[derive(Debug, Clone)]
pub struct ClaimResolvedInput {
    pub market: String,
    pub user: String,

    // Market/config context
    pub market_status: MarketStatus,
    pub now_ts: i64,
    pub resolution_timestamp: i64,
    pub claim_window_secs: i64,
    pub fee_bps: u16,
    pub resolved_outcome: Option<u8>,

    // Position context
    pub outcome_id: u8,
    pub position_claimed: bool,
    pub position_amount: u64,

    // Outcome pool relation (modeled)
    pub outcome_pool_exists: bool,
    pub outcome_pool_market: String,
    pub outcome_pool_outcome_id: u8,
    pub outcome_pool_amount: u64,

    // Token/account balances (modeled)
    pub vault_amount: u64,
    pub user_usdc_amount: u64,

    // Full account state snapshots
    pub market_state: Market,
    pub outcome_pool_state: OutcomePool,
    pub position_state: Position,
}

fn validate_claim_resolved_preconditions(input: &ClaimResolvedInput) -> Result<(), PitStopError> {
    // CLR-REJ-001 / CLR-ORD-001: status gate first (Swept fails here too).
    if input.market_status != MarketStatus::Resolved {
        return Err(PitStopError::MarketNotResolved);
    }

    // CLR-REJ-002: no double claim.
    if input.position_claimed {
        return Err(PitStopError::AlreadyClaimed);
    }

    // CLR-REJ-003: outcome pool must exist and match (market, outcome_id).
    if !input.outcome_pool_exists
        || input.outcome_pool_market != input.market
        || input.outcome_pool_outcome_id != input.outcome_id
    {
        return Err(PitStopError::OutcomeMismatch);
    }

    // CLR-REJ-004: now must be within claim window (inclusive end).
    let claim_window_end = input
        .resolution_timestamp
        .checked_add(input.claim_window_secs)
        .ok_or(PitStopError::Overflow)?;
    if input.now_ts > claim_window_end {
        return Err(PitStopError::ClaimWindowExpired);
    }

    Ok(())
}

fn compute_winner_payout(
    total_pool: u64,
    winner_pool: u64,
    position_amount: u64,
    fee_bps: u16,
) -> Result<u64, PitStopError> {
    // fee = floor(total_pool * fee_bps / 10_000)
    let fee = total_pool
        .checked_mul(fee_bps as u64)
        .ok_or(PitStopError::Overflow)?
        / 10_000u64;

    let prize_pool = total_pool.checked_sub(fee).ok_or(PitStopError::Underflow)?;

    if winner_pool == 0 {
        return Err(PitStopError::DivisionByZero);
    }

    // payout = floor(position_amount * prize_pool / winner_pool)
    let numerator = position_amount
        .checked_mul(prize_pool)
        .ok_or(PitStopError::Overflow)?;

    Ok(numerator / winner_pool)
}

/// Effects:
/// - payout computed using locked floor math
/// - if winner: vault -= payout, user += payout
/// - if loser: payout=0, no transfer
/// - position.claimed=true, position.payout=payout
/// - emit Claimed
pub fn claim_resolved(
    input: ClaimResolvedInput,
) -> Result<(Position, u64, u64, Claimed), PitStopError> {
    validate_claim_resolved_preconditions(&input)?;

    let is_winner = input.resolved_outcome == Some(input.outcome_id);

    let payout = if is_winner {
        compute_winner_payout(
            input.market_state.total_pool,
            input.outcome_pool_amount,
            input.position_amount,
            input.fee_bps,
        )?
    } else {
        0
    };

    let vault_amount = input
        .vault_amount
        .checked_sub(payout)
        .ok_or(PitStopError::Underflow)?;
    let user_usdc_amount = input
        .user_usdc_amount
        .checked_add(payout)
        .ok_or(PitStopError::Overflow)?;

    let mut position = input.position_state;
    position.claimed = true;
    position.payout = payout;

    let evt = Claimed {
        market: input.market,
        user: input.user,
        outcome_id: input.outcome_id,
        payout,
        claimed_at: input.now_ts,
    };

    Ok((position, vault_amount, user_usdc_amount, evt))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_market() -> Market {
        Market {
            market_id: [1u8; 32],
            event_id: [2u8; 32],
            lock_timestamp: 1_800_000_000,
            outcome_count: 3,
            max_outcomes: 3,
            total_pool: 1_000,
            status: MarketStatus::Resolved,
            resolved_outcome: Some(1),
            resolution_payload_hash: [9u8; 32],
            resolution_timestamp: 1_800_000_100,
            vault: "VaultA".to_string(),
            market_type: 0,
            rules_version: 1,
        }
    }

    fn base_input() -> ClaimResolvedInput {
        ClaimResolvedInput {
            market: "MarketA".to_string(),
            user: "UserA".to_string(),
            market_status: MarketStatus::Resolved,
            now_ts: 1_800_000_101,
            resolution_timestamp: 1_800_000_100,
            claim_window_secs: 600,
            fee_bps: 200, // 2%
            resolved_outcome: Some(1),
            outcome_id: 1,
            position_claimed: false,
            position_amount: 100,
            outcome_pool_exists: true,
            outcome_pool_market: "MarketA".to_string(),
            outcome_pool_outcome_id: 1,
            outcome_pool_amount: 250,
            vault_amount: 1_000,
            user_usdc_amount: 10,
            market_state: base_market(),
            outcome_pool_state: OutcomePool {
                market: "MarketA".to_string(),
                outcome_id: 1,
                pool_amount: 250,
            },
            position_state: Position {
                market: "MarketA".to_string(),
                user: "UserA".to_string(),
                outcome_id: 1,
                amount: 100,
                claimed: false,
                payout: 0,
            },
        }
    }

    #[test]
    fn clr_hp_001_winner_claim_transfers_payout_records_position_and_emits_event() {
        // total_pool=1000, fee= floor(1000*200/10000)=20, prize_pool=980
        // payout = floor(100*980/250) = 392
        let (p, vault, user_bal, e) = claim_resolved(base_input()).expect("claim should pass");
        assert_eq!(p.claimed, true);
        assert_eq!(p.payout, 392);
        assert_eq!(vault, 608);
        assert_eq!(user_bal, 402);
        assert_eq!(e.payout, 392);
    }

    #[test]
    fn clr_hp_002_loser_claim_sets_claimed_with_zero_payout_and_no_transfer() {
        let mut input = base_input();
        input.outcome_id = 0;
        input.position_amount = 777;
        input.position_state.outcome_id = 0;
        input.position_state.amount = 777;
        input.outcome_pool_outcome_id = 0;
        input.outcome_pool_amount = 123;

        let (p, vault, user_bal, e) = claim_resolved(input).expect("loser claim should pass");
        assert_eq!(p.claimed, true);
        assert_eq!(p.payout, 0);
        assert_eq!(vault, 1_000);
        assert_eq!(user_bal, 10);
        assert_eq!(e.payout, 0);
    }

    #[test]
    fn clr_hp_003_claim_window_end_is_inclusive() {
        let mut input = base_input();
        input.claim_window_secs = 10;
        input.now_ts = input.resolution_timestamp + 10;
        assert!(claim_resolved(input).is_ok());
    }

    #[test]
    fn clr_rej_001_to_004_error_matrix() {
        let mut bad = base_input();
        bad.market_status = MarketStatus::Locked;
        assert_eq!(claim_resolved(bad).unwrap_err(), PitStopError::MarketNotResolved);

        let mut bad = base_input();
        bad.position_claimed = true;
        assert_eq!(claim_resolved(bad).unwrap_err(), PitStopError::AlreadyClaimed);

        let mut bad = base_input();
        bad.outcome_pool_exists = false;
        assert_eq!(claim_resolved(bad).unwrap_err(), PitStopError::OutcomeMismatch);

        let mut bad = base_input();
        bad.now_ts = bad.resolution_timestamp + bad.claim_window_secs + 1;
        assert_eq!(claim_resolved(bad).unwrap_err(), PitStopError::ClaimWindowExpired);
    }

    #[test]
    fn clr_ord_001_swept_fails_by_status_before_outcome_mismatch() {
        let mut bad = base_input();
        bad.market_status = MarketStatus::Swept;
        bad.outcome_pool_exists = false;
        assert_eq!(claim_resolved(bad).unwrap_err(), PitStopError::MarketNotResolved);
    }

    #[test]
    fn clr_inv_001_winner_pool_zero_maps_to_division_by_zero() {
        let mut bad = base_input();
        bad.outcome_pool_amount = 0;
        assert_eq!(claim_resolved(bad).unwrap_err(), PitStopError::DivisionByZero);
    }

    #[test]
    fn clr_inv_002_underflow_and_overflow_cases_map_to_protocol_errors() {
        let mut bad = base_input();
        bad.fee_bps = 65_535;
        assert_eq!(claim_resolved(bad).unwrap_err(), PitStopError::Underflow);

        let mut bad = base_input();
        bad.market_state.total_pool = u64::MAX;
        bad.fee_bps = 0;
        bad.position_amount = u64::MAX;
        bad.position_state.amount = u64::MAX;
        bad.outcome_pool_amount = 1;
        assert_eq!(claim_resolved(bad).unwrap_err(), PitStopError::Overflow);
    }
}
