use crate::{
    state::{Contributor, Fundraiser},
    tests::setup,
};
use mollusk_svm::result::Check;
use pinocchio_token::state::TokenAccount;
use solana_sdk::{
    account::{AccountSharedData, ReadableAccount},
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};

#[test]
fn refund_test() {
    let (program_id, mollusk) = setup();
    let (token_program, token_program_account) = mollusk_token::token::keyed_account();

    let maker = Pubkey::new_from_array([0x1; 32]);
    let contributor = Pubkey::new_from_array([0x6; 32]);
    let (fundraiser, bump) =
        Pubkey::find_program_address(&[b"fundraiser".as_ref(), &maker.to_bytes().as_ref()], &program_id);
    let contributor_ta = Pubkey::new_from_array([0x7; 32]);
    let vault = Pubkey::new_from_array([0x8; 32]);
    let mint = Pubkey::new_from_array([0x9; 32]);

    let vault_account = crate::tests::pack_token_account(&fundraiser, &mint, 100_000_000);
    let contributor_ta_account = crate::tests::pack_token_account(&contributor, &mint, 0);
    let mut contributor_account = crate::tests::create_account(
        mollusk.sysvars.rent.minimum_balance(Contributor::LEN),
        Contributor::LEN,
        &program_id,
    );

    let mut fundraiser_account = AccountSharedData::new(
        mollusk.sysvars.rent.minimum_balance(Fundraiser::LEN),
        Fundraiser::LEN,
        &program_id,
    );


    fundraiser_account.set_data_from_slice(
        &[
            maker.to_bytes().to_vec(),
            mint.to_bytes().to_vec(),
            100_000_000u64.to_le_bytes().to_vec(), // amount_to_raise
            100_000u64.to_le_bytes().to_vec(), // current_amount
            i64::MAX.to_le_bytes().to_vec(), // time_ending
            bump.to_le_bytes().to_vec(),   // bump
        ]
        .concat(),
    );

    contributor_account.set_data_from_slice(&100_000u64.to_le_bytes());

    let refund_data = vec![vec![3]].concat();

    let refund_instruction = Instruction::new_with_bytes(
        program_id,
        &refund_data,
        vec![
            AccountMeta::new(fundraiser, true),
            AccountMeta::new(contributor, false),
            AccountMeta::new(contributor_ta, false),
            AccountMeta::new(vault, true),
            AccountMeta::new_readonly(token_program, false),
        ],
    );

    let result = mollusk.process_and_validate_instruction(
        &refund_instruction,
        &vec![
            (fundraiser, fundraiser_account),
            (contributor, contributor_account),
            (contributor_ta, contributor_ta_account),
            (vault, vault_account),
            (token_program, token_program_account),
        ],
        &[Check::success()],
    );
    assert!(
        !result.program_result.is_err(),
        "process_refund_instruction failed."
    );
    println!("Compute Units: {}", result.compute_units_consumed);

    let vault_result = result
        .get_account(&vault)
        .expect("Failed to find vault account");
    let vault_data = vault_result.data();
    let vault_ta = unsafe { TokenAccount::from_bytes(vault_data) };
    assert_ne!(vault_ta.amount(), 100_000, "Vault should be empty after refund");

    let contributor_ta_result = result
        .get_account(&contributor_ta)
        .expect("Failed to find contributor_ta account");
    let contributor_ta_data = contributor_ta_result.data();
    let contributor_ta = unsafe { TokenAccount::from_bytes(contributor_ta_data) };
    assert_eq!(
        contributor_ta.amount(),
        100_000,
        "Contributor should have received their refund"
    );
}