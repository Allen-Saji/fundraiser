use pinocchio::program_error::ProgramError;

pub mod checker;
pub mod contribute;
pub mod refund;
pub mod initialize;

pub use checker::*;
pub use contribute::*;
pub use refund::*;
pub use initialize::*;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum FundraiserInstruction {
    Initialize = 0,
    Checker = 1,
    Contribute = 2,
    Refund = 3,
}

impl TryFrom<&u8> for FundraiserInstruction {
    type Error = ProgramError;

    fn try_from(value: &u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(FundraiserInstruction::Initialize),
            1 => Ok(FundraiserInstruction::Contribute),
            2 => Ok(FundraiserInstruction::Checker),
            3 => Ok(FundraiserInstruction::Refund),
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}