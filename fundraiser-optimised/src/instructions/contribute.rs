use crate::state::Fundraiser;
use pinocchio::{account_info::AccountInfo, program_error::ProgramError, ProgramResult,  sysvars::{clock::Clock, Sysvar}};
use pinocchio_token::instructions::Transfer;

pub fn contribute_instruction(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let amount: u64 = unsafe { *(data.as_ptr() as *const u64) };

    let [signer, contributor, signer_ta, fundraiser, vault, _token_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    let fundraiser_account = Fundraiser::from_account_info_unchecked(fundraiser);

    let current_time = Clock::get()?.unix_timestamp;
    println!("current time: {:?}, fundraiser end time: {:?}", current_time, fundraiser_account.time_ending());
    
    assert!(
        current_time <= fundraiser_account.time_ending(),
        "Fundraiser has ended"
    );

    Transfer {
        from: signer_ta,
        to: vault,
        authority: signer,
        amount,
    }
    .invoke()?;

    unsafe {
        *(fundraiser.borrow_mut_data_unchecked().as_mut_ptr().add(72) as *mut u64) += amount;
        *(contributor.borrow_mut_data_unchecked().as_mut_ptr() as *mut u64) += amount;
    }

    Ok(())
}