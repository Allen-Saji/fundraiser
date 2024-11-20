use thiserror::Error;
use solana_program::program_error::ProgramError;

#[derive(Debug, Clone, Copy, Error, PartialEq, Eq)]
pub enum FundraiserError {
    #[error("The amount to raise has not been met")]
    TargetNotMet,
    
    #[error("The amount to raise has been achieved")]
    TargetMet,
    
    #[error("The contribution is too big")]
    ContributionTooBig,
    
    #[error("The contribution is too small")]
    ContributionTooSmall,
    
    #[error("The maximum amount to contribute has been reached")]
    MaximumContributionsReached,
    
    #[error("The fundraiser has not ended yet")]
    FundraiserNotEnded,
    
    #[error("The fundraiser has ended")]
    FundraiserEnded,
    
    #[error("Invalid total amount. It should be bigger than 3")]
    InvalidAmount,

    #[error("Invalid fundraiser account")]
    InvalidFundraiserAccount,
}

// Implement the conversion from FundraiserError to ProgramError
impl From<FundraiserError> for ProgramError {
    fn from(e: FundraiserError) -> Self {
        ProgramError::Custom(e as u32)
    }
}