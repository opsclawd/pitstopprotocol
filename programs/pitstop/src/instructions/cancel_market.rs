use crate::{
    error::PitStopError,
    events::MarketCancelled,
    state::{Market, MarketStatus},
};

#[derive(Debug, Clone)]
pub struct CancelMarketInput {
    pub authority: String,
    pub config_authority: String,
    pub close_destination: String,

    pub market: String,
    pub market_status: MarketStatus,
    pub now_ts: i64,
    pub lock_timestamp: i64,

    pub market_state: Market,
    pub vault_amount: u64,
}

fn validate_cancel_market_preconditions(input: &CancelMarketInput) -> Result<(), PitStopError> {
    // CNL-REJ-001: authority must match config authority.
    if input.authority != input.config_authority {
        return Err(PitStopError::Unauthorized);
    }

    // CNL-ADV-001: close_destination is expected to equal authority.
    // Deterministic single error to prevent rent redirection.
    if input.close_destination != input.authority {
        return Err(PitStopError::Unauthorized);
    }

    // CNL-REJ-002: market must be in Seeding.
    if input.market_status != MarketStatus::Seeding {
        return Err(PitStopError::MarketNotSeeding);
    }

    // CNL-REJ-003: cancel is only allowed before lock_timestamp.
    if input.now_ts >= input.lock_timestamp {
        return Err(PitStopError::TooLateToCancel);
    }

    // CNL-REJ-004: total_pool must be zero.
    if input.market_state.total_pool != 0 {
        return Err(PitStopError::MarketHasBets);
    }

    // CNL-REJ-005: vault must be empty prior to close.
    if input.vault_amount != 0 {
        return Err(PitStopError::VaultNotEmpty);
    }

    Ok(())
}

pub fn cancel_market(input: CancelMarketInput) -> Result<(Market, MarketCancelled), PitStopError> {
    validate_cancel_market_preconditions(&input)?;

    // Effects:
    // - close vault ATA with market PDA signer seeds (modeled via preconditions)
    // - market.status = Voided
    // - set resolution timestamp/hash baseline
    let mut market = input.market_state;
    market.status = MarketStatus::Voided;
    market.resolved_outcome = None;
    market.resolution_timestamp = input.now_ts;
    market.resolution_payload_hash = [0u8; 32];

    let evt = MarketCancelled {
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
            total_pool: 0,
            status: MarketStatus::Seeding,
            resolved_outcome: None,
            resolution_payload_hash: [9u8; 32],
            resolution_timestamp: 123,
            vault: "VaultA".to_string(),
            market_type: 0,
            rules_version: 1,
        }
    }

    fn base_input() -> CancelMarketInput {
        CancelMarketInput {
            authority: "AuthA".to_string(),
            config_authority: "AuthA".to_string(),
            close_destination: "AuthA".to_string(),
            market: "MarketA".to_string(),
            market_status: MarketStatus::Seeding,
            now_ts: 1_799_999_999,
            lock_timestamp: 1_800_000_000,
            market_state: base_market(),
            vault_amount: 0,
        }
    }

    #[test]
    fn cnl_hp_001_transitions_to_voided_sets_baseline_and_emits_event() {
        let now = base_input().now_ts;
        let (m, e) = cancel_market(base_input()).expect("cancel_market should pass");
        assert_eq!(m.status, MarketStatus::Voided);
        assert_eq!(m.resolution_timestamp, now);
        assert_eq!(m.resolution_payload_hash, [0u8; 32]);
        assert_eq!(m.resolved_outcome, None);
        assert_eq!(e.market, "MarketA");
        assert_eq!(e.timestamp, now);
    }

    #[test]
    fn cnl_rej_001_to_005_error_mapping() {
        let mut bad = base_input();
        bad.authority = "Other".to_string();
        assert_eq!(cancel_market(bad).unwrap_err(), PitStopError::Unauthorized);

        let mut bad = base_input();
        bad.market_status = MarketStatus::Open;
        assert_eq!(cancel_market(bad).unwrap_err(), PitStopError::MarketNotSeeding);

        let mut bad = base_input();
        bad.now_ts = bad.lock_timestamp;
        assert_eq!(cancel_market(bad).unwrap_err(), PitStopError::TooLateToCancel);

        let mut bad = base_input();
        bad.market_state.total_pool = 1;
        assert_eq!(cancel_market(bad).unwrap_err(), PitStopError::MarketHasBets);

        let mut bad = base_input();
        bad.vault_amount = 1;
        assert_eq!(cancel_market(bad).unwrap_err(), PitStopError::VaultNotEmpty);
    }

    #[test]
    fn cnl_adv_001_close_destination_must_equal_authority() {
        let mut bad = base_input();
        bad.close_destination = "Other".to_string();
        assert_eq!(cancel_market(bad).unwrap_err(), PitStopError::Unauthorized);
    }
}
