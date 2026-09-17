use {
    anchor_spl::associated_token,
    litesvm::LiteSVM,
    litesvm_token::CreateMint,
    solana_keypair::Keypair,
    solana_message::{Instruction, Message, VersionedMessage},
    solana_pubkey::Pubkey,
    solana_signer::Signer,
    solana_transaction::versioned::VersionedTransaction,
};

mod ix_handlers;
use ix_handlers::*;

fn send(
    svm: &mut LiteSVM,
    ixs: &[Instruction],
    payer: &Keypair,
    signers: &[&Keypair],
) -> litesvm::types::TransactionResult {
    svm.expire_blockhash();
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(ixs, Some(&payer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), signers).unwrap();
    svm.send_transaction(tx)
}

// Reads an SPL token account's balance directly from LiteSVM's ledger.
//
// NOTE: the Pack import below is my best guess at where litesvm_token
// re-exports it from. If this doesn't compile, the compiler error will
// name the correct path — swap the `use` line accordingly rather than
// guessing further.
fn token_balance(svm: &LiteSVM, address: &Pubkey) -> u64 {
    let account = svm
        .get_account(address)
        .expect("token account should exist");

    assert!(
        account.data.len() >= 72,
        "account data is too short to be an SPL token account"
    );

    u64::from_le_bytes(
        account.data[64..72]
            .try_into()
            .expect("token amount should be 8 bytes"),
    )
}

#[allow(clippy::type_complexity)]
fn setup() -> (
    LiteSVM,
    Keypair,
    Pubkey,
    Pubkey,
    Pubkey,
    Pubkey,
    Pubkey,
    Pubkey,
    Pubkey,
    Pubkey,
    Pubkey,
) {
    let program_id = amm_video::id();
    let payer = Keypair::new();
    let mut svm = LiteSVM::new();
    let bytes = include_bytes!("../../../target/deploy/amm_video.so");
    svm.add_program(program_id, bytes).unwrap();
    svm.airdrop(&payer.pubkey(), 1_000_000_000).unwrap();

    let mint_x = CreateMint::new(&mut svm, &payer)
        .decimals(6)
        .authority(&payer.pubkey())
        .send()
        .unwrap();

    let mint_y = CreateMint::new(&mut svm, &payer)
        .decimals(6)
        .authority(&payer.pubkey())
        .send()
        .unwrap();

    let config =
        Pubkey::find_program_address(&[b"config", &123u64.to_le_bytes()], &amm_video::id()).0;
    let mint_lp = Pubkey::find_program_address(&[b"lp", config.as_ref()], &amm_video::id()).0;

    let vault_x = associated_token::get_associated_token_address(&config, &mint_x);
    let vault_y = associated_token::get_associated_token_address(&config, &mint_y);

    let treasury =
        Pubkey::find_program_address(&[b"treasury", config.as_ref()], &amm_video::id()).0;
    let treasury_x = associated_token::get_associated_token_address(&treasury, &mint_x);
    let treasury_y = associated_token::get_associated_token_address(&treasury, &mint_y);

    (
        svm, payer, mint_x, mint_y, config, mint_lp, vault_x, vault_y, treasury, treasury_x,
        treasury_y,
    )
}

// Runs setup(), initializes with the given protocol_fee, and deposits
// initial liquidity (200_000_000 of each token — the first deposit takes
// max_x/max_y directly since the pool starts empty). Every treasury test
// below builds on this rather than repeating init+deposit boilerplate.
#[allow(clippy::type_complexity)]
fn setup_with_liquidity(
    protocol_fee: u16,
) -> (
    LiteSVM,
    Keypair,
    Pubkey,
    Pubkey,
    Pubkey,
    Pubkey,
    Pubkey,
    Pubkey,
    Pubkey,
    Pubkey,
    Pubkey,
) {
    let (mut svm, payer, mint_x, mint_y, config, mint_lp, vault_x, vault_y, treasury, treasury_x, treasury_y) =
        setup();

    let init_ix = create_initialise_ix(
        &mut svm, &payer, mint_x, mint_y, config, mint_lp, vault_x, vault_y, treasury,
        treasury_x, treasury_y, protocol_fee,
    );
    let deposit_ix = create_deposit_ix(
        &mut svm, &payer, mint_x, mint_y, mint_lp, config, vault_x, vault_y,
    );

    let res = send(&mut svm, &[init_ix, deposit_ix], &payer, &[&payer]);
    assert!(res.is_ok(), "setup init+deposit should succeed: {:?}", res.err());

    (
        svm, payer, mint_x, mint_y, config, mint_lp, vault_x, vault_y, treasury, treasury_x,
        treasury_y,
    )
}

#[test]
fn test_initialize() {
    let (mut svm, payer, mint_x, mint_y, config, mint_lp, vault_x, vault_y, treasury, treasury_x, treasury_y) =
        setup();

    let instruction = create_initialise_ix(
        &mut svm, &payer, mint_x, mint_y, config, mint_lp, vault_x, vault_y, treasury,
        treasury_x, treasury_y, 10,
    );
    let res = send(&mut svm, &[instruction], &payer, &[&payer]);
    assert!(res.is_ok());
}

#[test]
pub fn test_deposit() {
    let (mut svm, payer, mint_x, mint_y, config, mint_lp, vault_x, vault_y, treasury, treasury_x, treasury_y) =
        setup();
    let init_ix = create_initialise_ix(
        &mut svm, &payer, mint_x, mint_y, config, mint_lp, vault_x, vault_y, treasury,
        treasury_x, treasury_y, 10,
    );

    let deposit_ix = create_deposit_ix(
        &mut svm, &payer, mint_x, mint_y, mint_lp, config, vault_x, vault_y,
    );

    let res = send(&mut svm, &[init_ix, deposit_ix], &payer, &[&payer]);
    assert!(res.is_ok());
}

#[test]
pub fn test_withdraw() {
    let (mut svm, payer, mint_x, mint_y, config, mint_lp, vault_x, vault_y, treasury, treasury_x, treasury_y) =
        setup();
    let init_ix = create_initialise_ix(
        &mut svm, &payer, mint_x, mint_y, config, mint_lp, vault_x, vault_y, treasury,
        treasury_x, treasury_y, 10,
    );

    let deposit_ix = create_deposit_ix(
        &mut svm, &payer, mint_x, mint_y, mint_lp, config, vault_x, vault_y,
    );

    let withdraw_ix = create_withdraw_ix(
        &mut svm, &payer, mint_x, mint_y, mint_lp, config, vault_x, vault_y,
    );
    let res = send(
        &mut svm,
        &[init_ix, deposit_ix, withdraw_ix],
        &payer,
        &[&payer],
    );
    assert!(res.is_ok());
}

#[test]
pub fn test_swap() {
    let (mut svm, payer, mint_x, mint_y, config, mint_lp, vault_x, vault_y, treasury, treasury_x, treasury_y) =
        setup();
    let init_ix = create_initialise_ix(
        &mut svm, &payer, mint_x, mint_y, config, mint_lp, vault_x, vault_y, treasury,
        treasury_x, treasury_y, 10,
    );

    let deposit_ix = create_deposit_ix(
        &mut svm, &payer, mint_x, mint_y, mint_lp, config, vault_x, vault_y,
    );

    let swap_ix = create_swap_ix(
        &mut svm, &payer, mint_x, mint_y, mint_lp, config, vault_x, vault_y, treasury,
        treasury_x, treasury_y, true, 10_000_000, 5_000_000,
    );

    let res = send(&mut svm, &[init_ix, deposit_ix, swap_ix], &payer, &[&payer]);
    assert!(res.is_ok());
}

// ---------- treasury tests ----------

// 1. X→Y swaps pay their protocol fee in Y, and must never touch treasury_x.
#[test]
fn swap_x_to_y_routes_fee_to_treasury_x_untouched() {
    let (mut svm, payer, mint_x, mint_y, config, mint_lp, vault_x, vault_y, treasury, treasury_x, treasury_y) =
        setup_with_liquidity(10);

    let treasury_x_before = token_balance(&svm, &treasury_x);
    let treasury_y_before = token_balance(&svm, &treasury_y);

    let swap_ix = create_swap_ix(
        &mut svm, &payer, mint_x, mint_y, mint_lp, config, vault_x, vault_y, treasury,
        treasury_x, treasury_y, true, 10_000_000, 5_000_000,
    );
    send(&mut svm, &[swap_ix], &payer, &[&payer]).expect("x-to-y swap should succeed");

    let treasury_x_after = token_balance(&svm, &treasury_x);
    let treasury_y_after = token_balance(&svm, &treasury_y);

    assert_eq!(
        treasury_x_before, treasury_x_after,
        "an X-to-Y swap must never move funds into treasury_x"
    );
    assert!(
        treasury_y_after > treasury_y_before,
        "an X-to-Y swap should route its protocol fee into treasury_y"
    );
}

// 2. Y→X swaps pay their protocol fee in X, and must never touch treasury_y.
#[test]
fn swap_y_to_x_routes_fee_to_treasury_y_untouched() {
    let (mut svm, payer, mint_x, mint_y, config, mint_lp, vault_x, vault_y, treasury, treasury_x, treasury_y) =
        setup_with_liquidity(10);

    let treasury_x_before = token_balance(&svm, &treasury_x);
    let treasury_y_before = token_balance(&svm, &treasury_y);

    let swap_ix = create_swap_ix(
        &mut svm, &payer, mint_x, mint_y, mint_lp, config, vault_x, vault_y, treasury,
        treasury_x, treasury_y, false, 10_000_000, 5_000_000,
    );
    send(&mut svm, &[swap_ix], &payer, &[&payer]).expect("y-to-x swap should succeed");

    let treasury_x_after = token_balance(&svm, &treasury_x);
    let treasury_y_after = token_balance(&svm, &treasury_y);

    assert_eq!(
        treasury_y_before, treasury_y_after,
        "a Y-to-X swap must never move funds into treasury_y"
    );
    assert!(
        treasury_x_after > treasury_x_before,
        "a Y-to-X swap should route its protocol fee into treasury_x"
    );
}

// 3. The split is exact: treasury gets floor(total_out * protocol_fee / 10_000),
// the user gets the remainder, and nothing is created or lost in between.
#[test]
fn protocol_fee_split_is_exact() {
    let (mut svm, payer, mint_x, mint_y, config, mint_lp, vault_x, vault_y, treasury, treasury_x, treasury_y) =
        setup_with_liquidity(10);

    let user_y = associated_token::get_associated_token_address(&payer.pubkey(), &mint_y);

    let vault_y_before = token_balance(&svm, &vault_y);
    let treasury_y_before = token_balance(&svm, &treasury_y);
    let user_y_before = token_balance(&svm, &user_y);

    let swap_ix = create_swap_ix(
        &mut svm, &payer, mint_x, mint_y, mint_lp, config, vault_x, vault_y, treasury,
        treasury_x, treasury_y, true, 10_000_000, 5_000_000,
    );
    send(&mut svm, &[swap_ix], &payer, &[&payer]).expect("swap should succeed");

    let vault_y_after = token_balance(&svm, &vault_y);
    let treasury_y_after = token_balance(&svm, &treasury_y);
    let user_y_after = token_balance(&svm, &user_y);

    let total_withdrawn = vault_y_before - vault_y_after;
    let treasury_got = treasury_y_after - treasury_y_before;
    let user_got = user_y_after - user_y_before;

    let expected_treasury_cut = ((total_withdrawn as u128 * 10u128) / 10_000u128) as u64;

    assert_eq!(
        treasury_got, expected_treasury_cut,
        "treasury should receive exactly floor(total_out * protocol_fee / 10_000)"
    );
    assert_eq!(
        user_got,
        total_withdrawn - treasury_got,
        "user should receive exactly the remainder after the treasury's cut"
    );
    assert_eq!(
        user_got + treasury_got,
        total_withdrawn,
        "the split must fully account for every unit withdrawn from the vault"
    );
}

// 4. protocol_fee = 0 must be a true no-op on the treasury — not a
// zero-amount transfer, but literally no CPI touching that account.
#[test]
fn zero_protocol_fee_pays_no_treasury() {
    let (mut svm, payer, mint_x, mint_y, config, mint_lp, vault_x, vault_y, treasury, treasury_x, treasury_y) =
        setup_with_liquidity(0);

    let user_y = associated_token::get_associated_token_address(&payer.pubkey(), &mint_y);

    let vault_y_before = token_balance(&svm, &vault_y);
    let treasury_y_before = token_balance(&svm, &treasury_y);
    let user_y_before = token_balance(&svm, &user_y);

    let swap_ix = create_swap_ix(
        &mut svm, &payer, mint_x, mint_y, mint_lp, config, vault_x, vault_y, treasury,
        treasury_x, treasury_y, true, 10_000_000, 5_000_000,
    );
    send(&mut svm, &[swap_ix], &payer, &[&payer]).expect("swap should succeed even with a zero protocol fee");

    let vault_y_after = token_balance(&svm, &vault_y);
    let treasury_y_after = token_balance(&svm, &treasury_y);
    let user_y_after = token_balance(&svm, &user_y);

    assert_eq!(
        treasury_y_before, treasury_y_after,
        "treasury balance must not change when protocol_fee is 0"
    );
    assert_eq!(
        user_y_after - user_y_before,
        vault_y_before - vault_y_after,
        "with no protocol fee, the user should receive the full amount withdrawn from the vault"
    );
}

// 5. The min-out check must apply AFTER the protocol fee is subtracted,
// not just against the curve's raw (pre-fee) output.
#[test]
fn swap_respects_min_after_protocol_fee() {
    let mint_x_dummy_amount = 10_000_000u64;

    // Step 1: discover what the user actually receives, post-fee, with no
    // slippage constraint (min = 0), so this call cannot fail on its own.
    let (mut svm_a, payer_a, mint_x_a, mint_y_a, config_a, mint_lp_a, vault_x_a, vault_y_a, treasury_a, treasury_x_a, treasury_y_a) =
        setup_with_liquidity(10);
    let user_y_a = associated_token::get_associated_token_address(&payer_a.pubkey(), &mint_y_a);
    let user_y_before = token_balance(&svm_a, &user_y_a);

    let probe_ix = create_swap_ix(
        &mut svm_a, &payer_a, mint_x_a, mint_y_a, mint_lp_a, config_a, vault_x_a, vault_y_a,
        treasury_a, treasury_x_a, treasury_y_a, true, mint_x_dummy_amount, 0,
    );
    send(&mut svm_a, &[probe_ix], &payer_a, &[&payer_a]).expect("probe swap should succeed");
    let user_y_after = token_balance(&svm_a, &user_y_a);
    let actual_user_amount = user_y_after - user_y_before;

    // Step 2: fresh, identical pool. Same input amount, but min set to one
    // more than what the user actually received post-fee. The curve's own
    // internal check (against the higher, pre-fee withdraw amount) would
    // pass this min — only our post-fee re-check can catch it.
    let (mut svm_b, payer_b, mint_x_b, mint_y_b, config_b, mint_lp_b, vault_x_b, vault_y_b, treasury_b, treasury_x_b, treasury_y_b) =
        setup_with_liquidity(10);

    let failing_ix = create_swap_ix(
        &mut svm_b, &payer_b, mint_x_b, mint_y_b, mint_lp_b, config_b, vault_x_b, vault_y_b,
        treasury_b, treasury_x_b, treasury_y_b, true, mint_x_dummy_amount, actual_user_amount + 1,
    );
    let res = send(&mut svm_b, &[failing_ix], &payer_b, &[&payer_b]);

    assert!(
        res.is_err(),
        "a min just above what the user actually receives post-fee must fail, \
         even though the curve's own pre-fee check would have passed it"
    );
}