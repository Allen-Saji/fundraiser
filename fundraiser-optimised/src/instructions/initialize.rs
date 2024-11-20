use pinocchio::{
    account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey, ProgramResult,
};

use crate::state::fundraiser::Fundraiser;

// --Data Scheme--
// maker: Pubkey
// mint_to_raise: Pubkey
// amount_to_raise: u64
// amount_raised: u64 (initialized to 0, user should not pass this)
// time_started: i64
// duration: u8

pub fn initialize_fundraiser(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    // Destructure the accounts array
    let [fundraiser] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Verify the data length is exactly as expected
    if data.len() != Fundraiser::LEN - 8 { // We skip `amount_raised` since it will be initialized to 0
        return Err(ProgramError::InvalidInstructionData);
    }

    // Unsafe data manipulation:
    let data_ptr = unsafe {fundraiser.borrow_mut_data_unchecked().as_mut_ptr()};

    // Copy the maker key (32 bytes)
    unsafe { *(data_ptr as *mut Pubkey) = *(data.as_ptr() as *const Pubkey); };

    // Copy mint_to_raise (32 bytes) from input data to account data
    unsafe {
        *(data_ptr.add(32) as *mut Pubkey) = *(data.as_ptr() as *const Pubkey);
    }

    // Copy amount_to_raise (8 bytes) from input data to account data
    unsafe {
        *(data_ptr.add(64) as *mut u64) = *(data.as_ptr().add(32) as *const u64);
    }

    // Initialize amount_raised (8 bytes) to 0
    unsafe {
        *(data_ptr.add(72) as *mut u64) = 0;
    }

    // Copy time_started (8 bytes) from input data to account data
    unsafe {
        *(data_ptr.add(80) as *mut i64) = *(data.as_ptr().add(40) as *const i64);
    }

    // Copy duration (1 byte) from input data to account data
    unsafe {
        *(data_ptr.add(88) as *mut u8) = *(data.as_ptr().add(48) as *const u8);
    }

    Ok(())
}
