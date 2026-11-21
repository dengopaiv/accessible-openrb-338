---
id: TASK-259
title: >-
  Decide: which of ReBirth's ReWire outputs the plugin replicates
  (per-instrument drums, delay return, routing)
status: Done
assignee:
  - '@pingw33n'
created_date: '2026-11-21T06:32:00Z'
updated_date: '2026-11-21T06:46:00Z'
labels:
  - decision
  - player
milestone: m-7
dependencies:
  - TASK-256
  - TASK-257
references:
  - docs/rewire.md
  - docs/plugin.md
priority: medium
type: spike
ordinal: 253000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Why: task-257 documented how ReBirth's own ReWire (TopHat) outputs work (docs/rewire.md, the comparison with the plugin in its section 6). The user asked (2026-11-21) what the plugin should replicate of them beside task-256's outputs. This task holds that choice; it changes nothing by itself.

What ReWire does (docs/rewire.md, [C] from ReBirth.dll):
- 18 channels, the Mix-L/R stereo pair and 16 mono channels: Mix-L/R (the master after the delay, the master compressor and the master gain, f32), 303-1, 303-2, 808-Mix, 909-Mix, and six per drum machine: BD (BD A+B), SD, Tom (the three toms or congas), Per (808: RS/CL, CP/MA, CB; 909: RS, CP), HH (OH, CH), CY (808: CY; 909: crash, ride).
- A device channel is tapped after its inserts (dist, PCF, comp) and before its level, pan and on switch, mono; enabling it takes the channel out of Mix-L/R, but its delay send stays (scaled by the level fader), so its echo stays in the Mix.
- An instrument channel is the raw voice sum (the instrument's level knob and accent are in the voice gain), before the machine's bus: enabling it removes those voices from the machine's channel (inserts, delay send, level, pan) and from the Mix.
- No delay-return channel.

What the plugin has after task-256 (docs/plugin.md 13): a stereo Mix (the original's int16 output) and four stereo device buses 303-1, 303-2, 808, 909, tapped after the inserts AND after level and pan, as copies: the Mix never changes with what is connected.

Options:
A. Keep task-256 as is. Nothing more to build. No per-instrument drums.
B. Add per-instrument drum outputs as copies: 12 more outputs (808 and 909 x BD, SD, Tom, Per, HH, CY), each the group's raw voice sum as ReWire taps it, the Mix and the device buses unchanged (the instruments stay in them). Each output mono, or stereo with the same signal on both sides for hosts that handle mono aux buses poorly. Cost: 12 more taps per block in the drum models (a store per sample per group), the ring and the resampler carrying 12 more channels; the user mutes the Mix's drums with the 808/909 channel switches when routing the instruments out.
C. Replicate ReWire's routing: connecting an output (device or instrument) removes its signal from the Mix (and an instrument from its machine's bus), taps before level and pan, mono. Matches ReBirth in a ReWire host, but the Mix then depends on the host's connections (a host that connects every bus by default, as some do, would silence the Mix), and the device tap moves from task-256's post-fader point.
D. A delay-return output (the delay's wet signal alone). The original has none; it would need the delay to render its return into a buffer of its own before adding it to the master (the master sum order must stay the same for the Mix to stay bit-exact). Can be combined with A or B.
E. Per-instrument outputs post-insert/post-fader: not possible as ReWire does them (the inserts and the fader act on the machine's sum), so not offered.

Recommendation: B with stereo outputs carrying the mono signal, and A's device buses kept as they are. It gives what ReWire users used (separate drums) while keeping the plugin's property that the Mix is bit-exact whatever the host connects; C's removal from the Mix is a ReWire mixer convention a plugin host does not need (the user mutes in the panel). D only if the user wants it; the original never had it.

Superseded: the user decided A with the ReWire tap point (2026-11-21), see the final summary.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 The user picks the options (A to E) or another, and the plugin's outputs to build are written in the final summary with the date
- [x] #2 Follow-up tasks for the chosen outputs exist, or none are needed
<!-- AC:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Decided by the user on 2026-11-21: option A, with one change. The plugin keeps task-256's four device outputs (303-1, 303-2, 808, 909) and gets no per-instrument drum outputs. The device outputs move to ReWire's tap point: after the channel's inserts (distortion, PCF, compressor) and before its level, pan and on switch, as stereo buses carrying the mono signal on both sides. No delay-return output (D), and nothing is removed from the Mix (not C): the outputs stay copies and the Mix is unchanged. Follow-up: task-256 (not yet merged) is being reworked on its branch to the new tap point and records `Decision: task-259` in its commit; its task file is updated there by its own agent. The user also asked whether PCF and COMP could sit on several channels in ReWire: no, ReWire uses the app's mixer (one PCF on one channel, one compressor on one channel or the master; only the distortion has a per-channel switch, with shared parameters), docs/rewire.md section 4.
<!-- SECTION:FINAL_SUMMARY:END -->
