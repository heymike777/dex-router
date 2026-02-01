use crate::error::ErrorCode;
use crate::HopAccounts;
use anchor_lang::prelude::*;

pub fn swap<'a>(
    _remaining_accounts: &'a [AccountInfo<'a>],
    _amount_in: u64,
    _offset: &mut usize,
    _hop_accounts: &mut HopAccounts,
    _hop: usize,
    _proxy_swap: bool,
    _owner_seeds: Option<&[&[&[u8]]]>,
) -> Result<u64> {
    msg!("Dex::Humidifi ABORT");
    require!(true == false, ErrorCode::AdapterAbort);
    Ok(0)
}

pub fn swap2<'a>(
    _remaining_accounts: &'a [AccountInfo<'a>],
    _amount_in: u64,
    _offset: &mut usize,
    _hop_accounts: &mut HopAccounts,
    _hop: usize,
    _proxy_swap: bool,
    _owner_seeds: Option<&[&[&[u8]]]>,
    _swap_id: u64,
) -> Result<u64> {
    msg!("Dex::HumidifiSwap2 ABORT");
    require!(true == false, ErrorCode::AdapterAbort);
    Ok(0)
}
