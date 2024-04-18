use anchor_lang::{InstructionData, ToAccountMetas};
use solana_program_test::ProgramTest;
use solana_sdk::{instruction::Instruction, signature::Signer, transaction::Transaction};
use without_account::{ID, NOOP_PROGRAM_ID};

#[tokio::test]
async fn test_append_leaves() {
    let mut program_test = ProgramTest::default();
    program_test.add_program("without_account", ID, None);
    program_test.add_program("spl_noop", NOOP_PROGRAM_ID, None);
    program_test.set_compute_max_units(1_400_000u64);

    let mut context = program_test.start_with_context().await;

    let instruction_data = without_account::instruction::AppendLeaves {};
    let accounts = without_account::accounts::AppendLeaves {
        user: context.payer.pubkey(),
        log_wrapper: NOOP_PROGRAM_ID,
    };
    let instruction = Instruction {
        program_id: ID,
        accounts: accounts.to_account_metas(Some(true)),
        data: instruction_data.data(),
    };
    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    context
        .banks_client
        .process_transaction(transaction)
        .await
        .unwrap();
}
