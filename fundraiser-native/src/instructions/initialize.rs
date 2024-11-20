use borsh::BorshDeserialize;
use solana_program::{
    account_info::AccountInfo,
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction::create_account,
    sysvar::Sysvar,
};


use crate::{
    state::Fundraiser,
    ID,
};

pub fn process_initialize(
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> Result<(), ProgramError> {
    let [
        maker,
        fundraiser,
        mint_to_raise,
        _system_program
    ] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Deserialize instruction data
    let amount = u64::try_from_slice(&instruction_data[..8])?;
    let time_ending = i64::try_from_slice(&instruction_data[8..])?;

    // Verify accounts
    if !maker.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Verify mint account
    if mint_to_raise.owner != &spl_token::ID || mint_to_raise.owner != &spl_token_2022::ID {
        return Err(ProgramError::InvalidAccountData);
    }

    // Derive and verify PDA
    let (_fundraiser_pda, bump) = Pubkey::find_program_address(
        &[b"fundraiser", maker.key.as_ref()],
        &ID,
    );

    assert!(fundraiser.is_writable && fundraiser.data_is_empty() && fundraiser.owner != &crate::ID);

    // Create fundraiser account
    let minimum_balance = Rent::get()?.minimum_balance(Fundraiser::LEN);
    let init_ix = create_account(
        maker.key,
        fundraiser.key,
        minimum_balance,
        Fundraiser::LEN as u64,
        &ID,
    );

    invoke_signed(
        &init_ix,
        &[maker.clone(), fundraiser.clone()],
        &[&[
            b"fundraiser",
            maker.key.as_ref(),
            &[bump],
        ]],
    )?;

    // Initialize fundraiser data
    Fundraiser::init(
        fundraiser,
        *maker.key,
        *mint_to_raise.key,
        amount,
        time_ending,
        bump,
    )?;

    Ok(())
}