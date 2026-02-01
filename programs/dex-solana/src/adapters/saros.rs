use super::common::DexProcessor;
use crate::adapters::common::{before_check, invoke_process};
use crate::error::ErrorCode;
use crate::{HopAccounts, SWAP_SELECTOR, saros_dlmm_program, saros_program};
use anchor_lang::{prelude::*, solana_program::instruction::Instruction};
use anchor_spl::token_interface::TokenAccount;
use arrayref::array_ref;

pub struct SarosProcessor;
impl DexProcessor for SarosProcessor {}

const ARGS_LEN: usize = 17;

pub struct SarosAccounts<'info> {
    pub dex_program_id: &'info AccountInfo<'info>,
    pub swap_authority_pubkey: &'info AccountInfo<'info>,
    pub swap_source_token: InterfaceAccount<'info, TokenAccount>,
    pub swap_destination_token: InterfaceAccount<'info, TokenAccount>,

    pub pool: &'info AccountInfo<'info>,
    pub pool_authority: &'info AccountInfo<'info>,
    pub pool_token_in: &'info AccountInfo<'info>,
    pub pool_token_out: &'info AccountInfo<'info>,
    pub pool_lp_token_mint: &'info AccountInfo<'info>,
    pub protocol_lp_token: &'info AccountInfo<'info>,
    pub token_program: &'info AccountInfo<'info>,
}
const ACCOUNTS_LEN: usize = 11;

impl<'info> SarosAccounts<'info> {
    fn parse_accounts(accounts: &'info [AccountInfo<'info>], offset: usize) -> Result<Self> {
        let [
            dex_program_id,
            swap_authority_pubkey,
            swap_source_token,
            swap_destination_token,
            pool,
            pool_authority,
            pool_token_in,
            pool_token_out,
            pool_lp_token_mint,
            protocol_lp_token,
            token_program,
        ]: &[AccountInfo<'info>; ACCOUNTS_LEN] = array_ref![accounts, offset, ACCOUNTS_LEN];
        Ok(Self {
            dex_program_id,
            swap_authority_pubkey,
            swap_source_token: InterfaceAccount::try_from(swap_source_token)?,
            swap_destination_token: InterfaceAccount::try_from(swap_destination_token)?,
            pool,
            pool_authority,
            pool_token_in,
            pool_token_out,
            pool_lp_token_mint,
            protocol_lp_token,
            token_program,
        })
    }
}

pub fn swap<'a>(
    remaining_accounts: &'a [AccountInfo<'a>],
    amount_in: u64,
    offset: &mut usize,
    hop_accounts: &mut HopAccounts,
    hop: usize,
    proxy_swap: bool,
    owner_seeds: Option<&[&[&[u8]]]>,
) -> Result<u64> {
    msg!("Dex::Saros amount_in: {}, offset: {}", amount_in, offset);
    require!(remaining_accounts.len() >= *offset + ACCOUNTS_LEN, ErrorCode::InvalidAccountsLength);
    let mut swap_accounts = SarosAccounts::parse_accounts(remaining_accounts, *offset)?;
    if swap_accounts.dex_program_id.key != &saros_program::id() {
        return Err(ErrorCode::InvalidProgramId.into());
    }
    // log pool address
    swap_accounts.pool.key().log();

    // Check hop accounts & swap authority
    before_check(
        &swap_accounts.swap_authority_pubkey,
        &swap_accounts.swap_source_token,
        swap_accounts.swap_destination_token.key(),
        hop_accounts,
        hop,
        proxy_swap,
        owner_seeds,
    )?;

    let mut data = Vec::with_capacity(ARGS_LEN);
    data.extend_from_slice(&[1u8]); // instruction: 1 = swap
    data.extend_from_slice(&amount_in.to_le_bytes()); // amountIn
    data.extend_from_slice(&1u64.to_le_bytes()); // minimumAmountOut = 1

    // Accounts for Instruction
    let mut accounts = Vec::with_capacity(10);
    accounts.push(AccountMeta::new_readonly(swap_accounts.pool.key(), false));
    accounts.push(AccountMeta::new_readonly(swap_accounts.pool_authority.key(), false));
    accounts.push(AccountMeta::new_readonly(swap_accounts.swap_authority_pubkey.key(), true));
    accounts.push(AccountMeta::new(swap_accounts.swap_source_token.key(), false));
    accounts.push(AccountMeta::new(swap_accounts.pool_token_in.key(), false));
    accounts.push(AccountMeta::new(swap_accounts.pool_token_out.key(), false));
    accounts.push(AccountMeta::new(swap_accounts.swap_destination_token.key(), false));
    accounts.push(AccountMeta::new(swap_accounts.pool_lp_token_mint.key(), false));
    accounts.push(AccountMeta::new(swap_accounts.protocol_lp_token.key(), false));
    accounts.push(AccountMeta::new_readonly(swap_accounts.token_program.key(), false));

    // Accounts for pre & post invoke
    let account_infos = [
        swap_accounts.pool.to_account_info(),
        swap_accounts.pool_authority.to_account_info(),
        swap_accounts.swap_authority_pubkey.to_account_info(),
        swap_accounts.swap_source_token.to_account_info(),
        swap_accounts.pool_token_in.to_account_info(),
        swap_accounts.pool_token_out.to_account_info(),
        swap_accounts.swap_destination_token.to_account_info(),
        swap_accounts.pool_lp_token_mint.to_account_info(),
        swap_accounts.protocol_lp_token.to_account_info(),
        swap_accounts.token_program.to_account_info(),
    ];

    let instruction =
        Instruction { program_id: swap_accounts.dex_program_id.key(), accounts, data };

    let dex_processor = &SarosProcessor;
    let amount_out = invoke_process(
        amount_in,
        dex_processor,
        &account_infos,
        &mut swap_accounts.swap_source_token,
        &mut swap_accounts.swap_destination_token,
        hop_accounts,
        instruction,
        hop,
        offset,
        ACCOUNTS_LEN,
        proxy_swap,
        owner_seeds,
    )?;

    Ok(amount_out)
}

// DLMM
pub struct SarosDlmmProcessor;
impl DexProcessor for SarosDlmmProcessor {}

const ARGS_LEN_DLMM: usize = 26;

pub struct SarosDlmmAccounts<'info> {
    pub dex_program_id: &'info AccountInfo<'info>,
    pub swap_authority_pubkey: &'info AccountInfo<'info>,
    pub swap_source_token: InterfaceAccount<'info, TokenAccount>,
    pub swap_destination_token: InterfaceAccount<'info, TokenAccount>,

    pub pair: &'info AccountInfo<'info>,
    pub token_mint_x: &'info AccountInfo<'info>,
    pub token_mint_y: &'info AccountInfo<'info>,
    pub bin_array_lower: &'info AccountInfo<'info>,
    pub bin_array_upper: &'info AccountInfo<'info>,
    pub token_vault_x: &'info AccountInfo<'info>,
    pub token_vault_y: &'info AccountInfo<'info>,
    pub token_program_x: &'info AccountInfo<'info>,
    pub token_program_y: &'info AccountInfo<'info>,
    pub memo_program: &'info AccountInfo<'info>,
    pub pair_hook: &'info AccountInfo<'info>,
    pub rewarder_hook: &'info AccountInfo<'info>,
    pub event_authority: &'info AccountInfo<'info>,
    pub hook_bin_array_lower: &'info AccountInfo<'info>,
    pub hook_bin_array_upper: &'info AccountInfo<'info>,
}

const DLMM_ACCOUNTS_LEN: usize = 19;

impl<'info> SarosDlmmAccounts<'info> {
    fn parse_accounts(accounts: &'info [AccountInfo<'info>], offset: usize) -> Result<Self> {
        let [
            dex_program_id,
            swap_authority_pubkey,
            swap_source_token,
            swap_destination_token,
            pair,
            token_mint_x,
            token_mint_y,
            bin_array_lower,
            bin_array_upper,
            token_vault_x,
            token_vault_y,
            token_program_x,
            token_program_y,
            memo_program,
            pair_hook,
            rewarder_hook,
            event_authority,
            hook_bin_array_lower,
            hook_bin_array_upper,
        ]: &[AccountInfo<'info>; DLMM_ACCOUNTS_LEN] =
            array_ref![accounts, offset, DLMM_ACCOUNTS_LEN];
        Ok(Self {
            dex_program_id,
            swap_authority_pubkey,
            swap_source_token: InterfaceAccount::try_from(swap_source_token)?,
            swap_destination_token: InterfaceAccount::try_from(swap_destination_token)?,
            pair,
            token_mint_x,
            token_mint_y,
            bin_array_lower,
            bin_array_upper,
            token_vault_x,
            token_vault_y,
            token_program_x,
            token_program_y,
            memo_program,
            pair_hook,
            rewarder_hook,
            event_authority,
            hook_bin_array_lower,
            hook_bin_array_upper,
        })
    }
}

pub fn dlmm_swap<'a>(
    remaining_accounts: &'a [AccountInfo<'a>],
    amount_in: u64,
    offset: &mut usize,
    hop_accounts: &mut HopAccounts,
    hop: usize,
    proxy_swap: bool,
    owner_seeds: Option<&[&[&[u8]]]>,
) -> Result<u64> {
    msg!("Dex::Saros DLMM amount_in: {}, offset: {}", amount_in, offset);
    require!(
        remaining_accounts.len() >= *offset + DLMM_ACCOUNTS_LEN,
        ErrorCode::InvalidAccountsLength
    );
    let mut swap_accounts = SarosDlmmAccounts::parse_accounts(remaining_accounts, *offset)?;
    if swap_accounts.dex_program_id.key != &saros_dlmm_program::id() {
        return Err(ErrorCode::InvalidProgramId.into());
    }
    // log pool address
    swap_accounts.pair.key().log();

    // Check hop accounts & swap authority
    before_check(
        &swap_accounts.swap_authority_pubkey,
        &swap_accounts.swap_source_token,
        swap_accounts.swap_destination_token.key(),
        hop_accounts,
        hop,
        proxy_swap,
        owner_seeds,
    )?;
    let direction = swap_accounts.swap_source_token.mint == swap_accounts.token_mint_x.key();

    let (source_token_account, destination_token_account) = if direction {
        (swap_accounts.swap_source_token.clone(), swap_accounts.swap_destination_token.clone())
    } else {
        (swap_accounts.swap_destination_token.clone(), swap_accounts.swap_source_token.clone())
    };

    let mut data = Vec::with_capacity(ARGS_LEN_DLMM);
    data.extend_from_slice(SWAP_SELECTOR);
    data.extend_from_slice(&amount_in.to_le_bytes());
    data.extend_from_slice(&1u64.to_le_bytes()); // minimumAmountOut = 1
    data.extend_from_slice(&(direction as u8).to_le_bytes()); // swap for y
    data.extend_from_slice(&0u8.to_le_bytes()); // EXACT IN

    // Accounts for Instruction
    let mut accounts = Vec::with_capacity(19);
    accounts.push(AccountMeta::new(swap_accounts.pair.key(), false));
    accounts.push(AccountMeta::new_readonly(swap_accounts.token_mint_x.key(), false));
    accounts.push(AccountMeta::new_readonly(swap_accounts.token_mint_y.key(), false));
    accounts.push(AccountMeta::new(swap_accounts.bin_array_lower.key(), false));
    accounts.push(AccountMeta::new(swap_accounts.bin_array_upper.key(), false));
    accounts.push(AccountMeta::new(swap_accounts.token_vault_x.key(), false));
    accounts.push(AccountMeta::new(swap_accounts.token_vault_y.key(), false));
    accounts.push(AccountMeta::new(source_token_account.key(), false));
    accounts.push(AccountMeta::new(destination_token_account.key(), false));
    accounts.push(AccountMeta::new(swap_accounts.swap_authority_pubkey.key(), true));
    accounts.push(AccountMeta::new_readonly(swap_accounts.token_program_x.key(), false));
    accounts.push(AccountMeta::new_readonly(swap_accounts.token_program_y.key(), false));
    accounts.push(AccountMeta::new_readonly(swap_accounts.memo_program.key(), false));
    accounts.push(AccountMeta::new(swap_accounts.pair_hook.key(), false));
    accounts.push(AccountMeta::new_readonly(swap_accounts.rewarder_hook.key(), false));
    accounts.push(AccountMeta::new_readonly(swap_accounts.event_authority.key(), false));
    accounts.push(AccountMeta::new_readonly(swap_accounts.dex_program_id.key(), false));

    // Accounts for pre & post invoke
    let mut account_infos = arrayvec::ArrayVec::<_, 19>::new();
    account_infos.push(swap_accounts.pair.to_account_info());
    account_infos.push(swap_accounts.token_mint_x.to_account_info());
    account_infos.push(swap_accounts.token_mint_y.to_account_info());
    account_infos.push(swap_accounts.bin_array_lower.to_account_info());
    account_infos.push(swap_accounts.bin_array_upper.to_account_info());
    account_infos.push(swap_accounts.token_vault_x.to_account_info());
    account_infos.push(swap_accounts.token_vault_y.to_account_info());
    account_infos.push(swap_accounts.swap_source_token.to_account_info());
    account_infos.push(swap_accounts.swap_destination_token.to_account_info());
    account_infos.push(swap_accounts.swap_authority_pubkey.to_account_info());
    account_infos.push(swap_accounts.token_program_x.to_account_info());
    account_infos.push(swap_accounts.token_program_y.to_account_info());
    account_infos.push(swap_accounts.memo_program.to_account_info());
    account_infos.push(swap_accounts.pair_hook.to_account_info());
    account_infos.push(swap_accounts.rewarder_hook.to_account_info());
    account_infos.push(swap_accounts.event_authority.to_account_info());
    account_infos.push(swap_accounts.dex_program_id.to_account_info());

    let (pair, pair_hook) = (swap_accounts.pair.key(), swap_accounts.pair_hook.key());
    if pair_hook != pair {
        accounts.push(AccountMeta::new(swap_accounts.hook_bin_array_lower.key(), false));
        accounts.push(AccountMeta::new(swap_accounts.hook_bin_array_upper.key(), false));

        account_infos.push(swap_accounts.hook_bin_array_lower.to_account_info());
        account_infos.push(swap_accounts.hook_bin_array_upper.to_account_info());
    }

    let instruction =
        Instruction { program_id: swap_accounts.dex_program_id.key(), accounts, data };

    let dex_processor = &SarosDlmmProcessor;
    let amount_out = invoke_process(
        amount_in,
        dex_processor,
        &account_infos,
        &mut swap_accounts.swap_source_token,
        &mut swap_accounts.swap_destination_token,
        hop_accounts,
        instruction,
        hop,
        offset,
        DLMM_ACCOUNTS_LEN,
        proxy_swap,
        owner_seeds,
    )?;

    Ok(amount_out)
}
