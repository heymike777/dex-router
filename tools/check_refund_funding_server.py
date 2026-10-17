"""Read-only funding check on Velia. Raw RPC replies never leave the server.

Pins the previously qualified target identity, not the presence of a new ELF.
No signer, transfer, buffer, upgrade, trading, or runtime mutation.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import urllib.request
import check_refund_upgrade as check

EXECUTION = Path('/home/arb/apps/arbitrage-bot/rust-runtime/execution.json')
ROOT = Path('/home/arb/validation')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source', required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    check.require(re.fullmatch('[a-f0-9]{40}', args.source), 'invalid source')
    output = args.output
    check.require(output.parent == ROOT and re.fullmatch('refund-funding-[a-z0-9-]+', output.name), 'invalid output')
    check.require(not output.exists() and os.getuid() != 0, 'new non-root output required')
    os.umask(0o077)
    repo = Path(__file__).resolve().parents[1]
    files = ['tools/check_refund_upgrade.py', 'tools/check_refund_funding_server.py']
    hashes = {}
    for name in files:
        committed = subprocess.check_output(['git', '-C', str(repo), 'show', args.source + ':' + name])
        check.require(committed == (repo / name).read_bytes(), 'source bytes changed')
        hashes[name] = hashlib.sha256(committed).hexdigest()
    check.require(subprocess.run(['pgrep', '-x', 'arb-observe'], capture_output=True).returncode == 1, 'observer active')
    output.mkdir(mode=0o700)
    endpoint = json.loads(EXECUTION.read_text())['rpc_url']
    check.require(endpoint.startswith('https://'), 'HTTPS required')
    requests = 0

    def save(name, value):
        with (output / name).open('x') as file:
            json.dump(value, file, indent=2)
            file.write('\n')

    def rpc(method, params):
        nonlocal requests
        requests += 1
        request = urllib.request.Request(endpoint, data=json.dumps(dict(
            jsonrpc='2.0', id=requests, method=method, params=params)).encode(),
            headers={'Content-Type': 'application/json'})
        with urllib.request.urlopen(request, timeout=20) as response:
            raw = response.read(4 * 1024 * 1024 + 1)
        check.require(len(raw) <= 4 * 1024 * 1024, 'oversized RPC reply')
        value = json.loads(raw)
        check.require('error' not in value and 'result' in value, 'RPC rejected')
        return value['result']

    check.require(rpc('getGenesisHash', []) == check.GENESIS, 'wrong genesis')
    snapshot = rpc('getMultipleAccounts', [[check.PROGRAM, check.PROGRAMDATA, check.AUTHORITY, check.TRADER],
        dict(encoding='base64', commitment='finalized')])
    buffer_rent = rpc('getMinimumBalanceForRentExemption', [37 + check.REFUND_BYTES, dict(commitment='finalized')])
    program_rent = rpc('getMinimumBalanceForRentExemption', [45 + check.REFUND_BYTES, dict(commitment='finalized')])
    report = check.assess_identity(snapshot, check.REFUND_BYTES, check.REFUND_SHA, buffer_rent, program_rent, 100_000_000)
    save('snapshot.json', snapshot)
    save('rents.json', dict(buffer=buffer_rent, programdata=program_rent))
    # Export only explicitly selected scalars and opaque hashes, never account data.
    fields = ['slot', 'deployed_elf_sha256', 'deployed_capacity_bytes', 'refund_elf_sha256',
              'refund_elf_bytes', 'refund_abi_deployed', 'extension_bytes', 'buffer_rent_lamports',
              'extension_rent_lamports', 'fee_and_operating_reserve_lamports', 'temporary_required_lamports',
              'authority_balance_lamports', 'trader_balance_lamports', 'funding_deficit_lamports', 'funding_sufficient']
    public = {key: report[key] for key in fields}
    public.update(source=args.source, source_sha256=hashes, network_requests=requests,
                  new_sends=0, signer_instantiated=False, local_artifact_checked=False,
                  local_signer_identity_checked=False, production_changed=False)
    save('summary.json', public)
    checkpoint = {str(file): hashlib.sha256(file.read_bytes()).hexdigest() for file in sorted(output.iterdir())}
    save('completed-checkpoint.json', checkpoint)
    print(json.dumps(dict(summary=public, checkpoint_sha256=hashlib.sha256((output / 'completed-checkpoint.json').read_bytes()).hexdigest())))


if __name__ == '__main__':
    try:
        main()
    except Exception:
        raise SystemExit('Server funding check failed; no on-chain action was performed.') from None
