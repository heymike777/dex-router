# 2026-10-29 — Cleanup contract DEPLOYED and finalized; upgrade grant fulfilled; trading OFF

Program89P1rihVbww57idhSLHUhxXNkhzYcSghYrbjRoUuivPo upgraded at slot462899354.
Signature67RvkR8beQAYUVXTqLeB1My55seCg2XoL5ERdUhXamXoZLo1T9E3n82y4EoKPNS14jsCSsBaW9xrg7Tdy1toyRAR.
ELF960008bytes SHA85d841610a5835232c7af836425656543c2ed40bee7a30cad57326f964c08c6b
(sourceea8fec57195cc50d5aebb7ee1c929e612f39f9f5) matches the EXACT on-chain prefix.
Capacity961384bytes includes1376STRICTZEROtail; allocated payloadSHA
34f993f8ff75c81d04d57d821b8f5ef0d944171584ea88c9264d857139085cd7.
Do NOT confuse code prefix85...with full ProgramData payload34f993...in runtime generation pins.
AuthoritymikeSK7hxrzS1U7xyZ5D411AESvhh8zzEdDyhHL2AYg unchanged/upgradeable.
Samebuffer3xx...CLOSED,4877719480lamports returned; no new buffer identity.
Actual TOTALnetworkfees5045000<=10000000cap, retained extensionrent312115200.
Deployer4914919868lamports AFTERupgrade; trader7777555untouched. Human intends to return
SOLto trader themselves. No transfer by agent, no arbitrage sends, no live enrollment.
TemporaryserverauthoritykeyREMOVED; local original and historicalbufferkey retained privately.

Read baseline/current-trader-cleanup-deployment-2026-10-29.json and contract
 docs/deployments/2026-10-29-current-trader-cleanup.{json,html}. Private immutable ledger:
/home/arb/validation/current-trader-cleanup-deploy-20261029-a (old-router backup retained).
Initialwrite expired partly:950writes+initial fee4760000, SAMEbuffer resumed only after
finalizedexpiry reconciliation;50writes250000more. Five10240extensions passed;8864final
step failed preflight/-32002/invalidargument withoutfee. Mainnet minimum is10240; pure
planner updated through GitHub04bbe84c8001100686c0588876a676224d471a7e,29testsPASS Mac/Linux.
Sixth10240step passed. Roundedextra1376bytes add6990080retainedrent, within funded capital.
No fee ledger reset, no bypass of preflight/feature checks, no authority transfer/freeze.
Nested pure-plan mainnet_sends:0 is NOT an execution counter; actual loader sends occurred.
Public deployment receipt explicitly distinguishes loader transactions from arbitrage sends0.

The prior upgrade-only grant is now FULFILLED; do not redeploy/reprovision signer or wait
for old funding question. Both profit guards preserved; DO NOT start trading from this
upgrade grant. Keep Original compute policy; empirical router/CU profiles pinned to old
3eeb...or candidate85...are NOT automatically qualified for allocatedpayload34f993....
After trader funding, next evidence step is fresh unsigned LaserStream shadow recognizing
new deployed code; no on-chain arbitrage profitability claim. Installed app bins/runtime
were not changed by the loader upgrade. Older pending-funding/NOTDEPLOYED notes below
are historical and superseded by this section.

---

# Current-trader cleanup: fresh PumpSwap/DLMM proof and conditional upgrade grant

Candidate source ea8fec57195cc50d5aebb7ee1c929e612f39f9f5 remains unchanged.
ELF960008bytes/SHA85d841610a5835232c7af836425656543c2ed40bee7a30cad57326f964c08c6b.
Program89P1rihVbww57idhSLHUhxXNkhzYcSghYrbjRoUuivPo, authority
mikeSK7hxrzS1U7xyZ5D411AESvhh8zzEdDyhHL2AYg. No deployment yet.

A separate Rust1.97.1/LiteSVM0.18 tool executed FOUR ORIGINAL unsigned V1
captures from the fresh 180-second main-LaserStream shadow, representing TWO
route identities: PumpSwap->DLMM and DLMM->PumpSwap->DLMM. Source/market,
Clock/Rent, original wire, fee, tip and lifetime were retained. Only the VM-loaded
router ELF changed. All four original deployed reports remain field-identical
and fail cleanup6069 after exact swap outputs. The candidate fully succeeds:
virtual net after quoted fee/tip/retained rent is270619,270619,7909030,2529980
lamports. All new close targets disappear; existing USDC ATA stays byte/owner/
lamport identical. Both guard controls and one-CU exhaustion roll back watched
accounts except the real VM fee. Compact minimum binds complete original costs
and positive profit floor. These are conditional VM results, not live profits.

442 native test executions across all targets and strict Clippy pass. Four full
default reports match their sealed original reports. Fourteen altered proof
exports refuse even after updating the manifest hash. Missing key-only readonly
accounts remain2per case; signature/blockhash checks are disabled for the saved
unsigned lifetime. Runtime features/default sysvars and canonical bank coexistence
remain unproven. No code/state patch is applied to the captures themselves.
Details and scalar receipts are in sibling rust-bot docs/current-pump-dlmm-
candidate-2026-10-29.md and baseline/current-pump-dlmm-candidate-*.json.

## Human authorization received, funding condition not yet met

Human explicitly answered: **«Да, только upgrade после пополнения»** to the
reviewed cleanup upgrade, proposed fee cap0.01SOL, no trading. This supersedes
older statements that upgrade approval is pending. It does not authorize trading.
The subsequent finalized balance check still shows2080068lamports. No buffer
funding, extension, signing/send or upgrade has occurred. The local authority
public key was verified; its key has not been transferred to the server.

Current temporary buffer rent4877678840, retained ProgramData extension rent
305125120, approved network fee cap10000000 =>peak5192803960lamports. Round top-up
5.2SOL. Existing buffer3xxmsbhEuoDfi4yNxWNHnuVyREcuBF19zx68HWBjXTQK remains absent;
reuse its existing private identity, never create a replacement or use old56...
ELF/old command plan. Six explicit extension chunks total60064bytes.

The new PURE current_trader_upgrade_plan.py targets only85..., validates old
code plus strictly zero extension tails, same-authority exact/partial buffers,
actual remaining fee allowance and no redeploy after completed upgrade. It does
not execute commands or grant authority from JSON. It uses zero CU price and one
signing pass for loader writes. --use-quic selects the CLI implementation that
honors --use-rpc; no TPU client is then created. Preflight/feature checks remain.
See official pinned [Agave3.1.9 program implementation](https://raw.githubusercontent.com/anza-xyz/agave/v3.1.9/cli/src/program.rs)
and [parallel sender](https://raw.githubusercontent.com/anza-xyz/agave/v3.1.9/client/src/send_and_confirm_transactions_in_parallel.rs).
The earlier50000microLamport plan is NOT compatible with this new0.01SOL cap.

CLI does not enforce a total fee cap itself. Before execution, derive/quote the
actual write-message fees, verify one-signing-pass bound and reserve the remaining
cap for writes/each extension/final upgrade. Record actual fees across all resumes;
never reset them or blindly rerun after UNKNOWN. Verify finalized buffer bytes,
authority, allocation after each chunk, final code/authority and buffer closure.
No authority transfer/--final/automatic extension or trader-fund transfer is allowed.

The local27 readiness/planning tests (9new,18prior) pass. Fresh execution-time
identity/rent/fee/signer/buffer checks remain required after funding. Generic
continuation must not start trading. Existing empirical CU profiles remain pinned
to the older router; keep Original CU policy until a separate candidate matrix
qualifies any cuts. Seven-DEX full-mode coverage/live profit remains unproven.

## Execution-time mainnet minimum extension step

After funding, the original buffer identity was reused and its exact target ELF
was fully verified. Five10240-byte extensions finalized. The old final8864-byte
step was rejected by RPC preflight with `ExtendProgram requires a minimum of
10240 additional bytes`; no on-chain fee/allocation change for that rejected step.
The pure planner now rounds every required extension to10240bytes. From the
pinned original899944-byte allocation, the maximum reviewed capacity is961384:
960008unchanged ELF bytes plus1376strictly ZERO bytes. Both an unknown/nonzero
tail and an allocation beyond this bound refuse. A candidate with only the
bounded zero tail is verified as complete, never redeployed. Commands still use
one signing pass/zero priority fee/RPC writes/no-auto-extend/no-final.

The extra1376bytes retain6990080lamports more ProgramData rent than the old plan.
Total retained extension rent is312115200lamports; actual temporary buffer rent
is4877719480lamports (CLI reserves the ProgramData-sized rent). The funded account
covers this within the original5.2SOLtop-up; network-fee cap remains10000000.
ELF target85.../sourceea8...and both profit guards are unchanged. Local pure
readiness/planning tests29PASS; execution must reconcile the rejected preflight
and continue the SAMEbuffer/fee ledger, never restart funding or overwrite evidence.
