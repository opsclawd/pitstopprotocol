use anchor_lang::prelude::*;

// Protocol-wide constants used across instructions and state.
pub mod constants;
pub mod errors;
pub mod instructions;
pub mod state;

// Re-export instruction account contexts/types so entrypoints can refer to them directly.
pub use instructions::*;

declare_id!("PitSTop111111111111111111111111111111111111");

#[program]
pub mod pitstop_protocol {
    use super::*;

        /// Initialize a new race market PDA for an authority.
    ///
    /// This only creates the market state; no bets or funds move here.
    pub fn create_market(
        ctx: Context<CreateMarket>,
        race_id_hash: [u8; 32],
        open_ts: i64,
        close_ts: i64,
        driver_count: u8,
        fee_bps: u16,
    ) -> Result<()> {
        instructions::create_market(
            ctx,
            race_id_hash,
            open_ts,
            close_ts,
            driver_count,
            fee_bps,
        )
    }
}
