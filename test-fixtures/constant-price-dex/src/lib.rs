//! TEST ONLY. Loaded at a Raydium address inside LiteSVM, never deployed.
//! Implements the adapter's swap-v2 account interface using real SPL transfers.
//! A u64 numerator in pool data controls output = input * numerator / 1000.
use anchor_lang::solana_program::{account_info::AccountInfo, entrypoint,
    entrypoint::ProgramResult, program::{invoke, invoke_signed}, pubkey::Pubkey};
use anchor_spl::token::spl_token;
entrypoint!(process_instruction);

pub fn process_instruction(program: &Pubkey, accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let amount = u64::from_le_bytes(data[1..9].try_into().unwrap());
    let ratio = u64::from_le_bytes(accounts[1].try_borrow_data()?[..8].try_into().unwrap());
    let output = amount.checked_mul(ratio).unwrap() / 1000;
    let (authority, bump) = Pubkey::find_program_address(&[b"fixture"], program);
    assert_eq!(accounts[2].key, &authority);
    invoke(&spl_token::instruction::transfer(accounts[0].key, accounts[5].key,
        accounts[3].key, accounts[7].key, &[], amount)?,
        &[accounts[5].clone(), accounts[3].clone(), accounts[7].clone(), accounts[0].clone()])?;
    invoke_signed(&spl_token::instruction::transfer(accounts[0].key, accounts[4].key,
        accounts[6].key, accounts[2].key, &[], output)?,
        &[accounts[4].clone(), accounts[6].clone(), accounts[2].clone(), accounts[0].clone()],
        &[&[b"fixture", &[bump]]])?;
    Ok(())
}
