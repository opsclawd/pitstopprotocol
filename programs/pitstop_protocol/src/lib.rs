use anchor_lang::prelude::*;

pub mod constants;
pub mod errors;
pub mod instructions;
pub mod state;

declare_id!("PitSTop111111111111111111111111111111111111");

#[program]
pub mod pitstop_protocol {
    use super::*;

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
