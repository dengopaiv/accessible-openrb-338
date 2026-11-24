#!/usr/bin/env bash
# Builds dist/<target>/OpenRB-338-<YYYY.MMDD.HHMM>-<x64|arm64>-setup.exe, the NSIS installer of
# OpenRB-338 and orb for a Windows target (task-111; the version is the UTC build time,
# scripts/version.sh).
#
#   scripts/package-installer.sh x86_64-pc-windows-msvc|aarch64-pc-windows-msvc
#
# Builds the target with scripts/build.sh first; SKIP_BUILD=1 uses the binaries already in
# dist/<target>/bin/ and the plugin bundles in dist/<target>/plugins/ (release.yml downloads
# them from the package jobs; the plugin is the installer's optional component). makensis runs
# on any host (macOS: brew install makensis; Ubuntu: apt install nsis; Windows runners: choco install
# nsis), so one script serves this Mac and CI. The installer itself is scripts/installer/
# openrb.nsi; docs/building.md ("The Windows installer") describes it.
# Environment as scripts/build.sh. MAKENSIS names the makensis to use; MAKENSIS_FLAGS replaces
# the default flag -WX (release.yml sets it empty: Ubuntu's NSIS is older than the 3.13 this
# was written against, and a new warning there should not stop a release).
set -euo pipefail

root=$(cd "$(dirname "$0")/.." && pwd)
target=${1:?usage: scripts/package-installer.sh x86_64-pc-windows-msvc|aarch64-pc-windows-msvc}
dist=${DIST:-$root/dist}
case $target in
    x86_64-pc-windows-msvc) arch=x64 ;;
    aarch64-pc-windows-msvc) arch=arm64 ;;
    *) echo "not a Windows target: $target" >&2; exit 2 ;;
esac
makensis=${MAKENSIS:-makensis}
command -v "$makensis" >/dev/null || {
    echo "makensis is missing (brew install makensis; apt install nsis; choco install nsis)" >&2
    exit 1
}
export SOURCE_DATE_EPOCH=${SOURCE_DATE_EPOCH:-$(date +%s)}
version=$("$root/scripts/version.sh")

[ -n "${SKIP_BUILD:-}" ] || "$root/scripts/build.sh" "$target"
bin=$dist/$target/bin
for f in OpenRB-338.exe orb.exe; do
    [ -f "$bin/$f" ] || { echo "missing $bin/$f (run scripts/build.sh $target)" >&2; exit 1; }
done
plugins=$dist/$target/plugins
p=OpenRB-338.vst3
[ -e "$plugins/$p" ] || { echo "missing $plugins/$p (run scripts/build.sh $target)" >&2; exit 1; }

# The license page and the readme are shown by Windows programs: CRLF line ends.
stage=$dist/$target/installer-stage
rm -rf "$stage"
mkdir -p "$stage"
crlf() { awk '{ printf "%s\r\n", $0 }' "$1" > "$2"; }
crlf "$root/LICENSE" "$stage/LICENSE.txt"
# the x86_64 build carries ASIO: its SDK's license and notices (task-193)
notices=
if [ "$arch" = x64 ]; then
    "$root/scripts/asio-notices.sh" "$stage/notices.in"
    crlf "$stage/notices.in" "$stage/THIRD-PARTY-NOTICES.txt"
    rm "$stage/notices.in"
    notices=1
fi
cat > "$stage/readme.in" <<EOF
OpenRB-338 $version ($arch)

A reimplementation of the audio engine of Propellerhead ReBirth RB-338 2.01.
It needs no file of the original program: the standard drum kit is built in.

OpenRB-338.exe is the desktop app (Start menu: OpenRB-338). orb.exe is the command line tool:

  orb render SONG.rbs -o SONG.flac    render a song to FLAC, Ogg Vorbis or WAV
  orb --help                          all commands

To run orb from any folder, add this folder to your PATH: run the installer again with "Add orb to
the PATH" ticked, or add it in Settings > System > About > Advanced system settings > Environment
Variables. The uninstaller (Settings > Apps) removes what the installer added; your songs and the
settings file (%APPDATA%\OpenRB-338) are kept.

The installer is for all users and asks for administrator rights: the program goes to Program
Files, the optional PATH entry to the machine's PATH. The VST3 plugin (component "VST3 plugin",
on by default) goes to the plugin folder every host scans: OpenRB-338.vst3 to
%COMMONPROGRAMFILES%\VST3. Rescan plugins in your host. The uninstaller removes the plugin and
nothing else in that folder.

An earlier per-user install (versions before the plugins) is not removed: take it out in
Settings > Apps.

The installer and the programs are not signed, so Windows SmartScreen may say "Windows protected
your PC": choose "More info", then "Run anyway".
Licensed under the GNU General Public License, version 3 only (GPL-3.0-only; see LICENSE.txt).
EOF
if [ -n "$notices" ]; then
    cat >> "$stage/readme.in" <<'EOF'

OpenRB-338.exe can play through ASIO drivers: Options > Audio Output..., Driver: ASIO. The default
stays WASAPI; ASIO is never chosen by itself. THIRD-PARTY-NOTICES.txt holds the license of the
ASIO SDK.
ASIO is a trademark and software of Steinberg Media Technologies GmbH.
EOF
fi
crlf "$stage/readme.in" "$stage/README.txt"
rm "$stage/readme.in"
# NSIS stores the time of every file it packs: copy all of them here and give them the build
# time, so two runs of one build make the same installer (BSD date -r, GNU date -d)
cp "$bin/OpenRB-338.exe" "$bin/orb.exe" "$root/scripts/installer/path.ps1" "$stage/"
# the plugin bundle (task-237): a .vst3 folder on Windows
mkdir "$stage/plugins"
cp -R "$plugins/OpenRB-338.vst3" "$stage/plugins/"
[ -d "$stage/plugins/OpenRB-338.vst3" ] \
    || { echo "OpenRB-338.vst3 must be a folder" >&2; exit 1; }
stamp=$(TZ=UTC date -r "$SOURCE_DATE_EPOCH" +%Y%m%d%H%M.%S 2>/dev/null \
    || TZ=UTC date -d "@$SOURCE_DATE_EPOCH" +%Y%m%d%H%M.%S)
find "$stage" -exec touch -t "$stamp" {} +

out=$dist/$target/OpenRB-338-$version-$arch-setup.exe
rm -f "$out"
# shellcheck disable=SC2086  # the flags are words on purpose
# -WX (the default of MAKENSIS_FLAGS; set it empty to drop it): a warning fails the build. The time inside the file comes from SOURCE_DATE_EPOCH.
"$makensis" -V2 ${MAKENSIS_FLAGS--WX} "-DICON=$root/assets/app_icon.ico" \
    "-DVERSION=$version" "-DARCH=$arch" "-DBINDIR=$stage" "-DSTAGEDIR=$stage" \
    ${notices:+"-DNOTICES=$stage/THIRD-PARTY-NOTICES.txt"} \
    "-DPATHPS1=$stage/path.ps1" "-DPLUGINDIR=$stage/plugins" "-DOUTFILE=$out" \
    "$root/scripts/installer/openrb.nsi"
rm -rf "$stage"
ls -l "$out"
