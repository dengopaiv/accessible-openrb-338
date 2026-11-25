---
id: TASK-293
title: 'One name scheme: 303-1 / 303-2 everywhere, no ORB- prefix in plugin names'
status: Done
assignee:
  - '@agent-worker'
created_date: '2026-11-24T15:51:00Z'
updated_date: '2026-11-25T09:56:00Z'
labels:
  - ui
  - player
  - docs
milestone: m-9
dependencies:
  - TASK-290
references:
  - crates/orb-ui/src/menu.rs
  - crates/orb-ui/src/key_help.rs
  - crates/orb-plugin/src/params.rs
  - crates/orb-plugin/src/clip.rs
  - docs/ui_spec.md
  - docs/plugin.md
priority: medium
type: enhancement
ordinal: 267700
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Why:** the user decided on 2026-11-24, from task-290's survey, to use one scheme. The two 303s are "303-1" and "303-2" everywhere a user can read them. The drum machines stay "808" and "909". A pattern follows with a space: "303-1 A1". The plugin's "ORB-" prefix is dropped. task-290's notes hold the full survey table with file:line.

**What changes** (from that survey, the supervisor's quick grep, and your own grep for any other form):
- Status messages: `ROW_NAMES` in crates/orb-ui/src/menu.rs, "303 1" becomes "303-1", e.g. "303-1: A1 cleared".
- The Keyboard Shortcuts help, Select Patterns rows (key_help.rs around 497): "303 #1" becomes "303-1".
- The plugin's MIDI clip sections and track names (crates/orb-plugin/src/clip.rs SECTIONS, around 81 and 370): "303 #1" becomes "303-1". The drag-out file name, built from SECTIONS by replacing " #" with "-", must stay "303-1 A1.mid": simplify that code.
- The plugin's device names (crates/orb-plugin/src/params.rs `device_name`, around 270), used in parameter display names and in the VST3 parameter groups:
  - "ORB-303 1" becomes "303-1";
  - "ORB-303 2" becomes "303-2";
  - "ORB-808" becomes "808";
  - "ORB-909" becomes "909";
  so a parameter reads e.g. "303-1 Cutoff".
- The parameter string ids must not change: tests/param_ids.txt and the_ids_do_not_change stay green, so hosts keep their automation. This follows task-220's precedent, the last rename. Update tests/params.rs the_device_names_are_orb_names (rename it too).
- Every other user-visible form found: tooltips, window titles, menus, the CLI's text, the site (site/index.html), README.md.
- Docs that describe our app and plugin: docs/ui_spec.md, docs/plugin.md, docs/midi.md, docs/standard_midi_mapping.md ("303 #1", "303 1", "TB-303 #1" and "ORB-303 1" become "303-1"). The docs about the original program (docs/initial_investigation.md, docs/RBS42.txt, docs/song_load.md and the like) keep the original's names.

**Compatibility:** only what a release shipped matters; the latest is v2026.1118.0919.
- Parameter ids: unchanged, by the test.
- Display names, group names, bus names and clip names are labels: hosts store automation by id and routing by index.
- No settings key or saved state involves these names. Check and say so in the notes.

**Guard:** extend the existing name guard test (`no_hardware_names_in_user_visible_text`, crates/orb-ui/src/drum_editor.rs) or add one so "303 1", "303 #" and "ORB-" in the UI's text tables and the plugin's device and section names fail a test.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Every user-visible name of a 303 in the app, the plugin (parameters, groups, buses, MIDI clips and files), the CLI, the site, README and the docs of our app reads 303-1 / 303-2; no ORB- prefix remains in plugin names
- [x] #2 The parameter string ids are unchanged (param_ids.txt test green)
- [x] #3 A test guards against 303 1, 303 # and ORB- forms coming back
- [x] #4 cargo test --workspace, clippy -D warnings (macOS and Windows) and nightly fmt --check pass
- [x] #5 The user has checked the names in the app and in a host
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Code: ROW_NAMES, key_help rows, plugin SECTIONS/file_name (drop the replace), params device_name. 2. Tests: params names test renamed, clip tests, guard test extended. 3. Docs: ui_spec, plugin, midi, standard_midi_mapping, README, site. 4. Checks: test, clippy mac+windows, fmt; build app and bundle.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
2026-11-24, supervisor: from task-290's review, also fix the stale docs/ui_spec.md sentence near line 2468 ("The editor's selector only chooses what the editor shows"): the in-window 303 selector is gone since task-290.

Branch: worktree-agent-a0ee1908102eb95c9 (rebased on 559e6ab)

Names changed: status line ROW_NAMES 303 1 -> 303-1 (menu.rs); Keyboard Shortcuts help Select Patterns 303 #1 -> 303-1 (key_help.rs); plugin MIDI clip sections/track names 303 #1 -> 303-1 and the drag-out file name now built straight from SECTIONS (clip.rs, the replace is gone); plugin parameter names and VST3 groups ORB-303 1 -> 303-1, ORB-303 2 -> 303-2, ORB-808 -> 808, ORB-909 -> 909 (params.rs device_name; string ids untouched, the_ids_do_not_change green); README and site TB-303/TR-808/TR-909 -> 303/808/909; docs ui_spec, plugin, midi, standard_midi_mapping (303 #1, TB-303 #1, ORB-303 1 -> 303-1); the stale ui_spec sentence on the editor selector fixed. The CLI has no numbered 303 name. Settings: no key or saved state holds these names (ui.toml keys are pattern_editor_303_N_pos; plugin state holds ids and base64 bytes; hosts store automation by id), so no compatibility code. Guards: drum_editor.rs no_hardware_names_in_user_visible_text now also scans the 303 editor titles/menu items and rejects '303 1', '303 2', '303 #', 'ORB-'; plugin tests/params.rs no_orb_prefix_or_old_303_forms_in_names (every param name and group, SECTIONS). Checks: cargo test --workspace 1152 passed 0 failed; clippy -D warnings clean on macOS and x86_64-pc-windows-msvc; nightly fmt --check clean. Criterion 5 is the user's check, task left To Do.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
The two 303s read 303-1 and 303-2 everywhere a user sees them, and the plugin has no ORB- prefix. Before -> after: status line '303 1: A1 cleared' -> '303-1: A1 cleared'; Keyboard Shortcuts help '303 #1' -> '303-1'; plugin MIDI clip sections and track names '303 #1 A1' -> '303-1 A1' (the drag-out file name '303-1 A1.mid' is now built straight from SECTIONS); plugin parameter names and VST3 groups 'ORB-303 1 Cutoff' -> '303-1 Cutoff', 'ORB-303 2' -> '303-2', 'ORB-808' -> '808', 'ORB-909' -> '909'; README, site and the docs of the app and plugin TB-303/TR-808/TR-909, '303 #1', 'ORB-303 1' -> 303/808/909 and 303-1. Parameter string ids are unchanged (the_ids_do_not_change green). No settings key or saved state holds these names, so no compatibility code. Guards: drum_editor.rs no_hardware_names_in_user_visible_text (also scans the 303 editor titles) and plugin tests/params.rs no_orb_prefix_or_old_303_forms_in_names. Tests: cargo test --workspace 1152 passed, 0 failed; clippy -D warnings clean on macOS and x86_64-pc-windows-msvc; nightly fmt --check clean. Branch worktree-agent-a0ee1908102eb95c9, one commit. The user checked the names in the dev app and the plugin bundle built from f15309f on 2026-11-25 and signed off.
<!-- SECTION:FINAL_SUMMARY:END -->
