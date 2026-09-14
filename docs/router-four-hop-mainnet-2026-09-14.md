# Router mainnet upgrade — 14 September 2026

Finalized upgrade of `89P1rihVbww57idhSLHUhxXNkhzYcSghYrbjRoUuivPo` at slot **447050617**. Transaction: `gzPaJ2Xqmsh7F4nvL6NLCNaX71Gtd3DEwBkKtgP1BzdKRa2mibwMDgLwZwbMtkMFEGho7JDqhmJt4ME86mBCJqf`.

The router accepts up to four hops. The pending `close_arbitrage_intermediate` instruction is now included. Atomic min-return and net-profit guards remain enabled; upgrade authority is unchanged. No state-layout migration was needed.

ELF: 874912 bytes, SHA-256 `a9547680ab32f19cd7e83090ce7b95108ccffff8e0cec4e9e703a7dde232bda2`. Finalized ProgramData matches every ELF byte and has only zero padding afterward. ProgramData extended by 10,240 bytes (loader minimum); the initial 7,640-byte request was rejected at simulation before any send. Deployment buffer is closed.

Contract build source: `2a6af848f665668ef4636585017793ae54e769b5`. Rust four-hop encoder source: `596e3b50e2af21651ed212eeb2050d8fbd255e65`. Both server checkouts were updated through GitHub. Compatible arb-observe and arb-cli were built from that Rust source in the shared debug target. No trading process was started. Server arb-observe SHA-256: `734110ac2f2e38190ed78c367b2808c024af10f533e63c197763a473922b7cbf`.

Validation: three contract expansion/guard unit tests, eight Rust encoder tests, and actual router ELF execution in LiteSVM with deterministic mock DEXs. Four-hop compact and legacy cycles completed; min-return loss and fee/tip-adjusted profit failure reverted atomically. These fixtures demonstrate execution mechanics, not profitable real-market arbitrage.

Upgrade-authority balance changed from 6.760890880 to 7.791077784 SOL. ProgramData released 1.034889385 SOL of previously locked excess rent. Balance-reconciled transaction cost: 0.004702481 SOL. Buffer funding was temporary and refunded. This balance increase is rent recovery, not trading profit.

Public details are in `router-four-hop-mainnet-2026-09-14.json`. Local recovery artifacts, including the previous ELF, are kept under `/Users/heymike/Projects/arbitrage-bot/deployment-artifacts/router-four-hop-20260914`; private buffer key material remains outside Git.
