#!/usr/bin/env python3
"""Makes the landing page's screenshots and favicons (task-212). Needs Pillow.

    tools/site_images.py --standalone WINDOW.png --reaper WINDOW.png --plugin WINDOW.png \
        [--iconset DIR]
    tools/site_images.py --iconset DIR        (only the two icons)

The inputs are macOS window screenshots (Retina, with the window's shadow in the alpha
channel). Writes to site/img/:
  standalone.webp, standalone-full.webp   the app (page copy 1200 px wide, full size for the zoom)
  plugin.webp, plugin-full.webp           the plugin window laid over the REAPER window
  og.jpg                                  the link preview's picture, 1200 x 630, of the app
  favicon-32.png, apple-touch-icon.png    the icon's tile (the 52 px rounded square of the
                                          64 px icon, no margin) cropped from the 1024 px PNG
                                          of `orb-ui-art iconset DIR`, with --iconset; the
                                          screenshots are not needed for these alone
Metadata (EXIF, XMP, ICC) is dropped after converting the colors to sRGB. The plugin window's
place on the REAPER window, PLUGIN_AT, was found by matching the two shots against a full-screen
shot of the intended layout (the plugin's interior matched pixel for pixel).
"""
import argparse
import io
import os

from PIL import Image, ImageCms

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
OUT = os.path.join(ROOT, "site", "img")
PLUGIN_AT = (782, 424)  # the plugin shot's (0, 0) on the REAPER shot, device pixels
PAGE_W = 1200  # a column of the page is at most 490 px wide, so this is over 2x
PAGE_QUALITY = 75  # the page copy is seen scaled down; the full-size copy keeps QUALITY
QUALITY = 92
OG_SIZE = (1200, 630)
OG_MARGIN = 24
OG_DESK = (0x18, 0x1b, 0x1d)


def load_srgb(path):
    im = Image.open(path)
    icc = im.info.get("icc_profile")
    im = im.convert("RGBA")
    if icc:
        src = ImageCms.ImageCmsProfile(io.BytesIO(icc))
        dst = ImageCms.createProfile("sRGB")
        rgb = ImageCms.profileToProfile(im.convert("RGB"), src, dst, outputMode="RGB")
        rgb.putalpha(im.getchannel("A"))
        im = rgb
    return im


def save(im, name):
    # the bounding box of everything not fully transparent (shadows included)
    box = im.getchannel("A").point(lambda a: 255 if a else 0).getbbox()
    im = im.crop(box)
    full = os.path.join(OUT, name + "-full.webp")
    im.save(full, "WEBP", quality=QUALITY, method=6)
    page = im.resize((PAGE_W, round(im.height * PAGE_W / im.width)), Image.LANCZOS)
    page.save(os.path.join(OUT, name + ".webp"), "WEBP", quality=PAGE_QUALITY, method=6)
    print(name, im.size, "->", page.size, os.path.getsize(full) // 1024, "KiB full")


def save_og(im):
    """The link preview's picture (Open Graph), 1200 x 630 JPEG: the standalone window, its
    shadow cut off, fitted on the desk's color."""
    box = im.getchannel("A").point(lambda a: 255 if a == 255 else 0).getbbox()
    im = im.crop(box)
    w, h = OG_SIZE
    k = min((w - 2 * OG_MARGIN) / im.width, (h - 2 * OG_MARGIN) / im.height)
    im = im.resize((round(im.width * k), round(im.height * k)), Image.LANCZOS)
    canvas = Image.new("RGB", OG_SIZE, OG_DESK)
    canvas.paste(im, ((w - im.width) // 2, (h - im.height) // 2), im)
    out = os.path.join(OUT, "og.jpg")
    canvas.save(out, "JPEG", quality=85, optimize=True, progressive=True)
    print("og", canvas.size, os.path.getsize(out) // 1024, "KiB")


def main():
    ap = argparse.ArgumentParser(description=__doc__,
                                 formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--standalone")
    ap.add_argument("--reaper")
    ap.add_argument("--plugin")
    ap.add_argument("--iconset")
    a = ap.parse_args()
    shots = (a.standalone, a.reaper, a.plugin)
    if any(shots) and not all(shots):
        ap.error("--standalone, --reaper and --plugin go together")
    if not all(shots) and not a.iconset:
        ap.error("give the three screenshots, --iconset, or both")
    os.makedirs(OUT, exist_ok=True)
    if all(shots):
        standalone = load_srgb(a.standalone)
        save(standalone, "standalone")
        save_og(standalone)
        reaper = load_srgb(a.reaper)
        plugin = load_srgb(a.plugin)
        canvas = Image.new("RGBA", reaper.size, (0, 0, 0, 0))
        canvas.alpha_composite(reaper)
        canvas.alpha_composite(plugin, PLUGIN_AT)  # the shadows of both windows stay
        save(canvas, "plugin")
    if a.iconset:
        big = Image.open(os.path.join(a.iconset, "icon_512x512@2x.png")).convert("RGBA")
        tile = big.crop((96, 96, 928, 928))  # the 52 px tile of the 64 px icon, full bleed
        tile.resize((32, 32), Image.LANCZOS).save(os.path.join(OUT, "favicon-32.png"))
        tile.resize((180, 180), Image.LANCZOS).convert("RGB").save(
            os.path.join(OUT, "apple-touch-icon.png"))


if __name__ == "__main__":
    main()
