pub mod instructions;
pub mod state;

use instructions::*;

use pinocchio::{account_info::AccountInfo, entrypoint, pubkey::Pubkey, program_error::ProgramError, ProgramResult};

entrypoint!(process_instruction);

#[cfg(test)]
mod tests;

pub const ID: [u8; 32] =
    five8_const::decode_32_const("22222222222222222222222222222222222222222222");

fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let (discriminator, data) = instruction_data
        .split_first()
        .ok_or(ProgramError::InvalidInstructionData)?;

    match FundraiserInstruction::try_from(discriminator)? {
        FundraiserInstruction::Initialize => initialize_fundraiser(accounts, data),
        FundraiserInstruction::Contribute => contribute_instruction(accounts, data),
        FundraiserInstruction::Checker => checker_instruction(accounts),
        FundraiserInstruction::Refund => refund_instruction(accounts),
    }
}

