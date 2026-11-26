#!/bin/bash
# The idle-block count N at which orb's render of SAVED (a song the original saved after a
# command, in this folder) equals WAV (the original's export of that song after the same
# command, export.sh): orb compare --idle-blocks N for N = FROM, FROM+STEP, .. TO. Prints the
# first N that is EQUAL, or "no N".
#
# Usage: tests/songs/edit_init/fit.sh SAVED.rbs WAV [FROM TO STEP]   (default 0 30000 16)
#   e.g. tests/songs/edit_init/fit.sh init_loop.after_init_loop.rbs /tmp/edit_init/x.wav
# Needs the release build of orb: cargo build --release -p orb-cli.
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
root=$(cd "$here/../../.." && pwd)
orb=${ORB:-$root/target/release/orb}
saved=$1 wav=$2 from=${3:-0} to=${4:-30000} step=${5:-16}
for ((n = from; n <= to; n += step)); do
  if "$orb" compare "$here/$saved" --reference "$wav" --idle-blocks "$n" >/dev/null 2>&1; then
    echo "$saved N=$n"
    exit 0
  fi
done
echo "$saved no N in $from..$to step $step"
exit 1
