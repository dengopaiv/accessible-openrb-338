---
id: TASK-180
title: 'Decide: system file dialogs in screen reader mode'
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
  - crates/orb-ui/src/browser.rs
  - crates/orb-ui/Cargo.toml
  - docs/accessibility_plan.md
  - docs/accessibility.md
priority: high
type: spike
ordinal: 203000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Planned in docs/accessibility_plan.md as A11Y-D4; the analysis is docs/accessibility.md. Read both before starting.

- Options: A. In screen reader mode, Open, Save As, Export and Mods > Choose file use the
  system's dialog (`rfd::FileDialog`); the browser's "Use as songs folder" and "Use as mods
  folder" become Options menu items with a folder dialog. B. Make the in-app browser
  accessible (named list items, roles, focus) and keep it in both modes.
- Recommendation: A: the Windows dialog is already fully accessible and familiar; B is more
  work for a worse result. The in-app browser stays as it is with the mode off.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 The user has chosen; the final summary records the choice and the date
<!-- AC:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
**Decided (2026-11-05): A.** The system's dialogs in screen reader mode, the folder choices
in the Options menu; the in-app browser unchanged with the mode off.
<!-- SECTION:FINAL_SUMMARY:END -->
