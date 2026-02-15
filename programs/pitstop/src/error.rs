#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PitStopError {
    InvalidTokenProgram,
    InvalidMintDecimals,
    InvalidTreasuryMint,
    InvalidTreasuryOwner,
    InvalidCap,
    InvalidClaimWindow,
}
