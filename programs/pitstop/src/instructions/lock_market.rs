use crate::{
    error::PitStopError,
    events::MarketLocked,
    state::{Market, MarketStatus},
};

#[derive(Debug, Clone)]
pub struct LockMarketInput {
    pub authority: String,
    pub config_authority: String,
    pub market: String,
    pub market_status: MarketStatus,
    pub now_ts: i64,
    pub lock_timestamp: i64,
    pub market_state: Market,
}

fn validate_lock_market_preconditions(input: &LockMarketInput) -> Result<(), PitStopError> {
    // LKM-REJ-001: authority must match config authority.
    if input.authority != input.config_authority {
        return Err(PitStopError::Unauthorized);
    }
    // LKM-REJ-002: only Open markets can transition to Locked.
    if input.market_status != MarketStatus::Open {
        return Err(PitStopError::MarketNotOpen);
    }
    // LKM-REJ-003: lock is allowed at/after lock_timestamp; reject before it.
    if input.now_ts < input.lock_timestamp {
        return Err(PitStopError::TooEarlyToLock);
    }
    Ok(())
}

pub fn lock_market(input: LockMarketInput) -> Result<(Market, MarketLocked), PitStopError> {
    validate_lock_market_preconditions(&input)?;

    // Effect: transition status to Locked.
    let mut market = input.market_state;
    market.status = MarketStatus::Locked;

    // Event emitted only on successful status transition.
    let evt = MarketLocked {
        market: input.market,
        timestamp: input.now_ts,
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
            status: MarketStatus::Open,
            resolved_outcome: None,
            resolution_payload_hash: [0u8; 32],
            resolution_timestamp: 0,
            vault: "VaultA".to_string(),
            market_type: 0,
            rules_version: 1,
        }
    }

    fn base_input() -> LockMarketInput {
        LockMarketInput {
            authority: "AuthA".to_string(),
            config_authority: "AuthA".to_string(),
            market: "MarketA".to_string(),
            market_status: MarketStatus::Open,
            now_ts: 1_800_000_100,
            lock_timestamp: 1_800_000_000,
            market_state: base_market(),
        }
    }

    #[test]
    fn lkm_hp_001_transitions_to_locked_and_emits_event() {
        let (m, e) = lock_market(base_input()).expect("lock_market should pass");
        assert_eq!(m.status, MarketStatus::Locked);
        assert_eq!(e.market, "MarketA");
        assert_eq!(e.timestamp, 1_800_000_100);
    }

    #[test]
    fn lkm_rej_001_to_003_error_mapping() {
        let mut bad = base_input();
        bad.authority = "Other".to_string();
        assert_eq!(lock_market(bad).unwrap_err(), PitStopError::Unauthorized);

        let mut bad = base_input();
        bad.market_status = MarketStatus::Locked;
        assert_eq!(lock_market(bad).unwrap_err(), PitStopError::MarketNotOpen);

        let mut bad = base_input();
        bad.now_ts = 1_799_999_999;
        assert_eq!(lock_market(bad).unwrap_err(), PitStopError::TooEarlyToLock);
    }
}
