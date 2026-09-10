# PumpSwap optional pool-v2 parsing

The current PumpSwap fee program can select a zero creator fee while the pool's
coin creator is nonzero. Inferring the remaining-account layout only from the
creator vault authority then consumes buyback/next-hop accounts incorrectly.

Both buy3 and sell3 now recognize pool-v2 by its PDA derived from the base mint.
PumpSwap validates whether the selected fee actually requires it. Existing
cashback prefixes, account privilege construction and compact instruction tags
remain unchanged; no duplicate on-chain fee calculation was introduced.

Validation uses the rebuilt non-staging router with captured PumpSwap/fee/token
ELFs in LiteSVM. The companion arb-rust-bot repository retains source/ELF hashes,
Rust-generated compact instructions and 24 outcomes under
`fixtures/pump-integration-v1/`, with reproduction in
`tools/verify-pump-router.py`. All six synthetic routes succeed with this build;
the previous router rejects the three zero-creator-fee routes. Six excessive
minimum-return and six wrong-tail runs roll back. Prices, wallet/fee accounts and
ALT are synthetic; this is not mainnet profitability or deployment evidence.

Deploy this router correction before enabling the new Rust PumpSwap planner in
production. Cold/warm volume and fee-account rent belongs in the bot's wallet
preparation and still requires acceptance there.
