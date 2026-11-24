---
id: TASK-297
title: >-
  File dialog: Open accepts a typed full path like Enter does; the file list
  fills the window height
status: To Do
assignee:
  - '@agent-worker'
created_date: '2026-11-24T18:26:00Z'
updated_date: '2026-11-24T18:27:00Z'
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
- [ ] #1 The Open (and Save, Choose) button accepts a typed full file path, a folder path and a relative name exactly as Enter does; unit tests cover each
- [ ] #2 The file list fills the dialog window's height with no empty space below it, following window resizes, in the app and the plugin; task-285's centering after Up still works
- [ ] #3 cargo test --workspace, clippy -D warnings (macOS and Windows) and nightly fmt --check pass
- [ ] #4 The user has checked both in the app
<!-- AC:END -->
