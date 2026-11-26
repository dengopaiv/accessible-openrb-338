---
id: TASK-314
title: Pattern editors follow the device's selected pattern in song mode too
status: To Do
assignee:
  - '@agent-worker'
created_date: '2026-11-26T07:32:00Z'
labels:
  - ui
milestone: m-5
dependencies: []
references:
  - crates/orb-ui/src/pattern_editor.rs
  - crates/orb-ui/src/drum_editor.rs
  - crates/orb-ui/src/menu.rs
  - docs/ui_spec.md
priority: medium
type: enhancement
ordinal: 285700
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Why:** the user asked (2026-11-26) that the pattern editors (the two 303 editors, the 808's and the 909's) follow the active pattern in song mode too, exactly as in pattern mode: the editor's pattern is its device's selected pattern on the panel and vice versa. Today song mode is an exception (docs/ui_spec.md section 23 "Pattern selection", and the same paragraph in sections 24 and 25): the editor keeps its own pattern, a bank/number/arrow in it never selects on the panel, and the pattern the panel plays does not move it. The user also expects this to remove the special handling the exception needed in the Edit menu's pattern commands (copy, paste, clear, cut, duplicate, ...).

**Where the exception lives (start here, then grep `song_mode` in crates/orb-ui/src):**
- crates/orb-ui/src/pattern_editor.rs `editor_follow` (~line 393) and `editor_select` (~line 559): both skip the panel in song mode.
- crates/orb-ui/src/drum_editor.rs ~line 669 (`drum_follow`) and ~line 727 (the drum editors' select): the same.
- crates/orb-ui/src/menu.rs `pattern_target` (~line 1089) and `editor_pattern`: the commands target the focused editor's own pattern, because in song mode it could differ from the panel's; Duplicate (~line 1325) moves the editor itself "in song mode too, where it does not follow the panel's selection". With one rule for both modes the editor's pattern always equals the row's selected pattern, so this code should collapse: keep only what is still needed (e.g. which ROW a command targets: the focused editor's row, else the panel's focused row) and delete the rest.
- docs/ui_spec.md sections 23, 24 and 25 ("Pattern selection") and anything else describing the song-mode independence (grep "independent", "never records", "does not follow"), plus the doc comments in the code.

**The rule after the change, in both modes:** a bank, number or arrow in an editor selects that pattern of its device exactly as a click on the panel's pattern button does (`App::select_pattern`), whatever that click does in song mode today; and any new selection of the device's row on the panel (a click, a Select Patterns key, the engine, song playback changing the active pattern) moves the editor. Do not invent new song-mode behavior for the editor: it acts as the panel button. If what a panel pattern-button click does in song mode (for example recording) makes this wrong or surprising, stop and report it to the supervisor rather than deciding.

The plugin has no song mode, so it should be unaffected; check that its tests still pass.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 In song mode, choosing a pattern in any of the four editors selects it on the panel as a click on its pattern button does, and a new selection of that device on the panel (including the one song playback makes) moves the editor; tests cover both directions for a 303 editor and a drum editor
- [ ] #2 The song-mode branches in the editors' follow/select code are gone; one rule serves both modes
- [ ] #3 The Edit menu's pattern commands no longer special-case an editor pattern that differs from the panel's selection (pattern_target, editor_pattern, Duplicate's editor update are simplified or removed); their existing tests pass, rewritten only where they asserted the old song-mode independence
- [ ] #4 docs/ui_spec.md sections 23, 24, 25 and every other place describing the song-mode independence, and the code's doc comments, describe the new rule
- [ ] #5 cargo test --workspace, clippy -D warnings (macOS and Windows via cargo xwin) and nightly fmt --check pass
<!-- AC:END -->
