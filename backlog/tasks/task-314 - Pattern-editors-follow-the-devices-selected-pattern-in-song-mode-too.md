---
id: TASK-314
title: Pattern editors follow the device's selected pattern in song mode too
status: Done
assignee:
  - '@agent-worker'
created_date: '2026-11-26T07:32:00Z'
updated_date: '2026-11-26T08:15:00Z'
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
- [x] #1 In song mode, choosing a pattern in any of the four editors selects it on the panel as a click on its pattern button does, and a new selection of that device on the panel (including the one song playback makes) moves the editor; tests cover both directions for a 303 editor and a drum editor
- [x] #2 The song-mode branches in the editors' follow/select code are gone; one rule serves both modes
- [x] #3 The Edit menu's pattern commands no longer special-case an editor pattern that differs from the panel's selection (pattern_target, editor_pattern, Duplicate's editor update are simplified or removed); their existing tests pass, rewritten only where they asserted the old song-mode independence
- [x] #4 docs/ui_spec.md sections 23, 24, 25 and every other place describing the song-mode independence, and the code's doc comments, describe the new rule
- [x] #5 cargo test --workspace, clippy -D warnings (macOS and Windows via cargo xwin) and nightly fmt --check pass
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Remove the song-mode branches in editor_follow/editor_select/drum_follow/DrumAct::Select.
2. pattern_target: row = focused editor else panel focus, pattern = panel row's selected pattern; delete editor_pattern and Duplicate's editor update.
3. Rewrite the two song_mode_selections_are_independent tests as both-ways tests; fix other tests that relied on independence.
4. Update docs/ui_spec.md sections 23-25 and comments.
5. cargo test, clippy (mac + xwin), nightly fmt.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: worktree-agent-a764b3e9ee57ffd0e (rebased on 5c990b8). Found: a panel pattern click in song mode emits Event::Pattern (model.rs press_pattern), the same as pattern mode; Link::forward sends it and, with record on, the engine records it. So the editor now records like the panel button; no surprising behavior beyond that (reported to supervisor). Panel row follows engine pattern in song mode via adopt_pattern (link.rs follow_values), so the editor follows song playback through the panel row.

Done in code: song-mode branches removed (editor_follow, editor_select, drum_follow, DrumAct::Select); pattern_target = focused editor's row else panel focus, pattern = panel row's; editor_pattern and Duplicate's editor update deleted. Tests: the two song_mode tests rewritten as both-ways tests (303 and 808; the 909 shares drum code); OpenKey and playing-step drum tests adjusted (they set up an editor differing from the panel). cargo test --workspace --no-fail-fast: all pass; clippy -D warnings (mac, and cargo xwin x86_64-pc-windows-msvc) clean; nightly fmt --check OK. Not tested: engine-driven pattern change in song playback moving the editor (it goes through the panel row, Link::follow_values adopt_pattern, unchanged); the plugin's tests pass in the workspace run. docs/ui_spec.md sections 23, 24 updated, Duplicate row updated. Left uncommitted and To Do, awaiting the user's UI check.

Decision (user, 2026-11-26, after checking the build: "good"): in song mode with record on, an editor's selection records as a panel click does; accepted.

Superseded: the earlier note "Left uncommitted and To Do, awaiting the user's UI check" no longer holds; the user approved the build and the task is Done (commit amended). Review round 1 (reviewer sent back): fixed the stale "in pattern mode" wording in docs/ui_spec.md (section 23 top row, and the focused-editor paragraph near line 1466 of section 18), and in the doc comments of Act::Select and DrumAct::Select. The two tests that ended in two h.calls(4) were NOT reduced to one: the second call is needed (with one, both the_playing_step_is_shown_in_the_pattern_it_plays tests fail; the stop reaches the panel's shown step one call later), so the pairs are not duplicates and stay. The 'another pattern shown: no playing step' case was dropped earlier because a song-mode editor now always shows the panel's pattern, so it is unreachable. Checks: cargo test -p orb-ui passes, clippy -D warnings clean, nightly fmt --check OK; remaining "in pattern mode" greps in the three files are about other features (original's disabled items, Duplicate's bar-line timing).
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Pattern editors (both 303s, 808, 909) follow their device's selected pattern both ways in song mode too, as in pattern mode. Song-mode branches removed from editor_follow, editor_select, drum_follow and DrumAct::Select; pattern_target reduced to the focused editor's row (else the panel's focused row) and that row's selected pattern; editor_pattern and Duplicate's editor update deleted. docs/ui_spec.md sections 23 and 24 and the Duplicate row updated. Tests: song_mode_selections_follow_each_other_both_ways for a 303 and the 808 (panel and editor, both directions); OpenKey and playing-step drum tests adjusted. AC 1 note: following song playback is confirmed by the user's manual check and goes through the unchanged adopt_pattern path, with no automated test. cargo test --workspace --no-fail-fast passes, clippy -D warnings on macOS and cargo xwin, nightly fmt --check OK. User accepted recording of editor selections in song mode with record on (2026-11-26). Branch worktree-agent-a764b3e9ee57ffd0e.
<!-- SECTION:FINAL_SUMMARY:END -->
