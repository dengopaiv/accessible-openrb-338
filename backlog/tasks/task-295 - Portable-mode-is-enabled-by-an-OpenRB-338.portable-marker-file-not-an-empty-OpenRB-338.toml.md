---
id: TASK-295
title: >-
  Portable mode is enabled by an OpenRB-338.portable marker file, not an empty
  OpenRB-338.toml
status: Done
assignee:
  - '@agent-worker'
created_date: '2026-11-24T16:47:00Z'
updated_date: '2026-11-25T07:20:00Z'
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
- [x] #1 With OpenRB-338.portable present the app and the plugin use the local OpenRB-338.toml, creating it from the per-user settings (writing nothing per-user) when absent; with no marker they use the per-user file even if a local OpenRB-338.toml exists; tests cover each case
- [x] #2 The Windows zip ships OpenRB-338.portable and no OpenRB-338.toml; unzipping a new zip over an installation leaves its OpenRB-338.toml untouched; the installer ships neither
- [x] #3 README.txt, docs/building.md, docs/ui_spec.md and docs/plugin.md describe the marker, upgrading, and turning portable mode on and off
- [x] #4 cargo test --workspace, clippy -D warnings (macOS and Windows) and nightly fmt --check pass
- [x] #5 The user has checked on Windows: a portable run, then unzipping a newer zip over it keeps the settings
- [x] #6 The standalone_version and plugin_version keys are removed (no compatibility code; a file that still has them loads, the keys dropped at the next save); a search of the repository finds neither outside backlog/
- [x] #7 task-289's empty-file rule is gone, with no compatibility code; atomic writes stay
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. portable.rs: MARKER_NAME, folder() keyed on the marker.
2. settings.rs load_resolved: no marker = per-user; marker + absent toml = copy per-user in; marker + present toml = as is (damaged = error box); drop the empty-file rule.
3. tests/portable.rs rewritten for the cases; package.sh ships OpenRB-338.portable, no toml; README, building.md, ui_spec.md, plugin.md.
4. clippy mac + xwin, tests, package, test-machine run.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: worktree-agent-ad95bedadf5598e73 (rebased on 53ac7b4).

Evidence: cargo test --workspace 1142 passed 0 failed; orb-ui portable tests 11 pass; nightly fmt --check ok; clippy macOS ok; cargo xwin clippy --workspace --all-targets x86_64-pc-windows-msvc ok (PATH with llvm, VCINSTALLDIR=/xwin as task-193 did).
Zip: scratchpad dist295/x86_64-pc-windows-msvc/OpenRB-338-2026.1124.2018-x86_64-pc-windows-msvc.zip: LICENSE, OpenRB-338.exe, OpenRB-338.portable, OpenRB-338.vst3, README.txt, THIRD-PARTY-NOTICES.txt, orb.exe; no OpenRB-338.toml; installer script (scripts/installer/openrb.nsi) mentions neither file.
Test machine (all 6 permits held, released): unzip to C:/re/rb/p295/x, run OpenRB-338.exe 10 s: OpenRB-338.toml (253 bytes) appeared next to the exe; wrote a test OpenRB-338.toml, unzipped the same zip over the folder: toml byte-identical (cmp), marker still there; run again: toml still held the test settings. Not checked: the plugin in a host; the AC 6 check is the user's.

Scope addition (user, 2026-11-25): removed the standalone_version / plugin_version keys. task-289 added them only so a saved file was never blank (an empty file meant portable mode); the marker replaces that rule, and they never shipped (latest release v2026.1118.0919). settings.rs: the Role enum, the Settings.role field, the two Stored fields, the "keep the other key" merge and load_plugin() are gone (the plugin calls Settings::load()); load_resolved(local, user) lost its role argument. The loader reads a toml table key by key and ignores unknown keys (module doc), so a dev-build file with the keys loads and the next save drops them: no special handling, a test covers it with a generic unknown key. Atomic save kept. Docs: building.md, plugin.md, ui_spec.md, settings.rs module comment. A repository search for the two names outside backlog/: 0 hits.

Review round 1 (3 defects, fixed in the one commit): the save doc comment in settings.rs now describes the atomic, merging save (the version-key sentence is gone); the portable.rs test header no longer mentions version keys, and the test is renamed a_save_is_atomic_and_leaves_no_temporary_file; test count correction: the full-workspace run on this branch is 1141 passed, 0 failed (the "1142" in the earlier Evidence note was the run before the keys were removed and the plugin-copy test dropped; 1141 is the count of the final tree). orb-ui tests, clippy (macOS, x86_64-pc-windows-msvc) and nightly fmt --check pass after the fix.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Portable mode on Windows is now turned on by an OpenRB-338.portable marker file in the program's folder (app and plugin), not by an empty OpenRB-338.toml. With the marker: the local OpenRB-338.toml is used as is (damaged = error box and defaults), or created from the per-user settings when absent (nothing written per-user). Without the marker: per-user settings, a local toml ignored. task-289's empty-file rule is gone, with no compatibility code. The Windows zip ships the marker and no OpenRB-338.toml, so unzipping a new version over a portable folder keeps its settings; the installer ships neither. README.txt, docs/building.md, docs/ui_spec.md, docs/plugin.md updated.
Scope addition (user): the unreleased standalone_version / plugin_version keys are removed (Role enum, load_plugin and the key merge gone; a file that still has them loads, the next save drops them). Atomic save kept.
Branch worktree-agent-ad95bedadf5598e73, one commit. Tests: cargo test --workspace 1141 passed, 0 failed; clippy macOS and x86_64-pc-windows-msvc, nightly fmt --check clean. Test machine (earlier zip): portable start created the toml, unzip over kept it byte for byte. The user checked OpenRB-338-2026.1125.0711 on Windows (2026-11-25): portable run, then unzip over keeps the settings.
<!-- SECTION:FINAL_SUMMARY:END -->
