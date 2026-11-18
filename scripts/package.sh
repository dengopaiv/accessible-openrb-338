#!/usr/bin/env bash
# Builds one target (scripts/build.sh) and packs dist/<target>/bin/ into
# dist/<target>/OpenRB-338-<YYYY.MMDD.HHMM>-<target>.tar.gz (Linux, macOS) or .zip (Windows;
# the version is the UTC build time, scripts/version.sh), with
# the license and a short readme (task-108).
#
#   scripts/package.sh TARGET
#
# SKIP_BUILD=1 packs the binaries already in dist/<target>/bin/. The installers and bundles
# of task-110 (dmg), task-111 (Windows installer) and task-112 (AppImage) start from the same
# dist/<target>/bin/ and the same version (`scripts/version.sh`).
set -euo pipefail

root=$(cd "$(dirname "$0")/.." && pwd)
target=${1:?usage: scripts/package.sh TARGET}
dist=${DIST:-$root/dist}
# one build time for the name, the binaries (About box) and the archive
export SOURCE_DATE_EPOCH=${SOURCE_DATE_EPOCH:-$(date +%s)}
version=$("$root/scripts/version.sh")
name=OpenRB-338-$version-$target

[ -n "${SKIP_BUILD:-}" ] || "$root/scripts/build.sh" "$target"

stage=$dist/$target/stage/$name
rm -rf "$dist/$target/stage"
mkdir -p "$stage"
cp "$dist/$target/bin/"* "$stage/"
cp "$root/LICENSE" "$stage/LICENSE"
# the plugin bundles (task-204; scripts/build.sh made them): the folder holds both
for p in OpenRB-338.vst3 OpenRB-338.clap; do
    [ -e "$dist/$target/plugins/$p" ] || { echo "missing $dist/$target/plugins/$p" >&2; exit 1; }
done
mkdir -p "$stage/plugins"
cp -R "$dist/$target/plugins/OpenRB-338.vst3" "$dist/$target/plugins/OpenRB-338.clap" \
    "$stage/plugins/"

case $target in
    *-linux-*)
        # the menu entry (task-178): the icon, and the AppImage's desktop entry (the shared
        # scripts/OpenRB-338.desktop.in) with @DIR@ for the install folder, which the user
        # fills in; Exec and Icon must be absolute
        cp "$root/assets/ui/app_icon.svg" "$stage/OpenRB-338.svg"
        sed -e 's|@EXEC@|"@DIR@/OpenRB-338"|' -e 's|@ICON@|@DIR@/OpenRB-338.svg|' \
            "$root/scripts/OpenRB-338.desktop.in" > "$stage/OpenRB-338.desktop"
        ;;
esac

case $target in
    *-windows-*)
        run="OpenRB-338.exe (the app) and orb.exe (the command line)"
        unsigned='The programs are not signed, so Windows SmartScreen may say "Windows protected
your PC" when one of them first starts: choose "More info", then "Run anyway".'
        # the fork's Windows zip is portable (task-223; the user's decision, task-222)
        cp "$root/scripts/portable.txt" "$stage/portable.txt"
        # the fork's changes per release as plain text with CRLF (task-228): no heading marks,
        # bold, backticks or link targets, which a screen reader would read out
        sed -E -e 's/^#+ +//' -e 's/\*\*//g' -e 's/`//g' -e 's/\[([^]]*)\]\([^)]*\)/\1/g' \
            "$root/CHANGELOG.md" | awk '{ printf "%s\r\n", $0 }' >"$stage/CHANGELOG.txt"
        portable='This copy is portable: unpack the folder anywhere, a USB stick too, and run
OpenRB-338.exe. While portable.txt lies next to it, the settings and the logs are kept in the
folder data beside it and nothing is written to AppData (portable.txt says more).
CHANGELOG.txt lists what changed in each release of the fork.'
        ;;
    *)
        run="OpenRB-338 (the app) and orb (the command line)"
        unsigned="The programs are not signed, so the system may warn before the first start."
        portable=""
        ;;
esac
cat > "$stage/README.txt" <<EOF
OpenRB-338 $version ($target)

A reimplementation of the audio engine of Propellerhead ReBirth RB-338 2.01.

This package holds $run.
Neither needs any file of the original program: the standard drum kit is built in.

  orb render SONG.rbs -o SONG.flac    render a song to FLAC, Ogg Vorbis or WAV
  orb --help                          all commands
  OpenRB-338                          the desktop UI
${portable:+
$portable
}
$(cat "$root/scripts/fork-readme.txt")

$unsigned
Licensed under the GNU General Public License, version 3 only (GPL-3.0-only; see LICENSE).

Plugin (VST 3 and CLAP instrument)
----------------------------------

plugins/ holds OpenRB-338.vst3 and OpenRB-338.clap. Copy the one your host loads (or both)
into a plugin folder, then rescan plugins in the host:
EOF
case $target in
    *-linux-*) plugin_dirs='  VST3: ~/.vst3/           CLAP: ~/.clap/        (or /usr/lib/vst3, /usr/lib/clap)' ;;
    *-windows-*) plugin_dirs='  VST3: %COMMONPROGRAMFILES%\VST3\    CLAP: %COMMONPROGRAMFILES%\CLAP\
  (or %LOCALAPPDATA%\Programs\Common\VST3\ and \CLAP\ for the current user only)
  The installer (OpenRB-338-*-setup.exe) can do this for you: its "VST3 plugin" and "CLAP plugin"
  components, on by default, copy the bundles to the %COMMONPROGRAMFILES% folders above (the
  installer is for all users and asks for administrator rights); the uninstaller removes them.' ;;
    *-apple-darwin) plugin_dirs='  VST3: ~/Library/Audio/Plug-Ins/VST3/    CLAP: ~/Library/Audio/Plug-Ins/CLAP/
  (or the same under /Library for all users). The bundles are signed ad hoc only; if the host
  refuses them after a download, run in this folder (and again, with sudo, on the copies in
  /Library if you installed them there): xattr -dr com.apple.quarantine OpenRB-338.vst3 OpenRB-338.clap' ;;
esac
printf '%s\n' "$plugin_dirs" >> "$stage/README.txt"
case $target in
    *-linux-*)
        cat >> "$stage/README.txt" <<'EOF'

Menu entry (optional)
---------------------
OpenRB-338.desktop and OpenRB-338.svg make OpenRB-338 appear in the application menu with its
icon. A desktop file needs absolute paths, so OpenRB-338.desktop holds @DIR@ where the folder
goes. In this unpacked folder, run:

  mkdir -p ~/.local/share/applications && sed "s|@DIR@|$PWD|g" OpenRB-338.desktop > ~/.local/share/applications/OpenRB-338.desktop.new && mv ~/.local/share/applications/OpenRB-338.desktop.new ~/.local/share/applications/OpenRB-338.desktop

The line works in bash, zsh and fish. The file is written aside and moved into place, so a
failure leaves an existing entry alone (and a stray OpenRB-338.desktop.new, which you can delete).
The folder's path must not contain | & \ " $ ` or %; spaces are fine.

Keep the file name OpenRB-338.desktop: the desktop matches the running window (and its icon) to
the file of that name. If you move or rename this folder, run the line again. To remove the
entry:

  rm ~/.local/share/applications/OpenRB-338.desktop
EOF
        ;;
esac

# owner, group and (with GNU tar) mtime and order are fixed; bsdtar keeps the file mtimes
epoch=${SOURCE_DATE_EPOCH:-$(date +%s)}
case $target in
    *-windows-*)
        archive=$dist/$target/$name.zip
        rm -f "$archive"
        if command -v zip >/dev/null; then
            (cd "$dist/$target/stage" && zip -qr -X "$archive" "$name")
        elif command -v 7z >/dev/null; then
            (cd "$dist/$target/stage" && 7z a -tzip -bso0 "$archive" "$name")
        else
            echo "no zip or 7z on PATH (brew install zip; or install 7-Zip)" >&2
            exit 1
        fi
        ;;
    *)
        archive=$dist/$target/$name.tar.gz
        if tar --version 2>/dev/null | grep -q GNU; then
            (cd "$dist/$target/stage" && tar --sort=name --owner=0 --group=0 --numeric-owner \
                --mtime="@$epoch" -cf - "$name") | gzip -n > "$archive"
        else
            (cd "$dist/$target/stage" && tar --no-xattrs --uid 0 --gid 0 --numeric-owner \
                -cf - "$name") \
                | gzip -n > "$archive"
        fi
        ;;
esac
rm -rf "$dist/$target/stage"
ls -l "$archive"
