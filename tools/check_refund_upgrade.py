"""Read-only mainnet refund upgrade readiness. Never signs or creates a buffer.

Obtain the RPC URL through the existing Velia SSH alias, retaining it in memory.
Output contains public account roles, hashes and funding amounts only.
"""
import argparse
import base64
import hashlib
import json
import pathlib
import subprocess
import urllib.request

PROGRAM = "89P1rihVbww57idhSLHUhxXNkhzYcSghYrbjRoUuivPo"
PROGRAMDATA = "4qDn42Nyprcpeb5XPFRntQPBieZFdL3jEfMzQAFnzKZp"
AUTHORITY = "mikeSK7hxrzS1U7xyZ5D411AESvhh8zzEdDyhHL2AYg"
TRADER = "8NZMiChYeGFhrZPSrVMacVXkgvMhK5RvAgQLBcZJUSLp"
LOADER = "BPFLoaderUpgradeab1e11111111111111111111111"
SYSTEM = "11111111111111111111111111111111"
GENESIS = "5eykt4UsFv8P8NJdTREpY1vzqKqZKvdpKuc147dw2N9d"
OLD_SHA = "3eeb2d67fe1bea200e616cbf7056fad5b5150b7c838ec094337aab07ef91e762"
REFUND_SHA = "560560dac2c36615524dced687f6e5ca2657ac7f9a810ddf6b504b1602b23757"
ALPHABET = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz"


def require(condition, message):
    if not condition:
        raise ValueError(message)


def base58(raw):
    zeros = len(raw) - len(raw.lstrip(b"\0"))
    value = int.from_bytes(raw, "big")
    text = ""
    while value:
        value, digit = divmod(value, 58)
        text = ALPHABET[digit] + text
    return "1" * zeros + text


def data(account):
    require(account is not None, "required account absent")
    require(account["data"][1] == "base64", "unexpected account encoding")
    return base64.b64decode(account["data"][0], validate=True)


def assess(snapshot, elf, buffer_rent, target_program_rent, reserve):
    """Validate one finalized snapshot and compute temporary capital, not profit."""
    require(hashlib.sha256(elf).hexdigest() == REFUND_SHA, "unqualified refund ELF")
    program, programdata, authority, trader = snapshot["value"]
    require(program["owner"] == programdata["owner"] == LOADER, "wrong loader")
    require(program["executable"] and not programdata["executable"], "wrong executable flags")
    program_bytes, raw = data(program), data(programdata)
    require(len(program_bytes) == 36 and program_bytes[:4] == (2).to_bytes(4, "little"), "invalid program header")
    require(base58(program_bytes[4:]) == PROGRAMDATA, "ProgramData link changed")
    require(len(raw) >= 45 and raw[:4] == (3).to_bytes(4, "little") and raw[12] == 1, "invalid ProgramData header")
    require(base58(raw[13:45]) == AUTHORITY, "upgrade authority changed")
    for account in (authority, trader):
        require(account is not None and account["owner"] == SYSTEM and not account["executable"] and data(account) == b"", "wallet account role changed")
    image = raw[45:]
    old_sha = hashlib.sha256(image).hexdigest()
    deployed = image[:len(elf)] == elf and not any(image[len(elf):])
    require(deployed or old_sha == OLD_SHA, "unrecognized deployed ELF; review before upgrade")
    extension = max(0, len(elf) - len(image))
    extension_rent = max(0, target_program_rent - programdata["lamports"]) if extension else 0
    temporary = 0 if deployed else buffer_rent + extension_rent + reserve
    deficit = max(0, temporary - authority["lamports"])
    return {
        "schema": 1, "read_only": True, "commitment": "finalized",
        "slot": snapshot["context"]["slot"], "genesis": GENESIS,
        "program": PROGRAM, "programdata": PROGRAMDATA,
        "upgrade_authority": AUTHORITY, "trading_payer": TRADER,
        "deployment_slot": int.from_bytes(raw[4:12], "little"),
        "deployed_elf_sha256": old_sha, "deployed_capacity_bytes": len(image),
        "refund_elf_sha256": REFUND_SHA, "refund_elf_bytes": len(elf),
        "refund_abi_deployed": deployed, "extension_bytes": extension,
        "buffer_rent_lamports": buffer_rent,
        "extension_rent_lamports": extension_rent,
        "fee_and_operating_reserve_lamports": reserve,
        "temporary_required_lamports": temporary,
        "authority_balance_lamports": authority["lamports"],
        "trader_balance_lamports": trader["lamports"],
        "funding_deficit_lamports": deficit,
        "funding_sufficient": deficit == 0,
        "note": "No buffer, signature, transfer, deployment or trading. Reserve is an allowance, not an exact fee quote. Recheck immediately before deployment; refund activation needs separate qualification.",
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--artifact", type=pathlib.Path, required=True)
    parser.add_argument("--authority-keypair", type=pathlib.Path, required=True)
    parser.add_argument("--out", type=pathlib.Path, required=True)
    args = parser.parse_args()
    require(not args.out.exists(), "refusing to overwrite readiness evidence")
    elf = args.artifact.read_bytes()
    require(hashlib.sha256(elf).hexdigest() == REFUND_SHA, "unqualified refund ELF")
    public = subprocess.run(["solana-keygen", "pubkey", str(args.authority_keypair)], check=True, capture_output=True, text=True).stdout.strip()
    require(public == AUTHORITY, "configured signer is not router authority")
    # Existing remote configuration is read only; no source is uploaded to Velia.
    remote = "sudo -n -u arb python3 -c 'import json; print(json.load(open(\"/home/arb/apps/arbitrage-bot/rust-runtime/execution.json\"))[\"rpc_url\"])'"
    url = subprocess.run(["ssh", "-o", "BatchMode=yes", "arb-velia", remote], check=True, capture_output=True, text=True).stdout.strip()
    require(url.startswith("https://"), "RPC must use HTTPS")

    def rpc(method, params):
        request = urllib.request.Request(url, data=json.dumps({"jsonrpc": "2.0", "id": 1, "method": method, "params": params}).encode(), headers={"Content-Type": "application/json"})
        try:
            with urllib.request.urlopen(request, timeout=30) as response:
                result = json.load(response)
        except Exception:
            raise RuntimeError("RPC transport failed: " + method) from None
        require("error" not in result and "result" in result, "RPC rejected: " + method)
        return result["result"]

    require(rpc("getGenesisHash", []) == GENESIS, "not Solana mainnet")
    snapshot = rpc("getMultipleAccounts", [[PROGRAM, PROGRAMDATA, AUTHORITY, TRADER], {"encoding": "base64", "commitment": "finalized"}])
    buffer_rent = rpc("getMinimumBalanceForRentExemption", [37 + len(elf), {"commitment": "finalized"}])
    program_rent = rpc("getMinimumBalanceForRentExemption", [45 + len(elf), {"commitment": "finalized"}])
    report = assess(snapshot, elf, buffer_rent, program_rent, 100_000_000)
    args.out.parent.mkdir(parents=True, exist_ok=True)
    # Exclusive creation keeps completed reports immutable on subsequent runs.
    with args.out.open("x") as handle:
        handle.write(json.dumps(report, indent=2) + "\n")
    print(json.dumps(report))


if __name__ == "__main__":
    try:
        main()
    except (ValueError, RuntimeError) as error:
        raise SystemExit(str(error)) from None
    except Exception:
        # Never emit a transport/config exception that might contain the RPC URL.
        raise SystemExit("Readiness check failed; no on-chain action was performed.") from None
