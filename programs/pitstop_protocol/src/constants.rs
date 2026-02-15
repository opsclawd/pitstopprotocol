pub const MAX_DRIVERS: usize = 20;
pub const BPS_DENOMINATOR: u16 = 10_000;
pub const DEFAULT_FEE_BPS: u16 = 500; // 5%
pub const MIN_BET_LAMPORTS: u64 = 1_000_000; // 0.001 SOL
// Sentinel winner index until settlement occurs.
pub const WINNER_UNSET: u8 = 255;
