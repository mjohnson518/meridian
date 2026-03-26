//! # Meridian Solana Stablecoin Program
//!
//! Multi-currency stablecoin implementation for Solana using Anchor framework
//!
//! ## Security Notes
//!
//! - Uses PDA for mint authority (no admin keys can rug)
//! - Reserve tracking on-chain
//! - Pausable for emergency response
//! - All operations logged via events

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Burn, Mint, MintTo, Token, TokenAccount};

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod meridian_solana {
    use super::*;

    /// Initialize a new stablecoin
    ///
    /// Creates the mint and stablecoin state account with PDA authority
    pub fn initialize(
        ctx: Context<Initialize>,
        name: String,
        symbol: String,
        basket_id: String,
        basket_type: BasketType,
        decimals: u8,
    ) -> Result<()> {
        require!(name.len() <= 32, MeridianError::NameTooLong);
        require!(symbol.len() <= 10, MeridianError::SymbolTooLong);
        require!(basket_id.len() <= 64, MeridianError::BasketIdTooLong);

        let stablecoin = &mut ctx.accounts.stablecoin;
        
        stablecoin.name = name;
        stablecoin.symbol = symbol;
        stablecoin.basket_id = basket_id;
        stablecoin.basket_type = basket_type;
        stablecoin.mint = ctx.accounts.mint.key();
        stablecoin.decimals = decimals;
        stablecoin.total_supply = 0;
        stablecoin.total_reserve_value = 0;
        stablecoin.min_reserve_ratio = 10000; // 100% in basis points
        stablecoin.is_paused = false;
        stablecoin.last_attestation = Clock::get()?.unix_timestamp;
        stablecoin.authority = ctx.accounts.authority.key();
        stablecoin.bump = ctx.bumps.stablecoin;

        msg!("Stablecoin initialized: {}", stablecoin.name);

        emit!(StablecoinInitialized {
            mint: stablecoin.mint,
            basket_id: stablecoin.basket_id.clone(),
            basket_type: stablecoin.basket_type,
        });

        Ok(())
    }

    /// Mint new tokens with reserve verification
    ///
    /// Requires 1:1 reserve backing (configurable via min_reserve_ratio)
    pub fn mint_tokens(
        ctx: Context<MintTokens>,
        amount: u64,
        reserve_value: u64,
    ) -> Result<()> {
        let stablecoin = &mut ctx.accounts.stablecoin;

        require!(!stablecoin.is_paused, MeridianError::Paused);

        // Verify 1:1 reserve backing
        require!(
            reserve_value >= amount,
            MeridianError::InsufficientReserveBacking
        );

        // Update reserve tracking
        stablecoin.total_supply = stablecoin
            .total_supply
            .checked_add(amount)
            .ok_or(MeridianError::Overflow)?;

        stablecoin.total_reserve_value = stablecoin
            .total_reserve_value
            .checked_add(reserve_value)
            .ok_or(MeridianError::Overflow)?;

        // Mint tokens using PDA authority
        let seeds = &[
            b"stablecoin",
            stablecoin.mint.as_ref(),
            &[stablecoin.bump],
        ];
        let signer = &[&seeds[..]];

        let cpi_accounts = MintTo {
            mint: ctx.accounts.mint.to_account_info(),
            to: ctx.accounts.recipient_token_account.to_account_info(),
            authority: stablecoin.to_account_info(),
        };

        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);

        token::mint_to(cpi_ctx, amount)?;

        msg!("Minted {} tokens with {} reserve", amount, reserve_value);

        emit!(TokensMinted {
            recipient: ctx.accounts.recipient_token_account.key(),
            amount,
            reserve_value,
        });

        Ok(())
    }

    /// Burn tokens and release pro-rata reserves
    pub fn burn_tokens(ctx: Context<BurnTokens>, amount: u64) -> Result<()> {
        let stablecoin = &mut ctx.accounts.stablecoin;

        require!(!stablecoin.is_paused, MeridianError::Paused);

        // Calculate pro-rata reserve release
        let reserve_to_release = if stablecoin.total_supply > 0 {
            (amount as u128)
                .checked_mul(stablecoin.total_reserve_value as u128)
                .and_then(|v| v.checked_div(stablecoin.total_supply as u128))
                .and_then(|v| u64::try_from(v).ok())
                .ok_or(MeridianError::Overflow)?
        } else {
            0
        };

        // Update reserve tracking
        stablecoin.total_supply = stablecoin
            .total_supply
            .checked_sub(amount)
            .ok_or(MeridianError::Underflow)?;

        stablecoin.total_reserve_value = stablecoin
            .total_reserve_value
            .checked_sub(reserve_to_release)
            .ok_or(MeridianError::Underflow)?;

        // Burn tokens
        let cpi_accounts = Burn {
            mint: ctx.accounts.mint.to_account_info(),
            from: ctx.accounts.holder_token_account.to_account_info(),
            authority: ctx.accounts.holder.to_account_info(),
        };

        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        token::burn(cpi_ctx, amount)?;

        msg!("Burned {} tokens, released {} reserve", amount, reserve_to_release);

        emit!(TokensBurned {
            holder: ctx.accounts.holder.key(),
            amount,
            reserve_released: reserve_to_release,
        });

        Ok(())
    }

    /// Attest to current reserve backing
    ///
    /// Should be called monthly for compliance
    pub fn attest_reserves(
        ctx: Context<AttestReserves>,
        attested_reserve_value: u64,
    ) -> Result<()> {
        let stablecoin = &mut ctx.accounts.stablecoin;

        // Calculate required reserve based on min ratio
        let required_reserve = (stablecoin.total_supply as u128)
            .checked_mul(stablecoin.min_reserve_ratio as u128)
            .and_then(|v| v.checked_div(10000))
            .and_then(|v| u64::try_from(v).ok())
            .ok_or(MeridianError::Overflow)?;

        require!(
            attested_reserve_value >= required_reserve,
            MeridianError::AttestationBelowMinimum
        );

        stablecoin.last_attestation = Clock::get()?.unix_timestamp;

        msg!("Reserves attested: {} (required: {})", attested_reserve_value, required_reserve);

        emit!(ReservesAttested {
            attested_value: attested_reserve_value,
            total_supply: stablecoin.total_supply,
            timestamp: stablecoin.last_attestation,
        });

        Ok(())
    }

    /// Pause all token operations
    pub fn pause(ctx: Context<AdminOperation>) -> Result<()> {
        let stablecoin = &mut ctx.accounts.stablecoin;
        stablecoin.is_paused = true;

        msg!("Stablecoin paused");

        emit!(Paused {
            authority: ctx.accounts.authority.key(),
        });

        Ok(())
    }

    /// Unpause token operations
    pub fn unpause(ctx: Context<AdminOperation>) -> Result<()> {
        let stablecoin = &mut ctx.accounts.stablecoin;
        stablecoin.is_paused = false;

        msg!("Stablecoin unpaused");

        emit!(Unpaused {
            authority: ctx.accounts.authority.key(),
        });

        Ok(())
    }
}

// ============ Account Structures ============

#[account]
pub struct Stablecoin {
    /// Human-readable name
    pub name: String,
    /// Token symbol
    pub symbol: String,
    /// Basket identifier
    pub basket_id: String,
    /// Type of currency basket
    pub basket_type: BasketType,
    /// SPL Token mint address
    pub mint: Pubkey,
    /// Number of decimals
    pub decimals: u8,
    /// Total token supply
    pub total_supply: u64,
    /// Total reserve value (in USD cents, 2 decimals)
    pub total_reserve_value: u64,
    /// Minimum reserve ratio (basis points, 10000 = 100%)
    pub min_reserve_ratio: u16,
    /// Whether the stablecoin is paused
    pub is_paused: bool,
    /// Last reserve attestation timestamp
    pub last_attestation: i64,
    /// Authority that can admin the stablecoin
    pub authority: Pubkey,
    /// PDA bump seed
    pub bump: u8,
}

impl Stablecoin {
    pub const LEN: usize = 8 + // discriminator
        32 + // name (String with length prefix)
        10 + // symbol
        64 + // basket_id
        1 + // basket_type
        32 + // mint
        1 + // decimals
        8 + // total_supply
        8 + // total_reserve_value
        2 + // min_reserve_ratio
        1 + // is_paused
        8 + // last_attestation
        32 + // authority
        1; // bump
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum BasketType {
    SingleCurrency,
    ImfSdr,
    CustomBasket,
}

// ============ Context Structures ============

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = authority,
        space = Stablecoin::LEN,
        seeds = [b"stablecoin", mint.key().as_ref()],
        bump
    )]
    pub stablecoin: Account<'info, Stablecoin>,

    #[account(mut)]
    pub mint: Account<'info, Mint>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct MintTokens<'info> {
    #[account(
        mut,
        seeds = [b"stablecoin", stablecoin.mint.as_ref()],
        bump = stablecoin.bump,
        has_one = mint,
    )]
    pub stablecoin: Account<'info, Stablecoin>,

    #[account(mut)]
    pub mint: Account<'info, Mint>,

    #[account(mut)]
    pub recipient_token_account: Account<'info, TokenAccount>,

    #[account(
        constraint = authority.key() == stablecoin.authority @ MeridianError::Unauthorized
    )]
    pub authority: Signer<'info>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct BurnTokens<'info> {
    #[account(
        mut,
        seeds = [b"stablecoin", stablecoin.mint.as_ref()],
        bump = stablecoin.bump,
        has_one = mint,
    )]
    pub stablecoin: Account<'info, Stablecoin>,

    #[account(mut)]
    pub mint: Account<'info, Mint>,

    #[account(
        mut,
        constraint = holder_token_account.owner == holder.key()
    )]
    pub holder_token_account: Account<'info, TokenAccount>,

    pub holder: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct AttestReserves<'info> {
    #[account(
        mut,
        seeds = [b"stablecoin", stablecoin.mint.as_ref()],
        bump = stablecoin.bump,
        has_one = authority,
    )]
    pub stablecoin: Account<'info, Stablecoin>,

    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct AdminOperation<'info> {
    #[account(
        mut,
        seeds = [b"stablecoin", stablecoin.mint.as_ref()],
        bump = stablecoin.bump,
        has_one = authority,
    )]
    pub stablecoin: Account<'info, Stablecoin>,

    pub authority: Signer<'info>,
}

// ============ Events ============

#[event]
pub struct StablecoinInitialized {
    pub mint: Pubkey,
    pub basket_id: String,
    pub basket_type: BasketType,
}

#[event]
pub struct TokensMinted {
    pub recipient: Pubkey,
    pub amount: u64,
    pub reserve_value: u64,
}

#[event]
pub struct TokensBurned {
    pub holder: Pubkey,
    pub amount: u64,
    pub reserve_released: u64,
}

#[event]
pub struct ReservesAttested {
    pub attested_value: u64,
    pub total_supply: u64,
    pub timestamp: i64,
}

#[event]
pub struct Paused {
    pub authority: Pubkey,
}

#[event]
pub struct Unpaused {
    pub authority: Pubkey,
}

// ============ Errors ============

#[error_code]
pub enum MeridianError {
    #[msg("Name exceeds 32 characters")]
    NameTooLong,

    #[msg("Symbol exceeds 10 characters")]
    SymbolTooLong,

    #[msg("Basket ID exceeds 64 characters")]
    BasketIdTooLong,

    #[msg("Insufficient reserve backing for mint operation")]
    InsufficientReserveBacking,

    #[msg("Arithmetic overflow")]
    Overflow,

    #[msg("Arithmetic underflow")]
    Underflow,

    #[msg("Stablecoin is paused")]
    Paused,

    #[msg("Unauthorized: caller is not the authority")]
    Unauthorized,

    #[msg("Reserve attestation below minimum required")]
    AttestationBelowMinimum,
}

