"""Tests of tools/build_site.py: /usr/bin/python3 -m unittest tools/build_site_test.py
(the detection's test is `node site/detect_test.js`)."""
import json
import os
import re
import sys
import tempfile
import unittest

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import build_site  # noqa: E402

TAG = "v2026.1117.1748"


class Names(unittest.TestCase):
    def test_names_are_the_release_assets(self):
        # the 14 files of scripts/release-assets.sh, read from the script
        sh = open(os.path.join(build_site.ROOT, "scripts", "release-assets.sh")).read()
        expected = set()
        for m in re.finditer(r'^\s+"([^"]+)"$', sh, re.M):
            name = m.group(1).replace("$p", "OpenRB-338-$version")
            expected.add(name.replace("$version", TAG[1:]))
        ours = {a["name"] for a in build_site.assets(TAG[1:]) if a["name"] != "SHA256SUMS.txt"}
        self.assertEqual(ours, expected)
        self.assertEqual(len(expected), 14)


class Build(unittest.TestCase):
    def test_build(self):
        with tempfile.TemporaryDirectory() as d:
            out = os.path.join(d, "site")
            build_site.main(["--tag", TAG, "--out", out])
            page = open(os.path.join(out, "index.html"), encoding="utf-8").read()
            self.assertNotIn("{{", page)
            # what a search engine and a link preview read
            site = build_site.SITE_URL
            self.assertIn(f'<link rel="canonical" href="{site}">', page)
            self.assertIn(f'<meta property="og:image" content="{site}img/og.jpg">', page)
            self.assertEqual(page.count("<h1"), 1)
            # the visitor counter (GoatCounter), as given by its dashboard
            self.assertIn('<script data-goatcounter="https://pingw33n.goatcounter.com/count"\n'
                          '        async src="https://gc.zgo.at/count.js"></script>', page)
            self.assertLessEqual(len(build_site.DESCRIPTION), 160)
            ld = re.search(r'<script type="application/ld\+json">(.*?)</script>', page, re.S)
            self.assertEqual(json.loads(ld.group(1))["softwareVersion"], TAG[1:])
            self.assertIn(f"<loc>{site}</loc>", open(os.path.join(out, "sitemap.xml")).read())
            self.assertIn(f"releases/download/{TAG}/OpenRB-338-{TAG[1:]}-x64-setup.exe", page)
            for f in ("style.css", "app.js", "detect.js", "img/standalone.webp",
                      "img/plugin.webp", "img/logo.svg", "img/plate.webp", "img/og.jpg",
                      "fonts/Inter-Bold.woff", ".nojekyll"):
                self.assertTrue(os.path.exists(os.path.join(out, f)), f)

    def test_sizes_and_formats(self):
        sizes = {a["name"]: 13859023 for a in build_site.assets(TAG[1:])}
        page, _ = build_site.render("{{PANELS}}{{ALL_LIST}}{{MODS_BUTTON}}", TAG, sizes)
        # the list has every asset of the release, each system's with its mark
        self.assertEqual(page.count('<li>'), 15)
        for os_, n in (("mac", 6), ("windows", 4), ("linux", 4)):
            self.assertEqual(page.count(f'<li><svg class="osi" aria-hidden="true" width="16" '
                                        f'height="16"><use href="#i-{os_}"/>'), n)
        for text in (".dmg, 13.9 MB", ".pkg, 13.9 MB", ".exe, 13.9 MB", "AppImage, 13.9 MB",
                     ".tar.gz, 13.9 MB", "archive, .zip, 13.9 MB", ".zip, about 950 MB"):
            self.assertIn(text, page)
        self.assertEqual(build_site.size_text(1640), "2 kB")
        self.assertEqual(build_site.size_text(950_000_000), "950 MB")
        del sizes["SHA256SUMS.txt"]
        with self.assertRaises(SystemExit):
            build_site.render("{{PANELS}}", TAG, sizes)

    def test_bad_tag(self):
        with self.assertRaises(SystemExit):
            build_site.main(["--tag", "v1", "--out", "/nonexistent/x"])


if __name__ == "__main__":
    unittest.main()
