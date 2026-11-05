---
id: TASK-179
title: 'Decide: where a machine''s mixer channel is in the navigation'
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
  - assets/ui/layout.tsv
  - docs/accessibility.md
  - docs/accessibility_plan.md
priority: high
type: spike
ordinal: 202000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Planned in docs/accessibility_plan.md as A11Y-D3; the analysis is docs/accessibility.md. Read both before starting.

- Options: A. In the machine's own section (Bass Line 1's section ends with channel 1's level,
  pan, delay send, PCF, compressor, distortion and mute). B. A Mixer section with the four
  channels one after the other.
- Recommendation: A: in ReBirth a row is one machine with its channel, and the row focus that
  directs the keys follows the section.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 The user has chosen; the final summary records the choice and the date
<!-- AC:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
**Decided (2026-11-05): A.** Each machine's section ends with its mixer channel.
<!-- SECTION:FINAL_SUMMARY:END -->
