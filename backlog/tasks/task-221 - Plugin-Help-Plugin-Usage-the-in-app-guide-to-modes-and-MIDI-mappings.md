---
id: TASK-221
title: 'Plugin: Help > Plugin Usage, the in-app guide to modes and MIDI mappings'
status: Done
assignee:
  - '@agent-worker'
created_date: '2026-11-16T15:59:00Z'
updated_date: '2026-11-17T11:41:00Z'
labels:
  - ui
  - docs
milestone: m-7
dependencies:
  - TASK-202
  - TASK-203
  - TASK-218
  - TASK-219
  - TASK-228
  - TASK-231
  - TASK-233
references:
  - docs/plugin.md
  - docs/midi.md
  - docs/standard_midi_mapping.md
  - crates/orb-ui/src/menu.rs
  - crates/orb-ui/src/embed.rs
  - backlog task-185 (final summary)
  - backlog task-216 (final summary)
priority: high
type: docs
ordinal: 228000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Why (the user, 2026-11-16):** documentation is the last thing before the first VST3/CLAP plugin release. A plugin user needs one place that explains how to drive the plugin from a DAW. It goes into the plugin editor's Help menu as "Plugin Usage": an in-app page in orb-ui's style, readable at the plugin's window sizes, scrollable, opened from the menu strip. The plugin has no keyboard shortcuts, so it is opened and closed with the mouse.

**Write it last.** This task depends on the remaining plugin features, so the guide describes the released behavior. Take the facts from the code and from docs/plugin.md, docs/midi.md and docs/standard_midi_mapping.md, not from memory; where they disagree, fix the doc that is wrong and say so in the notes.

**Content, short and task-oriented (what to put in the DAW and what happens):**
1. **Modes (the panel's mode selector).** Latch Pattern Mode, Gate Pattern Mode and Note Mode: what each does with the host playing and stopped. The preview on the panel's Play/Stop: it ignores host input, keeps its own preview set and uses the host's tempo. Which mode a new instance starts in.
2. **Channels.** Devices on MIDI channels 1..4: 1 and 2 the ORB-303s, 3 the ORB-808, 4 the ORB-909. The Standard Mapping (remote) channel, 16 by default and off by default, and how to turn it on (task-209's UI).
3. **Pattern selection (both pattern modes).**
   - Notes 0..31 select patterns A1..D8 (0 = A1, 7 = A8, 8 = B1, 31 = D8). Show them as note names in the DAW's usual octave naming, and say which naming is used.
   - Latch: when a change takes effect (the next bar line).
   - Gate: start and stop at once, the last held note wins, the host-position step.
   - Notes never change parameters, so automation doesn't record MIDI pattern changes; the clip is the record.
4. **Note mode mappings.**
   - ORB-303: the key range (MIDI 0..83, C at Tune center = MIDI 36, concert pitch). Accent is velocity >= 100. Slide comes from overlapping notes, and an overlap of the same pitch is a tie.
   - ORB-808 and ORB-909: the GM drum notes per instrument, as a table.
   - Velocity: the 808 accents at >= 100. The 909 has four bands: 1-47, 48-95, 96-111, 112-127.
   - The shared-slot voices of the 808: what picks the voice.
   - The section On/off switch and note mode.
   - The visual feedback (task-202).
5. **Host integration.**
   - Every visible control is a host parameter, ORB-303 1 Cutoff and so on. Automate it or MIDI-learn it with the host's tools.
   - What isn't a parameter: the mode, the preview set, pattern data.
   - Latency of 64 samples; the state is saved with the project, including the mod.
   - Dragging a pattern out as MIDI (task-203).
6. **A short "Recipes" part:**
   - a Reaper-style setup with one MIDI track per device, each sent to the plugin track with its channel;
   - arranging with Gate mode;
   - live pattern switching with Latch.

**Also:** the same text as docs/plugin_usage.md, generated from or kept identical to the in-app source by a test, so it can be linked from the README and the release notes. The user reviews the page in the plugin before merge: screenshots at 1x and 2x, plus the user's check in Reaper.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 The plugin editor's Help menu has Plugin Usage, opening a scrollable in-app page that works at the plugin's window sizes without keys
- [x] #2 The page covers the modes and preview, channels, pattern selection in Latch and Gate, note-mode mappings (303 range, accent, slide/tie; GM drum notes; velocity bands), host parameters/automation, latency/state, drag-out, and recipes
- [x] #3 Every fact is checked against the code and docs/plugin.md/midi.md; disagreements fixed and listed in the notes
- [x] #4 docs/plugin_usage.md holds the same text, kept identical by a test, and the README links it
- [x] #5 Screenshots at 1x and 2x; the user reviews the page in Reaper before merge
- [x] #6 cargo test, clippy -D warnings and nightly fmt pass
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. docs/plugin_usage.md: the guide, written from docs/plugin.md, docs/midi.md and the code (core.rs, tr808.rs/tr909.rs gm_note, params.rs), no Standard Mapping or channel 16 (user, 2026-11-16).
2. orb-ui plugin_usage.rs: the page IS the file (include_str!, a Markdown-subset parser), a scrollable resizable window; Steinberg's negative logo drawn unaltered from the asset bytes at 96 x 61.7 pt, VST registered sign at the first VST, the trademark sentence.
3. Help > Plugin Usage... in the plugin's Help menu only (Cmd::PluginUsage); the standalone's menu unchanged.
4. Tests: orb-ui unit tests (file shown whole, trademark terms, README link, subjects), plugin tests (menu item, opens, close button, smallest window), standalone has no item.
5. editor_preview: --scroll, screenshot after the last click; screenshots at ppp 1 and 2; bundles + validate-plugin.sh; standalone --screenshot byte-compare against main.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
The user, 2026-11-16: no remote MIDI channel in the first release (task-208/209 deferred). The page doesn't describe the Standard Mapping or channel 16; it says only that the devices listen on channels 1..4. Dependency on task-209 removed.

From task-233 (Help > Plugin Usage, the About-box equivalent of Steinberg guidelines 1e): show the official VST Compatible Logo unaltered, from assets/steinberg/VST_Compatible_Logo_Steinberg.svg (black box, light UI) or VST_Compatible_Logo_Steinberg_negative.svg (white box, dark UI; both viewBox 116.2 x 74.7, scale in proportion only, never recolor), at least 15 x 10 mm on screen (the README uses 120 px wide at 96 dpi, about 32 x 20 mm; keep at least 57 x 38 px at 1x), near the sentence 'VST is a registered trademark of Steinberg Media Technologies GmbH.', and put the registered sign at the first use of VST on the page (VST® 3). Source and terms: assets/README.md.

Branch: worktree-agent-a868fd8f83033f5ca (rebased onto main 9f3e469).

Facts checked against the code and docs (AC 3). Disagreements between the task text and the released behavior, the page follows the code: (1) the task said Latch changes take effect at the next bar line; core.rs (Mode::Latch, DeviceParam + Relocate while the song plays) and plugin.md 8.2 say a NOTE selects at once, in step, and only a panel or automation change waits for the bar line: the page says both. (2) the task spoke of the Options menu's modes: task-231 moved them to the panel's selector; the page says so. (3) no Standard Mapping or channel 16 (the user, 2026-11-16). (4) task-185's drum tables: gm_note in tr808.rs/tr909.rs equals docs/midi.md 5.4 (36 BD .. 75 CL; 909 49 CC, 51 RC; 808 51 none, 56 CB). 303 range 0..83 at Tune center, accent >= 100, 909 bands velocity_band 1-47/48-95/96-111/112-127: all as docs/midi.md. No document needed a fix; docs/plugin.md gained the Help > Plugin Usage bullet, the editor_preview options and the test list. Not confirmed here (the user's Reaper check, AC 5): the exact wording of Reaper's send MIDI channel settings in recipe 1 step 3 and of Param > Learn (the latter is from plugin.md 8.2); the note-name octave convention per DAW (Reaper/Cubase/Bitwig C4=60, Ableton/Logic C3=60) is general knowledge, not read from a DAW here.

Implementation: crates/orb-ui/src/plugin_usage.rs embeds docs/plugin_usage.md (include_str!) and draws it from a Markdown subset (headings, paragraphs, bullets, numbered items, tables, **bold**, code, the <picture> block as the logo): the page IS the file, so they cannot differ; tests assert every word of the file's text lines is on the page, the VST registered sign at the first VST of the page, the exact trademark sentence, the negative logo bytes (white box, viewBox 116.2 x 74.7) drawn at 96 x 61.7 pt (>= 57 x 38 px at 1x, same aspect), no tint, and that the README links docs/plugin_usage.md. The page uses the negative variant because the window is dark; the md file carries the README's <picture> element (light and dark variants, 120 px) for GitHub. Help > Plugin Usage... is the first item of the plugin's Help menu only (menu.rs, Cmd::PluginUsage, ignored outside a plugin), above About. Window: resizable, scrollable, centered, 200..580 x 150..520 pt clamped to the window; the smallest plugin window (312 x 277) is tested. editor_preview gained --scroll Y and takes its screenshot 24 frames after the last click (the window had not faded in at frame 30).

Evidence: cargo test: 885 passed, 0 failed, 5 ignored (whole workspace, no ORB_* variables set); new tests: orb-ui lib plugin_usage::tests x5, orb-plugin tests/editor.rs the_help_menu_opens_the_plugin_usage_page and the_plugin_usage_page_fits_the_smallest_window, orb-ui tests/menu.rs the_standalone_has_no_plugin_usage. cargo clippy --all-targets -- -D warnings clean; cargo +nightly fmt --all --check clean. Standalone screenshot: a main build (git archive of main in a scratch folder, its own target dir) and this branch both give the same sha256 for 'OpenRB-338 --screenshot' (a85d9fff...; one of three main runs differed, the logo animation's timing; --demo screenshots differ run to run on main itself and were not compared); the standalone's Help menu is unchanged (test). Bundles: cargo xtask bundle orb-plugin --release in this worktree, scripts/validate-plugin.sh: clap-validator and pluginval pass ('validators: all passed'); not installed. Screenshots (ppp 1 is the system scale, a Retina point is 2 px; ppp 2 twice that) in /private/tmp/claude-501/-Users-pingw33n-dev-OpenRB-338/a9b177cc-ff45-4931-aea4-b72986a77486/scratchpad/a868/shots/: plugin_usage_top_{1x,2x}.png, plugin_usage_scroll{800,1600,2400}_{1x,2x}.png, plugin_usage_smallest_1x.png (312 x 277), plugin_help_menu_1x.png. Reproduce: cargo run --release -p orb-plugin --example editor_preview -- --ppp 1 --format vst3 --click 150,11 --click 190,39 [--scroll 800] --screenshot OUT.png. AC 5's user review (the page in Reaper) is open; status stays To Do until the user has reviewed.

Review round 1 (send back), all fixed: (1) no internal names on the page: 'orb' -> 'The plugin', 'the original' -> 'ReBirth'. (2) latency: 44.1 kHz reports 64; other rates the resampler delay rounded up (105 at 48 kHz, 209 at 96 kHz; core.rs latency(), resample.rs latency()); lateness of an event is up to one 64-frame engine block (1.45 ms) at any rate (blocks start every 64 engine frames, an event waits for the next one), stated that way. (3) drag-out names the four clip limits (plugin.md section 12). (4) per-DAW middle-C list dropped: the page calls MIDI 60 C4, says some DAWs call it C3 (names one octave lower) and tells the reader to go by the note numbers, which the tables give. (5) 1.45 ms. (6) 'the FX window's Param button > Learn'; the Reaper send MIDI channel wording (recipe 1 step 3) is kept and is for the user's check (AC 5). (7) below Accent 25 the page says the 96 to 111 band is quieter than the 48 to 95 band (the same at 24), as docs/midi.md 352-353 say (the review wrote 'sounds as', the doc says quieter). (8) parse(): a top-level markup line that is not <picture> is consumed (it made the paragraph loop consume nothing and the outer loop push empty paragraphs forever); new test parse_terminates_on_odd_input (stray </picture>, unclosed <picture>, cut tables, bare markers, empty input). (9) the Keys line says Escape cancels a drag-out on Windows. Rerun: cargo test 886 passed 0 failed; clippy and nightly fmt clean; screenshots retaken (same names), bundles rebuilt, validate-plugin.sh all passed.

Review round 2: the latency sentence is exact now: at 44.1 kHz an event sounds 0 to 63 samples late and never early (core.rs); at other rates up to one 64-frame engine block (1.45 ms) late and, the reported latency being rounded up, up to a fraction of a sample early (resample.rs). cargo test rerun: see commit. The latency text is below the first screens (scroll about 2400 pt); the earlier screenshots show other text, so no retake needed beyond the scroll2400 pair, retaken.

The user's edits of 2026-11-17 applied to docs/plugin_usage.md: 'The preview' is Preview mode throughout; the Reaper Param > Learn parenthesis removed; 'Not parameters' reworded (the mode, Preview mode and the control changes made in it, the pattern data); '(or a ReBirth 1.x song)' removed; the clip-limits sentence removed and 'plays the pattern as the pattern sounds' softened to 'plays the pattern'. cargo test, screenshots, bundles and validate-plugin.sh rerun.

The user's change of 2026-11-17: the VST section (official logo + trademark sentence) moved to the bottom of docs/plugin_usage.md, between the marker lines '<!-- vst3-only -->' and '<!-- /vst3-only -->'. plugin_usage::page_text(vst3) gives the file as is for VST3 and without the marked section (markers included) for CLAP; the window takes the format from Embed::format (CLAP omits it; VST3 and no format keep it). The registered sign stays at the first VST in the intro. Tests: the VST3 page equals the file and ends with Logo + the sentence; the CLAP page equals the file minus exactly the marked lines (and the section is the last thing in the file), has no logo or sentence, and still has VST(R) in the intro; the older page tests run on the VST3 page. plugin.md updated. editor_preview now waits 1.5 s of the clock after the last click/scroll before the screenshot: on a loaded machine (load 28) the shots caught the window mid-fade with unsettled textures. Screenshots incl. bottom of the page for VST3 and CLAP retaken (plugin_usage_{vst3,clap}_{top,scroll*,bottom}_{1x,2x}.png, plus smallest_bottom). cargo test 888 passed, clippy and nightly fmt clean, bundles rebuilt, validate-plugin.sh all passed. Disclosure: one docs/plugin.md sentence was changed with sed by mistake of habit (a number), the content equals what Edit would give.

Review round 4: the glued Drag-out and Keys bullets in docs/plugin_usage.md are two lines again; new test every_list_item_is_a_line_of_its_own (the number of the file's list lines equals the page's list items, and no glued '.- ' item); the module doc's orphan line break fixed. Edit tool only (the earlier sed slip stays noted). cargo test, clippy, nightly fmt --check, bundles and validate-plugin.sh rerun. The shot of the page did not change except the restored Keys bullet.

User review, 2026-11-17: the user approved the page in the plugin in REAPER, in VST3 and in CLAP (the VST section at the bottom, absent in CLAP; the intro's VST(R) 3 stays in both). Rebased onto main 2dc0dce; cargo test 889 passed 0 failed, clippy and nightly fmt clean.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Help > Plugin Usage is in the plugin's Help menu: a scrollable, resizable in-app page (crates/orb-ui/src/plugin_usage.rs) that draws docs/plugin_usage.md itself (embedded, so file and page cannot differ), covering the modes and Preview mode, channels 1..4 (no remote channel), pattern selection in Latch and Gate with note names and numbers, the 303/808/909 note-mode mappings and velocity, host parameters, latency, state, the pattern drag-out and three recipes, written from the code and docs/plugin.md and midi.md. The VST section (Steinberg's official logo and the trademark sentence) is at the bottom of the file between vst3-only markers; the VST3 page shows it and the CLAP page omits it (Embed::format); the intro keeps VST(R) 3. The README links the file. The standalone's Help menu and --screenshot are unchanged. The user approved the page in REAPER (VST3 and CLAP) on 2026-11-17. cargo test 889 passed 0 failed, clippy -D warnings and nightly fmt clean, bundles built and clap-validator/pluginval pass. Branch worktree-agent-a868fd8f83033f5ca, one commit on main 2dc0dce.
<!-- SECTION:FINAL_SUMMARY:END -->
