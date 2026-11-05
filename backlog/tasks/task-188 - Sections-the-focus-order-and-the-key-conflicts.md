---
id: TASK-188
title: Sections, the focus order and the key conflicts
status: To Do
assignee:
  - '@agent-complex'
created_date: '2026-11-05T19:32:00Z'
labels:
  - ui
milestone: m-7
dependencies:
  - TASK-185
  - TASK-187
  - TASK-178
  - TASK-179
  - TASK-182
references:
  - crates/orb-ui/src/app.rs
  - crates/orb-ui/src/keys.rs
  - docs/ui_spec.md
  - docs/accessibility.md
  - docs/accessibility_plan.md
priority: high
type: feature
ordinal: 211000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Planned in docs/accessibility_plan.md as A11Y-4; the analysis is docs/accessibility.md. Read both before starting.

- What, in screen reader mode only:
  - The sections of docs/accessibility.md 3.2 as Group nodes with names, in that order; F6 and
    Shift+F6 move the focus to the first control of the next or previous section; Tab and
    Shift+Tab move within the panel in section order (wrapping at the ends of the panel).
  - The row focus follows the section the focus is in (the two 303s, the 808, the 909), and the
    change is announced through A11Y-5's announcer when that exists, else by the section name.
  - The key map of D6: Tab no longer taps the 303 (Ctrl+Tab and Ctrl+Shift+Tab do), Space
    presses or toggles the focused control and is no longer Play/Stop (D2), F5 is Play/Stop,
    F9 Record, F11 Loop, F12 song/pattern mode (each the same `Panel::press` as the panel's
    button), Enter keeps the original's meaning, Up/Down
    change the focused value instead of the row focus, Ctrl+Up/Down move the row focus.
  - egui's own arrow-key focus movement is suppressed on the panel (the arrows change values).
  - The original's play and program keys (letters, digits, minus, equals, Enter in Program
    Synth mode) keep working, whatever control has the focus.
  - Auto-repeat of Up/Down/Page Up/Page Down on a focused value control is honored (the
    original ignores repeats for its own keys; that stays).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Tests: F6 visits the sections in order and wraps; Tab visits the controls of the panel in section order; the focused node after each step has the expected name.
- [ ] #2 Tests: while a 303 plays, Tab moves the focus and does not change the pattern; Ctrl+Tab taps the playing step as Tab did (Ctrl+Shift+Tab a rest); F5, F9, F11 and F12 give the same engine events as the panel's Play/Stop, Record, Loop and mode buttons; Space on a focused toggle toggles it and does not play; Enter with a 303 focused in Program Synth mode is the 303's Step.
- [ ] #3 With the mode off, `crates/orb-ui/tests/keyboard.rs` passes unchanged, and a test shows Tab and Up/Down behaving as on main.
- [ ] #4 docs/ui_spec.md section 10 has a "Screen reader mode" part with the key map, and Help > Keyboard Shortcuts lists the mode's keys when it is on.
<!-- AC:END -->
