"""Offline refusal/funding controls; synthetic bytes are not an ELF execution proof."""
import base64
import copy
import hashlib
import unittest
from unittest.mock import patch

import check_refund_upgrade as check


def decode58(text):
    value = 0
    for char in text:
        value = value * 58 + check.ALPHABET.index(char)
    return value.to_bytes(32, "big")


def account(owner, raw=b"", balance=0, executable=False):
    return {"owner": owner, "data": [base64.b64encode(raw).decode(), "base64"],
            "lamports": balance, "executable": executable}


class UpgradeReadinessTests(unittest.TestCase):
    def setUp(self):
        self.old = b"old image"
        self.new = b"\x7fELFnew qualified image"
        self.header = (3).to_bytes(4, "little") + (123).to_bytes(8, "little") + b"\1" + decode58(check.AUTHORITY)
        self.snapshot = {"context": {"slot": 456}, "value": [
            account(check.LOADER, (2).to_bytes(4, "little") + decode58(check.PROGRAMDATA), executable=True),
            account(check.LOADER, self.header + self.old, 100),
            account(check.SYSTEM, balance=1000), account(check.SYSTEM, balance=3000),
        ]}
        self.patches = [patch.object(check, "OLD_SHA", hashlib.sha256(self.old).hexdigest()),
                        patch.object(check, "REFUND_SHA", hashlib.sha256(self.new).hexdigest())]
        for item in self.patches:
            item.start()
            self.addCleanup(item.stop)

    def assess(self, snapshot=None):
        return check.assess(snapshot or self.snapshot, self.new, 2000, 500, 100)

    def test_buffer_and_extension_need_temporary_capital(self):
        result = self.assess()
        self.assertEqual(result["extension_rent_lamports"], 400)
        self.assertEqual(result["temporary_required_lamports"], 2500)
        self.assertEqual(result["funding_deficit_lamports"], 1500)
        self.assertFalse(result["funding_sufficient"])
        self.assertFalse(result["refund_abi_deployed"])
        self.snapshot["value"][2]["lamports"] = 2500
        self.assertTrue(self.assess()["funding_sufficient"])

    def test_existing_padded_refund_image_needs_no_second_buffer(self):
        self.snapshot["value"][1] = account(check.LOADER, self.header + self.new + bytes(128), 500)
        result = self.assess()
        self.assertTrue(result["refund_abi_deployed"])
        self.assertEqual(result["temporary_required_lamports"], 0)

    def test_unrecognized_upgrade_and_nonzero_tail_refuse(self):
        for raw in (b"unexpected upgrade", self.new + b"\0\1"):
            self.snapshot["value"][1] = account(check.LOADER, self.header + raw, 500)
            with self.assertRaisesRegex(ValueError, "unrecognized deployed ELF"):
                self.assess()

    def test_link_and_upgrade_authority_must_match(self):
        wrong_link = copy.deepcopy(self.snapshot)
        wrong_link["value"][0] = account(check.LOADER, (2).to_bytes(4, "little") + bytes(32), executable=True)
        with self.assertRaisesRegex(ValueError, "link changed"):
            self.assess(wrong_link)
        wrong_auth = copy.deepcopy(self.snapshot)
        wrong_auth["value"][1] = account(check.LOADER, self.header[:13] + bytes(32) + self.old, 100)
        with self.assertRaisesRegex(ValueError, "authority changed"):
            self.assess(wrong_auth)

    def test_wallet_role_and_loader_changes_refuse(self):
        for index, field, value in ((2, "owner", check.LOADER), (3, "executable", True), (1, "owner", check.SYSTEM)):
            changed = copy.deepcopy(self.snapshot)
            changed["value"][index][field] = value
            with self.assertRaises(ValueError):
                self.assess(changed)

    def test_unqualified_artifact_refuses(self):
        with self.assertRaisesRegex(ValueError, "unqualified refund ELF"):
            check.assess(self.snapshot, self.new + b"mutation", 2000, 500, 100)

    def test_existing_rent_surplus_is_not_funding_income(self):
        self.snapshot["value"][1]["lamports"] = 9000
        result = self.assess()
        self.assertEqual(result["extension_rent_lamports"], 0)
        self.assertEqual(result["temporary_required_lamports"], 2100)

    def test_identity_only_matches_artifact_assessment(self):
        self.assertEqual(self.assess(), check.assess_identity(
            self.snapshot, len(self.new), check.REFUND_SHA, 2000, 500, 100))

    def test_invalid_amounts_and_target_refuse(self):
        for size, digest, rent, reserve in [(0, check.REFUND_SHA, 2000, 100),
                (True, check.REFUND_SHA, 2000, 100), (len(self.new), '0'*64, 2000, 100),
                (len(self.new), check.REFUND_SHA, -1, 100),
                (len(self.new), check.REFUND_SHA, 2000, True)]:
            with self.assertRaises(ValueError):
                check.assess_identity(self.snapshot, size, digest, rent, 500, reserve)


if __name__ == "__main__":
    unittest.main()
