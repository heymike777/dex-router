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
