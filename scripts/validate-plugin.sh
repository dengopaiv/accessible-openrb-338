#!/usr/bin/env bash
# Runs the plugin validators on the bundles of `cargo xtask bundle orb-plugin --release`
# (task-204; docs/building.md "Validating the plugin"):
#   pluginval (Tracktion; it validates VST3) on OpenRB-338.vst3. It is downloaded as a pinned
#   release asset, checked against its SHA-256, into a temporary folder, and run from there.
#   A failure exits 1.
#
#   scripts/validate-plugin.sh [BUNDLE_DIR]      default: target/bundled
#
# Hosts: macOS (Apple silicon or Intel) and Linux x86_64 (pluginval needs a display there:
# the script runs it under xvfb-run when DISPLAY is empty; CI installs xvfb). Windows has no
# script: run pluginval by hand (docs/building.md).
set -euo pipefail

root=$(cd "$(dirname "$0")/.." && pwd)
bundles=${1:-$root/target/bundled}
vst3_bundle=$bundles/OpenRB-338.vst3
[ -e "$vst3_bundle" ] || {
    echo "no bundle in $bundles (cargo xtask bundle orb-plugin --release)" >&2
    exit 1
}

# The pinned version: pluginval 1.0.4. A new version is a deliberate edit of these lines with its checksum.
gh=https://github.com
pv_base=$gh/Tracktion/pluginval/releases/download/v1.0.4
case $(uname -s) in
    Darwin)
        pv_asset=pluginval_macOS.zip
        pv_sha=3c4c533bda0c5059eea3ddaea752d757ee2025041f0f47e6bcb0e87f6082b29f
        ;;
    Linux)
        [ "$(uname -m)" = x86_64 ] || { echo "validators: Linux x86_64 only" >&2; exit 1; }
        pv_asset=pluginval_Linux.zip
        pv_sha=c01c49d8063965c4c2dea8324468336768f5c9139e0b1caebde14c2400b55352
        ;;
    *) echo "validators: macOS and Linux only (docs/building.md)" >&2; exit 1 ;;
esac

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

sha256() { if command -v sha256sum >/dev/null; then sha256sum "$1"; else shasum -a 256 "$1"; fi; }
fetch() { # URL SHA256 DIR: download, check, unzip into DIR
    mkdir -p "$3"
    curl -fsSL --retry 3 -o "$3/asset.zip" "$1"
    [ "$(sha256 "$3/asset.zip" | cut -d' ' -f1)" = "$2" ] \
        || { echo "checksum mismatch for $1" >&2; exit 1; }
    unzip -q "$3/asset.zip" -d "$3"
}
fetch "$pv_base/$pv_asset" "$pv_sha" "$tmp/pv"
if [ "$(uname -s)" = Darwin ]; then
    pv=$tmp/pv/pluginval.app/Contents/MacOS/pluginval
else
    pv=$tmp/pv/pluginval
fi
chmod +x "$pv"

status=0
echo "=== pluginval (strictness 5): $vst3_bundle"
run_pv=("$pv" --strictness-level 5 --skip-gui-tests --validate "$vst3_bundle")
if [ "$(uname -s)" = Linux ] && [ -z "${DISPLAY:-}" ]; then
    xvfb-run -a "${run_pv[@]}" || status=1
else
    "${run_pv[@]}" || status=1
fi

[ $status -eq 0 ] && echo "validator: passed" || echo "validator: FAILED" >&2
exit $status
