//! PumpSwap's October 2026 compact trade ABI. Router roles are fixed (17);
//! old tags keep their v1 account layout. No optional remaining accounts.
use super::common::{before_check, invoke_process};
use super::pumpfunamm::PumpfunammSellProcessor;
use crate::{authority_pda, error::ErrorCode, pumpfunamm_program, HopAccounts};
use anchor_lang::{prelude::*, solana_program::instruction::Instruction};
use anchor_spl::token_interface::TokenAccount;

pub const BUY: [u8; 8] = [194, 171, 28, 70, 104, 77, 91, 47];
pub const SELL: [u8; 8] = [93, 246, 130, 60, 231, 233, 64, 178];
const ROLES: usize = 17;

// program, user, source, destination, pool, global, base mint, quote mint,
// base vault, quote vault, base program, quote program, system, user volume,
// fee config, buyback ATA, event authority.
fn cpi(keys: &[Pubkey; ROLES], buy: bool, amount: u64) -> Instruction {
    let order = [
        4,
        1,
        5,
        6,
        7,
        if buy { 3 } else { 2 },
        if buy { 2 } else { 3 },
        8,
        9,
        10,
        11,
        12,
        13,
        14,
        15,
        16,
        0,
    ];
    let accounts = order
        .iter()
        .enumerate()
        .map(|(i, role)| AccountMeta {
            pubkey: keys[*role],
            is_signer: i == 1,
            is_writable: matches!(i, 0 | 1 | 5 | 6 | 7 | 8 | 12 | 14),
        })
        .collect();
    let mut data = Vec::with_capacity(24);
    data.extend_from_slice(if buy { &BUY } else { &SELL });
    data.extend_from_slice(&amount.to_le_bytes());
    data.extend_from_slice(&1u64.to_le_bytes()); // outer router enforces final min_return
    Instruction {
        program_id: keys[0],
        accounts,
        data,
    }
}

#[allow(clippy::too_many_arguments)]
pub fn trade<'a>(
    remaining: &'a [AccountInfo<'a>],
    amount: u64,
    offset: &mut usize,
    hops: &mut HopAccounts,
    hop: usize,
    proxy: bool,
    owner_seeds: Option<&[&[&[u8]]]>,
    payer: Option<&AccountInfo<'a>>,
    buy: bool,
) -> Result<u64> {
    let end = offset
        .checked_add(ROLES)
        .ok_or(ErrorCode::InvalidAccountsLength)?;
    let a = remaining
        .get(*offset..end)
        .ok_or(ErrorCode::InvalidAccountsLength)?;
    require_keys_eq!(
        a[0].key(),
        pumpfunamm_program::id(),
        ErrorCode::InvalidProgramId
    );
    let mut source = InterfaceAccount::<TokenAccount>::try_from(&a[2])?;
    let mut destination = InterfaceAccount::<TokenAccount>::try_from(&a[3])?;
    before_check(
        &a[1],
        &source,
        destination.key(),
        hops,
        hop,
        proxy,
        owner_seeds,
    )?;
    let keys: [Pubkey; ROLES] = std::array::from_fn(|i| a[i].key());
    let instruction = cpi(&keys, buy, amount);
    let order = [
        4,
        1,
        5,
        6,
        7,
        if buy { 3 } else { 2 },
        if buy { 2 } else { 3 },
        8,
        9,
        10,
        11,
        12,
        13,
        14,
        15,
        16,
        0,
    ];
    let mut infos: Vec<AccountInfo> = order.iter().map(|i| a[*i].clone()).collect();
    // Both v2 directions may create the volume PDA or top up old pool rent.
    // Charge that cost to the payer, using the existing bounded rent hook.
    if a[1].key() == authority_pda::ID {
        infos.push(payer.ok_or(ErrorCode::InvalidAccountsLength)?.clone());
    } else {
        infos.push(a[1].clone());
    }
    invoke_process(
        amount,
        &PumpfunammSellProcessor,
        &infos,
        &mut source,
        &mut destination,
        hops,
        instruction,
        hop,
        offset,
        ROLES,
        proxy,
        owner_seeds,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instructions::Dex;
    #[test]
    fn v2_tags_append_without_changing_legacy_and_idl_roles_match() {
        assert_eq!(Dex::PumpfunammBuy3.try_to_vec().unwrap(), vec![73]);
        assert_eq!(Dex::PumpfunammSell3.try_to_vec().unwrap(), vec![74]);
        assert_eq!(Dex::PumpfunammBuyV2.try_to_vec().unwrap(), vec![104]);
        assert_eq!(Dex::PumpfunammSellV2.try_to_vec().unwrap(), vec![105]);
        let keys = std::array::from_fn(|_| Pubkey::new_unique());
        for buy in [true, false] {
            let ix = cpi(&keys, buy, 12345);
            assert_eq!(ix.accounts.len(), 17);
            assert_eq!(ix.data.len(), 24);
            assert_eq!(&ix.data[..8], if buy { &BUY } else { &SELL });
            assert_eq!(&ix.data[8..16], &12345u64.to_le_bytes());
            assert_eq!(&ix.data[16..], &1u64.to_le_bytes());
            assert_eq!(ix.accounts[5].pubkey, keys[if buy { 3 } else { 2 }]);
            assert_eq!(ix.accounts[6].pubkey, keys[if buy { 2 } else { 3 }]);
            for (i, meta) in ix.accounts.iter().enumerate() {
                assert_eq!(meta.is_signer, i == 1);
                assert_eq!(
                    meta.is_writable,
                    matches!(i, 0 | 1 | 5 | 6 | 7 | 8 | 12 | 14)
                );
            }
            assert_eq!(ix.accounts[16].pubkey, ix.program_id);
        }
    }
}
