#!/usr/bin/env bash
# Builds dist/<target>/OpenRB-338-Plugins-<YYYY.MMDD.HHMM>-<target>.pkg for one macOS target
# (task-238): the installer of the VST3 and CLAP plugins, a release asset of its own next to the
# .dmg (the user's decision, 2026-11-17).
#
#   scripts/package-pkg.sh aarch64-apple-darwin|x86_64-apple-darwin
#
# Takes the bundles scripts/build.sh made in dist/<target>/plugins/ (signed ad hoc by the
# bundler, checked here); release.yml downloads them from the macOS package job
# (`plugins-<target>`). It builds nothing else. Environment as scripts/package-dmg.sh.
#
# Why a pkg and not links in the .dmg: macOS creates neither /Library/Audio/Plug-Ins/VST3 nor
# /CLAP, so a link to them is broken on a fresh Mac and a drop onto a link does not work. A pkg
# creates its install path.
# Two component packages, one choice each (both selected by default, deselectable in
# Customize), joined by productbuild. The component plist is an empty array: pkgbuild then
# records no bundle in the package, so the Installer does not look for "the same bundle"
# elsewhere by identifier (both plugins have the same CFBundleIdentifier, and it would also
# find a copy in a project folder) and just writes the files at the fixed install path. The
# preinstall script removes the old bundle first, so an upgrade leaves no stale file inside it.
# The welcome page of the Installer (productbuild --resources) says how to open the unsigned
# package and how to remove the plugins, as nothing else is next to the file.
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
for spec in vst3:VST3 clap:CLAP; do
    ext=${spec%%:*}
    folder=${spec#*:}
    p=OpenRB-338.$ext
    [ -e "$out/plugins/$p" ] || { echo "missing $out/plugins/$p" >&2; exit 1; }
    codesign --verify --strict "$out/plugins/$p"
    lipo -archs "$out/plugins/$p/Contents/MacOS/OpenRB-338" | grep -qx "$arch" \
        || { echo "$p is not $arch only" >&2; exit 1; }
    mkdir -p "$pkgwork/root-$ext" "$pkgwork/scripts-$ext"
    cp -R "$out/plugins/$p" "$pkgwork/root-$ext/$p"
    # no extended attributes (a quarantine flag, com.apple.provenance) in the payload: pkgbuild
    # would store each as a ._ file
    xattr -cr "$pkgwork/root-$ext/$p"
    codesign --verify --strict "$pkgwork/root-$ext/$p"
    cat > "$pkgwork/scripts-$ext/preinstall" <<EOF
#!/bin/sh
# \$3 is the target volume ("/" for the startup disk)
rm -rf "\$3/Library/Audio/Plug-Ins/$folder/$p"
exit 0
EOF
    chmod 755 "$pkgwork/scripts-$ext/preinstall"
    cat > "$pkgwork/component-$ext.plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<array/>
</plist>
EOF
    pkgbuild --quiet --root "$pkgwork/root-$ext" \
        --component-plist "$pkgwork/component-$ext.plist" \
        --install-location "/Library/Audio/Plug-Ins/$folder" \
        --scripts "$pkgwork/scripts-$ext" \
        --identifier "net.emphased.openrb338.plugin.$ext" --version "$version" \
        --ownership recommended "$pkgwork/pkgs/$ext.pkg"
done

cat > "$pkgwork/resources/welcome.txt" <<EOF
OpenRB-338 plugins $version

The VST 3 and CLAP instrument of OpenRB-338, an engine that reimplements Propellerhead ReBirth
RB-338 2.01.

This installs, for all users (it asks for an admin password once):
  OpenRB-338.vst3 into /Library/Audio/Plug-Ins/VST3
  OpenRB-338.clap into /Library/Audio/Plug-Ins/CLAP
The folders are created if they do not exist. Both plugins are selected; Customize lets you
pick one. Installing again replaces the old bundles. Rescan plugins in your host afterwards.

This installer is not signed or notarized, so macOS blocks it the first time you open it:
  - macOS 15 (Sequoia) and newer: open it once (it is refused), then System Settings >
    Privacy & Security, scroll to Security, click "Open Anyway" and confirm.
  - macOS 14 and older: right-click the installer, choose Open, then Open.
The plugins are signed ad hoc only. They are not quarantined, so they need no xattr command
(checked on macOS 15). If a host still refuses them: sudo xattr -dr com.apple.quarantine on the
two bundles.

To remove the plugins, delete /Library/Audio/Plug-Ins/VST3/OpenRB-338.vst3 and
/Library/Audio/Plug-Ins/CLAP/OpenRB-338.clap (a pkg has no uninstaller).
EOF

cat > "$pkgwork/distribution.xml" <<EOF
<?xml version="1.0" encoding="utf-8"?>
<installer-gui-script minSpecVersion="2">
    <title>OpenRB-338 Plugins</title>
    <welcome file="welcome.txt" mime-type="text/plain"/>
    <options customize="allow" require-scripts="false" hostArchitectures="$hostarch"/>
    <domains enable_anywhere="false" enable_currentUserHome="false" enable_localSystem="true"/>
    <choices-outline>
        <line choice="vst3"/>
        <line choice="clap"/>
    </choices-outline>
    <choice id="vst3" title="VST3 plugin" start_selected="true" start_enabled="true" start_visible="true" description="OpenRB-338.vst3, installed in /Library/Audio/Plug-Ins/VST3 for all users.">
        <pkg-ref id="net.emphased.openrb338.plugin.vst3"/>
    </choice>
    <choice id="clap" title="CLAP plugin" start_selected="true" start_enabled="true" start_visible="true" description="OpenRB-338.clap, installed in /Library/Audio/Plug-Ins/CLAP for all users.">
        <pkg-ref id="net.emphased.openrb338.plugin.clap"/>
    </choice>
    <pkg-ref id="net.emphased.openrb338.plugin.vst3" version="$version" onConclusion="none">vst3.pkg</pkg-ref>
    <pkg-ref id="net.emphased.openrb338.plugin.clap" version="$version" onConclusion="none">clap.pkg</pkg-ref>
</installer-gui-script>
EOF
rm -f "$pkg"
productbuild --quiet --distribution "$pkgwork/distribution.xml" \
    --resources "$pkgwork/resources" --package-path "$pkgwork/pkgs" "$pkg"
rm -rf "$pkgwork"
ls -l "$pkg"
