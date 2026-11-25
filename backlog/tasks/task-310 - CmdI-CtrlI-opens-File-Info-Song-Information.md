---
id: TASK-310
title: Cmd+I / Ctrl+I opens File > Info (Song Information)
status: Done
assignee:
  - '@agent-worker'
created_date: '2026-11-25T15:40:00Z'
updated_date: '2026-11-25T15:54:00Z'
labels:
  - ui
milestone: m-5
dependencies: []
references:
  - crates/orb-ui/src/menu.rs
  - crates/orb-ui/src/key_help.rs
  - docs/ui_spec.md
priority: medium
type: enhancement
ordinal: 281700
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Why:** the user asked on 2026-11-25 for a keyboard shortcut for File > Info, the "Song Information" window (docs/ui_spec.md around line 1723, task-095): Cmd+I on macOS, and Ctrl+I on Windows and Linux, following the menu's existing Cmd/Ctrl convention (e.g. Ctrl+O, Cmd+O).

**Details:**
- The menu item shows the shortcut text the way the other File items do.
- The shortcut opens the window, or brings it to the front, exactly as the menu item does. It is disabled when the item is disabled (no song).
- Plain I stays a panel key (crates/orb-ui/src/keys.rs and key_help.rs: the pattern-select keys). Only the modified key is new. Check that Cmd/Ctrl+I does not also trigger the panel key, and that it is not taken while a text field has the focus, e.g. the song information's own fields (follow how Cmd+O or Cmd+S behave there).
- Add it to Help > Keyboard Shortcuts (key_help.rs) and to docs/ui_spec.md (the menu table and the shortcut list).
- The plugin's editor has no File > Info, or no shortcut texts (ui_spec section 20). Check which, and keep the plugin as it is.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Cmd+I (macOS) / Ctrl+I (Windows, Linux) opens File > Info as the menu item does; the item shows the shortcut; disabled without a song
- [x] #2 Plain I still works as the panel key, and the shortcut does not fire it or steal it from a focused text field; tests cover both
- [x] #3 Help > Keyboard Shortcuts and docs/ui_spec.md list it
- [x] #4 cargo test --workspace, clippy -D warnings (macOS and Windows) and nightly fmt --check pass
- [x] #5 The user has checked it in the app
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. shortcuts::INFO (COMMAND+I). 2. menu.rs File > Info item shows it, enabled only with a song (plugin: always, no text). 3. app.rs consumes it next to New/Open/Save, before the panel keys (so plain I, and text fields, stay as for Cmd+O). 4. key_help.rs row; docs/ui_spec.md menu table, behavior table, info focus list. 5. Tests: shortcut opens window and does not fire panel I; plain I still panel key; works with a focused field; menu shows text.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: worktree-agent-a06b94ef243edfa6e

Plugin: File > Info exists there (task-201) without shortcut texts; kept: no text, item stays enabled. App consumes Cmd/Ctrl+I in App::show beside New/Open/Save, before the panel keys, so it is not the plain I (pattern 8 of row 2) and works with a focused field like Cmd+O. Test ctrl_i_opens_the_info_window_and_plain_i_stays_a_panel_key (tests/menu.rs). cargo test --workspace: 1181 pass, 0 fail; clippy macOS and xwin Windows clean; nightly fmt clean. AC 5 left for the user's check.

The user signed off in the dev app built from bbd464c on 2026-11-25.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Cmd+I / Ctrl+I opens File > Info (shortcuts::INFO, menu item text, disabled without a song, App::show, Help > Keyboard Shortcuts row, docs/ui_spec.md). Plain I stays the panel key; the shortcut works with a focused Info field; the plugin is unchanged. Branch worktree-agent-a06b94ef243edfa6e, one commit. Tests: cargo test --workspace 1181 pass; clippy macOS and Windows; nightly fmt clean. The user checked it in the app on 2026-11-25 (AC 5).
<!-- SECTION:FINAL_SUMMARY:END -->
