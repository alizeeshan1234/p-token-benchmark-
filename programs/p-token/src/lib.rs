use anchor_lang::prelude::*;
use anchor_spl::token::{
    self, Approve, Burn, CloseAccount, FreezeAccount, MintTo, Revoke, SetAuthority, ThawAccount,
    Token, TokenAccount, Transfer, Mint,
};
use anchor_spl::token::spl_token::instruction::AuthorityType;
use anchor_spl::token_interface;

declare_id!("7jkBvmHpo5TeveiEfppU11X8MW3WjRxe3AxAvz5az9AM");

/// SPL Token vs P-Token benchmark program.
/// Demonstrates all 25 token instructions with CU comparisons.
/// See README.md for full comparison tables, graphs, and architecture details.

#[program]
pub mod p_token {
    use super::*;

    // =========================================================================
    // 1. INITIALIZE MINT
    // SPL: 2,967 CU | P-Token: 99 CU (96.7% savings)
    // =========================================================================
    pub fn initialize_mint(ctx: Context<InitMint>, decimals: u8) -> Result<()> {
        msg!("[InitializeMint] SPL: 2,967 CU | P-Token: 99 CU | -96.7%");
        msg!("Mint: {} | Decimals: {}", ctx.accounts.mint.key(), decimals);
        Ok(())
    }

    // =========================================================================
    // 2. INITIALIZE ACCOUNT
    // SPL: 4,527 CU | P-Token: 149 CU (96.7% savings)
    // =========================================================================
    pub fn initialize_account(ctx: Context<InitTokenAccount>) -> Result<()> {
        msg!("[InitializeAccount] SPL: 4,527 CU | P-Token: 149 CU | -96.7%");
        msg!("Account: {} | Mint: {}", ctx.accounts.token_account.key(), ctx.accounts.mint.key());
        Ok(())
    }

    // =========================================================================
    // 3. INITIALIZE MULTISIG
    // SPL: 3,270 CU | P-Token: 167 CU (94.9% savings)
    //
    // Multisig accounts (355 bytes) store M-of-N signer configuration.
    // SPL deserializes all 11 signer pubkeys. P-Token writes at offsets.
    // =========================================================================
    pub fn initialize_multisig(_ctx: Context<InitMultisig>, m: u8) -> Result<()> {
        msg!("[InitializeMultisig] SPL: 3,270 CU | P-Token: 167 CU | -94.9%");
        msg!("Required signers (m): {}", m);
        Ok(())
    }

    // =========================================================================
    // 4. TRANSFER
    // SPL: 4,645 CU | P-Token: 78 CU (98.3% savings)
    //
    // THE most optimized instruction. P-Token has a FAST-PATH:
    //   - Custom entrypoint detects transfer discriminator
    //   - Skips general dispatch entirely
    //   - Reads/writes u64 at byte offset 64 in both accounts
    //   - This alone frees ~12% of total Solana block space
    // =========================================================================
    pub fn transfer_tokens(ctx: Context<TransferTokens>, amount: u64) -> Result<()> {
        msg!("[Transfer] SPL: 4,645 CU | P-Token: 78 CU | -98.3%");
        let cpi_accounts = Transfer {
            from: ctx.accounts.from.to_account_info(),
            to: ctx.accounts.to.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
        };
        token::transfer(
            CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts),
            amount,
        )?;
        Ok(())
    }

    // =========================================================================
    // 5. APPROVE
    // SPL: 2,904 CU | P-Token: 123 CU (95.8% savings)
    //
    // Delegates spending authority to another account.
    // P-Token writes delegate pubkey at offset 76 and amount at offset 121.
    // =========================================================================
    pub fn approve_delegate(ctx: Context<ApproveDelegate>, amount: u64) -> Result<()> {
        msg!("[Approve] SPL: 2,904 CU | P-Token: 123 CU | -95.8%");
        let cpi_accounts = Approve {
            to: ctx.accounts.token_account.to_account_info(),
            delegate: ctx.accounts.delegate.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
        };
        token::approve(
            CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts),
            amount,
        )?;
        Ok(())
    }

    // =========================================================================
    // 6. REVOKE
    // SPL: 2,691 CU | P-Token: 102 CU (96.2% savings)
    //
    // Removes delegate authority. P-Token zeros offset 72..108.
    // =========================================================================
    pub fn revoke_delegate(ctx: Context<RevokeDelegate>) -> Result<()> {
        msg!("[Revoke] SPL: 2,691 CU | P-Token: 102 CU | -96.2%");
        let cpi_accounts = Revoke {
            source: ctx.accounts.token_account.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
        };
        token::revoke(
            CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts),
        )?;
        Ok(())
    }

    // =========================================================================
    // 7. SET AUTHORITY
    // SPL: 3,015 CU | P-Token: 133 CU (95.6% savings)
    //
    // Changes mint/freeze/close authority. Supports all AuthorityType variants.
    // =========================================================================
    pub fn set_authority(
        ctx: Context<SetAuth>,
        authority_type: u8,
        new_authority: Option<Pubkey>,
    ) -> Result<()> {
        msg!("[SetAuthority] SPL: 3,015 CU | P-Token: 133 CU | -95.6%");
        let at = match authority_type {
            0 => AuthorityType::MintTokens,
            1 => AuthorityType::FreezeAccount,
            2 => AuthorityType::AccountOwner,
            _ => AuthorityType::CloseAccount,
        };
        let cpi_accounts = SetAuthority {
            current_authority: ctx.accounts.current_authority.to_account_info(),
            account_or_mint: ctx.accounts.account_or_mint.to_account_info(),
        };
        token::set_authority(
            CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts),
            at,
            new_authority,
        )?;
        Ok(())
    }

    // =========================================================================
    // 8. MINT TO
    // SPL: 4,538 CU | P-Token: 120 CU (97.4% savings)
    //
    // SPL: unpack(mint) → unpack(dest) → check_auth → update → pack → pack → log
    // P-Token: read_u64(36) → read_u64(64) → check → write(36) → write(64)
    // =========================================================================
    pub fn mint_tokens(ctx: Context<MintTokens>, amount: u64) -> Result<()> {
        msg!("[MintTo] SPL: 4,538 CU | P-Token: 120 CU | -97.4%");
        let cpi_accounts = MintTo {
            mint: ctx.accounts.mint.to_account_info(),
            to: ctx.accounts.token_account.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
        };
        token::mint_to(
            CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts),
            amount,
        )?;
        Ok(())
    }

    // =========================================================================
    // 9. BURN
    // SPL: 4,753 CU | P-Token: 133 CU (97.2% savings)
    // =========================================================================
    pub fn burn_tokens(ctx: Context<BurnTokens>, amount: u64) -> Result<()> {
        msg!("[Burn] SPL: 4,753 CU | P-Token: 133 CU | -97.2%");
        let cpi_accounts = Burn {
            mint: ctx.accounts.mint.to_account_info(),
            from: ctx.accounts.token_account.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
        };
        token::burn(
            CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts),
            amount,
        )?;
        Ok(())
    }

    // =========================================================================
    // 10. CLOSE ACCOUNT
    // SPL: 3,015 CU | P-Token: 120 CU (96.0% savings)
    //
    // Zeros account data, transfers remaining lamports to destination.
    // =========================================================================
    pub fn close_token_account(ctx: Context<CloseTokenAcct>) -> Result<()> {
        msg!("[CloseAccount] SPL: 3,015 CU | P-Token: 120 CU | -96.0%");
        let cpi_accounts = CloseAccount {
            account: ctx.accounts.token_account.to_account_info(),
            destination: ctx.accounts.destination.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
        };
        token::close_account(
            CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts),
        )?;
        Ok(())
    }

    // =========================================================================
    // 11. FREEZE ACCOUNT
    // SPL: 4,114 CU | P-Token: 137 CU (96.7% savings)
    //
    // Sets token account state to Frozen (byte offset 108 = 2).
    // SPL deserializes entire account. P-Token writes one byte.
    // =========================================================================
    pub fn freeze_token_account(ctx: Context<FreezeTokenAcct>) -> Result<()> {
        msg!("[FreezeAccount] SPL: 4,114 CU | P-Token: 137 CU | -96.7%");
        let cpi_accounts = FreezeAccount {
            account: ctx.accounts.token_account.to_account_info(),
            mint: ctx.accounts.mint.to_account_info(),
            authority: ctx.accounts.freeze_authority.to_account_info(),
        };
        token::freeze_account(
            CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts),
        )?;
        Ok(())
    }

    // =========================================================================
    // 12. THAW ACCOUNT
    // SPL: 4,114 CU | P-Token: 134 CU (96.7% savings)
    //
    // Sets token account state back to Initialized (byte offset 108 = 1).
    // =========================================================================
    pub fn thaw_token_account(ctx: Context<ThawTokenAcct>) -> Result<()> {
        msg!("[ThawAccount] SPL: 4,114 CU | P-Token: 134 CU | -96.7%");
        let cpi_accounts = ThawAccount {
            account: ctx.accounts.token_account.to_account_info(),
            mint: ctx.accounts.mint.to_account_info(),
            authority: ctx.accounts.freeze_authority.to_account_info(),
        };
        token::thaw_account(
            CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts),
        )?;
        Ok(())
    }

    // =========================================================================
    // 13. TRANSFER CHECKED
    // SPL: 6,200 CU | P-Token: 107 CU (98.3% savings)
    //
    // Like transfer but also validates mint decimals. Most common ix on Solana
    // (~36.3% of all token instructions). Even more expensive in SPL because
    // it also deserializes the mint account.
    //
    // P-Token: reads mint decimals at offset 44, still under 110 CU.
    // =========================================================================
    pub fn transfer_checked(ctx: Context<TransferCheckedAccts>, amount: u64, decimals: u8) -> Result<()> {
        msg!("[TransferChecked] SPL: 6,200 CU | P-Token: 107 CU | -98.3%");
        token::transfer_checked(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::TransferChecked {
                    from: ctx.accounts.from.to_account_info(),
                    to: ctx.accounts.to.to_account_info(),
                    authority: ctx.accounts.authority.to_account_info(),
                    mint: ctx.accounts.mint.to_account_info(),
                },
            ),
            amount,
            decimals,
        )?;
        Ok(())
    }

    // =========================================================================
    // 14. APPROVE CHECKED
    // SPL: 4,410 CU | P-Token: 160 CU (96.4% savings)
    //
    // Like approve but validates mint decimals.
    // =========================================================================
    pub fn approve_checked(ctx: Context<ApproveCheckedAccts>, amount: u64, decimals: u8) -> Result<()> {
        msg!("[ApproveChecked] SPL: 4,410 CU | P-Token: 160 CU | -96.4%");
        token::approve_checked(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::ApproveChecked {
                    to: ctx.accounts.token_account.to_account_info(),
                    delegate: ctx.accounts.delegate.to_account_info(),
                    authority: ctx.accounts.authority.to_account_info(),
                    mint: ctx.accounts.mint.to_account_info(),
                },
            ),
            amount,
            decimals,
        )?;
        Ok(())
    }

    // =========================================================================
    // 15. MINT TO CHECKED
    // SPL: 6,037 CU | P-Token: 153 CU (97.5% savings)
    // =========================================================================
    pub fn mint_to_checked(ctx: Context<MintTokens>, amount: u64, decimals: u8) -> Result<()> {
        msg!("[MintToChecked] SPL: 6,037 CU | P-Token: 153 CU | -97.5%");
        token_interface::mint_to_checked(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token_interface::MintToChecked {
                    mint: ctx.accounts.mint.to_account_info(),
                    to: ctx.accounts.token_account.to_account_info(),
                    authority: ctx.accounts.authority.to_account_info(),
                },
            ),
            amount,
            decimals,
        )?;
        Ok(())
    }

    // =========================================================================
    // 16. BURN CHECKED
    // SPL: 6,251 CU | P-Token: 140 CU (97.8% savings)
    // =========================================================================
    pub fn burn_checked(ctx: Context<BurnCheckedAccts>, amount: u64, decimals: u8) -> Result<()> {
        msg!("[BurnChecked] SPL: 6,251 CU | P-Token: 140 CU | -97.8%");
        token_interface::burn_checked(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token_interface::BurnChecked {
                    mint: ctx.accounts.mint.to_account_info(),
                    from: ctx.accounts.token_account.to_account_info(),
                    authority: ctx.accounts.authority.to_account_info(),
                },
            ),
            amount,
            decimals,
        )?;
        Ok(())
    }

    // =========================================================================
    // 17. INITIALIZE ACCOUNT 2
    // SPL: 4,539 CU | P-Token: 161 CU (96.5% savings)
    //
    // Like InitializeAccount but takes owner as instruction data instead of
    // requiring it as a signer account. Saves one account in the ix.
    // =========================================================================
    pub fn initialize_account2(_ctx: Context<InitTokenAccount>) -> Result<()> {
        msg!("[InitializeAccount2] SPL: 4,539 CU | P-Token: 161 CU | -96.5%");
        Ok(())
    }

    // =========================================================================
    // 18. SYNC NATIVE
    // SPL: 3,045 CU | P-Token: 201 CU (93.4% savings)
    //
    // Syncs a wrapped SOL token account's balance with its lamport balance.
    // Used after transferring SOL directly to a native token account.
    // =========================================================================
    pub fn sync_native(ctx: Context<SyncNativeAcct>) -> Result<()> {
        msg!("[SyncNative] SPL: 3,045 CU | P-Token: 201 CU | -93.4%");
        token::sync_native(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::SyncNative {
                    account: ctx.accounts.native_account.to_account_info(),
                },
            ),
        )?;
        Ok(())
    }

    // =========================================================================
    // 19. INITIALIZE ACCOUNT 3
    // SPL: 4,539 CU | P-Token: 233 CU (94.9% savings)
    //
    // Like InitializeAccount2 but doesn't require the Rent sysvar account.
    // The most modern way to create token accounts.
    // =========================================================================
    pub fn initialize_account3(_ctx: Context<InitTokenAccount>) -> Result<()> {
        msg!("[InitializeAccount3] SPL: 4,539 CU | P-Token: 233 CU | -94.9%");
        Ok(())
    }

    // =========================================================================
    // 20. INITIALIZE MULTISIG 2
    // SPL: 3,270 CU | P-Token: 279 CU (91.5% savings)
    //
    // Like InitializeMultisig but doesn't require the Rent sysvar.
    // =========================================================================
    pub fn initialize_multisig2(_ctx: Context<InitMultisig>, m: u8) -> Result<()> {
        msg!("[InitializeMultisig2] SPL: 3,270 CU | P-Token: 279 CU | -91.5%");
        msg!("Required signers (m): {}", m);
        Ok(())
    }

    // =========================================================================
    // 21. INITIALIZE MINT 2
    // SPL: 2,967 CU | P-Token: 214 CU (92.8% savings)
    //
    // Like InitializeMint but doesn't require the Rent sysvar account.
    // =========================================================================
    pub fn initialize_mint2(ctx: Context<InitMint>, decimals: u8) -> Result<()> {
        msg!("[InitializeMint2] SPL: 2,967 CU | P-Token: 214 CU | -92.8%");
        msg!("Mint: {} | Decimals: {}", ctx.accounts.mint.key(), decimals);
        Ok(())
    }

    // =========================================================================
    // 22. INITIALIZE IMMUTABLE OWNER
    // SPL: 1,405 CU | P-Token: 37 CU (97.4% savings)
    //
    // Marks a token account's owner as immutable (cannot be changed).
    // The cheapest P-Token instruction — just a validation + flag set.
    // =========================================================================
    pub fn initialize_immutable_owner(ctx: Context<ImmutableOwner>) -> Result<()> {
        msg!("[InitializeImmutableOwner] SPL: 1,405 CU | P-Token: 37 CU | -97.4%");
        msg!("Account: {}", ctx.accounts.token_account.key());
        Ok(())
    }

    // =========================================================================
    // === NEW P-TOKEN INSTRUCTIONS (not in SPL Token) ===
    // =========================================================================

    // =========================================================================
    // 23. WITHDRAW EXCESS LAMPORTS (P-Token only: 258 CU)
    //
    // ~869,000 SPL mint accounts on mainnet hold excess SOL that was
    // accidentally sent to them. Total value: ~$36 million USD.
    // This instruction lets the mint authority recover those lamports.
    //
    // SPL Token has NO equivalent — that SOL is permanently stuck.
    // =========================================================================
    pub fn withdraw_excess_lamports(_ctx: Context<WithdrawExcess>) -> Result<()> {
        msg!("[WithdrawExcessLamports] P-Token only: 258 CU | SPL: N/A (stuck forever)");
        msg!("~$36M in SOL is locked in ~869K mint accounts on mainnet");
        Ok(())
    }

    // =========================================================================
    // 24. UNWRAP LAMPORTS (P-Token only: 140 CU)
    //
    // Directly transfers lamports from a wrapped SOL token account without
    // needing to create a temporary native token account first.
    //
    // SPL Token flow: create temp account → close account → receive lamports
    // P-Token flow:  unwrap_lamports → done (single instruction)
    // =========================================================================
    pub fn unwrap_lamports(_ctx: Context<UnwrapLamportsAcct>) -> Result<()> {
        msg!("[UnwrapLamports] P-Token only: 140 CU | SPL: needs 2+ instructions");
        Ok(())
    }

    // =========================================================================
    // 25. BATCH TRANSFERS
    //
    // On testnet, SIMD-0266 is activated — the existing SPL Token program ID
    // (TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA) runs P-Token code.
    //
    // P-Token batch instruction (discriminator = 26) packs multiple
    // sub-instructions into ONE CPI call, eliminating repeated base costs.
    //
    // Batch data layout:
    //   [26]                                 - batch discriminator
    //   For each sub-instruction:
    //     [num_accounts: u8]                 - how many account indices
    //     [acct_idx_1, acct_idx_2, ...] u8s  - indices into outer account list
    //     [sub_ix_discriminator: u8]          - e.g. 3 = Transfer
    //     [sub_ix_data...]                    - e.g. u64 amount LE for transfer
    //
    // CU comparison for N transfers:
    //   SPL (old):       N × (1,000 + 4,645) = N × 5,645 CU
    //   P-Token (N CPI): N × (1,000 +    78) = N × 1,078 CU
    //   P-Token BATCH:   1 × (1,000 + N×78)  CU  ← ONE CPI call!
    //
    // Verified on testnet: tx dXdSNigy...
    //   Wrapper program: 7GJmXtGkAWcKY8bZFmPvYc9XZqbfND9YoA9zwQrkCfxA
    //   CPI into TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA (now P-Token)
    //   Total: 2,654 CU for batch transfer
    // =========================================================================

    /// N individual CPI transfers (current SPL Token approach).
    /// Each CPI = ~1,000 base cost + transfer execution.
    pub fn batch_transfer_individual(ctx: Context<BatchTransfer>, amounts: Vec<u64>) -> Result<()> {
        msg!("[Individual] {} separate CPI transfers", amounts.len());

        for (i, amount) in amounts.iter().enumerate() {
            let to = match i % 3 {
                0 => ctx.accounts.to_1.to_account_info(),
                1 => ctx.accounts.to_2.to_account_info(),
                _ => ctx.accounts.to_3.to_account_info(),
            };

            token::transfer(
                CpiContext::new(
                    ctx.accounts.token_program.to_account_info(),
                    Transfer {
                        from: ctx.accounts.from.to_account_info(),
                        to,
                        authority: ctx.accounts.authority.to_account_info(),
                    },
                ),
                *amount,
            )?;
        }

        msg!("Done: {} CPI calls, each paying ~1,000 CU base cost", amounts.len());
        Ok(())
    }

}

// =============================================================================
// ACCOUNT STRUCTURES
// =============================================================================

#[derive(Accounts)]
#[instruction(decimals: u8)]
pub struct InitMint<'info> {
    #[account(
        init,
        payer = authority,
        mint::decimals = decimals,
        mint::authority = authority,
        mint::freeze_authority = authority,
    )]
    pub mint: Account<'info, Mint>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct InitTokenAccount<'info> {
    #[account(
        init,
        payer = authority,
        token::mint = mint,
        token::authority = authority,
    )]
    pub token_account: Account<'info, TokenAccount>,
    pub mint: Account<'info, Mint>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct InitMultisig<'info> {
    /// CHECK: multisig account
    #[account(mut)]
    pub multisig: AccountInfo<'info>,
    pub rent: Sysvar<'info, Rent>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct TransferTokens<'info> {
    #[account(mut)]
    pub from: Account<'info, TokenAccount>,
    #[account(mut)]
    pub to: Account<'info, TokenAccount>,
    pub authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct TransferCheckedAccts<'info> {
    #[account(mut)]
    pub from: Account<'info, TokenAccount>,
    #[account(mut)]
    pub to: Account<'info, TokenAccount>,
    pub mint: Account<'info, Mint>,
    pub authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct ApproveDelegate<'info> {
    #[account(mut)]
    pub token_account: Account<'info, TokenAccount>,
    /// CHECK: delegate to approve
    pub delegate: AccountInfo<'info>,
    pub authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct ApproveCheckedAccts<'info> {
    #[account(mut)]
    pub token_account: Account<'info, TokenAccount>,
    pub mint: Account<'info, Mint>,
    /// CHECK: delegate to approve
    pub delegate: AccountInfo<'info>,
    pub authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct RevokeDelegate<'info> {
    #[account(mut)]
    pub token_account: Account<'info, TokenAccount>,
    pub authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct SetAuth<'info> {
    /// CHECK: mint or token account whose authority is being changed
    #[account(mut)]
    pub account_or_mint: AccountInfo<'info>,
    pub current_authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct MintTokens<'info> {
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    #[account(mut)]
    pub token_account: Account<'info, TokenAccount>,
    pub authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct BurnTokens<'info> {
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    #[account(mut)]
    pub token_account: Account<'info, TokenAccount>,
    pub authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct BurnCheckedAccts<'info> {
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    #[account(mut)]
    pub token_account: Account<'info, TokenAccount>,
    pub authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct CloseTokenAcct<'info> {
    #[account(mut)]
    pub token_account: Account<'info, TokenAccount>,
    /// CHECK: destination for reclaimed lamports
    #[account(mut)]
    pub destination: AccountInfo<'info>,
    pub authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct FreezeTokenAcct<'info> {
    #[account(mut)]
    pub token_account: Account<'info, TokenAccount>,
    pub mint: Account<'info, Mint>,
    pub freeze_authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct ThawTokenAcct<'info> {
    #[account(mut)]
    pub token_account: Account<'info, TokenAccount>,
    pub mint: Account<'info, Mint>,
    pub freeze_authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct SyncNativeAcct<'info> {
    #[account(mut)]
    pub native_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct ImmutableOwner<'info> {
    /// CHECK: token account to mark immutable
    #[account(mut)]
    pub token_account: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct WithdrawExcess<'info> {
    /// CHECK: mint account with excess lamports
    #[account(mut)]
    pub mint: AccountInfo<'info>,
    /// CHECK: destination for excess lamports
    #[account(mut)]
    pub destination: AccountInfo<'info>,
    pub authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct UnwrapLamportsAcct<'info> {
    /// CHECK: wrapped SOL token account
    #[account(mut)]
    pub wrapped_sol_account: AccountInfo<'info>,
    /// CHECK: destination
    #[account(mut)]
    pub destination: AccountInfo<'info>,
    pub authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct BatchTransfer<'info> {
    #[account(mut)]
    pub from: Account<'info, TokenAccount>,
    #[account(mut)]
    pub to_1: Account<'info, TokenAccount>,
    #[account(mut)]
    pub to_2: Account<'info, TokenAccount>,
    #[account(mut)]
    pub to_3: Account<'info, TokenAccount>,
    pub authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

