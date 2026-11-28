---
id: TASK-247
title: >-
  Menu bar: Left and Right move between the menus as in Windows, never out of
  the bar
status: Done
assignee:
  - '@agent-worker'
created_date: '2026-11-28T06:50:00Z'
updated_date: '2026-11-28T07:26:00Z'
labels:
  - ui
milestone: m-7
dependencies: []
references:
  - docs/ui_spec.md
  - crates/orb-ui/src/menu.rs
  - crates/orb-ui/tests/a11y_menu_bar.rs
priority: medium
type: bug
ordinal: 264000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Why: found by the user in the NVDA pass of 2026-11-28 (TASK-244, reported next to checklist item 26): "Left in the menus moves back to the menu bar, but doesn't allow to move then to the other menus. This should either be removed, or made consistent: up down move between menu items, left right move between menu columns ... One right from the help menu moves to the transport section, it shouldn't leave the menu like that." Asked how it should work, the user chose Windows' way (2026-11-28).

Today (docs/ui_spec.md 18.2): the menu bar's keys are egui 0.37's own (containers/menu.rs, popup.rs, MenuState::handle_keyboard). On a focused title Left / Right move egui's spatial focus to the neighboring title, without wrapping, and from the last title (Help) Right moves the focus out of the bar to a panel control (the Transport section). In an open menu Left returns the focus to the title, after which the other menus cannot be reached the same way. OpenRB-338 adds F10 / a lone Alt, Down opening a menu (open_on_down), the panel's keys withheld while the bar is in use (menu_has_keys, task-202) and, in screen reader mode, the focus back to the panel control when the bar is left (task-212, App::menu_bar_focus, menu_return). The bar has six titles: File, Edit, View, Mods, Options, Help.

Wanted, in screen reader mode (with the mode off the menus stay as they are, egui's, as upstream; the ground rule of docs/accessibility_plan.md Part 4):
- On a focused title: Left / Right move to the previous / next title, wrapping (Help -> File, File -> Help); the focus never leaves the bar for the panel.
- In an open menu: Up / Down walk its items as now; Left / Right close it and open the previous / next menu with the focus on its first item, wrapping the same way.
- A submenu (the Mods menu's merged names, whose files are a submenu): Right on its item opens it with the focus on its first file; Left inside it closes it and returns to its item; Right on a file of an open submenu goes on to the next menu.
- Escape, Enter, F10 and a lone Alt as today (task-212's focus back included).
NVDA should hear each title or item once per move (no double speech).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 In screen reader mode, Left and Right on a title and in an open menu go through all six menus in both directions, wrapping at File and Help, an open menu opening the next with the focus on its first item; covered by a11y tests
- [x] #2 Right from Help (its title or its open menu) goes to File, never to a panel control; a test
- [x] #3 Right on a Mods submenu item opens it, Left in it goes back to its item; a test (with a mods folder of merged names, as the existing Mods menu tests make)
- [x] #4 With the mode off the menu bar's keys are as before (the existing a11y_menu_bar and keyboard tests pass unchanged); docs/ui_spec.md 18.2, Help > Keyboard Shortcuts, CHANGELOG.md's next section, docs/accessibility_plan.md and docs/accessibility.md updated; one by-ear criterion added to TASK-244
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Failing a11y tests (title walk with wrap, open-menu walk, Help->File, Mods submenu, mode off unchanged, Escape focus back). 2. menu.rs bar_arrows: in screen reader mode, before the bar is shown, take Left/Right from egui's focus move and move the focus/open menu myself, except egui's Right on a submenu button and Left inside a submenu; submenu buttons noted per frame. 3. Docs: ui_spec 18.2 (six titles), key_help, CHANGELOG, accessibility_plan/accessibility, TASK-244 criterion 8 (item 36).
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Supervisor, 2026-11-28 (review of the pass record): docs/ui_spec.md 18.2's last bullet still says the bar has five titles; it has six since task-233 (View). Fix it with this task's 18.2 update.

Branch: worktree-agent-acc3320da63ed03fc

Evidence: crates/orb-ui/src/menu.rs bar_arrows takes Left/Right from egui's focus move (move_focus(None)) in screen reader mode only, then moves the focus to the neighbor title, or closes the open menu and opens the neighbor with the Down-on-title two-frame opening (orb_menu_down_pending). Submenu buttons are noted per frame (note_submenu) so egui keeps Right on one and Left inside its submenu. Tests: a11y_menu_bar.rs (5 new) and a11y_mod_chooser.rs (1 new) passed at first run; the mode-off test shows egui's no-wrap and exit to the panel are unchanged. cargo test --release -p orb-ui --test a11y_menu_bar --test a11y_mod_chooser: 18 + 14 pass. Workspace: 1206 pass, 0 fail (gated CD tests skip). clippy -D warnings and nightly fmt --check clean. Not verifiable by agents: NVDA speech; added as TASK-244 criterion 8 (checklist item 36). Edit without a song has all items disabled, so a move to it leaves the focus on its title (documented in ui_spec 18.2).

Review round 1: (1) the 'no song' wording is now 'the demo panel (no player link)' in ui_spec 18.2 and the test comment; (2) ui_spec 18.2 now says a move between titles is one focus move and a move between open menus is title in one frame, first item in the next (open_on_down's two-frame opening), speech to be confirmed in item 36; (3) plan Part 1 row says (see git log), Part 1.2 intro explains why rows from TASK-246 on carry no hash (no TASK-246 row names a commit); (4) plan state paragraph rewritten; (5) the subs remove_temp and note_submenu are gated on screen reader mode, so mode off touches nothing; (6) new test left_from_file_wraps_to_help: it FAILS without bar_arrows (call disabled: panicked at a11y_menu_bar.rs:513) and passes with it. cargo test --release -p orb-ui: 734 pass, 0 fail; clippy -D warnings and nightly fmt --check clean.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
In screen reader mode Left and Right in the menu bar work as in Windows: on a title they go to the previous or next title with wrap (File<->Help), never to the panel; in an open menu they open the neighbor menu with the focus on its first item; in the Mods submenu Right opens it, Left returns to its item, Right on a file goes to the next menu. Code: menu.rs bar_arrows, note_submenu. Docs: ui_spec 18.2 (six titles), key_help F10 row, CHANGELOG, accessibility_plan (rows, item 36), accessibility.md; TASK-244 criterion 8 added. Tests: 6 new a11y tests, cargo test --release --workspace 1206 pass, clippy and fmt clean. Branch worktree-agent-acc3320da63ed03fc, one commit.
<!-- SECTION:FINAL_SUMMARY:END -->
