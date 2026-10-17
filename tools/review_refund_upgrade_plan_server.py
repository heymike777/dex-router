"""Review the existing unfunded buffer plan without regenerating any identity.

Private commands in a NEW output; original prepared files are immutable.
No RPC, signer/key loading or subprocess deployment.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import refund_upgrade_plan as p

ROOT=Path('/home/arb/validation')
PREPARED=ROOT/'refund-upgrade-prepared-20261017-a'
PREPARED_CP='e31e7bbf6f6cea19f6ccae95e36d65d709a36f025cf1697b7ae29cebfabe9bac'
TOOLCHAIN=ROOT/'agave-cli-3-1-9-20261017-a'
CLI_CP='83db4608a7337c289ec385341a78a11c46379a6fb92f5c8fb48ea933a58d52ec'
sha=lambda path:hashlib.sha256(path.read_bytes()).hexdigest()


def verify(directory,digest):
    cp=directory/'completed-checkpoint.json';assert sha(cp)==digest
    for name,expected in json.loads(cp.read_text()).items():assert sha(Path(name))==expected


def main():
    ap=argparse.ArgumentParser(description=__doc__);ap.add_argument('--source',required=True);ap.add_argument('--output',type=Path,required=True);a=ap.parse_args()
    assert __debug__ and re.fullmatch('[a-f0-9]{40}',a.source) and os.getuid()!=0
    repo=Path(__file__).resolve().parents[1]
    for name in ['review_refund_upgrade_plan_server.py','refund_upgrade_plan.py','check_refund_upgrade.py']:
        assert subprocess.check_output(['git','-C',str(repo),'show',a.source+':tools/'+name])==Path(__file__).with_name(name).read_bytes()
    out=a.output;assert out.parent==ROOT and re.fullmatch('refund-upgrade-reviewed-[a-z0-9-]+',out.name) and not out.exists()
    verify(PREPARED,PREPARED_CP);verify(TOOLCHAIN,CLI_CP)
    assert '--max-len' in (TOOLCHAIN/'write-buffer-help.txt').read_text()
    prepared=json.loads((PREPARED/'private-prepared.json').read_text());before=json.loads((PREPARED/'command-plan.json').read_text())
    old=before['write_buffer'];authority=old[old.index('--buffer-authority')+1]
    assert Path(old[old.index('--buffer')+1])==PREPARED/'buffer-keypair.json'
    assert Path(old[old.index('--config')+1])==PREPARED/'cli-config.json'
    commands=p.commands(TOOLCHAIN/'bin/solana',PREPARED/'cli-config.json',PREPARED/'target.so',
                        PREPARED/'buffer-keypair.json',authority,prepared['plan']['extension_chunks'])
    assert commands['deploy_verified_buffer']==before['deploy_verified_buffer']
    assert commands['extend_one_at_a_time']==before['extend_one_at_a_time']
    current=commands['write_buffer'];at=current.index('--max-len')
    assert current[:at]+current[at+2:]==old
    os.umask(0o077);out.mkdir(mode=0o700)
    (out/'command-plan.json').write_text(json.dumps(commands,indent=2)+'\n')
    summary=dict(source=a.source,original_prepared_checkpoint=PREPARED_CP,toolchain_checkpoint=CLI_CP,
        same_buffer_identity=True,new_buffer_generated=False,explicit_buffer_bytes=p.c.REFUND_BYTES,
        original_preparation_unchanged=True,network_rpc_requests=0,mainnet_sends=0,
        authority_signer_loaded=False,production_changed=False)
    (out/'summary.json').write_text(json.dumps(summary,indent=2)+'\n')
    verify(PREPARED,PREPARED_CP);verify(TOOLCHAIN,CLI_CP)
    pins={str(file):sha(file) for file in out.iterdir() if file.is_file()}
    (out/'completed-checkpoint.json').write_text(json.dumps(pins,indent=2)+'\n')
    print(json.dumps(dict(summary=summary,checkpoint_sha256=sha(out/'completed-checkpoint.json'))))


if __name__=='__main__':
    try:main()
    except Exception:raise SystemExit('Plan review refused; original buffer/preparation unchanged, no on-chain operations.') from None
