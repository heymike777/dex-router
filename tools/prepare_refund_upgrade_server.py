"""Prepare one recoverable refund buffer identity and a reviewed command plan.

Never signs with the authority, executes CLI deployment, funds a buffer, changes
the program or enables trading. Raw snapshots/private buffer key stay server-only.
"""
import argparse
import hashlib
import importlib.metadata
import json
import os
from pathlib import Path
import re
import subprocess
import urllib.request
from solders.keypair import Keypair
import check_refund_upgrade as c
import refund_upgrade_plan as p
from install_refund_cli_server import ARCHIVE_SHA

ROOT=Path('/home/arb/validation')
EXECUTION=Path('/home/arb/apps/arbitrage-bot/rust-runtime/execution.json')
sha=lambda path:hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    ap=argparse.ArgumentParser(description=__doc__)
    ap.add_argument('--source',required=True);ap.add_argument('--output',type=Path,required=True)
    ap.add_argument('--toolchain',type=Path,required=True);ap.add_argument('--toolchain-checkpoint',required=True)
    a=ap.parse_args();assert __debug__ and re.fullmatch('[a-f0-9]{40}',a.source) and re.fullmatch('[a-f0-9]{64}',a.toolchain_checkpoint)
    assert os.getuid()!=0 and importlib.metadata.version('solders')=='0.29.0'
    out=a.output;assert out.parent==ROOT and re.fullmatch('refund-upgrade-prepared-[a-z0-9-]+',out.name) and not out.exists()
    repo=Path(__file__).resolve().parents[1]
    source_hashes={}
    for name in ['prepare_refund_upgrade_server.py','check_refund_upgrade.py','refund_upgrade_plan.py','install_refund_cli_server.py']:
        raw=subprocess.check_output(['git','-C',str(repo),'show',a.source+':tools/'+name]);assert raw==Path(__file__).with_name(name).read_bytes()
        source_hashes[name]=hashlib.sha256(raw).hexdigest()
    assert subprocess.run(['pgrep','-x','arb-observe'],capture_output=True).returncode==1
    assert a.toolchain.parent==ROOT
    cp=a.toolchain/'completed-checkpoint.json';assert sha(cp)==a.toolchain_checkpoint
    for name,digest in json.loads(cp.read_text()).items():assert sha(Path(name))==digest
    toolchain=json.loads((a.toolchain/'summary.json').read_text());assert toolchain['archive_sha256']==ARCHIVE_SHA
    elf_path=repo/'artifacts/refund-router-20261011/dex_solana.so';elf=elf_path.read_bytes();assert sha(elf_path)==c.REFUND_SHA and len(elf)==c.REFUND_BYTES
    os.umask(0o077);out.mkdir(mode=0o700)
    # New unfunded buffer key only. Never instantiate/load the funded authority.
    key=Keypair();buffer_pubkey=str(key.pubkey());key_path=out/'buffer-keypair.json'
    with key_path.open('x') as f:json.dump(list(bytes(key)),f)
    assert key_path.stat().st_mode&0o777==0o600
    endpoint=json.loads(EXECUTION.read_text())['rpc_url'];assert endpoint.startswith('https://')
    calls=0
    def rpc(method,params):
        nonlocal calls
        calls+=1
        req=urllib.request.Request(endpoint,data=json.dumps(dict(jsonrpc='2.0',id=calls,method=method,params=params)).encode(),headers={'Content-Type':'application/json'})
        with urllib.request.urlopen(req,timeout=20) as reply:raw=reply.read(4*1024*1024+1)
        assert len(raw)<=4*1024*1024;value=json.loads(raw);assert 'error' not in value
        return value['result']
    assert rpc('getGenesisHash',[])==c.GENESIS
    snapshot=rpc('getMultipleAccounts',[[c.PROGRAM,c.PROGRAMDATA,c.AUTHORITY,c.TRADER,buffer_pubkey],dict(encoding='base64',commitment='finalized')])
    buffer=snapshot['value'][4];assert buffer is None,'new buffer identity already occupied; do not allocate another'
    original=dict(context=snapshot['context'],value=snapshot['value'][:4])
    buffer_rent=rpc('getMinimumBalanceForRentExemption',[37+len(elf),dict(commitment='finalized')])
    program_rent=rpc('getMinimumBalanceForRentExemption',[45+len(elf),dict(commitment='finalized')])
    plan=p.plan(original,elf,buffer,buffer_rent,program_rent)
    before=c.data(original['value'][1])[45:];(out/'program-before.so').write_bytes(before)
    (out/'target.so').write_bytes(elf)
    signing=ROOT/out.name.replace('refund-upgrade-prepared-','refund-upgrade-signing-',1)
    authority=signing/'authority-keypair.json' # Placeholder only: not read, created or copied.
    config=out/'cli-config.json'
    config.write_text(json.dumps(dict(json_rpc_url=endpoint,websocket_url='',keypair_path=str(authority),address_labels={},commitment='finalized'),indent=2)+'\n')
    parsed=subprocess.check_output([str(a.toolchain/'bin/solana'),'--config',str(config),'config','get'],text=True,timeout=20)
    assert endpoint in parsed and str(authority) in parsed
    command_plan=p.commands(a.toolchain/'bin/solana',config,out/'target.so',key_path,authority,plan['extension_chunks'])
    def save(name,value):
        with (out/name).open('x') as f:json.dump(value,f,indent=2);f.write('\n')
    save('snapshot.json',snapshot);save('command-plan.json',command_plan)
    save('private-prepared.json',dict(source=a.source,buffer=buffer_pubkey,artifact_sha256=c.REFUND_SHA,
        authority=c.AUTHORITY,program=c.PROGRAM,toolchain_checkpoint=a.toolchain_checkpoint,plan=plan))
    summary=dict(source=a.source,source_sha256=source_hashes,slot=snapshot['context']['slot'],
        extension_chunks=plan['extension_chunks'],buffer_observed_absent=True,buffer_identity_generated=True,
        buffer_funded=False,authority_signer_loaded=False,authority_signer_present=authority.exists(),
        authority_balance_lamports=plan['authority_balance_lamports'],required_lamports=plan['required_authority_lamports'],
        funding_deficit_lamports=plan['funding_deficit_lamports'],funding_sufficient=plan['funding_sufficient'],
        next_action=plan['next_action'],toolchain_qualified=True,cli_config_parsed=True,artifact_sha256=c.REFUND_SHA,
        mainnet_sends=0,network_rpc_requests=calls,deployed=False,live_refund_enabled=False,production_changed=False)
    save('summary.json',summary)
    pins={str(f):sha(f) for f in out.iterdir() if f.is_file()};save('completed-checkpoint.json',pins)
    print(json.dumps(dict(summary=summary,checkpoint_sha256=sha(out/'completed-checkpoint.json'))))


if __name__=='__main__':
    try:main()
    except Exception:raise SystemExit('Upgrade preparation refused; no authority signature or on-chain operation was performed.') from None
