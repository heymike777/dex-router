use crate::HopAccounts;
use crate::adapters::common::{before_check, invoke_process};
use crate::error::ErrorCode;
use crate::saber_stable_program;
use anchor_lang::{prelude::*, solana_program::instruction::Instruction};
use anchor_spl::token_interface::TokenAccount;
use arrayref::array_ref;

use super::common::DexProcessor;

const ARGS_LEN: usize = 17;

pub struct StableSwapProcessor;
impl DexProcessor for StableSwapProcessor {}

pub struct StableSwapAccounts<'info> {
    pub dex_program_id: &'info AccountInfo<'info>,
    pub swap_authority_pubkey: &'info AccountInfo<'info>,
    pub swap_source_token: InterfaceAccount<'info, TokenAccount>,
    pub swap_destination_token: InterfaceAccount<'info, TokenAccount>,

    pub swap_info: &'info AccountInfo<'info>,
    pub swap_authority: &'info AccountInfo<'info>,
    pub token_a_account: InterfaceAccount<'info, TokenAccount>,
    pub token_b_account: InterfaceAccount<'info, TokenAccount>,
    pub swap_admin_fee: &'info AccountInfo<'info>,
    pub token_program: &'info AccountInfo<'info>,
}
const ACCOUNTS_LEN: usize = 10;

impl<'info> StableSwapAccounts<'info> {
    fn parse_accounts(accounts: &'info [AccountInfo<'info>], offset: usize) -> Result<Self> {
        let [
            dex_program_id,
            swap_authority_pubkey,
            swap_source_token,
            swap_destination_token,
            swap_info,
            swap_authority,
            token_a_account,
            token_b_account,
            swap_admin_fee,
            token_program,
        ]: &[AccountInfo<'info>; ACCOUNTS_LEN] = array_ref![accounts, offset, ACCOUNTS_LEN];
        Ok(Self {
            dex_program_id,
            swap_authority_pubkey,
            swap_source_token: InterfaceAccount::try_from(swap_source_token)?,
            swap_destination_token: InterfaceAccount::try_from(swap_destination_token)?,
            swap_info,
            swap_authority,
            token_a_account: InterfaceAccount::try_from(token_a_account)?,
            token_b_account: InterfaceAccount::try_from(token_b_account)?,
            swap_admin_fee,
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
    msg!("Dex::StableSwap amount_in: {}, offset: {}", amount_in, offset);
    require!(remaining_accounts.len() >= *offset + ACCOUNTS_LEN, ErrorCode::InvalidAccountsLength);
    let mut swap_accounts = StableSwapAccounts::parse_accounts(remaining_accounts, *offset)?;
    if swap_accounts.dex_program_id.key != &saber_stable_program::id() {
        return Err(ErrorCode::InvalidProgramId.into());
    }
    // log pool address
    swap_accounts.swap_info.key().log();

    // check hop accounts & swap authority
    let swap_source_token = swap_accounts.swap_source_token.key();
    let swap_destination_token = swap_accounts.swap_destination_token.key();
    before_check(
        &swap_accounts.swap_authority_pubkey,
        &swap_accounts.swap_source_token,
        swap_destination_token,
        hop_accounts,
        hop,
        proxy_swap,
        owner_seeds,
    )?;

    let pool_source_pubkey;
    let pool_destination_pubkeyy;
    if (swap_accounts.swap_source_token.mint == swap_accounts.token_a_account.mint)
        && (swap_accounts.swap_destination_token.mint == swap_accounts.token_b_account.mint)
    {
        pool_source_pubkey = swap_accounts.token_a_account.key();
        pool_destination_pubkeyy = swap_accounts.token_b_account.key();
    } else if (swap_accounts.swap_source_token.mint == swap_accounts.token_b_account.mint)
        && (swap_accounts.swap_destination_token.mint == swap_accounts.token_a_account.mint)
    {
        pool_source_pubkey = swap_accounts.token_b_account.key();
        pool_destination_pubkeyy = swap_accounts.token_a_account.key();
    } else {
        return Err(ErrorCode::InvalidPool.into());
    }

    let mut data = Vec::with_capacity(ARGS_LEN);
    data.push(1);
    data.extend_from_slice(&amount_in.to_le_bytes());
    data.extend_from_slice(&1u64.to_le_bytes());

    let mut accounts = Vec::with_capacity(9);
    accounts.push(AccountMeta::new_readonly(swap_accounts.swap_info.key(), false));
    accounts.push(AccountMeta::new_readonly(swap_accounts.swap_authority.key(), false));
    accounts.push(AccountMeta::new_readonly(swap_accounts.swap_authority_pubkey.key(), true));
    accounts.push(AccountMeta::new(swap_source_token, false));
    accounts.push(AccountMeta::new(pool_source_pubkey, false));
    accounts.push(AccountMeta::new(pool_destination_pubkeyy, false));
    accounts.push(AccountMeta::new(swap_destination_token, false));
    accounts.push(AccountMeta::new(swap_accounts.swap_admin_fee.key(), false));
    accounts.push(AccountMeta::new_readonly(swap_accounts.token_program.key(), false));

    let account_infos = [
        swap_accounts.swap_info.to_account_info(),
        swap_accounts.swap_authority.to_account_info(),
        swap_accounts.swap_authority_pubkey.to_account_info(),
        swap_accounts.swap_source_token.to_account_info(),
        swap_accounts.token_a_account.to_account_info(),
        swap_accounts.token_b_account.to_account_info(),
        swap_accounts.swap_destination_token.to_account_info(),
        swap_accounts.swap_admin_fee.to_account_info(),
        swap_accounts.token_program.to_account_info(),
    ];

    let instruction =
        Instruction { program_id: swap_accounts.dex_program_id.key(), accounts, data };

    let dex_processor = &StableSwapProcessor;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_pack_swap_instruction() {
        let amount_in = 100u64;
        let mut data = Vec::with_capacity(ARGS_LEN);
        data.push(1);
        data.extend_from_slice(&amount_in.to_le_bytes());
        data.extend_from_slice(&1u64.to_le_bytes());

        msg!("data.len: {}", data.len());
        assert!(data.len() == ARGS_LEN);
    }
}
