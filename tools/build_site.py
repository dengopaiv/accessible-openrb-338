#!/usr/bin/env python3
"""Builds the GitHub Pages site into one folder (task-212).

    tools/build_site.py --tag v2026.1117.1748 --out _site [--release-sizes] [--check-urls]
        [--web DIR]

The landing page is site/ (index.html, style.css, app.js, detect.js, img/, fonts/). The download
links are written into the HTML at build time from the release TAG: the asset names are the ones
scripts/release-assets.sh checks, built from the tag's version, so the visitor's browser never
calls the GitHub API. --release-sizes reads the release's assets with `gh release view` and
writes each file's size on its button (without it a button names the format only); it fails
when the release lacks a file the page links. --check-urls sends an HTTP HEAD for every link
and fails on any that does not answer 200 (a draft release's assets are not public, so the
check fails until it is published). --web DIR copies the web app's build to OUT/web/ (task-175).
"""
import argparse
import html
import json
import os
import re
import shutil
import subprocess
import sys
import urllib.error
import urllib.request

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
REPO = "https://github.com/pingw33n/OpenRB-338"
# where the site is served: the canonical address, the link preview's and the sitemap's
SITE_URL = "https://pingw33n.github.io/OpenRB-338/"
DESCRIPTION = ("A free, open-source reimplementation of ReBirth RB-338 (two TB-303s, a TR-808 "
               "and a TR-909): standalone app and VST 3 plugin for macOS, Windows and "
               "Linux.")
SONGS_ZIP = "https://github.com/pingw33n/rb-songs/archive/refs/heads/main.zip"
MODS_ZIP = "https://github.com/pingw33n/rb-mods/archive/refs/heads/main.zip"
# GitHub makes these two zips on request and sends no length, so the sizes are written here:
# rb-songs measured 8,275,852 bytes on 2026-11-18, rb-mods is the task's figure.
SONGS_DETAIL = ".zip, about 8 MB"
MODS_DETAIL = ".zip, about 950 MB"

MAC = {"arm64": "aarch64-apple-darwin", "x64": "x86_64-apple-darwin"}
WIN = {"x64": "x86_64-pc-windows-msvc", "arm64": "aarch64-pc-windows-msvc"}
LIN = {"x64": "x86_64-unknown-linux-gnu", "arm64": "aarch64-unknown-linux-gnu"}
ARCH_LABEL = {"arm64": "ARM64", "x64": "x64"}
MAC_LABEL = {"arm64": "Apple silicon", "x64": "Intel"}
LINUX_APPIMAGE = {"arm64": "aarch64", "x64": "x86_64"}


def asset(id, os_, arch, roles, label, note, name):
    return dict(id=id, os=os_, arch=arch, roles=roles, label=label, note=note, name=name)


def assets(version):
    """Every download of the page. roles: app, plugin (the buttons), archive (plain packages)."""
    v = version
    out = []
    for arch in ("arm64", "x64"):
        out.append(asset(f"mac-{arch}-dmg", "mac", arch, ["app"], f"macOS, {MAC_LABEL[arch]}",
                         "standalone, .dmg", f"OpenRB-338-{v}-{MAC[arch]}.dmg"))
    for arch in ("arm64", "x64"):
        out.append(asset(f"mac-{arch}-pkg", "mac", arch, ["plugin"], f"macOS, {MAC_LABEL[arch]}",
                         "plugins, .pkg",
                         f"OpenRB-338-Plugins-{v}-{MAC[arch]}.pkg"))
    for arch in ("x64", "arm64"):
        out.append(asset(f"windows-{arch}-setup", "windows", arch, ["app", "plugin"],
                         f"Windows, {ARCH_LABEL[arch]}", "standalone and plugins, .exe",
                         f"OpenRB-338-{v}-{arch}-setup.exe"))
    for arch in ("x64", "arm64"):
        out.append(asset(f"linux-{arch}-appimage", "linux", arch, ["app"],
                         f"Linux, {ARCH_LABEL[arch]}", "standalone, AppImage",
                         f"OpenRB-338-{v}-{LINUX_APPIMAGE[arch]}.AppImage"))
    for arch in ("x64", "arm64"):
        out.append(asset(f"linux-{arch}-tar", "linux", arch, ["plugin", "archive"],
                         f"Linux, {ARCH_LABEL[arch]}", "archive, .tar.gz",
                         f"OpenRB-338-{v}-{LIN[arch]}.tar.gz"))
    # the plain archives with the two binaries
    for arch in ("arm64", "x64"):
        out.append(asset(f"mac-{arch}-tar", "mac", arch, ["archive"],
                         f"macOS, {MAC_LABEL[arch]}", "archive, .tar.gz",
                         f"OpenRB-338-{v}-{MAC[arch]}.tar.gz"))
    for arch in ("x64", "arm64"):
        out.append(asset(f"windows-{arch}-zip", "windows", arch, ["archive"],
                         f"Windows, {ARCH_LABEL[arch]}", "archive, .zip",
                         f"OpenRB-338-{v}-{WIN[arch]}.zip"))
    out.append(asset("sums", None, None, ["archive"], "Checksums", "SHA256SUMS.txt",
                     "SHA256SUMS.txt"))
    return out


def link(tag, a):
    return f"{REPO}/releases/download/{tag}/{a['name']}"


def file_format(name):
    """What a button says is downloaded: .dmg, .pkg, .exe, AppImage, .tar.gz, .zip."""
    if name.endswith(".tar.gz"):
        return ".tar.gz"
    ext = name.rsplit(".", 1)[-1]
    return ext if ext == "AppImage" else "." + ext


def size_text(size):
    """A size in bytes as the page shows it, in decimal megabytes: 13.9 MB, 950 MB."""
    mb = size / 1e6
    if mb < 0.1:
        return f"{max(1, round(size / 1e3))} kB"
    return f"{mb:.1f} MB" if mb < 100 else f"{round(mb)} MB"


def with_size(text, a):
    return f"{text}, {size_text(a['size'])}" if a.get("size") else text


def release_sizes(tag):
    """The size in bytes of each asset of the release TAG, by name, from `gh` (which also sees
    a draft release when it is logged in as the owner)."""
    cmd = ["gh", "release", "view", tag, "--repo", REPO.split("github.com/")[1], "--json",
           "assets"]
    try:
        out = subprocess.run(cmd, check=True, capture_output=True, text=True).stdout
    except (OSError, subprocess.CalledProcessError) as e:
        sys.exit(f"cannot read release {tag} with gh: {getattr(e, 'stderr', '') or e}")
    return {x["name"]: x["size"] for x in json.loads(out)["assets"]}


def os_icon(os_):
    """The system's mark (a symbol of index.html), or the same space left empty."""
    use = f'<use href="#i-{os_}"/>' if os_ else ""
    return f'<svg class="osi" aria-hidden="true" width="16" height="16">{use}</svg>'


def rows(items, tag):
    """Every asset of the release, by system (macOS, Windows, Linux), each with its mark."""
    order = {"mac": 0, "windows": 1, "linux": 2, None: 3}
    h = []
    for a in sorted(items, key=lambda a: order[a["os"]]):
        h.append(
            f'<li>{os_icon(a["os"])}<a class="dl" href="{html.escape(link(tag, a))}">'
            f'{html.escape(a["label"])}</a> '
            f'<span class="note">{html.escape(with_size(a["note"], a))}</span></li>')
    return "\n".join(h)


ICON = '<svg class="ico" aria-hidden="true" width="16" height="16"><use href="#i-download"/></svg>'


def button_html(href, text, detail, attrs=""):
    """A download button: what it is, and under it the file's format and size."""
    return (f'<a class="button" href="{html.escape(href)}"{attrs}>{ICON}<span class="bl">'
            f'<span class="bt">{html.escape(text)}</span>'
            f'<span class="bs">{html.escape(detail)}</span></span></a>')


def button(tag, a, text):
    return button_html(link(tag, a), text, with_size(file_format(a["name"]), a),
                       f' data-os="{a["os"]}" data-kind="{a["id"].split("-")[-1]}"')


def panels(tag, items):
    """One panel per system, in an arch group each: Standalone and Plugins buttons."""
    by = {a["id"]: a for a in items}
    h = []
    for os_, title, archs, arch_label in (
            ("mac", "macOS", ("arm64", "x64"), MAC_LABEL),
            ("windows", "Windows", ("x64", "arm64"), ARCH_LABEL),
            ("linux", "Linux", ("x64", "arm64"), ARCH_LABEL)):
        h.append(f'<div class="os" id="os-{os_}" data-os="{os_}"><h3>{os_icon(os_)}{title}</h3>')
        for arch in archs:
            h.append(f'<div class="arch" data-arch="{arch}"><h4>{arch_label[arch]}</h4><p>')
            if os_ == "mac":
                h.append(button(tag, by[f"mac-{arch}-dmg"], "Standalone"))
                h.append(button(tag, by[f"mac-{arch}-pkg"], "Plugins"))
            elif os_ == "windows":
                h.append(button(tag, by[f"windows-{arch}-setup"], "Standalone and plugins"))
            else:
                h.append(button(tag, by[f"linux-{arch}-appimage"], "Standalone"))
                h.append(button(tag, by[f"linux-{arch}-tar"], "Plugins"))
            h.append("</p></div>")
        h.append("</div>")
    return "\n".join(h)


def json_ld(tag):
    """The page's subject for search engines: a schema.org SoftwareApplication."""
    data = {
        "@context": "https://schema.org",
        "@type": "SoftwareApplication",
        "name": "OpenRB-338",
        "description": DESCRIPTION,
        "url": SITE_URL,
        "image": SITE_URL + "img/og.jpg",
        "applicationCategory": "MultimediaApplication",
        "operatingSystem": "macOS, Windows, Linux",
        "softwareVersion": tag[1:],
        "downloadUrl": f"{REPO}/releases/tag/{tag}",
        "license": "https://www.gnu.org/licenses/gpl-3.0.html",
        "isAccessibleForFree": True,
        "offers": {"@type": "Offer", "price": "0", "priceCurrency": "USD"},
    }
    # "</" cannot end the script element early
    return json.dumps(data, indent=1).replace("</", "<\\/")


def sitemap():
    return ('<?xml version="1.0" encoding="UTF-8"?>\n'
            '<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">\n'
            f"  <url><loc>{SITE_URL}</loc></url>\n</urlset>\n")


def render(template, tag, sizes=None):
    """The page for the release TAG. `sizes` (name to bytes, release_sizes) puts each file's
    size on its button; every download of the page must then be in it."""
    version = tag[1:]
    items = assets(version)
    if sizes is not None:
        missing = [a["name"] for a in items if a["name"] not in sizes]
        if missing:
            sys.exit(f"release {tag} lacks {len(missing)} asset(s): {', '.join(missing[:3])}")
        for a in items:
            a["size"] = sizes[a["name"]]
    subst = {
        "TAG": tag, "VERSION": version, "REPO": REPO,
        "SONGS_BUTTON": button_html(SONGS_ZIP, "Song collection", SONGS_DETAIL),
        "MODS_BUTTON": button_html(MODS_ZIP, "Mod collection", MODS_DETAIL),
        "RELEASE_URL": f"{REPO}/releases/tag/{tag}",
        "SITE_URL": SITE_URL, "DESCRIPTION": html.escape(DESCRIPTION),
        "JSON_LD": json_ld(tag),
        "PANELS": panels(tag, items), "ALL_LIST": rows(items, tag),
    }
    return re.sub(r"\{\{([A-Z_]+)\}\}", lambda m: subst[m.group(1)], template), items


def check_urls(urls):
    bad = []
    for u in urls:
        req = urllib.request.Request(u, method="HEAD")
        try:
            with urllib.request.urlopen(req, timeout=60) as r:
                status = r.status
        except urllib.error.HTTPError as e:
            status = e.code
        except OSError as e:
            status = str(e)
        print(f"HEAD {status} {u}")
        if status != 200:
            bad.append(u)
    return bad


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__,
                                 formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--tag", required=True, help="the release tag, v<YYYY.MMDD.HHMM>")
    ap.add_argument("--out", required=True, help="the output folder (emptied first)")
    ap.add_argument("--check-urls", action="store_true", help="HEAD every download link")
    ap.add_argument("--release-sizes", action="store_true",
                    help="read the release with gh: put each file's size on its button and "
                         "fail when the release lacks a file the page links")
    ap.add_argument("--web", help="the web app's build, copied to OUT/web/ (task-175)")
    a = ap.parse_args(argv)
    if not re.fullmatch(r"v\d{4}\.\d{4}\.\d{4}", a.tag):
        sys.exit(f"not a release tag: {a.tag}")
    src = os.path.join(ROOT, "site")
    with open(os.path.join(src, "index.html"), encoding="utf-8") as f:
        page, items = render(f.read(), a.tag, release_sizes(a.tag) if a.release_sizes else None)
    if a.check_urls:
        bad = check_urls([link(a.tag, x) for x in items] + [SONGS_ZIP, MODS_ZIP])
        if bad:
            sys.exit(f"{len(bad)} link(s) do not resolve (is the release published, with its "
                     f"assets?): {', '.join(bad[:3])}")
    if os.path.isdir(a.out):
        shutil.rmtree(a.out)
    os.makedirs(a.out)
    for name in ("style.css", "app.js", "detect.js"):
        shutil.copy(os.path.join(src, name), a.out)
    for folder in ("img", "fonts"):
        shutil.copytree(os.path.join(src, folder), os.path.join(a.out, folder))
    with open(os.path.join(a.out, "index.html"), "w", encoding="utf-8") as f:
        f.write(page)
    with open(os.path.join(a.out, "sitemap.xml"), "w", encoding="utf-8") as f:
        f.write(sitemap())
    open(os.path.join(a.out, ".nojekyll"), "w").close()
    if a.web:
        shutil.copytree(a.web, os.path.join(a.out, "web"))
    print(f"built {a.out} for {a.tag}: {len(items)} release assets linked")


if __name__ == "__main__":
    main()
