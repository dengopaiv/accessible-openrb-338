---
id: TASK-297
title: >-
  File dialog: Open accepts a typed full path like Enter does; the file list
  fills the window height
status: Done
assignee:
  - '@agent-worker'
created_date: '2026-11-24T18:26:00Z'
updated_date: '2026-11-25T08:28:00Z'
labels:
  - ui
milestone: m-5
dependencies: []
references:
  - crates/orb-ui/src/browser.rs
  - crates/orb-ui/tests/browser.rs
  - docs/ui_spec.md
priority: medium
type: bug
ordinal: 270700
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Why:** the user reported this on 2026-11-24 and is the first reviewer.

**1. The Open button does not take a typed path.** In the egui file dialog (crates/orb-ui/src/browser.rs) the user types or pastes a full file path into the dialog's text input:
- **Enter** opens the file correctly;
- the **Open** button shows "Select a file first.".

`Browser::target` (browser.rs around line 482) decides from the list's selected entry only, for the reading modes (OpenSong, ChooseMod). The Enter path goes through other code: `activate`/`accept` and the key handling around line 669. Find where Enter takes the typed text and make the button do exactly the same, in every mode the dialog has:
- a full path to a file opens it;
- a path to a folder goes there;
- a relative name resolves in the current folder.
Best done by one shared function that both call. The status-line error stays for a truly empty choice. Unit tests in crates/orb-ui/tests/browser.rs cover the button with a typed full path, a folder path and a relative name, next to the existing "Select a file first." test (line 150).

**2. The file list does not fill the window height.** The list leaves empty space below it inside the dialog window. Make the list take the window's remaining height, after the path row, the name row and the buttons. It should follow the window when the user resizes it, at any UI scale, in the app and the plugin. Keep task-285's row pitch and centering of the selected folder after Up working with the new height.

Update docs/ui_spec.md's file dialog section if it describes either point.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 The Open (and Save, Choose) button accepts a typed full file path, a folder path and a relative name exactly as Enter does; unit tests cover each
- [x] #2 The file list fills the dialog window's height with no empty space below it, following window resizes, in the app and the plugin; task-285's centering after Up still works
- [x] #3 cargo test --workspace, clippy -D warnings (macOS and Windows) and nightly fmt --check pass
- [x] #4 The user has checked both in the app
- [x] #5 The smallest main window (standalone viewport minimum and plugin editor minimum, one shared constant) holds every non-resizable window with the dialogs' margin; a test measures each at the minimum
- [x] #6 The file dialog's overwrite question has plain button captions (Replace, Back), the keys Y, Enter, Esc and N still work
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Browser::press_button: the main button runs go_to_text when the path bar holds a typed path (path_edited), else activate; same function the Enter key paths share. Selecting a row drops a typed path.
2. Height: the list height = screen less margins less the dialog's other rows (chrome_h measured in the previous frame). Root cause of the gap below the list: the button row used ui.with_layout(right_to_left), which takes all the height left in the window; now one row high. Places column in a ScrollArea of the list height (was list+60).
3. Tests in crates/orb-ui/tests/browser.rs (button with full path, folder, relative name, other modes, empty error, list fills height at 3 window heights); docs/ui_spec.md section 17.
4. fmt, clippy macOS and Windows, cargo test --workspace, build the dev app for the user.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: worktree-agent-ae344731103d1b1b2

Root cause of the gap: status_and_buttons used ui.with_layout(right_to_left), which takes all the height left in the window (the button row sat at the window bottom, 358 points under the list when the dialog was short). Fixed with a one-row allocate_ui_with_layout; the list height is now screen less margins less the other rows (chrome_h, measured last frame). Checks: cargo test --workspace 1147 pass 0 fail; clippy macOS and x86_64-pc-windows-msvc -D warnings clean; nightly fmt --check clean. New tests (tests/browser.rs): button with typed full file path, folder path, relative name (file and folder), Save/Open in other modes, empty error, list fills height at 400/510/900. The plugin shares browser.rs; its dialog was not run by hand. AC 4 is the user's check.

Review round 1 (user: keep the dialog's original height, only the list fits it). Reverted the screen-derived list height and the one-row button row. Measured the dialog at d84e0af (Open Song, 624 wide): window 300/400 -> 326 high, 510/600/900 -> 397. The list's nominal height (window-230 clamped 80..270) is unchanged; the Places column is taller than it on short windows (up to 199 vs 80/170), which left the gap below the list. Now the list takes max(nominal, Places column height). Dialog heights after: 326/326/397/397/397, identical (test asserts it). At the default 510 window the list was already full (270); the gap shows below about 500 points. Checks: cargo test --workspace 1147 pass, clippy macOS and xwin clean, fmt clean.

Scope addition (user, 2026-11-25): larger minimum window. Old minimum 312 x 277 (PANEL/2 + MENU_H, main.rs viewport and orb-plugin editor::resize_hint/adjust_logical). New 488 x 373 (layout::MIN_WINDOW_W/H, DIALOG_MARGIN 12). Measured at the minimum, default scale (the minimum is in points, ppp does not change it): file dialogs 464 wide x 326 (Open, Choose) / 349 (Save, Export); About 464 x 254; Audio Output 375 x 210; error box 404 x 98. Widest About 464 (+24 = 488), highest Save/Export 349 (+24 = 373). Before, the 312-wide window got a 322-wide dialog (overflow); the browser width is now window - 24 - 22 (frame) clamped 300..620, so the margin holds. Not measured: resizable windows (Song/Mod Information, Keyboard Shortcuts, Plugin Usage) and the 320-wide unsaved prompt. Test crates/orb-ui/tests/min_window.rs (fits at the minimum; the minimum equals what the widest and highest need). Plugin tests and docs/plugin.md updated. cargo test --workspace 1149 pass (one run had a flaky orb-player reopen test, passes alone and on rerun). Criterion 4 unchecked and a new criterion 5 added: the user checks again.

Review round 3 (user check failed: at the minimum window the Open dialog with 8 recent folders overflowed). Cause: the list took max(nominal, Places height) and the Places column was unbounded. Fix: the Places column scrolls inside the row (PLACES_H 199 = its height with no recent folder, MAX_RECENT 8); the row is max(nominal list, 199) as at d84e0af, so the dialog heights are unchanged (326 in a 300/400 window, 397 from 510; Save/Export +23); an error line or the overwrite question takes its height from the list (feedback of the last frame, list at least 60) so the dialog never grows; the button row is one row high again (with_layout filler made the status height unmeasurable). Worst cases measured at the minimum 488 x 373 and above (tests/browser.rs): 8 long recent folder names, a 5-line error, the overwrite question, all four modes: dialog inside the window with the 12-point margin, never taller than without the error, buttons inside. Other windows: error box with 12 long lines + one waiting 406 x 276 (scrolls); Audio Output now max_width window less margins and its content in a scroll area (max height window less margins less 50): at 488 x 373 and 488 x 200 it stays inside (its 'In use' rows and long device names could not be produced headless; bounded by construction); About 464 x 254 is fixed. Minimum unchanged: 488 x 373 (About 464 wide, Save/Export 349 high). cargo test --workspace 1151 pass, clippy macOS and xwin clean, fmt clean.

Review round 4 (user): the overwrite question's captions lose their key hints: "Replace  (Y)" -> "Replace", "Back  (Enter, Esc, N)" -> "Back" (the button was named Back, not Cancel; kept the name). Keys unchanged, existing tests (Y, Enter, Esc, N, button clicks by hit name) pass; ui_spec section 17 notes the captions. cargo test --workspace 1151 pass, clippy macOS and xwin and fmt clean.

Review round 5 (stale text only, no behavior change): comments in embed.rs (MIN_SCALE is a separate floor from the host minimum), orb-plugin editor.rs and tests/editor.rs now say 488 x 373; ui_spec section on the pattern editor and the pattern_editor.rs comments say the panel's native size (624 x 532 with the menu bar), not the smallest main window; docs/plugin.md paragraph reflowed; browser.rs width comment moved above the let; the min_window test renamed the_minimum_is_the_widest_and_highest_window_plus_the_margins; the commit body says 1151. cargo test -p orb-ui -p orb-plugin 650 pass, fmt clean.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Outcome: (1) the main button (Open, Save, Export, Choose) takes a path typed in the path bar like Enter there (Browser::press_button): folder entered, file chosen, relative name resolved in the current folder. (2) The file list fills its row with no gap; the dialog's height is unchanged from d84e0af (326 points in a 300 or 400 point window, 397 from 510 up, Save/Export 23 more); the Places column scrolls inside the row so recent folders never grow the dialog, and an error line or the overwrite question takes its height from the list. (3) The smallest main window is 488 x 373 points (was 312 x 277), layout::MIN_WINDOW_W/H shared by the standalone viewport and the plugin editor, with DIALOG_MARGIN 12; every non-resizable window fits (file dialogs 464 x 326/349, About 464 x 254, Audio Output scrolls and wraps, error box scrolls); the browser width keeps the margin at any window width. (4) The overwrite question's buttons read Replace and Back, keys unchanged. Commit: see git log of branch worktree-agent-ae344731103d1b1b2 (one commit on d84e0af). Tests: cargo test --workspace 1151 pass 0 fail (new: tests/min_window.rs, browser tests for the button, heights, worst cases); clippy -D warnings clean on macOS and x86_64-pc-windows-msvc; nightly fmt clean. User check: the user checked all of it in the dev app built from f0da6b1 and signed off on 2026-11-25 (typed path, list fit, minimum window with full recent folders, plain captions). The plugin's dialog and editor share browser.rs and the layout constants and were not opened separately.
<!-- SECTION:FINAL_SUMMARY:END -->
