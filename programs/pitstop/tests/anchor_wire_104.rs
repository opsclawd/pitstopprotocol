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
async fn issue_104_anchor_happy_path_bet_lock_resolve_void() {
    let mut ctx = program_test().start_with_context().await;

    // signers/fixtures
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

    let (config_pda, _) = Pubkey::find_program_address(&[b"config"], &pitstop::id());

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

    // create market
    let clock: Clock = ctx.banks_client.get_sysvar().await.unwrap();
    let lock_timestamp = clock.unix_timestamp + 5;

    let event_id = [7u8; 32];
    let market_type = 0u8;
    let rules_version = 1u16;
    let market_id = canonical_market_id(event_id, market_type, rules_version);

    let (market_pda, _) =
        Pubkey::find_program_address(&[b"market", market_id.as_ref()], &pitstop::id());
    let vault_ata = spl_associated_token_account::get_associated_token_address(
        &market_pda,
        &usdc_mint.pubkey(),
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
                lock_timestamp,
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

    // add outcomes
    for outcome_id in [0u8, 1u8] {
        let (pool_pda, _) = Pubkey::find_program_address(
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

    // finalize seeding -> open
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

    // place bet
    let outcome_id = 0u8;
    let amount = 10_000u64;
    let (pool_pda, _) = Pubkey::find_program_address(
        &[b"outcome", market_pda.as_ref(), &[outcome_id]],
        &pitstop::id(),
    );
    let (pos_pda, _) = Pubkey::find_program_address(
        &[b"position", market_pda.as_ref(), user.pubkey().as_ref(), &[outcome_id]],
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
            args: PlaceBetArgs { outcome_id, amount },
        }
        .data(),
    };
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&user.pubkey()),
        &[&user],
        ctx.last_blockhash,
    );
    ctx.banks_client.process_transaction(tx).await.unwrap();

    // assert token movement
    let ua = ctx
        .banks_client
        .get_account(user_usdc.pubkey())
        .await
        .unwrap()
        .unwrap();
    let ustate = spl_token::state::Account::unpack(&ua.data).unwrap();
    assert_eq!(ustate.amount, 500_000 - amount);

    let va = ctx.banks_client.get_account(vault_ata).await.unwrap().unwrap();
    let vstate = spl_token::state::Account::unpack(&va.data).unwrap();
    assert_eq!(vstate.amount, amount);

    // warp until lock time, then lock
    loop {
        let c: Clock = ctx.banks_client.get_sysvar().await.unwrap();
        if c.unix_timestamp >= lock_timestamp {
            break;
        }
        let mut slot = ctx.banks_client.get_root_slot().await.unwrap();
        slot += 10;
        ctx.warp_to_slot(slot).unwrap();
    }

    let ix = solana_sdk::instruction::Instruction {
        program_id: pitstop::id(),
        accounts: pitstop::accounts::LockMarket {
            authority: authority.pubkey(),
            config: config_pda,
            market: market_pda,
        }
        .to_account_metas(None),
        data: pitstop::instruction::LockMarket {}.data(),
    };
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&authority.pubkey()),
        &[&authority],
        ctx.last_blockhash,
    );
    ctx.banks_client.process_transaction(tx).await.unwrap();

    // resolve market (authority is oracle by default)
    let payload_hash = [0xabu8; 32];
    let ix = solana_sdk::instruction::Instruction {
        program_id: pitstop::id(),
        accounts: pitstop::accounts::ResolveMarket {
            oracle: authority.pubkey(),
            config: config_pda,
            market: market_pda,
            winning_outcome_pool: pool_pda,
            system_program: solana_sdk::system_program::id(),
        }
        .to_account_metas(None),
        data: pitstop::instruction::ResolveMarket {
            args: ResolveMarketArgs {
                winning_outcome_id: outcome_id,
                payload_hash,
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

    let acct = ctx
        .banks_client
        .get_account(market_pda)
        .await
        .unwrap()
        .unwrap();
    let m: Market = AccountDeserialize::try_deserialize(&mut acct.data.as_slice()).unwrap();
    assert_eq!(m.status, MarketStatus::Resolved);
    assert_eq!(m.resolved_outcome, Some(outcome_id));
    assert_eq!(m.resolution_payload_hash, payload_hash);

    // create + void a second market (locked -> voided)
    let event_id2 = [9u8; 32];
    let market_id2 = canonical_market_id(event_id2, market_type, rules_version);
    let (market2_pda, _) =
        Pubkey::find_program_address(&[b"market", market_id2.as_ref()], &pitstop::id());
    let vault2_ata = spl_associated_token_account::get_associated_token_address(
        &market2_pda,
        &usdc_mint.pubkey(),
    );
    let clock: Clock = ctx.banks_client.get_sysvar().await.unwrap();
    let lock2 = clock.unix_timestamp + 5;

    let ix = solana_sdk::instruction::Instruction {
        program_id: pitstop::id(),
        accounts: pitstop::accounts::CreateMarket {
            authority: authority.pubkey(),
            config: config_pda,
            market: market2_pda,
            vault: vault2_ata,
            usdc_mint: usdc_mint.pubkey(),
            token_program: spl_token::id(),
            associated_token_program: spl_associated_token_account::id(),
            system_program: solana_sdk::system_program::id(),
        }
        .to_account_metas(None),
        data: pitstop::instruction::CreateMarket {
            args: CreateMarketArgs {
                market_id: market_id2,
                event_id: event_id2,
                lock_timestamp: lock2,
                max_outcomes: 1,
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

    // outcome + open
    let outcome_id2 = 0u8;
    let (pool2_pda, _) = Pubkey::find_program_address(
        &[b"outcome", market2_pda.as_ref(), &[outcome_id2]],
        &pitstop::id(),
    );
    let ix = solana_sdk::instruction::Instruction {
        program_id: pitstop::id(),
        accounts: pitstop::accounts::AddOutcome {
            authority: authority.pubkey(),
            config: config_pda,
            market: market2_pda,
            outcome_pool: pool2_pda,
            system_program: solana_sdk::system_program::id(),
        }
        .to_account_metas(None),
        data: pitstop::instruction::AddOutcome {
            args: AddOutcomeArgs {
                outcome_id: outcome_id2,
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

    let ix = solana_sdk::instruction::Instruction {
        program_id: pitstop::id(),
        accounts: pitstop::accounts::FinalizeSeeding {
            authority: authority.pubkey(),
            config: config_pda,
            market: market2_pda,
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

    // warp + lock + void
    loop {
        let c: Clock = ctx.banks_client.get_sysvar().await.unwrap();
        if c.unix_timestamp >= lock2 {
            break;
        }
        let mut slot = ctx.banks_client.get_root_slot().await.unwrap();
        slot += 10;
        ctx.warp_to_slot(slot).unwrap();
    }

    let ix = solana_sdk::instruction::Instruction {
        program_id: pitstop::id(),
        accounts: pitstop::accounts::LockMarket {
            authority: authority.pubkey(),
            config: config_pda,
            market: market2_pda,
        }
        .to_account_metas(None),
        data: pitstop::instruction::LockMarket {}.data(),
    };
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&authority.pubkey()),
        &[&authority],
        ctx.last_blockhash,
    );
    ctx.banks_client.process_transaction(tx).await.unwrap();

    let payload_hash2 = [7u8; 32];
    let ix = solana_sdk::instruction::Instruction {
        program_id: pitstop::id(),
        accounts: pitstop::accounts::VoidMarket {
            oracle: authority.pubkey(),
            config: config_pda,
            market: market2_pda,
        }
        .to_account_metas(None),
        data: pitstop::instruction::VoidMarket {
            args: VoidMarketArgs {
                payload_hash: payload_hash2,
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

    let acct = ctx
        .banks_client
        .get_account(market2_pda)
        .await
        .unwrap()
        .unwrap();
    let m: Market = AccountDeserialize::try_deserialize(&mut acct.data.as_slice()).unwrap();
    assert_eq!(m.status, MarketStatus::Voided);
    assert_eq!(m.resolved_outcome, None);
    assert_eq!(m.resolution_payload_hash, payload_hash2);
}

#[tokio::test]
async fn issue_104_anchor_place_bet_rejects_outcome_mismatch() {
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

    let (config_pda, _) = Pubkey::find_program_address(&[b"config"], &pitstop::id());

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

    // market open with 1 outcome
    let clock: Clock = ctx.banks_client.get_sysvar().await.unwrap();
    let lock_timestamp = clock.unix_timestamp + 1000;
    let event_id = [7u8; 32];
    let market_type = 0u8;
    let rules_version = 1u16;
    let market_id = canonical_market_id(event_id, market_type, rules_version);

    let (market_pda, _) =
        Pubkey::find_program_address(&[b"market", market_id.as_ref()], &pitstop::id());
    let vault_ata = spl_associated_token_account::get_associated_token_address(
        &market_pda,
        &usdc_mint.pubkey(),
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
                lock_timestamp,
                max_outcomes: 1,
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

    let outcome_id = 0u8;
    let (pool_pda, _) = Pubkey::find_program_address(
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

    // Provide wrong outcome_pool account (use treasury as bogus account)
    let (pos_pda, _) = Pubkey::find_program_address(
        &[b"position", market_pda.as_ref(), user.pubkey().as_ref(), &[outcome_id]],
        &pitstop::id(),
    );

    let ix = solana_sdk::instruction::Instruction {
        program_id: pitstop::id(),
        accounts: pitstop::accounts::PlaceBet {
            user: user.pubkey(),
            config: config_pda,
            market: market_pda,
            outcome_pool: treasury.pubkey(),
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
                amount: 1,
            },
        }
        .data(),
    };

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&user.pubkey()),
        &[&user],
        ctx.last_blockhash,
    );
    let err = ctx.banks_client.process_transaction(tx).await.unwrap_err();
    let msg = format!("{err:?}");
    // OutcomeMismatch is a custom Anchor error; debug formatting may show only the code.
    assert!(
        msg.contains("OutcomeMismatch") || msg.contains("Custom(6027)"),
        "expected OutcomeMismatch (6027), got {msg}"
    );
}
