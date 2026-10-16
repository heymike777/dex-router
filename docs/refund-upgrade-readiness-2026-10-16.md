# Refund router upgrade readiness — 16 October 2026

The refund ABI is still **undeployed**. A fresh read-only mainnet check at
finalized slot 457757076 confirmed the expected program/ProgramData link,
upgrade authority and exact old ELF. The local authority key derives to the
on-chain authority; no private key was printed or exported. The Velia observer
was stopped. No buffer, signature, transfer, deployment or trading was performed.

| Item | SOL |
| --- | ---: |
| Authority/deployer balance | 0.002080068 |
| Separate trading payer balance | 3.289850881 |
| Temporary buffer rent | 4.877678840 |
| Permanent ProgramData extension rent | 0.305125120 |
| Fee and operating allowance | 0.100000000 |
| Temporary required authority balance | 5.282803960 |
| Funding deficit at this snapshot | **5.280723892** |

These are fresh snapshot amounts, replacing the historical 11 October funding
estimate. A **5.4 SOL top-up to the deployer** leaves a small margin above this
allowance. This is temporary deployment capital: buffer rent is normally returned
when the upgrade consumes/closes its buffer; extension rent stays in ProgramData.
The 0.1 SOL allowance is not an exact fee quote. Recheck after funding and
immediately before deployment. Trader funds are separate and were not moved.

Authority/deployer: `mikeSK7hxrzS1U7xyZ5D411AESvhh8zzEdDyhHL2AYg`.
Trading payer: `8NZMiChYeGFhrZPSrVMacVXkgvMhK5RvAgQLBcZJUSLp`.
Router: `89P1rihVbww57idhSLHUhxXNkhzYcSghYrbjRoUuivPo`.

The old deployment has 899944 image bytes, SHA-256
`3eeb2d67fe1bea200e616cbf7056fad5b5150b7c838ec094337aab07ef91e762`.
The qualified refund artifact has 960008 bytes, SHA-256
`560560dac2c36615524dced687f6e5ca2657ac7f9a810ddf6b504b1602b23757`;
extension needed is 60064 bytes. Contract source and ELF did not change.

[Snapshot](refund-upgrade-readiness-2026-10-16.json).
[ABI and prior native/VM qualification](arbitrage-refunds-2026-10-11.md).
The Rust repository also contains the 16-pair Pump refund/CU/tip qualification
at `docs/pump-refund-compute-pairs-2026-10-16.md`: it is synthetic offline proof,
not evidence of profitable mainnet execution or universal compute limits.

## Repeatable read-only check

From `/Users/heymike/Projects/arbitrage-bot`, use a NEW output path each time:

```sh
python3 dex-router/tools/check_refund_upgrade.py \
  --artifact dex-router/target/refund-elf/dex_solana.so \
  --authority-keypair keys/mikeSK7hxrzS1U7xyZ5D411AESvhh8zzEdDyhHL2AYg.json \
  --out /tmp/arb-refund-upgrade-readiness-NEXT.json
```

The standard-library checker reads the private RPC URL through SSH alias
`arb-velia`, retaining it only in memory. It loads no trader signing key,
uses `solana-keygen pubkey` only for authority identity, verifies mainnet genesis,
and requests all four accounts in ONE finalized `getMultipleAccounts` snapshot.
Two separate rent-exemption reads price the buffer and target ProgramData size.
It rejects altered links/authority/loader/wallet roles, unqualified local ELFs,
and unrecognized deployed images. Exact target images with zero allocation
padding are recognized. Completed output paths cannot be overwritten.
A successful check process returns a report, **not permission to deploy**;
`funding_sufficient` and `refund_abi_deployed` are explicit separate fields.

Seven offline controls cover funding boundary, rent surplus, padded target,
nonzero image tail, unknown deployment, wrong link/authority/wallet/loader and
mutated local artifact. Run from dex-router:

```sh
python3 -m unittest discover -s tools -p test_check_refund_upgrade.py -v
```

`tools/deploy_four_hop.py` still references the previous server and must NOT be
used unchanged for this upgrade. Prepare the new upgrade from verified committed
source via GitHub, preserve the old image and allocate a single recoverable
buffer, then verify finalized image/authority and buffer closure. No such
state-changing step has been attempted here because funding is insufficient.
After deployment, qualify a new bot installation/profile before enabling
`refund_new_rent`; keep both profit guards, 50% net-before-tip and conservative
live CU unchanged meanwhile. Do not reset the previous financial run or journal.
