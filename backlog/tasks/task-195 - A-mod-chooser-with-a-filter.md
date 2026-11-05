---
id: TASK-195
title: A mod chooser with a filter
status: To Do
assignee:
  - '@agent-worker'
created_date: '2026-11-05T19:32:00Z'
labels:
  - ui
  - rbm
milestone: m-7
dependencies:
  - TASK-193
references:
  - crates/orb-ui/src/menu.rs
  - crates/orb-ui/src/mod_info.rs
  - docs/rbm.md
  - docs/accessibility_plan.md
  - docs/accessibility.md
priority: medium
type: feature
ordinal: 218000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Planned in docs/accessibility_plan.md as A11Y-10; the analysis is docs/accessibility.md. Read both before starting.

- Why: with the user's mods folder the Mods menu holds 311 entries (the 320 files of the rb-mods
  collection; equal names merge with variants), in a scroll area inside a pop-up menu. Walking
  it with the arrows is slow, and egui menus have no type-ahead.
- What: in screen reader mode, the Mods menu's first item is "Choose Mod..." (with a shortcut,
  to decide in the task and add to the key help), opening a dialog with a filter field and a
  list. The list holds the entries of the menu in the menu's order, each read as its name, then
  "variants: N" when it has variants, "has demo" when it has a demo song, and "current" for the
  playing kit. Typing filters by name (ASCII case ignored, as the menu sorts). Enter chooses
  (`use_kit`, or for a merged name the preferred variant, with the variants reachable as
  sub-items), Escape closes, a key opens Mod Info for the selected entry. The built-in
  "Standard ReBirth" is the first entry.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Tests with a folder of test mods (the existing test mods, or `ORB_THIRDPARTY_MODS_DIR` gated with a skip message): the list equals the menu's entries in order; the filter narrows it; Enter loads the same kit as the menu item.
- [ ] #2 With the rb-mods collection, the dialog opens with all 311 entries in under 100 ms after the list is cached (measure; notes).
- [ ] #3 With NVDA, the list and the filter are announced as such (by hand; notes).
<!-- AC:END -->
