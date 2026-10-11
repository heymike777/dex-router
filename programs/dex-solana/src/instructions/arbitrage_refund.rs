//! Refund only accounts created earlier in this transaction. Old wallet rent
//! must never subsidise a losing arbitrage cycle.
use crate::{authority_pda, close_authority, error::ErrorCode, SA_AUTHORITY_SEED};
use anchor_lang::{
    prelude::*,
    solana_program::{
        instruction::{get_stack_height, AccountMeta, Instruction, TRANSACTION_LEVEL_STACK_HEIGHT},
        program::invoke_signed,
        sysvar::instructions::{load_current_index_checked, load_instruction_at_checked},
    },
};
use anchor_spl::token_2022::spl_token_2022::extension::{
    transfer_fee::TransferFeeConfig, BaseStateWithExtensions, StateWithExtensions,
};
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

const PUMP: Pubkey = pubkey!("pAMMBay6oceH9fJKBRHGP5D4bD4sWpmSwMn52FMfXEA");
const ATA: Pubkey = pubkey!("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");

fn earlier_creation(
    sysvar: &AccountInfo,
    program: Pubkey,
    data: &[u8],
    keys: &[Pubkey],
) -> Result<()> {
    // The instructions sysvar describes top-level instructions, not inner CPI.
    require!(
        get_stack_height() == TRANSACTION_LEVEL_STACK_HEIGHT,
        ErrorCode::InvalidTokenAccount
    );
    let current = load_current_index_checked(sysvar)?;
    for index in 0..current {
        let ix = load_instruction_at_checked(usize::from(index), sysvar)?;
        if ix.program_id == program
            && ix.data == data
            && ix.accounts.len() >= keys.len()
            && ix
                .accounts
                .iter()
                .zip(keys)
                .all(|(meta, key)| meta.pubkey == *key)
        {
            return Ok(());
        }
    }
    err!(ErrorCode::InvalidTokenAccount)
}

#[derive(Accounts)]
pub struct CloseArbitrageIntermediateV2<'info> {
    #[account(mut, address = close_authority::id() @ ErrorCode::InvalidSigner)]
    pub payer: Signer<'info>,
    /// CHECK: fixed router signer.
    #[account(address = authority_pda::id() @ ErrorCode::InvalidAuthorityPda)]
    pub sa_authority: AccountInfo<'info>,
    #[account(mut, token::mint = mint, token::authority = sa_authority, token::token_program = token_program)]
    pub intermediate: InterfaceAccount<'info, TokenAccount>,
    pub token_program: Interface<'info, TokenInterface>,
    #[account(mut, mint::token_program = token_program)]
    pub mint: InterfaceAccount<'info, Mint>,
    /// CHECK: fixed sysvar, checked loader below.
    #[account(address = anchor_lang::solana_program::sysvar::instructions::ID)]
    pub instructions: AccountInfo<'info>,
}

pub fn close_arbitrage_intermediate_v2_handler(
    ctx: Context<CloseArbitrageIntermediateV2>,
) -> Result<()> {
    let a = ctx.accounts;
    // ATA Create (empty data) is deliberately non-idempotent: an existing
    // account would have failed before reaching this close instruction.
    earlier_creation(
        &a.instructions,
        ATA,
        &[],
        &[
            a.payer.key(),
            a.intermediate.key(),
            a.sa_authority.key(),
            a.mint.key(),
            System::id(),
            a.token_program.key(),
        ],
    )?;
    if a.intermediate.amount != 0 {
        return Ok(());
    }
    if a.token_program.key() == anchor_spl::token_2022::ID {
        let taxed = {
            let info = a.mint.to_account_info();
            let data = info.try_borrow_data()?;
            let mint =
                StateWithExtensions::<anchor_spl::token_2022::spl_token_2022::state::Mint>::unpack(
                    &data,
                )?;
            mint.get_extension::<TransferFeeConfig>().is_ok()
        };
        if taxed {
            crate::utils::harvest_withheld_tokens_to_mint(
                a.token_program.to_account_info(),
                a.mint.to_account_info(),
                a.intermediate.to_account_info(),
                None,
            )?;
        }
    }
    crate::utils::close_token_account(
        a.intermediate.to_account_info(),
        a.payer.to_account_info(),
        a.sa_authority.to_account_info(),
        a.token_program.to_account_info(),
        Some(SA_AUTHORITY_SEED),
    )
}

#[derive(Accounts)]
pub struct ArbitragePumpVolume<'info> {
    #[account(mut, address = close_authority::id() @ ErrorCode::InvalidSigner)]
    pub payer: Signer<'info>,
    /// CHECK: validated user and signing seeds in handler.
    #[account(mut)]
    pub user: AccountInfo<'info>,
    /// CHECK: PDA, owner and lifecycle checked in handler.
    #[account(mut, seeds = [b"user_volume_accumulator", user.key().as_ref()], bump, seeds::program = pump_program.key())]
    pub volume: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
    /// CHECK: canonical Pump event PDA.
    #[account(seeds = [b"__event_authority"], bump, seeds::program = pump_program.key())]
    pub event_authority: AccountInfo<'info>,
    /// CHECK: fixed executable CPI program.
    #[account(address = PUMP, executable)]
    pub pump_program: AccountInfo<'info>,
    /// CHECK: fixed sysvar.
    #[account(address = anchor_lang::solana_program::sysvar::instructions::ID)]
    pub instructions: AccountInfo<'info>,
}

fn allowed_user(a: &ArbitragePumpVolume) -> Result<()> {
    require!(
        a.user.key() == a.payer.key() || a.user.key() == authority_pda::id(),
        ErrorCode::InvalidSigner
    );
    require_keys_eq!(*a.user.owner, System::id(), ErrorCode::InvalidSigner);
    require!(a.user.data_is_empty(), ErrorCode::InvalidSigner);
    Ok(())
}

pub fn prepare_arbitrage_pump_volume_handler(ctx: Context<ArbitragePumpVolume>) -> Result<()> {
    let a = ctx.accounts;
    allowed_user(a)?;
    // Reject existing/prefunded accounts, even if the client view was stale.
    require!(
        a.volume.lamports() == 0 && a.volume.data_is_empty() && *a.volume.owner == System::id(),
        ErrorCode::InvalidTokenAccount
    );
    let ix = Instruction {
        program_id: PUMP,
        data: vec![94, 6, 202, 115, 255, 96, 232, 183],
        accounts: vec![
            AccountMeta::new(a.payer.key(), true),
            AccountMeta::new_readonly(a.user.key(), false),
            AccountMeta::new(a.volume.key(), false),
            AccountMeta::new_readonly(System::id(), false),
            AccountMeta::new_readonly(a.event_authority.key(), false),
            AccountMeta::new_readonly(PUMP, false),
        ],
    };
    invoke_signed(
        &ix,
        &[
            a.payer.to_account_info(),
            a.user.to_account_info(),
            a.volume.to_account_info(),
            a.system_program.to_account_info(),
            a.event_authority.to_account_info(),
            a.pump_program.to_account_info(),
        ],
        &[],
    )?;
    Ok(())
}

pub fn close_arbitrage_pump_volume_handler(ctx: Context<ArbitragePumpVolume>) -> Result<()> {
    let a = ctx.accounts;
    allowed_user(a)?;
    let disc =
        anchor_lang::solana_program::hash::hash(b"global:prepare_arbitrage_pump_volume").to_bytes();
    earlier_creation(
        &a.instructions,
        crate::ID,
        &disc[..8],
        &[
            a.payer.key(),
            a.user.key(),
            a.volume.key(),
            System::id(),
            a.event_authority.key(),
            PUMP,
            a.instructions.key(),
        ],
    )?;
    require_keys_eq!(*a.volume.owner, PUMP, ErrorCode::InvalidTokenAccount);
    {
        let data = a.volume.try_borrow_data()?;
        require!(
            data.len() == 137
                && data[..8] == [86, 255, 112, 14, 102, 53, 154, 250]
                && data[8..40] == a.user.key().to_bytes(),
            ErrorCode::InvalidTokenAccount
        );
    }
    let before = a.user.lamports();
    let rent = a.volume.lamports();
    let ix = Instruction {
        program_id: PUMP,
        data: vec![249, 69, 164, 218, 150, 103, 84, 138],
        accounts: vec![
            AccountMeta::new(a.user.key(), true),
            AccountMeta::new(a.volume.key(), false),
            AccountMeta::new_readonly(a.event_authority.key(), false),
            AccountMeta::new_readonly(PUMP, false),
        ],
    };
    invoke_signed(
        &ix,
        &[
            a.user.to_account_info(),
            a.volume.to_account_info(),
            a.event_authority.to_account_info(),
            a.pump_program.to_account_info(),
        ],
        SA_AUTHORITY_SEED,
    )?;
    let refund = a
        .user
        .lamports()
        .checked_sub(before)
        .ok_or(ErrorCode::InvalidTokenAccount)?;
    require!(
        refund == rent && a.volume.lamports() == 0,
        ErrorCode::InvalidTokenAccount
    );
    if a.user.key() != a.payer.key() {
        crate::utils::transfer_sol(
            a.user.to_account_info(),
            a.payer.to_account_info(),
            refund,
            Some(SA_AUTHORITY_SEED),
        )?;
    }
    Ok(())
}
