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
    InitializeInstruction = 0,
    CheckerInstruction = 1,
    ContributeInstruction = 2,
    RefundInstruction = 3,
}

impl From<u8> for FundraiserInstruction {
    fn from(instruction: u8) -> Self {
        match instruction {
            0 => Self::InitializeInstruction,
            1 => Self::CheckerInstruction,
            2 => Self::ContributeInstruction,
            3 => Self::RefundInstruction,
            _ => panic!("Wrong Instruction")
        }
    }
}