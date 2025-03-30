use crate::state::Fundraiser;
use pinocchio::{account_info::AccountInfo, program_error::ProgramError, signer, ProgramResult, sysvars::{clock::Clock, Sysvar}};
use pinocchio_token::instructions::Transfer;

pub fn checker_instruction(accounts: &[AccountInfo]) -> ProgramResult {
    let [maker, maker_ta, fundraiser, vault, _token_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    let fundraiser_account = Fundraiser::from_account_info(fundraiser);
    let bump = fundraiser_account.bump();
    let fundraiser_seed = b"fundraiser".as_ref();
    let maker_seed = maker.key().as_ref();
    let bump_seed = &[bump];

    let current_time = Clock::get()?.unix_timestamp;

    assert!(
        current_time > fundraiser_account.time_ending(),
        "You can only withdraw funds if the fundraiser has ended"
    );

    Transfer {
        from: vault,
        to: maker_ta,
        authority: fundraiser,
        amount: fundraiser_account.amount_raised(),
    }
    .invoke_signed(&[signer!(fundraiser_seed, maker_seed, bump_seed)])?;

    Ok(())
}