"""Install only pinned Agave CLI/keygen into a new validation directory.

Official GitHub artifact, fixed SHA/length, selective regular-file extraction.
No PATH/config/service change, keys, RPC requests or transactions.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import subprocess
import tarfile
import urllib.request

ROOT=Path('/home/arb/validation')
URL='https://github.com/anza-xyz/agave/releases/download/v3.1.9/solana-release-x86_64-unknown-linux-gnu.tar.bz2'
ARCHIVE_SHA='8cffaecc3a5f3c47c8d8f94ea739c716fbae08683018fc29b65c3045a0e9bdd2'
ARCHIVE_BYTES=214999945


def sha(path):
    h=hashlib.sha256()
    with path.open('rb') as f:
        for part in iter(lambda:f.read(1024*1024),b''):h.update(part)
    return h.hexdigest()


def main():
    ap=argparse.ArgumentParser(description=__doc__);ap.add_argument('--source',required=True);ap.add_argument('--output',type=Path,required=True);a=ap.parse_args()
    assert __debug__ and re.fullmatch('[a-f0-9]{40}',a.source)
    assert platform.system()=='Linux' and platform.machine()=='x86_64' and os.getuid()!=0
    out=a.output;assert out.parent==ROOT and re.fullmatch('agave-cli-3-1-9-[a-z0-9-]+',out.name) and not out.exists()
    repo=Path(__file__).resolve().parents[1]
    assert subprocess.check_output(['git','-C',str(repo),'show',a.source+':tools/'+Path(__file__).name])==Path(__file__).read_bytes()
    os.umask(0o077);out.mkdir(mode=0o700);archive=out/'release.tar.bz2'
    req=urllib.request.Request(URL,headers={'User-Agent':'arb-refund-cli-qualification'})
    with urllib.request.urlopen(req,timeout=30) as response,archive.open('xb') as file:
        count=0
        while part:=response.read(1024*1024):
            count+=len(part);assert count<=ARCHIVE_BYTES;file.write(part)
    assert count==ARCHIVE_BYTES and sha(archive)==ARCHIVE_SHA
    (out/'bin').mkdir(mode=0o700)
    with tarfile.open(archive,'r:bz2') as tar:
        for name in ['solana','solana-keygen']:
            member=tar.getmember('solana-release/bin/'+name)
            assert member.isfile() and 0<member.size<64*1024*1024
            source=tar.extractfile(member);assert source is not None
            dest=out/'bin'/name
            with dest.open('xb') as file:
                while part:=source.read(1024*1024):file.write(part)
            dest.chmod(0o700);assert dest.stat().st_size==member.size
    version=subprocess.check_output([str(out/'bin/solana'),'--version'],text=True).strip()
    assert version.startswith('solana-cli 3.1.9 ')
    capabilities={}
    for subcommand,flags in [('write-buffer',['--buffer','--buffer-authority','--use-rpc','--max-sign-attempts']),
                             ('extend',['<ADDITIONAL_BYTES>']),
                             ('deploy',['--buffer','--no-auto-extend','--upgrade-authority','--use-rpc'])]:
        text=subprocess.check_output([str(out/'bin/solana'),'program',subcommand,'--help'],text=True)
        assert all(flag in text for flag in flags);capabilities[subcommand]=True
        (out/(subcommand+'-help.txt')).write_text(text)
    report=dict(source=a.source,version=version,archive_sha256=ARCHIVE_SHA,archive_bytes=ARCHIVE_BYTES,
        binary_sha256={name:sha(out/'bin'/name) for name in ['solana','solana-keygen']},capabilities=capabilities,
        mainnet_sends=0,network_rpc_requests=0,authority_key_loaded=False,production_changed=False)
    (out/'summary.json').write_text(json.dumps(report,indent=2)+'\n')
    pins={str(f):sha(f) for f in out.rglob('*') if f.is_file()}
    (out/'completed-checkpoint.json').write_text(json.dumps(pins,indent=2)+'\n')
    print(json.dumps(dict(summary=report,checkpoint_sha256=sha(out/'completed-checkpoint.json'))))


if __name__=='__main__':
    try:main()
    except Exception:raise SystemExit('CLI qualification refused; no on-chain operation was performed.') from None
