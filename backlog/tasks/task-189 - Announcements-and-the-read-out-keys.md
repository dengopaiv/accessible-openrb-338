---
id: TASK-189
title: Announcements and the read-out keys
status: To Do
assignee:
  - '@agent-complex'
created_date: '2026-11-05T19:32:00Z'
labels:
  - ui
milestone: m-7
dependencies:
  - TASK-186
  - TASK-182
references:
  - crates/orb-ui/src/app.rs
  - crates/orb-ui/src/menu.rs
  - crates/orb-ui/src/model.rs
  - docs/accessibility_plan.md
  - docs/accessibility.md
priority: high
type: feature
ordinal: 212000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Planned in docs/accessibility_plan.md as A11Y-5; the analysis is docs/accessibility.md. Read both before starting.

- What: `crates/orb-ui/src/a11y/announce.rs`: one node, polite live region, outside the focus
  order, whose text is the last message (repeated messages must be spoken again: add a
  changing invisible part or clear and set in two frames; test with NVDA which works). It
  speaks: the status line's messages; a song opened (its name and mod); a mod chosen; the
  keyboard mode changed (Ctrl+F, Ctrl+G); what an original key did when the focus is not on the
  control it changes (pattern selected with the pattern keys, tempo up/down, Record, Loop, mode,
  Play/Stop with F5 or Enter, Record with F9). The status line itself also gets a node with its
  text. Read-out
  keys (D6): F2 reads the focused section's pattern (A11Y-6 and A11Y-7 fill in the synths and
  drums; here the pattern row: "Pattern B3, 16 steps, shuffle off"), F3 the transport ("Playing,
  bar 5, 128 BPM, song mode, loop 1 to 8, recording off"), F4 the levels (master left and right,
  compressor gain reduction; in dB where A11Y-14 finds the engine's scale, else as numbers).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Tests: each listed event sets the announcer's text; nothing sets it during 10 seconds of playback without input (position, meters and LEDs moving).
- [ ] #2 Tests: F2, F3 and F4 set the expected texts on a known song.
- [ ] #3 With NVDA, the announcements are spoken (by hand; notes record the NVDA version and what was heard), including a message repeated twice.
<!-- AC:END -->
