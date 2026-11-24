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
# the plugin bundle (task-204; scripts/build.sh made it)
p=OpenRB-338.vst3
[ -e "$dist/$target/plugins/$p" ] || { echo "missing $dist/$target/plugins/$p" >&2; exit 1; }
case $target in
    *-windows-*)
        # portable by default (task-289): the app, the plugin as the single file (the binary
        # of the bundle, which hosts on Windows accept as it is) and the empty settings file
        # in one folder
        cp "$dist/$target/plugins/$p"/Contents/*-win/"$p" "$stage/$p"
        : > "$stage/OpenRB-338.toml"
        # the ASIO SDK's license and notices (task-193): ASIO is in the x86_64 build only
        case $target in
            x86_64-*) "$root/scripts/asio-notices.sh" "$stage/THIRD-PARTY-NOTICES.txt" ;;
        esac
        ;;
    *)
        mkdir -p "$stage/plugins"
        cp -R "$dist/$target/plugins/$p" "$stage/plugins/"
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
    *-windows-*) run="OpenRB-338.exe (the app) and orb.exe (the command line)" ;;
    *) run="OpenRB-338 (the app) and orb (the command line)" ;;
esac
cat > "$stage/README.txt" <<EOF
OpenRB-338 $version ($target)

A reimplementation of the audio engine of Propellerhead ReBirth RB-338 2.01.

This package holds $run.
Neither needs any file of the original program: the standard drum kit is built in.

  orb render SONG.rbs -o SONG.flac    render a song to FLAC, Ogg Vorbis or WAV
  orb --help                          all commands
  OpenRB-338                          the desktop UI

The binaries are not signed (docs/building.md says what the system shows for that).
Licensed under the GNU General Public License, version 3 only (GPL-3.0-only; see LICENSE).

Plugin (VST 3 instrument)
-------------------------

EOF
case $target in
    *-windows-*) ;;
    *) cat >> "$stage/README.txt" <<'EOF'
plugins/ holds OpenRB-338.vst3. Copy it into a plugin folder, then rescan plugins in the host:
EOF
    ;;
esac
case $target in
    *-linux-*) plugin_dirs='  VST3: ~/.vst3/   (or /usr/lib/vst3)' ;;
    *-windows-*) plugin_dirs='The plugin is the file OpenRB-338.vst3 in this same folder as OpenRB-338.exe. Point the host'"'"'s VST3
plugin path (its custom VST3 folder setting) at this folder and rescan plugins; the plugin is then
portable (see below). Or copy the file into a plugin folder, where it is not portable:

  VST3: %COMMONPROGRAMFILES%\VST3\
  (or %LOCALAPPDATA%\Programs\Common\VST3\ for the current user only)
  The installer (OpenRB-338-*-setup.exe) can do this for you: its "VST3 plugin" component, on by
  default, copies the bundle to the %COMMONPROGRAMFILES% folder above (the installer is for all
  users and asks for administrator rights); the uninstaller removes it.' ;;
    *-apple-darwin) plugin_dirs='  VST3: ~/Library/Audio/Plug-Ins/VST3/
  (or the same under /Library for all users). The bundle is signed ad hoc only; if the host
  refuses it after a download, run in this folder (and again, with sudo, on the copy in
  /Library if you installed it there): xattr -dr com.apple.quarantine OpenRB-338.vst3' ;;
esac
printf '%s\n' "$plugin_dirs" >> "$stage/README.txt"
case $target in
    *-windows-*)
        cat >> "$stage/README.txt" <<'EOF'

Portable mode
-------------
This folder holds an empty file, OpenRB-338.toml, which makes OpenRB-338.exe and the plugin
bundle next to it portable: they keep their settings in that file, and the crash logs and the
plugin's log (turned on by a file plugin.log in logs\) in the logs\ folder here, and write
nothing in your user profile. The app and the plugin share the one file. The file is empty at
first: the first start copies your per-user settings (%APPDATA%\OpenRB-338\OpenRB-338.toml) into
it, so the settings you have come along (from the older ui.toml there when there is no
OpenRB-338.toml yet); the per-user files are not changed. The plugin is portable
only when the host loads it from this folder (the host's custom VST3 path); a copy in a plugin
folder uses the per-user settings.

  To turn portable mode off: move OpenRB-338.toml to %APPDATA%\OpenRB-338\, replacing the
  OpenRB-338.toml there. To turn it on again: create an empty OpenRB-338.toml next to
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
