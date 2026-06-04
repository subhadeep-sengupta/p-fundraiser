use litesvm::LiteSVM;
use litesvm_token::{
    CreateAssociatedTokenAccount, CreateMint, MintTo,
    spl_token::{self},
};
use solana_clock::Clock;
use solana_instruction::{AccountMeta, Instruction};
use solana_keypair::Keypair;
use solana_message::Message;
use solana_native_token::LAMPORTS_PER_SOL;
use solana_pubkey::Pubkey;
use solana_signer::Signer;
use solana_transaction::Transaction;

const TOKEN_PROGRAM_ID: Pubkey = spl_token::ID;

fn program_id() -> Pubkey {
    Pubkey::from(p_fundraiser::ID)
}

fn setup() -> (LiteSVM, Keypair) {
    let mut svm = LiteSVM::new();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10 * LAMPORTS_PER_SOL)
        .expect("Airdrop failed");

    let bytes = include_bytes!("../target/deploy/p_fundraiser.so");
    svm.add_program(program_id(), bytes)
        .expect("Failed to add program");

    (svm, payer)
}

fn fundraiser_pda_maker(maker: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"fundraiser", maker.as_ref()], &program_id())
}

fn ata_program() -> Pubkey {
    "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL"
        .parse()
        .unwrap()
}

fn system_program() -> Pubkey {
    solana_sdk_ids::system_program::ID
}

fn contributor_pda(fundraiser: &Pubkey, contributor: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[b"contributor", fundraiser.as_ref(), contributor.as_ref()],
        &program_id(),
    )
}
struct FundraiserSetup {
    svm: LiteSVM,
    maker: Keypair,
    mint_to_raise: Pubkey,
    fundraiser: Pubkey,
    vault: Pubkey,
    init_cu: u64,
}

fn setup_initialize_with_discriminator(
    discriminator: u8,
    amount_to_raise: u64,
    duration: u8,
) -> FundraiserSetup {
    let (mut svm, maker) = setup();

    let mint_to_raise = CreateMint::new(&mut svm, &maker)
        .decimals(6)
        .authority(&maker.pubkey())
        .send()
        .unwrap();

    let (fundraiser, bump) = fundraiser_pda_maker(&maker.pubkey());
    let vault = CreateAssociatedTokenAccount::new(&mut svm, &maker, &mint_to_raise)
        .owner(&fundraiser)
        .send()
        .unwrap();

    let data = [
        vec![discriminator],
        vec![bump],
        amount_to_raise.to_le_bytes().to_vec(),
        vec![duration],
    ]
    .concat();

    let ix = Instruction {
        program_id: program_id(),
        accounts: vec![
            AccountMeta::new(maker.pubkey(), true),
            AccountMeta::new_readonly(mint_to_raise, false),
            AccountMeta::new(fundraiser, false),
            AccountMeta::new(vault, false),
            AccountMeta::new_readonly(system_program(), false),
            AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
            AccountMeta::new_readonly(ata_program(), false),
        ],
        data,
    };

    let msg = Message::new(&[ix], Some(&maker.pubkey()));
    let blockhash = svm.latest_blockhash();
    let tx = svm
        .send_transaction(Transaction::new(&[&maker], msg, blockhash))
        .unwrap();

    FundraiserSetup {
        svm,
        maker,
        mint_to_raise,
        fundraiser,
        vault,
        init_cu: tx.compute_units_consumed,
    }
}

fn setup_initialize(amount_to_raise: u64, duration: u8) -> FundraiserSetup {
    setup_initialize_with_discriminator(0, amount_to_raise, duration)
}

pub struct CreateContributorSetup {
    pub svm: LiteSVM,
    pub maker: Keypair,
    pub contributor: Keypair,
    pub mint: Pubkey,
    pub fundraiser_pda: Pubkey,
    pub vault: Pubkey,
    pub contributor_ata: Pubkey,
    pub contributor_state_pda: Pubkey,
    pub create_contributor_cu: u64,
}

pub fn setup_create_contributor(
    amount_to_raise: u64,
    duration: u8,
    contribute_amount: u64,
) -> CreateContributorSetup {
    let mut s = setup_initialize(amount_to_raise, duration);

    let contributor = Keypair::new();
    s.svm
        .airdrop(&contributor.pubkey(), 10 * LAMPORTS_PER_SOL)
        .unwrap();

    let contributor_ata =
        CreateAssociatedTokenAccount::new(&mut s.svm, &contributor, &s.mint_to_raise)
            .owner(&contributor.pubkey())
            .send()
            .unwrap();

    MintTo::new(
        &mut s.svm,
        &s.maker,
        &s.mint_to_raise,
        &contributor_ata,
        contribute_amount,
    )
    .send()
    .unwrap();

    let (contributor_state_pda, bump) = contributor_pda(&s.fundraiser, &contributor.pubkey());

    // discriminator 1 = create_contributor; data[1] = bump; data[2..8] = padding
    let mut data = vec![1u8, bump];
    data.extend_from_slice(&[0u8; 6]);

    let ix = Instruction {
        program_id: program_id(),
        accounts: vec![
            AccountMeta::new(contributor.pubkey(), true),
            AccountMeta::new_readonly(s.fundraiser, false),
            AccountMeta::new(contributor_state_pda, false),
            AccountMeta::new_readonly(Pubkey::new_from_array([0u8; 32]), false), // system_program
        ],
        data,
    };

    let msg = Message::new(&[ix], Some(&contributor.pubkey()));
    let blockhash = s.svm.latest_blockhash();
    let tx = s
        .svm
        .send_transaction(Transaction::new(&[&contributor], msg, blockhash))
        .unwrap();

    CreateContributorSetup {
        svm: s.svm,
        maker: s.maker,
        contributor,
        mint: s.mint_to_raise,
        fundraiser_pda: s.fundraiser,
        vault: s.vault,
        contributor_ata,
        contributor_state_pda,
        create_contributor_cu: tx.compute_units_consumed,
    }
}

pub struct ContributeSetup {
    pub svm: LiteSVM,
    pub maker: Keypair,
    pub contributor: Keypair,
    pub mint: Pubkey,
    pub fundraiser_pda: Pubkey,
    pub vault: Pubkey,
    pub contributor_ata: Pubkey,
    pub contributor_state_pda: Pubkey,
    pub contribute_cu: u64,
}

pub fn setup_contribute(
    amount_to_raise: u64,
    duration: u8,
    contribute_amount: u64,
) -> ContributeSetup {
    let mut s = setup_create_contributor(amount_to_raise, duration, contribute_amount);

    // discriminator 2 = contribute; data[1..9] = amount
    let mut data = vec![2u8];
    data.extend_from_slice(&contribute_amount.to_le_bytes());

    let ix = Instruction {
        program_id: program_id(),
        accounts: vec![
            AccountMeta::new(s.contributor.pubkey(), true),
            AccountMeta::new(s.fundraiser_pda, false),
            AccountMeta::new(s.vault, false),
            AccountMeta::new(s.contributor_ata, false),
            AccountMeta::new(s.contributor_state_pda, false),
            AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
        ],
        data,
    };

    let msg = Message::new(&[ix], Some(&s.contributor.pubkey()));
    let blockhash = s.svm.latest_blockhash();
    let tx = s
        .svm
        .send_transaction(Transaction::new(&[&s.contributor], msg, blockhash))
        .unwrap();

    ContributeSetup {
        svm: s.svm,
        maker: s.maker,
        contributor: s.contributor,
        mint: s.mint,
        fundraiser_pda: s.fundraiser_pda,
        vault: s.vault,
        contributor_ata: s.contributor_ata,
        contributor_state_pda: s.contributor_state_pda,
        contribute_cu: tx.compute_units_consumed,
    }
}

#[test]
fn test_initialize() {
    let s = setup_initialize(1_000_000_000, 7);
    println!("{:<12} | {:>6} CUs", "initialize", s.init_cu);
}

#[test]
fn test_create_contributor() {
    let s = setup_create_contributor(1_000_000_000, 7, 500_000_000);
    println!(
        "{:<20} | {:>6} CUs",
        "create_contributor", s.create_contributor_cu
    );
}

#[test]
fn test_contribute() {
    let s = setup_contribute(1_000_000_000, 7, 500_000_000);
    println!("{:<20} | {:>6} CUs", "contribute", s.contribute_cu);
}

#[test]
fn test_checker() {
    let amount = 1_000_000_000u64;
    let mut s = setup_contribute(amount, 7, amount);

    let maker_ata = CreateAssociatedTokenAccount::new(&mut s.svm, &s.maker, &s.mint)
        .owner(&s.maker.pubkey())
        .send()
        .unwrap();

    let ix = Instruction {
        program_id: program_id(),
        accounts: vec![
            AccountMeta::new(s.maker.pubkey(), true),
            AccountMeta::new(s.fundraiser_pda, false),
            AccountMeta::new(s.vault, false),
            AccountMeta::new(maker_ata, false),
            AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
        ],
        data: vec![3u8], // discriminator 3 = checker
    };

    let msg = Message::new(&[ix], Some(&s.maker.pubkey()));
    let blockhash = s.svm.latest_blockhash();
    let tx = s
        .svm
        .send_transaction(Transaction::new(&[&s.maker], msg, blockhash))
        .unwrap();

    println!("{:<20} | {:>6} CUs", "checker", tx.compute_units_consumed);
}

#[test]
fn test_refund() {
    let goal = 1_000_000_000u64;
    let contributed = 500_000_000u64;

    let mut s = setup_contribute(goal, 1, contributed);

    // warp clock past the 1-day deadline (time_started=0, deadline=86400)
    let mut clock = s.svm.get_sysvar::<Clock>();
    clock.unix_timestamp = 86401;
    s.svm.set_sysvar::<Clock>(&clock);

    let ix = Instruction {
        program_id: program_id(),
        accounts: vec![
            AccountMeta::new(s.contributor.pubkey(), true),
            AccountMeta::new(s.vault, false),
            AccountMeta::new(s.fundraiser_pda, false),
            AccountMeta::new(s.contributor_state_pda, false),
            AccountMeta::new(s.contributor_ata, false),
            AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
        ],
        data: vec![4u8], // discriminator 4 = refund
    };

    let msg = Message::new(&[ix], Some(&s.contributor.pubkey()));
    let blockhash = s.svm.latest_blockhash();
    let tx = s
        .svm
        .send_transaction(Transaction::new(&[&s.contributor], msg, blockhash))
        .unwrap();

    println!("{:<20} | {:>6} CUs", "refund", tx.compute_units_consumed);
}
