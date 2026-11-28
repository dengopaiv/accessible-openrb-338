---
id: TASK-249
title: File > New from the menu opens Save As at once instead of asking
status: Done
assignee:
  - '@agent-worker'
created_date: '2026-11-28T06:51:00Z'
updated_date: '2026-11-28T09:03:00Z'
labels:
  - ui
milestone: m-7
dependencies: []
references:
  - crates/orb-ui/src/menu.rs
  - crates/orb-ui/src/app.rs
priority: high
type: bug
ordinal: 266000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Why: found by the user in the NVDA pass of 2026-11-28 (TASK-244): "ctrl+n asks whether you want to save, if there are changes. File new just opens the save dialogue if there are changes." Asked what happens, the user answered that with unsaved changes, File > New chosen from the menu brings up the Windows Save As dialog at once. Wanted: File > New from the menu asks the same question as Ctrl+N (Save changes?), and only the user's answer to it saves, discards or cancels.

What is known: Ctrl+N (app.rs, shortcuts::NEW) and the menu item (menu.rs, item "New" -> Cmd::New) both call App::new_song, which sets self.prompt = Some(Then::New) when asks_to_save(); the unsaved-changes prompt (docs/ui_spec.md 18 and 20.4) takes S / Y / Enter as Save, D / N as Don't save, Escape as Cancel, and in screen reader mode moves the focus to Save on its first frame (App::prompt_focused). The supervisor's hypothesis, not checked: the Enter press that chose the menu item is read again by the prompt drawn later in the same frame (or the next), so the prompt answers Save at once, and Save on an untitled song opens Save As. Ctrl+N has no Enter, so it is not hit. task-212's focus back after a menu item (App::menu_bar_focus, menu_return) may take part too.

The same can hit every menu item that opens a question, which is worse where Enter means a destructive answer: Edit > Initialize Song from Pattern Mode and Initialize Loop (their question takes Enter as Initialize, docs/ui_spec.md 18), File > Open and Mods > Demo Song when they ask to save, Quit / close. Check each, in both modes (the menu bar is walked from the keyboard in both modes, F10, task-184), and fix the cause once: the key that chose a menu item must not reach the question it opens. A question opened from the menu must wait for a new key press.

Reproduce first with a headless test (crates/orb-ui/tests, the a11y harness AppTree; F10, the arrows and Enter as the user does), then fix.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 A test reproduces it: with unsaved changes, File > New chosen with Enter from the menu leaves the unsaved-changes question open (no Save As, no save), in both modes; then S / Enter answers it as before
- [x] #2 Every menu item that opens a question (Initialize Song and Loop from Pattern Mode, Open, Demo Song, and any other found) is tested the same way: the Enter that chose it does not answer it
- [x] #3 Ctrl+N and the other shortcuts behave as before (existing tests pass); docs/ui_spec.md where the questions are described, CHANGELOG.md's next section, docs/accessibility_plan.md and docs/accessibility.md updated; one by-ear criterion added to TASK-244
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Test first: crates/orb-ui/tests/a11y_menu_question.rs walks F10/arrows/Enter in both modes for File > New and Edit > Initialize Song; confirm it fails. 2. Fix once: a question (prompt_ui, init_song_ui) ignores keys on the frame it is first drawn and ignores key repeats, so the Enter that chose the item cannot answer it. 3. Check Open (browser Enter) and Demo Song paths. 4. Docs: ui_spec, CHANGELOG, accessibility_plan, accessibility.md, TASK-244 criterion 10.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: worktree-agent-acf1d9533ad445dc1

Reproduced first: tests/a11y_menu_question.rs (F10, arrows, Enter through the AccessKit tree; both modes) failed 5 of 5 on main's code: the question was answered at once (prompting false after the Enter). Cause, as the supervisor guessed: menu_bar runs before prompt_ui in the same frame, the Enter that chose the item is still in the input, and prompt_ui / init_song_ui consume it as Save / Initialize. Fix once: answer_key (menu.rs) takes a question's keys only from its second frame (App::prompt_armed, init_armed) and only as new presses (no repeat). After the fix: 6 pass (File > New: stays open, D answers, a second Enter saves as before, Escape cancels; Initialize Song: stays open, a new Enter initializes, Escape cancels; Open in the in-app browser, mode off). Checked and no change needed: File > Open in the browser (the browser consumes its Enter; the test is a guard, it passed also with the arming disabled); in screen reader mode Open uses the system dialog, no Enter. Not tested headless: Mods > Demo Song (needs a mod kit with Demo.rbs) and the window close (no menu item): both go through the same prompt_ui and its arming. Initialize Loop asks nothing. Screen reader mode off: the app is upstream's except this bug fix (named in CHANGELOG). TASK-244 criterion 10 added (by ear, checklist item 38). Commands: cargo test --release -p orb-ui --test a11y_menu_question; cargo test --release --workspace (0 failed); clippy -D warnings clean; nightly fmt applied.

Review round 1 (send back), fixed: (a) ui_spec no longer claims the browser; the browser's Enter is consumed by it. (b) Stale arm: prompt_armed and init_armed are now also cleared by an answer (answer(), the init answer); two tests answer a question and open the next in the very next frame with an Enter (they fail without the resets, checked by removing them). (c) New tests, file now has 11: S answers Save from the menu; Mods > Demo Song (Cmd::DemoSong with doc.kit set to a Kit::Mod, so no real mod is needed; the question is asked before the mod is read) and the window close (request_close) with an Enter in the opening frame; File > Open in screen reader mode with a stand-in picker (set_picker), the Enter in the frame that opens the question. AC 2 stays ticked: every menu path that opens a question is covered (New, Initialize Song, Open both modes, Demo Song, Quit); Demo and Quit are driven through the same App calls the menu item and close request use, not through a drawn menu item, because the Demo item needs a real mod with a Demo.rbs. (d) Correction of the earlier note: the first run failed 5 of 6 tests of the then 6-test file, not 5 of 5. Totals: orb-ui 753 passed, workspace 1226 passed, 0 failed, 7 ignored; clippy -D warnings and nightly fmt --check clean.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
File > New and Edit > Initialize Song from the menu now open their question and wait for a new key press. Cause: the Enter that chose the item was still in the frame that draws the question and answered it (Save, so Save As on an untitled song; Initialize). Fix: menu::answer_key, with App::prompt_armed and init_armed (cleared by an answer too), takes a question's keys only from its second frame and only as new presses; both modes. Tests: crates/orb-ui/tests/a11y_menu_question.rs, 11 tests (File > New with Enter, S, Escape; Initialize Song; Open in the browser and with the system dialog; Demo Song and close; a question opened right after an answer); 5 of the first 6 failed on the old code. orb-ui 753 passed, workspace 1226 passed, 0 failed; clippy and fmt clean. Docs: ui_spec, CHANGELOG, accessibility_plan (item 38), accessibility.md; TASK-244 criterion 10. Branch worktree-agent-acf1d9533ad445dc1, one commit.
<!-- SECTION:FINAL_SUMMARY:END -->
