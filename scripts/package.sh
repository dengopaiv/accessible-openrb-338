#!/usr/bin/env bash
# Builds one target (scripts/build.sh) and packs dist/<target>/bin/ into
# dist/<target>/OpenRB-338-<YYYY.MMDD.HHMM>-<target>.tar.gz (Linux, macOS) or .zip (Windows;
# the version is the UTC build time, scripts/version.sh), with
# the license and a short readme (task-108).
#
#   scripts/package.sh TARGET
#
# SKIP_BUILD=1 packs the binaries already in dist/<target>/bin/. The packages
# of task-110 (dmg) and task-112 (AppImage) start from the same
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
case $target in
    *-windows-*)
        # portable by default (task-289): the app and the portable marker in one folder; no
        # OpenRB-338.toml, so unzipping over an old folder keeps its settings (task-295)
        printf '%s\n' 'Portable mode is on while this file exists (see README.txt). Its content is ignored.' \
            > "$stage/OpenRB-338.portable"
        # the ASIO SDK's license and notices (task-193): ASIO is in the x86_64 build only
        case $target in
            x86_64-*) "$root/scripts/asio-notices.sh" "$stage/THIRD-PARTY-NOTICES.txt" ;;
        esac
        ;;
esac

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
        # the fork's Windows zip is portable (the user's decision, task-222): since the merge of
        # upstream f83c54f by upstream's marker OpenRB-338.portable above, which replaced the
        # fork's portable.txt (the user's choice, 2026-11-26)
        # the fork's changes per release as plain text with CRLF (task-228): no heading marks,
        # bold, backticks or link targets, which a screen reader would read out
        sed -E -e 's/^#+ +//' -e 's/\*\*//g' -e 's/`//g' -e 's/\[([^]]*)\]\([^)]*\)/\1/g' \
            "$root/CHANGELOG.md" | awk '{ printf "%s\r\n", $0 }' >"$stage/CHANGELOG.txt"
        portable='This copy is portable: unpack the folder anywhere, a USB stick too, and run
OpenRB-338.exe. The settings and the logs are kept in this folder and nothing is written to
AppData (Portable mode, below, says more).
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
EOF
case $target in
    *-windows-*)
        cat >> "$stage/README.txt" <<'EOF'

Portable mode
-------------
The file OpenRB-338.portable in this folder makes OpenRB-338.exe portable while it exists (its
content is ignored): it keeps its settings in OpenRB-338.toml here, and the crash logs in the
logs\ folder here, and writes nothing in your user profile. It is not in the zip: the first start
copies your per-user settings
(%APPDATA%\OpenRB-338\OpenRB-338.toml, or the older ui.toml there when there is no
OpenRB-338.toml yet) into it, so the settings you have come along; the per-user files are not
changed.

The songs and mods folders you choose inside this folder are remembered relative to it, so they
still work when a USB stick gets another drive letter; folders elsewhere are remembered as full
paths. A folder unpacked from an older release of this fork (one with portable.txt and the
folder data) stays portable: its first start copies the settings in data\ui.toml into
OpenRB-338.toml. After that portable.txt and data are no longer used, and you may delete them
once OpenRB-338.portable is in the folder (unzipping this release over it puts it there).

  To upgrade: unzip the new version over this folder. OpenRB-338.toml is not in the zip, so your
  settings stay.
  To turn portable mode off: delete OpenRB-338.portable, and move OpenRB-338.toml to
  %APPDATA%\OpenRB-338\, replacing the OpenRB-338.toml there, to keep the settings. Without
  the marker the program uses the per-user settings and ignores a local OpenRB-338.toml.
  To turn it on again: create a file OpenRB-338.portable (it may be empty) next to
  OpenRB-338.exe.

If the folder cannot be written (for instance under Program Files), the app runs with the
settings it read and says once that they cannot be saved; move the folder somewhere you can
write, or turn portable mode off as above.
EOF
        ;;
esac
case $target in
    x86_64-pc-windows-msvc)
        cat >> "$stage/README.txt" <<'EOF'

ASIO
----
OpenRB-338.exe can play through ASIO drivers: Options > Audio Output..., Driver: ASIO. The
default stays WASAPI; ASIO is never chosen by itself. An ASIO driver sets its buffer size in its
own control panel, and many drivers allow one program at a time. THIRD-PARTY-NOTICES.txt holds
the license of the ASIO SDK.
ASIO is a trademark and software of Steinberg Media Technologies GmbH.
EOF
        ;;
esac
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
