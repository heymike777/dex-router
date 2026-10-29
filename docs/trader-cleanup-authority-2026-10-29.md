# Follow-up: mixed guarded production V1 and current deployment review

The sourceea8fec5 candidate now also passes full historical DLMM → CLMM → CLMM
production-wire/native-V1 parity with both guards:1641bytes,252687CU,34calls of34,
all three outputs exact, payernet1610035excluding hypothetical separate setup.
Both stronger-guard controls, CU exhaustion and retired-closer control roll back.
Six older DLMM cases retain EXACT V1wire and all five native mode reports.
See sibling rust-bot/docs/guarded-mixed-route-2026-10-29.md and public baselines.

Fresh finalized review confirms CURRENT on-chain allocation899944ELFbytes while
candidate960008requires60064extension. Temporary buffer4877678840lamports;
additional retained ProgramData rent305125120; PROPOSEDnetworkfeecap10000000.
Peak5192803960, authoritybalance2080068, deficit5190723892:5.2SOLtop-up to
mikeSK7hxrzS1U7xyZ5D411AESvhh8zzEdDyhHL2AYg covers this snapshot. Buffer rent is
recoverable only after successful upgrade; extension rent remains in ProgramData.
The fee cap is a proposal, not an observed fee or user authorization. Source diff
only changes cleanup authority relative to its reviewed local parent; bytewise
build equivalence to the smaller deployed ELF is NOT claimed.

No upgrade or funding performed. Existing unfunded buffer identity preserved;
server signer placeholder absent, documented local key exists/private but was
NOT read. Use the new exact artifact/review, NEVER old October17 target/commands.
USDC ATA was separately authorized and finalized; it is not upgrade/trading
consent. Both bot profit guards unchanged; trading remains disabled.

---

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

Completed offline validation: source ea8fec57195cc50d5aebb7ee1c929e612f39f9f5,
95 unit tests pass; SBF960008bytes, cargo-build-sbf3.1.9/platform-tools1.52,
SHA25685d841610a5835232c7af836425656543c2ed40bee7a30cad57326f964c08c6b.
Six saved observations of ONE DLMM pool tuple/THREE input sizes complete the
FULL11instruction independent V1 recipe,including BOTHguards and rent cleanup.
Exact outputs,payernet1910169–2535228lamports afterroutecosts;separate hypothetical
USDC setup1633440excluded. 6compactfailuresix6/6001,6portfoliofailuresix10/6083,
6oneCUexhaustion and6retiredcloserfailuresix8/6069 allrollbackexceptVMfee.
The retired control has TWO ZERO signatures and an extra5000lamport signature
fee. No real signer,setup,network insideVM,live sends or deployment.

Candidate evidence: /home/arb/validation/guarded-dlmm-candidate-20261029-a.
Rust tools source642851d29764ea86f07e8469ea3ace93e4e5d5c3; gzip candidate and
provenance manifest delivered through GitHub in Rust fixtures. Never use the
fixture as evidence that this contract was deployed. Independent ABI recipe,
hypothetical separateUSDCsetup and retrospective nonpricing context are explicit;
productionwireparity,resourceadmission,canonicalbank and mainnetprofit unproven.
Next upgrade requires explicit authority under current unsigned-only mandate;
do not execute old deployment commands or generate a replacement buffer.

Evidence root for the deployed-code failure:
`/home/arb/validation/guarded-dlmm-replay-20261029-b`.
Original deployed router ELF:
`3eeb2d67fe1bea200e616cbf7056fad5b5150b7c838ec094337aab07ef91e762`,
deployment slot454457828. Inputs are saved market records with a clearly
hypothetical separate USDC ATA setup; this is not live profitability evidence.
