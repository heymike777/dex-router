use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use anchor_spl::token_interface::{TokenAccount as InterfaceTokenAccount, TokenInterface};
use crate::{
    authority_pda, close_authority, error::ErrorCode, wsol_program, Dex, Route, SwapAccounts,
    SwapArgs, MAX_HOPS, SA_AUTHORITY_SEED,
};

pub const ARBITRAGE_OUTPUT_SEED: &[u8] = b"arb_output";

/// Router-derived address, but SPL authority is the payer. Closing the native
/// account therefore needs only the payer signature, not an intermediary wallet.
#[derive(Accounts)]
pub struct PrepareArbitrageOutput<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        init_if_needed, payer = payer,
        seeds = [ARBITRAGE_OUTPUT_SEED, payer.key().as_ref()], bump,
        token::mint = mint, token::authority = payer,
        token::token_program = token_program,
    )]
    pub output: Account<'info, TokenAccount>,
    #[account(address = wsol_program::id())]
    pub mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn prepare_arbitrage_output_handler(ctx: Context<PrepareArbitrageOutput>) -> Result<()> {
    // Never repurpose an existing funded or delegated account as fresh output.
    require!(ctx.accounts.output.amount == 0, ErrorCode::InvalidTokenAccount);
    require!(ctx.accounts.output.delegate.is_none(), ErrorCode::InvalidTokenAccount);
    require!(ctx.accounts.output.close_authority.is_none(), ErrorCode::InvalidTokenAccount);
    Ok(())
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct CompactArbitrageArgs {
    pub amount_in: u64,
    pub min_return: u64,
    pub dexes: Vec<Dex>,
}

impl CompactArbitrageArgs {
    pub fn expand(self) -> Result<SwapArgs> {
        require!(self.amount_in > 0, ErrorCode::AmountInMustBeGreaterThanZero);
        require!(self.min_return >= self.amount_in, ErrorCode::MinReturnNotReached);
        require!((2..=MAX_HOPS).contains(&self.dexes.len()), ErrorCode::TooManyHops);
        let hops = self.dexes.into_iter().map(|dex| Route { dexes: vec![dex], weights: vec![100] }).collect();
        Ok(SwapArgs {
            amount_in: self.amount_in, expect_amount_out: self.min_return,
            min_return: self.min_return, amounts: vec![self.amount_in], routes: vec![hops],
        })
    }
}

pub fn arbitrage_compact_handler<'a>(
    ctx: Context<'_, '_, 'a, 'a, SwapAccounts<'a>>,
    args: CompactArbitrageArgs,
) -> Result<()> {
    require_keys_eq!(ctx.accounts.source_mint.key(), wsol_program::id(), ErrorCode::InvalidTokenMint);
    require_keys_eq!(ctx.accounts.destination_mint.key(), wsol_program::id(), ErrorCode::InvalidTokenMint);
    require_keys_neq!(ctx.accounts.source_token_account.key(), ctx.accounts.destination_token_account.key(), ErrorCode::InvalidTokenAccount);
    let (output, _) = Pubkey::find_program_address(
        &[ARBITRAGE_OUTPUT_SEED, ctx.accounts.payer.key().as_ref()], &crate::ID);
    require_keys_eq!(ctx.accounts.destination_token_account.key(), output, ErrorCode::InvalidTokenAccount);
    require_keys_eq!(ctx.accounts.destination_token_account.owner, ctx.accounts.payer.key(), ErrorCode::InvalidTokenAccount);
    let expanded = args.expand()?;
    // Same execution and min-return validation as the legacy ABI. The client's
    // final profit_check still covers network fees, rent, tip and WSOL close.
    crate::instructions::swap_handler(ctx, expanded, 0)
}

/// Every hop after the first is authorised by the router authority PDA, so an
/// arbitrage cycle parks its intermediate token in a PDA-owned account whose
/// rent the payer funds. This closes that account once the cycle emptied it
/// and refunds the rent to the payer in the same transaction, before
/// `profit_check`. Only the configured closer may call it; a non-empty
/// account is kept (the trade then fails in `profit_check`, never here).
#[derive(Accounts)]
pub struct CloseArbitrageIntermediate<'info> {
    #[account(mut, address = close_authority::id() @ ErrorCode::InvalidSigner)]
    pub payer: Signer<'info>,
    /// CHECK: router swap authority PDA, the SPL owner of the account
    #[account(address = authority_pda::id() @ ErrorCode::InvalidAuthorityPda)]
    pub sa_authority: AccountInfo<'info>,
    #[account(mut, token::authority = sa_authority, token::token_program = token_program)]
    pub intermediate: InterfaceAccount<'info, InterfaceTokenAccount>,
    pub token_program: Interface<'info, TokenInterface>,
}

pub fn close_arbitrage_intermediate_handler(ctx: Context<CloseArbitrageIntermediate>) -> Result<()> {
    if ctx.accounts.intermediate.amount != 0 {
        msg!("ArbitrageClose kept non-empty intermediate {}", ctx.accounts.intermediate.key());
        return Ok(());
    }
    crate::utils::close_token_account(
        ctx.accounts.intermediate.to_account_info(),
        ctx.accounts.payer.to_account_info(),
        ctx.accounts.sa_authority.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        Some(SA_AUTHORITY_SEED),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn compact_expands_to_single_route_and_preserves_minimum() {
        let args = CompactArbitrageArgs { amount_in: 1_000, min_return: 1_100,
            dexes: vec![Dex::RaydiumSwapV2, Dex::MeteoraDlmmSwap2] };
        let bytes = args.try_to_vec().unwrap();
        assert_eq!(bytes.len() + 8, 30);
        let expanded = CompactArbitrageArgs::try_from_slice(&bytes).unwrap().expand().unwrap();
        assert_eq!(expanded.amount_in, 1_000);
        assert_eq!(expanded.min_return, 1_100);
        assert_eq!(expanded.expect_amount_out, 1_100);
        assert_eq!(expanded.amounts, vec![1_000]);
        assert_eq!(expanded.routes.len(), 1);
        assert_eq!(expanded.routes[0].len(), 2);
        assert_eq!(expanded.routes[0][0].weights, vec![100]);
        assert_eq!(expanded.routes[0][1].dexes, vec![Dex::MeteoraDlmmSwap2]);
    }
    #[test]
    fn four_hops_expand_without_changing_amount_or_guard() {
        let args = CompactArbitrageArgs { amount_in: 1_000, min_return: 1_100,
            dexes: vec![Dex::MeteoraDlmmSwap2; 4] };
        assert_eq!(args.try_to_vec().unwrap().len() + 8, 32);
        let expanded = args.expand().unwrap();
        assert_eq!(expanded.routes[0].len(), 4);
        assert_eq!(expanded.amount_in, 1_000);
        assert_eq!(expanded.min_return, 1_100);
    }
    #[test]
    fn rejects_loss_zero_input_and_invalid_hop_counts() {
        for (input, min, count) in [(0, 0, 2), (100, 99, 2), (100, 100, 0), (100, 100, 1), (100, 100, 5)] {
            assert!(CompactArbitrageArgs { amount_in: input, min_return: min,
                dexes: vec![Dex::MeteoraDlmmSwap2; count] }.expand().is_err());
        }
    }
}
