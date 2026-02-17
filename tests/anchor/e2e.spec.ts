import * as anchor from "@coral-xyz/anchor";
import { assert } from "chai";
import {
  createMint,
  getOrCreateAssociatedTokenAccount,
  mintTo,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import { Keypair, PublicKey, SystemProgram } from "@solana/web3.js";

function bytes32FromString(s: string): number[] {
  const b = Buffer.alloc(32);
  Buffer.from(s).copy(b, 0);
  return [...b];
}

async function airdrop(connection: anchor.web3.Connection, pk: PublicKey, sol = 2) {
  const sig = await connection.requestAirdrop(pk, sol * anchor.web3.LAMPORTS_PER_SOL);
  await connection.confirmTransaction(sig, "confirmed");
}

async function getNow(connection: anchor.web3.Connection): Promise<number> {
  const slot = await connection.getSlot("confirmed");
  const bt = await connection.getBlockTime(slot);
  if (bt === null) throw new Error("getBlockTime returned null");
  return bt;
}

async function warpForwardSlots(connection: any, slots: number) {
  const current = await connection.getSlot("confirmed");
  const target = current + slots;
  // solana-test-validator custom rpc
  await connection._rpcRequest("warp_slot", [target]);
}

async function warpToAtLeastTimestamp(
  connection: any,
  targetUnixSecs: number,
  maxIterations = 20,
) {
  for (let i = 0; i < maxIterations; i++) {
    const now = await getNow(connection);
    if (now >= targetUnixSecs) return;
    // Advance a chunk of slots; local validator uses ~400ms/slot.
    await warpForwardSlots(connection, 400);
  }
  const now = await getNow(connection);
  throw new Error(`Failed to warp to ts>=${targetUnixSecs}; now=${now}`);
}

function findConfigPda(programId: PublicKey): [PublicKey, number] {
  return PublicKey.findProgramAddressSync([Buffer.from("config")], programId);
}

function findMarketPda(programId: PublicKey, marketId: number[]): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("market"), Buffer.from(marketId)],
    programId,
  );
}

function findOutcomePoolPda(
  programId: PublicKey,
  market: PublicKey,
  outcomeId: number,
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("outcome"), market.toBuffer(), Buffer.from([outcomeId])],
    programId,
  );
}

function findPositionPda(
  programId: PublicKey,
  market: PublicKey,
  user: PublicKey,
  outcomeId: number,
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("position"), market.toBuffer(), user.toBuffer(), Buffer.from([outcomeId])],
    programId,
  );
}

function expectAnchorErrCode(e: any, code: string) {
  // anchor@0.30 error shapes vary across RPC/TS versions; handle the common ones.
  const errCode =
    e?.error?.errorCode?.code ||
    e?.error?.errorCode?.number ||
    e?.error?.code ||
    e?.code ||
    (typeof e?.toString === "function" ? e.toString() : "");

  if (typeof errCode === "string" && errCode.includes(code)) return;

  if (e instanceof anchor.AnchorError) {
    assert.equal(e.error.errorCode.code, code);
    return;
  }

  // Parse logs if present.
  const parsed = anchor.AnchorError.parse(e?.logs ?? []);
  if (parsed) {
    assert.equal(parsed.error.errorCode.code, code);
    return;
  }

  assert.fail(`Expected Anchor error code=${code}, got=${JSON.stringify(errCode)}\n${e}`);
}

describe("anchor-e2e", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Pitstop as anchor.Program<any>;

  it("create→seed→open→bet→lock→resolve→claim→sweep (+ deterministic negative cases)", async () => {
    const connection: any = provider.connection;
    const payer = (provider.wallet as any).payer as Keypair;

    // Extra actors.
    const treasuryAuthority = Keypair.generate();
    const userA = Keypair.generate();
    const userB = Keypair.generate();

    await Promise.all([
      airdrop(connection, treasuryAuthority.publicKey),
      airdrop(connection, userA.publicKey),
      airdrop(connection, userB.publicKey),
    ]);

    // USDC mint + token accounts.
    const usdcMint = await createMint(
      connection,
      payer,
      payer.publicKey,
      null,
      6,
      undefined,
      undefined,
      TOKEN_PROGRAM_ID,
    );

    const treasuryAta = await getOrCreateAssociatedTokenAccount(
      connection,
      payer,
      usdcMint,
      treasuryAuthority.publicKey,
      true,
      "confirmed",
      undefined,
      TOKEN_PROGRAM_ID,
    );

    const userAAta = await getOrCreateAssociatedTokenAccount(
      connection,
      payer,
      usdcMint,
      userA.publicKey,
      true,
      "confirmed",
      undefined,
      TOKEN_PROGRAM_ID,
    );
    const userBAta = await getOrCreateAssociatedTokenAccount(
      connection,
      payer,
      usdcMint,
      userB.publicKey,
      true,
      "confirmed",
      undefined,
      TOKEN_PROGRAM_ID,
    );

    await mintTo(
      connection,
      payer,
      usdcMint,
      userAAta.address,
      payer.publicKey,
      1_000_000_000n,
      [],
      undefined,
      TOKEN_PROGRAM_ID,
    );
    await mintTo(
      connection,
      payer,
      usdcMint,
      userBAta.address,
      payer.publicKey,
      1_000_000_000n,
      [],
      undefined,
      TOKEN_PROGRAM_ID,
    );

    // initialize
    const [config] = findConfigPda(program.programId);
    await program.methods
      .initialize({
        treasuryAuthority: treasuryAuthority.publicKey,
        maxTotalPoolPerMarket: new anchor.BN(10_000_000_000),
        maxBetPerUserPerMarket: new anchor.BN(5_000_000_000),
        claimWindowSecs: new anchor.BN(2),
      })
      .accounts({
        authority: payer.publicKey,
        config,
        usdcMint,
        treasury: treasuryAta.address,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .rpc({ commitment: "confirmed" });

    // create_market (canonical market_id is checked on-chain, so use event_id/mkt_type/rules_version that match).
    const now0 = await getNow(connection);
    const lockTimestamp = now0 + 3;

    const eventId = bytes32FromString("event-1");

    // MarketId must be sha256(event_id||market_type||rules_version LE) per SPEC_CANONICAL.
    const preimage = Buffer.concat([
      Buffer.from(eventId),
      Buffer.from([0]),
      Buffer.from(Uint16Array.from([1]).buffer),
    ]);
    const marketId = [...anchor.utils.sha256.hash(preimage)];

    const [market] = findMarketPda(program.programId, marketId);
    const vault = await anchor.utils.token.associatedAddress({
      mint: usdcMint,
      owner: market,
    });

    await program.methods
      .createMarket({
        marketId,
        eventId,
        lockTimestamp: new anchor.BN(lockTimestamp),
        maxOutcomes: 2,
        marketType: 0,
        rulesVersion: 1,
      })
      .accounts({
        authority: payer.publicKey,
        config,
        market,
        vault,
        usdcMint,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: anchor.utils.token.ASSOCIATED_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .rpc({ commitment: "confirmed" });

    // add_outcome (0,1)
    const [pool0] = findOutcomePoolPda(program.programId, market, 0);
    const [pool1] = findOutcomePoolPda(program.programId, market, 1);

    await program.methods
      .addOutcome({ outcomeId: 0 })
      .accounts({
        authority: payer.publicKey,
        config,
        market,
        outcomePool: pool0,
        systemProgram: SystemProgram.programId,
      })
      .rpc({ commitment: "confirmed" });

    await program.methods
      .addOutcome({ outcomeId: 1 })
      .accounts({
        authority: payer.publicKey,
        config,
        market,
        outcomePool: pool1,
        systemProgram: SystemProgram.programId,
      })
      .rpc({ commitment: "confirmed" });

    // finalize_seeding -> Open
    await program.methods
      .finalizeSeeding()
      .accounts({ authority: payer.publicKey, config, market })
      .rpc({ commitment: "confirmed" });

    // place_bet
    const [posA0] = findPositionPda(program.programId, market, userA.publicKey, 0);
    const [posB1] = findPositionPda(program.programId, market, userB.publicKey, 1);

    await program.methods
      .placeBet({ outcomeId: 0, amount: new anchor.BN(1_000_000) })
      .accounts({
        user: userA.publicKey,
        config,
        market,
        outcomePool: pool0,
        position: posA0,
        userUsdc: userAAta.address,
        vault,
        usdcMint,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([userA])
      .rpc({ commitment: "confirmed" });

    await program.methods
      .placeBet({ outcomeId: 1, amount: new anchor.BN(2_000_000) })
      .accounts({
        user: userB.publicKey,
        config,
        market,
        outcomePool: pool1,
        position: posB1,
        userUsdc: userBAta.address,
        vault,
        usdcMint,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([userB])
      .rpc({ commitment: "confirmed" });

    // Deterministic negative: lock too early.
    try {
      await program.methods
        .lockMarket()
        .accounts({ authority: payer.publicKey, config, market })
        .rpc({ commitment: "confirmed" });
      assert.fail("Expected TooEarlyToLock");
    } catch (e) {
      expectAnchorErrCode(e, "TooEarlyToLock");
    }

    // Warp beyond lock timestamp then lock.
    await warpToAtLeastTimestamp(connection, lockTimestamp);
    await program.methods
      .lockMarket()
      .accounts({ authority: payer.publicKey, config, market })
      .rpc({ commitment: "confirmed" });

    // Deterministic negative: OutcomeMismatch by supplying wrong pool account.
    const [posA1] = findPositionPda(program.programId, market, userA.publicKey, 1);
    try {
      await program.methods
        .placeBet({ outcomeId: 1, amount: new anchor.BN(1) })
        .accounts({
          user: userA.publicKey,
          config,
          market,
          // wrong pool: outcomeId=1 but pool0 passed
          outcomePool: pool0,
          position: posA1,
          userUsdc: userAAta.address,
          vault,
          usdcMint,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([userA])
        .rpc({ commitment: "confirmed" });
      assert.fail("Expected OutcomeMismatch");
    } catch (e) {
      expectAnchorErrCode(e, "OutcomeMismatch");
    }

    // resolve_market: outcome 0 wins
    const payloadHash = bytes32FromString("payload-1");
    await program.methods
      .resolveMarket({ winningOutcomeId: 0, payloadHash })
      .accounts({
        oracle: payer.publicKey,
        config,
        market,
        winningOutcomePool: pool0,
        systemProgram: SystemProgram.programId,
      })
      .rpc({ commitment: "confirmed" });

    // claim_resolved for winner
    await program.methods
      .claimResolved({ outcomeId: 0 })
      .accounts({
        user: userA.publicKey,
        config,
        market,
        position: posA0,
        outcomePool: pool0,
        userUsdc: userAAta.address,
        vault,
        usdcMint,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([userA])
      .rpc({ commitment: "confirmed" });

    // sweep_remaining before claim window expires should fail.
    try {
      await program.methods
        .sweepRemaining()
        .accounts({
          authority: payer.publicKey,
          config,
          market,
          vault,
          treasury: treasuryAta.address,
          closeDestination: payer.publicKey,
          usdcMint,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .rpc({ commitment: "confirmed" });
      assert.fail("Expected ClaimWindowNotExpired");
    } catch (e) {
      expectAnchorErrCode(e, "ClaimWindowNotExpired");
    }

    // Warp beyond claim window then sweep.
    const afterResolution = await getNow(connection);
    await warpToAtLeastTimestamp(connection, afterResolution + 3);
    await program.methods
      .sweepRemaining()
      .accounts({
        authority: payer.publicKey,
        config,
        market,
        vault,
        treasury: treasuryAta.address,
        closeDestination: payer.publicKey,
        usdcMint,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc({ commitment: "confirmed" });

    const marketAcct = await program.account.market.fetch(market);
    assert.equal(marketAcct.status.swept !== undefined, true);
  });

  it("create→seed→open→bet→lock→void→claim_voided→sweep", async () => {
    const connection: any = provider.connection;
    const payer = (provider.wallet as any).payer as Keypair;

    const treasuryAuthority = Keypair.generate();
    const user = Keypair.generate();

    await Promise.all([
      airdrop(connection, treasuryAuthority.publicKey),
      airdrop(connection, user.publicKey),
    ]);

    const usdcMint = await createMint(
      connection,
      payer,
      payer.publicKey,
      null,
      6,
      undefined,
      undefined,
      TOKEN_PROGRAM_ID,
    );

    const treasuryAta = await getOrCreateAssociatedTokenAccount(
      connection,
      payer,
      usdcMint,
      treasuryAuthority.publicKey,
      true,
      "confirmed",
      undefined,
      TOKEN_PROGRAM_ID,
    );

    const userAta = await getOrCreateAssociatedTokenAccount(
      connection,
      payer,
      usdcMint,
      user.publicKey,
      true,
      "confirmed",
      undefined,
      TOKEN_PROGRAM_ID,
    );

    await mintTo(
      connection,
      payer,
      usdcMint,
      userAta.address,
      payer.publicKey,
      10_000_000n,
      [],
      undefined,
      TOKEN_PROGRAM_ID,
    );

    const [config] = findConfigPda(program.programId);

    // Idempotent init: if already exists from prior test, skip.
    try {
      await program.account.config.fetch(config);
    } catch {
      await program.methods
        .initialize({
          treasuryAuthority: treasuryAuthority.publicKey,
          maxTotalPoolPerMarket: new anchor.BN(10_000_000_000),
          maxBetPerUserPerMarket: new anchor.BN(5_000_000_000),
          claimWindowSecs: new anchor.BN(2),
        })
        .accounts({
          authority: payer.publicKey,
          config,
          usdcMint,
          treasury: treasuryAta.address,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .rpc({ commitment: "confirmed" });
    }

    const now0 = await getNow(connection);
    const lockTimestamp = now0 + 3;
    const eventId = bytes32FromString("event-void");
    const preimage = Buffer.concat([
      Buffer.from(eventId),
      Buffer.from([0]),
      Buffer.from(Uint16Array.from([1]).buffer),
    ]);
    const marketId = [...anchor.utils.sha256.hash(preimage)];

    const [market] = findMarketPda(program.programId, marketId);
    const vault = await anchor.utils.token.associatedAddress({ mint: usdcMint, owner: market });

    await program.methods
      .createMarket({
        marketId,
        eventId,
        lockTimestamp: new anchor.BN(lockTimestamp),
        maxOutcomes: 1,
        marketType: 0,
        rulesVersion: 1,
      })
      .accounts({
        authority: payer.publicKey,
        config,
        market,
        vault,
        usdcMint,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: anchor.utils.token.ASSOCIATED_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .rpc({ commitment: "confirmed" });

    const [pool0] = findOutcomePoolPda(program.programId, market, 0);
    await program.methods
      .addOutcome({ outcomeId: 0 })
      .accounts({ authority: payer.publicKey, config, market, outcomePool: pool0, systemProgram: SystemProgram.programId })
      .rpc({ commitment: "confirmed" });

    await program.methods
      .finalizeSeeding()
      .accounts({ authority: payer.publicKey, config, market })
      .rpc({ commitment: "confirmed" });

    const [pos0] = findPositionPda(program.programId, market, user.publicKey, 0);
    await program.methods
      .placeBet({ outcomeId: 0, amount: new anchor.BN(1_000_000) })
      .accounts({
        user: user.publicKey,
        config,
        market,
        outcomePool: pool0,
        position: pos0,
        userUsdc: userAta.address,
        vault,
        usdcMint,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([user])
      .rpc({ commitment: "confirmed" });

    await warpToAtLeastTimestamp(connection, lockTimestamp);
    await program.methods.lockMarket().accounts({ authority: payer.publicKey, config, market }).rpc({ commitment: "confirmed" });

    const payloadHash = bytes32FromString("payload-void");
    await program.methods
      .voidMarket({ payloadHash })
      .accounts({ oracle: payer.publicKey, config, market })
      .rpc({ commitment: "confirmed" });

    await program.methods
      .claimVoided({ outcomeId: 0 })
      .accounts({
        user: user.publicKey,
        config,
        market,
        position: pos0,
        userUsdc: userAta.address,
        vault,
        usdcMint,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([user])
      .rpc({ commitment: "confirmed" });

    const afterVoid = await getNow(connection);
    await warpToAtLeastTimestamp(connection, afterVoid + 3);
    await program.methods
      .sweepRemaining()
      .accounts({
        authority: payer.publicKey,
        config,
        market,
        vault,
        treasury: treasuryAta.address,
        closeDestination: payer.publicKey,
        usdcMint,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc({ commitment: "confirmed" });

    const m = await program.account.market.fetch(market);
    assert.equal(m.status.swept !== undefined, true);
  });

  it("create→cancel (seeding-only, empty vault)", async () => {
    const connection: any = provider.connection;
    const payer = (provider.wallet as any).payer as Keypair;

    const usdcMint = await createMint(connection, payer, payer.publicKey, null, 6, undefined, undefined, TOKEN_PROGRAM_ID);

    const treasuryAuthority = Keypair.generate();
    await airdrop(connection, treasuryAuthority.publicKey);

    const treasuryAta = await getOrCreateAssociatedTokenAccount(
      connection,
      payer,
      usdcMint,
      treasuryAuthority.publicKey,
      true,
      "confirmed",
      undefined,
      TOKEN_PROGRAM_ID,
    );

    const [config] = findConfigPda(program.programId);
    try {
      await program.account.config.fetch(config);
    } catch {
      await program.methods
        .initialize({
          treasuryAuthority: treasuryAuthority.publicKey,
          maxTotalPoolPerMarket: new anchor.BN(10_000_000_000),
          maxBetPerUserPerMarket: new anchor.BN(5_000_000_000),
          claimWindowSecs: new anchor.BN(2),
        })
        .accounts({
          authority: payer.publicKey,
          config,
          usdcMint,
          treasury: treasuryAta.address,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .rpc({ commitment: "confirmed" });
    }

    const now0 = await getNow(connection);
    const lockTimestamp = now0 + 30;
    const eventId = bytes32FromString("event-cancel");
    const preimage = Buffer.concat([
      Buffer.from(eventId),
      Buffer.from([0]),
      Buffer.from(Uint16Array.from([1]).buffer),
    ]);
    const marketId = [...anchor.utils.sha256.hash(preimage)];

    const [market] = findMarketPda(program.programId, marketId);
    const vault = await anchor.utils.token.associatedAddress({ mint: usdcMint, owner: market });

    await program.methods
      .createMarket({
        marketId,
        eventId,
        lockTimestamp: new anchor.BN(lockTimestamp),
        maxOutcomes: 1,
        marketType: 0,
        rulesVersion: 1,
      })
      .accounts({
        authority: payer.publicKey,
        config,
        market,
        vault,
        usdcMint,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: anchor.utils.token.ASSOCIATED_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .rpc({ commitment: "confirmed" });

    await program.methods
      .cancelMarket()
      .accounts({
        authority: payer.publicKey,
        config,
        market,
        vault,
        closeDestination: payer.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc({ commitment: "confirmed" });

    const m = await program.account.market.fetch(market);
    assert.equal(m.status.voided !== undefined, true);
  });
});
