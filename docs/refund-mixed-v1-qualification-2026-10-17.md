# Exact mixed route qualification and artifact delivery

The qualified refund image is now retained in GitHub at commit
`fc857e3c864ad52c807852ae0f08feaa1c59f0eb`:
`artifacts/refund-router-20261011/dex_solana.so`, with its public manifest.
SHA-256 `560560dac2c36615524dced687f6e5ca2657ac7f9a810ddf6b504b1602b23757`,
960008 bytes, contract source `5e464eb56f3a5816faca4b7dee0b5e2356985a74`.
Current program sources/Cargo inputs still match that commit. This is delivery
of the previously source-built/qualified artifact, not a new build or deployment.
No private keypair is included.

A server-side exact historical DLMM → PumpSwap route now has native Rust planner
and native **V1** execution qualification. The planner reproduces every original
cost/output and identifies one absent eligible Pump volume PDA without a
volume-owned fee/cashback ATA. Rent 1346200 moves from permanent expense to
temporary refundable capital. The old authority account remains unchanged.

LiteSVM 0.18.0 independently executes ten inline V1 cases: four successes/eight
exact hop outputs and six specific failures with rollback. Both guards remain.
At 145000 actual network fee, conditional payer gain is 767353 with the original
wallet, 2113553 with refund/original tip, and 1440453 with refund/50% tip. The
refund transaction is 1456 bytes, consumes 268255 CU and uses 35 invocations,
below its native bound of 39. Original wallet behavior is identical on old/new
router. Old ABI refusal, missing preparation, pre-existing volume, both guard
floors and actual 10000-CU exhaustion are negative controls.

These are **unsigned VM counterfactuals**, not profitable mainnet trades or full
canonical bank proof. Original signature is independently verified; zero-signature
variants bypass cryptographic/blockhash checks. Two unknown read-only rows remain
explicit and the exact historical feature set is not attested. Keep the 1.4M CU
fallback; do not generalize one route to all pool classes. Existing original
signed proofs and the refused Python experiments remain preserved.

See arb-rust-bot `docs/refund-mixed-v1-2026-10-17.md` and its baseline manifest.
Accepted private server checkpoint:
`427526b3dd9124bc47c0f6e0bf2c0550ef769e522b85a8613593cd6ff0c72911`.
Rust diagnostic source `4beba48b0067bfeaf969310c25ec032323494298`, binary
`d919b0025c7567d774dbb7c2f5bf8b4cc38cf591b107a8fc72f4e0cc375c2cac`.
Thirteen native planner and twelve native VM provenance controls pass, with
strict Clippy/release builds on Velia.

Deployment remains pending. Latest balance check slot458016618: deployer has
2080068 lamports; last full readiness requires 5282803960 temporarily. Suggested
top-up is **5.4 SOL** to `mikeSK7hxrzS1U7xyZ5D411AESvhh8zzEdDyhHL2AYg`.
Recheck full finalized readiness and signer identity immediately before upgrade.
Use one recoverable buffer, verify finalized ELF/authority/buffer closure, then
qualify the installed Rust profile before enabling `refund_new_rent` or trading.
No on-chain action, production change or trader funding transfer occurred here.
