use anchor_lang::prelude::*;

pub mod constants;
pub mod state;

declare_id!("PitSTop111111111111111111111111111111111111");

#[program]
pub mod pitstop_protocol {
    use super::*;

    pub fn placeholder(_ctx: Context<Placeholder>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Placeholder {}
