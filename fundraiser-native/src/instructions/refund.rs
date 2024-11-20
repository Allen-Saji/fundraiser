use borsh::{BorshSerialize, BorshDeserialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program::invoke_signed,
    program_pack::Pack,
    program_error::ProgramError,
    sysvar::{clock::Clock, Sysvar},
};
use spl_token::instruction::transfer;
use crate::{state::{Contributor, Fundraiser}, error::FundraiserError};

pub fn refund_instruction(
    accounts: &[AccountInfo],
    _instruction_data: &[u8],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let signer = next_account_info(account_info_iter)?;
    let maker = next_account_info(account_info_iter)?;
    let fundraiser_account = next_account_info(account_info_iter)?;
    let contributor_account_info = next_account_info(account_info_iter)?;
    let contributor_ta = next_account_info(account_info_iter)?;
    let vault = next_account_info(account_info_iter)?;
    let token_program = next_account_info(account_info_iter)?;

    // Ensure the contributor has signed the transaction
    if !signer.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Ownership checks
    if fundraiser_account.owner != &crate::ID {
        return Err(ProgramError::IncorrectProgramId);
    }

    if contributor_account_info.owner != &crate::ID {
        return Err(ProgramError::IncorrectProgramId);
    }

    // Deserialize the Fundraiser and Contributor state
    let mut fundraiser = Fundraiser::try_from_slice(&fundraiser_account.data.borrow())?;
    let mut contributor_account = Contributor::try_from_slice(&contributor_account_info.data.borrow())?;

    // Ensure the contributor has a positive balance
    if contributor_account.amount == 0 {
        return Err(ProgramError::InsufficientFunds);
    }

    // Fundraiser status check
    let current_time = Clock::get()?.unix_timestamp;
    if current_time < fundraiser.time_ending && fundraiser.current_amount < fundraiser.amount_to_raise {
        return Err(ProgramError::Custom(FundraiserError::FundraiserNotEnded as u32));
    }

    // Token Mint Verification
    let contributor_ata_data = spl_token::state::Account::unpack(&contributor_ta.try_borrow_data()?)?;
    if contributor_ata_data.mint != fundraiser.mint_to_raise {
        return Err(ProgramError::InvalidAccountData);
    }

    // Transfer funds from the vault to the contributor's ATA
    let transfer_ix = transfer(
        token_program.key,
        vault.key,
        contributor_ta.key,
        fundraiser_account.key,
        &[],
        contributor_account.amount,
    )?;

    let signer_seeds: &[&[&[u8]]] = &[&[
        b"fundraiser",
        maker.key.as_ref(),
        &[fundraiser.bump],
    ]];

    invoke_signed(
        &transfer_ix,
        &[
            vault.clone(),
            contributor_ta.clone(),
            fundraiser_account.clone(),
            token_program.clone(),
        ],
        signer_seeds,
    )?;

    // Update state: reduce the current amount in the fundraiser
    fundraiser.current_amount -= contributor_account.amount;

    // Reset contributor's amount to zero
    contributor_account.amount = 0;

    // Serialize the updated state back to the account data
    fundraiser.serialize(&mut &mut fundraiser_account.data.borrow_mut()[..])?;
    contributor_account.serialize(&mut &mut contributor_account_info.data.borrow_mut()[..])?;

    Ok(())
}
