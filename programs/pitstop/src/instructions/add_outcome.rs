use crate::{
    error::PitStopError,
    events::OutcomeAdded,
    state::{Market, MarketStatus, OutcomePool},
};

#[derive(Debug, Clone)]
pub struct AddOutcomeInput {
    pub authority: String,
    pub config_authority: String,
    pub market: String,
    pub market_status: MarketStatus,
    pub market_outcome_count: u8,
    pub market_max_outcomes: u8,
    pub outcome_id: u8,
    pub outcome_pool_market: String,
    pub market_state: Market,
    pub now_ts: i64,
}

fn validate_add_outcome_preconditions(input: &AddOutcomeInput) -> Result<(), PitStopError> {
    // ADO-REJ-001: only config authority can add outcomes.
    if input.authority != input.config_authority {
        return Err(PitStopError::Unauthorized);
    }
    // ADO-REJ-002: market must still be in Seeding lifecycle phase.
    if input.market_status != MarketStatus::Seeding {
        return Err(PitStopError::MarketNotSeeding);
    }
    // ADO-REJ-003: outcome_id must be within [0,99].
    if input.outcome_id > 99 {
        return Err(PitStopError::InvalidOutcomeId);
    }
    // ADO-REJ-004: market cannot exceed configured max_outcomes.
    if input.market_outcome_count >= input.market_max_outcomes {
        return Err(PitStopError::MaxOutcomesReached);
    }
    // ADO-REJ-005: outcome pool relation must bind to the same market account.
    if input.outcome_pool_market != input.market {
        return Err(PitStopError::OutcomeMismatch);
    }
    // Hardening: mirrored scalar fields must match canonical market_state values.
    if input.market_state.status != input.market_status {
        return Err(PitStopError::OutcomeMismatch);
    }
    if input.market_state.max_outcomes != input.market_max_outcomes {
        return Err(PitStopError::OutcomeMismatch);
    }
    // Keep single source-of-truth for market count updates.
    if input.market_state.outcome_count != input.market_outcome_count {
        return Err(PitStopError::OutcomeMismatch);
    }

    Ok(())
}

pub fn add_outcome(input: AddOutcomeInput) -> Result<(Market, OutcomePool, OutcomeAdded), PitStopError> {
    validate_add_outcome_preconditions(&input)?;

    // Effects contract: outcome pool starts at zero and market outcome_count increments once.
    let mut market = input.market_state;
    market.outcome_count += 1;

    let outcome_pool = OutcomePool {
        market: input.market.clone(),
        outcome_id: input.outcome_id,
        pool_amount: 0,
    };

    // Event contract: emit OutcomeAdded only on successful state update.
    let evt = OutcomeAdded {
        market: input.market,
        outcome_id: input.outcome_id,
        outcome_count: market.outcome_count,
        timestamp: input.now_ts,
    };

    Ok((market, outcome_pool, evt))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_market() -> Market {
        Market {
            market_id: [1u8; 32],
            event_id: [2u8; 32],
            lock_timestamp: 1_900_000_000,
            outcome_count: 1,
            max_outcomes: 3,
            total_pool: 0,
            status: MarketStatus::Seeding,
            resolved_outcome: None,
            resolution_payload_hash: [0u8; 32],
            resolution_timestamp: 0,
            vault: "VaultAtaA".to_string(),
            market_type: 0,
            rules_version: 1,
        }
    }

    fn base_input() -> AddOutcomeInput {
        AddOutcomeInput {
            authority: "AuthA".to_string(),
            config_authority: "AuthA".to_string(),
            market: "MarketPdaA".to_string(),
            market_status: MarketStatus::Seeding,
            market_outcome_count: 1,
            market_max_outcomes: 3,
            outcome_id: 2,
            outcome_pool_market: "MarketPdaA".to_string(),
            market_state: base_market(),
            now_ts: 1_800_000_000,
        }
    }

    #[test]
    fn ado_hp_001_adds_outcome_pool_and_increments_count() {
        let (m, p, e) = add_outcome(base_input()).expect("add_outcome should pass");
        assert_eq!(p.market, "MarketPdaA");
        assert_eq!(p.outcome_id, 2);
        assert_eq!(p.pool_amount, 0);
        assert_eq!(m.outcome_count, 2);
        assert_eq!(e.market, "MarketPdaA");
        assert_eq!(e.outcome_id, 2);
        assert_eq!(e.outcome_count, 2);
        assert_eq!(e.timestamp, 1_800_000_000);
    }

    #[test]
    fn ado_rej_001_to_005_error_mapping() {
        let mut bad = base_input();
        bad.authority = "Other".to_string();
        assert_eq!(add_outcome(bad).unwrap_err(), PitStopError::Unauthorized);

        let mut bad = base_input();
        bad.market_status = MarketStatus::Open;
        assert_eq!(add_outcome(bad).unwrap_err(), PitStopError::MarketNotSeeding);

        let mut bad = base_input();
        bad.outcome_id = 100;
        assert_eq!(add_outcome(bad).unwrap_err(), PitStopError::InvalidOutcomeId);

        let mut bad = base_input();
        bad.market_outcome_count = 3;
        assert_eq!(add_outcome(bad).unwrap_err(), PitStopError::MaxOutcomesReached);

        let mut bad = base_input();
        bad.outcome_pool_market = "OtherMarket".to_string();
        assert_eq!(add_outcome(bad).unwrap_err(), PitStopError::OutcomeMismatch);

        let mut bad = base_input();
        bad.market_state.outcome_count = 2;
        assert_eq!(add_outcome(bad).unwrap_err(), PitStopError::OutcomeMismatch);
    }

    #[test]
    fn ado_rej_hardening_mirrored_market_fields_must_match() {
        let mut bad = base_input();
        bad.market_status = MarketStatus::Seeding;
        bad.market_state.status = MarketStatus::Open;
        assert_eq!(add_outcome(bad).unwrap_err(), PitStopError::OutcomeMismatch);

        let mut bad = base_input();
        bad.market_max_outcomes = 3;
        bad.market_state.max_outcomes = 4;
        assert_eq!(add_outcome(bad).unwrap_err(), PitStopError::OutcomeMismatch);
    }

}
