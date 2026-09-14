"""Execute the actual router ELF in LiteSVM; no RPC or real wallet access.

python -m venv /tmp/arb-size-svm && /tmp/arb-size-svm/bin/pip install solders==0.29.0
cargo build-sbf --manifest-path ../dex-router/Cargo.toml --sbf-out-dir /tmp/arb-size-build -- --locked --offline
/tmp/arb-size-svm/bin/python tools/test_compact_arbitrage_svm.py /tmp/arb-size-build/dex_solana.so
"""
import hashlib
import struct
import sys
from solders.account import Account
from solders.instruction import AccountMeta, Instruction
from solders.keypair import Keypair
from solders.litesvm import LiteSVM
from solders.pubkey import Pubkey
from solders.transaction import Transaction
from solders.transaction_metadata import FailedTransactionMetadata
from solders.system_program import transfer, TransferParams

ROUTER = Pubkey.from_string('89P1rihVbww57idhSLHUhxXNkhzYcSghYrbjRoUuivPo')
TOKEN = Pubkey.from_string('TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA')
WSOL = Pubkey.from_string('So11111111111111111111111111111111111111112')
SYSTEM = Pubkey.default()

def discriminator(name):
    return hashlib.sha256(('global:' + name).encode()).digest()[:8]

def meta(key, signer=False, writable=False):
    return AccountMeta(key, signer, writable)

def setup():
    svm = LiteSVM()
    svm.add_program_from_file(ROUTER, sys.argv[1])
    mint_data = bytearray(82)
    mint_data[44:46] = bytes([9, 1])  # decimals, initialized
    svm.set_account(WSOL, Account(svm.minimum_balance_for_rent_exemption(82), bytes(mint_data), TOKEN))
    payer = Keypair()
    svm.airdrop(payer.pubkey(), 1_000_000_000)
    output = Pubkey.find_program_address([b'arb_output', bytes(payer.pubkey())], ROUTER)[0]
    return svm, payer, output

def prepare(payer, output, mint=WSOL):
    return Instruction(ROUTER, discriminator('prepare_arbitrage_output'),
                       [meta(payer.pubkey(), True, True), meta(output, writable=True),
                        meta(mint), meta(TOKEN), meta(SYSTEM)])

def send(svm, payer, instructions, success=True):
    from solders.address_lookup_table_account import AddressLookupTableAccount
    from solders.message import MessageV0
    from solders.transaction import VersionedTransaction
    lookup = Pubkey.new_unique()
    addresses = list(dict.fromkeys(m.pubkey for ix in instructions for m in ix.accounts if m.pubkey != payer.pubkey()))
    header = bytearray(56)
    struct.pack_into('<IQ', header, 0, 1, (1 << 64) - 1)
    header[20] = len(addresses)
    data = bytes(header) + b''.join(bytes(a) for a in addresses)
    svm.set_account(lookup, Account(svm.minimum_balance_for_rent_exemption(len(data)), data, Pubkey.from_string('AddressLookupTab1e1111111111111111111111111')))
    message = MessageV0.try_compile(payer.pubkey(), instructions, [AddressLookupTableAccount(lookup, addresses)], svm.latest_blockhash())
    tx = VersionedTransaction(message, [payer])
    assert len(bytes(tx)) <= 1232
    result = svm.send_transaction(tx)
    failed = isinstance(result, FailedTransactionMetadata)
    assert failed != success, result
    svm.expire_blockhash()
    return result

def native_account(svm, owner, amount=0):
    rent = svm.minimum_balance_for_rent_exemption(165)
    data = bytearray(165)
    data[:32] = bytes(WSOL)
    data[32:64] = bytes(owner)
    struct.pack_into('<Q', data, 64, amount)
    data[108] = 1
    struct.pack_into('<IQ', data, 109, 1, rent)
    return Account(rent + amount, bytes(data), TOKEN)

svm, payer, output = setup()
before = svm.get_balance(payer.pubkey())
send(svm, payer, [prepare(payer, output)])
account = svm.get_account(output)
assert account.owner == TOKEN and account.data[32:64] == bytes(payer.pubkey())
assert account.data[:32] == bytes(WSOL) and account.data[108] == 1
rent = account.lamports
assert before - svm.get_balance(payer.pubkey()) == rent + 5000
send(svm, payer, [prepare(payer, output)])  # idempotent while empty

# Funding, sync and close execute with the primary wallet as sole signer.
# This models WSOL recovery, not a profitable DEX cycle.
send(svm, payer, [
    transfer(TransferParams(from_pubkey=payer.pubkey(), to_pubkey=output, lamports=123456)),
    Instruction(TOKEN, bytes([17]), [meta(output, writable=True)]),
    Instruction(TOKEN, bytes([9]), [meta(output, writable=True), meta(payer.pubkey(), writable=True), meta(payer.pubkey(), True)]),
])
assert svm.get_account(output) is None or svm.get_account(output).lamports == 0
assert before - svm.get_balance(payer.pubkey()) == 15000, 'rent and native value must return; only 3 signature fees remain'
send(svm, payer, [prepare(payer, output)])  # recreate after close

for mutation in ['owner', 'mint', 'funded', 'delegate', 'close_authority']:
    svm, payer, output = setup()
    account = native_account(svm, payer.pubkey(), 1 if mutation == 'funded' else 0)
    data = bytearray(account.data)
    if mutation == 'owner': data[32:64] = bytes(Pubkey.new_unique())
    if mutation == 'mint': data[:32] = bytes(Pubkey.new_unique())
    if mutation == 'delegate': struct.pack_into('<I', data, 72, 1)
    if mutation == 'close_authority': struct.pack_into('<I', data, 129, 1)
    svm.set_account(output, Account(account.lamports, bytes(data), TOKEN))
    send(svm, payer, [prepare(payer, output)], success=False)

svm, payer, output = setup()
send(svm, payer, [prepare(payer, Pubkey.new_unique())], success=False)
send(svm, payer, [prepare(payer, output)])
source = Pubkey.new_unique()
svm.set_account(source, native_account(svm, payer.pubkey(), 1000))
keys = [meta(payer.pubkey(), True), meta(source, writable=True), meta(output, writable=True), meta(WSOL), meta(WSOL)]
for amount, minimum, hops in [(0, 0, [46, 46]), (1000, 999, [46, 46]), (1000, 1000, [46]), (1000, 1000, [46] * 5)]:
    payload = discriminator('arbitrage_compact') + struct.pack('<QQI', amount, minimum, len(hops)) + bytes(hops)
    result = send(svm, payer, [Instruction(ROUTER, payload, keys)], success=False)
    assert any('AnchorError' in line for line in result.meta().logs()), result
    assert struct.unpack_from('<Q', svm.get_account(source).data, 64)[0] == 1000

print('PASS LiteSVM: actual router ELF, native output init/reuse/close/recreate, one signature fee, rent recovery, wrong PDA/owner/mint/funded/delegated/close-authority rejection, compact loss/hop validation')

# Optional full-cycle fixture: actual router + SPL Token, deterministic mock DEX.
# This verifies transaction semantics; it is not mainnet profitability evidence.
if len(sys.argv) > 2:
    DEX = Pubkey.from_string('675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8')
    SA = Pubkey.from_string('3Km2q4WgGzbbdCm97YSvysaggFxpxLMZMM7AKNSy4bnn')
    USDC = Pubkey.from_string('EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v')

    def token_account(svm, owner, mint, amount):
        if mint == WSOL: return native_account(svm, owner, amount)
        data = bytearray(native_account(svm, owner, amount).data)
        data[:32] = bytes(mint)
        data[109:121] = bytes(12)
        return Account(svm.minimum_balance_for_rent_exemption(165), bytes(data), TOKEN)

    def cycle(compact, ratio, tip, expected_success, hops=2):
        svm, payer, output = setup()
        svm.add_program_from_file(DEX, sys.argv[2])
        pool_authority = Pubkey.find_program_address([b'fixture'], DEX)[0]
        mint = Pubkey.new_unique()
        source, middle, usdc_account, tip_recipient = [Pubkey.new_unique() for _ in range(4)]
        svm.set_account(source, native_account(svm, payer.pubkey(), 0))
        svm.set_account(middle, token_account(svm, SA, mint, 0))
        svm.set_account(usdc_account, token_account(svm, payer.pubkey(), USDC, 0))
        svm.airdrop(tip_recipient, 1_000_000)
        snapshot = Pubkey.find_program_address([b'profit_snapshot', bytes(payer.pubkey())], ROUTER)[0]
        snapshot_data = hashlib.sha256(b'account:ProfitSnapshot').digest()[:8] + bytes(payer.pubkey()) + bytes(16)
        svm.set_account(snapshot, Account(svm.minimum_balance_for_rent_exemption(len(snapshot_data)), snapshot_data, ROUTER))
        middles = [middle] + [Pubkey.new_unique() for _ in range(hops - 2)]
        mints = [WSOL, mint] + [Pubkey.new_unique() for _ in range(hops - 2)] + [WSOL]
        accounts = [source] + middles + [output]
        for i, account in enumerate(middles):
            svm.set_account(account, token_account(svm, SA, mints[i+1], 0))
        remaining = []
        legs = [(payer.pubkey() if i == 0 else SA, accounts[i], accounts[i+1], mints[i], mints[i+1], ratio if i == hops-1 else 1000) for i in range(hops)]
        for authority, src, dst, src_mint, dst_mint, rate in legs:
            pool, reserve_in, reserve_out = [Pubkey.new_unique() for _ in range(3)]
            svm.set_account(pool, Account(1_000_000, struct.pack('<Q', rate), DEX))
            svm.set_account(reserve_in, token_account(svm, pool_authority, src_mint, 1_000_000_000))
            svm.set_account(reserve_out, token_account(svm, pool_authority, dst_mint, 1_000_000_000))
            remaining += [meta(DEX), meta(authority, authority == payer.pubkey()), meta(src, writable=True),
                          meta(dst, writable=True), meta(TOKEN), meta(pool, writable=True), meta(pool_authority),
                          meta(reserve_in, writable=True), meta(reserve_out, writable=True)]
        amount = 100_000_000
        # Resolve the enum from generated IDL rather than guessing its numeric value.
        import json
        from pathlib import Path
        idl = json.loads(Path(sys.argv[1]).with_suffix('.json').read_text())
        variants = next(t for t in idl['types'] if t['name'] == 'Dex')['type']['variants']
        dex = next(i for i, variant in enumerate(variants) if variant['name'] == 'RaydiumSwapV2')
        if compact:
            swap_data = discriminator('arbitrage_compact') + struct.pack('<QQI', amount, amount, hops) + bytes([dex] * hops)
        else:
            swap_data = discriminator('swap') + struct.pack('<QQQIQII', amount, amount, amount, 1, amount, 1, hops)
            swap_data += (struct.pack('<IBIB', 1, dex, 1, 100) * hops) + struct.pack('<Q', 0)
        snapshot_ix = Instruction(ROUTER, discriminator('create_profit_snapshot') + struct.pack('<Q', 5000),
                                  [meta(payer.pubkey(), True, True), meta(usdc_account), meta(snapshot, writable=True), meta(SYSTEM)])
        swap_ix = Instruction(ROUTER, swap_data, [meta(payer.pubkey(), True), meta(source, writable=True),
                                                meta(output, writable=True), meta(WSOL), meta(WSOL)] + remaining)
        check_ix = Instruction(ROUTER, discriminator('profit_check'),
                               [meta(payer.pubkey(), True, True), meta(usdc_account), meta(snapshot, writable=True)])
        # Match the builder's explicit compute limit; default per-ix budgets would
        # change both scheduling and fee accounting in production.
        from solders.compute_budget import set_compute_unit_limit
        instructions = [set_compute_unit_limit(400_000), snapshot_ix, prepare(payer, output),
                        transfer(TransferParams(from_pubkey=payer.pubkey(), to_pubkey=source, lamports=amount)),
                        Instruction(TOKEN, bytes([17]), [meta(source, writable=True)]), swap_ix,
                        Instruction(TOKEN, bytes([9]), [meta(output, writable=True), meta(payer.pubkey(), writable=True), meta(payer.pubkey(), True)]),
                        transfer(TransferParams(from_pubkey=payer.pubkey(), to_pubkey=tip_recipient, lamports=tip)), check_ix]
        before = svm.get_balance(payer.pubkey())
        tip_before = svm.get_balance(tip_recipient)
        result = send(svm, payer, instructions, expected_success)
        if expected_success:
            assert svm.get_balance(payer.pubkey()) - before == amount * ratio // 1000 - amount - tip - 5000
            assert svm.get_balance(tip_recipient) - tip_before == tip
            assert struct.unpack_from('<Q', svm.get_account(source).data, 64)[0] == 0
            assert all(struct.unpack_from('<Q', svm.get_account(a).data, 64)[0] == 0 for a in middles)
            assert svm.get_account(output) is None or svm.get_account(output).lamports == 0
        else:
            assert before - svm.get_balance(payer.pubkey()) == 5000
            assert svm.get_balance(tip_recipient) == tip_before
            assert struct.unpack_from('<Q', svm.get_account(source).data, 64)[0] == 0
            assert all(struct.unpack_from('<Q', svm.get_account(a).data, 64)[0] == 0 for a in middles)
            assert svm.get_account(output) is None or svm.get_account(output).lamports == 0
        return result

    cycle(False, 1010, 200_000, True)
    cycle(True, 1010, 200_000, True)
    loss = cycle(True, 990, 200_000, False)
    assert any('MinReturnNotReached' in line for line in loss.meta().logs())
    net_loss = cycle(True, 1010, 950_000, False)
    assert any('ProfitGuardNoProfit' in line for line in net_loss.meta().logs())
    print('PASS LiteSVM full-cycle mock DEX: legacy/compact output parity, PDA-signed second hop, profit after tip/fee/rent, min-return loss rollback, net-profit guard rollback; failures cost only signature fee')

    cycle(True, 1010, 200_000, True, hops=4)
    cycle(False, 1010, 200_000, True, hops=4)
    loss = cycle(True, 990, 200_000, False, hops=4)
    assert any('MinReturnNotReached' in line for line in loss.meta().logs())
    loss = cycle(True, 1010, 950_000, False, hops=4)
    assert any('ProfitGuardNoProfit' in line for line in loss.meta().logs())
    print('PASS four-hop actual router ELF: compact/legacy, PDA hops, min-return and net-profit rollback')
