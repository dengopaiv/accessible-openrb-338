#!/usr/bin/env bash
# Builds dist/<target>/OpenRB-338-Plugins-<YYYY.MMDD.HHMM>-<target>.pkg for one macOS target
# (task-238): the installer of the VST3 plugin, a release asset of its own next to the
# .dmg (the user's decision, 2026-11-17).
#
#   scripts/package-pkg.sh aarch64-apple-darwin|x86_64-apple-darwin
#
# Takes the bundles scripts/build.sh made in dist/<target>/plugins/ (signed ad hoc by the
# bundler, checked here); release.yml downloads them from the macOS package job
# (`plugins-<target>`). It builds nothing else. Environment as scripts/package-dmg.sh.
#
# Why a pkg and not a link in the .dmg: macOS does not create /Library/Audio/Plug-Ins/VST3, so a
# link to it is broken on a fresh Mac and a drop onto a link does not work. A pkg creates its
# install path.
# One component package, joined by productbuild. The component plist is an empty array: pkgbuild
# then records no bundle in the package, so the Installer does not look for "the same bundle"
# elsewhere by identifier (and it would also find a copy in a project folder) and just writes the
# files at the fixed install path. The preinstall script (scripts/pkg/preinstall) removes the old
# bundle first, so an upgrade leaves no stale file inside it, and removes the OpenRB-338.clap of
# the earlier releases (CLAP was dropped, task-278) and forgets that release's CLAP receipt.
# The welcome page of the Installer (productbuild --resources) says how to open the unsigned
# package and how to remove the plugin, as nothing else is next to the file.
set -euo pipefail

root=$(cd "$(dirname "$0")/.." && pwd)
dist=${DIST:-$root/dist}
target=${1:?usage: scripts/package-pkg.sh aarch64-apple-darwin|x86_64-apple-darwin}
case $target in
    aarch64-apple-darwin) arch=arm64 hostarch=arm64 ;;
    x86_64-apple-darwin) arch=x86_64 hostarch=x86_64,arm64 ;;
    *) echo "not a macOS target: $target" >&2; exit 1 ;;
esac
export SOURCE_DATE_EPOCH=${SOURCE_DATE_EPOCH:-$(date +%s)}
version=$("$root/scripts/version.sh")
out=$dist/$target
pkg=$out/OpenRB-338-Plugins-$version-$target.pkg

pkgwork=$out/pkgwork
rm -rf "$pkgwork"
mkdir -p "$pkgwork/pkgs" "$pkgwork/resources"
p=OpenRB-338.vst3
[ -e "$out/plugins/$p" ] || { echo "missing $out/plugins/$p" >&2; exit 1; }
codesign --verify --strict "$out/plugins/$p"
lipo -archs "$out/plugins/$p/Contents/MacOS/OpenRB-338" | grep -qx "$arch" \
    || { echo "$p is not $arch only" >&2; exit 1; }
mkdir -p "$pkgwork/root" "$pkgwork/scripts"
cp -R "$out/plugins/$p" "$pkgwork/root/$p"
# no extended attributes (a quarantine flag, com.apple.provenance) in the payload: pkgbuild
# would store each as a ._ file
xattr -cr "$pkgwork/root/$p"
codesign --verify --strict "$pkgwork/root/$p"
# the preinstall script (scripts/pkg/preinstall, tested by scripts/test-pkg-scripts.sh): removes
# the old VST3 bundle and the OpenRB-338.clap of the earlier releases (only that bundle)
cp "$root/scripts/pkg/preinstall" "$pkgwork/scripts/preinstall"
chmod 755 "$pkgwork/scripts/preinstall"
cat > "$pkgwork/component.plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<array/>
</plist>
EOF
pkgbuild --quiet --root "$pkgwork/root" \
    --component-plist "$pkgwork/component.plist" \
    --install-location "/Library/Audio/Plug-Ins/VST3" \
    --scripts "$pkgwork/scripts" \
    --identifier "net.emphased.openrb338.plugin.vst3" --version "$version" \
    --ownership recommended "$pkgwork/pkgs/vst3.pkg"

cat > "$pkgwork/resources/welcome.txt" <<EOF
OpenRB-338 plugins $version

The VST 3 instrument of OpenRB-338, an engine that reimplements Propellerhead ReBirth
RB-338 2.01.

This installs, for all users (it asks for an admin password once):
  OpenRB-338.vst3 into /Library/Audio/Plug-Ins/VST3
The folder is created if it does not exist. Installing again replaces the old bundle. Rescan
plugins in your host afterwards.

CLAP was dropped. This installer deletes the OpenRB-338.clap of an earlier release from
/Library/Audio/Plug-Ins/CLAP (only that bundle). If you put a copy in your own
~/Library/Audio/Plug-Ins/CLAP, delete it by hand. A project saved with the CLAP plugin needs the
plugin replaced by its VST3 by hand in the host.

This installer is not signed or notarized, so macOS blocks it the first time you open it:
  - macOS 15 (Sequoia) and newer: open it once (it is refused), then System Settings >
    Privacy & Security, scroll to Security, click "Open Anyway" and confirm.
  - macOS 14 and older: right-click the installer, choose Open, then Open.
The plugin is signed ad hoc only. It is not quarantined, so they need no xattr command
(checked on macOS 15). If a host still refuses it: sudo xattr -dr com.apple.quarantine on the
bundle.

To remove the plugin, delete /Library/Audio/Plug-Ins/VST3/OpenRB-338.vst3 (a pkg has no
uninstaller).
EOF

cat > "$pkgwork/distribution.xml" <<EOF
<?xml version="1.0" encoding="utf-8"?>
<installer-gui-script minSpecVersion="2">
    <title>OpenRB-338 Plugins</title>
    <welcome file="welcome.txt" mime-type="text/plain"/>
    <options customize="never" require-scripts="false" hostArchitectures="$hostarch"/>
    <domains enable_anywhere="false" enable_currentUserHome="false" enable_localSystem="true"/>
    <choices-outline>
        <line choice="vst3"/>
    </choices-outline>
    <choice id="vst3" title="VST3 plugin" start_selected="true" start_enabled="true" start_visible="true" description="OpenRB-338.vst3, installed in /Library/Audio/Plug-Ins/VST3 for all users.">
        <pkg-ref id="net.emphased.openrb338.plugin.vst3"/>
    </choice>
    <pkg-ref id="net.emphased.openrb338.plugin.vst3" version="$version" onConclusion="none">vst3.pkg</pkg-ref>
</installer-gui-script>
EOF
rm -f "$pkg"
productbuild --quiet --distribution "$pkgwork/distribution.xml" \
    --resources "$pkgwork/resources" --package-path "$pkgwork/pkgs" "$pkg"
rm -rf "$pkgwork"
ls -l "$pkg"
