# Refund upgrade funding — 17 October 2026

Fresh read-only server check at finalized slot **458009333** confirms the old
deployed router image and expected program/ProgramData/authority/wallet roles.
The refund ABI remains undeployed. No signing key, buffer, transfer, upgrade or
trading was used. The trading wallet was not used to fund the deployer.

| Item | Lamports |
| --- | ---: |
| Authority balance | 2,080,068 |
| Trading payer balance, separate | 3,242,210,881 |
| Temporary buffer rent | 4,877,678,840 |
| Permanent ProgramData extension rent | 305,125,120 |
| Fee/operating allowance | 100,000,000 |
| Required authority balance | 5,282,803,960 |
| Funding deficit | **5,280,723,892** |

Suggested deployer top-up: **5.4 SOL** to
`mikeSK7hxrzS1U7xyZ5D411AESvhh8zzEdDyhHL2AYg`.
Buffer rent is normally recovered when consumed/closed; extension rent remains
in ProgramData. The allowance is not an exact fee quote. Recheck immediately
before upgrading. No funds were moved.

Target identity: 960008 bytes,
`560560dac2c36615524dced687f6e5ca2657ac7f9a810ddf6b504b1602b23757`.
Deployed identity: 899944 bytes,
`3eeb2d67fe1bea200e616cbf7056fad5b5150b7c838ec094337aab07ef91e762`.
Required extension: 60064 bytes. The target ELF and implementation were not
changed. Its earlier synthetic Pump full-wallet qualification remains distinct
from exact mixed DLMM/Pump qualification and mainnet profitability.

The new `tools/check_refund_funding_server.py` was delivered through GitHub at
`9a8eff55263d166e6a1fbbcb15fb2e3b63044574`, into an isolated source checkout.
It uses four bounded read-only requests: genesis, ONE finalized four-account
snapshot and two rent prices. Private RPC URL and account bytes stay on Velia;
stdout contains only selected scalars and hashes. Nine pure controls pass.
This funding/image check does not load or attest the target artifact or local
signer key on the server. Verify both separately before deployment. Prefer it
over the older laptop checker, which transfers full RPC replies to the laptop.

Private evidence `/home/arb/validation/refund-funding-20261017-a` includes the
raw snapshot, rents, selected summary and completed-checkpoint manifest.
Checkpoint SHA-256:
`e7a8d730b9b91ff3ccde9793e8454898cb6f4d980f30e05405385cd2df55b376`.
Existing completed outputs are not overwritten. Run a future check under arb,
from a new GitHub-delivered source checkout, with a NEW output directory:

```sh
python3 tools/check_refund_funding_server.py --source <committed-source-sha> \
  --output /home/arb/validation/refund-funding-NEXT
```

Replace NEXT with a lowercase dated unique name. Never use
`tools/deploy_four_hop.py` unchanged: it targets the previous server. Prepare a
recoverable single-buffer Velia upgrade, verify finalized ELF/authority/buffer
closure, and qualify the new bot profile before enabling `refund_new_rent`.
Both guards, 50% net-before-tip and conservative mixed-route CU stay enabled.
