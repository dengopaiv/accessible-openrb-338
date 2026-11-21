---
id: TASK-257
title: >-
  Document ReBirth's ReWire audio outputs: the 18 channels, their tap points and
  how they are filled
status: To Do
assignee:
  - '@agent-complex'
created_date: '2026-11-21T06:27:00Z'
labels:
  - re
  - docs
  - player
milestone: m-7
dependencies:
  - TASK-256
references:
  - docs/initial_investigation.md
  - docs/plugin.md
  - crates/orb-engine/src/frame/mod.rs
priority: medium
type: spike
ordinal: 251000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Why: task-256 gives the plugin four stereo outputs per device (303-1, 303-2, 808, 909) tapped after each mixer channel's inserts, level and pan, before the master sum. The user asked (2026-11-21) how ReBirth's own ReWire did multi-channel output, to decide afterwards what the plugin should replicate (per-instrument drum outputs, a delay return, mono vs stereo, tap points). This task only documents the original; it changes no code. The choice of what to replicate becomes a `Decide:` task at the end.

What is already known (supervisor's look, 2026-11-21, ReBirth.dll):
- `TopHatGetDeviceInfoProc` (0x1001B350; TopHat is ReWire's working name) fills a 0x8D8-byte device-info struct: name "ReBirth Audio", channel count 18 at +0x20, 32-byte channel names from +36: Mix-L, Mix-R, 303-1, 303-2, 808-Mix, 808-BD, 808-SD, 808-Tom, 808-Per, 808-HH, 808-CY, 909-Mix, 909-BD, 909-SD, 909-Tom, 909-Per, 909-HH, 909-CY (strings at 0x101534E0..0x101535F0). It sets byte flags at +2084..+2101 (one per channel) and +2148, +2149, +2212 (meaning unknown: stereo pairing? default routing?), and dword +561*4 = 8.
- No delay channel is declared.
- `TopHatDriveAudioProc` (0x1001B800) calls `sub_1001C0F4` (host sync, docs/plugin.md 8.1), `sub_1001CC40(*(dword_1011D0B0+880), ...)` (the render), `sub_1001C204`.
- docs/initial_investigation.md section 7.1 has a [G]: "For export the per-instrument ReWire outputs are null, so everything sums into bus 0", i.e. the drum voices have per-instrument bus pointers.
- `ReWire.dll` is a separate file on the CD (docs/test_machine.md); Rebirth.exe has no ReWire strings.

Questions to answer, each with addresses checked in the disassembly:
1. The device-info struct layout and the meaning of every field set (flags at +2084.., +2148, +2149, +2212, +561*4).
2. The drive-audio struct the host passes (the fields read in `TopHatDriveAudioProc` and its callees: buffers, channel-enable flags, frame count, tempo/position), and which channels are written when the host enables them.
3. For each of the 18 channels, where its signal is tapped: which voices feed it (which 808/909 instruments are in Tom, Per, HH, CY), whether it is before or after the voice/instrument level, the device's mixer channel (inserts DIST/PCF/COMP, level, pan, on/off), the delay send, master compressor and master level; mono or a mono sum of a stereo signal; and whether a sound routed to a per-instrument channel is removed from 808-Mix/909-Mix and from Mix-L/R.
4. What 303-1/303-2 carry (pre or post mixer channel), and whether the delay return reaches anything but Mix-L/R.
5. Whether this audio path differs from the export path (bus 0 summing) in any way that matters to orb's bit-exact renders.

Write it as a new section in docs/plugin.md (or a new docs/rewire.md linked from docs/plugin.md and docs/initial_investigation.md), with [C]/[G] marks and addresses, and correct the [G] in initial_investigation.md 7.1. Compare with what task-256 implemented and list the differences. Then create the `Decide:` task (`--type spike -l decision,player -a @pingw33n`, m-7) with the options for the plugin (e.g. per-instrument drum outputs as stereo or mono pairs, a delay-return output, keeping task-256 as is) and a recommendation. Use a copy of the IDA databases; never save or modify the user's. No test or tool may need the original's files.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 The device-info and drive-audio structs used by the TopHat procs are documented field by field with addresses, [C] or [G]
- [ ] #2 For each of the 18 ReWire channels the doc states its source voices, its tap point relative to the voice level, mixer channel inserts, level, pan, delay and master section, mono/stereo, and whether it is removed from the Mix, each with addresses
- [ ] #3 The [G] about per-instrument ReWire outputs in docs/initial_investigation.md 7.1 is confirmed or corrected
- [ ] #4 The doc lists the differences between ReWire's outputs and task-256's plugin outputs
- [ ] #5 A Decide: task assigned to @pingw33n holds the options for what the plugin replicates, with a recommendation
<!-- AC:END -->
