use crate::{
    error::PitStopError,
    events::MarketVoided,
    state::{Market, MarketStatus},
};

#[derive(Debug, Clone)]
pub struct VoidMarketInput {
    pub oracle: String,
    pub config_oracle: String,

    pub market: String,
    pub payload_hash: [u8; 32],
    pub now_ts: i64,

    /// Canonical market state (single source-of-truth).
    pub market_state: Market,
}

fn validate_void_market_preconditions(input: &VoidMarketInput) -> Result<(), PitStopError> {
    // VDM-REJ-001: oracle signer must match config.oracle.
    if input.oracle != input.config_oracle {
        return Err(PitStopError::UnauthorizedOracle);
    }

    // VDM-REJ-002/003: only Locked markets can transition to Voided.
    // Use canonical market state as source-of-truth for lifecycle checks.
    if input.market_state.status != MarketStatus::Locked {
        return Err(PitStopError::MarketNotLocked);
    }

    Ok(())
}

pub fn void_market(input: VoidMarketInput) -> Result<(Market, MarketVoided), PitStopError> {
    validate_void_market_preconditions(&input)?;

    let mut market = input.market_state;
    market.status = MarketStatus::Voided;
    market.resolved_outcome = None;
    market.resolution_payload_hash = input.payload_hash;
    market.resolution_timestamp = input.now_ts;

    let evt = MarketVoided {
        market: input.market,
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
            resolved_outcome: Some(1),
            resolution_payload_hash: [9u8; 32],
            resolution_timestamp: 1_799_999_000,
            vault: "VaultA".to_string(),
            market_type: 0,
            rules_version: 1,
        }
    }

    fn base_input() -> VoidMarketInput {
        VoidMarketInput {
            oracle: "OracleA".to_string(),
            config_oracle: "OracleA".to_string(),
            market: "MarketA".to_string(),
            payload_hash: [7u8; 32],
            now_ts: 1_800_000_100,
            market_state: base_market(),
        }
    }

    #[test]
    fn vdm_hp_001_transitions_to_voided_and_emits_event() {
        let input = base_input();
        let (m, e) = void_market(input).expect("void_market should pass");

        assert_eq!(m.status, MarketStatus::Voided);
        assert_eq!(m.resolved_outcome, None);
        assert_eq!(m.resolution_payload_hash, [7u8; 32]);
        assert_eq!(m.resolution_timestamp, 1_800_000_100);

        assert_eq!(e.market, "MarketA");
        assert_eq!(e.payload_hash, [7u8; 32]);
        assert_eq!(e.resolution_timestamp, 1_800_000_100);
    }

    #[test]
    fn vdm_rej_001_unauthorized_oracle() {
        let mut bad = base_input();
        bad.oracle = "Other".to_string();
        assert_eq!(void_market(bad).unwrap_err(), PitStopError::UnauthorizedOracle);
    }

    #[test]
    fn vdm_rej_002_003_market_not_locked() {
        let mut bad = base_input();
        bad.market_state.status = MarketStatus::Open;
        assert_eq!(void_market(bad).unwrap_err(), PitStopError::MarketNotLocked);

        let mut bad = base_input();
        bad.market_state.status = MarketStatus::Resolved;
        assert_eq!(void_market(bad).unwrap_err(), PitStopError::MarketNotLocked);
    }
}
