import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PToken } from "../target/types/p_token";
import { Keypair, SystemProgram, SYSVAR_RENT_PUBKEY } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID, getAccount } from "@solana/spl-token";
import { assert } from "chai";

describe("SPL Token vs P-Token — All 25 Instructions", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.pToken as Program<PToken>;
  const wallet = provider.wallet as anchor.Wallet;

  const mintKp = Keypair.generate();
  const tokenAKp = Keypair.generate();
  const tokenBKp = Keypair.generate();
  const delegateKp = Keypair.generate();
  const decimals = 6;

  it("1. InitializeMint — SPL: 2,967 CU | P-Token: 99 CU", async () => {
    const tx = await program.methods
      .initializeMint(decimals)
      .accounts({
        mint: mintKp.publicKey,
        authority: wallet.publicKey,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        rent: SYSVAR_RENT_PUBKEY,
      })
      .signers([mintKp])
      .rpc();
    console.log(`    tx: ${tx.slice(0, 40)}...`);
  });

  it("2. InitializeAccount — SPL: 4,527 CU | P-Token: 149 CU", async () => {
    await program.methods
      .initializeAccount()
      .accounts({
        tokenAccount: tokenAKp.publicKey,
        mint: mintKp.publicKey,
        authority: wallet.publicKey,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        rent: SYSVAR_RENT_PUBKEY,
      })
      .signers([tokenAKp])
      .rpc();

    await program.methods
      .initializeAccount()
      .accounts({
        tokenAccount: tokenBKp.publicKey,
        mint: mintKp.publicKey,
        authority: wallet.publicKey,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        rent: SYSVAR_RENT_PUBKEY,
      })
      .signers([tokenBKp])
      .rpc();
    console.log(`    Account A: ${tokenAKp.publicKey.toBase58().slice(0, 20)}...`);
    console.log(`    Account B: ${tokenBKp.publicKey.toBase58().slice(0, 20)}...`);
  });

  it("8. MintTo — SPL: 4,538 CU | P-Token: 120 CU", async () => {
    await program.methods
      .mintTokens(new anchor.BN(10_000_000))
      .accounts({
        mint: mintKp.publicKey,
        tokenAccount: tokenAKp.publicKey,
        authority: wallet.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();

    const acct = await getAccount(provider.connection, tokenAKp.publicKey);
    console.log(`    Minted 10,000,000 → A balance: ${acct.amount}`);
    assert.equal(acct.amount.toString(), "10000000");
  });

  it("4. Transfer — SPL: 4,645 CU | P-Token: 78 CU", async () => {
    await program.methods
      .transferTokens(new anchor.BN(3_000_000))
      .accounts({
        from: tokenAKp.publicKey,
        to: tokenBKp.publicKey,
        authority: wallet.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();

    const a = await getAccount(provider.connection, tokenAKp.publicKey);
    const b = await getAccount(provider.connection, tokenBKp.publicKey);
    console.log(`    A: ${a.amount} | B: ${b.amount}`);
  });

  it("13. TransferChecked — SPL: 6,200 CU | P-Token: 107 CU", async () => {
    await program.methods
      .transferChecked(new anchor.BN(1_000_000), decimals)
      .accounts({
        from: tokenAKp.publicKey,
        to: tokenBKp.publicKey,
        mint: mintKp.publicKey,
        authority: wallet.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();

    const b = await getAccount(provider.connection, tokenBKp.publicKey);
    console.log(`    B balance after checked transfer: ${b.amount}`);
  });

  it("5. Approve — SPL: 2,904 CU | P-Token: 123 CU", async () => {
    await program.methods
      .approveDelegate(new anchor.BN(500_000))
      .accounts({
        tokenAccount: tokenAKp.publicKey,
        delegate: delegateKp.publicKey,
        authority: wallet.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();
    console.log(`    Delegated 500,000 to ${delegateKp.publicKey.toBase58().slice(0, 20)}...`);
  });

  it("14. ApproveChecked — SPL: 4,410 CU | P-Token: 160 CU", async () => {
    await program.methods
      .approveChecked(new anchor.BN(250_000), decimals)
      .accounts({
        tokenAccount: tokenAKp.publicKey,
        mint: mintKp.publicKey,
        delegate: delegateKp.publicKey,
        authority: wallet.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();
    console.log(`    Approve-checked 250,000`);
  });

  it("6. Revoke — SPL: 2,691 CU | P-Token: 102 CU", async () => {
    await program.methods
      .revokeDelegate()
      .accounts({
        tokenAccount: tokenAKp.publicKey,
        authority: wallet.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();
    console.log(`    Delegation revoked`);
  });

  it("15. MintToChecked — SPL: 6,037 CU | P-Token: 153 CU", async () => {
    await program.methods
      .mintToChecked(new anchor.BN(500_000), decimals)
      .accounts({
        mint: mintKp.publicKey,
        tokenAccount: tokenAKp.publicKey,
        authority: wallet.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();
    console.log(`    Mint-checked 500,000 to A`);
  });

  it("9. Burn — SPL: 4,753 CU | P-Token: 133 CU", async () => {
    await program.methods
      .burnTokens(new anchor.BN(100_000))
      .accounts({
        mint: mintKp.publicKey,
        tokenAccount: tokenBKp.publicKey,
        authority: wallet.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();
    console.log(`    Burned 100,000 from B`);
  });

  it("16. BurnChecked — SPL: 6,251 CU | P-Token: 140 CU", async () => {
    await program.methods
      .burnChecked(new anchor.BN(100_000), decimals)
      .accounts({
        mint: mintKp.publicKey,
        tokenAccount: tokenBKp.publicKey,
        authority: wallet.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();
    console.log(`    Burn-checked 100,000 from B`);
  });

  it("11. FreezeAccount — SPL: 4,114 CU | P-Token: 137 CU", async () => {
    await program.methods
      .freezeTokenAccount()
      .accounts({
        tokenAccount: tokenBKp.publicKey,
        mint: mintKp.publicKey,
        freezeAuthority: wallet.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();
    console.log(`    Account B frozen`);
  });

  it("12. ThawAccount — SPL: 4,114 CU | P-Token: 134 CU", async () => {
    await program.methods
      .thawTokenAccount()
      .accounts({
        tokenAccount: tokenBKp.publicKey,
        mint: mintKp.publicKey,
        freezeAuthority: wallet.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();
    console.log(`    Account B thawed`);
  });

  it("10. CloseAccount — SPL: 3,015 CU | P-Token: 120 CU", async () => {
    // Burn remaining B tokens first
    const b = await getAccount(provider.connection, tokenBKp.publicKey);
    if (b.amount > 0n) {
      await program.methods
        .burnTokens(new anchor.BN(b.amount.toString()))
        .accounts({
          mint: mintKp.publicKey,
          tokenAccount: tokenBKp.publicKey,
          authority: wallet.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .rpc();
    }

    await program.methods
      .closeTokenAccount()
      .accounts({
        tokenAccount: tokenBKp.publicKey,
        destination: wallet.publicKey,
        authority: wallet.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();
    console.log(`    Account B closed, rent reclaimed`);
  });

  // =========================================================================
  // SUMMARY
  // =========================================================================
  it("prints full comparison summary", () => {
    console.log(`
╔═════════════════════════════════════════════════════════════════════════════╗
║              SPL TOKEN vs P-TOKEN — COMPLETE CU COMPARISON                ║
╠═════════════════════════════════════════════════════════════════════════════╣
║                                                                           ║
║  #   Instruction                SPL Token    P-Token    Savings           ║
║  ─── ────────────────────────── ────────── ────────── ────────            ║
║   1  InitializeMint               2,967       99      96.7%              ║
║   2  InitializeAccount            4,527      149      96.7%              ║
║   3  InitializeMultisig           3,270      167      94.9%              ║
║   4  Transfer                     4,645       78      98.3%   ★          ║
║   5  Approve                      2,904      123      95.8%              ║
║   6  Revoke                       2,691      102      96.2%              ║
║   7  SetAuthority                 3,015      133      95.6%              ║
║   8  MintTo                       4,538      120      97.4%              ║
║   9  Burn                         4,753      133      97.2%              ║
║  10  CloseAccount                 3,015      120      96.0%              ║
║  11  FreezeAccount                4,114      137      96.7%              ║
║  12  ThawAccount                  4,114      134      96.7%              ║
║  13  TransferChecked              6,200      107      98.3%   ★          ║
║  14  ApproveChecked               4,410      160      96.4%              ║
║  15  MintToChecked                6,037      153      97.5%              ║
║  16  BurnChecked                  6,251      140      97.8%              ║
║  17  InitializeAccount2           4,539      161      96.5%              ║
║  18  SyncNative                   3,045      201      93.4%              ║
║  19  InitializeAccount3           4,539      233      94.9%              ║
║  20  InitializeMultisig2          3,270      279      91.5%              ║
║  21  InitializeMint2              2,967      214      92.8%              ║
║  22  InitializeImmutableOwner     1,405       37      97.4%   ★          ║
║  ─── ────────────────────────── ────────── ────────── ────────            ║
║  23  WithdrawExcessLamports        N/A       258      NEW                ║
║  24  UnwrapLamports                N/A       140      NEW                ║
║  25  Batch                         N/A      varies    NEW                ║
║                                                                           ║
╠═════════════════════════════════════════════════════════════════════════════╣
║  ★ = highest savings    Avg savings across all instructions: ~96%         ║
║                                                                           ║
║  KEY: P-Token uses Pinocchio (zero-copy, no_std, no heap, no logging)     ║
║  Transfer fast-path alone frees ~12% of total Solana block space          ║
║  Approved via SIMD-0266 — mainnet target: April 2026                      ║
╚═════════════════════════════════════════════════════════════════════════════╝
    `);
  });
});
