use std::vec;

use crate::state::Fundraiser;
use crate::tests::setup;
use solana_sdk::account::ReadableAccount;
use solana_sdk::{
    account::AccountSharedData,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    program_pack::Pack,
    system_program
};
use spl_token::state::Mint;

#[test]
pub fn initialize_test() {
    let (program_id, mollusk) = setup();

    let maker = Pubkey::new_from_array([0x01; 32]);
    let maker_account = crate::tests::create_account(
        mollusk
            .sysvars
            .rent
            .minimum_balance(spl_token::state::Account::LEN),
        spl_token::state::Account::LEN,
        &program_id,
    );
    
    let (fundraiser, _) =
        Pubkey::find_program_address(&[b"fundraiser", &maker.to_bytes()], &program_id);
    
    // Create mint account
    let mint = Pubkey::new_from_array([0x02; 32]);
    let mut mint_data = vec![0; Mint::LEN];
    let mint_state = Mint {
        mint_authority: Some(maker).into(),
        supply: 0,
        decimals: 9,
        is_initialized: true,
        freeze_authority: None.into(),
    };
    Mint::pack(mint_state, &mut mint_data).unwrap();
    
    // Create mint account with data
    let mint_account = AccountSharedData::from(solana_sdk::account::Account {
        lamports: mollusk.sysvars.rent.minimum_balance(Mint::LEN),
        data: mint_data,
        owner: spl_token::ID,
        executable: false,
        rent_epoch: 0,
    });

    // Properly serialize the instruction data
    let amount: u64 = 100_000_000;
    let time_ending: i64 = i64::MAX;
    
    // Serialize both values together
    let instruction_data = [
        amount.to_le_bytes().to_vec(),
        time_ending.to_le_bytes().to_vec(),
    ].concat();

    let instruction = Instruction::new_with_bytes(
        program_id,
        &instruction_data,
        vec![
            AccountMeta::new(maker, true),  // Maker is signer
            AccountMeta::new(fundraiser, false),  // Fundraiser is not signer
            AccountMeta::new_readonly(mint, false),  
            AccountMeta::new_readonly(system_program::ID, false)
        ],
    );

    let lamports = mollusk.sysvars.rent.minimum_balance(Fundraiser::LEN);

    let result = mollusk.process_instruction(
        &instruction,
        &[
            (maker, maker_account),
            (fundraiser, AccountSharedData::new(lamports, Fundraiser::LEN, &system_program::ID)),
            (mint, mint_account), 
            (system_program::ID, AccountSharedData::new(0, 0, &system_program::ID)), 
        ],
    );

    assert!(
        !result.program_result.is_err(),
        "Initialize instruction failed: {:?}",
        result.program_result
    );

    let fundraiser_result_account = result
        .get_account(&fundraiser)
        .expect("Failed to find fundraiser account");
    let data = fundraiser_result_account.data();
    println!("{:?}", data);
    println!(
        "Amount to raise {}, Mint to raise {}",
        amount,
        mint
    );
}