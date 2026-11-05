---
id: TASK-194
title: System file dialogs in screen reader mode
status: Done
assignee:
  - '@agent-worker'
created_date: '2026-11-05T19:32:00Z'
updated_date: '2026-11-06T05:55:00Z'
labels:
  - ui
milestone: m-7
dependencies:
  - TASK-203
references:
  - crates/orb-ui/src/browser.rs
  - crates/orb-ui/src/app.rs
  - crates/orb-ui/src/menu.rs
  - crates/orb-ui/src/settings.rs
  - crates/orb-ui/Cargo.toml
  - docs/accessibility_plan.md
  - docs/accessibility.md
priority: medium
type: feature
ordinal: 217000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Planned in docs/accessibility_plan.md as A11Y-9; the analysis is docs/accessibility.md. Read both before starting.

- What (D4's recommendation): in screen reader mode, `open_browser` opens the system dialog for
  the mode instead (filters: songs `.rbs`, mods `.rbm`, export WAV/FLAC/Ogg as the browser
  offers), starting in the folder the browser would start in, and passes the result to the same
  `chosen` path. Options menu items "Set Songs Folder..." and "Set Mods Folder..." with a folder
  dialog, doing what the browser's "Use as ... folder" buttons do. The dialog blocks the frame
  loop while open; the audio thread keeps playing (check, and note it).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 The headless tests of the shared path (the system dialog's answer reaches open, save, export and mod loading through the same `chosen`) pass for all four commands and the folder items; the Open dialog was driven by hand on Windows (the other dialogs: TASK-197's hand check). In screen reader mode each of the four menu commands opens the system dialog
- [x] #2 The folder items change `ui.toml` as the browser's buttons do (test)
- [x] #3 With the mode off, the in-app browser opens as before
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. browser.rs: factor the start folder (Browser::new) into `start_dir(mode, settings)`, and the checks and bookkeeping of `Browser::finish` (mod status, file exists, note_recent, last_*_dir, settings.save) into free functions `check_choice` and `note_choice`, so the browser and the system dialog share them.
2. New module system_dialog.rs: `Request` (the four modes and the two folders), a `Picker` seam (boxed FnMut(Request, start dir) -> Option<PathBuf>), the default picker over rfd::FileDialog (filters rbs / rbm / wav,flac,ogg), and the browser's extension rule for a typed name.
3. app.rs: field `picker`, `set_picker` for tests; `open_browser` goes to the system dialog when `screen_reader_mode()`; the result goes through check_choice, note_choice and the same `App::chosen`. `App::choose_folder` sets songs_dir / mods_dir as the browser buttons do, saves ui.toml, refreshes the mod list.
4. menu.rs: Options > Set Songs Folder... and Set Mods Folder... (Cmd::SetSongsFolder, Cmd::SetModsFolder).
5. Tests headless with a fake picker (mode on and off, four modes, cancel, folders, ui.toml); then by hand once on Windows with the release build.
6. Cargo.toml: check that rfd needs no feature change.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: worktree-agent-a5bd6aa4041aa8564

Evidence and findings (as of the amended commit after review round 2).

rfd: rfd 0.17.2 `FileDialog` needs no feature on Windows (win_cid backend, IFileDialog) or macOS (NSOpenPanel/NSSavePanel); on Linux the already enabled `xdg-portal` feature (pollster, no system library) is the backend of the sync FileDialog too. `default-features = false, features = ["xdg-portal"]` therefore stays; `wayland` is not needed. Built and tested on Windows only here; Linux and macOS CI is the check for the other two. One new direct dependency since round 1: `raw-window-handle = "0.6"` (see round 1, item 2).

Design: `crates/orb-ui/src/system_dialog.rs` (Request, Folder, Parent, the `Picker` seam, `system_picker()` over rfd, `no_picker()`, `with_extension`, the Replace question). `browser.rs`: `start_dir`, `check_choice`, `note_choice` factored out of `Browser::new` / `Browser::finish`, so the browser and the dialog share the start folder, the mod/file checks and the recent/last folder bookkeeping. `App::open_browser` goes to `system_pick` when `screen_reader_mode()`; the result goes through `with_extension` (the browser's rule for a typed name: .rbs added to a song, .wav to an extension-less export, other export extension refused), `check_choice`, `note_choice`, then the same `App::chosen`. A cancelled dialog changes nothing (and ends a Save As that the prompt's Save started, as the browser's Cancel does). `App::choose_folder` + `Cmd::SetFolder(Folder)` + the Options items "Set Songs Folder..." and "Set Mods Folder..." (shown only in screen reader mode, so the mode off is the menu of main): sets songs_dir / mods_dir like the browser's buttons, saves ui.toml, refreshes the mod list for the mods folder; the dialog starts in the folder now set.

Known limits (not decided here): the Export dialog's default extension is always WAV (TASK-204); on Linux rfd's portal returns None when libdbus or the portal is missing, which reads as a silent cancel (TASK-206). The dialog is owned by the app's window since round 1, and a name the dialog did not confirm is asked about (TASK-203), so the earlier limits "no owner window" and "replaces without asking" are gone.

Modal dialog and the audio: the dialog call blocks the UI thread inside the frame. Audio is a cpal stream (crates/orb-player/src/output.rs, `Audio` in a cpal output stream) whose callback runs on the audio thread and owns the sequencer state, so a playing song keeps playing; only the messages from the UI (knobs, transport) wait. By hand I saw the window still responsive after the dialog and its state intact after Esc; I did not measure the audio itself.

Failed hypothesis / hazard found: the existing tests build `App::new` with the default setting `auto`; on this machine NVDA runs, so `auto` was on and Ctrl+O in tests/browser.rs opened a REAL modal dialog and the test hung (I killed the process, no dialog was left open). Round 1 fixed it at the root: `App::new` is inert (no system screen reader check, a picker that cancels), only `App::use_system_services()` (main.rs) connects the real ones, so no test line sets `screen_reader = Off` any more.

By hand (Windows 11, release build, throwaway ui.toml with screen_reader = "on" through ORB_UI_CONFIG, scripts handcheck.ps1 / handcheck2.ps1 in the scratchpad: start the app on tests/songs/diag/songs/303_accent0.rbs, bring it forward, send Ctrl+O, look at the windows through UI Automation, send Esc, kill the app): a window titled 'Open song' of class #32770 (the system's common file dialog) appeared; with the owner the main window is disabled while it is open and enabled and foreground after Esc. Only Open was driven; Save As, Export, Choose file and the folder items call the same function with another Request and filter and are TASK-197's hand check (checklist item 20). Nothing was left open.

Review round 1 (send back, 2026-11-06). Done in the amended commit:

1. Overwrite: the user chose "Ask in OpenRB-338" (recorded as TASK-203, created Done). `App::system_pick` keeps the path the dialog returned; when `with_extension` changes it and the result exists, `App::replace` holds a `system_dialog::Replace` and `replace_ui` draws a modal "Replace <name>?" (Yes/No, No has the focus, Y = Yes, N/Esc/Enter = No, drawn in the style of the unsaved-changes prompt, blocks the panel and the menu like it). Yes goes on through `finish_pick` (the browser's checks, the folder notes, `App::chosen`); No writes nothing and ends a Save As that the prompt's Save started.

2. Owner window: `system_dialog::Parent` (raw window and display handles taken from `eframe::Frame` every frame in `App::ui`) implements HasWindowHandle/HasDisplayHandle through `borrow_raw`; `Picker` is `FnMut(Request, &Path, Option<&Parent>)` and `system_picker` calls `FileDialog::set_parent`. New direct dependency `raw-window-handle = "0.6"` in crates/orb-ui/Cargo.toml (already in Cargo.lock through eframe, winit and rfd; no new crate, no system library). By hand: main window enabled before = True, while the dialog is open = False, after Esc = True and foreground.

3. Determinism: `App::new` is inert (probe false, `no_picker()`); `App::use_system_services()` installs the real probe and picker; main.rs calls it once after `Settings::load()`. Test Apps cannot reach the user's ui.toml: `Settings::default()` has no file, so `save()` writes nothing; only `Settings::load()` (main.rs) points at the real file, and the tests use `load_from(tempdir)`. Test a_new_app_is_inert.

4. AC #1 narrowed to the evidence; TASK-197 got the AC for the deferred hand check; item 20 added to the NVDA checklist. TASK-194 depends on TASK-203.

5. Follow-up TASK-204 (Export dialog keeps the format the user picks), not fixed here.

6. Commit subject area is "tools:".

Review round 2 (send back, 2026-11-06). Done in the amended commit:

A. Focus: the old code gave No the focus only when nothing held it, and egui's Modal requests none, so after a menu click or Ctrl+S the box opened without focus. `Replace` now has `focused: bool` (false when `system_pick` sets it); `replace_ui` calls `no.request_focus()` on the first frame only and stores true, so Tab is not fought afterwards. Test (tests/system_dialog.rs, `question_tree`): the knob Cutoff of Bass Line 1 is focused first through an AccessKit Focus request, then Save As opens the question: after the frames the focused node is the single "No" button.

B. My round-1 note "no roles yet" was wrong: egui 0.37's Modal is UiKind::Modal, whose AccessKit role is Dialog (ui_stack.rs:76). With the a11y harness the Dialog node existed but had no name (label None), and the question text is only a plain label in its subtree. The modal's Ui node (`UiStack::unique_id` of the frame of kind Modal) now gets the question as its label through `accesskit_node_builder`; test the_question_takes_the_focus_on_no_and_is_a_named_dialog: exactly one Dialog node, label "Replace a.txt.rbs?". TASK-193 still owns the roles and focus of the app's other windows.

C. Tests: n_escape_and_enter_answer_no_and_leave_the_file_alone runs N, Escape and Enter in frames (with the knob focused first) and checks that the question closes, the existing file is unchanged, no document path is set and no error shows; the focus test is in A.

D. The stale lines of the notes (limits, failed hypothesis) are corrected above (the whole notes were rewritten through the CLI).

E. TASK-206 (new, ui, m-7, low, @agent-worker): a file dialog that cannot open on Linux reads as a cancel.

Checks: see the final summary.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
In screen reader mode, Open, Save As, Export and Mods > Choose file open the system's file dialog (rfd::FileDialog), owned by the app's window, instead of the in-app browser, starting in the browser's start folder, with filters .rbs / .rbm / wav, flac, ogg. The answer goes through the browser's checks and folder notes (shared functions in browser.rs) and the same App::chosen. A name the dialog did not confirm that is a file already (`a.txt` became `a.txt.rbs`) is asked about first in a Yes/No box "Replace <name>?" (TASK-203, the user's choice): it is a named Dialog node, and No takes the focus on its first frame whatever held it before. Options > Set Songs Folder... / Set Mods Folder... (shown only in the mode) choose the folders and save ui.toml like the browser's "Use as ... folder" buttons. With the mode off nothing changes. App::new is inert (no system screen reader check, no real dialog); main.rs calls App::use_system_services().

Files: crates/orb-ui/src/system_dialog.rs (new), browser.rs, app.rs, menu.rs, main.rs, lib.rs, Cargo.toml (raw-window-handle 0.6, already in the lock); tests/system_dialog.rs (new, 10 tests); docs/accessibility_plan.md (NVDA checklist item 20). Follow-ups: TASK-203 (decision, Done), TASK-204 (Export dialog keeps the chosen format), TASK-206 (a dialog that cannot open on Linux reads as a cancel), TASK-197 got an AC for the dialogs not driven by hand.

Testing: cargo test --release -p orb-ui --no-fail-fast all pass except the known TASK-200 crash.rs failure (lib 85, main 5, a11y_names 4, a11y_menu_bar 7, browser 22, error_box 15, keyboard 12, menu 54, wired 22, system_dialog 10, others pass); clippy -D warnings and nightly-2026-10-18 fmt check clean. By hand on Windows: Ctrl+O with the mode on opens the system 'Open song' dialog; the main window is disabled while it is open, enabled and foreground again after Esc. Save As, Export, Choose file and the folder dialogs are TASK-197's hand check.

Branch worktree-agent-a5bd6aa4041aa8564, one commit (amended after review rounds 1 and 2).
<!-- SECTION:FINAL_SUMMARY:END -->
