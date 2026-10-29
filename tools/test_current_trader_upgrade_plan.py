import base64
import copy
import hashlib
import unittest
from unittest.mock import patch
import current_trader_upgrade_plan as p


def b58(text):
    n = 0
    for c in text:
        n = n * 58 + p.layout.ALPHABET.index(c)
    return n.to_bytes(32, 'big')


def account(owner, data=b'', lamports=0, executable=False):
    return dict(owner=owner, data=[base64.b64encode(data).decode(), 'base64'], lamports=lamports, executable=executable)


class CurrentPlan(unittest.TestCase):
    def setUp(self):
        self.old, self.new = b'old', b'\x7fELFnew'
        self.header = (3).to_bytes(4, 'little') + (10).to_bytes(8, 'little') + b'\1' + b58(p.AUTHORITY)
        self.snapshot = dict(context=dict(slot=20), value=[
            account(p.layout.LOADER, (2).to_bytes(4, 'little') + b58(p.PROGRAMDATA), executable=True),
            account(p.layout.LOADER, self.header + self.old, 100),
            account(p.layout.SYSTEM, lamports=1000), account(p.layout.SYSTEM, lamports=2000)])
        for name, value in [('OLD_SHA', hashlib.sha256(self.old).hexdigest()), ('OLD_BYTES', len(self.old)),
                            ('TARGET_SHA', hashlib.sha256(self.new).hexdigest()), ('TARGET_BYTES', len(self.new))]:
            mock = patch.object(p, name, value); mock.start(); self.addCleanup(mock.stop)

    def plan(self, s=None, buffer=None, **kw):
        return p.plan(s or self.snapshot, self.new, buffer, 200, 150, **kw)

    def test_funding_includes_buffer_extension_and_authorized_fee_cap(self):
        r = self.plan(); self.assertEqual(r['required_authority_lamports'], 200 + 50 + p.FEE_CAP)
        self.assertEqual(r['next_action'], 'await_funding')

    def test_funded_state_writes_same_buffer(self):
        self.snapshot['value'][2]['lamports'] = 20_000_000
        self.assertEqual(self.plan()['next_action'], 'write_same_buffer')

    def test_partial_and_complete_buffer_do_not_require_second_rent(self):
        self.snapshot['value'][2]['lamports'] = 20_000_000
        for raw, action in [(bytes(len(self.new)), 'write_same_buffer'), (self.new, 'extend_one_chunk')]:
            b = account(p.layout.LOADER, (1).to_bytes(4, 'little') + b'\1' + b58(p.AUTHORITY) + raw, 200)
            r = self.plan(buffer=b); self.assertEqual(r['next_action'], action)
            self.assertEqual(r['required_authority_lamports'], 50 + p.FEE_CAP)

    def test_zero_tail_extension_resumes_without_reallocation(self):
        self.snapshot['value'][2]['lamports'] = 20_000_000
        self.snapshot['value'][1] = account(p.layout.LOADER, self.header + self.old + bytes(len(self.new)-len(self.old)), 150)
        b = account(p.layout.LOADER, (1).to_bytes(4, 'little') + b'\1' + b58(p.AUTHORITY) + self.new, 200)
        r = self.plan(buffer=b); self.assertEqual(r['extension_chunks'], [])
        self.assertEqual(r['next_action'], 'deploy_verified_buffer')

    def test_deployed_target_never_asks_to_deploy_again(self):
        self.snapshot['value'][1] = account(p.layout.LOADER, self.header + self.new, 150)
        self.assertEqual(self.plan()['next_action'], 'verified_complete')

    def test_wrong_target_authority_prefix_and_nonzero_tail_refuse(self):
        for change in range(4):
            s = copy.deepcopy(self.snapshot)
            if change == 0: s['value'][1] = account(p.layout.LOADER, self.header + b'bad', 100)
            elif change == 1: s['value'][1] = account(p.layout.LOADER, self.header + self.old + b'\1', 100)
            elif change == 2: s['value'][0]['owner'] = p.layout.SYSTEM
            else: s['value'][1] = account(p.layout.LOADER, self.header[:13] + bytes(32) + self.old, 100)
            with self.assertRaises(ValueError): self.plan(s)
        with self.assertRaises(ValueError): p.plan(self.snapshot, b'wrong', None, 200, 150)

    def test_foreign_buffer_bytes_and_underfunding_refuse(self):
        for raw, rent in [(b'x' * len(self.new), 200), (self.new, 199)]:
            b = account(p.layout.LOADER, (1).to_bytes(4, 'little') + b'\1' + b58(p.AUTHORITY) + raw, rent)
            with self.assertRaises(ValueError): self.plan(buffer=b)

    def test_fee_spend_is_never_reset_or_over_budget(self):
        r = self.plan(fee_spent=123); self.assertEqual(r['fee_remaining_lamports'], p.FEE_CAP-123)
        for fee in [-1, p.FEE_CAP, p.FEE_CAP+1, True]:
            with self.assertRaises(ValueError): self.plan(fee_spent=fee)

    def test_command_shape_preserves_authority_bounded_fees_and_preflight(self):
        r = p.commands('/cli', '/private-config', '/current-elf', '/existing-buffer', '/authority', [10240, 10240])
        for command in [r['write_buffer'], r['deploy_verified_buffer']]:
            self.assertEqual(command[command.index('--with-compute-unit-price')+1], '0')
            self.assertEqual(command[command.index('--max-sign-attempts')+1], '1')
            self.assertIn('--use-rpc', command); self.assertIn('--use-quic', command)
            self.assertNotIn('--skip-feature-verify', command); self.assertNotIn('--skip-preflight', command)
        self.assertIn('--no-auto-extend', r['deploy_verified_buffer'])
        self.assertEqual(r['extend_one_at_a_time'][0][r['extend_one_at_a_time'][0].index('--keypair')+1], '/authority')
        self.assertNotIn('--final', r['deploy_verified_buffer'])

    def test_last_shortfall_rounds_up_to_mainnet_minimum_without_changing_elf(self):
        with patch.object(p,'OLD_BYTES',899944),patch.object(p,'TARGET_BYTES',960008):
            self.assertEqual(p.maximum_capacity(),961384)
            self.assertEqual(p.extension_chunks(899944),[10240]*6)
            self.assertEqual(p.extension_chunks(951144),[10240])
            self.assertEqual(p.extension_chunks(960008),[])
            self.assertEqual(p.extension_chunks(961384),[])
            with self.assertRaises(ValueError):p.extension_chunks(961385)
        for chunks in [[8864],[0],[True],[10240]*7]:
            with self.assertRaises(ValueError):p.commands('/cli','/config','/elf','/buffer','/authority',chunks)

    def test_exact_candidate_with_only_bounded_zero_padding_is_verified_not_redeployed(self):
        capacity=p.maximum_capacity()
        self.snapshot['value'][1]=account(p.layout.LOADER,self.header+self.new+bytes(capacity-len(self.new)),150)
        result=self.plan()
        self.assertEqual(result['next_action'],'verified_complete')
        self.assertEqual(result['allocation_bytes'],capacity)
        self.assertEqual(result['target_zero_padding_bytes'],capacity-len(self.new))
        for image in [self.new+bytes(capacity-len(self.new)-1)+b'\1',self.new+bytes(capacity-len(self.new)+1)]:
            self.snapshot['value'][1]=account(p.layout.LOADER,self.header+image,150)
            with self.assertRaises(ValueError):self.plan()


if __name__ == '__main__': unittest.main()
