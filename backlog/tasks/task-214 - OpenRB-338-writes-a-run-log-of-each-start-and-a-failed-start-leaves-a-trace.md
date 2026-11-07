---
id: TASK-214
title: OpenRB-338 writes a run log of each start, and a failed start leaves a trace
status: To Do
assignee:
  - '@agent-complex'
created_date: '2026-11-07T12:27:00Z'
labels:
  - ui
  - tools
milestone: m-7
dependencies: []
references:
  - crates/orb-ui/src/main.rs
  - crates/orb-ui/src/crash.rs
  - crates/orb-ui/src/app.rs
  - crates/orb-ui/src/screen_reader.rs
  - crates/orb-player/src/output.rs
  - docs/building.md
priority: high
type: feature
ordinal: 232000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Why: on the user's real Windows 10 PC, OpenRB-338 often does not start (the user, 2026-11-07: "We have a real win10 pc. The program doesn't even start too often."). Today such a start leaves no trace: the release build is a GUI-subsystem program, so when it is started from Explorer or the Start menu its stderr goes nowhere. Only a Rust panic writes something (the crash log of task-157, crates/orb-ui/src/crash.rs). Three failures leave nothing:
- an error returned by eframe::run_native (no usable graphics adapter, no window): main returns it, the text goes to the invisible stderr, the window never appears, no log, no box;
- a native crash (an access violation in a graphics or audio driver, or in the accessibility layer): the panic hook never runs;
- a hang during the start (no window, the process stays).
The libraries (wgpu, winit, eframe, cpal, AccessKit) also report their warnings through the `log` crate, and the app sets no logger, so those are lost.

What: a run log of every start, in the folder of the crash logs (`ORB_UI_LOG_DIR`, else `%LOCALAPPDATA%\OpenRB-338\logs`, the platform's on macOS and Linux), so that a start that fails on the Windows 10 PC can be read afterwards with a screen reader, as plain text. It is part of the debug kit of the follow-up task (tools/win_diag.ps1 collects these logs).

Design notes (from the supervisor's reading, 2026-11-07):
- crash.rs already has the folder (`log_dir`), the file naming by UTC time with `-N` for the same second (`write_log`), the rotation (`rotate`, the newest 20) and the native box (rfd). Reuse them with a file prefix (`run-` beside `crash-`), each kind rotated on its own.
- Write and flush each line as it happens, so a hang or a native crash keeps the lines before it.
- Stages to log, each with milliseconds since the start: the header; the song from the command line (opened or the error); the audio output (orb_player::output::Stream has device, config, format, resampled; or the error); run_native called; the creation closure entered with the graphics adapter (cc.wgpu_render_state adapter.get_info(): backend, device_type, name, driver, driver_info); the first frame drawn (App::ui; a cheap atomic once); the end.
- The header: version (about::VERSION), OS and arch, on Windows the real build number (RtlGetVersion from ntdll; GetVersionEx lies), the arguments, the executable path, whether SPI_GETSCREENREADER reports a screen reader (screen_reader.rs), the ORB_* and WGPU_* variables that are set.
- The hang watchdog: a thread that, if no frame has been drawn 15 s after the start, writes a line naming the last stage reached.
- The native crash on Windows: SetUnhandledExceptionFilter writing code, address and the module that holds the address (GetModuleHandleExW with GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS, GetModuleFileNameW); it must not block (try_lock the file) and returns EXCEPTION_CONTINUE_SEARCH so Windows Error Reporting still runs (LocalDumps keep working); with ORB_UI_NO_CRASH_DIALOG set it ends the process without the WER box (tests).
- Not in scope: a box at the next start about the previous run's failure; changing the release or dist profile's debug info (task-200 left that to the user); the PowerShell collection script (next task).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Every start of OpenRB-338 (not --help or a bad option) writes run-YYYYMMDD-HHMMSS.log into the crash log folder (ORB_UI_LOG_DIR or the platform's), each line flushed as written; the newest 20 run logs are kept, rotated apart from the crash logs
- [ ] #2 The run log has a header (version, OS and its version with the Windows build number, architecture, arguments, executable, screen reader reported by the system, the ORB_* and WGPU_* variables set) and a timed line for each stage: the song, the audio output (device, rate, channels, format, resampled, or the error), run_native called, the graphics adapter (backend, type, name, driver), the first frame drawn, and the end
- [ ] #3 An error returned by eframe::run_native is written to the run log and stderr and shown in the native box (unless ORB_UI_NO_CRASH_DIALOG), and the exit code is 1; the hidden option --debug-start-error takes that path before any window, and an integration test checks the log and the exit code
- [ ] #4 Messages of the log crate (wgpu, winit, eframe, cpal, AccessKit) at warn and error go to the run log; ORB_LOG=info, debug or trace raises the level
- [ ] #5 If no frame has been drawn 15 s after the start, the run log says so and names the last stage reached
- [ ] #6 On Windows a native crash writes the exception code, the address and its module to the run log and still reaches Windows Error Reporting; with ORB_UI_NO_CRASH_DIALOG set there is no WER box; the hidden option --debug-native-crash triggers it, and a Windows-only test checks the log line and a failing exit code
- [ ] #7 A panic adds a line naming its crash log to the run log
- [ ] #8 docs/building.md describes the run log, ORB_LOG and the hidden options (the variable table and the developer options); cargo test -p orb-ui, cargo clippy --all-targets -- -D warnings and nightly fmt --check pass
<!-- AC:END -->
