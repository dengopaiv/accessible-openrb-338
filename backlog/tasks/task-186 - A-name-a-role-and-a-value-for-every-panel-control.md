---
id: TASK-186
title: A name, a role and a value for every panel control
status: Done
assignee:
  - '@agent-complex'
created_date: '2026-11-05T19:31:00Z'
updated_date: '2026-11-05T21:27:00Z'
labels:
  - ui
milestone: m-7
dependencies:
  - TASK-183
  - TASK-181
references:
  - assets/ui/layout.tsv
  - crates/orb-ui/src/app.rs
  - crates/orb-ui/src/model.rs
  - docs/ui_spec.md
  - docs/accessibility.md
  - docs/accessibility_plan.md
priority: high
type: feature
ordinal: 209000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Planned in docs/accessibility_plan.md as A11Y-2; the analysis is docs/accessibility.md. Read both before starting.

- Why: today 250 of the 254 focusable elements have no name and no role
  (docs/accessibility.md 1.1).
- What: `crates/orb-ui/src/a11y/names.rs`: for each layout id its spoken name ("Cutoff",
  "Bass drum level", "Channel 3 mute") and its section's name ("Bass Line 1", "TR-808"), as a
  table checked against `layout.tsv` by a test; value text per kind (the 0..127 number for
  knobs and faders until A11Y-14 gives their units, words for switches and pan, the displays'
  units); roles: Slider (knob,
  fader), CheckBox or ToggleButton (toggle), RadioButton (radio), Button (trigger), the switch
  as a two-state control with its two words. Non-interactive parts (LEDs, meters, the step
  parts, the click areas, the mod picture) get no focus: the panel's `ui.interact` for them
  uses a sense without focus, or they get no node. The node data is written with
  `Context::accesskit_node_builder` or `Response::widget_info` in the place each control is
  drawn. Note that egui 0.37 deprecates `WidgetType` (now `egui::Role`) and `Id::new`.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 A test walks `layout.tsv`: every interactive part (every kind except led, meter, steps, area, modview, display) has a node with a non-empty name, the role of its kind and, for a value control, a value; no two nodes of one section share a name.
- [x] #2 No LED or meter node can take the focus; the number of focusable nodes is recorded in the baseline test (A11Y-0) and is lower than today's 254.
- [x] #3 `tools/uia_tree.ps1` on the release build shows the names (recorded in the notes).
- [x] #4 `crates/orb-ui/tests/frame_cost.rs` also measures a frame with AccessKit on (it enables it on its context; today it does not) and prints the time of building the tree; the shape, mesh and vertex bounds it asserts still hold, and the times before and after are in the notes.
- [x] #5 With screen reader mode off, the keyboard and interaction tests pass unchanged.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Measure first: run tests/frame_cost.rs (release) as it is, and with a temporary AccessKit-on variant, to record the "before" times.
2. New module crates/orb-ui/src/a11y/ (mod.rs, names.rs):
   - names.rs: the spoken name of every layout id (a table for the fixed ids, rules for the numbered families: pattern buttons, banks, 303 keys and LEDs, steps, gain reduction LEDs), the spoken section of every layout section (A11Y-D3: patternN and mixerN go to their machine's section: Bass Line 1, Bass Line 2, TR-808, TR-909; master and comp to "Master and compressor"; PCF, Delay, Distortion, Transport), the role of each kind (knob, fader Slider; toggle CheckBox; radio, grid RadioButton; trigger, repeat, wind Button; switch, hswitch, vswitch Switch with its two words as the value), and the value text: knobs and faders through one function `control_value_text(id, v)` (the 0..127 number, pan as "center", "left N", "right N"; TASK-192 extends it with units), switches as words, displays with their units ("140 BPM", "bar 3", "16 steps", "off" when disabled).
   - mod.rs: `describe(ctx, id, panel, ctl)` writes role, name, value (text, numeric value 0..127 with min, max and step) and toggled state into the hit area's AccessKit node with `Context::accesskit_node_builder` (no cost when AccessKit is off: the closure does not run); `describe_display` makes a non-focusable Label node for each display ("Tempo 140 BPM").
3. app.rs: call describe in `control`, `knob` and `fader` after the interaction; displays get their node. The step strips' and instrument selection areas' `ui.interact` use `Sense::CLICK | Sense::DRAG` (no FOCUSABLE): same pointer behavior, no Tab stop, no node. LEDs and meters already have no `ui.interact` and stay so.
4. Tests: tests/a11y_names.rs walks layout.tsv (AC1: every interactive part has a node with a non-empty name, the role of its kind and, for knobs, faders and switches, a value; toggles and radios a toggled state; names unique within a spoken section; no LED or meter node is focusable); unit tests in names.rs (table vs layout.tsv, value texts); a11y_baseline.rs updated with the new counts (focusable lower than 254); frame_cost.rs measures also with AccessKit on and prints the tree-building time.
5. tools/uia_tree.ps1 on the release build; output in the notes. docs/accessibility.md (1.1 follow-up, 3.3) and docs/ui_spec.md get the names and roles.
6. cargo test -p orb-ui, clippy, nightly fmt; one commit.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: worktree-agent-ad02d87aea61f5d1c

Research (2026-11-05):
- The 254 focusable elements of docs/accessibility.md 1.1 are: 244 interactive layout parts through `App::hit` (77 knobs, 9 faders, 46 triggers, 48 grid, 23 toggles, 9 radio, 18 repeat, 4 wind, 1 switch, 2 hswitch, 7 vswitch), 6 drum-machine click areas (`App::drum_strips`: strip808, strip909; `App::instrument_selection`: legend808, upper808, legend909, names909), all `Sense::click_and_drag()` (includes FOCUSABLE, egui sense.rs:81), and the 4 menu titles.
- LEDs and meters already have no `ui.interact` (`App::control`, kind "led" | "meter" only paints); "area" and "steps" layout entries have none either (the six areas above are separate rectangles). So AC2's drop comes from the six click areas: `Sense::CLICK | Sense::DRAG` keeps click and drag (the response's pointer state is the same; egui only skips `interested_in_focus` and the AccessKit node, context.rs:1520-1565).
- egui 0.37: `Context::accesskit_node_builder` runs its closure only with AccessKit on (returns None otherwise, context.rs:4203), so the descriptions cost nothing in a normal session.
- accesskit_consumer 0.39.2: a Label's name comes from its value unless a label is set (`label_comes_from_value`, node.rs:756; `write_label_direct` 778), and accesskit_windows 0.35.2 gives a Label no Value pattern (node.rs:592-597). So the displays carry their text in the label: "Tempo 140 BPM".
- Role::Switch maps to a UIA Button with localized type "switch" (accesskit_windows node.rs:102, 370); its value text (the position's word) is exposed through the Value pattern. No toggled state on switches (on/off would be wrong for saw/square).
- Naming choice: the mixer's on/off button is "Channel N on" (lit = the channel plays, docs/ui_spec.md 8), not "Channel N mute" as the task's example says: a checked "mute" would read inverted.

Frame cost BEFORE (main's app code, frame_cost.rs extended, release, 200 frames):
native 1x: 559 shapes, 524 meshes, 14112 vertices, 112 textures, 0 nodes; show 0.537 ms, tessellate 0.475 ms
1800x1100 at 2x: 559 shapes, 534 meshes, 16412 vertices, 161 textures, 0 nodes; show 0.629 ms, tessellate 0.482 ms
native 1x with AccessKit: 261 nodes; show 0.695 ms (tree +0.158 ms)
1800x1100 at 2x with AccessKit: 261 nodes; show 0.683 ms (tree +0.054 ms)

Tree after (headless, a11y_baseline.rs): nodes 267 (was 261), named 260 (4), Unknown role 0 (250), valued 96 (0), focusable 248 (254). Roles: Button 72, CheckBox 23, RadioButton 57, Slider 86, Switch 10, Label 12, GenericContainer 4, Menu 1, Pane 1, Window 1.

Implementation:
- crates/orb-ui/src/a11y/names.rs: `section_name`, `role`, `is_interactive`, the NAMES table (by `family.name`, `@kind` for the 303's accent knob and button, `{n}` for the mixer channel), `numbered` (pattern and bank buttons, 303 keys and their lights, drum steps, gain reduction lights), `name`, `control_value_text` (the one function TASK-192 extends), `switch_words`, `display_text`, `value_text`; 6 unit tests (every layout part named and in a section, every table key in layout.tsv, names unique per spoken section over all 348 parts, sample names, value texts, value_text per kind on the demo panel).
- crates/orb-ui/src/a11y/mod.rs: `describe` (role, label, value text, numeric value 0..127 with min/max/step for knobs and faders, toggled for toggles, radios and pattern/bank buttons), `describe_display` (a non-focusable Label "Tempo 140 BPM"), `hit_id`.
- app.rs: `describe` after the interaction in `control`, `knob`, `fader`; `describe_display` for displays; `hit` uses `a11y::hit_id`; the step strips and instrument-selection areas use `Sense::CLICK | Sense::DRAG`. model.rs: `Panel::grid_lit` (moved out of `part_name`, same logic).
- First UIA run showed the 12 Text elements unnamed: accesskit_windows names a Label from its value (`name`, node.rs:404-413: `label_comes_from_value` -> `write_value`). Fix: the display node carries the text as value and as label. Baseline valued count becomes 108 (86 knobs/faders, 10 switches, 12 displays).
- Rebased onto main 650d17f (TASK-185 merged) without conflicts.

AC3, tools/uia_tree.ps1 on the release build (worktree target\release\OpenRB-338.exe, --demo), after the fix:
window: 'OpenRB-338 - Untitled'
elements: 262
by type, named:
    86  Slider, True
    82  Button, True
    57  RadioButton, True
    23  CheckBox, True
    12  Text, True
     1  Menu, False
     1  Pane, False
can take the focus: 248, of them with a name: 248
names: Shuffle amount | Tempo up | Tempo down | Mode | Play | Stop | Rewind | Fast forward | Record | Bar up | Bar down | Loop start up | Loop start down | Loop length up | Loop length down | Loop | Pattern section on | Pattern 1 .. Pattern 8 | Bank A .. Bank D | Shuffle | Pattern length up | Pattern length down | (x4) | Waveform | Tune | Cutoff | Resonance | Envelope mod | Decay | Accent | Step forward | Step back | Pitch mode | Clear pattern | Key C .. Key high C | Octave down | Octave up | Step accent | Slide | Note or pause | (x2) | Accent level | Bass drum level | ... | Hand clap or maracas switch | (TR-909 knobs) | Flam | Flam entry | Channel 1 on | Channel 1 pan | Channel 1 delay send | Channel 1 level | Channel 1 distortion | Channel 1 PCF | Channel 1 compressor | (channels 2-4) | Master level | Master compressor | PCF on | PCF pattern up | PCF pattern down | PCF mode | Frequency | Q | Amount | Decay | Delay on | Delay steps up | Delay steps down | Step length | Delay pan | Feedback | Distortion on | Amount | Shape | Compressor on | Ratio | Threshold | File | Mods | Options | Help | Tempo 140 BPM | Bar off | Loop start off | Loop length off | Pattern length 16 steps (x4) | Edit step 1 (x2) | PCF pattern 0 | Delay steps 3
(-Dump: the 10 switches are UIA Button elements, localized type "switch"; before: 256 elements, 254 focusable, 4 named.)

AC4, frame_cost.rs release (200 frames per case, this machine is shared and noisy):
- before (main's app, 261 nodes): off 0.537 / 0.629 ms show; on 0.695 / 0.683 ms; tree +0.158 / +0.054 ms (native 1x / 1800x1100 at 2x).
- after (267 nodes), three quiet runs: tree +0.478 / +0.355 ms (final run: off 0.539 / 0.597, on 1.017 / 0.951 ms show); -0.033 / +0.298; +0.795 / -0.013. One run under load (other builds): off 2.743 / 3.200, on 4.936 / 3.949.
- So with AccessKit on, the named tree costs -0.03 to +0.80 ms of show per frame over the three quiet runs (median +0.48 ms native 1x, +0.30 ms at 2x), within this machine's noise; the before figure (+0.16 / +0.05 ms) is from one run. Nothing with AccessKit off. Shapes 559, meshes 524 / 534, vertices 14112 / 16412 in all four cases: the bounds hold.

Tests: cargo test -p orb-ui (debug), after the rebase: all pass (lib 86, main 5, a11y_actions 1, a11y_baseline 1, a11y_names 4, browser 22, cli_args 3, crash 2, error_box 15, frame_cost 1, interaction 15, keyboard 12, menu 54, mod_name 2, mod_order 1, part_textures 2, repaint 5, settings 9, song_mode_save 3, switch_values 2, texture_size 2, theme 1, wired 22). keyboard.rs, interaction.rs and wired.rs unchanged and passing (AC5; screen reader mode is not read by this task). clippy --all-targets -D warnings clean; nightly-2026-10-18 fmt --check clean.

The second full run's grep showed no result line for tests/menu.rs (the slow debug export test overlapped with another agent's menu test run); rerun alone: cargo test -p orb-ui --test menu: 54 passed, 0 failed. Total cargo test -p orb-ui after the rebase: 270 passed, 0 failed, 2 ignored. The known release-only crash.rs failure was not run (debug only); crash.rs passes in debug.

Second rebase onto main 582a030 (TASK-184, the menu bar from the keyboard), no conflicts. Its new test tests/a11y_menu_bar.rs `the_options_items_read_their_state` counted every RadioButton in the tree (expected the menu's 2, got 59 with the panel's 57 pattern, bank, PCF and Comp buttons); the test now leaves out the nodes of the panel's layout parts (`a11y::hit_id` of each `layout::controls()` entry) and passes. After the rebase: cargo test -p orb-ui (debug) 277 pass (a11y_menu_bar 7 added), 0 fail, 2 ignored; clippy -D warnings and nightly fmt --check clean; tools/uia_tree.ps1 on the rebuilt release build gives the same 262 elements, 248 focusable, all named.

Review round 1 (send back, text only): (1) docs/accessibility.md 6.3 said the 82 UIA buttons were 72 trigger, repeat and wind buttons and 10 switches; the layout has 46 + 18 + 4 = 68, and the 4 menu titles are buttons too. Fixed, and the paragraph now gives the make-up of the 86 sliders (77 knobs, 9 faders), 57 radio buttons (48 pattern/bank, 9 PCF/Comp) and 23 check boxes (toggles), all checked against layout.tsv. (2) The timing claim (0.3 to 0.5 ms) came from the final run only; the notes, the final summary and the commit body now give the range over the three quiet runs (-0.03 to +0.80 ms, medians +0.48 / +0.30 ms) and say that the before figure is from one run. (3) Nit taken: tests/common/a11y.rs `hit_id` now calls `orb_ui::a11y::hit_id_of` (new; `a11y::hit_id` uses it), so a change of the key is seen by the tests. a11y_actions 1, a11y_baseline 1, a11y_menu_bar 7, a11y_names 4 pass; nightly fmt --check and clippy -p orb-ui --all-targets -D warnings clean.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Every panel control now has a name, a role and its value or state in the AccessKit tree (A11Y-2).

- New crates/orb-ui/src/a11y/ (mod.rs, names.rs): the spoken name of all 348 layout parts (a table plus rules for the numbered families, checked against layout.tsv by unit tests), the spoken section of each (A11Y-D3: pattern section and mixer channel in the machine's section), roles by kind (Slider, CheckBox, RadioButton, Button, Switch with its two words, Label for displays), value texts (the 0..127 number for knobs and faders through `control_value_text`, which TASK-192 extends; pan in words; switch words; display units). Written into each hit area's node with `Context::accesskit_node_builder` in `App::control`, `knob`, `fader`; no cost with AccessKit off.
- The six drum-machine click areas sense click and drag without FOCUSABLE (mouse unchanged); LEDs and meters had no hit area and still have none.
- Headless tree (a11y_baseline.rs): 267 nodes (was 261), 260 named (4), 0 Unknown (250), 108 valued (0), 248 focusable (254). UI Automation on the release build: 262 elements, 248 focusable, all 248 named.
- frame_cost.rs measures each size with AccessKit off and on and prints the tree's cost: -0.03 to +0.80 ms of show with AccessKit on over three quiet runs (median +0.48 ms native, +0.30 ms at 2x; noisy shared machine); before: +0.16 / +0.05 ms for the unnamed tree, from one run; shape, mesh and vertex counts unchanged and within the bounds.
- New tests/a11y_names.rs (4 tests, AC1 and AC2). Name choice to note: the mixer's on/off button is "Channel N on" (lit while the channel plays), not "mute".
- docs: docs/ui_spec.md section 20 (names, roles, values), docs/accessibility.md 1.1 and 6.3 (the UIA output after).
- tests/a11y_menu_bar.rs (TASK-184) counted every RadioButton of the tree; it now leaves out the panel's nodes.
- Branch worktree-agent-ad02d87aea61f5d1c, one commit on main 582a030. cargo test -p orb-ui (debug): 277 pass, 0 fail (review round 1 rerun: a11y tests), 2 ignored; clippy -D warnings and nightly fmt --check clean.
<!-- SECTION:FINAL_SUMMARY:END -->
