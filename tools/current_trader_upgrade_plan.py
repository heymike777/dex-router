"""Pure plan for the human-authorized current-trader cleanup upgrade; no I/O."""
import hashlib
import check_refund_upgrade as layout

PROGRAM = layout.PROGRAM
PROGRAMDATA = layout.PROGRAMDATA
AUTHORITY = layout.AUTHORITY
TRADER = 'MikekzKTjw4ny4pMi6QTpF41DHHHAhNei3YNxPWi9mt'
BUFFER = '3xxmsbhEuoDfi4yNxWNHnuVyREcuBF19zx68HWBjXTQK'
TARGET_SHA = '85d841610a5835232c7af836425656543c2ed40bee7a30cad57326f964c08c6b'
TARGET_BYTES = 960008
OLD_SHA = '3eeb2d67fe1bea200e616cbf7056fad5b5150b7c838ec094337aab07ef91e762'
OLD_BYTES = 899944
FEE_CAP = 10_000_000


def require(ok, message):
    if not ok:
        raise ValueError(message)


def plan(snapshot, elf, buffer, buffer_rent, target_rent, fee_spent=0):
    require(len(elf) == TARGET_BYTES and hashlib.sha256(elf).hexdigest() == TARGET_SHA, 'wrong cleanup target')
    require(type(fee_spent) is int and 0 <= fee_spent <= FEE_CAP, 'fee budget exhausted or unaccounted')
    require(all(type(v) is int and v > 0 for v in [buffer_rent, target_rent]), 'invalid rent')
    program, pd, authority, trader = snapshot['value']
    require(program['owner'] == pd['owner'] == layout.LOADER and program['executable'] and not pd['executable'], 'loader role')
    header, data = layout.data(program), layout.data(pd)
    require(len(header) == 36 and header[:4] == (2).to_bytes(4, 'little') and layout.base58(header[4:]) == PROGRAMDATA, 'ProgramData link')
    require(len(data) >= 45 and data[:4] == (3).to_bytes(4, 'little') and data[12] == 1
            and layout.base58(data[13:45]) == AUTHORITY, 'upgrade authority')
    require(int.from_bytes(data[4:12], 'little') <= snapshot['context']['slot'], 'future deployment')
    for wallet in [authority, trader]:
        require(wallet is not None and wallet['owner'] == layout.SYSTEM and not wallet['executable'] and layout.data(wallet) == b'', 'wallet role')
    image = data[45:]
    deployed = image == elf
    require(deployed or fee_spent < FEE_CAP, 'fee budget exhausted before completion')
    old = (OLD_BYTES <= len(image) <= TARGET_BYTES and hashlib.sha256(image[:OLD_BYTES]).hexdigest() == OLD_SHA and not any(image[OLD_BYTES:]))
    require(deployed or old, 'unknown code or nonzero extension tail')
    state, complete, locked = 'absent', False, 0
    if buffer is not None:
        require(buffer['owner'] == layout.LOADER and not buffer['executable'], 'buffer role')
        raw = layout.data(buffer)
        require(len(raw) == 37 + len(elf) and raw[:4] == (1).to_bytes(4, 'little') and raw[4] == 1
                and layout.base58(raw[5:37]) == AUTHORITY, 'buffer layout/authority')
        require(all(a == 0 or a == b for a, b in zip(raw[37:], elf, strict=True)), 'foreign buffer bytes')
        require(buffer['lamports'] >= buffer_rent, 'buffer below rent')
        complete = raw[37:] == elf
        state, locked = ('complete' if complete else 'partial'), buffer['lamports']
    extra = max(0, len(elf) - len(image))
    chunks = [10240] * (extra // 10240) + ([extra % 10240] if extra % 10240 else [])
    retained = max(0, target_rent - pd['lamports']) if extra else 0
    remaining = FEE_CAP - fee_spent
    required = 0 if deployed else (buffer_rent if buffer is None else 0) + retained + remaining
    deficit = max(0, required - authority['lamports'])
    action = ('verify_buffer_closed' if buffer is not None else 'verified_complete') if deployed else (
        'await_funding' if deficit else 'write_same_buffer' if not complete else
        'extend_one_chunk' if chunks else 'deploy_verified_buffer')
    return dict(schema_version=1, read_only=True, target_sha256=TARGET_SHA, target_bytes=TARGET_BYTES,
                buffer_identity=BUFFER, buffer_state=state, buffer_locked_lamports=locked,
                deployed=deployed, allocation_bytes=len(image), extension_chunks=chunks,
                extension_rent_lamports=retained, temporary_buffer_rent_lamports=buffer_rent,
                fee_cap_lamports=FEE_CAP, fee_spent_lamports=fee_spent, fee_remaining_lamports=remaining,
                required_authority_lamports=required, authority_balance_lamports=authority['lamports'],
                funding_deficit_lamports=deficit, next_action=action, mainnet_sends=0)


def commands(cli, config, artifact, buffer_keypair, authority_keypair, chunks):
    require(all(type(n) is int and 0 < n <= 10240 for n in chunks) and sum(chunks) <= TARGET_BYTES - OLD_BYTES, 'extension bound')
    # Force QUIC implementation, where --use-rpc is honored; no TPU is created.
    common = [str(cli), '--config', str(config), '--keypair', str(authority_keypair),
              '--commitment', 'finalized', '--output', 'json', '--use-quic']
    bounded = ['--use-rpc', '--with-compute-unit-price', '0', '--max-sign-attempts', '1']
    write = common + ['program', 'write-buffer', str(artifact), '--buffer', str(buffer_keypair),
                      '--buffer-authority', str(authority_keypair), '--fee-payer', str(authority_keypair),
                      '--max-len', str(TARGET_BYTES)] + bounded
    extend = [common + ['program', 'extend', PROGRAM, str(n)] for n in chunks]
    deploy = common + ['program', 'deploy', '--program-id', PROGRAM, '--buffer', str(buffer_keypair),
                       '--upgrade-authority', str(authority_keypair), '--fee-payer', str(authority_keypair),
                       '--no-auto-extend'] + bounded
    return dict(write_buffer=write, extend_one_at_a_time=extend, deploy_verified_buffer=deploy,
                priority_fee_micro_lamports=0, signing_passes_per_write_message=1,
                fee_cap_lamports=FEE_CAP, automatic_retries_after_unknown=False,
                cli_does_not_enforce_total_fee_cap=True,
                prerequisites=['human_upgrade_only_grant', 'funding_condition_satisfied',
                               'fresh_mainnet_roles_code_buffer_rent_fee_and_signer_identity',
                               'reserve_remaining_fee_cap_before_each_stage',
                               'preserve_actual_total_fee_spend_across_all_resumes',
                               'verify_same_buffer_bytes_authority_at_finalized',
                               'verify_allocation_after_each_single_extension',
                               'reconcile_unknown_outcome_before_any_resume',
                               'verify_finalized_candidate_authority_and_buffer_closure'])
