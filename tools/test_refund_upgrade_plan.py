import base64
import copy
import hashlib
import unittest
from unittest.mock import patch
import check_refund_upgrade as c
import refund_upgrade_plan as p


def raw58(text):
    n=0
    for s in text:n=n*58+c.ALPHABET.index(s)
    return n.to_bytes(32,'big')


def account(owner,data=b'',amount=0,executable=False):
    return dict(owner=owner,data=[base64.b64encode(data).decode(),'base64'],lamports=amount,executable=executable)


class Controls(unittest.TestCase):
    def setUp(self):
        self.old=b'old';self.elf=b'\x7fELFknown';self.header=(3).to_bytes(4,'little')+bytes(8)+b'\1'+raw58(c.AUTHORITY)
        self.snapshot=dict(context=dict(slot=100),value=[account(c.LOADER,(2).to_bytes(4,'little')+raw58(c.PROGRAMDATA),executable=True),
            account(c.LOADER,self.header+self.old,100),account(c.SYSTEM,amount=3000),account(c.SYSTEM,amount=9999)])
        for key,value in [('OLD_SHA',hashlib.sha256(self.old).hexdigest()),('OLD_BYTES',len(self.old)),('REFUND_SHA',hashlib.sha256(self.elf).hexdigest())]:
            item=patch.object(c,key,value);item.start();self.addCleanup(item.stop)
        self.buffer_header=(1).to_bytes(4,'little')+b'\1'+raw58(c.AUTHORITY)

    def plan(self,buffer=None):
        with patch.object(c,'assess',wraps=c.assess):
            return p.plan(self.snapshot,self.elf,buffer,2000,500,100_000_000)

    def test_actual_extension_size_is_split_without_extra_allocation(self):
        out=p.chunks(899944,960008)
        self.assertEqual(out,[10240]*5+[8864]);self.assertEqual(sum(out),60064)
        for args in [(True,10),(0,-1),(0,100,10241)]:
            with self.assertRaises(ValueError):p.chunks(*args)

    def test_absent_partial_and_complete_buffer(self):
        self.assertEqual(p.buffer_state(None,self.elf)['state'],'absent')
        for payload,state in [(bytes(len(self.elf)),'partial'),(self.elf[:4]+bytes(len(self.elf)-4),'partial'),(self.elf,'complete')]:
            b=account(c.LOADER,self.buffer_header+payload,2000)
            self.assertEqual(p.buffer_state(b,self.elf)['state'],state)

    def test_foreign_authority_owner_layout_and_payload_refuse(self):
        good=account(c.LOADER,self.buffer_header+self.elf,2000)
        bad=[account(c.SYSTEM,self.buffer_header+self.elf,2000),
             account(c.LOADER,self.buffer_header[:5]+bytes(32)+self.elf,2000),
             account(c.LOADER,self.buffer_header+self.elf+b'\0',2000),
             account(c.LOADER,self.buffer_header+b'X'+self.elf[1:],2000)]
        self.assertTrue(p.buffer_state(good,self.elf)['complete'])
        for b in bad:
            with self.assertRaises(ValueError):p.buffer_state(b,self.elf)

    def test_funding_holds_and_existing_buffer_is_not_charged_twice(self):
        absent=self.plan();self.assertEqual(absent['next_action'],'await_funding')
        complete=self.plan(account(c.LOADER,self.buffer_header+self.elf,2000))
        self.assertEqual(absent['required_authority_lamports']-complete['required_authority_lamports'],2000)
        self.assertEqual(complete['funding_deficit_lamports'],100_000_400-3000)

    def test_known_extended_original_resumes_upgrade_without_extension(self):
        self.snapshot['value'][1]=account(c.LOADER,self.header+self.old+bytes(20),500)
        self.snapshot['value'][2]['lamports']=200_000_000
        result=self.plan(account(c.LOADER,self.buffer_header+self.elf,2000))
        self.assertEqual(result['extension_chunks'],[])
        self.assertEqual(result['next_action'],'deploy_verified_buffer')

    def test_funded_stages_and_finished_program_require_buffer_verification(self):
        self.snapshot['value'][2]['lamports']=200_000_000
        self.assertEqual(self.plan()['next_action'],'write_same_buffer')
        b=account(c.LOADER,self.buffer_header+self.elf,2000)
        self.assertEqual(self.plan(b)['next_action'],'extend_one_chunk')
        self.snapshot['value'][1]=account(c.LOADER,self.header+self.elf,500)
        self.assertEqual(self.plan(b)['next_action'],'verify_buffer_closed')
        self.assertEqual(self.plan()['next_action'],'verified_complete')

    def test_commands_never_choose_random_buffer_bypass_checks_or_bulk_close(self):
        commands=p.commands('/cli','/private/config','/elf','/private/buffer','/private/authority',[10240,5])
        for cmd in [commands['write_buffer'],commands['deploy_verified_buffer']]:
            self.assertIn('/private/buffer',cmd);self.assertNotIn('--skip-preflight',cmd);self.assertNotIn('--skip-feature-verify',cmd)
            self.assertNotIn('--final',cmd);self.assertNotIn('--url',cmd);self.assertNotIn('--buffers',cmd)
        self.assertIn('--no-auto-extend',commands['deploy_verified_buffer'])
        write=commands['write_buffer']
        self.assertEqual(write[write.index('--max-len')+1],str(c.REFUND_BYTES))
        self.assertNotIn('/elf',commands['deploy_verified_buffer'])
        self.assertEqual(len(commands['extend_one_at_a_time']),2)


if __name__=='__main__':unittest.main()
