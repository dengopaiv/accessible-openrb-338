---
id: TASK-206
title: 'Decide: plugin pattern alignment under host time signatures other than 4/4'
status: Done
assignee:
  - '@pingw33n'
created_date: '2026-11-16T09:11:00Z'
updated_date: '2026-11-16T09:38:00Z'
labels:
  - decision
  - player
milestone: m-7
dependencies: []
references:
  - docs/plugin.md
  - backlog task-198
priority: low
type: spike
ordinal: 220000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Context. task-198 makes the plugin's pattern mode follow the host position: orb's tick is the host's position in quarter notes times 192 (docs/plugin.md section 8.2), so bar 1 beat 1 is step 1 and, in 4/4, every host bar starts a 16-step pattern at step 1. task-185 says "bar 1 beat 1 = step 1" but not what happens in other time signatures. In 3/4 (or 7/8, or after a signature change) a host bar is not 768 ticks, so with the current rule a 16-step pattern runs across host bar lines: in 3/4 it restarts every 4/3 bars, and the patterns' bar-line latching (pattern changes, on/off) falls at orb's 768-tick bars, not the host's. nice-plug gives bar_start_pos_beats, bar_number and the time signature when the host reports them.

Options.
A. Keep the quarter-note rule (now): continuous, deterministic, independent of the host's signature; ReBirth itself only knows 4/4 (768 ticks per bar).
B. Restart at every host bar: tick = 768 * bar_number + (pos - bar_start) * 192, so each host bar starts at step 1 (a 3/4 bar plays steps 1..12 of a 16-step pattern, then jumps to step 1 at the next bar: a locate per bar).
C. A per-instance option between A and B.

Recommendation: A (the original is 4/4-only; B truncates patterns silently), with B as a later option if users ask.
<!-- SECTION:DESCRIPTION:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Decided by the user, 2026-11-16: option A. The plugin keeps the quarter-note rule: orb's tick = the host position in quarter notes x 192, whatever the host's time signature (ReBirth is 4/4-only, 768 ticks per bar). Continuous and deterministic; in other signatures a 16-step pattern runs across host bar lines. This is what task-198 implemented (docs/plugin.md section 8.2); no follow-up task. Restarting at every host bar (option B) or a per-instance option (C) only if users ask.
<!-- SECTION:FINAL_SUMMARY:END -->
