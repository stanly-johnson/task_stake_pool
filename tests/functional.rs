use {
    solana_program_test::*,
    solana_sdk::{account::Account, signature::Signer, transaction::Transaction},
    solana_sdk::{
        instruction::{AccountMeta, Instruction},
        pubkey::Pubkey,
    },
    task_stake_pool::{entrypoint::process_instruction, instruction::*, processor::*, state::*},
};

#[tokio::test]
async fn test_basic_flow() {
    let program_id = Pubkey::new_unique();
    let source_pubkey = Pubkey::new_unique();
    let destination_pubkey = Pubkey::new_unique();
    let stake_pot_account = Pubkey::new_unique();
    let system_pubkey = Pubkey::new_unique();
    let mut program_test =
        ProgramTest::new("task_bounty", program_id, processor!(process_instruction));
    program_test.add_account(
        source_pubkey,
        Account {
            lamports: 500,
            owner: program_id,
            ..Account::default()
        },
    );
    program_test.add_account(
        destination_pubkey,
        Account {
            lamports: 500,
            ..Account::default()
        },
    );
    program_test.add_account(
        system_pubkey,
        Account {
            lamports: 500,
            ..Account::default()
        },
    );
    program_test.add_account(
        stake_pot_account,
        Account {
            lamports: 500,
            ..Account::default()
        },
    );

    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    let mut transaction = Transaction::new_with_payer(
        &[Instruction::new_with_borsh(
            program_id,
            &TaskInstruction::CreateTask {
                task_audit_program: "".to_string(),
                stake_amount: 1,
                deadline: 1,
                stake_pot_account,
            },
            vec![
                AccountMeta::new(source_pubkey, false),
                AccountMeta::new(destination_pubkey, false),
                AccountMeta::new(system_pubkey, false),
            ],
        )],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();

    let mut transaction = Transaction::new_with_payer(
        &[Instruction::new_with_borsh(
            program_id,
            &TaskInstruction::SubmitTask("".to_string()),
            vec![
                AccountMeta::new(source_pubkey, false),
                AccountMeta::new(destination_pubkey, false),
                AccountMeta::new(system_pubkey, false),
                AccountMeta::new(stake_pot_account, false),
                AccountMeta::new(system_pubkey, false),
            ],
        )],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();

    let mut transaction = Transaction::new_with_payer(
        &[Instruction::new_with_borsh(
            program_id,
            &TaskInstruction::SetTaskToVoting(1),
            vec![
                AccountMeta::new(source_pubkey, false),
                AccountMeta::new(destination_pubkey, false),
                AccountMeta::new(system_pubkey, false),
            ],
        )],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();

    let mut transaction = Transaction::new_with_payer(
        &[Instruction::new_with_borsh(
            program_id,
            &TaskInstruction::Vote,
            vec![
                AccountMeta::new(source_pubkey, false),
                AccountMeta::new(destination_pubkey, false),
                AccountMeta::new(system_pubkey, false),
                AccountMeta::new(stake_pot_account, false),
                AccountMeta::new(system_pubkey, false),
            ],
        )],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();
}
