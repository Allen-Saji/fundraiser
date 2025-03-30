use crate::{
    state::{Contributor, Fundraiser},
    tests::setup,
};
use mollusk_svm::result::Check;
use pinocchio_token::state::TokenAccount;
use solana_sdk::{
    account::{AccountSharedData, ReadableAccount},
    instruction::{AccountMeta, Instruction},
    program_pack::Pack,
    pubkey::Pubkey,
};

#[test]
fn check_test() {
    let (program_id, mollusk) = setup();
    let (token_program, token_program_account) = mollusk_token::token::keyed_account();

    let maker = Pubkey::new_from_array([0x1; 32]);
    let signer = maker;
    let signer_account = crate::tests::create_account(
        mollusk
            .sysvars
            .rent
            .minimum_balance(spl_token::state::Account::LEN),
        spl_token::state::Account::LEN,
        &program_id,
    );
    let signer_ta = Pubkey::new_from_array([0x3; 32]);
    let (fundraiser, bump) =
        Pubkey::find_program_address(&[b"fundraiser".as_ref(), &maker.to_bytes().as_ref()], &program_id);
    let contributor = Pubkey::find_program_address(
        &[
            b"contributor",
            fundraiser.as_ref(),
            signer.to_bytes().as_ref(),
        ],
        &program_id,
    )
    .0;
    let mint = Pubkey::new_from_array([0x4; 32]);
    let vault = Pubkey::new_from_array([0x5; 32]);

    let mut mint_account = crate::tests::pack_mint(&signer, 1_000_000);
    let mut mint_account_data = mint_account.data().to_vec();
    mint_account_data[36..44].copy_from_slice(&1_000_000u64.to_le_bytes());
    mint_account.set_data_from_slice(&mint_account_data);

    let signer_ta_account = crate::tests::pack_token_account(&signer, &mint, 300_000_000);
    let vault_account = crate::tests::pack_token_account(&fundraiser, &mint, 0);

    let mut fundraiser_account = AccountSharedData::new(
        mollusk.sysvars.rent.minimum_balance(Fundraiser::LEN),
        Fundraiser::LEN,
        &program_id,
    );
    let contributor_account = crate::tests::create_account(
        mollusk.sysvars.rent.minimum_balance(Contributor::LEN),
        Contributor::LEN,
        &program_id,
    );

    // Get current time from mollusk's clock
    let current_time = mollusk.sysvars.clock.unix_timestamp;
    
    // Set end time to current time + 1 year (in the future) for contribution
    let one_year_from_now = current_time + (365 * 24 * 60 * 60);
    
    println!("Current time in test: {}", current_time);
    println!("Setting initial end time to: {}", one_year_from_now);

    // Properly set up the fundraiser data with future end time
    let mut buffer = vec![0u8; Fundraiser::LEN];
    buffer[0..32].copy_from_slice(&maker.to_bytes());
    buffer[32..64].copy_from_slice(&mint.to_bytes());
    buffer[64..72].copy_from_slice(&200_000_000u64.to_le_bytes());
    buffer[72..80].copy_from_slice(&0u64.to_le_bytes());
    buffer[80..88].copy_from_slice(&one_year_from_now.to_le_bytes()); // Future timestamp for contribution
    buffer[88..89].copy_from_slice(&[bump]); // bump

    fundraiser_account.set_data_from_slice(&buffer);

    assert_eq!(
        fundraiser_account.lamports(),
        mollusk.sysvars.rent.minimum_balance(Fundraiser::LEN)
    );
    assert_eq!(fundraiser_account.data().len(), Fundraiser::LEN);

    let amount_to_contribute: u64 = 100_000; 
    let contribute_data = [vec![1], amount_to_contribute.to_le_bytes().to_vec()].concat();

    let contribute_instruction = Instruction::new_with_bytes(
        program_id,
        &contribute_data,
        vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(contributor, true),
            AccountMeta::new(signer_ta, false),
            AccountMeta::new(fundraiser, false),
            AccountMeta::new(vault, false),
            AccountMeta::new(token_program, false),
        ],
    );

    let contribute_result = mollusk.process_and_validate_instruction(
        &contribute_instruction,
        &vec![
            (signer, signer_account.clone()),
            (contributor, contributor_account),
            (signer_ta, signer_ta_account.clone()),
            (fundraiser, fundraiser_account.clone()),
            (vault, vault_account.clone()),
            (token_program, token_program_account.clone()),
        ],
        &[Check::success()],
    );
    assert!(
        !contribute_result.program_result.is_err(),
        "process_contribute_instruction failed."
    );

    // Get updated accounts after contribution
    let updated_fundraiser_account = contribute_result
        .get_account(&fundraiser)
        .expect("Failed to find fundraiser account")
        .clone();
    
    let fundraiser_data = updated_fundraiser_account.data();
    println!(
        "Amount to raise: {:?}",
        u64::from_le_bytes(fundraiser_data[64..72].try_into().unwrap())
    );
    println!(
        "Current amount: {:?}",
        u64::from_le_bytes(fundraiser_data[72..80].try_into().unwrap())
    );
    
    // Get updated vault account with the tokens after contribution
    let updated_vault_account = contribute_result
        .get_account(&vault)
        .expect("Failed to find vault account")
        .clone();
    
    // Now modify the fundraiser account to have an end time in the past
    let one_day_ago = current_time - (24 * 60 * 60);
    println!("Setting end time to the past: {}", one_day_ago);
    
    // Create a new fundraiser account with the past end time but preserving the amount raised
    let mut updated_data = updated_fundraiser_account.data().to_vec();
    updated_data[80..88].copy_from_slice(&one_day_ago.to_le_bytes());
    
    // Create a new account with the modified data
    let mut final_fundraiser_account = AccountSharedData::new(
        updated_fundraiser_account.lamports(),
        Fundraiser::LEN,
        &program_id,
    );
    final_fundraiser_account.set_data_from_slice(&updated_data);
  
    let check_instruction = Instruction::new_with_bytes(
        program_id,
        &[vec![2]].concat(),
        vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(signer_ta, false),
            AccountMeta::new(fundraiser, true),
            AccountMeta::new(vault, true),
            AccountMeta::new_readonly(token_program, false),
        ],
    );

    // Use the updated vault account data from the contribute result
    let vault_data = updated_vault_account.data();
    let vault_ta_before = unsafe { TokenAccount::from_bytes(vault_data) };
    println!("Vault balance before: {:?}", vault_ta_before.amount());

    // Use the updated signer_ta from the contribute result
    let updated_signer_ta_account = contribute_result
        .get_account(&signer_ta)
        .expect("Failed to find signer_ta account")
        .clone();
    
    let signer_ta_data = updated_signer_ta_account.data();
    let signer_ta_before = unsafe { TokenAccount::from_bytes(signer_ta_data) };
    println!(
        "Signer Token Account balance before: {:?}",
        signer_ta_before.amount()
    );

    // Execute the check instruction using the updated accounts
    let result = mollusk.process_and_validate_instruction(
        &check_instruction,
        &vec![
            (signer, signer_account),
            (signer_ta, updated_signer_ta_account),
            (fundraiser, final_fundraiser_account),
            (vault, updated_vault_account),
            (token_program, token_program_account),
        ],
        &[Check::success()],
    );
    assert!(
        !result.program_result.is_err(),
        "process_check_instruction failed: {:?}", result.program_result
    );

    let vault_result_account = result
        .get_account(&vault)
        .expect("Failed to find vault account");
    let vault_data = vault_result_account.data();
    let vault_ta_after = unsafe { TokenAccount::from_bytes(vault_data) };
    println!("Vault balance after: {:?}", vault_ta_after.amount());

    let signer_ta_result_account = result
        .get_account(&signer_ta)
        .expect("Failed to find signer_ta account");
    let signer_ta_data = signer_ta_result_account.data();
    let signer_ta_after = unsafe { TokenAccount::from_bytes(signer_ta_data) };
    println!(
        "Signer Token Account balance after: {:?}",
        signer_ta_after.amount()
    );

    assert_eq!(
        vault_ta_after.amount(),
        0,
        "Vault balance should be 0 after transfer"
    );
}