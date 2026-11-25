#!/bin/bash
# One fresh ReBirth 2.01 session on the Windows test machine (task-306): start ReBirth, open
# SONG (a song of this folder), run Edit > Initialize Loop or Initialize Song from Pattern Mode
# (the song's alert answered with Enter, Initialize), export the loop as OUT.wav, stop ReBirth
# and fetch the WAV into LOCAL_DIR. Prints the open and export times (Unix seconds), from
# which an idle count is seeded: N about (export - open) * 44100 / 64.
#
# Usage: tests/songs/edit_init/export.sh SONG loop|song OUT LOCAL_DIR
#   e.g. tests/songs/edit_init/export.sh init_zero.rbs song zero_song /tmp/edit_init
#
# Needs: rcs_client.py (backlog doc-001; RCS names it, default
# ~/dev/dcre/common/tools/rcs_client.py) and all the permits of the semaphore redc-win-run,
# taken by the caller first (doc-001):
#   locky acquire redc-win-run N --owner BRANCH --acquire-timeout 3600 --lease-timeout 1800
#   ... this script, once per export ...
#   locky release redc-win-run all --owner BRANCH
# It stops only the ReBirth it started (by the PID tasklist gives right after the start; no
# other Rebirth.exe may run). Songs are uploaded to and the WAV written in C:\re\rb\t306
# (rcs path rb/t306), outside the installation. Menu places are relative to the main window's
# client area, which moves between starts (docs/test_machine.md section 3).
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
song=$1 cmd=$2 out=$3 local_dir=$4
rcs_py=${RCS:-$HOME/dev/dcre/common/tools/rcs_client.py}
R() { /usr/bin/python3 "$rcs_py" "$@"; }
case $cmd in
  loop) y=132 ;;
  song) y=154 ;;
  *) echo "the command is loop or song" >&2; exit 2 ;;
esac
mkdir -p "$local_dir"
R put "$here/$song" rb/t306/ >/dev/null
R exec cmd.exe /c start "" /D 'C:\ReBirth RB-338' 'C:\ReBirth RB-338\Rebirth.exe' >/dev/null
sleep 6
# the "Failed opening MIDI input" box of every start
R input --window "ReBirth RB-338" key:enter >/dev/null
sleep 1.5
id=$(R exec tasklist /FI 'IMAGENAME eq Rebirth.exe' /FO CSV /NH | grep '"id"' | tr -dc 0-9)
sleep 1.5
pid=$(R output "$id" | grep -i rebirth | cut -d, -f2 | tr -dc 0-9)
[ -n "$pid" ] || { echo "ReBirth did not start" >&2; exit 1; }
echo "pid $pid"
R input --window "ReBirth RB-338" keydown:ctrl key:o keyup:ctrl >/dev/null
sleep 1.5
R input --window "Open Song:" "text:C:\\re\\rb\\t306\\$song" key:enter >/dev/null
echo "opened $(date +%s)"
sleep 2
# Edit, then the item (client coordinates: Initialize Loop 132, Initialize Song 154)
R input --window "ReBirth RB-338" keydown:alt key:e keyup:alt >/dev/null
sleep 1
R input --window "ReBirth RB-338" "move:158,$y" click >/dev/null
sleep 1
if [ "$cmd" = song ]; then
  R input --window "ReBirth RB-338" key:enter >/dev/null
  sleep 1
fi
# File > Export Loop as Audio File... (client (108, 172) with Save enabled)
R input --window "ReBirth RB-338" keydown:alt key:f keyup:alt >/dev/null
sleep 1
R input --window "ReBirth RB-338" move:108,172 click >/dev/null
sleep 1.5
R input --window "Export audio as:" "text:C:\\re\\rb\\t306\\$out.wav" key:enter >/dev/null
echo "exported $(date +%s)"
sleep 8
R exec taskkill /F /PID "$pid" >/dev/null
sleep 2
R get "rb/t306/$out.wav" "$local_dir/" >/dev/null
shasum -a 256 "$local_dir/$out.wav"
