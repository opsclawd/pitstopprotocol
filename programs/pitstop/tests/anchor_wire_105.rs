use anchor_lang::{prelude::*, AccountDeserialize, InstructionData, ToAccountMetas};
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
    let accounts: &'c [AccountInfo<'c>] = unsafe { std::mem::transmute(accounts) };
    pitstop::entry(program_id, accounts, data)
}

fn program_test() -> ProgramTest {
    ProgramTest::new("pitstop", pitstop::id(), processor!(pitstop_entry))
}

async fn fund(ctx: &mut ProgramTestContext, kp: &Keypair, lamports: u64) {
    let tx = Transaction::new_signed_with_payer(
        &[solana_sdk::system_instruction::transfer(
            &ctx.payer.pubkey(),
            &kp.pubkey(),
            lamports,
        )],
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer],
        ctx.last_blockhash,
    );
    ctx.banks_client.process_transaction(tx).await.unwrap();
}

async fn create_mint(ctx: &mut ProgramTestContext, mint: &Keypair, mint_authority: &Pubkey) {
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

async fn mint_to(
    ctx: &mut ProgramTestContext,
    mint: &Pubkey,
    mint_authority: &Keypair,
    to: &Pubkey,
    amount: u64,
) {
    let ix = spl_token::instruction::mint_to(
        &spl_token::id(),
        mint,
        to,
        &mint_authority.pubkey(),
        &[],
        amount,
    )
    .unwrap();
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer, mint_authority],
        ctx.last_blockhash,
    );
    ctx.banks_client.process_transaction(tx).await.unwrap();
}

fn canonical_market_id(event_id: [u8; 32], market_type: u8, rules_version: u16) -> [u8; 32] {
    use sha2::{Digest, Sha256};
    let mut bytes = [0u8; 35];
    bytes[0..32].copy_from_slice(&event_id);
    bytes[32] = market_type;
    bytes[33..35].copy_from_slice(&rules_version.to_le_bytes());
    let digest = Sha256::digest(bytes);
    let mut out = [0u8; 32];
    out.copy_from_slice(&digest);
    out
}

#[tokio::test]
async fn issue_105_anchor_claim_and_sweep_status_guards() {
    let mut ctx = program_test().start_with_context().await;

    let authority = Keypair::new();
    let treasury_authority = Keypair::new();
    let user = Keypair::new();
    for kp in [&authority, &treasury_authority, &user] {
        fund(&mut ctx, kp, 2_000_000_000).await;
    }

    let usdc_mint = Keypair::new();
    create_mint(&mut ctx, &usdc_mint, &authority.pubkey()).await;

    let treasury = Keypair::new();
    create_token_account(
        &mut ctx,
        &treasury,
        &usdc_mint.pubkey(),
        &treasury_authority.pubkey(),
    )
    .await;

    let user_usdc = Keypair::new();
    create_token_account(&mut ctx, &user_usdc, &usdc_mint.pubkey(), &user.pubkey()).await;
    mint_to(&mut ctx, &usdc_mint.pubkey(), &authority, &user_usdc.pubkey(), 500_000).await;

    let (config_pda, _) = Pubkey::find_program_address(&[CONFIG_SEED], &pitstop::id());

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
                max_bet_per_user_per_market: 500_000,
                claim_window_secs: 3600,
            },
        }
        .data(),
    };
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer, &authority],
        ctx.last_blockhash,
    );
    ctx.banks_client.process_transaction(tx).await.unwrap();

    let event_id = [7u8; 32];
    let market_type = 0u8;
    let rules_version = 1u16;
    let market_id = canonical_market_id(event_id, market_type, rules_version);
    let (market_pda, _) = Pubkey::find_program_address(&[MARKET_SEED, market_id.as_ref()], &pitstop::id());
    let vault_ata = spl_associated_token_account::get_associated_token_address_with_program_id(
        &market_pda,
        &usdc_mint.pubkey(),
        &spl_token::id(),
    );

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
                lock_timestamp: i64::MAX,
                max_outcomes: 2,
                market_type,
                rules_version,
            },
        }
        .data(),
    };
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer, &authority],
        ctx.last_blockhash,
    );
    ctx.banks_client.process_transaction(tx).await.unwrap();

    for outcome_id in [1u8, 2u8] {
        let (pool_pda, _) = Pubkey::find_program_address(
            &[OUTCOME_SEED, market_pda.as_ref(), &[outcome_id]],
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
            Some(&ctx.payer.pubkey()),
            &[&ctx.payer, &authority],
            ctx.last_blockhash,
        );
        ctx.banks_client.process_transaction(tx).await.unwrap();
    }

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
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer, &authority],
        ctx.last_blockhash,
    );
    ctx.banks_client.process_transaction(tx).await.unwrap();

    let outcome_id = 1u8;
    let (pool_pda, _) = Pubkey::find_program_address(
        &[OUTCOME_SEED, market_pda.as_ref(), &[outcome_id]],
        &pitstop::id(),
    );
    let (pos_pda, _) = Pubkey::find_program_address(
        &[POSITION_SEED, market_pda.as_ref(), user.pubkey().as_ref(), &[outcome_id]],
        &pitstop::id(),
    );

    let ix = solana_sdk::instruction::Instruction {
        program_id: pitstop::id(),
        accounts: pitstop::accounts::PlaceBet {
            user: user.pubkey(),
            config: config_pda,
            market: market_pda,
            outcome_pool: pool_pda,
            position: pos_pda,
            user_usdc: user_usdc.pubkey(),
            vault: vault_ata,
            usdc_mint: usdc_mint.pubkey(),
            token_program: spl_token::id(),
            system_program: solana_sdk::system_program::id(),
        }
        .to_account_metas(None),
        data: pitstop::instruction::PlaceBet {
            args: PlaceBetArgs {
                outcome_id,
                amount: 100_000,
            },
        }
        .data(),
    };
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer, &user],
        ctx.last_blockhash,
    );
    ctx.banks_client.process_transaction(tx).await.unwrap();

    // market is still Open; claim_resolved and sweep_remaining must fail via status guards.
    let ix = solana_sdk::instruction::Instruction {
        program_id: pitstop::id(),
        accounts: pitstop::accounts::ClaimResolved {
            user: user.pubkey(),
            config: config_pda,
            market: market_pda,
            position: pos_pda,
            outcome_pool: pool_pda,
            user_usdc: user_usdc.pubkey(),
            vault: vault_ata,
            usdc_mint: usdc_mint.pubkey(),
            token_program: spl_token::id(),
        }
        .to_account_metas(None),
        data: pitstop::instruction::ClaimResolved {
            args: ClaimResolvedArgs { outcome_id },
        }
        .data(),
    };
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer, &user],
        ctx.last_blockhash,
    );
    assert!(ctx.banks_client.process_transaction(tx).await.is_err());

    let ix = solana_sdk::instruction::Instruction {
        program_id: pitstop::id(),
        accounts: pitstop::accounts::SweepRemaining {
            authority: authority.pubkey(),
            config: config_pda,
            market: market_pda,
            vault: vault_ata,
            treasury: treasury.pubkey(),
            close_destination: authority.pubkey(),
            usdc_mint: usdc_mint.pubkey(),
            token_program: spl_token::id(),
        }
        .to_account_metas(None),
        data: pitstop::instruction::SweepRemaining {}.data(),
    };
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer, &authority],
        ctx.last_blockhash,
    );
    assert!(ctx.banks_client.process_transaction(tx).await.is_err());
}

#[tokio::test]
async fn issue_105_anchor_cancel_market_closes_vault_and_sets_voided() {
    let mut ctx = program_test().start_with_context().await;

    let authority = Keypair::new();
    let treasury_authority = Keypair::new();
    for kp in [&authority, &treasury_authority] {
        fund(&mut ctx, kp, 2_000_000_000).await;
    }

    let usdc_mint = Keypair::new();
    create_mint(&mut ctx, &usdc_mint, &authority.pubkey()).await;

    let treasury = Keypair::new();
    create_token_account(
        &mut ctx,
        &treasury,
        &usdc_mint.pubkey(),
        &treasury_authority.pubkey(),
    )
    .await;

    let (config_pda, _) = Pubkey::find_program_address(&[CONFIG_SEED], &pitstop::id());
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
                max_bet_per_user_per_market: 500_000,
                claim_window_secs: 3600,
            },
        }
        .data(),
    };
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer, &authority],
        ctx.last_blockhash,
    );
    ctx.banks_client.process_transaction(tx).await.unwrap();

    let event_id = [8u8; 32];
    let market_type = 0u8;
    let rules_version = 1u16;
    let market_id = canonical_market_id(event_id, market_type, rules_version);
    let (market_pda, _) = Pubkey::find_program_address(&[MARKET_SEED, market_id.as_ref()], &pitstop::id());
    let vault_ata = spl_associated_token_account::get_associated_token_address_with_program_id(
        &market_pda,
        &usdc_mint.pubkey(),
        &spl_token::id(),
    );

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
                lock_timestamp: i64::MAX,
                max_outcomes: 2,
                market_type,
                rules_version,
            },
        }
        .data(),
    };
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer, &authority],
        ctx.last_blockhash,
    );
    ctx.banks_client.process_transaction(tx).await.unwrap();

    let ix = solana_sdk::instruction::Instruction {
        program_id: pitstop::id(),
        accounts: pitstop::accounts::CancelMarket {
            authority: authority.pubkey(),
            config: config_pda,
            market: market_pda,
            vault: vault_ata,
            close_destination: authority.pubkey(),
            token_program: spl_token::id(),
        }
        .to_account_metas(None),
        data: pitstop::instruction::CancelMarket {}.data(),
    };
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer, &authority],
        ctx.last_blockhash,
    );
    ctx.banks_client.process_transaction(tx).await.unwrap();

    let market_acct = ctx
        .banks_client
        .get_account(market_pda)
        .await
        .unwrap()
        .unwrap();
    let market: Market = AccountDeserialize::try_deserialize(&mut market_acct.data.as_slice()).unwrap();
    assert_eq!(market.status, MarketStatus::Voided);

    let vault_after = ctx.banks_client.get_account(vault_ata).await.unwrap();
    assert!(vault_after.is_none());
}
