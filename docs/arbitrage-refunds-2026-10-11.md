# New-account rent refund ABI

Adds `close_arbitrage_intermediate_v2`, `prepare_arbitrage_pump_volume` and
`close_arbitrage_pump_volume`; old ABIs, min-return and portfolio guards remain.
Only the configured closer may call these instructions. Token close requires an
earlier non-idempotent ATA Create with matching payer, owner, mint and program;
it harvests withheld Token-2022 fees to the linked mint before closing an empty
account. Pump prepare rejects existing/prefunded state; close requires its
matching earlier top-level preparation, signs the official Pump close CPI and
relays exactly the returned lamports from the router PDA. Old PDA SOL is not swept.
Instructions-sysvar introspection refuses using top-level creation as proof for
an unrelated inner close CPI. Runtime account/program/mint constraints are checked.

95 native tests pass. Rust-generated full wallet instructions with this SBF ELF
passed 76 independent VM cases (16 synthetic successes/60 failure+rollback
controls), including both guards. Evidence and detailed limitations live in
`arb-rust-bot/fixtures/arbitrage-refunds-v1` and its rent-refund document. These
synthetic messages use V0 with an ALT, not a native V1 VM proof or mainnet profit.

ELF 960008 bytes, SHA-256
`560560dac2c36615524dced687f6e5ca2657ac7f9a810ddf6b504b1602b23757`.
Existing deployment remains `3eeb2d67fe1bea200e616cbf7056fad5b5150b7c838ec094337aab07ef91e762`.
Upgrade is pending deployer funding; Rust feature defaults false and has not
been enabled on the server. Target program is `89P1rih…`, not `earnLL…`.
