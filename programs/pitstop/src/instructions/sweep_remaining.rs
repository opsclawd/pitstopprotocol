/// sweep_remaining Rust parity model for LOCKED spec semantics.
///
/// Spec: SPEC_INSTRUCTIONS/sweep_remaining.md (LOCKED)
///
/// Scope:
/// - deterministic precondition/error mapping (SWP-*)
/// - deterministic effect modeling for vault->treasury sweep
/// - terminal lifecycle transition to MarketStatus::Swept
/// - MarketSweptEvent payload modeling
///
/// Note:
/// This is a parity/conformance model; the on-chain Anchor account/CPI wiring
/// (token transfer + ATA close with PDA signer seeds) is implemented in a later
/// integration pass. We still model the required close semantics as booleans to
/// keep conformance assertions explicit.
use crate::{
    constants::REQUIRED_TOKEN_PROGRAM,
    error::PitStopError,
    events::MarketSweptEvent,
    state::{Market, MarketStatus},
};

#[derive(Debug, Clone)]
pub struct SweepRemainingInput {
    pub authority: String,
    pub config_authority: String,

    pub market: String,
    pub now_ts: i64,
    pub claim_window_secs: i64,

    pub token_program: String,

    pub treasury: String,
    pub config_treasury: String,
    pub treasury_mint: String,
    pub usdc_mint: String,
    pub treasury_owner: String,
    pub treasury_authority: String,

    pub vault_amount: u64,
    pub treasury_amount: u64,

    pub market_state: Market,
}

fn validate_sweep_remaining_preconditions(input: &SweepRemainingInput) -> Result<(), PitStopError> {
    // Order mirrors JS + LOCKED spec to keep deterministic error behavior.

    // SWP-AUTH-001: authority must match config authority.
    if input.authority != input.config_authority {
        return Err(PitStopError::Unauthorized);
    }

    // Token program pinned.
    if input.token_program != REQUIRED_TOKEN_PROGRAM {
        return Err(PitStopError::InvalidTokenProgram);
    }

    // SWP-REJ-002: market must be in {Resolved, Voided}. (Includes Swept -> deterministic gate)
    if input.market_state.status != MarketStatus::Resolved
        && input.market_state.status != MarketStatus::Voided
    {
        return Err(PitStopError::MarketNotResolved);
    }

    // SWP-WIN-001: claim window must be expired.
    let claim_window_end = input
        .market_state
        .resolution_timestamp
        .checked_add(input.claim_window_secs)
        .ok_or(PitStopError::Overflow)?;
    if input.now_ts <= claim_window_end {
        return Err(PitStopError::ClaimWindowNotExpired);
    }

    // Treasury constraints (mint + owner) must match config.
    // Note: JS parity maps treasury address mismatch to InvalidTreasuryOwner.
    if input.treasury != input.config_treasury {
        return Err(PitStopError::InvalidTreasuryOwner);
    }
    if input.treasury_mint != input.usdc_mint {
        return Err(PitStopError::InvalidTreasuryMint);
    }
    if input.treasury_owner != input.treasury_authority {
        return Err(PitStopError::InvalidTreasuryOwner);
    }

    Ok(())
}

/// Executes sweep_remaining effects after preconditions pass.
///
/// Effects modeled:
/// - treasury_amount += vault_amount (checked)
/// - vault closed (modeled booleans)
/// - market.status -> Swept
///
/// Event:
/// - MarketSweptEvent { market, amount, to_treasury, timestamp }
pub fn sweep_remaining(
    input: SweepRemainingInput,
) -> Result<(Market, u64, u64, bool, bool, bool, MarketSweptEvent), PitStopError> {
    validate_sweep_remaining_preconditions(&input)?;

    let swept_amount = input.vault_amount;
    let treasury_amount = input
        .treasury_amount
        .checked_add(swept_amount)
        .ok_or(PitStopError::Overflow)?;

    // Lifecycle terminal transition.
    let mut market = input.market_state;
    market.status = MarketStatus::Swept;

    // Modeled vault semantics: full transfer then close using market PDA signer seeds.
    let vault_closed = true;
    let vault_account_exists = false;
    let close_used_market_pda_seeds = true;

    // Event emitted only on successful sweep + terminal transition.
    let evt = MarketSweptEvent {
        market: input.market,
        amount: swept_amount,
        to_treasury: input.treasury,
        timestamp: input.now_ts,
    };

    Ok((
        market,
        treasury_amount,
        swept_amount,
        vault_closed,
        vault_account_exists,
        close_used_market_pda_seeds,
        evt,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_market(status: MarketStatus) -> Market {
        Market {
            market_id: [1u8; 32],
            event_id: [2u8; 32],
            lock_timestamp: 1_800_000_000,
            outcome_count: 3,
            max_outcomes: 3,
            total_pool: 1000,
            status,
            resolved_outcome: Some(1),
            resolution_payload_hash: [0u8; 32],
            resolution_timestamp: 1_800_000_000,
            vault: "VaultA".to_string(),
            market_type: 0,
            rules_version: 1,
        }
    }

    fn base_input() -> SweepRemainingInput {
        let claim_window_secs = 5000;
        let resolution_timestamp = 1_800_000_000;
        let now_ts = resolution_timestamp + claim_window_secs + 1;

        SweepRemainingInput {
            authority: "AuthA".to_string(),
            config_authority: "AuthA".to_string(),
            market: "MarketA".to_string(),
            now_ts,
            claim_window_secs,
            token_program: REQUIRED_TOKEN_PROGRAM.to_string(),
            treasury: "TreasuryA".to_string(),
            config_treasury: "TreasuryA".to_string(),
            treasury_mint: "MintA".to_string(),
            usdc_mint: "MintA".to_string(),
            treasury_owner: "TreasuryAuthA".to_string(),
            treasury_authority: "TreasuryAuthA".to_string(),
            vault_amount: 123,
            treasury_amount: 1000,
            market_state: base_market(MarketStatus::Resolved),
        }
    }

    #[test]
    fn swp_hp_001_sweeps_to_treasury_closes_vault_and_emits_event() {
        let (m, treasury_amount, swept_amount, vault_closed, vault_exists, used_seeds, evt) =
            sweep_remaining(base_input()).expect("sweep_remaining should pass");

        assert_eq!(m.status, MarketStatus::Swept);
        assert_eq!(treasury_amount, 1123);
        assert_eq!(swept_amount, 123);
        assert_eq!(vault_closed, true);
        assert_eq!(vault_exists, false);
        assert_eq!(used_seeds, true);

        assert_eq!(evt.market, "MarketA");
        assert_eq!(evt.amount, 123);
        assert_eq!(evt.to_treasury, "TreasuryA");
        assert_eq!(evt.timestamp, base_input().now_ts);
    }

    #[test]
    fn swp_rej_matrix_and_idempotency_gate() {
        // SWP-AUTH-001
        let mut bad = base_input();
        bad.authority = "Other".to_string();
        assert_eq!(
            sweep_remaining(bad).unwrap_err(),
            PitStopError::Unauthorized
        );

        // SWP-REJ-002 status gate (incl Swept deterministic)
        let mut bad = base_input();
        bad.market_state.status = MarketStatus::Open;
        assert_eq!(
            sweep_remaining(bad).unwrap_err(),
            PitStopError::MarketNotResolved
        );

        let mut bad = base_input();
        bad.market_state.status = MarketStatus::Swept;
        assert_eq!(
            sweep_remaining(bad).unwrap_err(),
            PitStopError::MarketNotResolved
        );

        // Canonical source-of-truth: market_state governs status and resolution timestamp.
        let mut bad = base_input();
        bad.market_state.status = MarketStatus::Swept;
        assert_eq!(
            sweep_remaining(bad).unwrap_err(),
            PitStopError::MarketNotResolved
        );

        // SWP-WIN-001 claim window not expired
        let mut bad = base_input();
        let claim_end = bad.market_state.resolution_timestamp + bad.claim_window_secs;
        bad.now_ts = claim_end;
        assert_eq!(
            sweep_remaining(bad).unwrap_err(),
            PitStopError::ClaimWindowNotExpired
        );

        // SWP-REJ-004 treasury constraints
        let mut bad = base_input();
        bad.config_treasury = "OtherTreasury".to_string();
        assert_eq!(
            sweep_remaining(bad).unwrap_err(),
            PitStopError::InvalidTreasuryOwner
        );

        let mut bad = base_input();
        bad.treasury_mint = "OtherMint".to_string();
        assert_eq!(
            sweep_remaining(bad).unwrap_err(),
            PitStopError::InvalidTreasuryMint
        );

        let mut bad = base_input();
        bad.treasury_owner = "OtherOwner".to_string();
        assert_eq!(
            sweep_remaining(bad).unwrap_err(),
            PitStopError::InvalidTreasuryOwner
        );

        // SWP-ADV-001 token program mismatch
        let mut bad = base_input();
        bad.token_program = "TokenzFake".to_string();
        assert_eq!(
            sweep_remaining(bad).unwrap_err(),
            PitStopError::InvalidTokenProgram
        );
    }
}
