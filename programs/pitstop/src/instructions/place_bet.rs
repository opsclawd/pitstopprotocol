/// place_bet Rust parity model for LOCKED spec semantics.
///
/// Scope:
/// - deterministic precondition/error mapping (PBT-REJ-*)
/// - deterministic state/effect modeling for market/outcome_pool/position/vault
/// - BetPlaced event payload modeling
///
/// Note:
/// This is parity logic for spec/conformance verification; full Anchor account/CPI
/// wiring is implemented in a later on-chain integration pass.

use crate::{
    constants::REQUIRED_TOKEN_PROGRAM,
    error::PitStopError,
    events::BetPlaced,
    state::{Market, MarketStatus, OutcomePool, Position},
};

#[derive(Debug, Clone)]
pub struct PlaceBetInput {
    pub config_paused: bool,
    pub market_status: MarketStatus,
    pub now_ts: i64,
    pub market_lock_timestamp: i64,
    pub outcome_id: u8,
    pub market_outcome_count: u8,
    pub market_max_outcomes: u8,
    pub amount: u64,
    pub token_program: String,
    pub outcome_pool_exists: bool,
    pub outcome_pool_market: String,
    pub outcome_pool_outcome_id: u8,
    pub market: String,
    pub user: String,
    pub market_total_pool: u64,
    pub max_total_pool_per_market: u64,
    pub user_position_amount: u64,
    pub max_bet_per_user_per_market: u64,
    pub outcome_pool_amount: u64,
    pub vault_amount: u64,
    pub market_state: Market,
    pub outcome_pool_state: OutcomePool,
    pub position_state: Position,
}

fn validate_place_bet_preconditions(input: &PlaceBetInput) -> Result<(), PitStopError> {
    // Order mirrors locked precondition contract to keep deterministic error behavior.
    if input.config_paused {
        return Err(PitStopError::ProtocolPaused);
    }
    if input.market_status != MarketStatus::Open {
        return Err(PitStopError::MarketNotOpen);
    }
    if input.now_ts >= input.market_lock_timestamp {
        return Err(PitStopError::BettingClosed);
    }
    if input.outcome_id > 99 {
        return Err(PitStopError::InvalidOutcomeId);
    }
    if input.market_outcome_count != input.market_max_outcomes {
        return Err(PitStopError::MarketNotReady);
    }
    if input.amount == 0 {
        return Err(PitStopError::ZeroAmount);
    }
    if input.token_program != REQUIRED_TOKEN_PROGRAM {
        return Err(PitStopError::InvalidTokenProgram);
    }

    // Checked math first so u64 overflow is surfaced as protocol error (Overflow),
    // never as wraparound, panic, or misclassified cap rejection.
    let next_market_total = input
        .market_total_pool
        .checked_add(input.amount)
        .ok_or(PitStopError::Overflow)?;
    if next_market_total > input.max_total_pool_per_market {
        return Err(PitStopError::MarketCapExceeded);
    }

    let next_user_pos = input
        .user_position_amount
        .checked_add(input.amount)
        .ok_or(PitStopError::Overflow)?;
    if next_user_pos > input.max_bet_per_user_per_market {
        return Err(PitStopError::UserBetCapExceeded);
    }

    // Outcome existence + relation checks (spec requires deterministic OutcomeMismatch
    // for wrong relation and modeled missing/uninitialized cases in this parity layer).
    if !input.outcome_pool_exists
        || input.outcome_pool_market != input.market
        || input.outcome_pool_outcome_id != input.outcome_id
    {
        return Err(PitStopError::OutcomeMismatch);
    }

    Ok(())
}

/// Executes place_bet effects after preconditions pass.
///
/// Effects modeled:
/// - market.total_pool += amount
/// - outcome_pool.pool_amount += amount
/// - position.amount += amount
/// - vault_amount += amount
///
/// Post-effect event:
/// - BetPlaced { market, user, outcome_id, amount, market_total_pool, outcome_pool_amount, timestamp }
pub fn place_bet(
    input: PlaceBetInput,
) -> Result<(Market, OutcomePool, Position, u64, BetPlaced), PitStopError> {
    validate_place_bet_preconditions(&input)?;

    let market_total_pool = input.market_total_pool.checked_add(input.amount).ok_or(PitStopError::Overflow)?;
    let outcome_pool_amount = input.outcome_pool_amount.checked_add(input.amount).ok_or(PitStopError::Overflow)?;
    let position_amount = input.user_position_amount.checked_add(input.amount).ok_or(PitStopError::Overflow)?;
    let vault_amount = input.vault_amount.checked_add(input.amount).ok_or(PitStopError::Overflow)?;

    let mut market = input.market_state;
    market.total_pool = market_total_pool;

    let mut outcome_pool = input.outcome_pool_state;
    outcome_pool.pool_amount = outcome_pool_amount;

    let mut position = input.position_state;
    position.amount = position_amount;

    // Event emitted only after successful state/effect updates (EVT-MTX alignment).
    let evt = BetPlaced {
        market: input.market,
        user: input.user,
        outcome_id: input.outcome_id,
        amount: input.amount,
        market_total_pool,
        outcome_pool_amount,
        timestamp: input.now_ts,
    };

    Ok((market, outcome_pool, position, vault_amount, evt))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_market() -> Market {
        Market {
            market_id: [1u8; 32],
            event_id: [2u8; 32],
            lock_timestamp: 1_800_000_100,
            outcome_count: 3,
            max_outcomes: 3,
            total_pool: 1000,
            status: MarketStatus::Open,
            resolved_outcome: None,
            resolution_payload_hash: [0u8; 32],
            resolution_timestamp: 0,
            vault: "VaultA".to_string(),
            market_type: 0,
            rules_version: 1,
        }
    }

    fn base_input() -> PlaceBetInput {
        PlaceBetInput {
            config_paused: false,
            market_status: MarketStatus::Open,
            now_ts: 1_800_000_000,
            market_lock_timestamp: 1_800_000_100,
            outcome_id: 1,
            market_outcome_count: 3,
            market_max_outcomes: 3,
            amount: 100,
            token_program: REQUIRED_TOKEN_PROGRAM.to_string(),
            outcome_pool_exists: true,
            outcome_pool_market: "MarketA".to_string(),
            outcome_pool_outcome_id: 1,
            market: "MarketA".to_string(),
            user: "UserA".to_string(),
            market_total_pool: 1000,
            max_total_pool_per_market: 10_000,
            user_position_amount: 200,
            max_bet_per_user_per_market: 1000,
            outcome_pool_amount: 400,
            vault_amount: 1000,
            market_state: base_market(),
            outcome_pool_state: OutcomePool {
                market: "MarketA".to_string(),
                outcome_id: 1,
                pool_amount: 400,
            },
            position_state: Position {
                market: "MarketA".to_string(),
                user: "UserA".to_string(),
                outcome_id: 1,
                amount: 200,
            },
        }
    }

    #[test]
    fn pbt_hp_updates_balances_and_event() {
        // PBT-HP-001/002 baseline: successful transfer/effect/event modeling.
        let (m, o, p, vault, e) = place_bet(base_input()).expect("place_bet should pass");
        assert_eq!(m.total_pool, 1100);
        assert_eq!(o.pool_amount, 500);
        assert_eq!(p.amount, 300);
        assert_eq!(vault, 1100);
        assert_eq!(e.market_total_pool, 1100);
        assert_eq!(e.outcome_pool_amount, 500);
    }

    #[test]
    fn pbt_rej_matrix() {
        // PBT-REJ-001..010 deterministic error mapping coverage.
        let mut bad = base_input();
        bad.config_paused = true;
        assert_eq!(place_bet(bad).unwrap_err(), PitStopError::ProtocolPaused);

        let mut bad = base_input();
        bad.market_status = MarketStatus::Locked;
        assert_eq!(place_bet(bad).unwrap_err(), PitStopError::MarketNotOpen);

        let mut bad = base_input();
        bad.now_ts = bad.market_lock_timestamp;
        assert_eq!(place_bet(bad).unwrap_err(), PitStopError::BettingClosed);

        let mut bad = base_input();
        bad.outcome_id = 100;
        assert_eq!(place_bet(bad).unwrap_err(), PitStopError::InvalidOutcomeId);

        let mut bad = base_input();
        bad.market_outcome_count = 2;
        assert_eq!(place_bet(bad).unwrap_err(), PitStopError::MarketNotReady);

        let mut bad = base_input();
        bad.amount = 0;
        assert_eq!(place_bet(bad).unwrap_err(), PitStopError::ZeroAmount);

        let mut bad = base_input();
        bad.market_total_pool = 9_950;
        assert_eq!(place_bet(bad).unwrap_err(), PitStopError::MarketCapExceeded);

        let mut bad = base_input();
        bad.user_position_amount = 950;
        assert_eq!(place_bet(bad).unwrap_err(), PitStopError::UserBetCapExceeded);

        let mut bad = base_input();
        bad.outcome_pool_exists = false;
        assert_eq!(place_bet(bad).unwrap_err(), PitStopError::OutcomeMismatch);

        let mut bad = base_input();
        bad.token_program = "TokenzFake".to_string();
        assert_eq!(place_bet(bad).unwrap_err(), PitStopError::InvalidTokenProgram);
    }

    #[test]
    fn pbt_rej_wrong_outcome_relation_cases() {
        let mut bad = base_input();
        bad.outcome_pool_market = "OtherMarket".to_string();
        assert_eq!(place_bet(bad).unwrap_err(), PitStopError::OutcomeMismatch);

        let mut bad = base_input();
        bad.outcome_pool_outcome_id = 2;
        assert_eq!(place_bet(bad).unwrap_err(), PitStopError::OutcomeMismatch);
    }

    #[test]
    fn pbt_rej_overflow_maps_to_overflow() {
        let mut bad = base_input();
        bad.market_total_pool = u64::MAX;
        bad.amount = 1;
        bad.max_total_pool_per_market = u64::MAX;
        assert_eq!(place_bet(bad).unwrap_err(), PitStopError::Overflow);
    }

}
