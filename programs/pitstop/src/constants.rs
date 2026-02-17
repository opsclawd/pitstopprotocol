use anchor_lang::prelude::{pubkey, Pubkey};

pub const USDC_DECIMALS: u8 = 6;
pub const MAX_CLAIM_WINDOW_SECS: i64 = 7_776_000;
pub const REQUIRED_TOKEN_PROGRAM: &str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";
pub const REQUIRED_TOKEN_PROGRAM_ID: Pubkey = pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");

pub const MAX_OUTCOMES: u8 = 100;
pub const SUPPORTED_MARKET_TYPE: u8 = 0;
pub const SUPPORTED_RULES_VERSION: u16 = 1;
