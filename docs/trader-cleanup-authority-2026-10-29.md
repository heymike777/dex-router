# Current trader cleanup authority — 29 October 2026

Full native V1 replay of six saved DLMM/DLMM observations reaches both swaps
with exact quoted outputs, then the DEPLOYED router refuses instruction8,
CloseArbitrageIntermediate, with InvalidSigner6069. Its expected closer is the
retired compromised trader8NZMiChYeGFhrZPSrVMacVXkgvMhK5RvAgQLBcZJUSLp, while
the current payer is MikekzKTjw4ny4pMi6QTpF41DHHHAhNei3YNxPWi9mt. All changes
roll back except the VM's145000lamport fee. No mainnet sends occurred.

The source now sets close_authority to the CURRENT trader. The shared constraint
applies to classic intermediate cleanup, v2 cleanup and Pump volume lifecycle
instructions. Claim authority, upgrade authority, router PDA, program ID, ABI,
amount calculation and both profit guards are unchanged. Arbitrary signers do
not gain permission; the retired trader loses cleanup permission after an upgrade.

This source change is NOT deployed. The current mandate prohibits signing/live
sends. Build the candidate ELF and validate it separately in offline native VM,
including both guard-failure controls, rollback and rejection of the retired
closer. Keep the actual deployed-code failure evidence. A candidate build must
be labeled as a counterfactual upgrade, never as captured historical bytecode.

Evidence root for the deployed-code failure:
`/home/arb/validation/guarded-dlmm-replay-20261029-b`.
Original deployed router ELF:
`3eeb2d67fe1bea200e616cbf7056fad5b5150b7c838ec094337aab07ef91e762`,
deployment slot454457828. Inputs are saved market records with a clearly
hypothetical separate USDC ATA setup; this is not live profitability evidence.
