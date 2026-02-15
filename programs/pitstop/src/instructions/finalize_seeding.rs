use crate::{
    error::PitStopError,
    events::MarketOpened,
    state::{Market, MarketStatus},
};

#[derive(Debug, Clone)]
pub struct FinalizeSeedingInput {
    pub authority: String,
    pub config_authority: String,
    pub market: String,
    pub market_status: MarketStatus,
    pub market_outcome_count: u8,
    pub market_max_outcomes: u8,
    pub lock_timestamp: i64,
    pub now_ts: i64,
    pub market_state: Market,
}

fn validate_finalize_seeding_preconditions(input: &FinalizeSeedingInput) -> Result<(), PitStopError> {
    // FSE-REJ-001: only config authority can finalize seeding.
    if input.authority != input.config_authority {
        return Err(PitStopError::Unauthorized);
    }
    // FSE-REJ-002: market must still be in Seeding.
    if input.market_status != MarketStatus::Seeding {
        return Err(PitStopError::MarketNotSeeding);
    }
    // Hardening: mirrored status must match canonical market_state.
    if input.market_state.status != input.market_status {
        return Err(PitStopError::OutcomeMismatch);
    }
    // FSE-REJ-003: all outcomes must be seeded before open transition.
    if input.market_outcome_count != input.market_max_outcomes {
        return Err(PitStopError::SeedingIncomplete);
    }
    // Hardening: mirrored counts must match canonical market_state.
    if input.market_state.outcome_count != input.market_outcome_count
        || input.market_state.max_outcomes != input.market_max_outcomes
    {
        return Err(PitStopError::OutcomeMismatch);
    }
    // FSE-REJ-004: now must be strictly before lock timestamp.
    if input.now_ts >= input.lock_timestamp {
        return Err(PitStopError::TooLateToOpen);
    }
    // Hardening: mirrored lock timestamp must match canonical market_state.
    if input.market_state.lock_timestamp != input.lock_timestamp {
        return Err(PitStopError::OutcomeMismatch);
    }

    Ok(())
}

pub fn finalize_seeding(input: FinalizeSeedingInput) -> Result<(Market, MarketOpened), PitStopError> {
    validate_finalize_seeding_preconditions(&input)?;

    // Effect contract: transition Seeding -> Open only.
    let mut market = input.market_state;
    market.status = MarketStatus::Open;

    // Event contract: emit MarketOpened only on successful transition.
    let evt = MarketOpened {
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
            lock_timestamp: 1_800_000_100,
            outcome_count: 3,
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

    fn base_input() -> FinalizeSeedingInput {
        FinalizeSeedingInput {
            authority: "AuthA".to_string(),
            config_authority: "AuthA".to_string(),
            market: "MarketPdaA".to_string(),
            market_status: MarketStatus::Seeding,
            market_outcome_count: 3,
            market_max_outcomes: 3,
            lock_timestamp: 1_800_000_100,
            now_ts: 1_800_000_000,
            market_state: base_market(),
        }
    }

    #[test]
    fn fse_hp_001_transitions_market_to_open_and_emits_event() {
        let (m, e) = finalize_seeding(base_input()).expect("finalize_seeding should pass");
        assert_eq!(m.status, MarketStatus::Open);
        assert_eq!(e.market, "MarketPdaA");
        assert_eq!(e.timestamp, 1_800_000_000);
    }

    #[test]
    fn fse_rej_001_to_004_error_mapping() {
        let mut bad = base_input();
        bad.authority = "Other".to_string();
        assert_eq!(finalize_seeding(bad).unwrap_err(), PitStopError::Unauthorized);

        let mut bad = base_input();
        bad.market_status = MarketStatus::Open;
        assert_eq!(finalize_seeding(bad).unwrap_err(), PitStopError::MarketNotSeeding);

        let mut bad = base_input();
        bad.market_outcome_count = 2;
        assert_eq!(finalize_seeding(bad).unwrap_err(), PitStopError::SeedingIncomplete);

        let mut bad = base_input();
        bad.now_ts = 1_800_000_100;
        assert_eq!(finalize_seeding(bad).unwrap_err(), PitStopError::TooLateToOpen);
    }

    #[test]
    fn fse_rej_hardening_mirrored_fields_must_match_market_state() {
        let mut bad = base_input();
        bad.market_state.status = MarketStatus::Open;
        assert_eq!(finalize_seeding(bad).unwrap_err(), PitStopError::OutcomeMismatch);

        let mut bad = base_input();
        bad.market_state.outcome_count = 2;
        assert_eq!(finalize_seeding(bad).unwrap_err(), PitStopError::OutcomeMismatch);

        let mut bad = base_input();
        bad.market_state.max_outcomes = 4;
        assert_eq!(finalize_seeding(bad).unwrap_err(), PitStopError::OutcomeMismatch);

        let mut bad = base_input();
        bad.market_state.lock_timestamp = 1_800_000_200;
        assert_eq!(finalize_seeding(bad).unwrap_err(), PitStopError::OutcomeMismatch);
    }
}
