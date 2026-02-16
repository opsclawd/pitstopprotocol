use crate::{
    error::PitStopError,
    events::MarketResolved,
    state::{Market, MarketStatus, OutcomePool},
};

#[derive(Debug, Clone)]
pub struct ResolveMarketInput {
    pub oracle: String,
    pub config_oracle: String,
    pub market: String,
    pub market_state: Market,
    pub winning_outcome_id: u8,
    pub payload_hash: [u8; 32],
    pub winning_outcome_pool_state: Option<OutcomePool>,
    pub now_ts: i64,
}

fn validate_resolve_market_preconditions(input: &ResolveMarketInput) -> Result<(), PitStopError> {
    // RSM-REJ-001: only config oracle can resolve markets.
    if input.oracle != input.config_oracle {
        return Err(PitStopError::UnauthorizedOracle);
    }

    // RSM-REJ-002: market must be locked before resolution.
    if input.market_state.status != MarketStatus::Locked {
        return Err(PitStopError::MarketNotLocked);
    }

    // RSM-REJ-003: winning_outcome_id must be in [0, 99].
    if input.winning_outcome_id > 99 {
        return Err(PitStopError::InvalidOutcomeId);
    }

    // RSM-REJ-004: winning outcome must exist in seeded outcome range.
    if input.winning_outcome_id >= input.market_state.outcome_count {
        return Err(PitStopError::InvalidOutcomeId);
    }

    // RSM-REJ-004 / RSM-ADV-001: missing or mismatched outcome pool => OutcomeMismatch.
    let winning_pool = input
        .winning_outcome_pool_state
        .as_ref()
        .ok_or(PitStopError::OutcomeMismatch)?;

    if winning_pool.market != input.market || winning_pool.outcome_id != input.winning_outcome_id {
        return Err(PitStopError::OutcomeMismatch);
    }

    Ok(())
}

pub fn resolve_market(input: ResolveMarketInput) -> Result<(Market, MarketResolved), PitStopError> {
    validate_resolve_market_preconditions(&input)?;

    let mut market = input.market_state;
    market.status = MarketStatus::Resolved;
    market.resolved_outcome = Some(input.winning_outcome_id);
    market.resolution_payload_hash = input.payload_hash;
    market.resolution_timestamp = input.now_ts;

    let evt = MarketResolved {
        market: input.market,
        winning_outcome: input.winning_outcome_id,
        payload_hash: input.payload_hash,
        resolution_timestamp: input.now_ts,
    };

    Ok((market, evt))
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
            total_pool: 1000,
            status: MarketStatus::Locked,
            resolved_outcome: None,
            resolution_payload_hash: [0u8; 32],
            resolution_timestamp: 0,
            vault: "VaultA".to_string(),
            market_type: 0,
            rules_version: 1,
        }
    }

    fn base_pool() -> OutcomePool {
        OutcomePool {
            market: "MarketA".to_string(),
            outcome_id: 1,
            pool_amount: 500,
        }
    }

    fn base_input() -> ResolveMarketInput {
        ResolveMarketInput {
            oracle: "OracleA".to_string(),
            config_oracle: "OracleA".to_string(),
            market: "MarketA".to_string(),
            market_state: base_market(),
            winning_outcome_id: 1,
            payload_hash: [0xabu8; 32],
            winning_outcome_pool_state: Some(base_pool()),
            now_ts: 1_800_000_500,
        }
    }

    #[test]
    fn rsm_hp_001_transitions_market_and_emits_event() {
        let (m, e) = resolve_market(base_input()).expect("resolve_market should pass");
        assert_eq!(m.status, MarketStatus::Resolved);
        assert_eq!(m.resolved_outcome, Some(1));
        assert_eq!(m.resolution_payload_hash, [0xabu8; 32]);
        assert_eq!(m.resolution_timestamp, 1_800_000_500);

        assert_eq!(e.market, "MarketA");
        assert_eq!(e.winning_outcome, 1);
        assert_eq!(e.payload_hash, [0xabu8; 32]);
        assert_eq!(e.resolution_timestamp, 1_800_000_500);
    }

    #[test]
    fn rsm_rej_001_to_004_error_mapping() {
        let mut bad = base_input();
        bad.oracle = "Other".to_string();
        assert_eq!(
            resolve_market(bad).unwrap_err(),
            PitStopError::UnauthorizedOracle
        );

        let mut bad = base_input();
        bad.market_state.status = MarketStatus::Open;
        assert_eq!(
            resolve_market(bad).unwrap_err(),
            PitStopError::MarketNotLocked
        );

        let mut bad = base_input();
        bad.winning_outcome_id = 100;
        assert_eq!(
            resolve_market(bad).unwrap_err(),
            PitStopError::InvalidOutcomeId
        );

        let mut bad = base_input();
        bad.winning_outcome_id = 3;
        assert_eq!(
            resolve_market(bad).unwrap_err(),
            PitStopError::InvalidOutcomeId
        );

        let mut bad = base_input();
        bad.winning_outcome_pool_state = Some(OutcomePool {
            outcome_id: 2,
            ..base_pool()
        });
        assert_eq!(
            resolve_market(bad).unwrap_err(),
            PitStopError::OutcomeMismatch
        );

        let mut bad = base_input();
        bad.winning_outcome_pool_state = Some(OutcomePool {
            market: "OtherMarket".to_string(),
            ..base_pool()
        });
        assert_eq!(
            resolve_market(bad).unwrap_err(),
            PitStopError::OutcomeMismatch
        );
    }

    #[test]
    fn rsm_adv_001_missing_outcome_pool_maps_to_outcome_mismatch() {
        let mut bad = base_input();
        bad.winning_outcome_pool_state = None;
        assert_eq!(
            resolve_market(bad).unwrap_err(),
            PitStopError::OutcomeMismatch
        );
    }
}
