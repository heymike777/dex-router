"""Pure single-buffer upgrade planning; no keys, subprocesses, RPC or sends."""
import hashlib
import check_refund_upgrade as c


def chunks(current, target, maximum=10240):
    for value in (current, target, maximum):
        c.require(type(value) is int and 0 <= value < 2**64, 'invalid allocation')
    c.require(0 < maximum <= 10240, 'extension chunk too large')
    remain = max(0,target-current)
    c.require(remain <= c.REFUND_BYTES, 'allocation bound')
    return [maximum]*(remain//maximum)+([remain%maximum] if remain%maximum else [])


def buffer_state(account, elf):
    """Only this authority's exact target buffer, including safe partial writes."""
    c.require(hashlib.sha256(elf).hexdigest()==c.REFUND_SHA,'unqualified target')
    if account is None:
        return dict(state='absent',complete=False,locked_lamports=0,nonzero_bytes=0)
    c.require(account['owner']==c.LOADER and account['executable'] is False,'wrong buffer role')
    raw=c.data(account)
    c.require(len(raw)==37+len(elf) and raw[:4]==(1).to_bytes(4,'little') and raw[4]==1,'wrong buffer layout')
    c.require(c.base58(raw[5:37])==c.AUTHORITY,'wrong buffer authority')
    payload=raw[37:]
    c.require(all(a==0 or a==b for a,b in zip(payload,elf,strict=True)),'foreign buffer bytes')
    amount=account['lamports'];c.require(type(amount) is int and 0<amount<2**64,'invalid buffer funding')
    complete=payload==elf
    return dict(state='complete' if complete else 'partial',complete=complete,
                locked_lamports=amount,nonzero_bytes=sum(n!=0 for n in payload))


def plan(snapshot, elf, buffer, buffer_rent, program_rent, reserve=100_000_000):
    c.require(type(reserve) is int and reserve>=100_000_000,'deployment reserve below policy')
    status=c.assess(snapshot,elf,buffer_rent,program_rent,reserve)
    b=buffer_state(buffer,elf)
    if b['state']!='absent':
        c.require(b['locked_lamports']>=buffer_rent,'buffer below rent minimum')
    extension=chunks(status['deployed_capacity_bytes'],len(elf))
    # Existing buffer rent is already locked; do not demand a second buffer's rent.
    required=0 if status['refund_abi_deployed'] else (
        (buffer_rent if b['state']=='absent' else 0)+status['extension_rent_lamports']+reserve)
    deficit=max(0,required-status['authority_balance_lamports'])
    if status['refund_abi_deployed']:
        action='verify_buffer_closed' if b['state']!='absent' else 'verified_complete'
    elif deficit:
        action='await_funding'
    elif b['state']!='complete':
        action='write_same_buffer'
    elif extension:
        action='extend_one_chunk'
    else:
        action='deploy_verified_buffer'
    return dict(schema_version=1,read_only=True,refund_abi_deployed=status['refund_abi_deployed'],
        deployed_code_sha256=status['deployed_code_sha256'],deployed_capacity_bytes=status['deployed_capacity_bytes'],
        buffer=b,extension_chunks=extension,extension_rent_lamports=status['extension_rent_lamports'],
        buffer_rent_lamports=buffer_rent,fee_reserve_lamports=reserve,required_authority_lamports=required,
        authority_balance_lamports=status['authority_balance_lamports'],funding_deficit_lamports=deficit,
        funding_sufficient=deficit==0,next_action=action,mainnet_sends=0)


def commands(cli, config, artifact, buffer_keypair, authority_keypair, extension_chunks):
    """Explicit reviewed stages, not a script that automatically retries them."""
    common=[str(cli),'--config',str(config),'--commitment','finalized','--output','json']
    write=common+['program','write-buffer',str(artifact),'--buffer',str(buffer_keypair),
        '--buffer-authority',str(authority_keypair),'--fee-payer',str(authority_keypair),
        '--max-len',str(c.REFUND_BYTES),'--use-rpc','--with-compute-unit-price','50000','--max-sign-attempts','3']
    extend=[common+['program','extend',c.PROGRAM,str(n)] for n in extension_chunks]
    # Buffer is already uploaded and its bytes verified. No random buffer or
    # implicit extension, no feature/preflight bypass, no authority finalization.
    deploy=common+['program','deploy','--program-id',c.PROGRAM,'--buffer',str(buffer_keypair),
        '--upgrade-authority',str(authority_keypair),'--fee-payer',str(authority_keypair),
        '--no-auto-extend','--use-rpc','--with-compute-unit-price','50000','--max-sign-attempts','3']
    return dict(write_buffer=write,extend_one_at_a_time=extend,deploy_verified_buffer=deploy,
        prerequisites=['fresh_mainnet_identity_funding_and_signer_check','verify_same_buffer_after_write',
                       'verify_finalized_allocation_after_each_extension',
                       'reconcile_unknown_cli_outcome_before_any_retry',
                       'verify_finalized_elf_authority_and_buffer_closure'])
