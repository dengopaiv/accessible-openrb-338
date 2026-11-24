#!/usr/bin/env bash
# Writes the third-party notices of the x86_64 Windows packages, which carry ASIO (task-193): the
# trademark line, the ASIO SDK's LICENSE.txt (the GPL Version 3 option is the one orb uses) and
# the BSD notice of the SDK's host helpers (host/asiodrivers.*, host/pc/asiolist.*,
# host/ginclude.h), which is the header comment of host/ginclude.h.
#
#   scripts/asio-notices.sh OUT
set -euo pipefail

root=$(cd "$(dirname "$0")/.." && pwd)
out=${1:?usage: scripts/asio-notices.sh OUT}
sdk=$root/third_party/asiosdk

{
    echo "Third-party notices (the Windows x86_64 package)"
    echo "================================================"
    echo
    echo "This package can play through ASIO drivers, using the ASIO SDK 2.3.4 of Steinberg Media"
    echo "Technologies GmbH (the files in third_party/asiosdk of the source, unmodified), under the"
    echo "terms of the GNU General Public License Version 3, which also covers OpenRB-338 itself"
    echo "(see LICENSE). Its host helper files are under the BSD license quoted in part 2."
    echo
    echo "ASIO is a trademark and software of Steinberg Media Technologies GmbH."
    echo
    echo
    echo "1. ASIO SDK: LICENSE.txt"
    echo "------------------------"
    echo
    tr -d '\r' < "$sdk/LICENSE.txt"
    echo
    echo
    echo "2. ASIO SDK host helpers: BSD 3-clause license"
    echo "----------------------------------------------"
    echo
    # the license part of the header comment of host/ginclude.h, without the comment marks
    tr -d '\r' < "$sdk/host/ginclude.h" \
        | sed -n '/^\/\/ LICENSE/,/^$/p' \
        | sed -e 's|^// \{0,1\}||' -e '/^---/d' -e 's|^//$||' -e 's| *$||'
} > "$out"
