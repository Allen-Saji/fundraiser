use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::invoke,
    program_pack::Pack,
    program_error::ProgramError,
    sysvar::{clock::Clock, Sysvar},
};
use spl_token::instruction::transfer;


use crate::{
    state::{Fundraiser, Contributor},
    error::*,
};

pub fn contribute(
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let signer = next_account_info(accounts_iter)?;
    let contributor_account_info = next_account_info(accounts_iter)?;
    let signer_ta = next_account_info(accounts_iter)?;
    let fundraiser_account = next_account_info(accounts_iter)?;
    let vault = next_account_info(accounts_iter)?;
    let token_program = next_account_info(accounts_iter)?;


    if fundraiser_account.owner != &crate::ID {
        msg!("Invalid owner for fundraiser account");
        return Err(ProgramError::IncorrectProgramId);
    }

    if contributor_account_info.owner != &crate::ID {
        msg!("Invalid owner for contributor account");
        return Err(ProgramError::IncorrectProgramId);
    }

    let mut fundraiser: Fundraiser = Fundraiser::try_from_slice(&fundraiser_account.data.borrow())?;
    let mut contributor_account: Contributor = Contributor::try_from_slice(&contributor_account_info.data.borrow())?;
    let amount = u64::from_le_bytes(instruction_data.try_into().unwrap());

    if amount == 0 {
        msg!("Contribution amount must be greater than zero");
        return Err(ProgramError::InvalidInstructionData);
    }

    // Fundraiser duration check
    let current_time = Clock::get()?.unix_timestamp;
    if current_time > fundraiser.time_ending {
        msg!("Fundraiser has ended");
        return Err(ProgramError::Custom(FundraiserError::FundraiserEnded as u32));
    }

    let contributor_ta_data = spl_token::state::Account::unpack(&signer_ta.try_borrow_data()?)?;
    if contributor_ta_data.mint != fundraiser.mint_to_raise {
        msg!("Contributor token account mint does not match the fundraiser mint");
        return Err(ProgramError::InvalidAccountData);
    }

    if contributor_ta_data.amount < amount {
        msg!("Insufficient token balance in contributor's token account");
        return Err(ProgramError::InsufficientFunds);
    }

    // Transfer funds from contributor to the vault
    let transfer_ix = transfer(
        token_program.key,
        signer_ta.key,
        vault.key,
        signer.key,
        &[signer.key],
        amount,
    )?;
    invoke(
        &transfer_ix,
        &[
            signer_ta.clone(),
            vault.clone(),
            signer.clone(),
            token_program.clone(),
        ],
    )?;

    // Update state data
    fundraiser.current_amount += amount;
    contributor_account.amount += amount;

    // Serialize state back to account data
    fundraiser.serialize(&mut *fundraiser_account.data.borrow_mut())?;
    contributor_account.serialize(&mut *contributor_account_info.data.borrow_mut())?;

    Ok(())
}
