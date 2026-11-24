---
id: TASK-289
title: >-
  Settings file renamed to OpenRB-338.toml; the Windows zip is portable (app and
  plugin)
status: Done
assignee:
  - '@agent-worker'
created_date: '2026-11-24T11:25:00Z'
updated_date: '2026-11-24T13:47:00Z'
labels:
  - ui
  - player
  - tools
milestone: m-5
dependencies: []
references:
  - crates/orb-ui/src/settings.rs
  - crates/orb-ui/src/crash.rs
  - crates/orb-plugin/src/editor.rs
  - crates/orb-plugin/src/plugin_log.rs
  - scripts/package.sh
  - scripts/installer/openrb.nsi
  - docs/building.md
priority: medium
type: feature
ordinal: 264700
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Why:** GitHub issue 19 asks for portable use on Windows. Today the settings always go to %APPDATA%\OpenRB-338\ui.toml. The user designed this on 2026-11-24, and portable mode is Windows-only.

**1. The rename, on every platform, backward compatible.** The settings file becomes `OpenRB-338.toml` in the same per-user folder (crates/orb-ui/src/settings.rs around 205-218: %APPDATA% on Windows, ~/Library/Application Support on macOS, $XDG_CONFIG_HOME or ~/.config on Linux, under OpenRB-338/).
- On load, if `OpenRB-338.toml` is absent and `ui.toml` exists there, copy `ui.toml` to `OpenRB-338.toml` and use it.
- `ui.toml` is never deleted or written again, so a downgraded older version still finds its (stale) settings.
- This applies to both the standalone app and the plugin, which uses orb_ui::settings (crates/orb-plugin/src/editor.rs).

**2. Portable mode, Windows only, for the standalone app and the plugin** (the user's decision of 2026-11-24). On startup each looks for `OpenRB-338.toml` in its own folder:
- the standalone app: the folder of the running OpenRB-338.exe (std::env::current_exe's parent);
- the plugin: the folder that holds the `OpenRB-338.vst3` bundle, never inside the bundle. The plugin's binary is OpenRB-338.vst3\Contents\x86_64-win\OpenRB-338.vst3; find it from the loaded module's path (e.g. GetModuleHandleExW with the address of a function in the plugin, then GetModuleFileNameW) and go up to the bundle's parent.

When the app and the plugin sit in one folder, they share the one file there. The rules:
- **absent:** use the per-user file as in point 1, with its fallback to ui.toml;
- **present and not empty:** use it, read and write it there, and never touch the per-user file;
- **present and empty:** read the per-user settings (with the ui.toml fallback), write them into the local file, and from then on use the local file.

"Empty" means the file holds no settings: zero bytes, whitespace only, or comments only (it parses to an empty table). A local file that does not parse is an error, shown as today's settings errors are. Portable mode never falls back to the per-user file.

macOS and Linux never look for a local file. An installed plugin in %COMMONPROGRAMFILES%\VST3 has no local file there, so it behaves as today (per-user).

**3. The file is never blank.** Every save writes `standalone_version` (in the app) or `plugin_version` (in the plugin), the version string of the About box, along with the other settings.
- Each writer keeps the other's key as it found it.
- Every write is atomic: write a temporary file in the same folder, then rename it over the old one. A crash or full disk mid-write must never leave a zero-byte file, which would read as "portable, copy the global settings in" or lose the settings.

**4. Portable mode keeps everything in the local folder.**
- The crash logs (crates/orb-ui/src/crash.rs log_dir: %LOCALAPPDATA%\OpenRB-338\logs) go to a `logs` folder in the local folder.
- So do the plugin's diagnostic log and its switch file (crates/orb-plugin/src/plugin_log.rs, SWITCH_FILE `plugin.log` in that log folder).
- List any other per-user file the app or the plugin writes on Windows (grep for APPDATA, LOCALAPPDATA, dirs, home) in the notes and move it too. The issue's requirement is no files written outside the portable folder.

**5. A local folder that can't be written** (e.g. the zip unpacked under Program Files): keep running with the local settings as read, show a status message once ("Settings cannot be saved in <folder>: <reason>"), and do not write per-user. The same holds for the plugin, in its window's status line.

Two hosts or an app and a plugin sharing one portable file: the atomic writes keep it whole, and the last save wins, as with the per-user file today.

**6. Packaging.**
- The Windows zip (scripts/package.sh, the *-windows-* branch) is portable by default. It holds OpenRB-338.exe, the OpenRB-338.vst3 bundle and a zero-byte `OpenRB-338.toml` in one folder, so the app and the plugin used from that folder share it. Check the zip's current layout and move the bundle there if it is elsewhere; record the layout in the notes.
- The readme says that a host must be pointed at that folder (its custom VST3 path) for the plugin to run portable.
- The installer (scripts/installer/openrb.nsi) must not install it, so an installed app stays per-user.
- The package's readme or the release notes text says how to turn portable mode off (delete the file) and on (create an empty one).
- docs/building.md and docs/ui_spec.md (settings) describe the rules.

**Out of scope (possible follow-up):** storing paths inside the exe's folder (recent files, the mods folder) relative to it, for a USB stick whose drive letter changes.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Both the app and the plugin use OpenRB-338.toml on every platform, copying ui.toml into it once when only ui.toml exists; ui.toml is left untouched
- [x] #2 Every save writes standalone_version or plugin_version atomically (temporary file and rename), keeping the other key; a test checks a written file is never blank
- [x] #3 On Windows the standalone app (folder of the exe) and the plugin (folder holding the .vst3 bundle) follow the absent / non-empty / empty rules for a local OpenRB-338.toml, with tests for each case and for a comments-only file
- [x] #4 In portable mode the crash logs, the plugin's log and switch file and every other file the app or plugin writes stay in the local folder; the list of per-user files is in the notes
- [x] #5 An unwritable local folder gives a one-time status message (app and plugin) and no per-user writes
- [x] #6 macOS and Linux never read a local file; an installed plugin behaves as today; cargo test --workspace, clippy -D warnings (including Windows cfg code) and nightly fmt --check pass
- [x] #7 The Windows zip holds OpenRB-338.exe, the OpenRB-338.vst3 bundle and a zero-byte OpenRB-338.toml in one folder, and the installer does not install the file; the readme and docs/building.md and docs/ui_spec.md describe the rules
- [x] #8 The user has checked portable mode on Windows from a built zip, in the app and in the plugin loaded from that folder
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. settings.rs: OpenRB-338.toml, one-time copy of ui.toml, Role with standalone_version/plugin_version on every save (other key kept), atomic writes, load_resolved(role, local, user) for the portable rules. 2. portable.rs: Windows module path (GetModuleHandleExW + GetModuleFileNameW), folder_of for the .vst3 bundle; crash log_dir follows it. 3. App::show shows load_error (box) and notice (status line); plugin loads as Role::Plugin. 4. package.sh portable Windows zip layout and README; docs. 5. Tests, cargo test, clippy mac and xwin, fmt, build the zip.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: worktree-agent-ac33aa943478f515d

Decisions. (1) The plugin finds its folder with the same code as the app: portable::module_path asks GetModuleHandleExW (FROM_ADDRESS|UNCHANGED_REFCOUNT) for the module that holds a function of orb-ui and GetModuleFileNameW for its path; the plugin links orb-ui in, so that is the plugin binary OpenRB-338.vst3\Contents\x86_64-win\OpenRB-338.vst3, and portable::folder_of goes up to the bundle's parent (tested with forward-slash paths on any OS). In the app the same call gives the exe, so no role is needed to find the folder. (2) Zip layout (Windows only; the other packages keep plugins/): OpenRB-338.exe, orb.exe, OpenRB-338.vst3/, OpenRB-338.toml (0 bytes), LICENSE, README.txt in one folder. Before: the bundle was in plugins/. (3) ORB_UI_CONFIG, when set, wins over portable mode. (4) Version keys: Settings.role (Standalone/Plugin); the plugin loads with Settings::load_plugin. (5) Unwritable portable folder: Settings.save returns Ok and sets Settings.notice once (App::show puts it on the status line, in app and plugin); a damaged local file sets Settings.load_error (error box via App::fail) with defaults and no per-user fallback. Writes were already atomic (temp file and rename); now one write_atomic helper, also used for the ui.toml copy.
Per-user files written on Windows (grep APPDATA, LOCALAPPDATA, temp_dir, create_dir_all): settings (moved), crash logs (moved to logs\ in the portable folder), plugin.log and its switch (same log_dir, moved). NOT moved: std::env::temp_dir()\OpenRB-338\clips and \mods (plugin: drag-out clips, a mod's unpacked copy; editor.rs 1234, clip.rs 376) and the drag's temp file (drag/windows.rs 604). They are temporary files and need a writable place whatever the folder is (a read-only Program Files folder); documented in docs/building.md. If the user wants them moved too, it is a small follow-up.
Nothing in the task turned out wrong. The export CL=-Wno-incompatible-pointer-types that scripts/build.sh sets is also needed for cargo xwin clippy off Windows (orb-cli's vorbis dependency).
Results: cargo test --workspace 1105 passed, 0 failed, 5 ignored; cargo clippy --all-targets -D warnings clean on macOS; cargo xwin clippy --target x86_64-pc-windows-msvc -p orb-ui -p orb-plugin --all-targets -D warnings clean (the Windows cfg code); cargo +nightly fmt --check clean. New: crates/orb-ui/tests/portable.rs (9 tests: ui.toml copy, existing new file wins, version keys, absent, non-empty, empty/whitespace/comments-only, empty with no per-user file, damaged, unwritable once) and 4 unit tests in portable.rs. Zip: dist/x86_64-pc-windows-msvc/OpenRB-338-2026.1124.1225-x86_64-pc-windows-msvc.zip (scripts/package.sh x86_64-pc-windows-msvc). AC 1-7 are verified by the above but not checked off yet; AC 8 (user checks on Windows) is open.

Review round 1 (the user's check of README.txt and the zip). Changes: (1) the installer sentence is gone from the portable section of the Windows zip README. (2) Turning portable mode off now says: move OpenRB-338.toml to %APPDATA%\OpenRB-338\, replacing the OpenRB-338.toml there (README.txt, docs/building.md, docs/ui_spec.md). (3) The CLAP paragraph is removed from every readme that had it: scripts/package.sh (the Linux, Windows and macOS archive readmes), scripts/package-installer.sh (Windows installer readme), scripts/package-dmg.sh (dmg readme), scripts/package-pkg.sh (macOS plugin pkg readme). The installer and pkg scripts that delete an old .clap are unchanged (only the text), as is scripts/changelog.sh's release-notes line (not a readme). (4) Versions: every write of OpenRB-338.toml goes through Settings::save, which always writes the writer's version key and keeps the other's. The one place that did not was the one-time ui.toml copy (a byte copy, no version key): it now loads ui.toml's values and saves them to the new file with the role's key (load_user takes the role, so the plugin writes plugin_version). The empty-file adoption and every save already wrote it. Applies to per-user and portable, on every platform. Not covered: the load of an existing file that has no version key writes nothing until the first save (a load never writes). One consequence of the change: the copy keeps only the known keys of ui.toml. Tests: the copy (app and plugin role) and the adoption check the key. (5) The README says the plugin is in the same folder as OpenRB-338.exe and that the host's VST3 path should point at that folder (the layout is unchanged: OpenRB-338.vst3\ next to OpenRB-338.exe).

Round 1 addition: the Windows zip ships the plugin as the single file OpenRB-338.vst3 (the binary of the bundle, package.sh copies Contents/*-win/OpenRB-338.vst3) next to OpenRB-338.exe, not the bundle folder; the installer still installs the bundle. portable::folder_of handles both shapes (tests: single file and bundle, in portable.rs). README, docs/building.md, docs/plugin.md updated. Temporary files stay in %TEMP% as decided.

Sign-off: the user tested commit 723acf9 on Windows on 2026-11-24 (portable app, portable plugin loaded from the zip's folder, empty-file adoption) and said '289 looks good'. Found during that test, out of scope here: a REAPER crash in egui-baseview 0.9, which frees textures before queue.submit; it gets its own task.

Review round 2 (send back): (1) The empty-file adoption no longer writes any per-user file: it reads OpenRB-338.toml, else ui.toml, with read_user (no write) and writes only the local file; the ui.toml-to-OpenRB-338.toml conversion stays for the per-user (non-portable) path only. The adoption test now has ui.toml only in the per-user folder, and asserts that no OpenRB-338.toml appears there and the folder's file count stays 1. (2) write_atomic syncs the temporary file (File::sync_all) before the rename. (3) Commit message updated: tests 1106 pass, the user's Windows sign-off of 2026-11-24. (4) Temporary files (%TEMP%\OpenRB-338 clips and mods, the drag's temp file) stay in %TEMP%: the user's decision of 2026-11-24 (temporaries can be written as it is now); this completes criterion 4's evidence. (5) docs/plugin.md reworded: the installer's bundle is portable only when an OpenRB-338.toml is put beside it; the zip README mentions the ui.toml fallback; the unwritable-folder notice comes on the first failed save (a portable folder that cannot be written is found when a save fails, not at the start, except in an empty-file adoption, which writes at the start). (6) Correction to the round 1 note: the CLAP paragraph was removed from the package readmes (scripts/package*.sh), not from the root README.md, which is untouched (the user is asked).

Round 2 addition (the user said yes): the CLAP paragraph (old lines 74-78) and the mention of the old .clap in the macOS pkg step are removed from the root README.md; the pkg step now ends at the VST3 folder, and the archives sentence says the Linux and macOS archives hold the bundle in plugins/ (the Windows zip has the file next to the exe). Surrounding text reads on without a gap.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
The settings file is OpenRB-338.toml on every platform (ui.toml is converted into it once through a normal save, so it carries the version key, and is never touched). Every save writes standalone_version or plugin_version atomically and keeps the other. On Windows the app (exe folder) and the plugin (folder of the single-file .vst3, or of the bundle) use a local OpenRB-338.toml by the absent / non-empty / empty rules; crash logs and the plugin log go to logs\ there; an unwritable folder gives one status-line notice and no per-user writes. The Windows zip is portable by default (exe, orb.exe, OpenRB-338.vst3 single file, empty OpenRB-338.toml, README); the installer is unchanged. CLAP paragraphs removed from all readmes. Commit on branch worktree-agent-ac33aa943478f515d. Testing: cargo test --workspace 1106 passed, 0 failed; clippy -D warnings on macOS and x86_64-pc-windows-msvc; nightly fmt --check; 13 new tests (tests/portable.rs, portable.rs). The user checked portable app, portable plugin and adoption on Windows from the built zip (2026-11-24). Temp files stay in %TEMP%.
<!-- SECTION:FINAL_SUMMARY:END -->
