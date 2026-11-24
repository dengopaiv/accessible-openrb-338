---
id: TASK-289
title: >-
  Settings file renamed to OpenRB-338.toml; the Windows zip is portable (app and
  plugin)
status: To Do
assignee:
  - '@agent-worker'
created_date: '2026-11-24T11:25:00Z'
updated_date: '2026-11-24T11:29:00Z'
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
- [ ] #1 Both the app and the plugin use OpenRB-338.toml on every platform, copying ui.toml into it once when only ui.toml exists; ui.toml is left untouched
- [ ] #2 Every save writes standalone_version or plugin_version atomically (temporary file and rename), keeping the other key; a test checks a written file is never blank
- [ ] #3 On Windows the standalone app (folder of the exe) and the plugin (folder holding the .vst3 bundle) follow the absent / non-empty / empty rules for a local OpenRB-338.toml, with tests for each case and for a comments-only file
- [ ] #4 In portable mode the crash logs, the plugin's log and switch file and every other file the app or plugin writes stay in the local folder; the list of per-user files is in the notes
- [ ] #5 An unwritable local folder gives a one-time status message (app and plugin) and no per-user writes
- [ ] #6 macOS and Linux never read a local file; an installed plugin behaves as today; cargo test --workspace, clippy -D warnings (including Windows cfg code) and nightly fmt --check pass
- [ ] #7 The Windows zip holds OpenRB-338.exe, the OpenRB-338.vst3 bundle and a zero-byte OpenRB-338.toml in one folder, and the installer does not install the file; the readme and docs/building.md and docs/ui_spec.md describe the rules
- [ ] #8 The user has checked portable mode on Windows from a built zip, in the app and in the plugin loaded from that folder
<!-- AC:END -->
