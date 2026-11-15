---
id: TASK-193
title: ASIO output on Windows
status: To Do
assignee:
  - '@agent-worker'
created_date: '2026-11-15T12:45:00Z'
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
priority: low
type: feature
ordinal: 206000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
GitHub issue #12 (2026-11-15) asks for ASIO drivers (e.g. ASIO4ALL) on Windows. The original ReBirth 2.01 had no ASIO: Rebirth.exe knows only MME (`MME_CHANNEL`, 1024-frame buffers) and DirectSound (`CDirectSoundChannel`, `dsound.dll`), with no "ASIO" string in either binary (docs/export_state.md 2.1). So ASIO is an addition of orb's.

cpal has an `asio` feature (Windows only). Building it needs Steinberg's ASIO SDK (`CPAL_ASIO_DIR`) and LLVM/clang, because `asio-sys` generates its bindings with bindgen at build time. Whether and how the SDK may be used and shipped is task-192's decision; this task follows it.

Scope:
- the feature in the Windows build only, so macOS and Linux builds and cross builds are unaffected; the Windows packages come from native Windows runners (cross builds with cargo-xwin probably cannot run bindgen against the SDK: check and document);
- CI (`ci.yml`) and the release (`release.yml`, `scripts/package.sh`) install the SDK and LLVM on the Windows runners as task-192 decides;
- ASIO appears as a host in the Audio Output window (task-191) and is never picked by Automatic (task-190), since ASIO drivers are often exclusive;
- ASIO drivers set their buffer in their own control panel and may run at another rate than 44.1 kHz (orb resamples then, as for any device); the window shows what the driver granted. Many drivers allow one application at a time: a driver that is busy or fails to open shows the error box and falls back to Automatic.

Testing has no Windows machine on the dev side: the Windows test machine (docs/test_machine.md, doc-001 permits) with ASIO4ALL is the place for the manual check.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 The Windows build has cpal's asio feature as task-192 decided; macOS, Linux and their cross builds are unchanged and still pass CI
- [ ] #2 CI and the release workflow build the Windows packages with ASIO; docs/building.md says what to install (SDK, LLVM, CPAL_ASIO_DIR) and whether cross builds can include ASIO
- [ ] #3 ASIO is listed as a host in the Audio Output window, is never chosen by Automatic, and a busy or failing driver shows the error box and falls back to Automatic
- [ ] #4 A manual check on the Windows test machine with ASIO4ALL (play a song, switch to ASIO and back, the window's facts) is recorded in the notes
<!-- AC:END -->
