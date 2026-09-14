"""Explicit prepare/deploy for the authorized existing mainnet router.
Provider credentials stay in memory; public reports never contain RPC URLs/keys.
Uses the existing SSH configuration to obtain the provider URL, not the signer.
"""
import argparse, base64, hashlib, json, pathlib, subprocess, urllib.request
from solders.pubkey import Pubkey
from solders.keypair import Keypair

PROGRAM = '89P1rihVbww57idhSLHUhxXNkhzYcSghYrbjRoUuivPo'
DATA = '4qDn42Nyprcpeb5XPFRntQPBieZFdL3jEfMzQAFnzKZp'
AUTHORITY = 'mikeSK7hxrzS1U7xyZ5D411AESvhh8zzEdDyhHL2AYg'
GENESIS = '5eykt4UsFv8P8NJdTREpY1vzqKqZKvdpKuc147dw2N9d'
p = argparse.ArgumentParser()
p.add_argument('mode', choices=['prepare', 'deploy', 'verify'])
p.add_argument('--artifact', required=True)
p.add_argument('--authority', required=True)
p.add_argument('--out', required=True)
a = p.parse_args()
out = pathlib.Path(a.out)
if a.mode == 'prepare': out.mkdir(mode=0o700, parents=True, exist_ok=False)
assert out.is_dir()
url = subprocess.check_output(['ssh', '-i', str(pathlib.Path.home()/'.ssh/id_ed25519'), '-o', 'BatchMode=yes',
    'arb@84.32.176.175', "python3 -c \"import json;print(json.load(open('/home/arb/validation/rust-guarded-20260913-fresh-live/execution.json'))['rpc_url'])\""], text=True).strip()
def rpc(method, params):
    req = urllib.request.Request(url, data=json.dumps({'jsonrpc':'2.0','id':1,'method':method,'params':params}).encode(), headers={'Content-Type':'application/json'})
    result = json.load(urllib.request.urlopen(req, timeout=30))
    if 'error' in result: raise RuntimeError('RPC request rejected: '+method)
    return result['result']
def account(key):
    return rpc('getAccountInfo',[key,{'encoding':'base64','commitment':'finalized'}])['value']
def save(name, value): (out/name).write_text(json.dumps(value, indent=2)+'\n')
def command(name, args):
    with (out/(name+'.log')).open('w') as f:
        proc = subprocess.Popen(args, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
        for line in proc.stdout: f.write(line.replace(url,'<RPC>')); f.flush()
        code = proc.wait()
    if code: raise RuntimeError(name+' failed; inspect its redacted log; do not start a new buffer')
assert rpc('getGenesisHash',[]) == GENESIS
assert subprocess.check_output(['solana-keygen','pubkey',a.authority],text=True).strip() == AUTHORITY
program = account(PROGRAM); data = account(DATA)
assert program['owner'] == data['owner'] == 'BPFLoaderUpgradeab1e11111111111111111111111'
program_bytes = base64.b64decode(program['data'][0]); raw = base64.b64decode(data['data'][0])
assert program_bytes[:4] == (2).to_bytes(4,'little') and str(Pubkey.from_bytes(program_bytes[4:36])) == DATA
assert raw[:4] == (3).to_bytes(4,'little') and raw[12] == 1 and str(Pubkey.from_bytes(raw[13:45])) == AUTHORITY
elf = pathlib.Path(a.artifact).read_bytes(); sha = hashlib.sha256(elf).hexdigest()
assert elf[:4] == b'\x7fELF'
balance = rpc('getBalance',[AUTHORITY,{'commitment':'finalized'}])['value']
if a.mode == 'prepare':
    (out/'program-before.so').write_bytes(raw[45:])
    (out/'program-after-build.so').write_bytes(elf)
    buffer = Keypair(); (out/'buffer-keypair.json').write_text(json.dumps(list(bytes(buffer))))
    (out/'buffer-keypair.json').chmod(0o600)
    buffer_rent = rpc('getMinimumBalanceForRentExemption',[37+len(elf)])
    program_rent = rpc('getMinimumBalanceForRentExemption',[45+len(elf)])
    extension_rent = max(0,program_rent-data['lamports'])
    report = {'program':PROGRAM,'programdata':DATA,'authority':AUTHORITY,'genesis':GENESIS,
        'build_sha256':sha,'build_bytes':len(elf),'previous_bytes':len(raw)-45,
        'previous_sha256':hashlib.sha256(raw[45:]).hexdigest(),'previous_slot':int.from_bytes(raw[4:12],'little'),
        'extension_bytes':max(0,len(elf)-(len(raw)-45)), 'buffer':str(buffer.pubkey()),
        'buffer_rent_lamports':buffer_rent,'extension_rent_lamports':extension_rent,
        'authority_before_lamports':balance,'fee_reserve_lamports':100_000_000}
    save('prepared.json',report); print(json.dumps(report))
    assert balance >= buffer_rent+extension_rent+100_000_000, 'insufficient deployment funding'
elif a.mode == 'deploy':
    prepared = json.loads((out/'prepared.json').read_text()); assert prepared['build_sha256'] == sha
    assert hashlib.sha256(raw[45:]).hexdigest() == prepared['previous_sha256'], 'program changed since preparation'
    assert balance >= prepared['buffer_rent_lamports']+prepared['extension_rent_lamports']+100_000_000
    if prepared['extension_bytes']:
        extension = max(10_240, prepared['extension_bytes'])
        required_rent = rpc('getMinimumBalanceForRentExemption',[len(raw)+extension])
        assert balance >= prepared['buffer_rent_lamports']+max(0, required_rent-data['lamports'])+100_000_000
        prepared['allocated_extension_bytes'] = extension
        save('prepared.json', prepared)
        command('extend',['solana','program','extend',PROGRAM,str(extension), '--keypair',a.authority,'--url',url,'--commitment','finalized','--output','json'])
    command('deploy',['solana','program','deploy',a.artifact,'--program-id',PROGRAM,
        '--upgrade-authority',a.authority,'--keypair',a.authority,'--buffer',str(out/'buffer-keypair.json'),
        '--url',url,'--with-compute-unit-price','50000','--max-sign-attempts','10','--output','json'])
    print('Deployment command completed; run verify at finalized commitment.')
else:
    prepared=json.loads((out/'prepared.json').read_text()); assert sha == prepared['build_sha256']
    assert raw[45:45+len(elf)] == elf and not any(raw[45+len(elf):]), 'on-chain ELF mismatch'
    slot=int.from_bytes(raw[4:12],'little'); assert slot > prepared['previous_slot']
    buffer=account(prepared['buffer'])
    assert buffer is None or buffer['lamports'] == 0, 'deployment buffer remains funded'
    report={'program':PROGRAM,'programdata':DATA,'authority':AUTHORITY,'deployment_slot':slot,
        'elf_sha256':sha,'elf_bytes':len(elf),'programdata_bytes':len(raw),'elf_matches':True,
        'buffer_closed':True,'authority_after_lamports':balance,
        'authority_delta_lamports':balance-prepared['authority_before_lamports']}
    save('verified.json',report); print(json.dumps(report))
