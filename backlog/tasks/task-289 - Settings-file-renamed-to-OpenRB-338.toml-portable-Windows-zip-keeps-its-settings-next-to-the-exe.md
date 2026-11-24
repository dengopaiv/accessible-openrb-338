---
id: TASK-289
title: >-
  Settings file renamed to OpenRB-338.toml; portable Windows zip keeps its
  settings next to the exe
status: To Do
assignee:
  - '@agent-worker'
created_date: '2026-11-24T11:25:00Z'
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

**2. Portable mode, standalone app on Windows only.** On startup the standalone OpenRB-338.exe looks for `OpenRB-338.toml` in the folder of the running exe (std::env::current_exe's parent):
- **absent:** use the per-user file as in point 1, with its fallback to ui.toml;
- **present and not empty:** use it, read and write it there, and never touch the per-user file;
- **present and empty:** read the per-user settings (with the ui.toml fallback), write them into the local file, and from then on use the local file.

"Empty" means the file holds no settings: zero bytes, whitespace only, or comments only (it parses to an empty table). A local file that does not parse is an error, shown as today's settings errors are. Portable mode never falls back to the per-user file.

macOS, Linux and the plugin never look for a local file; they always use the per-user one. The plugin is installed in an admin-only folder, and the user chose Windows-only.

**3. The file is never blank.** Every save writes `standalone_version` (in the app) or `plugin_version` (in the plugin), the version string of the About box, along with the other settings.
- Each writer keeps the other's key as it found it.
- Every write is atomic: write a temporary file in the same folder, then rename it over the old one. A crash or full disk mid-write must never leave a zero-byte file, which would read as "portable, copy the global settings in" or lose the settings.

**4. Portable mode keeps everything next to the exe.**
- The crash logs (crates/orb-ui/src/crash.rs log_dir: %LOCALAPPDATA%\OpenRB-338\logs) go to a `logs` folder next to the exe.
- List any other per-user file the standalone app writes on Windows (grep for APPDATA, LOCALAPPDATA, dirs, home) in the notes and move it too. The issue's requirement is no files written outside the exe's folder.

**5. A local folder that can't be written** (e.g. the zip unpacked under Program Files): keep running with the local settings as read, show a status message once ("Settings cannot be saved next to OpenRB-338.exe: <reason>"), and do not write per-user.

**6. Packaging.**
- The Windows zip (scripts/package.sh, the *-windows-* branch) includes a zero-byte `OpenRB-338.toml` next to OpenRB-338.exe, so the zip is portable by default.
- The installer (scripts/installer/openrb.nsi) must not install it, so an installed app stays per-user.
- The package's readme or the release notes text says how to turn portable mode off (delete the file) and on (create an empty one).
- docs/building.md and docs/ui_spec.md (settings) describe the rules.

**Out of scope (possible follow-up):** storing paths inside the exe's folder (recent files, the mods folder) relative to it, for a USB stick whose drive letter changes.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Both the app and the plugin use OpenRB-338.toml on every platform, copying ui.toml into it once when only ui.toml exists; ui.toml is left untouched
- [ ] #2 On Windows the standalone app follows the absent / non-empty / empty rules for OpenRB-338.toml next to the exe, with tests for each case and for a comments-only file
- [ ] #3 Every save writes standalone_version or plugin_version atomically (temporary file and rename), keeping the other key; a test checks a written file is never blank
- [ ] #4 In portable mode the crash logs and every other file the app writes stay next to the exe; the list of per-user files is in the notes
- [ ] #5 An unwritable local folder gives a one-time status message and no per-user writes
- [ ] #6 The Windows zip contains a zero-byte OpenRB-338.toml next to OpenRB-338.exe and the installer does not install it; docs/building.md and docs/ui_spec.md describe the rules
- [ ] #7 macOS, Linux and the plugin never read a local file; cargo test --workspace, clippy -D warnings (including Windows cfg code) and nightly fmt --check pass
- [ ] #8 The user has checked portable mode on Windows from a built zip
<!-- AC:END -->
