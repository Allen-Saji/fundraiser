use crate::state::Fundraiser;
use crate::tests::setup;
use solana_sdk::account::ReadableAccount;
use solana_sdk::{
    account::AccountSharedData,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};

#[test]
pub fn initialize_test() {
    let (program_id, mollusk) = setup();

    let maker = Pubkey::new_from_array([0x01; 32]);
    let (fundraiser, _) =
        Pubkey::find_program_address(&[b"fundraiser", &maker.to_bytes()], &program_id);
    let mint = Pubkey::new_from_array([0x02; 32]);
   
    let data = [
        vec![0],
        maker.to_bytes().to_vec(),   // maker pubkey           
        mint.to_bytes().to_vec(),    //mint to raise          
        100_000_000u64.to_le_bytes().to_vec(), //amount to raise
        i64::MAX.to_le_bytes().to_vec(),     // time ending 
        1u8.to_le_bytes().to_vec(),           //bump
        
    ]
    .concat();

    let instruction = Instruction::new_with_bytes(
        program_id,
        &data,
        vec![
            AccountMeta::new(fundraiser, false),
        ],
    );

    let lamports = mollusk.sysvars.rent.minimum_balance(Fundraiser::LEN);

    let result: mollusk_svm::result::InstructionResult = mollusk.process_instruction(
        &instruction,
        &[
            (
                fundraiser,
                AccountSharedData::new(lamports, Fundraiser::LEN, &program_id),
            ),
        ],
    );
    assert!(
        !result.program_result.is_err(),
        "Initialize instruction failed."
    );

    let fundraiser_result_account = result
        .get_account(&fundraiser)
        .expect("Failed to find fundraiser account");
    let data = fundraiser_result_account.data();
    println!("{:?}", data);
    println!(
        "Amount to raise {}, Mint to raise {}",
        100_000_000u64,
        mint
    );
}