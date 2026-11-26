---
id: TASK-233
title: 'Merge upstream f83c54f: skins, pattern editors, Audio Output, portable mode'
status: Done
assignee:
  - '@supervisor'
created_date: '2026-11-26T12:07:00Z'
updated_date: '2026-11-27T11:43:00Z'
labels:
  - ui
  - tools
  - docs
milestone: m-7
dependencies:
  - TASK-232
references:
  - https://github.com/pingw33n/openrb-338/commit/f83c54f
priority: high
type: task
ordinal: 250000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Why: the user decided on 2026-11-26 (task-232) to bring upstream's main into the fork: github.com/pingw33n/openrb-338 main at f83c54f, 120 commits after 6858253 (the base of task-230, the last upstream merge). task-232's description lists what came in.

How: a merge commit of f83c54f into main (not a cherry-pick or a rebase, as task-224 and task-230: the fork's main is public and later upstream merges need the right base), on the branch sync-upstream-f83c54f in the main checkout. A trial `git merge-tree` of the fork's main c9ae5ed with f83c54f conflicts in README.md, crates/orb-ui/src/{about,app,behavior,browser,crash,key_help,lib,main,menu,model,portable,settings,shortcuts}.rs, docs/building.md, docs/rbm.md, docs/ui_spec.md and scripts/package-installer.sh, plus modify/delete conflicts of upstream backlog files and site/ files the fork dropped. Each is resolved so that upstream's behavior is kept and the fork's screen reader work (m-7, docs/accessibility_plan.md) and its Windows-only, portable-zip packaging are kept; where both changed the same thing, the resolution is named in the notes.

From task-232's decisions:
- Portable mode combined (decision 1): upstream's marker OpenRB-338.portable, OpenRB-338.toml beside the exe and logs\ there; the fork's relative folders inside the program's folder (task-223); the fork's older portable.txt still makes a folder portable, and on the first start without OpenRB-338.toml the fork's data\ui.toml is copied into it. scripts/portable.txt goes; the zip carries upstream's marker.
- Everything new comes in (decision 2); the fork's screen reader hooks move from app.rs into upstream's controller.rs, so the panel works in screen reader mode in both skins. The new windows get their own m-7 tasks (task-234 on).
- Upstream's backlog from task-176 and its milestones from m-7 stay out (task-229's rule): task-250..task-316, m-8, m-9, and upstream's edits of the upstream task files the fork had dropped. The landing page stays out.

Tests: in the session of 2026-11-26 the user asked for no tests yet (decision 3: the merge stays on its branch, checked with cargo check, until the user says to run the tests and the review). This task is done when the criteria below hold; it then lands on main as task-230 did.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 main has f83c54f as the second parent of a merge commit; each conflicting file is resolved and the resolution is named in the notes
- [x] #2 No file of upstream's backlog task-250..task-316 or its milestones m-8 and m-9 is in the fork; backlog task list shows no duplicate id
- [x] #3 Portable mode is task-232's combination: OpenRB-338.portable or portable.txt beside the exe makes it portable, OpenRB-338.toml and logs\ there, the folders inside the program's folder stored relative, data\ui.toml copied once; covered by tests in portable.rs and the settings tests
- [x] #4 cargo build --release, cargo test --workspace, cargo clippy --all-targets -- -D warnings and cargo +nightly fmt --all --check pass
- [x] #5 The fork's screen reader tests (a11y_* and the announcement tests in orb-ui) pass with the modern skin; where the merge changes what they meet (the View title, Audio Output in Options, the gain-reduction read-out, the copy button, the sample patterns of a new song) they are adjusted, each change named in the notes; the classic skin's tree is task-238's
- [x] #6 Upstream's new status-line messages go through App::set_status, so screen reader mode announces them
- [x] #7 CHANGELOG.md's next section says in plain language what came from upstream and that a copy with the old portable.txt keeps its settings
- [x] #8 Reviewed by agent-reviewer with a merge verdict before it lands
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: sync-upstream-f83c54f (in the main checkout, no worktree, to share its target folder).
Upstream fetched by URL (no upstream remote is configured): f83c54f. git merge --no-ff --no-commit f83c54f: 18 content conflicts and 11 modify/delete conflicts.

Refs: the local branch made for the fetch and the stale tracking ref refs/remotes/upstream/main (6858253, an ancestor of main) were deleted: Backlog.md's cross-branch id check read upstream's task-249 and task-316 from them and numbered the first new task 317. With them gone the fork's ids go on at task-232.

Dropped in the merge (task-229's rule, task-232): upstream's backlog/tasks task-250..task-316 (66 files), the archived task-282, task-190's move to the archive, its milestones m-8 and m-9, its edits of the files task-186, 187, 191, 193, 215 and 244 (upstream's, dropped by task-230); site/app.js, site/index.html, tools/build_site.py, tools/build_site_test.py (the landing page).

Resolutions (code):
- portable.rs (add/add): upstream's file (OpenRB-338.portable, folder_of, candidate, folder) plus the fork's relative / relative_with / same / resolve and their tests; new: LEGACY_MARKER portable.txt, LEGACY_DATA data, is_portable (either marker, as a file), legacy_settings (data\ui.toml); folder() uses is_portable.
- settings.rs: both sides' keys (the fork's screen_reader, speak_raw_values, step_editor_follows, pitch_keys_make_notes, export_format; upstream's theme, pattern_editor_*_pos, audio_*), both sides' fields (portable_root; portable, warned, notice, load_error). default_file / file_for take ORB_UI_CONFIG only (file_for(config)); load_resolved checks is_portable, sets portable_root to the program's folder, reads OpenRB-338.toml with load_from_in (relative folders), and on the first start copies the fork's data\ui.toml (read inside the folder) before falling back to the per-user settings. Stored keeps the fork's Cow paths (relative inside the root).
- crash.rs: the fork's testable log_dir_for(os, env, portable folder) -> portable/logs, fed by portable::folder(); its test points at the folder, not data.
- main.rs: the module text names OpenRB-338.toml; upstream's Controller::start with the fork's run log stages (starting, notes, failure, the audio line); app.settings = the settings read before the audio, then use_system_services, then the start note through set_status; the end: the fork's match with runlog::end and start_failed, each followed by upstream's terminate_if_asio_was_used (also after a start error, the audio opening before the window); the run log's portable note names which marker was found and the real settings file.
- app.rs (22 hunks): upstream's skin structure (skin, fade, classic, splash_in_frame, audio window fields, panel_splash_texture, the skins' paint and crossfade); the fork's fields and functions kept (announcer, key_focus, nav, windows, bar, prompts; Win, focus_into_window, set_key_mode, move_nav, follow_section, window_focus, menu_bar_focus, win_open, window_focus_due, activate, announce_303, pin_patterns, step_editor_key, mouse_down, drive_value, spin_button, pattern_selector, focused_target, drum_grid). Removed from app.rs as upstream moved them into controller.rs: Gesture, Hold, keyboard, act, key_held, focus_click, control, mod_view, hit, gesture, first_press, repeat, pointer, press_origin, knob, fader, drum_strips, instrument_selection. self.ctls is gone: focused_target and the activate lookup use self.skin.layout().controls. New sync_nav: the focus order is made from the drawn skin's layout when the skin changes (nav_skin). blocked / the shortcut gate: upstream's asking() plus the fork's replace dialog. The shortcuts: the fork's Ctrl+M (Choose Mod) and upstream's Ctrl+I (File > Info) both; upstream's Ctrl+E before the fork's text-field / menu-bar gate. replace_ui and init_song_ui, chooser_ui and raise_windows: both. The screenshot exit: runlog::end then crash::end_process(0).
- controller.rs (not conflicting, the fork's code ported into it): the fork's keyboard (screen reader key map, numpad hook, step editor and grid keys, value keys, read-out keys, window gating), release and act (announcements, Hold::seen) replace upstream's equal-but-for-the-move versions; panel_input puts each control in its section's group in screen reader mode and adds the drum grids; control() has the fork's display handling (spin buttons, 303 step editor, described displays), the pattern selector before pattern1, joined controls (CLICK | DRAG, no node), the AccessKit Click and describe; knob and fader drive_value and describe; strips and selects sense CLICK | DRAG (not focusable, task-186). Hit ids: Placed::first gives a control's first place in the layout the fork's a11y::hit_id (the tree's and the focus order's id); a second place of the same control is keyed by its place, as upstream's.
- menu.rs: upstream's paste through the clipboard JSON, Clear Pattern, Initialize Song / Loop and its question; menu_items takes the fork's bar and focus_file; Edit and View are noted in the bar's keyboard walk (six titles now: File, Edit, View, Mods, Options, Help); Options: Audio Output... after the startup sound, then the fork's screen reader items; the fork's menu_bar_key with upstream's EDIT_ITEMS (with Option shortcuts).
- model.rs: the fork's grid_lit on skin::Ctl (with upstream's n.max(1) guard), used by upstream's look(); part_name is upstream's (the modern skin's naming).
- behavior.rs: the fork's fader_from_engine with upstream's fader_handle_top(p, &Fader); both tests.
- browser.rs: the fork's start_dir / check_choice / note_choice and upstream's PLACES_H / MIN_BODY_H.
- key_help.rs: upstream's empty note for the transport group (its user's cuts, task-284); the fork's groups stay (Screen reader mode, The numpad, Main-keyboard transport keys); upstream's guard test the_users_cuts_and_renames_hold lists the fork's titles and checks that Numpad * is only in the fork's numpad group (task-225's hook gives that key); groups() takes is_mac and screen_reader in the upstream tests and drum_editor's guard.
- shortcuts.rs: INFO and PATTERN_EDITOR from upstream, CHOOSE_MOD and CHOOSER_INFO from the fork (CHOOSER_INFO is Ctrl+I as INFO: the mod chooser takes it first while open); the test lists all 15.
- about.rs: the version line is the fork's win::line, in upstream's row with the copy button; the button gets an AccessKit name ("Copy version", "Copied").
- lib.rs: both modules lists. shortcuts / scripts/package-installer.sh: the fork's nsis_path on upstream's new NOTICES define.
- a11y/announce.rs: F4's gain reduction from upstream's gr_byte and gr_lit: "N of 9" (was the fork's "N of 4" from gr_leds, which upstream replaced); its unit test and a11y_announce's playing test follow.
- runlog.rs: GetModuleHandleExW declared as portable.rs declares it (address *const c_void), so the two externs do not clash (a lint error).
- Upstream's new status lines (menu.rs pattern paste, clear, initialize, undo; audio_window.rs notes; the settings notice in App::show) go through set_status, so screen reader mode announces them.
- scripts/package.sh: the fork's portable.txt copy is gone (upstream's OpenRB-338.portable is in the zip); its README.txt text names the new layout, the relative folders and the old portable.txt; scripts/portable.txt deleted.
- tests: a11y_drive's grab uses fader_handle_top(p, &c.fader()); tests/settings.rs file_for test for the one-argument form.

Resolutions (docs, by agent-worker on the supervisor's brief; no build): README.md (the fork's plugin paragraph, VST3 only; upstream's macOS first-start sentence; the zip text for the combined portable mode), docs/building.md (upstream's ORB_UI_CONFIG row with the fork's log rows; upstream's ASIO and portable zip sections with the fork's portable.txt and relative-folder bullets; the fork's landing-page-not-in-fork section; the crash log and Portable mode paragraphs rewritten), docs/rbm.md (both run records), docs/ui_spec.md (the settings paragraph, the menu table with upstream's rows and the fork's Export and Choose Mod rows, the Options rows of both, the Keyboard Shortcuts text with a sentence on the fork's numpad group), AGENTS.md "This fork" release line, tools/win_diag.ps1 (logs beside the exe with either marker, and data\logs listed when it exists).

CHANGELOG.md: a "Next release (not released yet)" section in plain language.

Checks run (2026-11-26, no tests by the user's word): cargo check --workspace --all-targets clean; cargo clippy --workspace --all-targets -- -D warnings clean; cargo +nightly fmt --all applied (then --check clean). Not run: cargo build --release, cargo test, the corpus, the reviewer.

To check when the tests run: the menu-bar walk tests (a11y_menu_bar.rs, a11y_mod_chooser.rs: one more title, View, between Edit and Mods), a11y_baseline's node counts (the View title, the About copy button), the key help guard on the fork's groups (mouse-only words), the settings and portable tests on the new file_for and load_resolved, the fork's tests that used data\ui.toml.

First test run, 2026-11-27, on 745616d: cargo test --workspace --no-fail-fast: 1374 passed, 17 failed, 7 ignored (ORB_CD_ISO and the third-party folders not set: the gated tests skipped). The failures, by cause:
- The new View title between Edit and Mods (expected): a11y_menu_bar f10_focuses_file_right_moves_to_mods_down_opens_it_escape_closes, enter_on_an_item_runs_it, speak_raw_values_shows_in_screen_reader_mode_only, the_options_items_read_their_state, an_item_that_opens_a_window_returns_to_the_control_when_it_closes, an_item_that_opens_no_window_gives_the_focus_back; a11y_mod_chooser the_menu_item_opens_the_dialog, with_the_mode_off_there_is_no_chooser, the_mods_menu_marks_the_playing_kit, ctrl_i_shows_mod_info_of_the_selection (the walks land on View or Classic one step early); a11y_baseline the_tree_today (nodes 205, was 204) and the_tree_in_screen_reader_mode (600, was 599).
- F4's new gain-reduction wording (expected): a11y_announce the_read_out_keys_on_a_known_song wants 'gain reduction 1 of 4', now '5 of 9'.
- Upstream's task-288, a new song has four sample patterns in A1: a11y_grid playback_moving_the_pattern_is_silent_on_a_grid assumes A1 is empty.
- Upstream's About links (f83c54f, the project site): a11y_windows about_can_be_read_line_by_line looks for 'Mod collection:', which is gone.
- A fork bug the merge exposed: tests/portable.rs a_local_file_with_settings_is_used_alone and an_absent_local_file_with_the_marker_takes_the_per_user_settings_in: a folder stored as /local reads back as C:/local, because portable::resolve joins a rooted path without a drive letter to the portable folder; a rooted path should stay as it is.
- orb-plugin texture_free: the test process ends with STATUS_ACCESS_VIOLATION after its first test passes (upstream's plugin test, task-292; the plugin goes with task-231).

Fixes after the first run (2026-11-27), to be amended into the merge commit:
- portable::resolve keeps a rooted path without a drive letter (\local) as it is; joined to the portable folder it took the folder's drive (upstream's tests/portable.rs a_local_file_with_settings_is_used_alone and an_absent_local_file_with_the_marker_takes_the_per_user_settings_in). A unit test pins it.
- A real clash, not a stale test: upstream's Ctrl+I (File > Info) took the key before the fork's mod chooser, whose Ctrl+I is Mod Info of the selection (task-195); App's shortcut gate now skips INFO while the chooser is open (a11y_mod_chooser ctrl_i_shows_mod_info_of_the_selection).
- The View title: the menu-bar walks of a11y_menu_bar and a11y_mod_chooser take one more Right (open_mods_menu past File, Edit and View; Options after 4 steps, Help after 5); an_item_that_opens_no_window_gives_the_focus_back one more Down (upstream's Audio Output... above the screen reader items); a11y_baseline 205 nodes, 198 named, 196 focusable, View in BASELINE_NAMES.
- a11y_announce the_read_out_keys_on_a_known_song: F4 says 'gain reduction 5 of 9'.
- a11y_grid playback_moving_the_pattern_is_silent_on_a_grid: Edit > Clear Pattern of the 808's A1 first (upstream's task-288 fills a new song's A1), then the steps as before.
- a11y_windows about_can_be_read_line_by_line: six Tab stops, the second 'Copy version' (upstream's task-279 button, named by the merge).
- orb-plugin tests/texture_free.rs: on Windows the headless wgpu instance uses DX12 then GL, no Vulkan, as the fork's app does (task-217): this machine's Intel Vulkan driver crashed the test process (STATUS_ACCESS_VIOLATION). All 3 pass.
- orb-ui without the app feature (the plugin's build): check_screen_reader is cfg(feature = app) and was_focused allows dead code there, so that build has no warnings.

Second run, 2026-11-27: cargo test --workspace --no-fail-fast: 1394 passed, 0 failed, 7 ignored (gated tests skipped: no ORB_CD_ISO or third-party folders here). nightly fmt --check clean.

Added for criterion 3: tests/portable.rs an_older_fork_copy_keeps_its_settings_and_relative_folders (portable.txt and data/ui.toml with a relative songs folder: portable, the old settings read inside the folder, written relative to OpenRB-338.toml, the old file unchanged, nothing per-user). tests/portable.rs 10 passed. cargo build --release passed; clippy --workspace --all-targets -D warnings clean.

Engine corpus, the user's decision of 2026-11-27, asked in this session. The question: the fork's rule asks for the corpus check of an engine change, which needs the CD image (ORB_CD_ISO, not set on this machine); four upstream commits in this merge touch crates/orb-engine (git log 6858253..f83c54f -- crates/orb-engine): d2c2b4b (Initialize Song / Loop: orb's tracks equal the original's saved TRAK chunks 4 of 4), 2ea3e21 (per-drum outputs: 'corpus_golden EQUAL (diag, app, ...)'), 4ad0ac6 (note mode: 'Corpus unchanged (note mode only; corpus_golden passes)'), 82e8836 (plugin outputs: 'corpus_golden passes, orb compare --manifest 164 of 164 equal'); the fork changes no engine code. Options: accept upstream's check (as task-230); give the image path; wait for the test machine. The user chose 'Accept upstream's check'. The no-image corpus_golden sets passed in cargo test --workspace here.

Review round 1 (agent-reviewer): send back. Fixed in the amend:
- Blocking 1: key_help.rs, the PgUp / PgDn row of the screen reader group had literal lines of 101 and 107 columns (the fork's text): split by hand, every fork Rust line is at most 100 columns (session.rs:90, 102 columns, is upstream's own line, unchanged in f83c54f).
- Blocking 2: the commit's Testing line named 1395; the full run gave 1394 (the added portable test passed in its file's run). Fixed to the runs as they were.
- Minor 3: the subject names ui first (ui,tools), the main area of the change.
- Minor 5: App::sync_nav keeps the skin (Arc, compared with Arc::ptr_eq) instead of its address, which a freed skin's successor could reuse.
- Minor 6: CHANGELOG's Ctrl+I line says that in Choose Mod Ctrl+I is still Mod Info of the selection.
- Minor 4 (the Co-Authored-By model): the supervisor is Claude Opus 5.5; the line stands.
After the fixes: nightly fmt --check clean, clippy --workspace --all-targets -D warnings clean, cargo test -p orb-ui 709 passed, 0 failed.

Review round 2 (agent-reviewer): merge. Landing: the merge commit fast-forwarded onto main (a two-parent merge, as task-224 and task-230), pushed to origin.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Upstream's main f83c54f (120 commits after 6858253) is merged into the fork as a two-parent merge commit (parents c9ae5ed and f83c54f), made on the branch sync-upstream-f83c54f and fast-forwarded onto main, per the user's decisions of 2026-11-26 (task-232).
Taken: the skin layer and the classic skin with View > Classic / Modern, the 303, 808 and 909 pattern editors, Copy Pattern as JSON, Clear Pattern, Initialize Song / Loop, Options > Audio Output with ASIO, Ctrl+I for File > Info, the meters' log scale and the gain-reduction bar, a new song's sample patterns, the About copy button, OpenRB-338.toml and upstream's portable mode, the plugin's work. Left out (task-229's rule): upstream's backlog task-250..task-316, m-8 and m-9, and the landing page.
The main resolutions: portable mode combined (upstream's OpenRB-338.portable, OpenRB-338.toml and logs\ beside the exe; the fork's relative folders; an older portable.txt copy stays portable and its data\ui.toml is copied once); the fork's screen reader hooks ported into upstream's controller.rs, with the focus order made from the drawn skin's layout (App::sync_nav) and the controls' ids kept (Placed::first, a11y::hit_id); upstream's new status lines through set_status; Ctrl+I stays Mod Info inside the mod chooser; the menu bar has six titles. A fork bug the merge exposed was fixed (portable::resolve kept no rooted path). The notes list every conflict's resolution.
Follow-ups: task-234 (Audio Output), task-235 and task-236 (the pattern editors, each with a Decide first), task-237 (the Initialize Song question), task-238 (screen reader mode with the classic skin), task-239 (spoken names 808, 909, 303; the user's decision of 2026-11-26). task-231 notes the plugin parts upstream added.
Engine corpus: the user accepted upstream's recorded corpus runs (2026-11-27, asked in this session; no ORB_CD_ISO here); the fork changes no engine code.
Tests: cargo test --workspace 1394 passed, 0 failed, 7 ignored (gated tests skipped), the added portable test (tests/portable.rs 10 passed), after the review fixes cargo test -p orb-ui 709 passed; cargo build --release; clippy --workspace --all-targets -D warnings and nightly fmt --check clean. Review: agent-reviewer, round 1 send back (two over-long lines, the test count in the message; minor: subject area, sync_nav's key, the CHANGELOG's Ctrl+I line), round 2 merge.
<!-- SECTION:FINAL_SUMMARY:END -->
