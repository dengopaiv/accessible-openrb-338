---
id: TASK-298
title: >-
  Next release run: verify the Windows x86_64 ASIO build (LLVM on the release
  runner)
status: To Do
assignee:
  - '@pingw33n'
created_date: '2026-11-24T19:53:00Z'
labels:
  - tools
  - tests
milestone: m-5
dependencies:
  - TASK-193
references:
  - .github/workflows/release.yml
  - scripts/install-llvm-windows.sh
priority: medium
type: task
ordinal: 271700
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
task-193 added ASIO to the x86_64 Windows build. The build needs LLVM (bindgen's libclang, for asio-sys) and the vendored ASIO SDK subset in third_party/asiosdk. release.yml now does this for the x86_64-pc-windows-msvc package job only (the windows-11-arm job is unchanged: the asio feature is x86_64 only): before the build it runs scripts/install-llvm-windows.sh, which uses the runner's LLVM (C:\Program Files\LLVM\bin) or installs it with Chocolatey, and sets LIBCLANG_PATH in GITHUB_ENV; the build finds the SDK through CPAL_ASIO_DIR (.cargo/config.toml [env], third_party/asiosdk, no download). scripts/package.sh and package-installer.sh put THIRD-PARTY-NOTICES.txt (scripts/asio-notices.sh) in the x86_64 zip and the x64 installer. ci.yml runs the same install step on the Windows x86_64 test job; its first run after the merge is watched by the supervisor and is not part of this task. A release run cannot be tried locally (the zip was built with cargo-xwin and checked on the Windows test machine), so the release job itself is the check.

What to look at in the next release run: the package (x86_64-pc-windows-msvc) job is green (the LLVM step, then the build), the zip OpenRB-338-<version>-x86_64-pc-windows-msvc.zip holds THIRD-PARTY-NOTICES.txt, and the program in it lists ASIO as a driver in Options > Audio Output (the installer's x64 setup.exe also installs THIRD-PARTY-NOTICES.txt). If the LLVM step fails (no libclang.dll, or bindgen cannot find the MSVC headers), fix scripts/install-llvm-windows.sh or the job and file a bug.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 The release's x86_64 Windows package job builds with ASIO (LLVM installed by scripts/install-llvm-windows.sh, the job green), and the zip carries THIRD-PARTY-NOTICES.txt
<!-- AC:END -->
