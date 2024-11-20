use solana_program::{
    account_info::AccountInfo, declare_id, entrypoint::ProgramResult, program_error::ProgramError, pubkey::Pubkey
};
declare_id!("CsPs2Z5QLAuqWCw2enAmCfCNmdGahBYysL3cvxv9mmxH");

mod state;
mod instructions;
mod error;


use instructions::*;

solana_program::entrypoint!(process_instruction);
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    if program_id.ne(&ID) {
        return Err(ProgramError::IncorrectProgramId);
    }

    let (instruction_discriminant, instruction_inner_data) = instruction_data.split_at(1);

    match FundraiserInstruction::try_from(instruction_discriminant[0]).unwrap() {
        FundraiserInstruction::InitializeInstruction => process_initialize( accounts, instruction_inner_data)?,
        FundraiserInstruction::CheckerInstruction => check_contributions(program_id,accounts, instruction_data)?,
        FundraiserInstruction::RefundInstruction => refund_instruction(accounts, instruction_data)?,
        FundraiserInstruction::ContributeInstruction => contribute(accounts, instruction_inner_data)?
    }

    Ok(())
}
