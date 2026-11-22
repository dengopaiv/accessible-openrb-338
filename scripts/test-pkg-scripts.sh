#!/usr/bin/env bash
# Checks the plugin .pkg's preinstall script (scripts/pkg/preinstall, task-278) without
# installing anything: it runs the script against a scratch "volume" in a temporary folder
# (the script's $3) with a stand-in pkgutil first on PATH, and asserts that
#   - the old OpenRB-338.vst3 and OpenRB-338.clap are gone,
#   - every other file in the VST3 and CLAP folders (and the folders) is still there,
#   - the receipt of the old CLAP package is forgotten (pkgutil --forget the identifier).
# It also runs the script on a volume with none of the bundles (a fresh Mac). No /Library is
# read or written. Runs on macOS and Linux (CI: ci.yml).
#
#   scripts/test-pkg-scripts.sh
set -euo pipefail

root=$(cd "$(dirname "$0")/.." && pwd)
script=$root/scripts/pkg/preinstall
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
fail() { echo "FAIL: $*" >&2; exit 1; }

# a stand-in pkgutil: records its arguments
mkdir "$tmp/bin"
cat > "$tmp/bin/pkgutil" <<EOF
#!/bin/sh
echo "\$@" >> "$tmp/pkgutil.log"
EOF
chmod +x "$tmp/bin/pkgutil"

vol=$tmp/vol
plug=$vol/Library/Audio/Plug-Ins
mkdir -p "$plug/VST3/OpenRB-338.vst3/Contents/MacOS" "$plug/VST3/Other.vst3" \
    "$plug/CLAP/OpenRB-338.clap/Contents/MacOS" "$plug/CLAP/Other.clap" \
    "$plug/CLAP/OpenRB-338.clap.keep"
touch "$plug/VST3/OpenRB-338.vst3/Contents/MacOS/OpenRB-338" \
    "$plug/CLAP/OpenRB-338.clap/Contents/MacOS/OpenRB-338" "$plug/CLAP/note.txt"

PATH=$tmp/bin:$PATH sh "$script" "" "" "$vol" || fail "the script failed"

[ ! -e "$plug/VST3/OpenRB-338.vst3" ] || fail "the old VST3 bundle is still there"
[ ! -e "$plug/CLAP/OpenRB-338.clap" ] || fail "the old CLAP bundle is still there"
[ -d "$plug/VST3/Other.vst3" ] || fail "another VST3 plugin was removed"
[ -d "$plug/CLAP/Other.clap" ] || fail "another CLAP plugin was removed"
[ -d "$plug/CLAP/OpenRB-338.clap.keep" ] || fail "a sibling of the bundle was removed"
[ -f "$plug/CLAP/note.txt" ] || fail "a file in the CLAP folder was removed"
[ -d "$plug/CLAP" ] && [ -d "$plug/VST3" ] || fail "a plugin folder was removed"
grep -qx -- "--volume $vol --forget net.emphased.openrb338.plugin.clap" "$tmp/pkgutil.log" \
    || fail "the old CLAP receipt is not forgotten: $(cat "$tmp/pkgutil.log")"

# a fresh Mac: no folders at all; and a pkgutil that fails changes nothing
fresh=$tmp/fresh
mkdir "$fresh"
cat > "$tmp/bin/pkgutil" <<EOF
#!/bin/sh
exit 1
EOF
PATH=$tmp/bin:$PATH sh "$script" "" "" "$fresh" || fail "the script failed on a fresh volume"
[ -z "$(ls -A "$fresh")" ] || fail "the script created something on a fresh volume"

echo "pkg scripts: all passed"
