---
id: TASK-208
title: 'Decide: how the plugin shows the Standard Mapping (Remote MIDI) setting'
status: To Do
assignee:
  - '@pingw33n'
created_date: '2026-11-16T09:43:00Z'
labels:
  - decision
  - ui
milestone: m-7
dependencies: []
references:
  - backlog task-207
  - backlog task-201
  - docs/midi.md
  - docs/standard_midi_mapping.md
  - docs/ui_spec.md
priority: medium
type: spike
ordinal: 213500
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
In the plugin the Standard Mapping (the original's Remote MIDI Control) is off by default; when on, it listens on one channel, 16 by default, any channel but the device channels 1..4 (task-207, 2026-11-16). The plugin editor (task-201) has no native menu bar: its menus are drawn in the panel. How does the user turn it on and pick the channel?

The original: Options > Preferences has a MIDI section with "Remote MIDI Control" (on/off), the channel, and Standard vs Quick mapping (docs/midi.md; Quick Mapping is task-188, standalone only).

Options:
A. **A submenu in the panel's Options menu:** "Remote MIDI (Standard Mapping)" > Off / Channel 5 ... Channel 16 (radio items; 1..4 absent). One click, no dialog, the state shows as the checked item.
B. **A small dialog** (Options > MIDI...) like the original's preferences: a "Remote MIDI Control" checkbox and a channel drop-down (5..16). Closer to the original, one more window to build inside a plugin window.
C. **No UI:** plugin state only, set by... nothing practical; rejected unless the feature is dropped.
D. **A host parameter** (off, 5..16): automatable and visible in the host's generic UI, but it is a setting, not a sound control, and task-185 makes only visible panel controls parameters.

Recommendation: A (the cheapest, no dialog in a plugin window, the current value visible in the menu).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 The outcome and its date are in the final summary, and task-201 says what to build
<!-- AC:END -->
