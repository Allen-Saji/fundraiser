use borsh::BorshDeserialize;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
    program_pack::Pack,
    sysvar::{clock::Clock, Sysvar},
};
use spl_token::instruction as token_instruction;
use spl_token::state::Account as TokenAccount;
use crate::{state::Fundraiser, error::FundraiserError};

pub fn check_contributions(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    _instruction_data: &[u8],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    // Get all account infos
    let maker_info = next_account_info(account_info_iter)?;
    let mint_to_raise_info = next_account_info(account_info_iter)?;
    let fundraiser_info = next_account_info(account_info_iter)?;
    let vault_info = next_account_info(account_info_iter)?;
    let maker_ta_info = next_account_info(account_info_iter)?;
    let token_program_info = next_account_info(account_info_iter)?;

    // **1. Verify the maker is a signer**
    if !maker_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // **2. Verify ownership of the fundraiser account**
    if fundraiser_info.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }

    // **3. Deserialize the fundraiser account**
    let fundraiser: Fundraiser = Fundraiser::try_from_slice(&fundraiser_info.data.borrow())?;

    // **4. Verify the fundraiser PDA**
    let (fundraiser_pda, bump_seed) = Pubkey::find_program_address(
        &[b"fundraiser", maker_info.key.as_ref()],
        program_id,
    );
    if fundraiser_pda != *fundraiser_info.key {
        return Err(ProgramError::InvalidSeeds);
    }

    // **5. Check if the fundraiser has ended (no need to check if the target is reached)**
    // let current_time = Clock::get()?.unix_timestamp;
    // if current_time < fundraiser.time_ending {
    //     return Err(ProgramError::Custom(FundraiserError::FundraiserNotEnded as u32));
    // }

    // **6. Verify vault ownership (vault should be owned by the fundraiser PDA)**
    let vault_data = TokenAccount::unpack(&vault_info.try_borrow_data()?)?;
    if vault_data.owner != fundraiser_pda {
        return Err(ProgramError::InvalidAccountData);
    }

    // **7. Verify the maker TA uses the correct mint**
    let maker_ata_data = TokenAccount::unpack(&maker_ta_info.try_borrow_data()?)?;
    if maker_ata_data.mint != *mint_to_raise_info.key {
        return Err(ProgramError::InvalidAccountData);
    }

    // **8. Check if the vault has sufficient balance for transfer**
    let transfer_amount = vault_data.amount;
    if transfer_amount == 0 {
        return Err(ProgramError::InsufficientFunds);
    }

    // **9. Transfer all tokens from vault to maker's ATA**
    let transfer_ix = token_instruction::transfer(
        token_program_info.key,
        vault_info.key,
        maker_ta_info.key,
        &fundraiser_pda,
        &[],
        transfer_amount,
    )?;

    // Sign the transfer with the fundraiser PDA
    invoke_signed(
        &transfer_ix,
        &[
            vault_info.clone(),
            maker_ta_info.clone(),
            fundraiser_info.clone(),
            token_program_info.clone(),
        ],
        &[&[b"fundraiser", maker_info.key.as_ref(), &[bump_seed]]],
    )?;

    // **10. Close the fundraiser account by transferring its lamports to the maker**
    let dest_starting_lamports = maker_info.lamports();
    **maker_info.lamports.borrow_mut() = dest_starting_lamports
        .checked_add(fundraiser_info.lamports())
        .ok_or(ProgramError::ArithmeticOverflow)?;
    **fundraiser_info.lamports.borrow_mut() = 0;

    // **11. Clear the fundraiser data to prevent reuse**
    fundraiser_info.data.borrow_mut().fill(0);

    Ok(())
}
