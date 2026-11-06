---
id: TASK-187
title: >-
  Sliders, switches, spin buttons and the pattern selector that the reader can
  drive
status: Done
assignee:
  - '@agent-complex'
created_date: '2026-11-05T19:32:00Z'
updated_date: '2026-11-06T06:50:00Z'
labels:
  - ui
milestone: m-7
dependencies:
  - TASK-186
  - TASK-205
references:
  - crates/orb-ui/src/model.rs
  - crates/orb-ui/src/behavior.rs
  - assets/ui/layout.tsv
  - docs/accessibility.md
  - docs/accessibility_plan.md
priority: high
type: feature
ordinal: 210000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Planned in docs/accessibility_plan.md as A11Y-3; the analysis is docs/accessibility.md. Read both before starting.

- What:
  - Knobs and faders: numeric value 0..127, min, max, step 1, actions Increment, Decrement and
    SetValue; a request sets the value through `set_knob(id, knob_from_engine(v))` (faders:
    the fader position whose `fader_send` is `v`).
  - Each display with its up and down repeat buttons becomes one SpinButton (tempo, bar, loop
    start, loop length, each row's pattern length, delay steps, PCF pattern) whose Increment
    and Decrement call `Panel::arrow` with the button's id, as one click does; the separate
    buttons and the display leave the focus order.
  - Each row's 8 pattern buttons and 4 bank buttons become one selector, "Pattern B3", whose
    Increment and Decrement move through the 32 patterns with the same `press` calls a click on
    the bank and the pattern button makes.
  - With the focus on such a control, Up/Down, Page Up/Page Down (8 for sliders, 1 bank for the
    pattern selector), Home and End change it (in screen reader mode; A11Y-4 does the routing
    of keys, this task the operations).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 For each kind, a test sends the requests and compares the engine events (`Panel::events`) with those of the same change made by mouse in `crates/orb-ui/tests/interaction.rs`'s way: equal.
- [x] #2 A selection made with the pattern selector during song-mode recording is recorded like a click (a test with a link, as `crates/orb-ui/tests/wired.rs`).
- [x] #3 Values read back as words or numbers ("64" until A11Y-14, "center", "128 BPM", "Pattern B3"), through one function that A11Y-14 extends with the units.
- [x] #4 The tree has one node per spin button and per pattern selector, not the parts.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. New module crates/orb-ui/src/a11y/drive.rs: the operations, pure on `Panel` and unit-tested.
   - Sliders (knob, fader): set through `set_knob(id, knob_from_engine(v))` / `set_fader(id, fader_from_engine(v))` (new `behavior::fader_from_engine`, the expression `Panel::adopt` already uses); Up/Down 1, Page Up/Down 8, Home 0, End 127, clamped, nothing when the value does not change.
   - Spin buttons: the 10 displays with up/down arrows (tempo, bar, loop start, loop length, 4 pattern lengths, delay steps, PCF pattern); Increment/Decrement and Up/Down call `Panel::arrow("<display>_up" / "_down")` once, as one click.
   - Pattern selector per row: value "B3", Increment/Decrement and the keys move through the 32 patterns (Page Up/Down one bank, Home A1, End D8, clamped); each move is `press("patternR.bankX")` when the shown bank differs, then `press("patternR.patternN")`, as the two clicks.
2. app.rs hooks: knob and fader read Increment, Decrement, SetValue requests; the spin displays get a focusable (FOCUSABLE only) hit area with role SpinButton; each row gets one focusable selector node over its 12 buttons (registered where pattern1 is drawn, so the Tab order stays the panel's); the repeat arrows, the bar's two wind arrows and the 48 pattern/bank buttons sense click and drag without FOCUSABLE and get no node. A screen reader's Click on the remaining buttons, toggles and switches does what one mouse down does (only the AccessKit request, not egui's Space/Enter fake click, so mode off is unchanged).
3. Keys, screen reader mode only: with the egui focus on a slider, spin button or selector, Up/Down/Page Up/Page Down/Home/End (no modifiers, repeats honored) act on it and do not reach the original's key map; the focused value control sets egui's focus lock filter for the vertical arrows so egui does not move the focus. Mode off: keyboard() unchanged.
4. Tests: new crates/orb-ui/tests/a11y_drive.rs (AC1 per kind: requests vs mouse drags and clicks, `Panel::events` equal; AC3 value texts; AC4 one node per spin button and selector, the parts have none; keys in mode on, mode off unchanged); a wired.rs test for AC2 (a song-mode recording of a selector change equals the clicks'); update a11y_names.rs and a11y_baseline.rs.
5. docs/ui_spec.md section 20 and docs/accessibility.md 3.3; uia_tree.ps1 counts in the notes; frame_cost run.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: worktree-agent-a4cac0cc4eee34ee5

Research (egui 0.37 sources, ~/.cargo/registry/src/*/egui-0.37.0/src):
- context.rs:1561: only a FOCUSABLE widget gets an AccessKit node from `interact`; so a part interacted with `Sense::CLICK | Sense::DRAG` and not described has no node at all (how the 20 arrows and 48 grid buttons leave the tree).
- hit_test.rs:205-260: clicks and drags go only to widgets that sense them, so a FOCUSABLE-only area (the pattern selector over its 12 buttons, the spin display) takes no click from the buttons.
- context.rs:1759-1770: egui fakes a primary click both for Space/Enter on a focused widget and for an AccessKit Click. The panel's Click handling reads only the AccessKit request (`has_accesskit_action_request`), so Space/Enter on a focused control stays without effect with the mode off.
- memory/mod.rs:568-600: egui moves the focus on the arrows unless the focused widget's EventFilter matches; `Memory::set_focus_lock_filter(id, EventFilter { vertical_arrows: true, .. })` set each frame for the focused value control (mode on only) keeps Up/Down for the value.
- Tempo range is 20..500 BPM (`Transport::tempo_step`, model.rs:78).

Implementation: crates/orb-ui/src/a11y/drive.rs (Step, Target, Request, SPIN_DISPLAYS, SELECTOR_SIZE, joined, selector_id, requests, request, step, set_slider, select_pattern, pattern_text, 5 unit tests); a11y/mod.rs (slider actions Increment/Decrement/SetValue, describe_spin, describe_selector); app.rs (spin_button, pattern_selector, drive_value, mouse_down, focused_target, the keyboard hook, joined parts without FOCUSABLE); behavior::fader_from_engine (also used by Panel::adopt, same expression as before).

Interpretation: the selector is a SpinButton named "Pattern" whose value is "B3", so it reads "Pattern, spin button, B3" (AC3's "Pattern B3"). Not wrapping at A1/D8. Spin buttons and selectors carry value text only (no numeric value, so no UIA RangeValue): NVDA reads the text; the selector takes no SetValue.

Not settled by the task: Page Up/Page Down/Home/End on a spin button. Created TASK-205 (Decide, @dengopaiv, m-7, recommendation C); for now they do nothing (drive::step). Also a screen reader's Click on buttons/toggles/switches is now one mouse down (the title's "switches"; before it did nothing because the panel acts on `is_pointer_button_down_on`, not on `clicked()`).

Failed attempts: wired.rs AC2 test first selected a pattern every 3 engine calls: nothing past tick 0 was recorded (a new pattern starts at the next bar line, so only bar-line changes record); with 80 calls (> 1 bar at 140 BPM) between selections the recording holds 1..8 (and a 0 at the loop wrap), equal for clicks and selector. TrackEvent is orb_engine::seq::Event, not song::TrackEvent.

Numbers (headless baseline, tests/a11y_baseline.rs): nodes 267 -> 203, named 260 -> 196, valued 108 -> 112, focusable 248 -> 194, Unknown 0; roles: Slider 86, SpinButton 14, Button 52, CheckBox 23, RadioButton 9, Switch 10, Label 2.
frame_cost (release, AccessKit on): tree of 203 nodes costs 0.105 ms of show at native 1x, 0.623 ms at 1800x1100 2x (one run; noisy); shape/mesh/vertex bounds hold.

Rebased onto main fe950d3 (TASK-194) with no conflict.

tools/uia_tree.ps1 on the release build (pwsh tools/uia_tree.ps1 -Exe target/release/OpenRB-338.exe, --demo): 198 elements; named by type: Slider 86, Button 62, CheckBox 23, Spinner 14, RadioButton 9, Text 2 (the 303s' "Edit step 1"), Menu 1 and Pane 1 unnamed; can take the focus 194, all named. The names list reads "... Tempo | Mode | Play | ... | Bar | Loop start | Loop length | Loop | Pattern section on | Pattern | Shuffle | Pattern length | ... | PCF pattern | ... | Delay steps ...": no "Tempo up", no "Pattern 1".."Pattern 8", no "Bank A".."Bank D" any more. (Before, at task-186: the arrows and the 48 grid buttons were Buttons and RadioButtons of their own.)

Tests (release, after the rebase): cargo test --release -p orb-ui --no-fail-fast: 299 passed, 1 failed (crash.rs a_deliberate_panic_exits_101_and_writes_the_log, the known TASK-200 failure), 2 ignored. New: tests/a11y_drive.rs 8 tests, wired.rs a_selection_with_the_pattern_selector_is_recorded_as_the_clicks, drive.rs 5 unit tests, behavior fader round trip. keyboard.rs (12) and interaction.rs (15) unchanged and passing. cargo clippy --workspace --all-targets -D warnings clean; cargo +nightly-2026-10-18 fmt --all --check clean.

Help > Keyboard Shortcuts (key_help.rs) is unchanged: it lists no screen reader mode keys yet; TASK-188 (A11Y-4) AC4 adds the mode's key list, which should include the value keys of docs/ui_spec.md 20.1.

Acceptance criteria, evidence:
- AC1: tests/a11y_drive.rs, two apps from the same state, one by mouse (interaction.rs's way: the panel at 1:1, pointer events), one by request; Panel::events equal per change: sliders_increment_decrement_and_set_as_the_mouse (3 knobs, 2 faders: Increment, Decrement vs a drag to v+-1; SetValue vs a 20 px drag), spin_buttons_click_their_arrows (all 10, song mode), the_pattern_selector_clicks_bank_and_pattern (4 rows, A1..B1 and back, bank crossings), buttons_toggles_and_switches_take_the_click_request (toggle, switch, hswitch, vswitch, radio, trigger, wind).
- AC2: wired.rs a_selection_with_the_pattern_selector_is_recorded_as_the_clicks: song-mode recording with a real Link and player; 8 selections by clicks vs by Increment: the messages sent ({1, p} sub 40, p = 1..8) and the merged tracks are equal, and the 303's track records controller 1 = 1..8.
- AC3: values_read_as_words_and_numbers: "64" (Cutoff), "center" (Channel 1 pan), "128 BPM" (Tempo), "16 steps", "off" (Bar in pattern mode), "Pattern" "B3". Knob and fader texts still come from names::control_value_text (unchanged, TASK-192 extends it); spin texts from names::display_text; the selector from drive::pattern_text.
- AC4: one_node_per_spin_button_and_pattern_selector: no node for any of the 68 joined parts, 14 SpinButton nodes; baseline 203 nodes; UIA probe 14 Spinner and no arrow or grid button.

Round 2 (coordinator, 2026-11-06): the user decided TASK-205 as recommended (C). Implemented in this commit: on a spin button in screen reader mode, Page Up/Page Down = 10 arrow clicks; Home/End click to the end of the range on the short ranges only (`drive::short_range`: pattern length 1..16, delay steps 1..32, PCF pattern 0..54; unit test the_short_ranges_are_the_arrows_clamps checks them against Panel::arrow's clamps), nothing on tempo, bar, loop start, loop length. Test a11y_drive.rs page_home_and_end_on_a_spin_button: for all 10 spin buttons in song mode, Page Up/Down give the same events as 10 mouse clicks of the arrow (10 events; 20 for loop start, which sends start and end per click); End/Home on the short ranges reach hi/lo with one event per value crossed, End at the end sends nothing; Home/End on the 4 transport spin buttons send nothing. First run failed on the count for loop start (2 events per click), fixed in the test.
TASK-205 closed (final summary, AC checked, Done) in this commit; TASK-187 keeps its dependency on it.
Tests after: cargo test --release -p orb-ui --no-fail-fast 301 passed, 1 failed (crash.rs, TASK-200, known), 2 ignored; clippy -D warnings clean; nightly fmt check clean. main is still fe950d3 (no rebase needed).

Review round 1 (send back; core implementation judged sound). Fixed:
1. Rebased onto main e48d24e (TASK-192, physical units). One conflict, the import line of tests/a11y_names.rs: kept both (`drive, names, units`); main's version of the pan line taken. a11y/mod.rs merged by git with main's `describe` (speech read before the node builder). a11y_drive.rs now calls `names::control_value_text(&panel, id, v, Speech::Plain)` (its harness is mode off).
2. AC3, one function: `pattern_text` moved from drive.rs to names.rs; `names::value_text` has a "grid" arm (a pattern or bank button's value = its row's selected pattern, "B3") besides the "display" arm (a spin button's value). `describe_spin` and `describe_selector` (now given the row's `pattern1` Ctl) take their value from `names::value_text(panel, c, speech)` with the mode's speech read from the context. Unit test names::pattern_names; the_panel_values now expects "grid" valued.
3. New test a11y_drive.rs with_the_mode_off_a_focused_value_control_takes_no_keys: with the mode off, a focused knob, tempo spin button, pattern-length spin button and pattern selector each take Up, Down, Page Up, Page Down, Home, End exactly as an app with nothing focused (equal engine events, row focus, cutoff, tempo, pattern length, every row's pattern and bank); and Up on a focused selector is the original's row focus up (wrapping).
5. mouse_down: the unreachable "grid" arm removed, with a comment why (joined parts have no node and no Click). knob/fader: comment that a request in a frame where the mouse drags the same control is dropped (the drag sets the position from its own start each frame).
The tree is unchanged by this round (baseline 203 nodes still passes), so tools/uia_tree.ps1 was not re-run.
Tests: cargo test --release -p orb-ui --no-fail-fast 320 passed, 1 failed (crash.rs, TASK-200, known), 3 ignored; clippy --workspace --all-targets -D warnings clean; nightly fmt check clean.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
A screen reader can now drive the panel's value controls, through the same Panel methods as the mouse (branch worktree-agent-a4cac0cc4eee34ee5, one commit on main e48d24e; review round 1 addressed).

- Sliders (86 knobs and faders): Increment, Decrement, SetValue on 0..127, stored as set_knob(id, knob_from_engine(v)) / set_fader(id, fader_from_engine(v)) (new behavior::fader_from_engine).
- 10 spin buttons (tempo, bar, loop start, loop length, 4 pattern lengths, delay steps, PCF pattern): one focusable SpinButton node each ("Tempo", "140 BPM"); Increment/Decrement = one Panel::arrow click. The 20 arrows have no node and no focus.
- 4 pattern selectors: SpinButton "Pattern" valued "B3"; Increment/Decrement move through A1..D8 with the bank and pattern presses of a click. The 48 pattern and bank buttons have no node and no focus.
- Every value text comes from names::value_text with the mode's speech (knobs and faders through control_value_text, spin buttons through the display arm, the selector through the grid arm and names::pattern_text).
- A screen reader's Click on the remaining buttons, toggles and switches = one mouse down (only the AccessKit request; Space/Enter unchanged).
- Screen reader mode only: Up/Down, Page Up/Down (8 / one bank), Home/End on the focused slider or selector; on a spin button Up/Down one click, Page Up/Down 10 clicks, Home/End to the end of the short ranges (pattern length, delay steps, PCF pattern) and nothing on tempo and the transport positions (TASK-205, decided by the user 2026-11-06, closed in this commit). egui keeps the focus on Up/Down. Mode off: keys unchanged (tested for a knob, two spin buttons and a selector).

Files:
- crates/orb-ui/src/a11y/drive.rs (new): the operations, Step, Target, Request, SPIN_DISPLAYS, short_range, joined, selector_id; unit tests.
- crates/orb-ui/src/a11y/mod.rs: slider actions, describe_spin, describe_selector.
- crates/orb-ui/src/a11y/names.rs: pattern_text, value_text arm "grid"; tests.
- crates/orb-ui/src/app.rs: spin_button, pattern_selector, drive_value, mouse_down, focused_target, the keyboard hook, joined parts without FOCUSABLE.
- crates/orb-ui/src/behavior.rs: fader_from_engine; crates/orb-ui/src/model.rs: adopt uses it.
- crates/orb-ui/tests/a11y_drive.rs (new, 10 tests), crates/orb-ui/tests/wired.rs (song-mode recording test), crates/orb-ui/tests/a11y_baseline.rs (new numbers), crates/orb-ui/tests/a11y_names.rs (joined parts, spin button).
- docs/ui_spec.md 20.1, docs/accessibility.md 3.3.
- backlog/tasks/task-187 (this task), backlog/tasks/task-205 (Decide, Done).

Numbers: headless tree 267 -> 203 nodes, focusable 248 -> 194, named 196, valued 112, 14 SpinButton; UIA probe 198 elements, 194 focusable all named, 14 Spinner. Tree build 0.105 ms at 1x.
Tests: cargo test --release -p orb-ui 320 pass, 1 known failure (crash.rs, TASK-200), 3 ignored; clippy and nightly fmt clean.

Left open: the mode's key list in Help > Keyboard Shortcuts is TASK-188's (AC4).
<!-- SECTION:FINAL_SUMMARY:END -->
