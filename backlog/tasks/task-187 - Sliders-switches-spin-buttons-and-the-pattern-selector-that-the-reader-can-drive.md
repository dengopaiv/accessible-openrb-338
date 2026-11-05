---
id: TASK-187
title: >-
  Sliders, switches, spin buttons and the pattern selector that the reader can
  drive
status: To Do
assignee:
  - '@agent-complex'
created_date: '2026-11-05T19:32:00Z'
labels:
  - ui
milestone: m-7
dependencies:
  - TASK-186
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
- [ ] #1 For each kind, a test sends the requests and compares the engine events (`Panel::events`) with those of the same change made by mouse in `crates/orb-ui/tests/interaction.rs`'s way: equal.
- [ ] #2 A selection made with the pattern selector during song-mode recording is recorded like a click (a test with a link, as `crates/orb-ui/tests/wired.rs`).
- [ ] #3 Values read back as words or numbers ("64" until A11Y-14, "center", "128 BPM", "Pattern B3"), through one function that A11Y-14 extends with the units.
- [ ] #4 The tree has one node per spin button and per pattern selector, not the parts.
<!-- AC:END -->
