"""Tests of tools/thirdparty_mods_manifest.py: /usr/bin/python3 -m unittest
tools/thirdparty_mods_manifest_test.py. A synthetic mod and a sparse RB20FUL.DAT stand-in."""
import hashlib
import json
import os
import struct
import sys
import tempfile
import unittest

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import rbm  # noqa: E402
import rbm_test  # noqa: E402
import thirdparty_mods_manifest as tm  # noqa: E402


class Manifest(unittest.TestCase):
    def test_entry_has_hashes_and_names_only(self):
        samples = [(i * 37) % 2000 - 1000 for i in range(300)]
        aif = rbm_test.aiff(samples)
        files = [("TR808BD.aif", aif), ("12017.jpg", b"\xff\xd8" + bytes(20))]
        with tempfile.TemporaryDirectory() as d:
            orig, mods = os.path.join(d, "orig"), os.path.join(d, "mods")
            os.mkdir(orig)
            os.mkdir(mods)
            with open(os.path.join(orig, "Syn.rbm"), "wb") as f:
                f.write(rbm_test.build_mod(files))
            with open(os.path.join(mods, "Syn.rbm"), "wb") as f:
                f.write(b"recrypted stand-in")
            dat = os.path.join(d, "RB20FUL.DAT")
            with open(dat, "wb") as f:
                f.write(struct.pack(">I", rbm.DAT_MAGIC))
                f.truncate(rbm.DAT_HEADER + rbm.DAT_BODY)
            m = tm.build_manifest(orig, mods, dat)
            text = tm.dumps(m)
        self.assertEqual(json.loads(text), m)
        (e,) = m["mods"]
        self.assertEqual(e["file"], "Syn.rbm")
        self.assertEqual(e["name"], "Synthetic")
        self.assertIsNone(e["error"])
        self.assertEqual(e["sha256"], hashlib.sha256(b"recrypted stand-in").hexdigest())
        self.assertEqual([n for n, _ in e["files"]], ["TR808BD.aif", "12017.jpg"])
        self.assertEqual(e["files"][0][1], hashlib.sha256(aif).hexdigest())
        pcm = struct.pack(">300h", *samples)
        self.assertEqual(e["slots"], {"TR808BD.aif": hashlib.sha256(pcm).hexdigest()})
        self.assertEqual(e["slot_errors"], {})
        # no file content: only 64-hex digests and names
        self.assertNotIn("ffd8", text)

    def test_mod_without_original_and_kept_entry(self):
        aif = rbm_test.aiff([(i * 37) % 2000 - 1000 for i in range(300)])
        files = [("TR808BD.aif", aif), (tm.PAD_NAME, bytes(8))]
        with tempfile.TemporaryDirectory() as d:
            mods = os.path.join(d, "mods")
            os.mkdir(mods)
            for n, body in (("Made.rbm", rbm_test.build_mod(files)), ("Old.rbm", b"old bytes")):
                with open(os.path.join(mods, n), "wb") as f:
                    f.write(body)
            dat = os.path.join(d, "RB20FUL.DAT")
            with open(dat, "wb") as f:
                f.write(struct.pack(">I", rbm.DAT_MAGIC))
                f.truncate(rbm.DAT_HEADER + rbm.DAT_BODY)
            old = {"file": "Old.rbm", "sha256": hashlib.sha256(b"old bytes").hexdigest(),
                   "name": "kept"}
            m = tm.build_manifest(None, mods, dat, keep={"mods": [old]})
        made, kept = m["mods"]
        self.assertEqual(made["file"], "Made.rbm")
        self.assertIs(made["original"], False)
        self.assertEqual([n for n, _ in made["files"]], ["TR808BD.aif"])  # the pad is dropped
        self.assertEqual(kept, old)
        # --check skips what has no original
        real = tm.recrypt_originals
        tm.recrypt_originals = lambda *a: {"Old.rbm": old["sha256"]}
        try:
            self.assertEqual(tm.check_manifest("o", "d", m, "orb"), [])
        finally:
            tm.recrypt_originals = real


if __name__ == "__main__":
    unittest.main()
