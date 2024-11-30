use borsh::BorshDeserialize;
use solana_program::{
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey::Pubkey,
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

    if !maker.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    if mint_to_raise.owner != &spl_token::ID && mint_to_raise.owner != &spl_token_2022::ID {
        return Err(ProgramError::InvalidAccountData);
    }

    let (_fundraiser_pda, bump) = Pubkey::find_program_address(
        &[b"fundraiser", maker.key.as_ref()],
        &ID,
    );

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