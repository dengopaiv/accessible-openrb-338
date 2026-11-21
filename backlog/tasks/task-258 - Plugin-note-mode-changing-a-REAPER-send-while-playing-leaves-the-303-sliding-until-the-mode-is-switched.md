---
id: TASK-258
title: >-
  Plugin note mode: changing a REAPER send while playing leaves the 303 sliding
  until the mode is switched
status: To Do
assignee:
  - '@agent-complex'
created_date: '2026-11-21T06:27:00Z'
updated_date: '2026-11-21T06:32:00Z'
labels:
  - player
milestone: m-7
dependencies:
  - TASK-244
references:
  - /Users/pingw33n/documents/reaper_routing_on_off_bug.mp4
  - crates/orb-plugin/src/core.rs
  - docs/plugin.md
  - docs/plugin_usage.md
priority: high
type: bug
ordinal: 252000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Reported by the user on 2026-11-21 while testing task-256's branch in REAPER (it may predate task-256; find out). Setup: track 1 holds the OpenRB-338 instrument; four tracks hold MIDI clips made by dragging the plugin's patterns out (one per device) and send MIDI to track 1. While REAPER plays, adding or removing a send on the 303 track makes the 303 sound wrong: as if a note is stuck and every following note slides. It stays that way until the plugin's mode is switched (Note to Latch or Gate and back). Video: ~/documents/reaper_routing_on_off_bug.mp4 (audio matters; extract with ffmpeg as needed).

Likely cause to check first: removing a MIDI send mid-note drops the note-off, so the 303 voice keeps a held note and every later note overlaps it, which the 303 plays as a slide (legato). Adding a send mid-note gives a note-off without its note-on, or a note-on into an already-held key. Check what REAPER sends when routing changes (all-notes-off CC123, CC120, nothing), what nice-plug delivers to the plugin, and how note mode tracks held notes per device (a held-note set or counter that can go out of step). Switching modes evidently resets that state; find what it resets.

The fix must make the plugin recover on its own: e.g. honour CC123/CC120 and the host's transport stop/reset, and not let a stray or missing note-off leave the 303 sliding forever (decide with care what a real 303 in note mode should do with overlapping notes: slide only while the previous key is really held). If the right behavior is a choice for the user, create a `Decide:` task. Reproduce in a test with the plugin core (a note-on with no note-off, then later notes; a note-off with no note-on), not only in REAPER. The user checks the fix in REAPER before the agent review.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 The cause is found and written in the task notes, with whether it happens on main before task-256
- [ ] #2 A test reproduces the stuck slide from a missing or stray note-off and passes after the fix
- [ ] #3 The plugin recovers without a mode switch: on all-notes-off or all-sound-off, on transport stop, and from the stuck state the test reproduces
- [ ] #4 docs/plugin.md describes how note mode treats held, missing and stray note-offs
- [ ] #5 The user confirms in REAPER that adding or removing a send while playing no longer leaves the 303 sliding
- [ ] #6 A case in task-244's REAPER harness (a note-on whose note-off never arrives, then more 303 notes in note mode) passes after the fix
<!-- AC:END -->
