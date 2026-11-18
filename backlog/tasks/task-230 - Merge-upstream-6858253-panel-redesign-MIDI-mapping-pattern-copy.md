---
id: TASK-230
title: 'Merge upstream 6858253: panel redesign, MIDI mapping, pattern copy'
status: Done
assignee:
  - '@supervisor'
created_date: '2026-11-18T19:10:00Z'
updated_date: '2026-11-18T20:25:00Z'
labels:
  - ui
  - tools
  - docs
milestone: m-7
dependencies:
  - TASK-229
references:
  - https://github.com/pingw33n/openrb-338/commit/6858253
priority: high
type: task
ordinal: 248000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Why: the user decided on 2026-11-18 (task-229) to bring upstream's main into the fork: github.com/pingw33n/openrb-338 main at 6858253, 107 commits after 32a642d (the base of task-224, the last upstream merge). task-229's description lists what came in.

How: a merge commit of 6858253 into main (not a cherry-pick or a rebase, as task-224: the fork's main is public and later upstream merges need the right base). A trial `git merge-tree` of the fork's main 0251417 with 6858253 conflicts in README.md, crates/orb-ui/Cargo.toml, crates/orb-ui/src/{about,app,browser,crash,error_box,lib,main,menu,model,settings,shortcuts}.rs, crates/orb-ui/tests/{frame_cost,settings,wired}.rs, docs/building.md, docs/ui_spec.md and scripts/package-installer.sh. Each is resolved so that upstream's behavior is kept and the fork's screen reader work (m-7, docs/accessibility_plan.md) and its Windows-only, portable-zip packaging are kept; where both changed the same thing, the resolution is named in the notes.

In the same merge commit, from task-229's decisions:
- Upstream's new backlog files are left out: backlog/tasks/task-176 .. task-249 as upstream numbered them and backlog/milestones/m-7 "MIDI input and the VST3/CLAP plugin". The fork's own task-176..task-228 and its m-7 stay as they are. Upstream's edits to tasks the two sides share (ids below 176) and to backlog docs and decisions come in.
- The landing page is removed: site/, .github/workflows/pages.yml and the tools that only build the site (tools/build_site.py, tools/site_textures and whatever else serves only site/), with any doc or workflow reference to them updated.
- AGENTS.md "This fork" gets a line: task numbers from 176 up in upstream's docs, code comments and commit messages are upstream's tasks, not the fork's.

The plugin stays in this merge as upstream has it; task-231 removes it.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 main has 6858253 as the second parent of a merge commit; each conflicting file is resolved and the resolution is named in the notes
- [x] #2 No file of upstream's backlog/tasks task-176..task-249 or its m-7 milestone is in the fork; backlog task list shows no duplicate id
- [x] #3 AGENTS.md 'This fork' says that upstream's task numbers from 176 up refer to upstream's backlog
- [x] #4 cargo build --release, cargo test --workspace (the optimized test profile of upstream's task-205), cargo clippy --all-targets -- -D warnings and cargo +nightly fmt --all --check pass
- [x] #5 The fork's screen reader tests (a11y_* and the announcement tests in orb-ui) pass; where the merge changes what they meet (the Edit title, the redesigned pattern keys) they are adjusted to it, each change named in the notes
- [x] #6 site/, .github/workflows/pages.yml and the site builder tools are gone, and nothing in the tree refers to them except the explanatory notes in AGENTS.md, docs/building.md and docs/accessibility_plan.md and the archived upstream task-175 (a shared task's record, naming the landing page and upstream's task-212)
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: task-230-upstream-6858253 (in the main checkout, no worktree, to share its target folder).
Upstream fetched as refs/remotes/upstream/main = 6858253. git merge --no-ff --no-commit upstream/main: 19 files conflicted (the list in the description).

Dropped in the merge (task-229): upstream's backlog/tasks task-176..task-249 (74 files) and backlog/milestones m-7 "MIDI input and the VST3/CLAP plugin" (git rm of the 75 added files; upstream's edit of the archived task-175 comes in); site/, .github/workflows/pages.yml, tools/build_site.py, build_site_test.py, site_fonts.py, site_images.py, site_textures.py, tools/site_textures/, the .gitignore /_site/ entry, README's Website link, docs/building.md's landing page section (replaced by a short "not in this fork" section).

Resolutions:
- crates/orb-ui/Cargo.toml: upstream's optional eframe and the egui dependency, the fork's log.
- settings.rs (the hard one): upstream's merging save (upstream task-229: Values, base, noted, dirty, setters) with the fork's keys and portable folders. The fork's portable field `base: Option<PathBuf>` is renamed `portable_root` (upstream uses `base` for its Values). Values gains screen_reader, speak_raw_values, step_editor_follows, pitch_keys_make_notes, export_format; merged() applies each when it differs from base (no setters); read_in() resolves the file's folders inside the portable root before the merge, both at load and in merged(); save writes the merged folders through stored() (relative inside the root). PartialEq also compares portable_root. ExportFormat derives Default (Wav).
- browser.rs: the fork's note_choice() uses upstream's set_last_mod_dir / set_last_song_dir.
- app.rs: both imports (embed lits, a11y); the fork's announcer and key_focus fields and upstream's clip, undo and edit_items; the click-on-bar check keeps the fork's replace dialog and takes upstream's layer test (upstream task-235); set_speech, then upstream's flush_part_textures and embed_refresh; the plugin branch, then the fork's condition with replace; hidden() then the fork's section groups; displays: the fork's spin button, step editor and describe code, then upstream's centered digits; knob: upstream's double click to the default first, then the fork's drive_value and describe.
- main.rs: the fork's graphics rule (task-217: DX12 then GL, run log) kept; upstream's wgpu_backends / wgpu_setup and their test dropped; upstream's start_error_text kept (hint text now "Windows tries dx12 first, then gl") and fed to the fork's start_failed (run log, box, exit 1); upstream's PROCESS_START; the audio line goes to the run log with upstream's buffer text.
- menu.rs: upstream moved the bar into menu_items(); it now takes the fork's `bar` and `focus_file` and notes each title (File, Edit, Mods, Options, Help) for menu_has_keys and the focus back; the fork's Choose Mod..., checked_item entries, Screen Reader Mode, Speak Raw Values, Step Editor Follows, Pitch Keys Make Notes and Set Folder items are in it (after the plugin's early return); menu_bar keeps the fork's F10, Alt and leave logic and takes upstream's PastePattern request and repaint. command(): the fork's set_key_mode and settings toggles plus upstream's SetPluginMode, Undo and pattern commands. Upstream's pattern commands wrote `self.status = Some(..)` directly: changed to `self.set_status(..)` so they are announced in screen reader mode (the fork's rule, a11y/announce.rs).
- model.rs part_name: upstream's held routing key, the fork's grid_lit (no `sec`).
- shortcuts.rs test: both lists of shortcuts. about.rs: ROW0 with the fork's win::line. crash.rs, error_box.rs, lib.rs: both.
- tests/settings.rs, tests/wired.rs: both sides appended tests at the same place; rebuilt as the fork's tests, then upstream's (from git's stage 2 and 3 copies); s.base becomes s.portable_root. frame_cost.rs: upstream's 200-frame header plus the fork's AccessKit paragraph.
- docs/building.md: both new sections (the fork's run log and release sections, upstream's test profile). docs/ui_spec.md: the settings paragraph merged (portable, merging save, the fork's keys); the menu table with the Edit rows and the fork's Export and Choose Mod rows; the fork's 18.2 kept with a line on the Edit title; section 19 is upstream's redesign text, the fork's section 20 (screen reader) stays 20, upstream's plugin section becomes 21. README: the fork's text; upstream's Website link dropped; a short plugin section (task-231 removes it together with plugin_usage's README test).
- scripts/package-installer.sh: the fork's nsis_path wrapping plus upstream's -DPLUGINDIR.

Made to build and pass after the merge:
- orb-plugin needs `App: Send` and builds orb-ui without eframe: system_dialog::Picker is `+ Send`, `unsafe impl Send for Parent` (raw handles used on the UI thread only), tests/system_dialog.rs uses Arc<Mutex> instead of Rc<RefCell>; the fork's modules import `egui` instead of `eframe::egui`; graphics.rs and Parent::of are behind the `app` feature. None of this changes the standalone app.
- a11y names: upstream's redesign added tb303_N.rest_led, now named "Rest light". drive::SELECTOR_SIZE is (61.4, 56.6) for the new keycaps (the test compares within 0.01).
- Tests for the fifth title: a11y_menu_bar.rs (one more Right to reach Mods, Options, Help), a11y_mod_chooser.rs (open_mods_menu passes Edit), a11y_baseline.rs (nodes 204, named 197, focusable 195: one more each, the Edit title), menu.rs the_undo_item_names_its_command_and_ctrl_z_runs_it: the Edit menu is closed before Ctrl+C, since the fork withholds the panel's keys while a menu is open (task-202).

Commands: cargo check --workspace --all-targets; cargo check -p orb-ui --no-default-features --lib; cargo +nightly fmt --all --check; cargo clippy --all-targets -- -D warnings; cargo test --workspace --no-fail-fast.
Results: first full run 1131 passed, 20 failed (plugin exports build, names, selector size, baseline counts, menu walks, README link); after the fixes 1151 passed, 0 failed, 6 ignored (a11y_baseline rerun on its own after its focusable count). ORB_CD_ISO and the third-party folders are not set on this machine, so the gated tests skipped; corpus_golden's songs that need no image ran (the test profile runs them in full now).
Checked: every fn the fork had added since 32a642d (634 names) still exists.

On the criteria: #5 cargo build --release passed (4 min); the tests ran as cargo test --workspace, which since upstream's task-205 builds in the optimized test profile (opt-level 3 with debug assertions), not --release. #6 the screen reader tests pass, but not all unchanged: the ones that walk the menu bar by arrow keys take one more Right for the new Edit title (a11y_menu_bar.rs, a11y_mod_chooser.rs), the tree baseline counts one more node (a11y_baseline.rs), and drive.rs's selector size follows the redesigned keys. What they check is the same. No duplicate task id in backlog/tasks.

Review round 1 (agent-reviewer): send back. Fixed:
- Corpus with the CD image (blocking 1): not possible here (no ORB_CD_ISO on this machine). The user decided on 2026-11-18, asked in this session: accept upstream's corpus verification. The fork's engine is upstream's, unchanged (the fork has no engine commits since 32a642d), and upstream ran its corpus with each engine commit it brings. So corpus_mods and orb compare with --mod-dir were not run for this merge; the no-image corpus_golden sets passed in cargo test --workspace.
- Criteria (blocking 2): #5 and #6 reworded to what was done (cargo test --workspace in the optimized test profile; the screen reader tests pass, adjusted where the merge changes what they meet).
- app.rs choose_folder (blocking 3): Set Songs/Mods Folder use set_songs_dir / set_mods_dir, so the choice wins over another instance's change to the file.
- settings.rs merged(): a noted recent folder replaces the same folder in another case (portable::same), as read_in dedupes.
- a11y_baseline.rs: BASELINE_NAMES pins "Edit".
- Wording: Edit is the second of the bar's five titles (commit message, accessibility_plan.md); the Rest LED's name keeps the names table complete, but LEDs have no node, so the commit no longer says screen reader mode covers it (names.rs comment says so).
Left as they are, with the reason:
- AGENTS.md line 5 (upstream's text, "MIDI in scope in milestone m-7"): upstream's wording outside "This fork", kept for rebasing; the new "This fork" line says m-7 and up in upstream's text mean upstream's.
- backlog/archive/tasks/task-175 (upstream's edit of an archived upstream task) names the landing page and upstream's task-212: history of a shared task, which task-229 lets in; backlog files are not edited by hand. An exception to AC #3's "nothing left refers to them".
- The plugin in the zip until task-231 (scripts/package.sh needs the bundles): no release before task-231; the plugin's Send bounds (system_dialog.rs) and its menu_items early return go with task-231 (added to its description).
- Commit area "tools" and the task files appearing as Done in the merge commit: as task-224 did.

Review round 2 (agent-reviewer): send back for the records only, the code fixes checked. A: the corpus decision is now in the final summary with the question, the options and the user's answer, and upstream's recorded corpus results for each engine commit. B: AC #3 reworded to name the archived task-175 exception. C: task-231's description was written by the supervisor, who created the task in this session (as task-229..231 were); no agent has started it, so no agent's file was overwritten.

The rewording renumbered the criteria: the site criterion is now #6, the build and test criterion #4, the screen reader tests #5.

Review round 3 (agent-reviewer): records only. The final summary now counts the engine commits (seven touch crates/orb-engine, six cite their corpus runs, 4e3d22b is the test profile) and explains f84d164's 175 (164 + the 11 mod songs); AC #6 says 'nothing in the tree ... except'. The reviewer cannot see the user's answer to the corpus question; the supervisor quoted it to the reviewer from the question tool's result.

Review round 4 (agent-reviewer): everything merge-ready; its one condition was the user's own acceptance of the corpus waiver, which it cannot see. The supervisor holds it: the user answered the question in this session with 'Accept upstream's check' (2026-11-18). Merged by the supervisor on that basis. Landing shape: a two-parent merge commit fast-forwarded onto main, as task-224, so later upstream merges have 6858253 as their base.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Upstream's main 6858253 (107 commits after 32a642d) is merged into the fork as a merge commit on branch task-230-upstream-6858253, per the user's decisions of 2026-11-18 (task-229). Taken: the panel redesign, the double click to a knob's default, the Edit menu (Undo, Cut, Copy, Paste, Duplicate Pattern), the merging settings save, the optimized test profile, the MIDI Standard Mapping layer, fixes. Left out: upstream's backlog from task-176 and its m-7 (id collisions; AGENTS.md says so) and the GitHub Pages landing page with GoatCounter (removed). The plugin stays until task-231.
The 19 conflicts are resolved as the notes list. The main ones: settings.rs combines upstream's merging save with the fork's keys and portable folders (field renamed portable_root); menu.rs puts the fork's screen reader items and the bar's keyboard walk into upstream's menu_items, with Edit the second of five titles; the fork's DX12 rule stays, with upstream's start error text; upstream's Edit status messages go through set_status, so NVDA hears them.
Engine corpus, the user's decision of 2026-11-18: asked in this session (the question: the CD-image corpus tests could not run here, no ORB_CD_ISO; options: give the image path, accept upstream's check, wait for the test machine), the user chose "Accept upstream's check": the fork's engine is upstream's, unchanged, and is not checked again here with the CD image. Seven upstream commits in this merge touch crates/orb-engine (git log 32a642d..6858253 -- crates/orb-engine); six record their corpus runs, and the seventh, 4e3d22b, is the test profile that runs the corpus workloads in full: f84d164 "Corpus: 175 of 175 equal (corpus_golden diag 145, app 11, shipped 8; orb compare with the recrypted CD mods 175 of 175)" (175 is those 164 plus the 11 mod songs), f50af89 "corpus_golden 4/4 pass, bit-exact", 8b2e54c "corpus_golden 4 pass, no existing engine path changed", c91b55f "corpus_golden 4/4 zero differences", 961e4cc "corpus_golden 4/4 pass, renders unchanged", 79e0033 "Corpus unchanged". Here the no-image corpus_golden sets passed in cargo test --workspace.
Tests: cargo test --workspace 1151 passed, 0 failed, 6 ignored; after the review fixes cargo test -p orb-ui 546 passed, 4 ignored; cargo build --release; clippy -D warnings and nightly fmt --check clean; gated tests skipped.
<!-- SECTION:FINAL_SUMMARY:END -->
