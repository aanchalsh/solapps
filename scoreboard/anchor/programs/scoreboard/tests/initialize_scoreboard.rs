use litesvm::LiteSVM;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_program,
    transaction::Transaction,
};
use std::str::FromStr;

const PROGRAM_ID: &str = "AH4kBFYyJiR1aFkCuJ6zC4PyqiUxsR2hJTFVdfqoPZYn";

fn program_id() -> Pubkey {
    Pubkey::from_str(PROGRAM_ID).unwrap()
}

/// Derive the scoreboard PDA for a given matches value
fn scoreboard_pda(matches: u64) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[&matches.to_le_bytes()], &program_id())
}

/// Build the initialize_scoreboard instruction data (discriminator + args)
fn build_ix_data(matches: u64, place: &str, start_time: u64, end_time: u64) -> Vec<u8> {
    // Anchor discriminator for initialize_scoreboard from IDL
    let discriminator: [u8; 8] = [108, 93, 142, 221, 107, 73, 195, 247];
    let mut data = discriminator.to_vec();
    data.extend_from_slice(&matches.to_le_bytes());
    // String is encoded as 4-byte little-endian length + bytes
    let place_bytes = place.as_bytes();
    data.extend_from_slice(&(place_bytes.len() as u32).to_le_bytes());
    data.extend_from_slice(place_bytes);
    data.extend_from_slice(&start_time.to_le_bytes());
    data.extend_from_slice(&end_time.to_le_bytes());
    data
}

fn setup_svm() -> (LiteSVM, Keypair) {
    let mut svm = LiteSVM::new();
    svm.add_program_from_file(
        program_id(),
        "../../target/deploy/scoreboard.so",
    )
    .expect("Failed to load scoreboard.so — run `anchor build` first");

    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();
    (svm, payer)
}

#[test]
fn test_initialize_scoreboard_succeeds() {
    let (mut svm, payer) = setup_svm();

    let matches: u64 = 5;
    let place = "Mumbai";
    let start_time: u64 = 1_000_000;
    let end_time: u64 = 2_000_000;

    let (scoreboard_pda, _bump) = scoreboard_pda(matches);

    let ix = Instruction {
        program_id: program_id(),
        accounts: vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new(scoreboard_pda, false),
        ],
        data: build_ix_data(matches, place, start_time, end_time),
    };

    let blockhash = svm.latest_blockhash();
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&payer.pubkey()),
        &[&payer],
        blockhash,
    );

    let result = svm.send_transaction(tx);
    assert!(result.is_ok(), "initialize_scoreboard failed: {:?}", result.err());
}

#[test]
fn test_scoreboard_account_data_is_stored_correctly() {
    let (mut svm, payer) = setup_svm();

    let matches: u64 = 10;
    let place = "Delhi";
    let start_time: u64 = 500;
    let end_time: u64 = 1500;

    let (scoreboard_pda, _bump) = scoreboard_pda(matches);

    let ix = Instruction {
        program_id: program_id(),
        accounts: vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new(scoreboard_pda, false),
        ],
        data: build_ix_data(matches, place, start_time, end_time),
    };

    let blockhash = svm.latest_blockhash();
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&payer.pubkey()),
        &[&payer],
        blockhash,
    );

    svm.send_transaction(tx).expect("Transaction failed");

    // Read and verify the account exists and has lamports (was allocated)
    let account = svm.get_account(&scoreboard_pda).expect("Scoreboard account not found");
    assert!(account.lamports > 0, "Scoreboard account should have lamports");
    assert_eq!(account.owner, program_id(), "Account owner should be the scoreboard program");

    // Verify account data contains our values (skip 8-byte discriminator)
    let data = &account.data[8..];
    let stored_matches = u64::from_le_bytes(data[0..8].try_into().unwrap());
    assert_eq!(stored_matches, matches);

    let place_len = u32::from_le_bytes(data[8..12].try_into().unwrap()) as usize;
    let stored_place = std::str::from_utf8(&data[12..12 + place_len]).unwrap();
    assert_eq!(stored_place, place);

    let offset = 12 + place_len;
    let stored_start = u64::from_le_bytes(data[offset..offset + 8].try_into().unwrap());
    let stored_end = u64::from_le_bytes(data[offset + 8..offset + 16].try_into().unwrap());
    assert_eq!(stored_start, start_time);
    assert_eq!(stored_end, end_time);
}

#[test]
fn test_duplicate_initialization_fails() {
    let (mut svm, payer) = setup_svm();

    let matches: u64 = 3;
    let (scoreboard_pda, _bump) = scoreboard_pda(matches);

    let make_ix = || Instruction {
        program_id: program_id(),
        accounts: vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new(scoreboard_pda, false),
        ],
        data: build_ix_data(matches, "Chennai", 100, 200),
    };

    // First init should succeed
    let blockhash = svm.latest_blockhash();
    let tx1 = Transaction::new_signed_with_payer(&[make_ix()], Some(&payer.pubkey()), &[&payer], blockhash);
    svm.send_transaction(tx1).expect("First init should succeed");

    // Second init on the same PDA should fail (account already exists)
    let blockhash = svm.latest_blockhash();
    let tx2 = Transaction::new_signed_with_payer(&[make_ix()], Some(&payer.pubkey()), &[&payer], blockhash);
    let result = svm.send_transaction(tx2);
    assert!(result.is_err(), "Second init on same PDA should fail");
}
