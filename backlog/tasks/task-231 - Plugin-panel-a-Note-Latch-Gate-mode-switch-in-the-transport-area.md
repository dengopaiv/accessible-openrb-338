---
id: TASK-231
title: 'Plugin panel: a Note/Latch/Gate mode switch in the transport area'
status: Done
assignee:
  - '@agent-worker'
created_date: '2026-11-17T08:06:00Z'
updated_date: '2026-11-17T09:51:00Z'
labels:
  - ui
  - player
  - docs
milestone: m-7
dependencies:
  - TASK-228
  - TASK-232
references:
  - crates/orb-plugin/src/instance.rs
  - crates/orb-plugin/src/editor.rs
  - crates/orb-ui/src/embed.rs
  - crates/orb-ui-art
  - docs/plugin.md
  - task-213
  - task-228
  - task-221
priority: high
type: feature
ordinal: 600
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
The user's idea (2026-11-17): the plugin panel's transport strip has empty space between Play/Stop, Bar and the logo (Fast forward was removed, task-213). Fill it with a 3-position mode switch: Note Mode, Latch Pattern Mode, Gate Pattern Mode (instance.rs `Mode`, today only in the Options menu as radio items). The mode decides what the plugin does with MIDI and a wrong mode is the likeliest "no sound", so it belongs on the panel.

The user settles the design in their own session in this task's worktree (branch task-231-mode-switch). Starting proposal (the supervisor's): between Play/Stop and Bar; a switch in the panel's style labeled NOTE · LATCH · GATE, Note left (the default); click a position to select it, no keys (the plugin's no-keyboard rule, docs/plugin.md); the Options menu radio items stay and follow it; a one-line tooltip per position; optionally Bar shows "---" in Note Mode. New art parts go in orb-ui-art; the standalone app's panel is unchanged (its --screenshot byte-identical).

Write the settled design into this description or the plan before implementing. task-221 (Help > Plugin Usage) should describe the switch.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 The design is recorded in the task (placement, look, labels, behavior).
- [x] #2 The plugin panel shows the switch; clicking a position (or dragging the knob) sets the mode, and the Options menu has no mode items (the user's decision of 2026-11-17: the selector is the only place; tests).
- [x] #3 The mode change from the switch is saved with the plugin state.
- [x] #4 The standalone app's --screenshot output is byte-identical to main's; plugin preview screenshots at 1x and 2x.
- [x] #5 docs/plugin.md describes the switch; cargo test, clippy -D warnings and nightly fmt pass; the user's Reaper check is recorded.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
THE SETTLED DESIGN (the user's session of 2026-11-17, in this worktree; it replaces the starting proposal of the description where they differ)

A working visual mockup of everything below is in this worktree, branch task-231-mode-switch, as UNCOMMITTED changes: crates/orb-ui-art/src/parts.rs (gray_knob_cap, the PLUGIN_MODE_* constants, transport_plugin_bg, the parts plugin_logo_vst3/clap, mode_label_0..2_off/on, led_mode_on), crates/orb-ui-art/src/plugin_logos.rs (new), crates/orb-ui-art/src/lib.rs, crates/orb-ui/src/embed.rs (the block in plugin_ui), crates/orb-ui/src/app.rs (paint_part_rotated is pub(crate)), assets/ui (bg_transport_plugin.svg changed, 9 new SVGs). Start from it: `git status` and `git diff` first. Do not reset or clean the worktree. The user approved its look; keep the geometry and styles unless stated below. See it: `cargo run --release -p orb-plugin --example editor_preview -- --mode latch|gate|note` (ORB_MOCK_LOGO=clap shows the CLAP logo; that variable is mockup scaffolding and must go).

1. Placement (transport coordinates; panel x = transport x + 16). The selector is in the empty space after the Bar display (not between Play/Stop and Bar): the knob's center at (250, 26), the three rows at y = 13, 26, 39, their LEDs at x = 285, the labels from x = 290.5. No "Mode" title (tried above and left of the knob, dropped by the user). The plugin format's logo is right of it, before the wordmark.

2. The knob. The gray knob (Shuffle's design, knob_gray) at 1.6 times its size, without Shuffle's 7 ticks: its collar and cap are drawn into bg_transport_plugin; the pointer is drawn by orb-ui, turned to 45 degrees (Latch), 90 (Gate), 135 (Note); 0 = up, clockwise. The mockup paints the part knob_gray_pointer in a rectangle 1.6 times larger; a part of its own for the pointer is fine if cleaner.

3. The lines. One per position: out of the knob at the position's angle from radius 15.8, turning level at the row's y and running to 4.5 px left of the LED; the middle one is straight. Engraved, not printed: a dark groove (#1b262c, width 0.8, opacity 0.6) over a light copy 0.55 px lower (#edf1f3, width 0.6, opacity 0.35), round caps and joins.

4. The LEDs. One per row, the yellow lamp of the mixer's routing keys (led_round "Y", r 1.9): dark in the background, the selected mode's lit (part led_mode_on, 11 x 11, as led_transport_on).

5. The labels. "Latch Pattern", "Gate Pattern", "Note", top to bottom (the Options menu's order; the menu keeps its wording "Latch Pattern Mode", "Gate Pattern Mode", "Note Mode"). Unselected: exactly the style of Sync and MIDI (7.2 px, weight 500, letter spacing 0.15, fill LIGHT #c9d3d8). Selected: the same size and spacing, weight 700, fill #edf1f3. Parts mode_label_{0,1,2}_{off,on}, 62 x 10, baseline 7.7, drawn by orb-ui at (LED x + 5.5, row y - 5).

6. The plugin format's logo. ONE logo, chosen by the build: the VST3 logo in the VST3 plugin, the CLAP logo in the CLAP plugin (never both; nothing in the standalone app). Engraved and discreet: the logo's path in #1b262c at opacity 0.38 over a copy 0.6 px lower in #edf1f3 at opacity 0.22. As large as the 50 px strip allows: VST3 40 px high (about 66 wide), CLAP 42 px high over its drawn content (about 39 wide), vertically centered at y = 25, RIGHT-ALIGNED about 6 px before the wordmark (the mockup: the 112 x 50 part at transport x = 344 + 19 for VST3, 344 + 32 for CLAP). Path data: crates/orb-ui-art/src/plugin_logos.rs, from https://www.olivierclero.com/art/vst3-logo/ (vst3-logo.zip, svg/vst3-logo-vector-full-black.svg; the author allows its use in a VST plugin without credit) and https://upload.wikimedia.org/wikipedia/commons/a/ae/CLAP_-_Clever_Audio_Plugin_Logo.svg.

7. Behavior. The user settled the look only; the behavior below is the task's starting proposal plus the session's recommendation, and the user did not confirm it in the session. Implement it, say so in the final summary, and let the user's Reaper check (AC #5) judge it:
- a click on a row (its LED, label or line) selects that mode;
- a drag on the knob steps through the three positions (snapping; vertical drag as the panel's knobs); a plain click on the knob does nothing;
- no keys (the plugin's no-keyboard rule, docs/plugin.md);
- the switch calls the same Embed::set_mode as the Options menu, so both always agree and the mode is saved with the plugin state the same way (AC #2, #3);
- no tooltips (the user's decision of 2026-11-17);
- the Bar display needs no change: it is already dark in Note Mode (embed::host_position).

IMPLEMENTATION NOTES

- The format for the logo: the editor must learn which plugin format it runs as. Add it to the Embed trait (for example `fn format(&self) -> Option<PluginFormat>`, default None = no logo) and answer it in crates/orb-plugin/src/editor.rs from the wrapper in use; find how orb-plugin can tell VST3 from CLAP at run time (both formats come from one crate) and record what you find in the notes. Give examples/editor_preview.rs an option (for example `--format vst3|clap`) for the screenshots, and remove ORB_MOCK_LOGO.
- orb-ui-art is only a dev-dependency of orb-ui: the geometry orb-ui needs is duplicated as constants in embed.rs (as PLUGIN_BAR is) and a test checks them against the art (see the_displays_and_the_leds_match_the_art). The mockup has the numbers inline in plugin_ui: make them constants with such a test.
- Regenerate the art with `cargo run -p orb-ui-art --bin ui-art -- gen` and commit assets/ui. The standalone's parts must not change: its --screenshot stays byte-identical to main's (AC #4). The mockup moved the gray knob's body into gray_knob_cap; check knob_gray_body.svg is unchanged (`git diff --stat assets/ui`).
- The mockup's code is a sketch: tidy it (names, doc comments, the hit rectangles in the layout or in plugin_ui), run nightly fmt and clippy.
- editor_preview's --screenshot sometimes saves a blurry frame (taken before the parts are rasterized at full resolution); retake until sharp, and note it. A fix belongs to a follow-up task unless trivial.
- The logos' terms: record the two sources and their terms in assets/README.md. VST is a trademark of Steinberg with its own logo usage guidelines, and the CLAP logo's terms were not checked in the session. If either does not clearly allow this use, create a `Decide:` task for @pingw33n with the options (keep, replace by plain engraved text "VST3" / "CLAP", drop) and make this task depend on it; do not decide yourself.
- docs/plugin.md: describe the selector and the logo; task-221 (Help > Plugin Usage) describes the switch after this task.
- Tests: a click on each row sets the mode and the Options menu's radio item follows, and the menu sets the selector (tests/editor.rs of orb-plugin, headless egui); the mode set from the selector survives a state save and restore; the art constants test.
- Dependency: task-228 (the mode on File > Open, Note Mode default, the mode in the state) is To Do and this task depends on it.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
2026-11-17: the user settled the look in their session in this worktree (branch task-231-mode-switch) and asked for the implementation to be handed to an agent through the supervisor. The design and the handoff instructions are in the plan. The visual mockup is uncommitted in this worktree (see the plan); it has no click or drag handling, no tests, and was not run through cargo test, clippy or nightly fmt.

Branch: task-231-mode-switch (worktree .claude/worktrees/task-231-mode-switch). Mockup committed as WIP, rebased onto main c3b41d6.

2026-11-17 implementation (agent-worker). Rebased the mockup (WIP commit) onto main c3b41d6. Not done: task-228 is not on main yet (this branch has main's File > Open Latch switch; the rebase onto it is the supervisor's next step). Behavior implemented as the plan proposes and NOT confirmed by the user (AC 5's Reaper check judges it): a click on a row (its LED or label; the line itself is not a hit area) selects the mode; a vertical drag on the knob steps through the modes (12 panel px per step, from the mode the drag started in, up toward Latch, down toward Note, ends clamp); a click on the knob does nothing; no keys; nothing is hit while the file browser, a prompt or the error box is open. It calls App::command(Cmd::SetPluginMode), the Options menu's path (Embed::set_mode). Code: orb-ui embed.rs (mode_selector, PLUGIN_MODE_* / PLUGIN_LOGO constants checked against the art in the_mode_selector_matches_the_art, Embed::format, PluginFormat), orb-ui-art parts.rs (PLUGIN_LOGO; the logo's right-aligned shift is baked into the parts now, so orb-ui places both at one x), orb-plugin editor.rs (EditorEmbed::format). ORB_MOCK_LOGO is gone. Format detection: nice-plug GuiContext::plugin_api() (nice-plug-0.4.2 wrapper/vst3/context.rs, wrapper/clap/context.rs) is Vst3 in the VST3 wrapper, Clap in the CLAP wrapper, Standalone otherwise (no logo); editor_preview takes --format vst3|clap (default clap). Logo terms: CLAP = MIT (Alexandre Bique, Wikimedia Commons extmetadata); VST3 = Olivier Clero's redraw, free to use in a plugin, but the VST trademark/guidelines of Steinberg are unconfirmed, so Decide task-232 was created and this task depends on it (assets/README.md records both sources). Tests (orb-plugin tests/editor.rs): the_mode_selector_and_the_options_menu_agree, a_drag_on_the_mode_knob_steps_the_modes, the_mode_from_the_selector_is_saved_with_the_state, the_editor_knows_its_plugin_format; orb-ui: the_mode_selector_matches_the_art, a_drag_steps_through_the_modes_and_stops_at_the_ends. cargo test: 869 pass, 0 fail (whole workspace); clippy -D warnings clean; nightly fmt --check clean. Standalone: orb-ui --screenshot built from main and from this branch compared with cmp: IDENTICAL; assets/ui diff vs main is bg_transport_plugin.svg plus 9 new SVGs only (knob_gray_body.svg unchanged). Preview screenshots (editor_preview --format F --ppp P --mode M --screenshot, a sharp frame each time): /private/tmp/claude-501/-Users-pingw33n-dev-OpenRB-338/a9b177cc-ff45-4931-aea4-b72986a77486/scratchpad/shots/{vst3,clap}-{1x,2x}-{note,latch}.png and {vst3,clap}-1x-gate.png. Bundles: cargo xtask bundle orb-plugin --release in this worktree: target/bundled/OpenRB-338.clap and .vst3 in the worktree, scripts/validate-plugin.sh: clap-validator and pluginval pass (not installed). Task id 232 for the Decide was assigned by the CLI in this worktree: check it does not collide with an id created on main meanwhile.

Review round 1 (2026-11-17): (1) dependencies restored: task-228 and task-232. (2) a long literal split. (3) the row hit rects now start at the knob collar's right edge (PLUGIN_MODE_KNOB_RADIUS = 7.7 x 1.6, checked against the art in the art test) and the knob's drag area is the collar's square, so the whole line is a hit area and the two never overlap; the earlier note 'the line is not a hit area' is wrong, withdrawn; docs/plugin.md says line, LED or label. (4) tests: a_click_on_a_rows_line_selects_it, the_mode_selector_takes_no_hit_under_the_browser_or_the_error_box (a prompt, App::prompt, is pub(crate): not reachable from orb-plugin's tests, not covered), the_format_follows_the_wrappers_plugin_api (Vst3, Clap, Standalone through EditorEmbed::new with a stand-in GuiContext). cargo test 872 pass, clippy and nightly fmt clean, bundles rebuilt and validate-plugin.sh passed. AC 5 still unchecked, status To Do.

Review round 2 (2026-11-17, the user's decisions): (a) task-232 decided: the VST3 panel shows the plain text 'VST3' (Inter bold 28 px, engraved like the CLAP logo), no VST logo; Steinberg's terms and the attribution sentence recorded in assets/README.md and docs/plugin.md (registered sign in the title); the official logo and the attribution for the README, release notes and Help > Plugin Usage are task-233 (task-221 depends on it). The unaltered official logo was tried first and does not fit (screenshots seen). (b) the Options menu's three mode radio items are removed (menu.rs); the selector is the only way to change the mode (Embed::set_mode / Cmd::SetPluginMode stay as its path); AC 2 rewritten; tests: the_options_menu_has_no_mode_items replaces the menu-sets-the-mode tests, a_click_on_a_row_of_the_mode_selector_sets_the_mode; docs/plugin.md and the doc comments updated; task-221's line 'Modes (Options menu)' now says the panel's mode selector. Checks before the rebase onto main c100bbe: cargo test 872 pass, clippy and nightly fmt clean, bundles rebuilt and validate-plugin.sh passed, VST3 1x/2x screenshots in the scratchpad shots folder.

Review round 2 (2026-11-17): the registered sign added at the first VST of the logos section in assets/README.md; the final summary rewritten for the current state (selector only, VST3 text, 873 tests, AC 5 open). Docs only; cargo test rerun.

Review round 3 (2026-11-17, the user): the mode selector's tooltips (the rows' and the knob's, MODE_TIPS) are removed, with their mentions in docs/plugin.md and this task's plan, notes and final summary. cargo test, clippy, nightly fmt, bundles and validate-plugin.sh rerun.

2026-11-17: the user's Reaper check passed on the combined build with the code of 494c728: selector clicks and drag, no Options mode items, the VST3 text and the CLAP mark, the mode kept across a project save and reload ('the rest looks good'). The only change asked, dropping the tooltips, is in 87a0f02 (now this commit).
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
The plugin panel has the Note/Latch/Gate mode selector of the user's design (gray knob, three engraved lines, LEDs, labels) after the Bar display, and the plugin format's mark right of it: the CLAP logo (MIT) in the CLAP build and the plain text VST3 (Inter bold, engraved) in the VST3 build (task-232, the user's decision of 2026-11-17: Steinberg's logo does not fit; its terms and the attribution are recorded, the logo and attribution for the README and Help > Plugin Usage are task-233). The selector is the only way to set the mode: the Options menu's three mode items are removed (the user's decision of 2026-11-17). Behavior, the plan's proposal not yet confirmed by the user: a click on a row (its line, LED or label) selects the mode; a vertical drag on the knob steps through the modes (12 panel px per step, up toward Latch); a click on the knob does nothing; no keys; nothing is hit under the file browser or the error box. It calls Embed::set_mode and the mode is saved in the plugin state. Checks: cargo test 877 pass, clippy -D warnings and nightly fmt clean, the standalone --screenshot byte-identical to main's, VST3 1x/2x and CLAP 1x/2x preview screenshots taken, bundles built and validated (clap-validator, pluginval). Branch task-231-mode-switch, one commit, rebased onto main ae0869d. The user's Reaper check of 2026-11-17 passed (selector clicks and drag, no Options mode items, the VST3 text and CLAP marks, the mode kept across a project save and reload); the only change asked, dropping the tooltips, is done.
<!-- SECTION:FINAL_SUMMARY:END -->
