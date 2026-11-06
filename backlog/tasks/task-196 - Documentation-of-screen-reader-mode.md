---
id: TASK-196
title: Documentation of screen reader mode
status: Done
assignee:
  - '@agent-worker'
created_date: '2026-11-05T19:32:00Z'
updated_date: '2026-11-06T22:54:00Z'
labels:
  - docs
  - ui
milestone: m-7
dependencies:
  - TASK-185
  - TASK-186
  - TASK-187
  - TASK-188
  - TASK-189
  - TASK-190
  - TASK-191
  - TASK-193
  - TASK-194
  - TASK-195
  - TASK-184
  - TASK-192
references:
  - crates/orb-ui/src/key_help.rs
  - docs/ui_spec.md
  - docs/accessibility.md
  - docs/accessibility_plan.md
priority: medium
type: docs
ordinal: 219000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Planned in docs/accessibility_plan.md as A11Y-12; the analysis is docs/accessibility.md. Read both before starting.

- What: a README section "Using OpenRB-338 with a screen reader" (turning the mode on, the
  sections, the keys, programming a pattern, choosing a mod, known limits); docs/ui_spec.md's
  "Screen reader mode" section complete and consistent with the code; Help > Keyboard Shortcuts
  complete for the mode; docs/accessibility.md's status line set to what is implemented, with
  the measurements of section 1.1 taken again (`tools/uia_tree.ps1`) and recorded.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Every key of the mode is in the README, the ui_spec and the key help, with the same meaning (a test compares the key help's table with the key map in code, if the key help has one).
- [x] #2 The new measurement is in docs/accessibility.md next to the old one.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Check ui_spec 20 and section 10 against code; README section; key help additions and a test for keys outside the key map; re-measure with uia_tree.ps1 (on/off/auto) and a11y_baseline; update status lines.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: task-sr-docs

Probe run 3 times (on, off, auto; NVDA was running so auto = on) on 6cc85ab. Mode on: 593 elements, 198 focusable all named; off: 198 elements, 194 focusable. Headless (a11y_baseline --nocapture): off 203 nodes/196 named/112 valued/194 focusable; on 598/591/198. Key map cannot compare keys handled outside keys::map_screen_reader (Shift+Space, Ctrl+M/I, step editor, grid, Escape): added test the_keys_handled_outside_the_key_map_are_named_in_the_group (names check only). ui_spec section 10 table: F2-F4 no longer 'Free', added Enter/Backspace, Ctrl+I, Escape rows. Key help: added Escape row, Backspace in Enter row. No code behavior bugs found. crash.rs failure is the known TASK-200.

Review round 1: key-name test now checks the rows' keys field (descriptions for Shift+Space, Backspace etc.); stated that keys outside keys::map_screen_reader are checked by name only and a new key in app.rs needs its name added to the lists by hand; README gets a Left / Right row, F3 pattern-mode wording, Enter in Program Synth mode; accessibility.md headless focusable 'not measured' for 2026-11-05.

Merge (supervisor, 2026-11-07): review round 1 sent back six small defects (the Escape check matched another row's text; the names-only test not stated as such; no Left / Right row in the README table; F3 song vs pattern mode; Enter as Step only in Program Synth mode; a UIA figure in the headless row), fixed in the amended commit. Round 2: merge. Fast-forwarded onto main.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
README has 'Using OpenRB-338 with a screen reader' (mode, sections, keys table, 303 and drum programming, mod chooser, limits) replacing the stale status text. ui_spec section 10/20 aligned with the code; key help has Escape and Backspace and a new test that the keys handled outside the key map are named in the group. accessibility.md 1.1 and 6.3 carry the 2026-11-07 measurement at 6cc85ab next to the old; status lines set in accessibility.md and accessibility_plan.md. Branch task-sr-docs. Tests: cargo test -p orb-ui all pass except the known crash.rs (TASK-200); fmt and clippy clean.
<!-- SECTION:FINAL_SUMMARY:END -->
