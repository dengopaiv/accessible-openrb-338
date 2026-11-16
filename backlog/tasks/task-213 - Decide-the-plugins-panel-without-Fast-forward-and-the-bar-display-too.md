---
id: TASK-213
title: 'Decide: the plugin''s panel without Fast forward and the bar display too'
status: Done
assignee:
  - '@pingw33n'
created_date: '2026-11-16T11:28:00Z'
updated_date: '2026-11-16T13:31:00Z'
labels:
  - decision
  - ui
milestone: m-7
dependencies: []
references:
  - backlog task-201
  - backlog task-185
  - crates/orb-ui/src/embed.rs
priority: low
type: spike
ordinal: 222000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
task-185 lists what the plugin's panel hides: the Pattern/Song switch, the loop switch and range, Rewind, Record (and the tempo's arrows). It does not name Fast forward and the bar display with its arrows. In the original (and orb's standalone app) these act only in song mode: their presses are tested inside the song-mode block of sub_42A058 (docs/ui_spec.md section 3), and in pattern mode the bar display is dark. The plugin has no song mode (task-185), so in the plugin they would be a key that does nothing and a dark display.

task-201 (the editor) hides them too, so that the transport keeps Play and Stop in one well and has room for the menus (embed.rs PLUGIN_HIDDEN, the art part bg_transport_plugin). The user reviews the plugin's look in task-201 before merge.

Options:
A. Hide Fast forward and the bar display with its arrows (as task-201 does now). Recommended: they do nothing without song mode, and the well of Play and Stop reads as the mode switch.
B. Show them, inert, as in the standalone's pattern mode (needs the plugin art to keep the five-key well and the bar display, and the menus to move elsewhere).

Recommendation: A.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 The user chose A or B; task-201's PLUGIN_HIDDEN and bg_transport_plugin follow the choice
<!-- AC:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Decided by the user on 2026-11-16: A, Fast forward and the loop bar display stay hidden in the plugin, like Rewind and Record (the host owns the transport). task-201 already implements A (PLUGIN_HIDDEN, bg_transport_plugin.svg), so nothing changes.
<!-- SECTION:FINAL_SUMMARY:END -->
