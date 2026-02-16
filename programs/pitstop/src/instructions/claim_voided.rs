/// claim_voided Rust parity model for LOCKED spec semantics.
///
/// Spec reference:
/// - SPEC_INSTRUCTIONS/claim_voided.md (LOCKED v1.0.3)
/// - SPEC_EVENTS.md (LOCKED) -> Claimed
/// - SPEC_ERRORS.md -> claim_voided mapping
///
/// Notes:
/// - Deterministic parity model: focuses on precondition ordering +
///   state/effect/event modeling rather than Anchor account wiring.

use crate::{
    error::PitStopError,
    events::Claimed,
    state::{MarketStatus, Position},
};

#[derive(Debug, Clone)]
pub struct ClaimVoidedInput {
    pub market: String,
    pub user: String,

    pub market_status: MarketStatus,
    pub resolution_timestamp: i64,
    pub claim_window_secs: i64,
    pub now_ts: i64,

    /// Instruction input; also part of the position PDA seeds in the on-chain program.
    pub outcome_id: u8,

    pub user_usdc_amount: u64,
    pub vault_amount: u64,

    pub position_state: Position,
}

fn validate_claim_voided_preconditions(input: &ClaimVoidedInput) -> Result<(), PitStopError> {
    // Failure ordering (locked): status gate is evaluated before any vault usage.
    // CLV-REJ-001
    if input.market_status != MarketStatus::Voided {
        return Err(PitStopError::MarketNotVoided);
    }

    // CLV-REJ-002
    if input.position_state.claimed {
        return Err(PitStopError::AlreadyClaimed);
    }

    // CLV-REJ-003: claim window is inclusive at end.
    let window_end = input
        .resolution_timestamp
        .checked_add(input.claim_window_secs)
        .ok_or(PitStopError::Overflow)?;
    if input.now_ts > window_end {
        return Err(PitStopError::ClaimWindowExpired);
    }

    Ok(())
}

/// Executes claim_voided effects.
///
/// Effects:
/// - payout := position.amount
/// - transfer payout from vault -> user_usdc
/// - position.claimed = true; position.payout = payout
/// - emit Claimed { market, user, outcome_id, payout, claimed_at }
pub fn claim_voided(
    input: ClaimVoidedInput,
) -> Result<(Position, u64, u64, Claimed), PitStopError> {
    validate_claim_voided_preconditions(&input)?;

    let payout = input.position_state.amount;

    // First vault usage ops (checked math).
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

    Ok((position, user_usdc_amount, vault_amount, evt))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_position() -> Position {
        Position {
            market: "MarketA".to_string(),
            user: "UserA".to_string(),
            outcome_id: 7,
            amount: 250,
            claimed: false,
            payout: 0,
        }
    }

    fn base_input() -> ClaimVoidedInput {
        ClaimVoidedInput {
            market: "MarketA".to_string(),
            user: "UserA".to_string(),
            market_status: MarketStatus::Voided,
            resolution_timestamp: 1_800_000_000,
            claim_window_secs: 3600,
            now_ts: 1_800_000_100,
            outcome_id: 7,
            user_usdc_amount: 1_000,
            vault_amount: 10_000,
            position_state: base_position(),
        }
    }

    #[test]
    fn clv_hp_001_refunds_principal_marks_claimed_sets_payout_and_emits_event() {
        let (pos, user_usdc, vault, evt) =
            claim_voided(base_input()).expect("claim_voided should pass");

        assert!(pos.claimed);
        assert_eq!(pos.payout, 250);
        assert_eq!(pos.amount, 250);

        assert_eq!(user_usdc, 1_250);
        assert_eq!(vault, 9_750);

        assert_eq!(evt.market, "MarketA");
        assert_eq!(evt.user, "UserA");
        assert_eq!(evt.outcome_id, 7);
        assert_eq!(evt.payout, 250);
        assert_eq!(evt.claimed_at, 1_800_000_100);
    }

    #[test]
    fn clv_rej_001_to_003_error_mapping() {
        // CLV-REJ-001: market must be voided.
        let mut bad = base_input();
        bad.market_status = MarketStatus::Resolved;
        assert_eq!(claim_voided(bad).unwrap_err(), PitStopError::MarketNotVoided);

        // CLV-REJ-002: already claimed.
        let mut bad = base_input();
        bad.position_state.claimed = true;
        assert_eq!(claim_voided(bad).unwrap_err(), PitStopError::AlreadyClaimed);

        // CLV-REJ-003: expired window (strictly greater).
        let mut bad = base_input();
        bad.now_ts = bad.resolution_timestamp + bad.claim_window_secs + 1;
        assert_eq!(claim_voided(bad).unwrap_err(), PitStopError::ClaimWindowExpired);
    }

    #[test]
    fn clv_inv_001_does_not_mutate_position_principal() {
        let input = base_input();
        let original_amount = input.position_state.amount;

        let (pos, _user_usdc, _vault, _evt) =
            claim_voided(input).expect("claim_voided should pass");
        assert_eq!(pos.amount, original_amount);
    }

    #[test]
    fn clv_ord_001_swept_fails_by_status_error_before_vault_underflow() {
        // Locked ordering requirement: in Swept, must fail with MarketNotVoided
        // deterministically (not with vault/account errors).
        let mut bad = base_input();
        bad.market_status = MarketStatus::Swept;
        bad.vault_amount = 0; // would underflow if evaluated

        assert_eq!(claim_voided(bad).unwrap_err(), PitStopError::MarketNotVoided);
    }
}
