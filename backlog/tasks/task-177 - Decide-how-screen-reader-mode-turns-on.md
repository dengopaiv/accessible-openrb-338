---
id: TASK-177
title: 'Decide: how screen reader mode turns on'
status: Done
assignee:
  - '@dengopaiv'
created_date: '2026-11-05T19:31:00Z'
updated_date: '2026-11-05T19:31:00Z'
labels:
  - decision
  - ui
milestone: m-7
dependencies: []
references:
  - crates/orb-ui/src/settings.rs
  - crates/orb-ui/src/menu.rs
  - docs/accessibility.md
  - docs/accessibility_plan.md
priority: high
type: spike
ordinal: 200000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Planned in docs/accessibility_plan.md as A11Y-D1; the analysis is docs/accessibility.md. Read both before starting.

- Why: the accessible layer changes a few keys (A11Y-4) and the file dialogs (A11Y-9), so it is
  a mode that must be on for screen reader users and off for everyone else, without the user
  having to find a setting they cannot hear.
- Options:
  - A. On by itself when the system reports a screen reader (Windows: the screen reader flag,
    `SystemParametersInfo(SPI_GETSCREENREADER)`, which screen readers are meant to set; that
    NVDA sets it is to be checked in A11Y-1, and if it does not, the detection falls back to
    looking for NVDA's process or its controller DLL's answer; macOS: VoiceOver's state; Linux:
    the AT-SPI "screen reader enabled" property), plus an Options menu item "Screen Reader Mode"
    with three states (automatic, on, off) kept in `ui.toml`.
  - B. Only by hand, from the Options menu, kept in `ui.toml`.
  - C. Always on: no mode at all.
- Recommendation: A. C would change the keys for every user and could not go upstream.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 The user has chosen; the final summary records the choice and the date
<!-- AC:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
**Decided (2026-11-05): A.** Automatic when the system reports a screen reader, with the
Options menu item "Screen Reader Mode" (automatic, on, off), kept in `ui.toml`.
<!-- SECTION:FINAL_SUMMARY:END -->
