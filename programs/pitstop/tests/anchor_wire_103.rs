use anchor_lang::{prelude::*, InstructionData, ToAccountMetas};
use solana_program_test::*;
use solana_sdk::{
    entrypoint::ProgramResult,
    program_pack::Pack,
    signature::{Keypair, Signer},
    transaction::Transaction,
};

use pitstop::{self, anchor_accounts::*};

fn pitstop_entry<'a, 'b, 'c, 'd>(
    program_id: &'a Pubkey,
    accounts: &'b [AccountInfo<'c>],
    data: &'d [u8],
) -> ProgramResult {
    // `anchor_lang`'s generated `entry` expects the slice lifetime to match the
    // inner AccountInfo lifetime. `solana-program-test` passes them as the same
    // lifetime in practice, but its processor signature is more general.
    //
    // This shim uses an unsafe lifetime coercion for test-only execution.
    let accounts: &'c [AccountInfo<'c>] = unsafe { std::mem::transmute(accounts) };
    pitstop::entry(program_id, accounts, data)
}

fn program_test() -> ProgramTest {
    ProgramTest::new("pitstop", pitstop::id(), processor!(pitstop_entry))
}

async fn create_mint(
    ctx: &mut ProgramTestContext,
    mint: &Keypair,
    mint_authority: &Pubkey,
) {
    let rent = ctx.banks_client.get_rent().await.unwrap();
    let space = spl_token::state::Mint::LEN;
    let lamports = rent.minimum_balance(space);

    let create = solana_sdk::system_instruction::create_account(
        &ctx.payer.pubkey(),
        &mint.pubkey(),
        lamports,
        space as u64,
        &spl_token::id(),
    );
    let init = spl_token::instruction::initialize_mint(
        &spl_token::id(),
        &mint.pubkey(),
        mint_authority,
        None,
        6,
    )
    .unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[create, init],
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer, mint],
        ctx.last_blockhash,
    );
    ctx.banks_client.process_transaction(tx).await.unwrap();
}

async fn create_token_account(
    ctx: &mut ProgramTestContext,
    acct: &Keypair,
    mint: &Pubkey,
    owner: &Pubkey,
) {
    let rent = ctx.banks_client.get_rent().await.unwrap();
    let space = spl_token::state::Account::LEN;
    let lamports = rent.minimum_balance(space);

    let create = solana_sdk::system_instruction::create_account(
        &ctx.payer.pubkey(),
        &acct.pubkey(),
        lamports,
        space as u64,
        &spl_token::id(),
    );
    let init = spl_token::instruction::initialize_account(
        &spl_token::id(),
        &acct.pubkey(),
        mint,
        owner,
    )
    .unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[create, init],
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer, acct],
        ctx.last_blockhash,
    );
    ctx.banks_client.process_transaction(tx).await.unwrap();
}

#[tokio::test]
async fn issue_103_anchor_happy_path_initialize_create_market_add_outcome_finalize() {
    let mut pt = program_test();
    let mut ctx = pt.start_with_context().await;

    // Fixtures.
    let authority = Keypair::new();
    let treasury_authority = Keypair::new();

    // Fund signers.
    for kp in [&authority, &treasury_authority] {
        let tx = Transaction::new_signed_with_payer(
            &[solana_sdk::system_instruction::transfer(
                &ctx.payer.pubkey(),
                &kp.pubkey(),
                2_000_000_000,
            )],
            Some(&ctx.payer.pubkey()),
            &[&ctx.payer],
            ctx.last_blockhash,
        );
        ctx.banks_client.process_transaction(tx).await.unwrap();
    }

    let usdc_mint = Keypair::new();
    create_mint(&mut ctx, &usdc_mint, &authority.pubkey()).await;

    let treasury = Keypair::new();
    create_token_account(&mut ctx, &treasury, &usdc_mint.pubkey(), &treasury_authority.pubkey())
        .await;

    let (config_pda, _config_bump) = Pubkey::find_program_address(&[b"config"], &pitstop::id());

    // initialize
    let ix = solana_sdk::instruction::Instruction {
        program_id: pitstop::id(),
        accounts: pitstop::accounts::Initialize {
            authority: authority.pubkey(),
            config: config_pda,
            usdc_mint: usdc_mint.pubkey(),
            treasury: treasury.pubkey(),
            token_program: spl_token::id(),
            system_program: solana_sdk::system_program::id(),
        }
        .to_account_metas(None),
        data: pitstop::instruction::Initialize {
            args: InitializeArgs {
                treasury_authority: treasury_authority.pubkey(),
                max_total_pool_per_market: 1_000_000,
                max_bet_per_user_per_market: 100_000,
                claim_window_secs: 3600,
            },
        }
        .data(),
    };

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&authority.pubkey()),
        &[&authority],
        ctx.last_blockhash,
    );
    ctx.banks_client.process_transaction(tx).await.unwrap();

    // create_market
    let event_id = [7u8; 32];
    let market_type = 0u8;
    let rules_version = 1u16;
    let market_id = {
        use sha2::{Digest, Sha256};
        let mut bytes = [0u8; 35];
        bytes[0..32].copy_from_slice(&event_id);
        bytes[32] = market_type;
        bytes[33..35].copy_from_slice(&rules_version.to_le_bytes());
        let digest = Sha256::digest(bytes);
        let mut out = [0u8; 32];
        out.copy_from_slice(&digest);
        out
    };

    let (market_pda, _market_bump) =
        Pubkey::find_program_address(&[b"market", market_id.as_ref()], &pitstop::id());
    let vault_ata = spl_associated_token_account::get_associated_token_address(&market_pda, &usdc_mint.pubkey());

    let ix = solana_sdk::instruction::Instruction {
        program_id: pitstop::id(),
        accounts: pitstop::accounts::CreateMarket {
            authority: authority.pubkey(),
            config: config_pda,
            market: market_pda,
            vault: vault_ata,
            usdc_mint: usdc_mint.pubkey(),
            token_program: spl_token::id(),
            associated_token_program: spl_associated_token_account::id(),
            system_program: solana_sdk::system_program::id(),
        }
        .to_account_metas(None),
        data: pitstop::instruction::CreateMarket {
            args: CreateMarketArgs {
                market_id,
                event_id,
                lock_timestamp: 1_900_000_000,
                max_outcomes: 2,
                market_type,
                rules_version,
            },
        }
        .data(),
    };

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&authority.pubkey()),
        &[&authority],
        ctx.last_blockhash,
    );
    ctx.banks_client.process_transaction(tx).await.unwrap();

    // add_outcome twice
    for outcome_id in [0u8, 1u8] {
        let (pool_pda, _pool_bump) = Pubkey::find_program_address(
            &[b"outcome", market_pda.as_ref(), &[outcome_id]],
            &pitstop::id(),
        );
        let ix = solana_sdk::instruction::Instruction {
            program_id: pitstop::id(),
            accounts: pitstop::accounts::AddOutcome {
                authority: authority.pubkey(),
                config: config_pda,
                market: market_pda,
                outcome_pool: pool_pda,
                system_program: solana_sdk::system_program::id(),
            }
            .to_account_metas(None),
            data: pitstop::instruction::AddOutcome {
                args: AddOutcomeArgs { outcome_id },
            }
            .data(),
        };
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&authority.pubkey()),
            &[&authority],
            ctx.last_blockhash,
        );
        ctx.banks_client.process_transaction(tx).await.unwrap();
    }

    // finalize_seeding
    let ix = solana_sdk::instruction::Instruction {
        program_id: pitstop::id(),
        accounts: pitstop::accounts::FinalizeSeeding {
            authority: authority.pubkey(),
            config: config_pda,
            market: market_pda,
        }
        .to_account_metas(None),
        data: pitstop::instruction::FinalizeSeeding {}.data(),
    };

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&authority.pubkey()),
        &[&authority],
        ctx.last_blockhash,
    );
    ctx.banks_client.process_transaction(tx).await.unwrap();

    // Fetch and assert market status is Open.
    let acct = ctx
        .banks_client
        .get_account(market_pda)
        .await
        .unwrap()
        .expect("market exists");
    let market: Market = AccountDeserialize::try_deserialize(&mut acct.data.as_slice()).unwrap();
    assert_eq!(market.status, MarketStatus::Open);
    assert_eq!(market.outcome_count, 2);
    assert_eq!(market.max_outcomes, 2);
}

#[tokio::test]
async fn issue_103_anchor_create_market_rejects_wrong_authority() {
    let mut ctx = program_test().start_with_context().await;

    let authority = Keypair::new();
    let treasury_authority = Keypair::new();
    for kp in [&authority, &treasury_authority] {
        let tx = Transaction::new_signed_with_payer(
            &[solana_sdk::system_instruction::transfer(
                &ctx.payer.pubkey(),
                &kp.pubkey(),
                2_000_000_000,
            )],
            Some(&ctx.payer.pubkey()),
            &[&ctx.payer],
            ctx.last_blockhash,
        );
        ctx.banks_client.process_transaction(tx).await.unwrap();
    }

    let usdc_mint = Keypair::new();
    create_mint(&mut ctx, &usdc_mint, &authority.pubkey()).await;

    let treasury = Keypair::new();
    create_token_account(&mut ctx, &treasury, &usdc_mint.pubkey(), &treasury_authority.pubkey())
        .await;

    let (config_pda, _config_bump) = Pubkey::find_program_address(&[b"config"], &pitstop::id());

    // initialize with authority as config authority.
    let ix = solana_sdk::instruction::Instruction {
        program_id: pitstop::id(),
        accounts: pitstop::accounts::Initialize {
            authority: authority.pubkey(),
            config: config_pda,
            usdc_mint: usdc_mint.pubkey(),
            treasury: treasury.pubkey(),
            token_program: spl_token::id(),
            system_program: solana_sdk::system_program::id(),
        }
        .to_account_metas(None),
        data: pitstop::instruction::Initialize {
            args: InitializeArgs {
                treasury_authority: treasury_authority.pubkey(),
                max_total_pool_per_market: 1_000_000,
                max_bet_per_user_per_market: 100_000,
                claim_window_secs: 3600,
            },
        }
        .data(),
    };

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&authority.pubkey()),
        &[&authority],
        ctx.last_blockhash,
    );
    ctx.banks_client.process_transaction(tx).await.unwrap();

    // attempt create_market by a different signer.
    let other = Keypair::new();
    let tx = Transaction::new_signed_with_payer(
        &[solana_sdk::system_instruction::transfer(
            &ctx.payer.pubkey(),
            &other.pubkey(),
            2_000_000_000,
        )],
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer],
        ctx.last_blockhash,
    );
    ctx.banks_client.process_transaction(tx).await.unwrap();

    let event_id = [7u8; 32];
    let market_type = 0u8;
    let rules_version = 1u16;
    let market_id = {
        use sha2::{Digest, Sha256};
        let mut bytes = [0u8; 35];
        bytes[0..32].copy_from_slice(&event_id);
        bytes[32] = market_type;
        bytes[33..35].copy_from_slice(&rules_version.to_le_bytes());
        let digest = Sha256::digest(bytes);
        let mut out = [0u8; 32];
        out.copy_from_slice(&digest);
        out
    };

    let (market_pda, _market_bump) =
        Pubkey::find_program_address(&[b"market", market_id.as_ref()], &pitstop::id());
    let vault_ata = spl_associated_token_account::get_associated_token_address(&market_pda, &usdc_mint.pubkey());

    let ix = solana_sdk::instruction::Instruction {
        program_id: pitstop::id(),
        accounts: pitstop::accounts::CreateMarket {
            authority: other.pubkey(),
            config: config_pda,
            market: market_pda,
            vault: vault_ata,
            usdc_mint: usdc_mint.pubkey(),
            token_program: spl_token::id(),
            associated_token_program: spl_associated_token_account::id(),
            system_program: solana_sdk::system_program::id(),
        }
        .to_account_metas(None),
        data: pitstop::instruction::CreateMarket {
            args: CreateMarketArgs {
                market_id,
                event_id,
                lock_timestamp: 1_900_000_000,
                max_outcomes: 2,
                market_type,
                rules_version,
            },
        }
        .data(),
    };

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&other.pubkey()),
        &[&other],
        ctx.last_blockhash,
    );
    let err = ctx.banks_client.process_transaction(tx).await.unwrap_err();

    // Custom Anchor error codes start at 6000 by default.
    let msg = format!("{err:?}");
    assert!(msg.contains("Custom(6000)"), "expected Unauthorized (6000), got {msg}");
}
