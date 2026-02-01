use super::common::DexProcessor;
use crate::adapters::common::{before_check, invoke_process};
use crate::error::ErrorCode;
use crate::{DEPOSIT_SELECTOR, HopAccounts, REDEEM_SELECTOR, jupiter_lend_program};
use anchor_lang::{prelude::*, solana_program::instruction::Instruction};
use anchor_spl::token_interface::TokenAccount;
use arrayref::array_ref;

const ARGS_LEN: usize = 16;

const DEPOSIT_ACCOUNTS_LEN: usize = 18;
const DEPOSIT_CPI_ACCOUNTS_LEN: usize = 17;

const REDEEM_ACCOUNTS_LEN: usize = 19;
const REDEEM_CPI_ACCOUNTS_LEN: usize = 18;

struct JupiterLendProcessor;
impl DexProcessor for JupiterLendProcessor {}

pub struct DepositAccounts<'info> {
    pub dex_program_id: &'info AccountInfo<'info>,
    pub swap_authority_pubkey: &'info AccountInfo<'info>,
    pub swap_source_token: InterfaceAccount<'info, TokenAccount>,
    pub swap_destination_token: InterfaceAccount<'info, TokenAccount>,

    pub mint: &'info AccountInfo<'info>,
    pub lending_admin: &'info AccountInfo<'info>,
    pub lending: &'info AccountInfo<'info>,
    pub f_token_mint: &'info AccountInfo<'info>,
    pub token_reserves: &'info AccountInfo<'info>,
    pub supply_position: &'info AccountInfo<'info>,
    pub rate_model: &'info AccountInfo<'info>,
    pub vault: &'info AccountInfo<'info>,
    pub liquidity: &'info AccountInfo<'info>,
    pub liquidity_program: &'info AccountInfo<'info>,
    pub rewards_rate_model: &'info AccountInfo<'info>,
    pub token_program: &'info AccountInfo<'info>,
    pub associated_token_program: &'info AccountInfo<'info>,
    pub system_program: &'info AccountInfo<'info>,
}

impl<'info> DepositAccounts<'info> {
    fn parse_accounts(accounts: &'info [AccountInfo<'info>], offset: usize) -> Result<Self> {
        let [
            dex_program_id,
            swap_authority_pubkey,
            swap_source_token,
            swap_destination_token,
            mint,
            lending_admin,
            lending,
            f_token_mint,
            token_reserves,
            supply_position,
            rate_model,
            vault,
            liquidity,
            liquidity_program,
            rewards_rate_model,
            token_program,
            associated_token_program,
            system_program,
        ]: &[AccountInfo<'info>; DEPOSIT_ACCOUNTS_LEN] =
            array_ref![accounts, offset, DEPOSIT_ACCOUNTS_LEN];

        Ok(Self {
            dex_program_id,
            swap_authority_pubkey,
            swap_source_token: InterfaceAccount::try_from(swap_source_token)?,
            swap_destination_token: InterfaceAccount::try_from(swap_destination_token)?,
            mint,
            lending_admin,
            lending,
            f_token_mint,
            token_reserves,
            supply_position,
            rate_model,
            vault,
            liquidity,
            liquidity_program,
            rewards_rate_model,
            token_program,
            associated_token_program,
            system_program,
        })
    }
}

pub fn deposit_handler<'a>(
    remaining_accounts: &'a [AccountInfo<'a>],
    amount_in: u64,
    offset: &mut usize,
    hop_accounts: &mut HopAccounts,
    hop: usize,
    proxy_swap: bool,
    owner_seeds: Option<&[&[&[u8]]]>,
) -> Result<u64> {
    msg!("Dex::JupiterLendDeposit amount_in: {}, offset: {}", amount_in, offset);
    require!(
        remaining_accounts.len() >= *offset + DEPOSIT_ACCOUNTS_LEN,
        ErrorCode::InvalidAccountsLength
    );
    let mut swap_accounts = DepositAccounts::parse_accounts(remaining_accounts, *offset)?;
    if swap_accounts.dex_program_id.key != &jupiter_lend_program::id() {
        return Err(ErrorCode::InvalidProgramId.into());
    }
    swap_accounts.lending.key().log(); // log pool address

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
    data.extend_from_slice(DEPOSIT_SELECTOR);
    data.extend_from_slice(&amount_in.to_le_bytes()); // assets amount_in

    let mut accounts = Vec::with_capacity(DEPOSIT_CPI_ACCOUNTS_LEN);
    accounts.push(AccountMeta::new(swap_accounts.swap_authority_pubkey.key(), true)); // signer
    accounts.push(AccountMeta::new(swap_accounts.swap_source_token.key(), false)); // depositor_token_account (mut)
    accounts.push(AccountMeta::new(swap_accounts.swap_destination_token.key(), false)); // recipient_token_account (mut)
    accounts.push(AccountMeta::new_readonly(swap_accounts.mint.key(), false)); // mint (readonly)
    accounts.push(AccountMeta::new_readonly(swap_accounts.lending_admin.key(), false)); // lending_admin (readonly)
    accounts.push(AccountMeta::new(swap_accounts.lending.key(), false)); // lending (mut)
    accounts.push(AccountMeta::new(swap_accounts.f_token_mint.key(), false)); // f_token_mint (mut)
    accounts.push(AccountMeta::new(swap_accounts.token_reserves.key(), false)); // supply_token_reserves_liquidity (mut)
    accounts.push(AccountMeta::new(swap_accounts.supply_position.key(), false)); // lending_supply_position_on_liquidity (mut)
    accounts.push(AccountMeta::new_readonly(swap_accounts.rate_model.key(), false)); // rate_model (readonly)
    accounts.push(AccountMeta::new(swap_accounts.vault.key(), false)); // vault (mut)
    accounts.push(AccountMeta::new(swap_accounts.liquidity.key(), false)); // liquidity (mut)
    accounts.push(AccountMeta::new(swap_accounts.liquidity_program.key(), false)); // liquidity_program (mut)
    accounts.push(AccountMeta::new_readonly(swap_accounts.rewards_rate_model.key(), false)); // rewards_rate_model (readonly)
    accounts.push(AccountMeta::new_readonly(swap_accounts.token_program.key(), false)); // token_program (readonly)
    accounts.push(AccountMeta::new_readonly(swap_accounts.associated_token_program.key(), false)); // associated_token_program (readonly)
    accounts.push(AccountMeta::new_readonly(swap_accounts.system_program.key(), false)); // system_program (readonly)

    let mut account_infos = arrayvec::ArrayVec::<_, DEPOSIT_CPI_ACCOUNTS_LEN>::new();
    account_infos.push(swap_accounts.swap_authority_pubkey.to_account_info());
    account_infos.push(swap_accounts.swap_source_token.to_account_info());
    account_infos.push(swap_accounts.swap_destination_token.to_account_info());
    account_infos.push(swap_accounts.mint.to_account_info());
    account_infos.push(swap_accounts.lending_admin.to_account_info());
    account_infos.push(swap_accounts.lending.to_account_info());
    account_infos.push(swap_accounts.f_token_mint.to_account_info());
    account_infos.push(swap_accounts.token_reserves.to_account_info());
    account_infos.push(swap_accounts.supply_position.to_account_info());
    account_infos.push(swap_accounts.rate_model.to_account_info());
    account_infos.push(swap_accounts.vault.to_account_info());
    account_infos.push(swap_accounts.liquidity.to_account_info());
    account_infos.push(swap_accounts.liquidity_program.to_account_info());
    account_infos.push(swap_accounts.rewards_rate_model.to_account_info());
    account_infos.push(swap_accounts.token_program.to_account_info());
    account_infos.push(swap_accounts.associated_token_program.to_account_info());
    account_infos.push(swap_accounts.system_program.to_account_info());

    let instruction =
        Instruction { program_id: swap_accounts.dex_program_id.key(), accounts, data };

    let amount_out = invoke_process(
        amount_in,
        &JupiterLendProcessor,
        &account_infos,
        &mut swap_accounts.swap_source_token,
        &mut swap_accounts.swap_destination_token,
        hop_accounts,
        instruction,
        hop,
        offset,
        DEPOSIT_ACCOUNTS_LEN,
        proxy_swap,
        owner_seeds,
    )?;

    Ok(amount_out)
}

pub struct RedeemAccounts<'info> {
    pub dex_program_id: &'info AccountInfo<'info>,
    pub swap_authority_pubkey: &'info AccountInfo<'info>,
    pub swap_source_token: InterfaceAccount<'info, TokenAccount>,
    pub swap_destination_token: InterfaceAccount<'info, TokenAccount>,

    pub lending_admin: &'info AccountInfo<'info>,
    pub lending: &'info AccountInfo<'info>,
    pub mint: &'info AccountInfo<'info>,
    pub f_token_mint: &'info AccountInfo<'info>,
    pub token_reserves: &'info AccountInfo<'info>,
    pub supply_position: &'info AccountInfo<'info>,
    pub rate_model: &'info AccountInfo<'info>,
    pub vault: &'info AccountInfo<'info>,
    pub claim_account: &'info AccountInfo<'info>,
    pub liquidity: &'info AccountInfo<'info>,
    pub liquidity_program: &'info AccountInfo<'info>,
    pub rewards_rate_model: &'info AccountInfo<'info>,
    pub token_program: &'info AccountInfo<'info>,
    pub associated_token_program: &'info AccountInfo<'info>,
    pub system_program: &'info AccountInfo<'info>,
}

impl<'info> RedeemAccounts<'info> {
    fn parse_accounts(accounts: &'info [AccountInfo<'info>], offset: usize) -> Result<Self> {
        let [
            dex_program_id,
            swap_authority_pubkey,
            swap_source_token,
            swap_destination_token,
            lending_admin,
            lending,
            mint,
            f_token_mint,
            token_reserves,
            supply_position,
            rate_model,
            vault,
            claim_account,
            liquidity,
            liquidity_program,
            rewards_rate_model,
            token_program,
            associated_token_program,
            system_program,
        ]: &[AccountInfo<'info>; REDEEM_ACCOUNTS_LEN] =
            array_ref![accounts, offset, REDEEM_ACCOUNTS_LEN];

        Ok(Self {
            dex_program_id,
            swap_authority_pubkey,
            swap_source_token: InterfaceAccount::try_from(swap_source_token)?,
            swap_destination_token: InterfaceAccount::try_from(swap_destination_token)?,
            lending_admin,
            lending,
            mint,
            f_token_mint,
            token_reserves,
            supply_position,
            rate_model,
            vault,
            claim_account,
            liquidity,
            liquidity_program,
            rewards_rate_model,
            token_program,
            associated_token_program,
            system_program,
        })
    }
}

pub fn redeem_handler<'a>(
    remaining_accounts: &'a [AccountInfo<'a>],
    amount_in: u64,
    offset: &mut usize,
    hop_accounts: &mut HopAccounts,
    hop: usize,
    proxy_swap: bool,
    owner_seeds: Option<&[&[&[u8]]]>,
) -> Result<u64> {
    msg!("Dex::JupiterLendRedeem amount_in: {}, offset: {}", amount_in, offset);
    require!(
        remaining_accounts.len() >= *offset + REDEEM_ACCOUNTS_LEN,
        ErrorCode::InvalidAccountsLength
    );
    let mut swap_accounts = RedeemAccounts::parse_accounts(remaining_accounts, *offset)?;
    if swap_accounts.dex_program_id.key != &jupiter_lend_program::id() {
        return Err(ErrorCode::InvalidProgramId.into());
    }
    swap_accounts.lending.key().log(); // log pool address

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
    data.extend_from_slice(REDEEM_SELECTOR);
    data.extend_from_slice(&amount_in.to_le_bytes()); // shares amount_in

    let mut accounts = Vec::with_capacity(REDEEM_CPI_ACCOUNTS_LEN);
    accounts.push(AccountMeta::new(swap_accounts.swap_authority_pubkey.key(), true)); // signer
    accounts.push(AccountMeta::new(swap_accounts.swap_source_token.key(), false)); // owner_token_account (mut)
    accounts.push(AccountMeta::new(swap_accounts.swap_destination_token.key(), false)); // recipient_token_account (mut)
    accounts.push(AccountMeta::new_readonly(swap_accounts.lending_admin.key(), false)); // lending_admin (readonly)
    accounts.push(AccountMeta::new(swap_accounts.lending.key(), false)); // lending (mut)
    accounts.push(AccountMeta::new_readonly(swap_accounts.mint.key(), false)); // mint (readonly)
    accounts.push(AccountMeta::new(swap_accounts.f_token_mint.key(), false)); // f_token_mint (mut)
    accounts.push(AccountMeta::new(swap_accounts.token_reserves.key(), false)); // supply_token_reserves_liquidity (mut)
    accounts.push(AccountMeta::new(swap_accounts.supply_position.key(), false)); // lending_supply_position_on_liquidity (mut)
    accounts.push(AccountMeta::new_readonly(swap_accounts.rate_model.key(), false)); // rate_model (readonly)
    accounts.push(AccountMeta::new(swap_accounts.vault.key(), false)); // vault (mut)
    accounts.push(AccountMeta::new(swap_accounts.claim_account.key(), false)); // claim_account (mut)
    accounts.push(AccountMeta::new(swap_accounts.liquidity.key(), false)); // liquidity (mut)
    accounts.push(AccountMeta::new(swap_accounts.liquidity_program.key(), false)); // liquidity_program (mut)
    accounts.push(AccountMeta::new_readonly(swap_accounts.rewards_rate_model.key(), false)); // rewards_rate_model (readonly)
    accounts.push(AccountMeta::new_readonly(swap_accounts.token_program.key(), false)); // token_program (readonly)
    accounts.push(AccountMeta::new_readonly(swap_accounts.associated_token_program.key(), false)); // associated_token_program (readonly)
    accounts.push(AccountMeta::new_readonly(swap_accounts.system_program.key(), false)); // system_program (readonly)

    let mut account_infos = arrayvec::ArrayVec::<_, REDEEM_CPI_ACCOUNTS_LEN>::new();
    account_infos.push(swap_accounts.swap_authority_pubkey.to_account_info());
    account_infos.push(swap_accounts.swap_source_token.to_account_info());
    account_infos.push(swap_accounts.swap_destination_token.to_account_info());
    account_infos.push(swap_accounts.lending_admin.to_account_info());
    account_infos.push(swap_accounts.lending.to_account_info());
    account_infos.push(swap_accounts.mint.to_account_info());
    account_infos.push(swap_accounts.f_token_mint.to_account_info());
    account_infos.push(swap_accounts.token_reserves.to_account_info());
    account_infos.push(swap_accounts.supply_position.to_account_info());
    account_infos.push(swap_accounts.rate_model.to_account_info());
    account_infos.push(swap_accounts.vault.to_account_info());
    account_infos.push(swap_accounts.claim_account.to_account_info());
    account_infos.push(swap_accounts.liquidity.to_account_info());
    account_infos.push(swap_accounts.liquidity_program.to_account_info());
    account_infos.push(swap_accounts.rewards_rate_model.to_account_info());
    account_infos.push(swap_accounts.token_program.to_account_info());
    account_infos.push(swap_accounts.associated_token_program.to_account_info());
    account_infos.push(swap_accounts.system_program.to_account_info());

    let instruction =
        Instruction { program_id: swap_accounts.dex_program_id.key(), accounts, data };

    let amount_out = invoke_process(
        amount_in,
        &JupiterLendProcessor,
        &account_infos,
        &mut swap_accounts.swap_source_token,
        &mut swap_accounts.swap_destination_token,
        hop_accounts,
        instruction,
        hop,
        offset,
        REDEEM_ACCOUNTS_LEN,
        proxy_swap,
        owner_seeds,
    )?;

    Ok(amount_out)
}
