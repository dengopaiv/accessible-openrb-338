---
id: TASK-193
title: ASIO output on Windows
status: To Do
assignee:
  - '@agent-worker'
created_date: '2026-11-15T12:45:00Z'
updated_date: '2026-11-16T07:55:00Z'
labels:
  - player
  - ui
  - docs
milestone: m-5
dependencies:
  - TASK-191
  - TASK-192
references:
  - crates/orb-player/Cargo.toml
  - crates/orb-player/src/output.rs
  - .github/workflows/ci.yml
  - .github/workflows/release.yml
  - scripts/package.sh
  - docs/building.md
  - task-191
  - task-192
  - https://github.com/pingw33n/OpenRB-338/issues/12
  - https://download.steinberg.net/sdk_downloads/ASIO-SDK_2.3.4_2025-10-15.zip
priority: low
type: feature
ordinal: 206000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
GitHub issue #12 (2026-11-15) asks for ASIO drivers (e.g. ASIO4ALL) on Windows. The original ReBirth 2.01 had no ASIO: Rebirth.exe knows only MME (`MME_CHANNEL`, 1024-frame buffers) and DirectSound (`CDirectSoundChannel`, `dsound.dll`), with no "ASIO" string in either binary (docs/export_state.md 2.1). So ASIO is an addition of orb's.

cpal has an `asio` feature (Windows only). Building it needs Steinberg's ASIO SDK (`CPAL_ASIO_DIR`) and LLVM/clang, because `asio-sys` generates its bindings with bindgen at build time.

task-192 decided (2026-11-16) how the SDK is used: ASIO SDK 2.3.4 under its GPLv3 option (host helpers BSD-3-clause), **vendored minimally** in the repository. See task-192's final summary and notes for the version, checksum and license quotes.

Scope:
- **Vendor the SDK subset:** from ASIO-SDK_2.3.4_2025-10-15.zip (https://download.steinberg.net/sdk_downloads/ASIO-SDK_2.3.4_2025-10-15.zip, SHA-256 d5ebf0c20dd2c5f43771fd0c1418f4b361bf52434ee670097cfa6b3a335e2eca), only the files cpal's asio-sys build needs (check its build.rs: likely common/asio.h, asiosys.h, iasiodrv.h, asio.cpp, host/asiodrivers.*, host/ginclude.h, host/pc/asiolist.*), unmodified, in the SDK's own layout under a third-party folder (e.g. `third_party/asiosdk/`), with the SDK's LICENSE.txt and a README giving the version, the zip's URL and SHA-256, and the file list. No driver-side files, PDFs or logo artwork. CPAL_ASIO_DIR points there in the build (no download).
- The feature in the Windows build only, so macOS and Linux builds and cross builds are unaffected; the Windows packages come from native Windows runners (cross builds with cargo-xwin probably cannot run bindgen against the SDK: check and document).
- CI (`ci.yml`) and the release (`release.yml`, `scripts/package.sh`) install LLVM on the Windows runners and use the vendored SDK.
- **Notices:** the Windows package's third-party notices include the SDK's LICENSE.txt and the BSD copyright and disclaimer of the host helpers; wherever the name ASIO appears (README, docs/building.md, the Audio Output window) the line "ASIO is a trademark and software of Steinberg Media Technologies GmbH". No ASIO logo, and "ASIO" never in a product name.
- ASIO appears as a host in the Audio Output window (task-191) and is never picked by Automatic (task-190), since ASIO drivers are often exclusive.
- ASIO drivers set their buffer in their own control panel and may run at another rate than 44.1 kHz (orb resamples then, as for any device); the window shows what the driver granted. Many drivers allow one application at a time: a driver that is busy or fails to open shows the error box and falls back to Automatic.

Testing has no Windows machine on the dev side: the Windows test machine (docs/test_machine.md, doc-001 permits) with ASIO4ALL is the place for the manual check.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 The minimal ASIO SDK 2.3.4 subset is vendored unmodified with its LICENSE.txt and a README (version, URL, SHA-256, file list); the Windows build uses it via CPAL_ASIO_DIR with no download; macOS, Linux and their cross builds are unchanged and still pass CI
- [ ] #2 CI and the release workflow build the Windows packages with ASIO; docs/building.md says what to install (LLVM) and whether cross builds can include ASIO
- [ ] #3 The Windows package's third-party notices carry the SDK license and the host helpers' BSD notice; the trademark line appears wherever ASIO is named
- [ ] #4 ASIO is listed as a host in the Audio Output window, is never chosen by Automatic, and a busy or failing driver shows the error box and falls back to Automatic
- [ ] #5 A manual check on the Windows test machine with ASIO4ALL (play a song, switch to ASIO and back, the window's facts) is recorded in the notes
<!-- AC:END -->
