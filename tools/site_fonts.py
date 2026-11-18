#!/usr/bin/env python3
"""Makes the landing page's fonts (task-212). Needs fontTools.

    tools/site_fonts.py

The panel's labels are Inter (crates/orb-ui-art/fonts/, SIL Open Font License 1.1), so the page
uses it too. Writes to site/fonts/ a WOFF of Inter Medium and of Inter Bold with the printable
ASCII characters and a few signs only, and a copy of the license.
"""
import os
import shutil

from fontTools import subset

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SRC = os.path.join(ROOT, "crates", "orb-ui-art", "fonts")
OUT = os.path.join(ROOT, "site", "fonts")
# printable ASCII, no-break space, copyright, middle dot, dashes, quotes, ellipsis, arrows
UNICODES = "U+0020-007E,U+00A0,U+00A9,U+00B7,U+2013,U+2014,U+2018-201D,U+2026,U+2192"


def main():
    os.makedirs(OUT, exist_ok=True)
    for name in ("Inter-Medium", "Inter-Bold"):
        out = os.path.join(OUT, name + ".woff")
        subset.main([os.path.join(SRC, name + ".ttf"), "--unicodes=" + UNICODES,
                     "--layout-features=kern,liga,calt,tnum", "--flavor=woff",
                     "--output-file=" + out])
        print(name, os.path.getsize(out) // 1024, "KiB")
    shutil.copy(os.path.join(SRC, "LICENSE.txt"), os.path.join(OUT, "LICENSE.txt"))


if __name__ == "__main__":
    main()
