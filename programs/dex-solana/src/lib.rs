use anchor_lang::prelude::*;
pub mod adapters;
pub mod allocator;
pub mod constants;
pub mod cpi_event;
pub mod error;
pub mod instructions;
pub mod processor;
pub mod utils;

pub use constants::*;
pub use instructions::*;
pub use processor::*;

#[cfg(feature = "staging")]
declare_id!("preZmu827KVPCoQ4LYwSoec13x6seQrKA3QpjgDtx1R");

#[cfg(not(feature = "staging"))]
declare_id!("earnLLspPkK9ku4sWbu3EhdQQxorKKzJKj9AKuncy5f");

#[program]
pub mod dex_solana {
    use super::*;

    #[cfg_attr(feature = "log-metrics", dex_macros::log_metrics)]
    pub fn swap<'a>(
        ctx: Context<'_, '_, 'a, 'a, SwapAccounts<'a>>,
        data: SwapArgs,
        order_id: u64,
    ) -> Result<()> {
        instructions::swap_handler(ctx, data, order_id)
    }

    pub fn create_profit_snapshot<'a>(
        ctx: Context<'_, '_, 'a, 'a, CreateProfitSnapshotAccounts<'a>>,
    ) -> Result<()> {
        instructions::create_profit_snapshot_handler(ctx)
    }

    pub fn profit_check<'a>(ctx: Context<'_, '_, 'a, 'a, ProfitCheckAccounts<'a>>) -> Result<()> {
        instructions::profit_check_handler(ctx)
    }

    // ******************** Proxy Swap ******************** //
    #[cfg_attr(feature = "log-metrics", dex_macros::log_metrics)]
    pub fn proxy_swap<'a>(
        ctx: Context<'_, '_, 'a, 'a, ProxySwapAccounts<'a>>,
        data: SwapArgs,
        order_id: u64,
    ) -> Result<()> {
        instructions::proxy_swap_handler(ctx, data, order_id)
    }

    // ******************** Swap V3 ******************** //
    #[cfg_attr(feature = "log-metrics", dex_macros::log_metrics)]
    pub fn swap_v3<'a>(
        ctx: Context<'_, '_, 'a, 'a, CommissionProxySwapAccountsV3<'a>>,
        args: SwapArgs,
        commission_info: u32,
        platform_fee_rate: u16,
        order_id: u64,
    ) -> Result<()> {
        instructions::swap_toc_handler(
            ctx,
            args,
            commission_info,
            order_id,
            Some(platform_fee_rate),
        )
    }

    #[cfg_attr(feature = "log-metrics", dex_macros::log_metrics)]
    pub fn swap_v3_with_cpi_event<'a>(
        ctx: Context<'_, '_, 'a, 'a, SwapAccountsV3WithCpiEvent<'a>>,
        args: SwapArgs,
        commission_info: u32,
        platform_fee_rate: u16,
        order_id: u64,
    ) -> Result<()> {
        instructions::swap_toc_with_cpi_event_handler(
            ctx,
            args,
            commission_info,
            order_id,
            Some(platform_fee_rate),
        )
    }

    #[cfg_attr(feature = "log-metrics", dex_macros::log_metrics)]
    pub fn swap_tob_v3<'a>(
        ctx: Context<'_, '_, 'a, 'a, CommissionProxySwapAccountsV3<'a>>,
        args: SwapArgs,
        commission_info: u32,
        trim_rate: u8,
        platform_fee_rate: u16,
        order_id: u64,
    ) -> Result<()> {
        instructions::swap_tob_handler(
            ctx,
            args,
            commission_info,
            order_id,
            Some(trim_rate),
            Some(platform_fee_rate),
        )
    }

    /// Swap ToB with optional specified receiver
    /// - For normal token swaps: sol_receiver should be None
    /// - For swap to SOL with custom receiver: sol_receiver should be Some and acc_close_flag must be true
    #[cfg_attr(feature = "log-metrics", dex_macros::log_metrics)]
    pub fn swap_tob_v3_with_receiver<'a>(
        ctx: Context<'_, '_, 'a, 'a, CommissionProxySwapAccountsV3WithReceiver<'a>>,
        args: SwapArgs,
        commission_info: u32,
        trim_rate: u8,
        platform_fee_rate: u16,
        order_id: u64,
    ) -> Result<()> {
        instructions::swap_tob_specified_receiver_handler(
            ctx,
            args,
            commission_info,
            order_id,
            Some(trim_rate),
            Some(platform_fee_rate),
        )
    }

    #[cfg_attr(feature = "log-metrics", dex_macros::log_metrics)]
    pub fn swap_tob_v3_enhanced<'a>(
        ctx: Context<'_, '_, 'a, 'a, CommissionProxySwapAccountsV3<'a>>,
        args: SwapArgs,
        commission_info: u32,
        trim_rate: u8,
        charge_rate: u16,
        platform_fee_rate: u16,
        order_id: u64,
    ) -> Result<()> {
        instructions::swap_tob_enhanced_handler(
            ctx,
            args,
            commission_info,
            order_id,
            trim_rate,
            charge_rate,
            Some(platform_fee_rate),
        )
    }

    pub fn wrap_unwrap_v3<'a>(
        ctx: Context<'_, '_, 'a, 'a, PlatformFeeWrapUnwrapAccounts<'a>>,
        args: PlatformFeeWrapUnwrapArgs,
    ) -> Result<()> {
        instructions::platform_fee_wrap_unwrap_handler_v3(ctx, args)
    }

    /// Wrap/Unwrap with optional specified receiver
    /// - Wrap (SOL -> WSOL): receiver is WSOL token account (ATA)
    /// - Unwrap (WSOL -> SOL): receiver is system account (EOA)
    /// Transfer amount:
    /// - From fee: amount_in
    /// - To fee: amount_in - fees
    pub fn wrap_unwrap_v3_with_receiver<'a>(
        ctx: Context<'_, '_, 'a, 'a, PlatformFeeWrapUnwrapAccountsWithReceiver<'a>>,
        args: PlatformFeeWrapUnwrapArgs,
    ) -> Result<()> {
        instructions::platform_fee_wrap_unwrap_handler_v3_with_receiver(ctx, args)
    }

    pub fn create_token_account<'a>(
        ctx: Context<'_, '_, 'a, 'a, CreateTokenAccountAccounts<'a>>,
        bump: u8,
    ) -> Result<()> {
        instructions::create_token_account_handler(ctx, bump)
    }

    pub fn create_token_account_with_seed<'a>(
        ctx: Context<'_, '_, 'a, 'a, CreateTokenAccountWithSeedAccounts<'a>>,
        bump: u8,
        seed: u32,
    ) -> Result<()> {
        instructions::create_token_account_with_seed_handler(ctx, bump, seed)
    }

    // ******************** Claim ******************** //
    pub fn claim<'a>(ctx: Context<'_, '_, 'a, 'a, ClaimAccounts<'a>>) -> Result<()> {
        instructions::claim_handler(ctx)
    }
}
