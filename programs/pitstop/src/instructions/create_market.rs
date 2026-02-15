use crate::{
    constants::{MAX_OUTCOMES, REQUIRED_TOKEN_PROGRAM, SUPPORTED_MARKET_TYPE, SUPPORTED_RULES_VERSION},
    error::PitStopError,
    events::MarketCreated,
    state::{Market, MarketStatus},
};

#[derive(Debug, Clone)]
pub struct CreateMarketInput {
    pub authority: String,
    pub config_authority: String,
    pub token_program: String,
    pub market: String,
    pub vault: String,
    pub market_id: [u8; 32],
    pub event_id: [u8; 32],
    pub lock_timestamp: i64,
    pub now_ts: i64,
    pub max_outcomes: u8,
    pub market_type: u8,
    pub rules_version: u16,
}

fn recompute_market_id(event_id: [u8; 32], market_type: u8, rules_version: u16) -> [u8; 32] {
    // Canonical formula (SPEC_CANONICAL):
    // market_id = sha256(event_id[32] || market_type[u8] || rules_version[u16-le])
    use sha2::{Digest, Sha256};

    let mut bytes = [0u8; 35];
    bytes[0..32].copy_from_slice(&event_id);
    bytes[32] = market_type;
    bytes[33..35].copy_from_slice(&rules_version.to_le_bytes());

    let digest = Sha256::digest(bytes);
    let mut out = [0u8; 32];
    out.copy_from_slice(&digest);
    out
}

fn validate_create_market_preconditions(input: &CreateMarketInput) -> Result<(), PitStopError> {
    // CRM-REJ-001
    if input.authority != input.config_authority {
        return Err(PitStopError::Unauthorized);
    }
    // CRM-REJ-002
    if input.token_program != REQUIRED_TOKEN_PROGRAM {
        return Err(PitStopError::InvalidTokenProgram);
    }
    // CRM-REJ-003
    if input.lock_timestamp <= input.now_ts {
        return Err(PitStopError::LockInPast);
    }
    // CRM-REJ-004a
    if input.max_outcomes == 0 {
        return Err(PitStopError::ZeroOutcomes);
    }
    // CRM-REJ-004b
    if input.max_outcomes > MAX_OUTCOMES {
        return Err(PitStopError::TooManyOutcomes);
    }
    // CRM-REJ-005a
    if input.market_type != SUPPORTED_MARKET_TYPE {
        return Err(PitStopError::UnsupportedMarketType);
    }
    // CRM-REJ-005b
    if input.rules_version != SUPPORTED_RULES_VERSION {
        return Err(PitStopError::UnsupportedRulesVersion);
    }
    // CRM-REJ-006
    let recomputed = recompute_market_id(input.event_id, input.market_type, input.rules_version);
    if recomputed != input.market_id {
        return Err(PitStopError::InvalidMarketId);
    }

    Ok(())
}

pub fn create_market(input: CreateMarketInput) -> Result<(Market, MarketCreated), PitStopError> {
    validate_create_market_preconditions(&input)?;

    let market = Market {
        market_id: input.market_id,
        event_id: input.event_id,
        lock_timestamp: input.lock_timestamp,
        outcome_count: 0,
        max_outcomes: input.max_outcomes,
        total_pool: 0,
        status: MarketStatus::Seeding,
        resolved_outcome: None,
        resolution_payload_hash: [0u8; 32],
        resolution_timestamp: 0,
        vault: input.vault.clone(),
        market_type: input.market_type,
        rules_version: input.rules_version,
    };

    let evt = MarketCreated {
        market: input.market,
        market_id: input.market_id,
        event_id: input.event_id,
        lock_timestamp: input.lock_timestamp,
        max_outcomes: input.max_outcomes,
        market_type: input.market_type,
        rules_version: input.rules_version,
        timestamp: input.now_ts,
    };

    Ok((market, evt))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_input() -> CreateMarketInput {
        let event_id = [7u8; 32];
        let market_id = recompute_market_id(event_id, SUPPORTED_MARKET_TYPE, SUPPORTED_RULES_VERSION);
        CreateMarketInput {
            authority: "AuthA".to_string(),
            config_authority: "AuthA".to_string(),
            token_program: REQUIRED_TOKEN_PROGRAM.to_string(),
            market: "MarketPdaA".to_string(),
            vault: "VaultAtaA".to_string(),
            market_id,
            event_id,
            lock_timestamp: 1_900_000_000,
            now_ts: 1_800_000_000,
            max_outcomes: 20,
            market_type: SUPPORTED_MARKET_TYPE,
            rules_version: SUPPORTED_RULES_VERSION,
        }
    }



    #[test]
    fn recompute_market_id_matches_locked_vector_b() {
        // SPEC_CANONICAL Vector B:
        // event_id = 32 zero bytes, market_type=0, rules_version=1 (LE 0100)
        // expected market_id hex = b17820b1fb10fa804a7147ca7fd1e1666c62ef002e9adfd12019b35a28377664
        let event_id = [0u8; 32];
        let got = recompute_market_id(event_id, 0, 1);
        let expected: [u8; 32] = [
            0xb1, 0x78, 0x20, 0xb1, 0xfb, 0x10, 0xfa, 0x80,
            0x4a, 0x71, 0x47, 0xca, 0x7f, 0xd1, 0xe1, 0x66,
            0x6c, 0x62, 0xef, 0x00, 0x2e, 0x9a, 0xdf, 0xd1,
            0x20, 0x19, 0xb3, 0x5a, 0x28, 0x37, 0x76, 0x64,
        ];
        assert_eq!(got, expected);
    }

    #[test]
    fn crm_hp_001_builds_market_and_event() {
        let (m, e) = create_market(base_input()).expect("create_market should pass");
        assert_eq!(m.status, MarketStatus::Seeding);
        assert_eq!(m.outcome_count, 0);
        assert_eq!(m.total_pool, 0);
        assert_eq!(m.resolved_outcome, None);
        assert_eq!(m.resolution_timestamp, 0);
        assert_eq!(m.resolution_payload_hash, [0u8; 32]);
        assert_eq!(m.vault, "VaultAtaA");

        assert_eq!(e.market, "MarketPdaA");
        assert_eq!(e.market_id, m.market_id);
        assert_eq!(e.event_id, m.event_id);
        assert_eq!(e.max_outcomes, 20);
        assert_eq!(e.market_type, SUPPORTED_MARKET_TYPE);
        assert_eq!(e.rules_version, SUPPORTED_RULES_VERSION);
        assert_eq!(e.timestamp, 1_800_000_000);
    }

    #[test]
    fn crm_rej_001_to_006_error_mapping() {
        let mut bad = base_input();
        bad.authority = "Other".to_string();
        assert_eq!(create_market(bad).unwrap_err(), PitStopError::Unauthorized);

        let mut bad = base_input();
        bad.token_program = "TokenzFake".to_string();
        assert_eq!(create_market(bad).unwrap_err(), PitStopError::InvalidTokenProgram);

        let mut bad = base_input();
        bad.lock_timestamp = bad.now_ts;
        assert_eq!(create_market(bad).unwrap_err(), PitStopError::LockInPast);

        let mut bad = base_input();
        bad.max_outcomes = 0;
        assert_eq!(create_market(bad).unwrap_err(), PitStopError::ZeroOutcomes);

        let mut bad = base_input();
        bad.max_outcomes = MAX_OUTCOMES + 1;
        assert_eq!(create_market(bad).unwrap_err(), PitStopError::TooManyOutcomes);

        let mut bad = base_input();
        bad.market_type = 2;
        assert_eq!(create_market(bad).unwrap_err(), PitStopError::UnsupportedMarketType);

        let mut bad = base_input();
        bad.rules_version = 2;
        assert_eq!(create_market(bad).unwrap_err(), PitStopError::UnsupportedRulesVersion);

        let mut bad = base_input();
        bad.market_id = [9u8; 32];
        assert_eq!(create_market(bad).unwrap_err(), PitStopError::InvalidMarketId);
    }
}
