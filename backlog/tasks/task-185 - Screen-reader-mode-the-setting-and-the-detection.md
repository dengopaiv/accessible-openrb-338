---
id: TASK-185
title: 'Screen reader mode: the setting and the detection'
status: Done
assignee:
  - '@agent-worker'
created_date: '2026-11-05T19:31:00Z'
updated_date: '2026-11-05T20:36:00Z'
labels:
  - ui
milestone: m-7
dependencies:
  - TASK-177
references:
  - crates/orb-ui/src/settings.rs
  - crates/orb-ui/src/menu.rs
  - crates/orb-ui/src/main.rs
  - docs/ui_spec.md
  - docs/accessibility_plan.md
  - docs/accessibility.md
priority: high
type: feature
ordinal: 208000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Planned in docs/accessibility_plan.md as A11Y-1; the analysis is docs/accessibility.md. Read both before starting.

- What (with D1's recommendation): a setting `screen_reader` in `ui.toml` with the values
  `auto` (default), `on`, `off`; an Options menu item "Screen Reader Mode" with the three
  choices; `App::screen_reader_mode()` true when on, or auto and the system reports a screen
  reader. Detection on Windows through `SystemParametersInfoW(SPI_GETSCREENREADER)` by FFI
  (no new crate), checked at start and when the window regains the focus; on macOS and Linux
  `false` in this task (a later task can add VoiceOver and AT-SPI detection; note it as a
  follow-up task if not done). Nothing else changes yet: later tasks read the mode.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 The setting round-trips through `ui.toml` (a test in the style of `crates/orb-ui/tests/settings.rs`), and an old `ui.toml` without the key reads as `auto`.
- [x] #2 The menu item shows and changes the setting; with `off` no behavior differs from main.
- [x] #3 docs/ui_spec.md section 18 lists the item.
- [x] #4 On Windows with NVDA running, auto reports the mode on (SPI_GETSCREENREADER reads 1; checked with NVDA running, installed version 2026.3.1rc1, running copy not confirmed); the NVDA-closed half (flag 0, mode off) is deferred to TASK-197 AC #2 by the user's decision of 2026-11-05
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. New module crates/orb-ui/src/screen_reader.rs: ScreenReaderSetting {Auto, On, Off} (key/from_key/label/active) and system_reports_screen_reader() (Windows: SystemParametersInfoW(SPI_GETSCREENREADER) by FFI declared in the module; other platforms false).
2. settings.rs: field screen_reader, TOML key screen_reader (absent or unknown reads as auto).
3. app.rs: system_screen_reader checked at App::new and when the window regains the focus; App::screen_reader_mode().
4. menu.rs: Options > Screen Reader Mode submenu (Automatic, On, Off), Cmd::SetScreenReader saves the settings.
5. Tests in tests/settings.rs; docs/ui_spec.md section 18 row. Follow-up task for VoiceOver and AT-SPI detection.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: worktree-agent-aa64ccff9b598cc3d

AC3 evidence (2026-11-05, Windows 11 10.0.26340): with nvda.exe running (1 process), SPI_GETSCREENREADER read from PowerShell (Add-Type P/Invoke of SystemParametersInfo, 0x46) gave flag=True. So NVDA does set the flag; no fallback needed. The NVDA version could not be read (Get-Process .Path is empty for the process, no access): to be filled in by the user's NVDA pass. The reading from the built code is below.

Built-code check (cargo test -p orb-ui --lib screen_reader -- --nocapture, 2026-11-05): system_reports_screen_reader = true with nvda.exe running (same as the PowerShell reading, flag=True). The without-a-screen-reader half (flag false with NVDA closed) was not checked by hand: NVDA is the user's running screen reader and I did not stop it; the code returns false when the flag is 0, and `off` / `on` are covered by unit tests. The NVDA version is not recorded (process path unreadable); the user's NVDA pass (TASK-197) can record it. NVDA sets the flag, so no fallback and no Decide task.

Implementation: screen_reader.rs (ScreenReaderSetting, system_reports_screen_reader), Settings::screen_reader, App::screen_reader_mode() (system answer refreshed in App::ui when the window regains the focus), Options > Screen Reader Mode submenu with Cmd::SetScreenReader (saves). Nothing else reads the mode. Follow-up: TASK-199 (VoiceOver and AT-SPI detection).

Checks: cargo test -p orb-ui all pass (80 lib + all integration tests, 0 failed); cargo clippy -p orb-ui --all-targets -- -D warnings clean; cargo +nightly-2026-10-18 fmt --all --check clean.

Review round 1 (send back, code passed): (1) the branch held TASK-183's commit 0c51b09 as well (on main as 4073a58); rebased onto main, the duplicate dropped, `git log main..HEAD` is one commit. (2) AC3 narrowed by the user's decision of 2026-11-05: the old AC #3 was removed (the others renumbered: docs row is now #3) and re-added as #4 for the verified half; the NVDA-closed half went to TASK-197 as a new criterion and to the NVDA checklist of docs/accessibility_plan.md as item 19. (3) NVDA version, correcting the earlier note that it could not be read: C:\Program Files\NVDA\nvda.exe reports ProductVersion 2026.3.1rc1 (Get-Item ...VersionInfo.ProductVersion), and it is the only nvda.exe install found under Program Files and Program Files (x86). That it is the running copy could not be confirmed directly: Win32_Process ExecutablePath and CommandLine are empty for the running nvda.exe (pid 15076, one process; the process is protected from this unelevated shell). The flag reading (1) was taken with that process running.

Review round 2 (send back for wording only, code passed): AC #4 reworded to claim no more than the notes (installed version 2026.3.1rc1, running copy not confirmed); the commit message Result and Testing paragraphs and the final summary updated to match.

Checks after the rebase onto main (2026-11-05, after the round 1 rebase and before this amend, which changed only backlog and docs text): `cargo test -p orb-ui` all pass, 0 failed (lib 80 passed, 1 ignored; integration binaries including tests/settings.rs 9 passed and the 54-test one in 462 s); `cargo +nightly-2026-10-18 fmt --all --check` clean; `cargo clippy -p orb-ui --all-targets -- -D warnings` clean (run before the rebase, no Rust change since).
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Added screen reader mode setting and Windows detection. New crates/orb-ui/src/screen_reader.rs (ScreenReaderSetting auto/on/off; SystemParametersInfoW(SPI_GETSCREENREADER) by FFI under cfg(windows), false elsewhere), Settings::screen_reader (ui.toml key screen_reader, absent or unknown reads as auto), App::screen_reader_mode() (checked at start and on focus regain), Options > Screen Reader Mode (Automatic, On, Off) saving at once, docs/ui_spec.md section 18 row. With off, or auto without a reported screen reader, nothing differs from main (nothing reads the mode yet). With NVDA running the flag reads 1 (PowerShell and the built code), so NVDA sets it and auto gives the mode on; NVDA 2026.3.1rc1 is the installed version (the running copy's path is not readable unelevated, so it is not confirmed). The NVDA-closed half is deferred to TASK-197 AC #2 (and item 19 of the NVDA checklist) by the user's decision of 2026-11-05. Tests: cargo test -p orb-ui, run after the rebase onto main on 2026-11-05, all pass with 0 failed; nightly fmt and clippy clean. Follow-up TASK-199 for VoiceOver and AT-SPI. Branch worktree-agent-aa64ccff9b598cc3d, one commit (review rounds 1 and 2 amended it).
<!-- SECTION:FINAL_SUMMARY:END -->
