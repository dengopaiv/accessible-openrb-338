#!/usr/bin/env python3
"""Makes the landing page's textures (task-212). Needs Pillow and Firefox.

    tools/site_textures.py [--firefox /Applications/Firefox.app/Contents/MacOS/firefox]

The panel's plate, wood and desk are SVG noise filters (feTurbulence). As page backgrounds
they made Safari redraw the filters while scrolling (slow, with flicker), so the page uses
pictures of them: this draws each SVG once in headless Firefox and writes a WebP to site/img/:
  plate.webp  tools/site_textures/plate.svg  the gray speckled plate, a 256 x 256 tile
  wood.webp   tools/site_textures/wood.svg   the wooden side, a 16 x 512 tile that repeats down
  desk.webp   assets/ui/desk_tile.svg        the desk's speckle and brushing on the desk's color
                                             DESK, a 256 x 256 tile
Each is drawn at SCALE pixels per SVG unit; the noises are stitched, so the tiles repeat
without a seam.
"""
import argparse
import os
import subprocess
import tempfile

from PIL import Image

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
OUT = os.path.join(ROOT, "site", "img")
SCALE = 2
# between DESK_TOP and DESK_BOTTOM of crates/orb-ui/src/backdrop.rs; style.css has the same
DESK = "#181b1d"
QUALITY = 80
# name: (source, width, height in SVG units)
TEXTURES = {
    "plate": (os.path.join("tools", "site_textures", "plate.svg"), 256, 256),
    "wood": (os.path.join("tools", "site_textures", "wood.svg"), 16, 512),
    "desk": (os.path.join("assets", "ui", "desk_tile.svg"), 256, 256),
}


def main():
    ap = argparse.ArgumentParser(description=__doc__,
                                 formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--firefox", default="/Applications/Firefox.app/Contents/MacOS/firefox",
                    help="the Firefox binary")
    a = ap.parse_args()
    with tempfile.TemporaryDirectory() as tmp:
        for name, (src, w, h) in TEXTURES.items():
            w, h = w * SCALE, h * SCALE
            png = os.path.join(tmp, name + ".png")
            page = os.path.join(tmp, name + ".html")
            with open(page, "w", encoding="utf-8") as f:
                f.write(f'<!doctype html><body style="margin:0;background:{DESK}">'
                        f'<img src="file://{os.path.join(ROOT, src)}" width="{w}" height="{h}" '
                        f'style="display:block">')
            # its own profile, so a running Firefox does not take the command over
            subprocess.run(
                [a.firefox, "--headless", "--no-remote", "--profile",
                 os.path.join(tmp, "profile-" + name), f"--window-size={max(w, 500)},{h}",
                 "--screenshot", png, "file://" + page],
                check=True, capture_output=True, timeout=120)
            im = Image.open(png).convert("RGB").crop((0, 0, w, h))
            out = os.path.join(OUT, name + ".webp")
            im.save(out, "WEBP", quality=QUALITY, method=6)
            print(name, im.size, os.path.getsize(out) // 1024, "KiB")


if __name__ == "__main__":
    main()
