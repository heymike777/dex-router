use crate::adapters::common::{before_check, invoke_process, invoke_processes};
use crate::error::ErrorCode;
use crate::{HopAccounts, lido_sol_mint, marinade_sol_mint, sanctum_router_program, wsol_program};
use anchor_lang::{prelude::*, solana_program::instruction::Instruction};
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};
use arrayref::array_ref;
use arrayvec::ArrayVec;

use super::common::DexProcessor;
pub struct SanctumRouterProcessor;
impl DexProcessor for SanctumRouterProcessor {}

pub const PREFUND_WITHDRAW_STAKE_IX_ACCOUNTS_LEN: usize = 17;
pub const SPL_STAKEPOOL_WITHDRAW_STAKE_ACCOUNTS_LEN: usize = 10;
pub const LIDO_WITHDRAW_STAKE_IX_ACCOUNTS_LEN: usize = 10;
pub const SPL_STAKEPOOL_WITHDRAW_SOL_ACCOUNTS_LEN: usize = 9;
pub const SPL_STAKEPOOL_DEPOSIT_ACCOUNTS_LEN: usize = 12;
pub const SPL_STAKE_POOL_DEPOSIT_SOL_IX_ACCOUNTS_LEN: usize = 5;
pub const DEPOSIT_STAKE_IX_ACCOUNTS_LEN: usize = 6;
pub const STAKE_WRAPPED_SOL_IX_ACCOUNTS_LEN: usize = 11;
pub const MARINADE_DEPOSIT_SOL_IX_ACCOUNTS_LEN: usize = 7;
pub const MARINADE_DEPOSIT_STAKE_IX_ACCOUNTS_LEN: usize = 11;
pub const WITHDRAW_WRAPPED_SOL_IX_ACCOUNTS_LEN: usize = 8;

#[derive(AnchorDeserialize, AnchorSerialize)]
pub struct SwapViaStakeArgs {
    pub amount: u64,
    pub bridge_stake_seed: u32,
}

pub trait StakeDexAccounts<'info> {
    fn append_accountmetas(&self, metas: &mut Vec<AccountMeta>);
    fn append_accountinfos<I: Extend<AccountInfo<'info>>>(&self, infos: &mut I);
    fn accountmetas_len(&self) -> usize;
}

enum StakeDexAccountsEnum<'info> {
    MarinadeSolDeposit(MarinadeSolDeposit<'info>),
    SplSolDeposit(SplSolDeposit<'info>),
    LidoWithdrawStake(LidoWithdrawStake<'info>),
    SplStakePoolWithdrawStake(SplStakePoolWithdrawStake<'info>),
    MarinadeStakeDeposit(MarinadeStakeDeposit<'info>),
    SplStakeDeposit(SplStakeDeposit<'info>),
    SplStakePoolWithdrawSol(SplStakePoolWithdrawSol<'info>),
}

impl<'info> StakeDexAccounts<'info> for StakeDexAccountsEnum<'info> {
    fn append_accountmetas(&self, metas: &mut Vec<AccountMeta>) {
        match self {
            StakeDexAccountsEnum::MarinadeSolDeposit(accounts) => {
                accounts.append_accountmetas(metas)
            }
            StakeDexAccountsEnum::SplSolDeposit(accounts) => accounts.append_accountmetas(metas),
            StakeDexAccountsEnum::LidoWithdrawStake(accounts) => {
                accounts.append_accountmetas(metas)
            }
            StakeDexAccountsEnum::SplStakePoolWithdrawStake(accounts) => {
                accounts.append_accountmetas(metas)
            }
            StakeDexAccountsEnum::MarinadeStakeDeposit(accounts) => {
                accounts.append_accountmetas(metas)
            }
            StakeDexAccountsEnum::SplStakeDeposit(accounts) => accounts.append_accountmetas(metas),
            StakeDexAccountsEnum::SplStakePoolWithdrawSol(accounts) => {
                accounts.append_accountmetas(metas)
            }
        }
    }

    fn append_accountinfos<I: Extend<AccountInfo<'info>>>(&self, infos: &mut I) {
        match self {
            StakeDexAccountsEnum::MarinadeSolDeposit(accounts) => {
                accounts.append_accountinfos(infos)
            }
            StakeDexAccountsEnum::SplSolDeposit(accounts) => accounts.append_accountinfos(infos),
            StakeDexAccountsEnum::LidoWithdrawStake(accounts) => {
                accounts.append_accountinfos(infos)
            }
            StakeDexAccountsEnum::SplStakePoolWithdrawStake(accounts) => {
                accounts.append_accountinfos(infos)
            }
            StakeDexAccountsEnum::MarinadeStakeDeposit(accounts) => {
                accounts.append_accountinfos(infos)
            }
            StakeDexAccountsEnum::SplStakeDeposit(accounts) => accounts.append_accountinfos(infos),
            StakeDexAccountsEnum::SplStakePoolWithdrawSol(accounts) => {
                accounts.append_accountinfos(infos)
            }
        }
    }

    fn accountmetas_len(&self) -> usize {
        match self {
            StakeDexAccountsEnum::MarinadeSolDeposit(accounts) => accounts.accountmetas_len(),
            StakeDexAccountsEnum::SplSolDeposit(accounts) => accounts.accountmetas_len(),
            StakeDexAccountsEnum::LidoWithdrawStake(accounts) => accounts.accountmetas_len(),
            StakeDexAccountsEnum::SplStakePoolWithdrawStake(accounts) => {
                accounts.accountmetas_len()
            }
            StakeDexAccountsEnum::MarinadeStakeDeposit(accounts) => accounts.accountmetas_len(),
            StakeDexAccountsEnum::SplStakeDeposit(accounts) => accounts.accountmetas_len(),
            StakeDexAccountsEnum::SplStakePoolWithdrawSol(accounts) => accounts.accountmetas_len(),
        }
    }
}

pub struct SanctumStakeWsol<'info> {
    pub dex_program_id: &'info AccountInfo<'info>,
    ///The authority of wsol_account
    pub swap_authority_pubkey: &'info AccountInfo<'info>,
    ///The wrapped SOL token account to stake wrapped SOL from
    pub wsol_from: InterfaceAccount<'info, TokenAccount>,
    ///The liquid staked SOL token account to receive the resulting tokens
    pub dest_token_to: InterfaceAccount<'info, TokenAccount>,
    ///The PDA that serves as the wSOL account to bridge user's wSOL to SOL. Pubkey::create_with_seed(). base = sol_bridge_out.pubkey, seed = 'wsol_bridge_in'. owner = token_program
    pub wsol_bridge_in: &'info AccountInfo<'info>,
    ///The PDA that serves as the system account to bridge user's wSOL to SOL. Seeds = ['sol_bridge_out']
    pub sol_bridge_out: &'info AccountInfo<'info>,
    ///The liquid staked SOL token account collecting fees. PDA. Seeds = ['fee', dest_token_mint.pubkey]
    pub dest_token_fee_token_account: &'info AccountInfo<'info>,
    ///The liquid staked SOL mint
    pub dest_token_mint: &'info AccountInfo<'info>,
    ///wSOL token mint
    pub wsol_mint: &'info AccountInfo<'info>,
    pub token_program: &'info AccountInfo<'info>,
    ///System program. The deposit SOL accounts slice follows.
    pub system_program: &'info AccountInfo<'info>,
}

impl<'info> SanctumStakeWsol<'info> {
    fn parse_accounts(accounts: &'info [AccountInfo<'info>], offset: usize) -> Result<Self> {
        let [
            dex_program_id,
            swap_authority_pubkey,
            wsol_from,
            dest_token_to,
            wsol_bridge_in,
            sol_bridge_out,
            dest_token_fee_token_account,
            dest_token_mint,
            wsol_mint,
            token_program,
            system_program,
        ]: &[AccountInfo<'info>; STAKE_WRAPPED_SOL_IX_ACCOUNTS_LEN] =
            array_ref![accounts, offset, STAKE_WRAPPED_SOL_IX_ACCOUNTS_LEN];
        Ok(SanctumStakeWsol {
            dex_program_id,
            swap_authority_pubkey,
            wsol_from: InterfaceAccount::try_from(wsol_from)?,
            dest_token_to: InterfaceAccount::try_from(dest_token_to)?,
            wsol_bridge_in,
            sol_bridge_out,
            dest_token_fee_token_account,
            dest_token_mint,
            wsol_mint,
            token_program,
            system_program,
        })
    }
    fn dex_program_id(&self) -> &AccountInfo<'info> {
        self.dex_program_id
    }

    fn src_token_account(&self) -> &InterfaceAccount<'info, TokenAccount> {
        &self.wsol_from
    }

    fn dst_token_account(&self) -> &InterfaceAccount<'info, TokenAccount> {
        &self.dest_token_to
    }

    fn get_token_accounts_mut(
        &mut self,
    ) -> (&mut InterfaceAccount<'info, TokenAccount>, &mut InterfaceAccount<'info, TokenAccount>)
    {
        (&mut self.wsol_from, &mut self.dest_token_to)
    }

    fn accountmetas_len(&self) -> usize {
        STAKE_WRAPPED_SOL_IX_ACCOUNTS_LEN - 1
    }

    fn append_accountmetas(&self, metas: &mut Vec<AccountMeta>) {
        metas.extend([
            AccountMeta {
                pubkey: self.swap_authority_pubkey.key(),
                is_signer: true,
                is_writable: false,
            },
            AccountMeta { pubkey: self.wsol_from.key(), is_signer: false, is_writable: true },
            AccountMeta { pubkey: self.dest_token_to.key(), is_signer: false, is_writable: true },
            AccountMeta { pubkey: self.wsol_bridge_in.key(), is_signer: false, is_writable: true },
            AccountMeta { pubkey: self.sol_bridge_out.key(), is_signer: false, is_writable: true },
            AccountMeta {
                pubkey: self.dest_token_fee_token_account.key(),
                is_signer: false,
                is_writable: true,
            },
            AccountMeta { pubkey: self.dest_token_mint.key(), is_signer: false, is_writable: true },
            AccountMeta { pubkey: self.wsol_mint.key(), is_signer: false, is_writable: false },
            AccountMeta {
                pubkey: self.token_program.key().key(),
                is_signer: false,
                is_writable: false,
            },
            AccountMeta { pubkey: self.system_program.key(), is_signer: false, is_writable: false },
        ]);
    }
    fn append_accountinfos<I: Extend<AccountInfo<'info>>>(&self, infos: &mut I) {
        infos.extend([
            self.dex_program_id.to_account_info(),
            self.swap_authority_pubkey.to_account_info(),
            self.wsol_from.to_account_info(),
            self.dest_token_to.to_account_info(),
            self.wsol_bridge_in.to_account_info(),
            self.sol_bridge_out.to_account_info(),
            self.dest_token_fee_token_account.to_account_info(),
            self.dest_token_mint.to_account_info(),
            self.wsol_mint.to_account_info(),
            self.token_program.to_account_info(),
            self.system_program.to_account_info(),
        ]);
    }
}

pub struct MarinadeSolDeposit<'info> {
    pub marinade_program: &'info AccountInfo<'info>,
    pub marinade_state: &'info AccountInfo<'info>,
    pub marinade_liq_pool_sol_leg: &'info AccountInfo<'info>,
    pub marinade_liq_pool_msol_leg: &'info AccountInfo<'info>,
    pub marinade_liq_pool_msol_leg_auth: &'info AccountInfo<'info>,
    pub marinade_reserve: &'info AccountInfo<'info>,
    pub msol_mint_authority: &'info AccountInfo<'info>,
}

impl<'info> MarinadeSolDeposit<'info> {
    fn parse_accounts(accounts: &'info [AccountInfo<'info>], offset: usize) -> Result<Self> {
        let [
            marinade_program,
            marinade_state,
            msol_mint_authority,
            marinade_reserve,
            marinade_liq_pool_msol_leg,
            marinade_liq_pool_msol_leg_auth,
            marinade_liq_pool_sol_leg,
        ]: &[AccountInfo<'info>; MARINADE_DEPOSIT_SOL_IX_ACCOUNTS_LEN] =
            array_ref![accounts, offset, MARINADE_DEPOSIT_SOL_IX_ACCOUNTS_LEN];
        Ok(MarinadeSolDeposit {
            marinade_program,
            marinade_state,
            marinade_liq_pool_sol_leg,
            marinade_liq_pool_msol_leg,
            marinade_liq_pool_msol_leg_auth,
            marinade_reserve,
            msol_mint_authority,
        })
    }
}

impl<'info> StakeDexAccounts<'info> for MarinadeSolDeposit<'info> {
    fn append_accountinfos<I: Extend<AccountInfo<'info>>>(&self, infos: &mut I) {
        infos.extend([
            self.marinade_program.to_account_info(),
            self.marinade_state.to_account_info(),
            self.msol_mint_authority.to_account_info(),
            self.marinade_reserve.to_account_info(),
            self.marinade_liq_pool_msol_leg.to_account_info(),
            self.marinade_liq_pool_msol_leg_auth.to_account_info(),
            self.marinade_liq_pool_sol_leg.to_account_info(),
        ]);
    }
    fn append_accountmetas(&self, metas: &mut Vec<AccountMeta>) {
        metas.extend([
            AccountMeta {
                pubkey: self.marinade_program.key(),
                is_signer: false,
                is_writable: false,
            },
            AccountMeta { pubkey: self.marinade_state.key(), is_signer: false, is_writable: true },
            AccountMeta {
                pubkey: self.marinade_liq_pool_sol_leg.key(),
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: self.marinade_liq_pool_msol_leg.key(),
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: self.marinade_liq_pool_msol_leg_auth.key(),
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: self.marinade_reserve.key(),
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: self.msol_mint_authority.key(),
                is_signer: false,
                is_writable: false,
            },
        ]);
    }

    fn accountmetas_len(&self) -> usize {
        MARINADE_DEPOSIT_SOL_IX_ACCOUNTS_LEN
    }
}

pub struct SplSolDeposit<'info> {
    pub spl_stake_pool_program: &'info AccountInfo<'info>,
    pub stake_pool: &'info AccountInfo<'info>,
    pub stake_pool_withdraw_authority: &'info AccountInfo<'info>,
    pub stake_pool_reserve_stake: &'info AccountInfo<'info>,
    pub stake_pool_manager_fee: &'info AccountInfo<'info>,
}

impl<'info> SplSolDeposit<'info> {
    fn parse_accounts(accounts: &'info [AccountInfo<'info>], offset: usize) -> Result<Self> {
        let [
            spl_stake_pool_program,
            stake_pool,
            stake_pool_withdraw_authority,
            stake_pool_reserve_stake,
            stake_pool_manager_fee,
        ]: &[AccountInfo<'info>; SPL_STAKE_POOL_DEPOSIT_SOL_IX_ACCOUNTS_LEN] =
            array_ref![accounts, offset, SPL_STAKE_POOL_DEPOSIT_SOL_IX_ACCOUNTS_LEN];
        Ok(SplSolDeposit {
            spl_stake_pool_program,
            stake_pool,
            stake_pool_withdraw_authority,
            stake_pool_reserve_stake,
            stake_pool_manager_fee,
        })
    }
}

impl<'info> StakeDexAccounts<'info> for SplSolDeposit<'info> {
    fn append_accountmetas(&self, metas: &mut Vec<AccountMeta>) {
        metas.extend([
            AccountMeta {
                pubkey: self.spl_stake_pool_program.key(),
                is_signer: false,
                is_writable: false,
            },
            AccountMeta { pubkey: self.stake_pool.key(), is_signer: false, is_writable: true },
            AccountMeta {
                pubkey: self.stake_pool_withdraw_authority.key(),
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: self.stake_pool_reserve_stake.key(),
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: self.stake_pool_manager_fee.key(),
                is_signer: false,
                is_writable: true,
            },
        ]);
    }
    fn append_accountinfos<I: Extend<AccountInfo<'info>>>(&self, infos: &mut I) {
        infos.extend([
            self.spl_stake_pool_program.to_account_info(),
            self.stake_pool.to_account_info(),
            self.stake_pool_withdraw_authority.to_account_info(),
            self.stake_pool_reserve_stake.to_account_info(),
            self.stake_pool_manager_fee.to_account_info(),
        ]);
    }

    fn accountmetas_len(&self) -> usize {
        SPL_STAKE_POOL_DEPOSIT_SOL_IX_ACCOUNTS_LEN
    }
}

pub struct SanctumPrefundWithdrawStake<'info> {
    pub dex_program_id: &'info AccountInfo<'info>,
    ///The withdraw authority of stake_account. Needs to be mutable and system account to receive slumlord flash loan.
    pub swap_authority_pubkey: &'info AccountInfo<'info>,
    ///The token account to burn src tokens from in order to withdraw stake
    pub src_token_from: InterfaceAccount<'info, TokenAccount>,
    ///The bridge stake account thats withdrawn and given to the user. PDA. seeds = ['bridge_stake', user.pubkey, SwapArgs.bridge_stake_seed]. Might be long-lived, make sure the seed is not already in use
    pub bridge_stake: &'info AccountInfo<'info>,
    ///Input LST token mint
    pub src_token_mint: &'info AccountInfo<'info>,
    ///The system account PDA that contains enough SOL to prefund 2 stake accounts for withdrawal. Soinfoone must send SOL to here to initialize it. Seeds = ['prefunder']
    pub prefunder: &'info AccountInfo<'info>,
    ///The slumdog stake account is split from bridge_stake upon stake withdraw and instant unstaked to repay slumlord's flash loan. create_with_seed(bridge_stake.pubkey, 'slumdog', stake_program). Might be long-lived, but should be not in use as long as bridge_stake is not in use
    pub slumdog_stake: &'info AccountInfo<'info>,
    ///Sanctum unstake program. unpXTU2Ndrc7WWNyEhQWe4udTzSibLPi25SXv2xbCHQ
    pub unstakeit_program: &'info AccountInfo<'info>,
    ///Sanctum unstake pool. FypPtwbY3FUfzJUtXHSyVRokVKG2jKtH29FmK4ebxRSd
    pub unstake_pool: &'info AccountInfo<'info>,
    ///Sanctum unstake pool SOL reserves. 3rBnnH9TTgd3xwu48rnzGsaQkSr1hR64nY71DrDt6VrQ
    pub pool_sol_reserves: &'info AccountInfo<'info>,
    ///Sanctum unstake pool Fee account. 5Pcu8WeQa3VbBz2vdBT49Rj4gbS4hsnfzuL1LmuRaKFY
    pub unstake_fee: &'info AccountInfo<'info>,
    ///Sanctum unstake pool stake account record for slumdog stake. PDA of sanctum unstake program. Seeds = [unstakePool.pubkey, slumdogStake.pubkey].
    pub slumdog_stake_acc_record: &'info AccountInfo<'info>,
    ///Sanctum unstake pool protocol fee account. 2hN9UhvRFVfPYKL6rZJ5YiLEPCLTpN755pgwDJHWgFbU
    pub unstake_protocol_fee: &'info AccountInfo<'info>,
    ///Sanctum unstake pool protocol fee destination. unstakeProtocolFee.destination
    pub unstake_protocol_fee_dest: &'info AccountInfo<'info>,
    ///sysvar clock
    pub clock: &'info AccountInfo<'info>,
    ///stake program
    pub stake_program: &'info AccountInfo<'info>,
    ///System program. The withdraw stake accounts slices follow.
    pub system_program: &'info AccountInfo<'info>,
}

impl<'info> SanctumPrefundWithdrawStake<'info> {
    fn parse_accounts(accounts: &'info [AccountInfo<'info>], offset: usize) -> Result<Self> {
        let [
            dex_program_id,
            swap_authority_pubkey,
            src_token_from,
            bridge_stake,
            src_token_mint,
            prefunder,
            slumdog_stake,
            unstakeit_program,
            unstake_pool,
            pool_sol_reserves,
            unstake_fee,
            slumdog_stake_acc_record,
            unstake_protocol_fee,
            unstake_protocol_fee_dest,
            clock,
            stake_program,
            system_program,
        ]: &[AccountInfo<'info>; PREFUND_WITHDRAW_STAKE_IX_ACCOUNTS_LEN] =
            array_ref![accounts, offset, PREFUND_WITHDRAW_STAKE_IX_ACCOUNTS_LEN];
        Ok(SanctumPrefundWithdrawStake {
            dex_program_id,
            swap_authority_pubkey,
            src_token_from: InterfaceAccount::try_from(src_token_from)?,
            bridge_stake,
            src_token_mint,
            prefunder,
            slumdog_stake,
            unstakeit_program,
            unstake_pool,
            pool_sol_reserves,
            unstake_fee,
            slumdog_stake_acc_record,
            unstake_protocol_fee,
            unstake_protocol_fee_dest,
            clock,
            stake_program,
            system_program,
        })
    }

    fn dex_program_id(&self) -> &AccountInfo<'info> {
        self.dex_program_id
    }

    fn src_token_account_mut(&mut self) -> &mut InterfaceAccount<'info, TokenAccount> {
        &mut self.src_token_from
    }

    fn accountmetas_len(&self) -> usize {
        PREFUND_WITHDRAW_STAKE_IX_ACCOUNTS_LEN - 1
    }

    fn append_accountmetas(&self, metas: &mut Vec<AccountMeta>) {
        metas.extend([
            AccountMeta {
                pubkey: self.swap_authority_pubkey.key(),
                is_signer: true,
                is_writable: true,
            },
            AccountMeta { pubkey: self.src_token_from.key(), is_signer: false, is_writable: true },
            AccountMeta { pubkey: self.bridge_stake.key(), is_signer: false, is_writable: true },
            AccountMeta { pubkey: self.src_token_mint.key(), is_signer: false, is_writable: true },
            AccountMeta { pubkey: self.prefunder.key(), is_signer: false, is_writable: true },
            AccountMeta { pubkey: self.slumdog_stake.key(), is_signer: false, is_writable: true },
            AccountMeta {
                pubkey: self.unstakeit_program.key(),
                is_signer: false,
                is_writable: false,
            },
            AccountMeta { pubkey: self.unstake_pool.key(), is_signer: false, is_writable: true },
            AccountMeta {
                pubkey: self.pool_sol_reserves.key(),
                is_signer: false,
                is_writable: true,
            },
            AccountMeta { pubkey: self.unstake_fee.key(), is_signer: false, is_writable: false },
            AccountMeta {
                pubkey: self.slumdog_stake_acc_record.key(),
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: self.unstake_protocol_fee.key(),
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: self.unstake_protocol_fee_dest.key(),
                is_signer: false,
                is_writable: true,
            },
            AccountMeta { pubkey: self.clock.key(), is_signer: false, is_writable: false },
            AccountMeta { pubkey: self.stake_program.key(), is_signer: false, is_writable: false },
            AccountMeta { pubkey: self.system_program.key(), is_signer: false, is_writable: false },
        ]);
    }

    fn append_accountinfos<I: Extend<AccountInfo<'info>>>(&self, infos: &mut I) {
        infos.extend([
            self.dex_program_id.to_account_info(),
            self.swap_authority_pubkey.to_account_info(),
            self.src_token_from.to_account_info(),
            self.bridge_stake.to_account_info(),
            self.src_token_mint.to_account_info(),
            self.prefunder.to_account_info(),
            self.slumdog_stake.to_account_info(),
            self.unstakeit_program.to_account_info(),
            self.unstake_pool.to_account_info(),
            self.pool_sol_reserves.to_account_info(),
            self.unstake_fee.to_account_info(),
            self.slumdog_stake_acc_record.to_account_info(),
            self.unstake_protocol_fee.to_account_info(),
            self.unstake_protocol_fee_dest.to_account_info(),
            self.clock.to_account_info(),
            self.stake_program.to_account_info(),
            self.system_program.to_account_info(),
        ]);
    }
}
pub struct LidoWithdrawStake<'info> {
    pub lido_program: &'info AccountInfo<'info>,
    pub withdraw_stake_solido: &'info AccountInfo<'info>,
    pub withdraw_stake_voter: &'info AccountInfo<'info>,
    pub withdraw_stake_stake_to_split: &'info AccountInfo<'info>,
    pub withdraw_stake_stake_authority: &'info AccountInfo<'info>,
    pub withdraw_stake_validator_list: &'info AccountInfo<'info>,
    pub clock: &'info AccountInfo<'info>,
    pub token_program: &'info AccountInfo<'info>,
    pub stake_program: &'info AccountInfo<'info>,
    pub system_program: &'info AccountInfo<'info>,
}
impl<'info> StakeDexAccounts<'info> for LidoWithdrawStake<'info> {
    fn append_accountmetas(&self, metas: &mut Vec<AccountMeta>) {
        metas.extend([
            AccountMeta { pubkey: self.lido_program.key(), is_signer: false, is_writable: false },
            AccountMeta {
                pubkey: self.withdraw_stake_solido.key(),
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: self.withdraw_stake_voter.key(),
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: self.withdraw_stake_stake_to_split.key(),
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: self.withdraw_stake_stake_authority.key(),
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: self.withdraw_stake_validator_list.key(),
                is_signer: false,
                is_writable: true,
            },
            AccountMeta { pubkey: self.clock.key(), is_signer: false, is_writable: false },
            AccountMeta { pubkey: self.token_program.key(), is_signer: false, is_writable: false },
            AccountMeta { pubkey: self.stake_program.key(), is_signer: false, is_writable: false },
            AccountMeta { pubkey: self.system_program.key(), is_signer: false, is_writable: false },
        ]);
    }
    fn append_accountinfos<I: Extend<AccountInfo<'info>>>(&self, infos: &mut I) {
        infos.extend([
            self.lido_program.to_account_info(),
            self.withdraw_stake_solido.to_account_info(),
            self.withdraw_stake_voter.to_account_info(),
            self.withdraw_stake_stake_to_split.to_account_info(),
            self.withdraw_stake_stake_authority.to_account_info(),
            self.withdraw_stake_validator_list.to_account_info(),
            self.clock.to_account_info(),
            self.token_program.to_account_info(),
            self.stake_program.to_account_info(),
            self.system_program.to_account_info(),
        ]);
    }

    fn accountmetas_len(&self) -> usize {
        LIDO_WITHDRAW_STAKE_IX_ACCOUNTS_LEN
    }
}
impl<'info> LidoWithdrawStake<'info> {
    fn parse_accounts(accounts: &'info [AccountInfo<'info>], offset: usize) -> Result<Self> {
        let [
            lido_program,
            withdraw_stake_solido,
            withdraw_stake_voter,
            withdraw_stake_stake_to_split,
            withdraw_stake_stake_authority,
            withdraw_stake_validator_list,
            clock,
            token_program,
            stake_program,
            system_program,
        ]: &[AccountInfo<'info>; LIDO_WITHDRAW_STAKE_IX_ACCOUNTS_LEN] =
            array_ref![accounts, offset, LIDO_WITHDRAW_STAKE_IX_ACCOUNTS_LEN];
        Ok(LidoWithdrawStake {
            lido_program,
            withdraw_stake_solido,
            withdraw_stake_voter,
            withdraw_stake_stake_to_split,
            withdraw_stake_stake_authority,
            withdraw_stake_validator_list,
            clock,
            token_program,
            stake_program,
            system_program,
        })
    }
}

pub struct SplStakePoolWithdrawStake<'info> {
    pub spl_stake_pool_program: &'info AccountInfo<'info>,
    pub withdraw_stake_spl_stake_pool: &'info AccountInfo<'info>,
    pub withdraw_stake_validator_list: &'info AccountInfo<'info>,
    pub withdraw_stake_withdraw_authority: &'info AccountInfo<'info>,
    pub withdraw_stake_stake_to_split: &'info AccountInfo<'info>,
    pub withdraw_stake_manager_fee: &'info AccountInfo<'info>,
    pub clock: &'info AccountInfo<'info>,
    pub token_program: &'info AccountInfo<'info>,
    pub stake_program: &'info AccountInfo<'info>,
    pub system_program: &'info AccountInfo<'info>,
}

impl<'info> SplStakePoolWithdrawStake<'info> {
    fn parse_accounts(accounts: &'info [AccountInfo<'info>], offset: usize) -> Result<Self> {
        let [
            spl_stake_pool_program,
            withdraw_stake_spl_stake_pool,
            withdraw_stake_validator_list,
            withdraw_stake_withdraw_authority,
            withdraw_stake_stake_to_split,
            withdraw_stake_manager_fee,
            clock,
            token_program,
            stake_program,
            system_program,
        ]: &[AccountInfo<'info>; SPL_STAKEPOOL_WITHDRAW_STAKE_ACCOUNTS_LEN] =
            array_ref![accounts, offset, SPL_STAKEPOOL_WITHDRAW_STAKE_ACCOUNTS_LEN];
        Ok(SplStakePoolWithdrawStake {
            spl_stake_pool_program,
            withdraw_stake_spl_stake_pool,
            withdraw_stake_validator_list,
            withdraw_stake_withdraw_authority,
            withdraw_stake_stake_to_split,
            withdraw_stake_manager_fee,
            clock,
            token_program,
            stake_program,
            system_program,
        })
    }
}
impl<'info> StakeDexAccounts<'info> for SplStakePoolWithdrawStake<'info> {
    fn append_accountmetas(&self, metas: &mut Vec<AccountMeta>) {
        metas.extend([
            AccountMeta {
                pubkey: self.spl_stake_pool_program.key(),
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: self.withdraw_stake_spl_stake_pool.key(),
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: self.withdraw_stake_validator_list.key(),
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: self.withdraw_stake_withdraw_authority.key(),
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: self.withdraw_stake_stake_to_split.key(),
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: self.withdraw_stake_manager_fee.key(),
                is_signer: false,
                is_writable: true,
            },
            AccountMeta { pubkey: self.clock.key(), is_signer: false, is_writable: false },
            AccountMeta { pubkey: self.token_program.key(), is_signer: false, is_writable: false },
            AccountMeta { pubkey: self.stake_program.key(), is_signer: false, is_writable: false },
            AccountMeta { pubkey: self.system_program.key(), is_signer: false, is_writable: false },
        ]);
    }

    fn append_accountinfos<I: Extend<AccountInfo<'info>>>(&self, infos: &mut I) {
        infos.extend([
            self.spl_stake_pool_program.to_account_info(),
            self.withdraw_stake_spl_stake_pool.to_account_info(),
            self.withdraw_stake_validator_list.to_account_info(),
            self.withdraw_stake_withdraw_authority.to_account_info(),
            self.withdraw_stake_stake_to_split.to_account_info(),
            self.withdraw_stake_manager_fee.to_account_info(),
            self.clock.to_account_info(),
            self.token_program.to_account_info(),
            self.stake_program.to_account_info(),
            self.system_program.to_account_info(),
        ]);
    }

    fn accountmetas_len(&self) -> usize {
        SPL_STAKEPOOL_WITHDRAW_STAKE_ACCOUNTS_LEN
    }
}

pub struct SanctumDepositStake<'info> {
    pub dex_program_id: &'info AccountInfo<'info>,
    ///The withdraw authority of stake_account. Needs to be mutable to support marinade deposit stake.
    pub swap_authority_pubkey: &'info AccountInfo<'info>,
    ///The stake account to deposit
    pub stake_account: &'info AccountInfo<'info>,
    ///The token account to receive dest tokens to
    pub dest_token_to: InterfaceAccount<'info, TokenAccount>,
    ///The dest_token_mint token account collecting fees. PDA. Seeds = ['fee', dest_token_mint.pubkey]
    pub dest_token_fee_token_account: &'info AccountInfo<'info>,
    ///Output token mint. If this is wrapped SOL, the account can be set to read-only. The deposit stake accounts slice follows.
    pub dest_token_mint: &'info AccountInfo<'info>,
}

impl<'info> SanctumDepositStake<'info> {
    fn parse_accounts(accounts: &'info [AccountInfo<'info>], offset: usize) -> Result<Self> {
        let [
            dex_program_id,
            swap_authority_pubkey,
            stake_account,
            dest_token_to,
            dest_token_fee_token_account,
            dest_token_mint,
        ]: &[AccountInfo<'info>; DEPOSIT_STAKE_IX_ACCOUNTS_LEN] =
            array_ref![accounts, offset, DEPOSIT_STAKE_IX_ACCOUNTS_LEN];
        Ok(SanctumDepositStake {
            dex_program_id,
            swap_authority_pubkey,
            stake_account,
            dest_token_to: InterfaceAccount::try_from(dest_token_to)?,
            dest_token_fee_token_account,
            dest_token_mint,
        })
    }

    fn dex_program_id(&self) -> &AccountInfo<'info> {
        self.dex_program_id
    }

    fn dst_token_account(&self) -> &InterfaceAccount<'info, TokenAccount> {
        &self.dest_token_to
    }

    fn dst_token_account_mut(&mut self) -> &mut InterfaceAccount<'info, TokenAccount> {
        &mut self.dest_token_to
    }

    fn accountmetas_len(&self) -> usize {
        DEPOSIT_STAKE_IX_ACCOUNTS_LEN - 1
    }

    fn append_accountmetas(&self, metas: &mut Vec<AccountMeta>) {
        metas.extend([
            AccountMeta {
                pubkey: self.swap_authority_pubkey.key(),
                is_signer: true,
                is_writable: true,
            },
            AccountMeta { pubkey: self.stake_account.key(), is_signer: false, is_writable: true },
            AccountMeta { pubkey: self.dest_token_to.key(), is_signer: false, is_writable: true },
            AccountMeta {
                pubkey: self.dest_token_fee_token_account.key(),
                is_signer: false,
                is_writable: true,
            },
            AccountMeta { pubkey: self.dest_token_mint.key(), is_signer: false, is_writable: true },
        ]);
    }

    fn append_accountinfos<I: Extend<AccountInfo<'info>>>(&self, infos: &mut I) {
        infos.extend([
            self.dex_program_id.to_account_info(),
            self.swap_authority_pubkey.to_account_info(),
            self.stake_account.to_account_info(),
            self.dest_token_to.to_account_info(),
            self.dest_token_fee_token_account.to_account_info(),
            self.dest_token_mint.to_account_info(),
        ]);
    }
}

pub struct MarinadeStakeDeposit<'info> {
    pub marinade_program: &'info AccountInfo<'info>,
    pub deposit_stake_marinade_state: &'info AccountInfo<'info>,
    pub deposit_stake_validator_list: &'info AccountInfo<'info>,
    pub deposit_stake_stake_list: &'info AccountInfo<'info>,
    pub deposit_stake_duplication_flag: &'info AccountInfo<'info>,
    pub deposit_stake_msol_mint_auth: &'info AccountInfo<'info>,
    pub clock: &'info AccountInfo<'info>,
    pub rent: &'info AccountInfo<'info>,
    pub system_program: &'info AccountInfo<'info>,
    pub token_program: &'info AccountInfo<'info>,
    pub stake_program: &'info AccountInfo<'info>,
}

impl<'info> MarinadeStakeDeposit<'info> {
    fn parse_accounts(accounts: &'info [AccountInfo<'info>], offset: usize) -> Result<Self> {
        let [
            marinade_program,
            deposit_stake_marinade_state,
            deposit_stake_validator_list,
            deposit_stake_stake_list,
            deposit_stake_duplication_flag,
            deposit_stake_msol_mint_auth,
            clock,
            rent,
            system_program,
            token_program,
            stake_program,
        ]: &[AccountInfo<'info>; MARINADE_DEPOSIT_STAKE_IX_ACCOUNTS_LEN] =
            array_ref![accounts, offset, MARINADE_DEPOSIT_STAKE_IX_ACCOUNTS_LEN];
        Ok(MarinadeStakeDeposit {
            marinade_program,
            deposit_stake_marinade_state,
            deposit_stake_validator_list,
            deposit_stake_stake_list,
            deposit_stake_duplication_flag,
            deposit_stake_msol_mint_auth,
            clock,
            rent,
            system_program,
            token_program,
            stake_program,
        })
    }
}

impl<'info> StakeDexAccounts<'info> for MarinadeStakeDeposit<'info> {
    fn append_accountmetas(&self, metas: &mut Vec<AccountMeta>) {
        metas.extend([
            AccountMeta {
                pubkey: self.marinade_program.key(),
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: self.deposit_stake_marinade_state.key(),
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: self.deposit_stake_validator_list.key(),
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: self.deposit_stake_stake_list.key(),
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: self.deposit_stake_duplication_flag.key(),
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: self.deposit_stake_msol_mint_auth.key(),
                is_signer: false,
                is_writable: false,
            },
            AccountMeta { pubkey: self.clock.key(), is_signer: false, is_writable: false },
            AccountMeta { pubkey: self.rent.key(), is_signer: false, is_writable: false },
            AccountMeta { pubkey: self.system_program.key(), is_signer: false, is_writable: false },
            AccountMeta { pubkey: self.token_program.key(), is_signer: false, is_writable: false },
            AccountMeta { pubkey: self.stake_program.key(), is_signer: false, is_writable: false },
        ]);
    }
    fn append_accountinfos<I: Extend<AccountInfo<'info>>>(&self, infos: &mut I) {
        infos.extend([
            self.marinade_program.to_account_info(),
            self.deposit_stake_marinade_state.to_account_info(),
            self.deposit_stake_validator_list.to_account_info(),
            self.deposit_stake_stake_list.to_account_info(),
            self.deposit_stake_duplication_flag.to_account_info(),
            self.deposit_stake_msol_mint_auth.to_account_info(),
            self.clock.to_account_info(),
            self.rent.to_account_info(),
            self.system_program.to_account_info(),
            self.token_program.to_account_info(),
            self.stake_program.to_account_info(),
        ]);
    }

    fn accountmetas_len(&self) -> usize {
        MARINADE_DEPOSIT_STAKE_IX_ACCOUNTS_LEN
    }
}

pub struct SplStakeDeposit<'info> {
    pub spl_stake_pool_program: &'info AccountInfo<'info>,
    pub deposit_stake_spl_stake_pool: &'info AccountInfo<'info>,
    pub deposit_stake_validator_list: &'info AccountInfo<'info>,
    pub deposit_stake_deposit_authority: &'info AccountInfo<'info>,
    pub deposit_stake_withdraw_authority: &'info AccountInfo<'info>,
    pub deposit_stake_validator_stake: &'info AccountInfo<'info>,
    pub deposit_stake_reserve_stake: &'info AccountInfo<'info>,
    pub deposit_stake_manager_fee: &'info AccountInfo<'info>,
    pub clock: &'info AccountInfo<'info>,
    pub stake_history: &'info AccountInfo<'info>,
    pub token_program: &'info AccountInfo<'info>,
    pub stake_program: &'info AccountInfo<'info>,
}

impl<'info> StakeDexAccounts<'info> for SplStakeDeposit<'info> {
    fn append_accountmetas(&self, metas: &mut Vec<AccountMeta>) {
        metas.extend([
            AccountMeta {
                pubkey: self.spl_stake_pool_program.key(),
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: self.deposit_stake_spl_stake_pool.key(),
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: self.deposit_stake_validator_list.key(),
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: self.deposit_stake_deposit_authority.key(),
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: self.deposit_stake_withdraw_authority.key(),
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: self.deposit_stake_validator_stake.key(),
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: self.deposit_stake_reserve_stake.key(),
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: self.deposit_stake_manager_fee.key(),
                is_signer: false,
                is_writable: true,
            },
            AccountMeta { pubkey: self.clock.key(), is_signer: false, is_writable: false },
            AccountMeta { pubkey: self.stake_history.key(), is_signer: false, is_writable: false },
            AccountMeta { pubkey: self.token_program.key(), is_signer: false, is_writable: false },
            AccountMeta { pubkey: self.stake_program.key(), is_signer: false, is_writable: false },
        ]);
    }

    fn append_accountinfos<I: Extend<AccountInfo<'info>>>(&self, infos: &mut I) {
        infos.extend([
            self.spl_stake_pool_program.to_account_info(),
            self.deposit_stake_spl_stake_pool.to_account_info(),
            self.deposit_stake_validator_list.to_account_info(),
            self.deposit_stake_deposit_authority.to_account_info(),
            self.deposit_stake_withdraw_authority.to_account_info(),
            self.deposit_stake_validator_stake.to_account_info(),
            self.deposit_stake_reserve_stake.to_account_info(),
            self.deposit_stake_manager_fee.to_account_info(),
            self.clock.to_account_info(),
            self.stake_history.to_account_info(),
            self.token_program.to_account_info(),
            self.stake_program.to_account_info(),
        ]);
    }

    fn accountmetas_len(&self) -> usize {
        SPL_STAKEPOOL_DEPOSIT_ACCOUNTS_LEN
    }
}

impl<'info> SplStakeDeposit<'info> {
    fn parse_accounts(accounts: &'info [AccountInfo<'info>], offset: usize) -> Result<Self> {
        let [
            spl_stake_pool_program,
            deposit_stake_spl_stake_pool,
            deposit_stake_validator_list,
            deposit_stake_deposit_authority,
            deposit_stake_withdraw_authority,
            deposit_stake_validator_stake,
            deposit_stake_reserve_stake,
            deposit_stake_manager_fee,
            clock,
            stake_history,
            token_program,
            stake_program,
        ]: &[AccountInfo<'info>; SPL_STAKEPOOL_DEPOSIT_ACCOUNTS_LEN] =
            array_ref![accounts, offset, SPL_STAKEPOOL_DEPOSIT_ACCOUNTS_LEN];
        Ok(SplStakeDeposit {
            spl_stake_pool_program,
            deposit_stake_spl_stake_pool,
            deposit_stake_validator_list,
            deposit_stake_deposit_authority,
            deposit_stake_withdraw_authority,
            deposit_stake_validator_stake,
            deposit_stake_reserve_stake,
            deposit_stake_manager_fee,
            clock,
            stake_history,
            token_program,
            stake_program,
        })
    }
}

pub struct SplStakePoolWithdrawSol<'info> {
    pub spl_stake_pool_program: &'info AccountInfo<'info>,
    pub withdraw_sol_spl_stake_pool: &'info AccountInfo<'info>,
    pub withdraw_sol_withdraw_authority: &'info AccountInfo<'info>,
    pub withdraw_sol_reserve_stake: &'info AccountInfo<'info>,
    pub withdraw_sol_manager_fee: &'info AccountInfo<'info>,
    pub clock: &'info AccountInfo<'info>,
    pub stake_history: &'info AccountInfo<'info>,
    pub stake_program: &'info AccountInfo<'info>,
    ///possible duplicate to account for token-22 stake pools
    pub token_program: &'info AccountInfo<'info>,
}

impl<'info> SplStakePoolWithdrawSol<'info> {
    fn parse_accounts(accounts: &'info [AccountInfo<'info>], offset: usize) -> Result<Self> {
        let [
            spl_stake_pool_program,
            withdraw_sol_spl_stake_pool,
            withdraw_sol_withdraw_authority,
            withdraw_sol_reserve_stake,
            withdraw_sol_manager_fee,
            clock,
            stake_history,
            stake_program,
            token_program,
        ]: &[AccountInfo<'info>; SPL_STAKEPOOL_WITHDRAW_SOL_ACCOUNTS_LEN] =
            array_ref![accounts, offset, SPL_STAKEPOOL_WITHDRAW_SOL_ACCOUNTS_LEN];
        Ok(SplStakePoolWithdrawSol {
            spl_stake_pool_program,
            withdraw_sol_spl_stake_pool,
            withdraw_sol_withdraw_authority,
            withdraw_sol_reserve_stake,
            withdraw_sol_manager_fee,
            clock,
            stake_history,
            stake_program,
            token_program,
        })
    }
}

impl<'info> StakeDexAccounts<'info> for SplStakePoolWithdrawSol<'info> {
    fn append_accountmetas(&self, metas: &mut Vec<AccountMeta>) {
        metas.extend([
            AccountMeta {
                pubkey: self.spl_stake_pool_program.key(),
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: self.withdraw_sol_spl_stake_pool.key(),
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: self.withdraw_sol_withdraw_authority.key(),
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: self.withdraw_sol_reserve_stake.key(),
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: self.withdraw_sol_manager_fee.key(),
                is_signer: false,
                is_writable: true,
            },
            AccountMeta { pubkey: self.clock.key(), is_signer: false, is_writable: false },
            AccountMeta { pubkey: self.stake_history.key(), is_signer: false, is_writable: false },
            AccountMeta { pubkey: self.stake_program.key(), is_signer: false, is_writable: false },
            AccountMeta { pubkey: self.token_program.key(), is_signer: false, is_writable: false },
        ]);
    }
    fn append_accountinfos<I: Extend<AccountInfo<'info>>>(&self, infos: &mut I) {
        infos.extend([
            self.spl_stake_pool_program.to_account_info(),
            self.withdraw_sol_spl_stake_pool.to_account_info(),
            self.withdraw_sol_withdraw_authority.to_account_info(),
            self.withdraw_sol_reserve_stake.to_account_info(),
            self.withdraw_sol_manager_fee.to_account_info(),
            self.clock.to_account_info(),
            self.stake_history.to_account_info(),
            self.stake_program.to_account_info(),
            self.token_program.to_account_info(),
        ]);
    }

    fn accountmetas_len(&self) -> usize {
        SPL_STAKEPOOL_WITHDRAW_SOL_ACCOUNTS_LEN
    }
}

pub struct SanctumWithdrawWsol<'info> {
    pub dex_program_id: &'info AccountInfo<'info>,
    ///The withdraw authority of src_token_from. Needs to be mutable and system accounts.
    pub swap_authority_pubkey: SystemAccount<'info>,
    ///The token account to burn and redeem LSTs from
    pub src_token_from: InterfaceAccount<'info, TokenAccount>,
    ///The wSOL token account to receive withdrawn wrapped SOL to
    pub wsol_to: InterfaceAccount<'info, TokenAccount>,
    ///The dest_token_mint token account collecting fees. PDA. Seeds = ['fee', dest_token_mint.pubkey]
    pub wsol_fee_token_account: &'info AccountInfo<'info>,
    ///Input LST token mint
    pub src_token_mint: InterfaceAccount<'info, Mint>,
    ///wSOL token mint
    pub wsol_mint: InterfaceAccount<'info, Mint>,
    ///Token program. The withdraw SOL accounts slice follows
    pub token_program: Interface<'info, TokenInterface>,
}

impl<'info> SanctumWithdrawWsol<'info> {
    fn parse_accounts(accounts: &'info [AccountInfo<'info>], offset: usize) -> Result<Self> {
        let [
            dex_program_id,
            swap_authority_pubkey,
            src_token_from,
            wsol_to,
            wsol_fee_token_account,
            src_token_mint,
            wsol_mint,
            token_program,
        ]: &[AccountInfo<'info>; WITHDRAW_WRAPPED_SOL_IX_ACCOUNTS_LEN] =
            array_ref![accounts, offset, WITHDRAW_WRAPPED_SOL_IX_ACCOUNTS_LEN];
        Ok(SanctumWithdrawWsol {
            dex_program_id,
            swap_authority_pubkey: SystemAccount::try_from(swap_authority_pubkey)?,
            src_token_from: InterfaceAccount::try_from(src_token_from)?,
            wsol_to: InterfaceAccount::try_from(wsol_to)?,
            wsol_fee_token_account,
            src_token_mint: InterfaceAccount::try_from(src_token_mint)?,
            wsol_mint: InterfaceAccount::try_from(wsol_mint)?,
            token_program: Interface::try_from(token_program)?,
        })
    }
    fn dex_program_id(&self) -> &AccountInfo<'info> {
        self.dex_program_id
    }

    fn dst_token_account(&self) -> &InterfaceAccount<'info, TokenAccount> {
        &self.wsol_to
    }

    fn get_token_accounts_mut(
        &mut self,
    ) -> (&mut InterfaceAccount<'info, TokenAccount>, &mut InterfaceAccount<'info, TokenAccount>)
    {
        (&mut self.src_token_from, &mut self.wsol_to)
    }
    fn accountmetas_len(&self) -> usize {
        WITHDRAW_WRAPPED_SOL_IX_ACCOUNTS_LEN - 1
    }

    fn append_accountmetas(&self, metas: &mut Vec<AccountMeta>) {
        metas.extend([
            AccountMeta {
                pubkey: self.swap_authority_pubkey.key(),
                is_signer: true,
                is_writable: false,
            },
            AccountMeta { pubkey: self.src_token_from.key(), is_signer: false, is_writable: true },
            AccountMeta { pubkey: self.wsol_to.key(), is_signer: false, is_writable: true },
            AccountMeta {
                pubkey: self.wsol_fee_token_account.key(),
                is_signer: false,
                is_writable: true,
            },
            AccountMeta { pubkey: self.src_token_mint.key(), is_signer: false, is_writable: true },
            AccountMeta { pubkey: self.wsol_mint.key(), is_signer: false, is_writable: false },
            AccountMeta { pubkey: self.token_program.key(), is_signer: false, is_writable: false },
        ]);
    }
    fn append_accountinfos<I: Extend<AccountInfo<'info>>>(&self, infos: &mut I) {
        infos.extend([
            self.dex_program_id.to_account_info(),
            self.swap_authority_pubkey.to_account_info(),
            self.src_token_from.to_account_info(),
            self.wsol_to.to_account_info(),
            self.wsol_fee_token_account.to_account_info(),
            self.src_token_mint.to_account_info(),
            self.wsol_mint.to_account_info(),
            self.token_program.to_account_info(),
        ]);
    }
}

pub fn sanctum_router_handler<'a>(
    remaining_accounts: &'a [AccountInfo<'a>],
    amount_in: u64,
    offset: &mut usize,
    hop_accounts: &mut HopAccounts,
    hop: usize,
    proxy_swap: bool,
    order_id: u64,
    owner_seeds: Option<&[&[&[u8]]]>,
) -> Result<u64> {
    //check wsol mint at index 8 if its a stake wsol ix
    if remaining_accounts[6 + *offset].key() == wsol_program::id() {
        withdraw_wsol_handler(
            remaining_accounts,
            amount_in,
            offset,
            hop_accounts,
            hop,
            proxy_swap,
            order_id,
            owner_seeds,
        )
    } else if remaining_accounts[8 + *offset].key() == wsol_program::id() {
        stake_wsol_handler(
            remaining_accounts,
            amount_in,
            offset,
            hop_accounts,
            hop,
            proxy_swap,
            order_id,
            owner_seeds,
        )
    } else {
        prefund_swap_via_stake_handler(
            remaining_accounts,
            amount_in,
            offset,
            hop_accounts,
            hop,
            proxy_swap,
            order_id,
            owner_seeds,
        )
    }
}

pub fn stake_wsol_handler<'a>(
    remaining_accounts: &'a [AccountInfo<'a>],
    amount_in: u64,
    offset: &mut usize,
    hop_accounts: &mut HopAccounts,
    hop: usize,
    proxy_swap: bool,
    _order_id: u64,
    owner_seeds: Option<&[&[&[u8]]]>,
) -> Result<u64> {
    msg!("Dex::SanctumRouterStakeWSol amount_in: {}, offset: {}", amount_in, offset);
    let mut accounts_len: usize = 0;
    let mut stake_wsol_accounts = SanctumStakeWsol::parse_accounts(remaining_accounts, *offset)?;
    accounts_len += STAKE_WRAPPED_SOL_IX_ACCOUNTS_LEN;

    let deposit_accounts: StakeDexAccountsEnum<'a>;
    if stake_wsol_accounts.dest_token_mint.key() == marinade_sol_mint::id() {
        deposit_accounts = StakeDexAccountsEnum::MarinadeSolDeposit(
            MarinadeSolDeposit::parse_accounts(remaining_accounts, *offset + accounts_len)?,
        );
        accounts_len += MARINADE_DEPOSIT_SOL_IX_ACCOUNTS_LEN;
    } else {
        deposit_accounts = StakeDexAccountsEnum::SplSolDeposit(SplSolDeposit::parse_accounts(
            remaining_accounts,
            *offset + accounts_len,
        )?);
        accounts_len += SPL_STAKE_POOL_DEPOSIT_SOL_IX_ACCOUNTS_LEN;
    }

    require_keys_eq!(
        sanctum_router_program::id(),
        stake_wsol_accounts.dex_program_id().key(),
        ErrorCode::InvalidProgramId
    );

    before_check(
        &stake_wsol_accounts.swap_authority_pubkey,
        &stake_wsol_accounts.src_token_account(),
        stake_wsol_accounts.dst_token_account().key(),
        hop_accounts,
        hop,
        proxy_swap,
        owner_seeds,
    )?;

    let mut deposit_stake_ix_data = Vec::with_capacity(1 + 8);
    deposit_stake_ix_data.push(0); //stake wsol discriminator
    deposit_stake_ix_data.extend_from_slice(&amount_in.to_le_bytes());

    let mut deposit_accountmetas: Vec<AccountMeta> = Vec::with_capacity(
        stake_wsol_accounts.accountmetas_len() + deposit_accounts.accountmetas_len(),
    );
    stake_wsol_accounts.append_accountmetas(&mut deposit_accountmetas);
    deposit_accounts.append_accountmetas(&mut deposit_accountmetas);

    let mut swap_accounts: ArrayVec<
        AccountInfo<'a>,
        { STAKE_WRAPPED_SOL_IX_ACCOUNTS_LEN + MARINADE_DEPOSIT_SOL_IX_ACCOUNTS_LEN },
    > = ArrayVec::new();
    stake_wsol_accounts.append_accountinfos(&mut swap_accounts);
    deposit_accounts.append_accountinfos(&mut swap_accounts);

    let (src_token_account_mut, dst_token_account_mut) =
        stake_wsol_accounts.get_token_accounts_mut();
    let dex_processor = &SanctumRouterProcessor;
    invoke_process(
        amount_in,
        dex_processor,
        swap_accounts.as_slice(),
        src_token_account_mut,
        dst_token_account_mut,
        hop_accounts,
        Instruction {
            program_id: sanctum_router_program::id(),
            accounts: deposit_accountmetas,
            data: deposit_stake_ix_data,
        },
        hop,
        offset,
        accounts_len,
        proxy_swap,
        owner_seeds,
    )
}

pub fn prefund_swap_via_stake_handler<'a>(
    remaining_accounts: &'a [AccountInfo<'a>],
    amount_in: u64,
    offset: &mut usize,
    hop_accounts: &mut HopAccounts,
    hop: usize,
    proxy_swap: bool,
    order_id: u64,
    owner_seeds: Option<&[&[&[u8]]]>,
) -> Result<u64> {
    msg!("Dex::SanctumRouterPrefundSwapViaStake amount_in: {}, offset: {}", amount_in, offset);
    require!(order_id > 0, ErrorCode::InvalidBridgeSeed);

    let mut accounts_len: usize = 0;

    let mut prefund_withdraw_accounts =
        SanctumPrefundWithdrawStake::parse_accounts(remaining_accounts, *offset)?;
    accounts_len += PREFUND_WITHDRAW_STAKE_IX_ACCOUNTS_LEN;

    let withdraw_accounts: StakeDexAccountsEnum<'a>;
    if prefund_withdraw_accounts.src_token_mint.key() == lido_sol_mint::id() {
        withdraw_accounts = StakeDexAccountsEnum::LidoWithdrawStake(
            LidoWithdrawStake::parse_accounts(remaining_accounts, *offset + accounts_len)?,
        );
        accounts_len += LIDO_WITHDRAW_STAKE_IX_ACCOUNTS_LEN;
    } else {
        withdraw_accounts = StakeDexAccountsEnum::SplStakePoolWithdrawStake(
            SplStakePoolWithdrawStake::parse_accounts(remaining_accounts, *offset + accounts_len)?,
        );
        accounts_len += SPL_STAKEPOOL_WITHDRAW_STAKE_ACCOUNTS_LEN;
    }

    let mut deposit_stake_accounts =
        SanctumDepositStake::parse_accounts(remaining_accounts, *offset + accounts_len)?;
    accounts_len += DEPOSIT_STAKE_IX_ACCOUNTS_LEN;

    let pool_deposit_accounts: StakeDexAccountsEnum<'a>;
    if deposit_stake_accounts.dest_token_mint.key() == marinade_sol_mint::id() {
        pool_deposit_accounts = StakeDexAccountsEnum::MarinadeStakeDeposit(
            MarinadeStakeDeposit::parse_accounts(remaining_accounts, *offset + accounts_len)?,
        );
        accounts_len += MARINADE_DEPOSIT_STAKE_IX_ACCOUNTS_LEN;
    } else {
        pool_deposit_accounts = StakeDexAccountsEnum::SplStakeDeposit(
            SplStakeDeposit::parse_accounts(remaining_accounts, *offset + accounts_len)?,
        );
        accounts_len += SPL_STAKEPOOL_DEPOSIT_ACCOUNTS_LEN;
    }

    if prefund_withdraw_accounts.dex_program_id().key != &sanctum_router_program::id()
        || deposit_stake_accounts.dex_program_id().key != &sanctum_router_program::id()
    {
        return Err(ErrorCode::InvalidProgramId.into());
    }
    require_keys_eq!(
        prefund_withdraw_accounts.swap_authority_pubkey.key(),
        deposit_stake_accounts.swap_authority_pubkey.key(),
        ErrorCode::InvalidSwapAuthorityAccounts
    );

    let dst_token_account = deposit_stake_accounts.dst_token_account().key();
    let bridge_seed = get_seed_from_orderid(order_id);
    before_check(
        &prefund_withdraw_accounts.swap_authority_pubkey,
        &prefund_withdraw_accounts.src_token_from,
        dst_token_account,
        hop_accounts,
        hop,
        proxy_swap,
        owner_seeds,
    )?;

    let mut prefund_withdraw_ix_data = Vec::with_capacity(1 + 8 + 4);
    prefund_withdraw_ix_data.push(6); //prefund withdraw stake discriminator
    prefund_withdraw_ix_data.extend_from_slice(&amount_in.to_le_bytes()); //amount in
    prefund_withdraw_ix_data.extend_from_slice(&bridge_seed);

    let mut withdraw_accountmetas: Vec<AccountMeta> = Vec::with_capacity(
        prefund_withdraw_accounts.accountmetas_len() + withdraw_accounts.accountmetas_len(),
    );
    prefund_withdraw_accounts.append_accountmetas(&mut withdraw_accountmetas);
    withdraw_accounts.append_accountmetas(&mut withdraw_accountmetas);

    let mut deposit_stake_ix_data = Vec::with_capacity(1);
    deposit_stake_ix_data.push(5); //deposit discriminator

    let mut deposit_accountmetas: Vec<AccountMeta> = Vec::with_capacity(
        deposit_stake_accounts.accountmetas_len() + pool_deposit_accounts.accountmetas_len(),
    );
    deposit_stake_accounts.append_accountmetas(&mut deposit_accountmetas);
    pool_deposit_accounts.append_accountmetas(&mut deposit_accountmetas);

    let mut withdraw_account_infos: ArrayVec<
        AccountInfo<'a>,
        { PREFUND_WITHDRAW_STAKE_IX_ACCOUNTS_LEN + LIDO_WITHDRAW_STAKE_IX_ACCOUNTS_LEN },
    > = ArrayVec::new();
    prefund_withdraw_accounts.append_accountinfos(&mut withdraw_account_infos);
    withdraw_accounts.append_accountinfos(&mut withdraw_account_infos);

    let mut deposit_account_infos: ArrayVec<
        AccountInfo<'a>,
        { DEPOSIT_STAKE_IX_ACCOUNTS_LEN + SPL_STAKEPOOL_DEPOSIT_ACCOUNTS_LEN },
    > = ArrayVec::new();
    deposit_stake_accounts.append_accountinfos(&mut deposit_account_infos);
    pool_deposit_accounts.append_accountinfos(&mut deposit_account_infos);

    let dex_processor = &SanctumRouterProcessor;

    invoke_processes(
        amount_in,
        dex_processor,
        &[withdraw_account_infos.as_slice(), deposit_account_infos.as_slice()],
        prefund_withdraw_accounts.src_token_account_mut(),
        deposit_stake_accounts.dst_token_account_mut(),
        hop_accounts,
        &[
            Instruction {
                program_id: sanctum_router_program::id(),
                accounts: withdraw_accountmetas,
                data: prefund_withdraw_ix_data,
            },
            Instruction {
                program_id: sanctum_router_program::id(),
                accounts: deposit_accountmetas,
                data: deposit_stake_ix_data,
            },
        ],
        hop,
        offset,
        accounts_len,
        proxy_swap,
        owner_seeds,
    )
}

pub fn withdraw_wsol_handler<'a>(
    remaining_accounts: &'a [AccountInfo<'a>],
    amount_in: u64,
    offset: &mut usize,
    hop_accounts: &mut HopAccounts,
    hop: usize,
    proxy_swap: bool,
    _order_id: u64,
    owner_seeds: Option<&[&[&[u8]]]>,
) -> Result<u64> {
    msg!("Dex::SanctumRouterWithdrawWSol amount_in: {}, offset: {}", amount_in, offset);
    let mut accounts_len: usize = 0;

    let mut withdraw_wsol_accounts =
        SanctumWithdrawWsol::parse_accounts(remaining_accounts, *offset)?;
    accounts_len += WITHDRAW_WRAPPED_SOL_IX_ACCOUNTS_LEN;

    require_keys_eq!(
        sanctum_router_program::id(),
        withdraw_wsol_accounts.dex_program_id().key(),
        ErrorCode::InvalidProgramId
    );

    let stake_dex_withdraw_accounts = StakeDexAccountsEnum::SplStakePoolWithdrawSol(
        SplStakePoolWithdrawSol::parse_accounts(remaining_accounts, *offset + accounts_len)?,
    );
    accounts_len += SPL_STAKEPOOL_WITHDRAW_SOL_ACCOUNTS_LEN;

    let dst_token_account = withdraw_wsol_accounts.dst_token_account().key();
    before_check(
        &withdraw_wsol_accounts.swap_authority_pubkey,
        &withdraw_wsol_accounts.src_token_from,
        dst_token_account,
        hop_accounts,
        hop,
        proxy_swap,
        owner_seeds,
    )?;

    let mut ix_data = Vec::with_capacity(1 + 8);
    ix_data.push(8); //withdraw wsol discriminator
    ix_data.extend_from_slice(&amount_in.to_le_bytes());

    let mut withdraw_accout_metas: Vec<AccountMeta> = Vec::with_capacity(
        withdraw_wsol_accounts.accountmetas_len() + stake_dex_withdraw_accounts.accountmetas_len(),
    );
    withdraw_wsol_accounts.append_accountmetas(&mut withdraw_accout_metas);
    stake_dex_withdraw_accounts.append_accountmetas(&mut withdraw_accout_metas);

    let mut withdraw_accout_infos: ArrayVec<
        AccountInfo<'a>,
        { WITHDRAW_WRAPPED_SOL_IX_ACCOUNTS_LEN + SPL_STAKEPOOL_WITHDRAW_SOL_ACCOUNTS_LEN },
    > = ArrayVec::new();
    withdraw_wsol_accounts.append_accountinfos(&mut withdraw_accout_infos);
    stake_dex_withdraw_accounts.append_accountinfos(&mut withdraw_accout_infos);

    let (src_token_account, dst_token_account) = withdraw_wsol_accounts.get_token_accounts_mut();
    invoke_process(
        amount_in,
        &SanctumRouterProcessor,
        withdraw_accout_infos.as_slice(),
        src_token_account,
        dst_token_account,
        hop_accounts,
        Instruction {
            program_id: sanctum_router_program::id(),
            accounts: withdraw_accout_metas,
            data: ix_data,
        },
        hop,
        offset,
        accounts_len,
        proxy_swap,
        owner_seeds,
    )
}

fn get_seed_from_orderid(order_id: u64) -> [u8; 4] {
    let last_4_bytes = (order_id & 0xFFFFFFFF) as u32;
    let bytes: [u8; 4] = last_4_bytes.to_le_bytes();
    bytes
}
