---
id: TASK-295
title: >-
  Portable mode is enabled by an OpenRB-338.portable marker file, not an empty
  OpenRB-338.toml
status: To Do
assignee:
  - '@agent-worker'
created_date: '2026-11-24T16:47:00Z'
labels:
  - ui
  - player
  - tools
milestone: m-5
dependencies: []
references:
  - crates/orb-ui/src/portable.rs
  - crates/orb-ui/src/settings.rs
  - scripts/package.sh
  - docs/building.md
  - docs/ui_spec.md
  - docs/plugin.md
priority: medium
type: enhancement
ordinal: 170
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Why:** the user decided on 2026-11-24. task-289 made the Windows zip portable by shipping a zero-byte OpenRB-338.toml next to OpenRB-338.exe. An empty local file meant "adopt the per-user settings" and a non-empty one "use me".
- Upgrading a portable installation means unzipping the new version over the old folder.
- That overwrites the user's filled OpenRB-338.toml with the zip's empty one, and the next start copies the per-user settings over the portable ones. The portable settings are lost.
A separate marker file fixes it: the zip ships the marker and never the settings file, so unzipping over an installation cannot touch the settings.

**The new rule** (Windows only, the standalone app and the plugin, as task-289):
- **Marker present:** an `OpenRB-338.portable` file sits in the local folder: the exe's folder for the app, the folder holding the .vst3 (single file or bundle) for the plugin, found as in task-289's `portable::folder_of`. Its content is ignored, and it may be empty. Then portable mode is on, and the settings live in the local OpenRB-338.toml.
  - If OpenRB-338.toml is absent there, the first start reads the per-user settings (OpenRB-338.toml, else ui.toml, writing nothing per-user, as task-289's `read_user`) and writes them into the local file.
  - If it is present, it is used as is. A damaged one is an error box and defaults, never a fallback.
  - The logs go to the local logs folder, and the unwritable-folder notice is as before.
- **Marker absent:** per-user settings, as for an installed copy, even if a local OpenRB-338.toml exists. That file is ignored, not deleted.
- The empty-file and whitespace-only rules of task-289 go: an empty local OpenRB-338.toml with no marker means nothing. task-289 never shipped in a release (the latest is v2026.1118.0919, before it), so there is no compatibility code for it.
- Keep task-289's atomic writes, sync before rename, and version keys.

**Packaging:**
- The Windows zip (scripts/package.sh) ships an `OpenRB-338.portable` marker (empty, or a line saying what it is) next to OpenRB-338.exe and OpenRB-338.vst3, and no OpenRB-338.toml.
- The installer ships neither.

**README.txt and docs:**
- Portable mode is on while OpenRB-338.portable exists.
- To upgrade, unzip the new version over the folder; the settings in OpenRB-338.toml stay.
- To turn portable mode off, delete OpenRB-338.portable, and move OpenRB-338.toml to %APPDATA%\OpenRB-338\ (replacing the one there) to keep the settings.
- To turn it on, create the marker.
- Update docs/building.md, docs/ui_spec.md and docs/plugin.md.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 With OpenRB-338.portable present the app and the plugin use the local OpenRB-338.toml, creating it from the per-user settings (writing nothing per-user) when absent; with no marker they use the per-user file even if a local OpenRB-338.toml exists; tests cover each case
- [ ] #2 task-289's empty-file rule is gone, with no compatibility code; atomic writes and version keys stay
- [ ] #3 The Windows zip ships OpenRB-338.portable and no OpenRB-338.toml; unzipping a new zip over an installation leaves its OpenRB-338.toml untouched; the installer ships neither
- [ ] #4 README.txt, docs/building.md, docs/ui_spec.md and docs/plugin.md describe the marker, upgrading, and turning portable mode on and off
- [ ] #5 cargo test --workspace, clippy -D warnings (macOS and Windows) and nightly fmt --check pass
- [ ] #6 The user has checked on Windows: a portable run, then unzipping a newer zip over it keeps the settings
<!-- AC:END -->
