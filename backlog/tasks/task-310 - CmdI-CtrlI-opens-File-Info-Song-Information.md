---
id: TASK-310
title: Cmd+I / Ctrl+I opens File > Info (Song Information)
status: To Do
assignee:
  - '@agent-worker'
created_date: '2026-11-25T15:40:00Z'
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
- [ ] #1 Cmd+I (macOS) / Ctrl+I (Windows, Linux) opens File > Info as the menu item does; the item shows the shortcut; disabled without a song
- [ ] #2 Plain I still works as the panel key, and the shortcut does not fire it or steal it from a focused text field; tests cover both
- [ ] #3 Help > Keyboard Shortcuts and docs/ui_spec.md list it
- [ ] #4 cargo test --workspace, clippy -D warnings (macOS and Windows) and nightly fmt --check pass
- [ ] #5 The user has checked it in the app
<!-- AC:END -->
