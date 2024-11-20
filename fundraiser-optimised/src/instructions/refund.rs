use crate::state::{Contributor, Fundraiser};
use pinocchio::account_info::AccountInfo;
use pinocchio::program_error::ProgramError;
use pinocchio::{signer, ProgramResult};
use pinocchio_token::instructions::Transfer;

pub fn refund_instruction(accounts: &[AccountInfo]) -> ProgramResult {
    let [fundraiser, contributor, contributor_ta, vault, _token_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    let fundraiser_account = Fundraiser::from_account_info(fundraiser);
    let contributor_account = Contributor::from_account_info(contributor);

    //checking if the contributor has any contributions
    assert!(contributor_account.amount() > 0, "No amount to refund");

    let maker = fundraiser_account.maker();
    let bump = fundraiser_account.bump();
    let fundraiser_seed = b"fundraiser".as_ref();
    let maker_seed = maker.as_ref();
    let bump_seed = &[bump];

    Transfer {
        from: vault,
        to: contributor_ta,
        authority: fundraiser,
        amount: contributor_account.amount(),
    }
    .invoke_signed(&[signer!(fundraiser_seed, maker_seed, bump_seed)])?;

    unsafe {
        *(fundraiser.borrow_mut_data_unchecked().as_mut_ptr().add(72) as *mut u64) -= contributor_account.amount();
        *(contributor.borrow_mut_data_unchecked().as_mut_ptr() as *mut u64) = 0;
    }

    Ok(())
}