---
id: TASK-223
title: 'Portable Windows zip: settings and logs beside the program, relative folders'
status: Done
assignee:
  - '@supervisor'
created_date: '2026-11-09T18:14:00Z'
updated_date: '2026-11-09T18:31:00Z'
labels:
  - ui
  - tools
  - docs
milestone: m-7
dependencies:
  - TASK-222
references:
  - crates/orb-ui/src/settings.rs
  - crates/orb-ui/src/crash.rs
  - scripts/package.sh
priority: high
type: feature
ordinal: 241000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Why: the user's decision TASK-222 (2026-11-09): the fork's releases are a portable zip only, and a portable copy stores the folders inside its own folder as relative paths, so a copy on a USB stick works on any PC. Before this, the zip still wrote ui.toml to %APPDATA%\OpenRB-338 (settings.rs default_file) and its logs to %LOCALAPPDATA%\OpenRB-338\logs (crash.rs log_dir, used by the crash log and the run log of task-214). Nothing else is written outside the program's folder.

What:
- crates/orb-ui/src/portable.rs: portable mode when a file portable.txt is next to the running OpenRB-338.exe; its data folder is data\ beside it. Not portable without the marker, so upstream's packages and a build without it behave as before.
- settings.rs default_file: ORB_UI_CONFIG first, then data\ui.toml in portable mode, then the platform's folder. Settings gets a base folder (the portable folder): on save, songs_dir, mods_dir, recent, last_song_dir and last_mod_dir inside it are written relative to it (Windows compares without case); on load a relative path is joined to it. Folders elsewhere stay full paths.
- crash.rs log_dir: ORB_UI_LOG_DIR first, then data\logs in portable mode, then the platform's.
- The run log names the mode and the data folder.
- scripts/package.sh: the Windows zip carries portable.txt (it explains itself) and README.txt says the zip is portable.
- Releases: the zip only (docs/building.md, the release steps of the fork); README.md Quick start: the zip, unpacked anywhere.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 With portable.txt next to OpenRB-338.exe, ui.toml is read and written in data\ beside it and the logs go to data\logs; without it the paths are as before; ORB_UI_CONFIG and ORB_UI_LOG_DIR still win
- [x] #2 In portable mode folders inside the portable folder are saved relative to it and read back as full paths there; folders elsewhere stay full paths; tests cover both and a different drive letter case
- [x] #3 The release zip carries portable.txt, README.txt says it is portable; the run log names the mode; README.md and docs/building.md describe the portable zip as the fork's one download
- [x] #4 fmt, clippy -D warnings and cargo test --release -p orb-ui pass
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: task-223-portable (in the main checkout, to reuse the build cache).

Work (2026-11-09): crates/orb-ui/src/portable.rs (MARKER portable.txt, DATA data, root_in, root (OnceLock of current_exe's folder), data_dir, relative/relative_with (components compared without case on Windows; a path with .. after the base stays full), resolve). settings.rs: default_file after ORB_UI_CONFIG takes data\ui.toml in portable mode; Settings.base; load() = load_from_in(file, portable::root()); load_from = load_from_in(file, None); save writes folders inside base relative (Settings::stored, Cow paths); a recent folder named twice after resolving is kept once. crash.rs log_dir: ORB_UI_LOG_DIR, then data\logs in portable mode, then the platform's. main.rs: run log note of the mode. scripts/package.sh + scripts/portable.txt: the Windows zip carries the marker, README.txt has a portable paragraph; tar.gz unchanged.

Checks: cargo +nightly fmt --check clean; cargo clippy -p orb-ui --all-targets --release -- -D warnings clean; cargo test --release -p orb-ui 461 pass, 0 fail, 3 ignored (new: portable unit tests 5, settings tests a_portable_copy_stores_its_own_folders_relative, without_a_portable_folder_the_paths_stay_full, a_portable_copy_merges_a_recent_folder_named_twice). The real exe: target/release/OpenRB-338.exe copied to a scratch folder with portable.txt, run with --debug-start-error and ORB_UI_NO_CRASH_DIALOG=1: run log in data\logs beside it with "portable (portable.txt found): settings and logs in ...\data", nothing in %LOCALAPPDATA%\OpenRB-338\logs; without portable.txt the log went to %LOCALAPPDATA% ("not portable: ..."), then removed. Packaging: DIST=<scratch> SKIP_BUILD=1 scripts/package.sh x86_64-pc-windows-msvc: the zip lists portable.txt, README.txt has the paragraph; x86_64-unknown-linux-gnu: no portable.txt, no paragraph.

Review round 1 (agent-reviewer, send back), each point:
1-2. docs/building.md: the crash-log folder and the "Outside the script" run-log sentence now give data\logs for a portable copy.
3-4. runlog.rs and main.rs module docs name the portable paths.
5. The run log note now reports the settings file (settings::default_file) and the log folder (crash::log_dir) actually used, overrides included: "portable (portable.txt found): settings file ...; logs in ...".
6. Precedence is testable: settings::file_for(config, portable_data) and crash::log_dir_for(os, env, portable_data); tests orb_ui_config_wins_over_a_portable_folder (tests/settings.rs) and crash::tests::a_portable_copy_logs_beside_itself_unless_overridden.
7. Real exe, three runs with --debug-start-error and ORB_UI_NO_CRASH_DIALOG=1 (2026-11-09): with portable.txt, "settings file <app>\data\ui.toml; logs in <app>\data\logs" and the log there; with ORB_UI_CONFIG and ORB_UI_LOG_DIR set, both overrides reported and the log in the override folder; without portable.txt (ORB_UI_LOG_DIR set to keep AppData clean), "not portable: settings file %APPDATA%\OpenRB-338\ui.toml". %LOCALAPPDATA%\OpenRB-338 does not exist afterwards. The settings are saved to Settings.file, which is default_file's path, so the reported path is the one written.
8. The recent dedup compares with portable::same (case-insensitive on Windows); test a_portable_copy_merges_a_recent_folder_in_another_case (Windows).
9. The Windows-only drive letter tests run here (Windows 11): ok.
10. 8bf6bbe is TASK-222's own commit (a Decide record, as task-220's); both land by fast-forward.
Also found: tools/win_diag.ps1 Get-LogDir looked only in %LOCALAPPDATA%; it now follows the program's rule (ORB_UI_LOG_DIR, data\logs beside a portable exe, else %LOCALAPPDATA%). PowerShell parser: 0 errors.
After: cargo test --release -p orb-ui 464 pass, 0 fail, 3 ignored (11 new); clippy -D warnings and nightly fmt clean.

Review round 2 (merge). Its nits taken: README.md says the zip keeps its settings and logs in its own folder (not 'writes nothing outside'), docs/building.md 'the program writes nothing else of its own', and the rewrapped lines there. Merged by the supervisor (fast-forward of 8bf6bbe, TASK-222's record, and this commit) on 2026-11-09.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
The fork's Windows zip is portable: with portable.txt next to OpenRB-338.exe the settings are data\ui.toml and the logs data\logs beside it, nothing goes to AppData, and the songs and mods folders inside the program's folder are stored relative to it, so a USB stick works under any drive letter (crates/orb-ui/src/portable.rs, settings.rs, crash.rs; the run log names the mode). Without the marker nothing changes. scripts/package.sh puts the marker and a README paragraph into the Windows zip only. README.md, docs/ui_spec.md, docs/building.md (with the fork's release steps) and AGENTS.md describe the portable zip as the one download (task-222). Tests: orb-ui 461 pass (8 new), clippy and nightly fmt clean; the real exe checked in both modes. Branch task-223-portable.
<!-- SECTION:FINAL_SUMMARY:END -->
