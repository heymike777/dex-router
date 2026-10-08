//! Pool-only Pump multi-hop CPI. A segment has one source/destination and is
//! one hop to the outer atomic profit guard. Curve lamport flows are excluded.
use super::common::{before_check, invoke_process, DexProcessor};
use super::pumpfunamm::PumpfunammSellProcessor;
use crate::{authority_pda, error::ErrorCode, pumpfunamm_program, HopAccounts};
use anchor_lang::{prelude::*, solana_program::instruction::Instruction};
use anchor_spl::token_interface::TokenAccount;
const SELECTOR: [u8; 8] = [43, 100, 73, 19, 233, 246, 111, 148];
struct RentProcessor;
impl DexProcessor for RentProcessor {
    fn before_invoke(&self, infos: &[AccountInfo]) -> Result<u64> {
        // The multi-hop CPI places user at 0, whereas v2 places it at 1.
        PumpfunammSellProcessor.before_invoke(&[infos[11].clone(), infos[0].clone()])
    }
    fn after_invoke(
        &self,
        infos: &[AccountInfo],
        hop: usize,
        seeds: Option<&[&[&[u8]]]>,
        before: u64,
    ) -> Result<u64> {
        PumpfunammSellProcessor.after_invoke(
            &[
                infos[11].clone(),
                infos[0].clone(),
                infos.last().unwrap().clone(),
            ],
            hop,
            seeds,
            before,
        )
    }
}
fn cpi(keys: &[Pubkey], amount: u64) -> Instruction {
    let accounts = keys[1..]
        .iter()
        .enumerate()
        .map(|(i, key)| AccountMeta {
            pubkey: *key,
            is_signer: i == 0,
            is_writable: matches!(i, 0 | 1 | 2 | 5 | 6) || (i >= 16 && (i - 16) % 5 >= 2),
        })
        .collect();
    let mut data = SELECTOR.to_vec();
    data.extend(amount.to_le_bytes());
    data.extend(1u64.to_le_bytes());
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
    seeds: Option<&[&[&[u8]]]>,
    payer: Option<&AccountInfo<'a>>,
    length: usize,
) -> Result<u64> {
    require!((2..=4).contains(&length), ErrorCode::InvalidAccountsLength);
    let roles = 17 + 5 * length;
    let end = offset
        .checked_add(roles)
        .ok_or(ErrorCode::InvalidAccountsLength)?;
    let a = remaining
        .get(*offset..end)
        .ok_or(ErrorCode::InvalidAccountsLength)?;
    require_keys_eq!(
        a[0].key(),
        pumpfunamm_program::id(),
        ErrorCode::InvalidProgramId
    );
    require_keys_eq!(
        a[12].key(),
        pumpfunamm_program::id(),
        ErrorCode::InvalidProgramId
    );
    for group in a[17..].chunks_exact(5) {
        // Bonding-curve SOL does not settle through the source token account.
        require_keys_eq!(
            *group[2].owner,
            pumpfunamm_program::id(),
            ErrorCode::InvalidProgramId
        );
    }
    let mut source = InterfaceAccount::<TokenAccount>::try_from(&a[2])?;
    let mut target = InterfaceAccount::<TokenAccount>::try_from(&a[3])?;
    before_check(&a[1], &source, target.key(), hops, hop, proxy, seeds)?;
    let keys: Vec<Pubkey> = a.iter().map(|a| a.key()).collect();
    let ix = cpi(&keys, amount);
    let mut infos: Vec<AccountInfo> = a[1..].to_vec();
    if a[1].key() == authority_pda::ID {
        infos.push(payer.ok_or(ErrorCode::InvalidAccountsLength)?.clone());
    } else {
        infos.push(a[1].clone());
    }
    invoke_process(
        amount,
        &RentProcessor,
        &infos,
        &mut source,
        &mut target,
        hops,
        ix,
        hop,
        offset,
        roles,
        proxy,
        seeds,
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn roles_selector_and_tags_are_append_only() {
        for (length, dex) in [
            (2, crate::Dex::PumpMultiHop2),
            (3, crate::Dex::PumpMultiHop3),
            (4, crate::Dex::PumpMultiHop4),
        ] {
            assert_eq!(dex.try_to_vec().unwrap(), vec![104 + length as u8]);
            let keys: Vec<_> = (0..17 + 5 * length).map(|_| Pubkey::new_unique()).collect();
            let ix = cpi(&keys, 42);
            assert_eq!(ix.accounts.len(), 16 + 5 * length);
            assert_eq!(&ix.data[..8], &SELECTOR);
            assert_eq!(u64::from_le_bytes(ix.data[8..16].try_into().unwrap()), 42);
            for (i, meta) in ix.accounts.iter().enumerate() {
                assert_eq!(meta.pubkey, keys[i + 1]);
                assert_eq!(meta.is_signer, i == 0);
                assert_eq!(
                    meta.is_writable,
                    matches!(i, 0 | 1 | 2 | 5 | 6) || (i >= 16 && (i - 16) % 5 >= 2)
                );
            }
        }
    }
}
