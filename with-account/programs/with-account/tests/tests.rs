use std::mem;

use anchor_lang::{solana_program::system_instruction, InstructionData, ToAccountMetas};
use solana_program_test::ProgramTest;
use solana_sdk::{
    instruction::Instruction,
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use with_account::{Buffers, ID, NOOP_PROGRAM_ID};

#[tokio::test]
async fn test_append_leaves() {
    let mut program_test = ProgramTest::default();
    program_test.add_program("with_account", ID, None);
    program_test.add_program("spl_noop", NOOP_PROGRAM_ID, None);

    let mut context = program_test.start_with_context().await;

    let buffers_keypair = Keypair::new();
    let buffers_size = mem::size_of::<Buffers>() + 8;
    let buffers_rent = context
        .banks_client
        .get_rent()
        .await
        .unwrap()
        .minimum_balance(buffers_size);
    let create_account_ix = system_instruction::create_account(
        &context.payer.pubkey(),
        &buffers_keypair.pubkey(),
        buffers_rent,
        buffers_size as u64,
        &ID,
    );

    let instruction_data = with_account::instruction::AppendLeaves {};
    let accounts = with_account::accounts::AppendLeaves {
        user: context.payer.pubkey(),
        buffers: buffers_keypair.pubkey(),
        log_wrapper: NOOP_PROGRAM_ID,
    };
    let instruction = Instruction {
        program_id: ID,
        accounts: accounts.to_account_metas(Some(true)),
        data: instruction_data.data(),
    };
    let transaction = Transaction::new_signed_with_payer(
        &[create_account_ix, instruction],
        Some(&context.payer.pubkey()),
        &[&buffers_keypair, &context.payer],
        context.last_blockhash,
    );

    context
        .banks_client
        .process_transaction(transaction)
        .await
        .unwrap();
}
