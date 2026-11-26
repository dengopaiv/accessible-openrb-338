#!/usr/bin/env bash
# Makes LLVM's libclang available to bindgen on a Windows x86_64 runner (task-193): asio-sys, which
# cpal's `asio` feature builds, generates its bindings at build time. The runner image has LLVM
# in C:\Program Files\LLVM; when it does not, Chocolatey installs it. Sets LIBCLANG_PATH for the
# steps after it (GITHUB_ENV).
set -euo pipefail

dir='C:\Program Files\LLVM\bin'
if [ ! -f 'C:/Program Files/LLVM/bin/libclang.dll' ]; then
    choco install llvm --no-progress -y
fi
[ -f 'C:/Program Files/LLVM/bin/libclang.dll' ] || {
    echo "libclang.dll is missing in C:/Program Files/LLVM/bin" >&2
    exit 1
}
echo "LIBCLANG_PATH=$dir" >> "${GITHUB_ENV:?GITHUB_ENV is not set}"
