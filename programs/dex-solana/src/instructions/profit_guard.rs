use crate::error::ErrorCode;
use crate::*;
use anchor_spl::token_interface::TokenAccount;

pub const PROFIT_SNAPSHOT_SEED: &[u8] = b"profit_snapshot";
pub const SOL_PRICE_USDC: u64 = 65;
pub const USDC_DECIMALS_MULTIPLIER: u64 = 1_000_000;
pub const LAMPORTS_PER_SOL: u64 = 1_000_000_000;

// Covers transaction fees charged before the first instruction can snapshot.
// Jito/Helius tips are normal transfer instructions and are captured directly.
pub const REQUIRED_PROFIT_USDC_MICRO: u64 = 10_000;

#[account]
pub struct ProfitSnapshot {
    pub owner: Pubkey,
    pub sol_lamports: u64,
    pub usdc_amount: u64,
}

impl ProfitSnapshot {
    pub const SPACE: usize = 8 + 32 + 8 + 8;
}

#[derive(Accounts)]
pub struct CreateProfitSnapshotAccounts<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        constraint = usdc_token_account.owner == signer.key() @ ErrorCode::InvalidTokenAccount,
        constraint = usdc_token_account.mint == usdc_mint::id() @ ErrorCode::InvalidTokenMint,
    )]
    pub usdc_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = signer,
        space = ProfitSnapshot::SPACE,
        seeds = [PROFIT_SNAPSHOT_SEED, signer.key().as_ref()],
        bump
    )]
    pub profit_snapshot: Account<'info, ProfitSnapshot>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ProfitCheckAccounts<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        constraint = usdc_token_account.owner == signer.key() @ ErrorCode::InvalidTokenAccount,
        constraint = usdc_token_account.mint == usdc_mint::id() @ ErrorCode::InvalidTokenMint,
    )]
    pub usdc_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [PROFIT_SNAPSHOT_SEED, signer.key().as_ref()],
        bump
    )]
    pub profit_snapshot: Account<'info, ProfitSnapshot>,
}

pub fn create_profit_snapshot_handler<'a>(
    ctx: Context<'_, '_, 'a, 'a, CreateProfitSnapshotAccounts<'a>>,
) -> Result<()> {
    let snapshot = &mut ctx.accounts.profit_snapshot;
    snapshot.owner = ctx.accounts.signer.key();
    snapshot.sol_lamports = ctx.accounts.signer.lamports();
    snapshot.usdc_amount = ctx.accounts.usdc_token_account.amount;

    msg!(
        "ProfitGuard snapshot owner={} sol_lamports={} usdc_amount={}",
        snapshot.owner,
        snapshot.sol_lamports,
        snapshot.usdc_amount
    );

    Ok(())
}

pub fn profit_check_handler<'a>(
    ctx: Context<'_, '_, 'a, 'a, ProfitCheckAccounts<'a>>,
) -> Result<()> {
    let snapshot = &ctx.accounts.profit_snapshot;
    require!(
        snapshot.owner == ctx.accounts.signer.key(),
        ErrorCode::InvalidSigner
    );

    let start_value = portfolio_value_usdc_micro(snapshot.sol_lamports, snapshot.usdc_amount)?;
    let current_value = portfolio_value_usdc_micro(
        ctx.accounts.signer.lamports(),
        ctx.accounts.usdc_token_account.amount,
    )?;
    let required_value = start_value
        .checked_add(REQUIRED_PROFIT_USDC_MICRO as u128)
        .ok_or(ErrorCode::CalculationError)?;

    msg!(
        "ProfitGuard check start_value={} current_value={} required_value={} sol_price_usdc={}",
        start_value,
        current_value,
        required_value,
        SOL_PRICE_USDC
    );

    require!(
        current_value >= required_value,
        ErrorCode::ProfitGuardNoProfit
    );

    Ok(())
}

fn portfolio_value_usdc_micro(sol_lamports: u64, usdc_amount: u64) -> Result<u128> {
    let sol_value = (sol_lamports as u128)
        .checked_mul(SOL_PRICE_USDC as u128)
        .ok_or(ErrorCode::CalculationError)?
        .checked_mul(USDC_DECIMALS_MULTIPLIER as u128)
        .ok_or(ErrorCode::CalculationError)?
        .checked_div(LAMPORTS_PER_SOL as u128)
        .ok_or(ErrorCode::CalculationError)?;

    sol_value
        .checked_add(usdc_amount as u128)
        .ok_or(ErrorCode::CalculationError.into())
}
