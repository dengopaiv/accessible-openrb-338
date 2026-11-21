---
id: TASK-257
title: >-
  Document ReBirth's ReWire audio outputs: the 18 channels, their tap points and
  how they are filled
status: Done
assignee:
  - '@agent-complex'
created_date: '2026-11-21T06:27:00Z'
updated_date: '2026-11-21T06:46:00Z'
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
- [x] #1 The device-info and drive-audio structs used by the TopHat procs are documented field by field with addresses, [C] or [G]
- [x] #2 For each of the 18 ReWire channels the doc states its source voices, its tap point relative to the voice level, mixer channel inserts, level, pan, delay and master section, mono/stereo, and whether it is removed from the Mix, each with addresses
- [x] #3 The [G] about per-instrument ReWire outputs in docs/initial_investigation.md 7.1 is confirmed or corrected
- [x] #4 The doc lists the differences between ReWire's outputs and task-256's plugin outputs
- [x] #5 A Decide: task assigned to @pingw33n holds the options for what the plugin replicates, with a recommendation
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Read the TopHat procs in a scratch copy of ReBirth.dll.i64 (idalib, headless, closed with save=false): TopHatGetDeviceInfoProc 0x1001B350, TopHatDriveAudioProc 0x1001B800 and its callees sub_1001C0F4, sub_1001C204, sub_10013240 (the render), the mixer block sub_10013444, the 808/909 audio slots 0x100280B0/0x1002B6F0 and the 303 audio slot 0x10023F10.
2. Write docs/rewire.md: the device-info struct, the drive-audio struct, the per-channel table (sources, tap point, inserts, level/pan, delay, master, mono/stereo, removal from the Mix), the export-path comparison and the differences from task-256's outputs.
3. Link it from docs/plugin.md 8.1 and correct the [G] of docs/initial_investigation.md 7.1 (lines task-256 does not touch, so the branches rebase cleanly).
4. Create the Decide: task (spike, decision,player, @pingw33n, m-7) with the options and a recommendation.
5. Check the ACs, final summary, Done, one docs: commit.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: worktree-agent-aaf74e609b8056c8c

IDA evidence (scratch copy of ReBirth.dll.i64, session closed save=false):
- TopHatGetDeviceInfoProc 0x1001B350: rep stosd 0x236 dwords (0x8D8 bytes) zero; +0x8C4 = 8 (0x1001B394); name 'ReBirth Audio' sprintf at +0; +0x20 = 0x12 (0x1001B3B4); 18 names at +0x24 + 32i (0x1001B3AC..0x1001B54B); bytes +0x824+i = 1 for i<18 (loop 0x1001B562, cmp 12h); +0x864, +0x865, +0x8A4 = 1 (0x1001B57A..0x1001B586). TopHatGetDeviceNameAndVersionProc 0x1001B320 also returns 8 (*a1 = 8), so +0x8C4 is the version [G]. 64-channel arrays (names 0x24..0x824, flags 0x824..0x864, 0x864..0x8A4, 32 pair bytes 0x8A4..0x8C4) inferred from the offsets [G].
- TopHatDriveAudioProc 0x1001B800: +0x95C -> byte_10121ED8 = (+0x95C == 0); clears +0x1188..+0x11C7 (64 bytes) and +0x1184; try-lock sub_10009450 on dword_1011D0B0+0x370; sub_1001C0F4 (sync in), sub_1001CC40(+0x944) (message drain), sub_1001C204 (render + sync out), unlock.
- sub_1001C0F4: +0x950 tempo, +0x954 play, +0x958 reposition, +0x94C tick; with byte_10121ED8 == 0 also +0x960/+0x964/+0x968 -> globals +0xC/+0x10/byte 3 (loop) and transport +0x148/+0x14C/+0x150. sub_1001C000/sub_1001C0A0: in mode 46 with the host owning the loop the panel's loop edits go only to transport +0x148../+0x150.
- sub_1001C204: sub_10013240(song, drive) renders; then writes back +0x974 (transport+0x144 tempo), +0x978/+0x97C/+0x980 (loop), +0x982/+0x983 (echo of +0x96A/+0x96B), +0x96C/+0x96D/+0x970 (transport +0x151/+0x152/+0x154, the request bytes cleared after).
- sub_10013240: song try-lock (song+0x154); per channel 0..17: +0x804[ch] == 0 -> dword_100DEF08[ch] = dword_100DEEC0[ch] = 0 and +0x1188[ch] = 0; else DEF08[ch] = +0x844[ch] (host float*), DEEC0[ch] = song+0x20C+256ch (leftover), +0x1188[ch] = 1. Then the same leftover/64-frame/remainder logic as sub_100139E0 calling sub_10013444(DEF08) / sub_10013444(DEEC0). No n clamp, no int16, no preview.
- sub_10013444: channel pointers = &arr[2], &arr[3], &arr[4], &arr[11] (0x10013524..0x10013546). Entry non-null: device slot 14 gets &arr[k] and renders into the host buffer (0x1001356E). Null: arr[k] temporarily = unk_1010012C, device called, arr[k] restored (0x10013584..0x100135AD). Inserts dist/PCF/comp on the bus (0x100135AF..0x100135F1), peak meter, delay send loop (0x1001364A..0x1001369A) on the bus in both cases; cmp bus, unk_1010012C at 0x100136AB: a host buffer skips the gain smoothing and the master mix (jnz 0x10013808). Delay slot 14 with (send, L, R) and master comp with (L, R) only, master gain on arr[0]/arr[1].
- 808 slot 14 0x100280B0: zero arr[4]; BD arr[5] FILTER +0x18FC,+0x1900; SD arr[6] DUAL +0x1908; Tom arr[7] PITCHED +0x18F0,+0x18F4,+0x18F8; Per arr[8] SIMPLE +0x18DC,+0x18E0,+0x18E4 (RS/CL, CP/MA, CB); HH arr[9] SIMPLE +0x18E8,+0x18EC (OH, CH); CY arr[10] FILTER +0x1904. A non-null entry is zeroed first and the group renders into it instead of arr[4].
- 909 slot 14 0x1002B6F0: zero arr[11]; BD arr[12] FILTER +0x1904,+0x1908; SD arr[13] DUAL +0x190C; Tom arr[14] PITCHED +0x18F0..+0x18F8; Per arr[15] SIMPLE +0x18E0,+0x18E4 (RS, CP); HH arr[16] SIMPLE +0x18E8,+0x18EC (CH, OH); CY arr[17] PITCHED +0x18FC,+0x1900 (crash, ride).
- 303 slot 14 0x10023F10 reads only arr[k][0] (eax = arg 0x10023F26, edi = eax 0x10023FB8, ebx = [edi] 0x10023FEA); the 2.0 engine 0x100248C4 takes the same argument (esi = arg, 0x10024930).

Results:
- Device info: name, count 18, 18 x 32-byte names (room for 64), per-channel exists bytes +0x824, default-on bytes +0x864/+0x865 (Mix-L/R), stereo-pair byte +0x8A4 (pair 0), version 8 at +0x8C4 (matches TopHatGetDeviceNameAndVersionProc). Meanings of the flag arrays are [G] (the host's code is in ReWire.dll, not read).
- Device channels (303-1/2, 808-Mix, 909-Mix): mono, after the inserts, before level/pan/on, removed from Mix-L/R; the delay send stays (send gain includes T[lev]).
- Instrument channels: raw voice sums, removed from the machine's bus (so from inserts, send, level, pan, Mix), zero when idle.
- Delay return and master section only in Mix-L/R; no delay channel.
- Quirks: a routed channel's gain smoothing freezes (0x100136B7 skipped); a disabled Mix-L/R master bus is the never-refilled scratch. Not output-affecting defects, so not in original_bugs.md.
- Export path: same block with entries 2..17 null; the [G] of initial_investigation 7.1 confirmed and rewritten as [C]. No effect on orb's renders.
- Comparison with task-256 (6e57c73, docs/plugin.md 13) in docs/rewire.md 6.
- Decide task: task-259 (options A keep, B per-instrument copies, C ReWire routing, D delay return). The user decided A with the tap change on 2026-11-21 (the device outputs at ReWire's tap point, no per-instrument outputs), see task-259.

AC evidence: 1 docs/rewire.md sections 2 and 3 (every field with its address, [C]/[G]); 2 docs/rewire.md section 4 (the 808/909 group tables and the per-channel summary table, addresses 0x10013444, 0x100280B0, 0x1002B6F0, 0x10023F10); 3 docs/initial_investigation.md 7.1 rewritten as [C] task-257, plus a pointer in section 4; 4 docs/rewire.md section 6 table; 5 task-259 (@pingw33n, spike, decision,player, m-7, depends on task-256 and task-257).

Review round 1 (send back), fixes:
1. TopHatOpenDeviceProc now with evidence (scratch IDA copy reopened, closed save=false): sub_10017940("ReBirth DLL->Panel") at 0x1001B5F0 is CreateFileMappingA + GetLastError()==183 (0x10017940), so true when the mapping already exists -> return 4 (0x1001B68C); operator new(0x430) 0x1001B60C; ctor sub_1001C7B0 with 0x2E (46) at 0x1001B62A; stored at dword_1011D0B0+0x370 (0x1001B638); return 1 (0x1001B645), 5 on exception (0x1001B668). The ctor creates the transport (operator new(0x174), sub_1001BB80(46), dword_1011D0B4 at 0x1001C884); the 0x430 object is the engine object, not the transport (doc corrected). 'The app holds the engine' marked [G].
2. Quirks moved to docs/original_bugs.md as Q-6 (frozen gain smoothing, 0x100136A2/0x100136AB/0x100136B1, smoothing 0x100136B7..0x10013732, mix loop 0x10013742..0x100137FA) and Q-7 (disabled Mix-L/R scratch, 0x1001349D..0x100134A3, 0x100134DF..0x100134E6, fills 0x100134BA/0x100134FF); linked from rewire.md section 4. Gain offsets corrected (channel block +0xC/+8 = song +0xB0+44c/+0xAC+44c).
3. Summary table has an Addresses column per row.
4. +0x95C: the [C] part is the behavior (host loop applied 0x1001C12F..0x1001C16A, panel loop setters sub_1001C000/sub_1001C0A0 redirected); 'the host owns the loop' is [G]. sub_1001C050 dropped from the doc (not read).
5. Both docs now say: mov eax, offset unk_1010012C at 0x100136A2, cmp eax, ebp at 0x100136AB, jnz at 0x100136B1.
6. Rows for +0x982 (0x1001C276) and +0x983 (0x1001C270) added.
7. task-259: '18 channels, the Mix-L/R stereo pair and 16 mono channels'; rewire.md section 6 the same.
Also: routing sentence (one PCF, one comp, per-channel dist) in rewire.md section 4; the user's decision (2026-11-21, corrected: option A with the device outputs moved to ReWire's tap point, stereo with the mono signal, no per-instrument outputs, no delay return, nothing removed from the Mix) in rewire.md section 6 and task-259's final summary, task-259 Done.

Review round 2 (send back), fixes: the Results line and the final summary now give the user's decision (A with the tap change, 2026-11-21) instead of recommendation B; task-259's description has a 'Superseded' line after the recommendation; task-259's final summary says task-256 (not yet merged) is reworked on its own branch and records Decision: task-259 (task-256's file left to its agent); rewire.md section 6 says task-256 will implement it (see task-259); the split sentence of rewire.md section 3 step 1 rejoined.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Documented ReBirth's ReWire (TopHat) audio outputs in the new docs/rewire.md, from the disassembly of ReBirth.dll (a scratch copy of the IDA database, closed without saving): the exports, the 0x8D8-byte device info (18 channels, 64-channel arrays, Mix-L/R the only stereo pair, version 8), the drive-audio struct (enable bytes +0x804, buffers +0x844, frames +0x944, transport in/out fields, written bytes +0x1188) and the render sub_10013240 (the export's chunking without int16, clamp or preview).

Per channel: Mix-L/R are the master after the delay, master comp and master gain (f32); 303-1, 303-2, 808-Mix and 909-Mix are mono, after the channel's inserts and before level/pan/on, and leave Mix-L/R when enabled while their delay send stays; the twelve drum channels (BD, SD, Tom, Per, HH, CY per machine) are raw voice sums taken out of the machine's bus. No delay output. The export path is the same block with entries 2..17 null, so orb's renders are unaffected; the [G] in initial_investigation.md 7.1 is now [C]. docs/rewire.md section 6 lists the differences from task-256's outputs (pre- vs post-fader, mono vs stereo, removal from the Mix, per-instrument drums, int16 vs f32 Mix).

Links: docs/plugin.md 8.1, docs/initial_investigation.md 4 and 7.1 (hunks away from task-256's). Decide task: task-259; the user decided A with the tap change on 2026-11-21 (task-256's four device outputs moved to ReWire's tap point, after the inserts and before level, pan and on, stereo with the mono signal; no per-instrument outputs), see task-259. Branch worktree-agent-aaf74e609b8056c8c, one docs commit. Tests: none (docs only).
<!-- SECTION:FINAL_SUMMARY:END -->
