---
id: TASK-249
title: File > New from the menu opens Save As at once instead of asking
status: To Do
assignee: []
created_date: '2026-11-28T06:51:00Z'
updated_date: '2026-11-28T06:55:00Z'
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
- [ ] #1 A test reproduces it: with unsaved changes, File > New chosen with Enter from the menu leaves the unsaved-changes question open (no Save As, no save), in both modes; then S / Enter answers it as before
- [ ] #2 Every menu item that opens a question (Initialize Song and Loop from Pattern Mode, Open, Demo Song, and any other found) is tested the same way: the Enter that chose it does not answer it
- [ ] #3 Ctrl+N and the other shortcuts behave as before (existing tests pass); docs/ui_spec.md where the questions are described, CHANGELOG.md's next section, docs/accessibility_plan.md and docs/accessibility.md updated; one by-ear criterion added to TASK-244
<!-- AC:END -->
