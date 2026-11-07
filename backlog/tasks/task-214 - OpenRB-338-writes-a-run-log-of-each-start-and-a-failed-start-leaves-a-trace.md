---
id: TASK-214
title: OpenRB-338 writes a run log of each start, and a failed start leaves a trace
status: Done
assignee:
  - '@agent-complex'
created_date: '2026-11-07T12:27:00Z'
updated_date: '2026-11-07T13:22:00Z'
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
- [x] #1 Every start of OpenRB-338 (not --help or a bad option) writes run-YYYYMMDD-HHMMSS.log into the crash log folder (ORB_UI_LOG_DIR or the platform's), each line flushed as written; the newest 20 run logs are kept, rotated apart from the crash logs
- [x] #2 The run log has a header (version, OS and its version with the Windows build number, architecture, arguments, executable, screen reader reported by the system, the ORB_* and WGPU_* variables set) and a timed line for each stage: the song, the audio output (device, rate, channels, format, resampled, or the error), run_native called, the graphics adapter (backend, type, name, driver), the first frame drawn, and the end
- [x] #3 An error returned by eframe::run_native is written to the run log and stderr and shown in the native box (unless ORB_UI_NO_CRASH_DIALOG), and the exit code is 1; the hidden option --debug-start-error takes that path before any window, and an integration test checks the log and the exit code
- [x] #4 Messages of the log crate (wgpu, winit, eframe, cpal, AccessKit) at warn and error go to the run log; ORB_LOG=info, debug or trace raises the level
- [x] #5 If no frame has been drawn 15 s after the start, the run log says so and names the last stage reached
- [x] #6 On Windows a native crash writes the exception code, the address and its module to the run log and still reaches Windows Error Reporting; with ORB_UI_NO_CRASH_DIALOG set there is no WER box; the hidden option --debug-native-crash triggers it, and a Windows-only test checks the log line and a failing exit code
- [x] #7 A panic adds a line naming its crash log to the run log
- [x] #8 docs/building.md describes the run log, ORB_LOG and the hidden options (the variable table and the developer options); cargo test -p orb-ui, cargo clippy --all-targets -- -D warnings and nightly fmt --check pass
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. crash.rs: generalize write_log/rotate/order_key to a file prefix (crash-, run-), keep the crash API as it is.
2. New module crates/orb-ui/src/runlog.rs: a global run log (OnceLock<Mutex<File>> plus start Instant); start(version, args) writes the header; stage(text) writes "+NNNNN ms  text" and remembers the last stage; frame() once; end(text); a log::Log implementation (warn by default, ORB_LOG level); the watchdog thread (15 s); on Windows the unhandled exception filter; the Windows version via RtlGetVersion.
3. main.rs: open the run log after argument parsing; stages for the song, the audio output, run_native, the adapter; run_native's Err to the log, stderr and the box, exit 1; hidden --debug-start-error and --debug-native-crash.
4. app.rs: runlog::frame() in App::ui.
5. crash.rs handle(): a line in the run log naming the crash log.
6. Tests: unit tests for the prefix rotation, the header and line format, the level parsing; integration tests (tests/run_log.rs) for --debug-start-error, --debug-panic and (Windows) --debug-native-crash.
7. docs/building.md: the run log section, ORB_LOG, the hidden options.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: task-214-run-log (in the main checkout, worked in the foreground by the supervisor: the user reads with NVDA and asked for no background churn)

Done (2026-11-07, Windows 11 26H2 build 26340, the development machine):
- crash.rs: create_log(dir, prefix, secs), rotate(dir, prefix, keep), file_name_of; write_log unchanged for crash logs; no_dialog() pub. The panic hook adds `end: panic on thread T: MSG (LOC); crash log PATH` to the run log.
- runlog.rs (new): header, timed lines, log-crate logger (warn default, ORB_LOG), 2000-line cap for library messages, watchdog thread, Windows SetUnhandledExceptionFilter line (code, address, module+offset; try_lock; EXECUTE_HANDLER with ORB_UI_NO_CRASH_DIALOG, else CONTINUE_SEARCH so WER and LocalDumps run), RtlGetVersion + registry UBR/DisplayVersion.
- main.rs: main returns ExitCode; stages for the song, audio output (device, rate, channels, format, resampled / error), run_native, adapter (always; ORB_GPU_INFO still prints to stderr), app ready; run_native Err -> start_failed (stderr, run log, rfd box, exit 1). Hidden --debug-start-error, --debug-native-crash, --debug-hang (watchdog 1 s, waits 3 s, then the start error path).
- app.rs: runlog::frame() in App::ui; the screenshot path writes its end before process::exit(0).
A real start (debug, --demo --size 624x510 --screenshot, ORB_LOG=info): 41 ms creating the window; Vulkan loader warnings at 1189 ms; egui_wgpu lists 6 adapters at 1710 ms; 2044 ms window created; graphics: Vulkan, DiscreteGpu "Quadro P2000", driver "NVIDIA" 556.12; 2052 ms first frame drawn; 4083 ms end: the screenshot was saved.
Tests: runlog unit 7 pass; crash unit 8 pass (new: run_logs_and_crash_logs_are_named_and_rotated_apart); bin OpenRB-338 6 pass (new: the_debug_failures_parse); tests/run_log.rs 5 pass in debug and in release (start error exit 1, panic names crash log, three starts three logs, watchdog line between the stage and the end, native crash exit 0xC0000005 with `access violation (0xC0000005) writing address 0x8, at 0x..., ...OpenRB-338.exe+0x...`); tests/crash.rs 2 pass (now counts crash-* files only); cli_args 3 pass.
cargo test -p orb-ui (debug): all pass except system_dialog::a_name_the_dialog_did_not_confirm_asks_before_it_replaces_a_file, which fails the same way on main without this change (git stash, 2026-11-07): epaint 0.37 textures.rs:337 debug_assert "Dropped TexturesDelta with 1 unapplied deltas". Not caused by this task; reported to the user for a follow-up task.
clippy -p orb-ui --all-targets -D warnings clean; cargo +nightly fmt --all --check clean.
Not seen: the native box of a start error on screen (tests turn it off; it is the crash hook's rfd call, task-157). The native crash line under WER's own box (tests use the no-dialog path).

Review round 1 (agent-reviewer, 2026-11-07): merge, with notes. Applied: the watchdog no longer writes after the end line (runlog.rs watchdog: skips when the last stage starts with 'end: ', e.g. a start error's box open more than 15 s); main.rs --debug-native-crash's SAFETY comment says undefined behavior on purpose; runlog.rs describe() comments the loader lock GetModuleHandleExW takes. Raised for the user: with screen reader mode off the app now also writes a run log and shows a box on a start error (the fork's 'behaves as upstream' rule); the user asked for both on 2026-11-07. After: run_log 5, crash 2, bin 6, runlog 7 pass; clippy and nightly fmt clean.

Re-review round 2 (agent-reviewer, 2026-11-07): merge. At merge the supervisor corrected the loader-lock comment in runlog.rs describe() (the lock is recursive; only another thread holding it can make the filter wait), the reviewer's note.
Merged by the supervisor 2026-11-07: review merge on the second round (round 1 was merge with notes, three applied). The fork-rule note (a run log and a start-error box also with screen reader mode off) was raised with the user, who asked for both that day.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
OpenRB-338 writes run-YYYYMMDD-HHMMSS.log on every start (crates/orb-ui/src/runlog.rs): a header with the version, the Windows build, arguments, screen reader and variables; timed stages (song, audio output, window, graphics adapter, first frame); the libraries' log messages (ORB_LOG); a 15 s watchdog line naming the last stage; on Windows a native crash line (code, address, module) before Windows Error Reporting; and the end (window closed, start error, panic with its crash log). An error of eframe::run_native now goes to stderr, the run log and a native box, exit 1, instead of vanishing. Hidden --debug-start-error, --debug-native-crash and --debug-hang test the paths. docs/building.md: the run log section, ORB_LOG, the options. Branch task-214-run-log. Tests: tests/run_log.rs 5 pass in debug and release, runlog 7, crash 8+2, bin 6; clippy and nightly fmt clean. A pre-existing debug-only failure on main (system_dialog, epaint TexturesDelta debug_assert) is not this task's.
<!-- SECTION:FINAL_SUMMARY:END -->
