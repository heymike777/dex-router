# Recoverable refund upgrade prepared on Velia

The upgrade is prepared but **not sent**. Fresh finalized snapshot slot 458020011
still shows the old program, deployer balance 2080068 lamports and a funding
deficit of 5280723892 lamports. Temporary required authority balance remains 5282803960. A
**5.4 SOL** top-up to `mikeSK7hxrzS1U7xyZ5D411AESvhh8zzEdDyhHL2AYg` covers the
current allowance. Trader funds were not moved. The separate trading wallet
does not need another top-up for this contract preparation.

## Recovery fix

After ProgramData extension, the original qualified ELF acquires a zero-filled
tail. The old readiness checker treated the allocation's new hash as an unknown
program, preventing resumption after a completed extension. It now recognizes
only the exact 899944-byte original code prefix plus an entirely zero tail.
Changed prefixes/nonzero tails still refuse. `deployed_elf_sha256` retains its
historical meaning as the allocation hash; the new `deployed_code_sha256` gives
the recognized code identity. Padding recognition never declares refunds deployed.

New pure planning checks validate this authority's buffer header, exact target
allocation, full image or safe partial writes. Foreign owner/authority/content
and underfunded buffers refuse. Existing locked buffer rent is not demanded
again. Completed target with a still-present buffer asks for closure verification,
not a second program deployment. Extension is explicit and split into
`[10240,10240,10240,10240,10240,8864]`, totaling 60064 bytes without excess allocation.

Eighteen readiness/recovery/command-shape controls pass locally and on Velia.
The one-line follow-up at `f7e7daf` makes preparation report observed deployment
status rather than a hard-coded false value; the actual prepared old-program
snapshot/result remains unchanged. No pricing, quote, Sender or hot-path RPC
behavior changed.

## Qualified tools and private preparation

Agave CLI 3.1.9 matches the already installed laptop CLI. Its Linux artifact was
downloaded from the [official Agave release](https://github.com/anza-xyz/agave/releases/tag/v3.1.9)
with pinned archive length **214999945 bytes**, SHA-256
`8cffaecc3a5f3c47c8d8f94ea739c716fbae08683018fc29b65c3045a0e9bdd2`.
Only regular-file CLI/keygen entries were extracted; no installer shell script,
validator/service, PATH or default config change. Version and required CLI
capabilities passed. The prepared JSON CLI config was parsed by the installed
CLI with an offline `config get`; its private RPC URL is not in command arguments
or exported reports.

Toolchain path: `/home/arb/validation/agave-cli-3-1-9-20261017-a`.
Completed checkpoint:
`83db4608a7337c289ec385341a78a11c46379a6fb92f5c8fb48ea933a58d52ec`.
CLI SHA `d9b8059481db0c7054fb1d15ed1454127a24cb0f568fb68ab1429bafed5538d5`;
keygen SHA `9a698b14260b6a49faccffe40b0d97dd4eef5377517fce1ef9fe6c75b77eb678`.

Preparation path: `/home/arb/validation/refund-upgrade-prepared-20261017-a`.
It retains one new private buffer keypair, its explicitly observed absent account,
the original ELF, exact qualified target, private finalized snapshot, CLI config
and `command-plan.json`. The key file is 0600 in a 0700 validation directory.
The buffer is **not funded or created on-chain**. Only its unfunded identity was
generated; no authority/trader signing key was loaded or transferred to Velia.
The authority public identity was separately verified from the existing local key.
Server authority signing-path placeholder is in a separate future signing
directory, outside immutable preparation evidence, and is currently absent.
Never commit signing keys or private config to GitHub.

Prepared checkpoint:
`e31e7bbf6f6cea19f6ccae95e36d65d709a36f025cf1697b7ae29cebfabe9bac`.
Actual tool source: `605a1793f30d2dbcc7c07c331f075e0c856d1862`.
Four read-only RPC requests were used; zero authority signatures, transfers,
buffer writes, extensions, upgrades or trading sends. Raw snapshots/config,
buffer identity/key and full command plan remain SERVER ONLY. Do not download
them or generate a replacement buffer to resume this preparation.

The current commands are in
`/home/arb/validation/refund-upgrade-reviewed-20261017-a/command-plan.json`.
Review source `103305ce4fd86098fe4edd1191c1ede1a7009b6b` adds an explicit
`--max-len 960008` to the write-buffer command so allocation matches the quoted
rent exactly. It verifies the original preparation/toolchain checkpoints,
reuses the SAME buffer/config/target and changes no extension/deploy command.
It generates no new buffer, loads no signer and makes no RPC request or send.
The original prepared command file remains immutable historical evidence.
Reviewed checkpoint:
`2e3091b393e269c14136066dd5b0e4f2af490debe3ff0aeb24b2a2bbe3de875f`.
The current eighteen controls also pass on this source locally and on Velia.

## Execution remains a separate step

The preparer does not execute commands or automatically retry transactions.
The fixed command plan uses this buffer, RPC writes, bounded signing attempts,
explicit authority/fee payer and finalized commitment. Feature/preflight checks
are retained for deployment. Final deploy has no program-file upload and uses
`--no-auto-extend`; it consumes the already verified on-chain buffer.

Once funded, recheck mainnet program/link/authority, allocation, fresh rents,
balance, target hash and signer identity. Use the existing local authority key
with an appropriate secure signing setup; server signing has not been provisioned.
Write the SAME buffer, verify its full bytes/authority at finalized commitment,
extend one chunk at a time with verification, then upgrade. Inspect ambiguous
CLI outcomes before any retry; a nonzero exit is not proof nothing landed.
Do not invent a new buffer or reset state to bypass UNKNOWN. Verify finalized
target image, authority and buffer closure before bot profile qualification or
enabling `refund_new_rent`. Both guards and 50% tip remain unchanged.

Fresh production inspection found clean source `dc37dc0e`, the original binaries,
no observer, stopped journal with held [0,0] and one historical dispatch. Last
financial predecessor remains `rust-signed-stages-live-20261017-a`; this phase
adds zero mainnet fees/sends. Its existing financial protection remains the
native V1 qualification checkpoint 427526b3...2911. The toolchain and prepared
checkpoints and the reviewed plan above are additional sibling evidence, not financial resets. Merge
their maps with exact overlapping hashes and pin each checkpoint itself.

Independent native V1 mixed-wallet qualification remains in Rust repository
`docs/refund-mixed-v1-2026-10-17.md`: 4 successes/8 exact hops/6 negative controls,
1456 bytes/268255 CU with both guards. Those remain conditional VM results;
profitable mainnet execution is still unproven.
