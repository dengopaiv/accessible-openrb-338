---
id: TASK-284
title: >-
  Keyboard Shortcuts help: the user's review cuts (no menu repeats, no
  mouse-only rows, no TR-/TB- names)
status: Done
assignee:
  - '@agent-worker'
created_date: '2026-11-22T16:51:00Z'
updated_date: '2026-11-24T08:51:00Z'
labels:
  - ui
milestone: m-9
dependencies:
  - TASK-281
references:
  - crates/orb-ui/src/key_help.rs
  - docs/ui_spec.md
priority: medium
type: enhancement
ordinal: 260250
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Why:** the user reviewed Help > Keyboard Shortcuts (crates/orb-ui/src/key_help.rs) on 2026-11-22, after the 909 editor (task-281) merged, and asked for these changes. The user is the first reviewer of the result (they check it in the app before the agent review).

**Remove:**
1. The note "Keys with the command key held are left to the menu."
2. The row "Cmd+E: Opens the focused row pattern editor (303, 808, 909)". The Edit menu shows the shortcut already.
3. Cmd+E and Cmd+Z from every pattern editor group: menu shortcuts are not repeated there.
4. The note "Both keyboard modes, whatever row has the focus."
5. The note "egui cannot tell the numpad from the main keyboard, so these keys are resolved by the mode and the focus."
6. The row "Numpad *: Nothing: egui gives no key for it (Insert records)".
7. The note "Every key and mouse action of the editor's window; the panel's keys work as without it, except while a value is typed." (it was in both the 808 and 909 groups).
8. The row "Shift / Cmd / Option / Ctrl + click or drag: The same as without: the editor reads no modifier".
9. Every DragValue row: "Drag up / down: On the pattern number or Length: change it" and any other row describing dragging, typing into or Shift+dragging a DragValue (the pattern number, Length, Shuffle and the like). egui's built-in DragValue behavior (Shift+drag is 1/10 speed) is not listed.
10. The "808 Pattern Editor" and "909 Pattern Editor" groups entirely.
11. The "The panel's TR-909 step buttons" group.

**Rename:**
- The group title "Always active" becomes "Global".
- Drop the TR- and TB- prefixes everywhere in the help: "TB-303 #1" becomes "303 #1", "TR-808" becomes "808", "TR-909" becomes "909", and so on in titles, rows and notes. The user's rule: no direct references to the original Roland hardware. This task covers the help's text. Other UI text keeps its names unless the user widens the rule; list any TR-/TB- text found elsewhere in the UI in the notes for the user.

**Keep:** every other row, including the 303 Pattern Editor group's own keys (minus Cmd+E and Cmd+Z).

**Tests and docs:** update key_help.rs's tests to the new groups. Add tests that would fail if a removed row, note or group came back or a TR-/TB- name reappeared. Update docs/ui_spec.md (the Help > Keyboard Shortcuts section and wherever sections 24 and 25 or the panel's 909 step buttons point at the removed groups).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Each of the 11 removals is gone from Help > Keyboard Shortcuts
- [x] #2 "Always active" reads "Global" and no TR- or TB- prefix appears in the help's text
- [x] #3 key_help tests pass and guard the removals and the renames; docs/ui_spec.md matches
- [x] #4 TR-/TB- text elsewhere in the UI is listed in the notes, unchanged
- [x] #5 cargo test --workspace, clippy -D warnings and nightly fmt --check pass
- [x] #6 The user has checked the help in the app
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Cut the groups, rows and notes in key_help.rs; drop Step909 and Row.editor (the editors' Input stays in drum_editor.rs for tests/drum_editor.rs); rename Always active to Global and TR-/TB- to 808/909/303 in the help; replace the editor/step-group tests by one guard test; update docs/ui_spec.md sections 7, 18, 24, 25.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: worktree-agent-ac7ea56519d503955 (rebased on main 856b7e4).

Done: all 11 removals; 'Menu shortcuts' keeps Undo (Cmd+Z) as the menu's own row, the pattern editor row (Cmd+E) is gone; empty notes are not drawn. Option+Up row of task-285 kept. The task says to keep 'the 303 Pattern Editor group's own keys', but key_help has no such group (it never had one), so nothing to keep there. Test guard: key_help::tests::the_users_cuts_and_renames_hold (group titles, notes, rows, no click/drag/hold/Length rows, no TR-/TB- text, renamed rows, Option+Up). Removed tests each_drum_editor_group_lists_every_input_of_its_editor and the_panels_909_step_group_names_the_keys_that_do_it with their groups; the editor inputs are still run by tests/drum_editor.rs and the step modifiers by behavior.rs / tests/interaction.rs. cargo test --workspace: 1069 passed, 0 failed; clippy -D warnings and nightly fmt --check clean. Dev app: target/OpenRB-338.app in the worktree.
TR-/TB- text elsewhere, unchanged (for task-286 / the user): crates/orb-ui/src/menu.rs:155,157 (ROW_NAMES 'TB-303 1', 'TB-303 2', 'TR-808', 'TR-909'; ROW_KINDS; these show in status messages like 'TR-909: A1 copied', 'The clipboard holds a TB-303 pattern', and in the clipboard JSON marker 'OpenRB-338 pattern: TB-303 1 A1'); crates/orb-ui/src/drum_editor.rs:165-166 (Machine::title 'TR-808'/'TR-909', the editor window titles) and :454,:459 (Input::OpenKey text); crates/orb-cli/src/render.rs:115,118 (CLI text 'TR-808', 'TR-909'). Tests that assert these strings: tests/wired.rs, tests/menu.rs, tests/drum_editor.rs, drum_editor.rs:1331, orb-plugin tests/editor.rs:455, orb-cli tests/standard_mod.rs:223. Process note: I used a python script for the key_help.rs bulk cut, against the Edit/Write rule.

The user checked the help in the app on 2026-11-24 and signed it off: good.

Review round 1 (send back, stale references): ui_spec.md 2353 and 2424 no longer name the removed key_help tests; the doc comments of drum_editor.rs Input and tests/drum_editor.rs no longer say the help lists the inputs; the coverage of the removed test (OpenKey keys = shortcuts PATTERN_EDITOR text, Undo = UNDO text, both platforms, both machines) is in drum_editor.rs the_help_texts_name_the_platforms_keys.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
The help lists keys only: the 11 cuts and the renames (Global, 303/808/909) are in crates/orb-ui/src/key_help.rs, guarded by the test the_users_cuts_and_renames_hold; docs/ui_spec.md sections 7, 18, 24, 25 follow. Branch worktree-agent-ac7ea56519d503955, one commit (amended). cargo test --workspace 1069 passed, clippy -D warnings and nightly fmt --check clean. Deviations: key_help had no 303 Pattern Editor group, so nothing to keep there; the bulk cut was done with a Python script, against the Edit/Write rule. TR-/TB- text elsewhere is listed in the notes, unchanged, for task-286. The user signed it off 2026-11-24.
<!-- SECTION:FINAL_SUMMARY:END -->
