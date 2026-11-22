---
id: TASK-282
title: Keyboard Shortcuts help lists every key and mouse modifier in every context
status: To Do
assignee:
  - '@agent-worker'
created_date: '2026-11-22T09:49:00Z'
labels:
  - ui
  - docs
milestone: m-9
dependencies:
  - TASK-281
references:
  - crates/orb-ui/src/key_help.rs
  - crates/orb-ui/src/keys.rs
  - crates/orb-ui/src/shortcuts.rs
  - crates/orb-ui/src/controller.rs
  - crates/orb-ui/src/pattern_editor.rs
  - docs/ui_spec.md
priority: medium
type: task
ordinal: 260300
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Why:** the user requires (2026-11-22) that Help > Keyboard Shortcuts list every key that OpenRB-338 supports, in every context.
- Today crates/orb-ui/src/key_help.rs covers the panel's keyboard binds (`keys::map`, with tests that run each bind both ways) and the menu shortcuts (`crate::shortcuts`).
- It isn't known to cover:
  - keys and modifiers handled elsewhere: mouse modifiers on the panel (the 909 step modifiers, any Shift/Alt/Cmd drag or click behavior on knobs, switches and steps), Escape and Enter in windows and dialogs, the text fields;
  - the pattern editors' own keys and modifiers (303 from task-274; 808 and 909 from tasks 280 and 281, which add their own entries);
  - plugin-only differences, such as keys the host takes, or keys that differ in the plugin.

**What:**
- **Audit** every key and modifier the UI reacts to. Grep the input handling: `i.modifiers`, `key_pressed`, `consume_key`, `consume_shortcut`, `Event::Key`, egui's `Modifiers` in crates/orb-ui and crates/orb-plugin. List them in the task notes by context:
  - the panel in each keyboard mode and focused row;
  - each pattern editor;
  - each window and dialog (About, Keyboard Shortcuts, Info, file browser, unsaved-changes prompt, error box, mod selector, export, …);
  - the plugin.
- **Make the help list them all**, grouped by context, with platform-correct names (Cmd/Option on macOS, Ctrl/Alt elsewhere) and mouse modifiers written like "Cmd+click", "Shift+drag".
  - Mark where something differs in the plugin.
  - Don't list egui's built-in text-editing keys one by one. One row per text field kind is enough ("text fields: standard editing keys").
- **Tests:** extend the bind tests so a key or modifier that a handler checks but the help doesn't list fails a test, as far as the code structure allows. Where a check can't be automatic, a test lists the expected rows per context, so a removal shows. Record in the notes what can't be checked.
- **Docs:** docs/ui_spec.md (Help > Keyboard Shortcuts) describes the groups.

**Order:** after the 808 and 909 editors (tasks 280 and 281), so the audit covers them. Fix any entries they missed.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 The task notes list every key and modifier the UI handles, by context (panel modes and rows, each pattern editor, each window and dialog, the plugin), with file:line of each handler
- [ ] #2 Help > Keyboard Shortcuts lists all of them grouped by context, with platform-correct names and mouse modifiers, marking plugin differences
- [ ] #3 Tests fail when a handled key or modifier is missing from the help (as far as the code allows; the limits are recorded in the notes)
- [ ] #4 docs/ui_spec.md describes the help's groups; cargo test --workspace passes, clippy -D warnings and nightly fmt clean
- [ ] #5 The user reviewed the help window in the app and the plugin before the agent review
<!-- AC:END -->
