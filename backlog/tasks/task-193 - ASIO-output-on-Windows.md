---
id: TASK-193
title: ASIO output on Windows
status: Done
assignee:
  - '@agent-worker'
created_date: '2026-11-15T12:45:00Z'
updated_date: '2026-11-24T20:04:00Z'
labels:
  - player
  - ui
  - docs
milestone: m-5
dependencies:
  - TASK-191
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
priority: medium
type: feature
ordinal: 160
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
- **Notices:** the Windows package's third-party notices include the SDK's LICENSE.txt and the BSD copyright and disclaimer of the host helpers; wherever the name ASIO appears (README, docs/building.md; not the Audio Output window, by the user's review 2026-11-24) the line "ASIO is a trademark and software of Steinberg Media Technologies GmbH". No ASIO logo, and "ASIO" never in a product name.
- ASIO appears as a host in the Audio Output window (task-191) and is never picked by Automatic (task-190), since ASIO drivers are often exclusive.
- ASIO drivers set their buffer in their own control panel and may run at another rate than 44.1 kHz (orb resamples then, as for any device); the window shows what the driver granted. Many drivers allow one application at a time: a driver that is busy or fails to open shows the error box and falls back to Automatic.

Testing has no Windows machine on the dev side: the Windows test machine (docs/test_machine.md, doc-001 permits) with ASIO4ALL is the place for the manual check.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 The minimal ASIO SDK 2.3.4 subset is vendored unmodified with its LICENSE.txt and a README (version, URL, SHA-256, file list); the Windows build uses it via CPAL_ASIO_DIR with no download; macOS, Linux and their cross builds are unchanged and still pass CI
- [x] #2 ci.yml and release.yml run an LLVM install step (scripts/install-llvm-windows.sh, shellcheck and actionlint clean) on the x86_64 Windows jobs before the build, and docs/building.md says what to install (LLVM) and that cross builds can include ASIO (the release run itself is checked in task-298, the first ci.yml run confirms the ci.yml step)
- [x] #3 The Windows package's third-party notices carry the SDK license and the host helpers' BSD notice; the trademark line appears wherever ASIO is named (README, docs/building.md, the package README and notices; not the Audio Output window, by the user's review)
- [x] #4 ASIO is listed as a host in the Audio Output window, is never chosen by Automatic, and a busy or failing driver shows the error box and falls back to Automatic
- [x] #5 A manual check on the Windows test machine is recorded in the notes: with FlexASIO, a song plays through ASIO and the window shows the facts, the rate change resamples and a missing ASIO device falls back to System default (my runs; the user heard FlexASIO directly and over RDP); with ASIO4ALL (the user's runs, 2026-11-24) it was silent over the HDMI output, an ASIO4ALL limit, and the exit with its panel open, fixed in round 2, was re-tested
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Vendor the ASIO SDK 2.3.4 files asio-sys compiles (common/asio.*, asiosys.h, iasiodrv.h, host/asiodrivers.*, ginclude.h, host/pc/asiolist.*) plus LICENSE.txt and a README, CRLF kept (-text); CPAL_ASIO_DIR via .cargo/config.toml [env].
2. cpal asio feature for cfg(all(windows, x86_64)) in orb-player only.
3. devices.rs: ASIO-aware enumeration (cpal loads each driver while enumerating, one driver at a time: cache while a stream holds one), first device stands for the default, no default-device watch; output.rs: no buffer request for ASIO.
4. UI: trademark line, buffer row text; Driver Settings button only if asio-sys/cpal expose ASIOControlPanel.
5. scripts/asio-notices.sh, package.sh and the installer ship THIRD-PARTY-NOTICES.txt in the x86_64 Windows packages; LLVM install script in ci.yml and release.yml; docs/building.md and README.
6. Check: cargo test, clippy mac + xwin, build the zip with xwin, run on the test machine with FlexASIO (ASIO4ALL is not installed there).
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
2026-11-24, supervisor: task-190 was merged into task-191, which now has the Driver/Device/Sample rate/Buffer size window (the user's redesign). For ASIO the window replaces the buffer size row with "Set in the driver's control panel" and a "Driver Settings..." button. Check whether cpal or asio-sys exposes ASIOControlPanel; if it does not, the button is left out and the text points to the driver's own tool. The sample rate stays selectable. The user is the first reviewer.

Branch: worktree-agent-ad40cb43127c2f9dd (rebased onto fa022dc, local main with task-191).

2026-11-24 evidence.
- xwin: asio-sys builds under cargo-xwin (bindgen finds the xwin SDK headers); its build.rs searches vcvarsall.bat on any host unless VCINSTALLDIR is set, so scripts/build.sh sets it. cargo xwin clippy (x86_64) and scripts/package.sh x86_64-pc-windows-msvc pass; aarch64 windows clippy of orb-player passes without asio (feature is x86_64 only: arm64 processes cannot load x64 ASIO drivers).
- asio-sys compiles every .cpp of common/, host/, host/pc/; the vendored subset (9 files + LICENSE.txt, README with SHA-256s) builds without the driver-side files.
- Driver Settings button: NOT possible. asio-sys 0.4.0 allowlists only the calls cpal needs (no ASIOControlPanel) and cpal 0.18.2 has no control-panel API. Button left out; the buffer row reads 'Set in the driver's own control panel'.
- Found while reading cpal: ASIO enumeration loads each driver in turn and stops at the first other driver while a stream holds one. Fix: orb_player::devices::all_devices caches the last full enumeration (thread_local) while a DriverHold (in Stream) exists; first ASIO device stands for the default; the 1 s default-device watch skips ASIO; no buffer size is requested from ASIO. A first test run failed tests/no_alloc.rs (it bans Mutex in orb-player/src): changed the cache to thread_local RefCell.
- Test machine (permits: all 6 held, released after; ASIO4ALL is not installed, FlexASIO is): zip unpacked to C:/re/rb/asio (removed after), OpenRB-338.exe --audio --play --screenshot with a portable OpenRB-338.toml. audio_driver=ASIO: In use 'ASIO: FlexASIO', 44100 Hz not resampled, 2 ch f32, 882 frames (20.0 ms), latency 60.0 ms, 0 underruns, trademark line shown. + audio_sample_rate=48000: 'resampled from 44100 Hz', 960 frames. Missing device 'asio:Gone Driver': status line 'Audio device Gone Driver is not available; using System default' and plays on FlexASIO. Default settings: WASAPI, 2048 frames asked, 970 granted, 0 underruns. Two instances on ASIO at once both worked (FlexASIO shares), so a really busy driver could not be provoked; the failure path is the task-191 open_chain fallback (unchanged). The run used the first zip build (before the thread_local cache change); the final zip differs only in that cache.
- Checks on the final tree: cargo test --workspace 1140 pass 0 fail; clippy -D warnings macOS, x86_64-pc-windows-msvc (xwin) clean; nightly fmt clean; shellcheck and actionlint clean.
- Zip on the Mac: /private/tmp/claude-501/-Users-pingw33n-dev-OpenRB-338/a9b177cc-ff45-4931-aea4-b72986a77486/scratchpad/dist/x86_64-pc-windows-msvc/OpenRB-338-2026.1124.1822-x86_64-pc-windows-msvc.zip

Review round 1 (user, 2026-11-24):
1. The ASIO trademark line is gone from the Audio Output window (the user sees none in REAPER or Ableton); it stays in README.md, docs/building.md, the package README.txt and THIRD-PARTY-NOTICES.txt. Task description and docs/building.md updated; the trademark unit test removed.
2. The In use numbers (output latency, underruns, granted size) were read live on every repaint, so they changed faster while the mouse moved. This is task-191's code, fixed here: AudioUi::sample (crates/orb-ui/src/audio_window.rs) keeps a cached copy and takes a new reading at most every SAMPLE_EVERY = 500 ms (the window's own repaint period), at once when the stream's setup changed (a reopen) and clears when there is no stream; the window draws the copy. Unit tests with an explicit clock: the_numbers_are_sampled_on_a_timer_and_not_on_every_repaint, a_new_stream_shows_at_once_and_no_stream_shows_nothing.
3. The user's tests on the real Windows machine (directly and over RDP): FlexASIO plays audibly in both. ASIO4ALL is silent there, probably because the output is HDMI: an ASIO4ALL limit, not orb's.
Rebased onto aa1bf25 earlier; full re-run follows in the commit.

Round 1 checks on the rebased tree (aa1bf25): cargo test --workspace 1141 pass, 0 fail; clippy -D warnings macOS and x86_64-pc-windows-msvc (xwin) clean; nightly fmt clean. Zip rebuilt: /private/tmp/claude-501/-Users-pingw33n-dev-OpenRB-338/a9b177cc-ff45-4931-aea4-b72986a77486/scratchpad/dist/x86_64-pc-windows-msvc/OpenRB-338-2026.1124.1912-x86_64-pc-windows-msvc.zip

Review round 2 (user, 2026-11-24): closing the app with ASIO4ALL's configuration window open left the process hanging (PID 17596, 'Not Responding', only window CicMarshalWnd).
Cause (minidump, process not killed: dump.ps1 with MiniDumpWriteDump, full memory and thread info, 163 MB, read on the Mac with a hand-written parser plus the export names of ntdll and kernel32): one thread left, the main thread, in the process exit. Stack, top first: ntdll!ZwDelayExecution < KERNELBASE!Sleep < asio4all64.dll+0xd6b8/0xc77c/0x1f1d0 (ASIO4ALL 2.22 of 2026-07-23) < ntdll!LdrShutdownProcess (the DLL_PROCESS_DETACH calls) < ntdll!RtlExitUserProcess < kernel32!ExitProcess < OpenRB-338.exe+0xcfd227 (mainCRTStartup) < BaseThreadInitThunk < RtlUserThreadStart. So main had returned and the stream was stopped; asio4all64.dll's DllMain sleeps in a wait loop for a thread of its own (the panel's UI thread), which ExitProcess had killed before the detach calls: the wait never ends. A driver behavior, not cpal's.
Fix: orb-ui main ends with TerminateProcess (no DLL detach) after run_native returns (window closed, settings saved, player stopped) when the process made the ASIO host (orb_player::devices::asio_was_used, set in devices::host for ASIO); other processes end as before. Documented in docs/building.md (ASIO section).
Reproduction: not reproduced before the fix. Opening the panel with a4apanel.exe (a separate process connecting to the driver in ours) and closing the app (taskkill, WM_CLOSE) exited cleanly; the hang needs the panel the driver opens itself from its in-process tray icon, and my clicks on the tray icons (rcs input on the user's desktop) opened nothing, so I stopped. After the fix: ASIO4ALL selected, song playing, closed with taskkill: process gone within 8 s, exit 0 (new zip). The user re-tests with the panel open. PID 17596 was left alone (the user's); my own processes and the files under C:/re/rb (asio2, dump) are removed; permits released.
Checks: cargo test --workspace 1141 pass, 0 fail; clippy -D warnings macOS and x86_64-pc-windows-msvc clean; nightly fmt clean. Zip: /private/tmp/claude-501/-Users-pingw33n-dev-OpenRB-338/a9b177cc-ff45-4931-aea4-b72986a77486/scratchpad/dist/x86_64-pc-windows-msvc/OpenRB-338-2026.1124.1931-x86_64-pc-windows-msvc.zip

Sign-off 2026-11-24: the user tested the round-2 zip (2026.1124.1931, ddda328) on Windows, including closing with ASIO4ALL's configuration window open, and signed off with 'good'. That covers the decisions as built: ASIO on x86_64 Windows only, the cached device list while a stream is open, the first ASIO driver standing in for System default, no Driver Settings button, no trademark line in the window, steady In use numbers, TerminateProcess after ASIO.
Finish: rebased onto a922dba (only backlog commits since aa1bf25). Criterion 2 reworded: the release.yml LLVM step is only checkable by a release run, so the new task-298 (the next release run) checks it; here it covers the script, shellcheck and actionlint and the workflow wiring. The ci.yml LLVM step runs on the first push after the merge: the first CI run confirms it (not claimed before). Likewise 'still pass CI' of criterion 1 rests on the local macOS clippy and tests and on the feature being x86_64 Windows only; the first CI run confirms Linux and the other targets.
Evidence per criterion: 1 third_party/asiosdk (10 files, SHA-256s in its README, CPAL_ASIO_DIR in .cargo/config.toml [env], no download; cargo xwin build and package pass; macOS clippy and tests pass, aarch64 Windows clippy passes without the feature). 2 scripts/install-llvm-windows.sh, shellcheck and actionlint clean, wired in ci.yml (Windows x86_64 test job) and release.yml (x86_64-pc-windows-msvc package job), docs/building.md ASIO section. 3 scripts/asio-notices.sh output seen in the zip (THIRD-PARTY-NOTICES.txt: trademark line, LICENSE.txt, BSD notice), installer script ships it, README and package README.txt carry the line. 4 the zip run on the test machine: ASIO listed and playing, Automatic stays WASAPI, missing device falls back with the note; a really busy driver could not be provoked (FlexASIO shares), the failure path is task-191's open_chain. 5 the test-machine runs above (FlexASIO), plus the user's own tests (FlexASIO audible, ASIO4ALL silent on HDMI).
Final checks: cargo test --workspace 1141 pass; clippy macOS and x86_64-pc-windows-msvc clean; nightly fmt clean. One run on the dirty tree failed the_vst3_factory_has_the_identity (the plugin's version string says dev-<hash>-dirty while the test binary is clean: uncommitted backlog edits at the time); it passes on the committed tree. Zip: /private/tmp/claude-501/-Users-pingw33n-dev-OpenRB-338/a9b177cc-ff45-4931-aea4-b72986a77486/scratchpad/dist/x86_64-pc-windows-msvc/OpenRB-338-2026.1124.1931-x86_64-pc-windows-msvc.zip

Review round 3 (reviewer, sent back after Done; amended, the task stays Done):
1. Commit body: Decision: task-192 added; subject areas player,ui,tools (scripts changed).
2. Criterion 5 reworded to match the evidence (FlexASIO: my runs and the user's hearing it directly and over RDP; ASIO4ALL: the user's runs, silent over HDMI, exit with panel re-tested) and checked.
3. docs/building.md: 'the fallbacks fire' replaced by what was seen (rate change, missing-device fallback; a busy driver could not be provoked).
4. Other exit paths with ASIO loaded: orb_ui::crash::end_process(code) (TerminateProcess when an ASIO host was made, else std::process::exit) now ends the start-error path in main.rs, the panic hook in crash.rs (both exits, after the crash log is written and flushed) and the --screenshot exit in app.rs; terminate_if_asio_was_used(0) after run_native. The help and usage exits run before any host is made. main.rs comment and docs say the player has stopped because App and its Controller are dropped inside run_native.
Checks: cargo test --workspace, clippy macOS and x86_64-pc-windows-msvc, nightly fmt: see the final lines below.

Round 3 final checks: cargo test --workspace 1141 pass, 0 fail; clippy -D warnings macOS and x86_64-pc-windows-msvc clean; nightly fmt clean. Zip: /private/tmp/claude-501/-Users-pingw33n-dev-OpenRB-338/a9b177cc-ff45-4931-aea4-b72986a77486/scratchpad/dist/x86_64-pc-windows-msvc/OpenRB-338-2026.1124.2002-x86_64-pc-windows-msvc.zip
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
ASIO output on Windows x86_64 (GitHub issue 12), decided by task-192 (SDK 2.3.4 under its GPLv3 option, vendored minimally). Vendored in third_party/asiosdk (10 files, unmodified, README with URL and SHA-256s), CPAL_ASIO_DIR set in .cargo/config.toml; cpal's asio feature for x86_64 Windows only (arm64 cannot load x64 drivers). ASIO is a driver in the Audio Output window, after the default, never chosen by Automatic; the first ASIO driver stands in for System default, the buffer is the driver's (row: Set in the driver's own control panel), a failing driver falls back through task-191's chain. cpal enumerates ASIO by loading each driver in turn and one driver loads at a time, so the device list is cached while a stream holds a driver. No Driver Settings button (neither cpal nor asio-sys exposes ASIOControlPanel). The In use numbers are sampled every 500 ms (a fix of task-191's window). ASIO4ALL's DllMain hangs the process at exit when its panel is open (minidump: main thread in LdrShutdownProcess, asio4all64.dll, Sleep): orb-ui ends with TerminateProcess after an ASIO host was made (docs/building.md). The x86_64 zip and installer carry THIRD-PARTY-NOTICES.txt (scripts/asio-notices.sh: trademark line, SDK LICENSE.txt, host helpers' BSD notice); scripts/install-llvm-windows.sh in ci.yml and release.yml; build.sh sets VCINSTALLDIR for cargo-xwin, which builds the ASIO package. Branch worktree-agent-ad40cb43127c2f9dd, one commit (user review: two rounds, signed off 2026-11-24). Tests: cargo test --workspace 1141 pass, clippy -D warnings macOS and x86_64-pc-windows-msvc, nightly fmt clean; manual runs on the test machine with FlexASIO. Open: the release.yml LLVM step is checked by a release run (task-298); the first ci.yml run confirms the ci.yml step and the other platforms.
<!-- SECTION:FINAL_SUMMARY:END -->
