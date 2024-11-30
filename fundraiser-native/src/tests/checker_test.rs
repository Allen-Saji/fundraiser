use crate::{
    state::Fundraiser,
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

    let mut mint_account = crate::tests::pack_mint(&signer, 1_000_000_000);
    let mut mint_account_data = mint_account.data().to_vec();
    mint_account_data[36..44].copy_from_slice(&1_000_000u64.to_le_bytes());
    mint_account.set_data_from_slice(&mint_account_data);

    let signer_ta_account = crate::tests::pack_token_account(&signer, &mint, 300_000);
    let vault_account = crate::tests::pack_token_account(&fundraiser, &mint, 0);


    let mut fundraiser_account = AccountSharedData::new(
        mollusk.sysvars.rent.minimum_balance(Fundraiser::LEN),
        Fundraiser::LEN,
        &program_id,
    );
    let contributor_account = crate::tests::create_account(
        mollusk.sysvars.rent.minimum_balance(8),
        8,
        &program_id,
    );

    fundraiser_account.set_data_from_slice(
        &[
            maker.to_bytes().to_vec(),
            mint.to_bytes().to_vec(),
            200_000u64.to_le_bytes().to_vec(),
            0u64.to_le_bytes().to_vec(),
            i64::MAX.to_le_bytes().to_vec(), 
            bump.to_le_bytes().to_vec(),
        ]
        .concat(),
    );

    assert_eq!(
        fundraiser_account.lamports(),
        mollusk.sysvars.rent.minimum_balance(Fundraiser::LEN)
    );
    assert_eq!(fundraiser_account.data().len(), Fundraiser::LEN);

    let amount_to_contribute: u64 = 100_000; 
    let contribute_data = [vec![2], amount_to_contribute.to_le_bytes().to_vec()].concat();

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

    let result = mollusk.process_and_validate_instruction(
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
        !result.program_result.is_err(),
        "process_contribute_instruction failed."
    );

    let fundraiser_result_account = result
        .get_account(&fundraiser)
        .expect("Failed to find fundraiser account");
    let fundraiser_data = fundraiser_result_account.data();
    println!(
        "Amount to raise: {:?}",
        u64::from_le_bytes(fundraiser_data[64..72].try_into().unwrap())
    );
    println!(
        "Current amount: {:?}",
        u64::from_le_bytes(fundraiser_data[72..80].try_into().unwrap())
    );
  

    let check_instruction = Instruction::new_with_bytes(
        program_id,
        &[vec![1]].concat(),
        vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(mint, false),
            AccountMeta::new(fundraiser, true),
            AccountMeta::new(vault, true),
            AccountMeta::new(signer_ta, false),
            AccountMeta::new_readonly(token_program, false),
        ],
    );

    let vault_result_account = result
        .get_account(&vault)
        .expect("Failed to find vault account");
    let vault_data = vault_result_account.data();
    let vault_ta_before = unsafe { TokenAccount::from_bytes(vault_data) };
    println!("Vault balance before: {:?}", vault_ta_before.amount());

    let signer_ta_result_account = result
        .get_account(&signer_ta)
        .expect("Failed to find signer_ta account");
    let signer_ta_data = signer_ta_result_account.data();
    let signer_ta_before = unsafe { TokenAccount::from_bytes(signer_ta_data) };
    println!(
        "Signer Token Account balance before: {:?}",
        signer_ta_before.amount()
    );

    let result = mollusk.process_and_validate_instruction(
        &check_instruction,
        &vec![
            (signer, signer_account),
            (mint, mint_account),
            (fundraiser, fundraiser_account),
            (vault, vault_result_account.clone()),
            (signer_ta, signer_ta_account),
            (token_program, token_program_account),
        ],
        &[Check::success()],
    );
    assert!(
        !result.program_result.is_err(),
        "process_check_instruction failed."
    );

    let vault_result_account = result
        .get_account(&vault)
        .expect("Failed to find vault account");
    let vault_data = vault_result_account.data();
    let vault_ta_after = unsafe { TokenAccount::from_bytes(vault_data) };
    println!("Vault balance: {:?}", vault_ta_after.amount());

    let signer_ta_result_account = result
        .get_account(&signer_ta)
        .expect("Failed to find signer_ta account");
    let signer_ta_data = signer_ta_result_account.data();
    let signer_ta_after = unsafe { TokenAccount::from_bytes(signer_ta_data) };
    println!(
        "Signer Token Account balance: {:?}",
        signer_ta_after.amount()
    );

    assert_eq!(
        vault_ta_after.amount(),
        0,
        "Vault balance should be 0 after transfer"
    );
   
}