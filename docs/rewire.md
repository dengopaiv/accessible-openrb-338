# ReBirth's ReWire (TopHat) audio outputs

ReBirth 2.01 talks to a ReWire mixer through seven exports of `ReBirth.dll` named `TopHat*`
(TopHat is ReWire's working name; `ReWire.dll` is a separate file on the CD, and `Rebirth.exe` has
no ReWire code). This document describes the audio side: the device the DLL declares, the
structure the host passes on each audio call, and where each of the 18 channels takes its signal.
It was written for task-257 from the disassembly of `ReBirth.dll` (addresses `0x10xxxxxx`); the
host sync is in upstream's plugin document (removed, task-231) 8.1. orb does not implement ReWire; what the plugin replicates
of it was decided in task-259 (section 6).

[C] is checked in the disassembly; [G] is a reading of the values without the host's code (the
meaning of a field the DLL only writes, or only copies).

## 1. The exports

| Export | Address | What it does |
|---|---|---|
| `TopHatGetDeviceNameAndVersionProc` | 0x1001B320 | `*version = 8`, name "ReBirth Audio" [C] |
| `TopHatGetDeviceInfoProc` | 0x1001B350 | Fills the device info (section 2) [C] |
| `TopHatOpenDeviceProc` | 0x1001B5C0 | Returns 4 (0x1001B68C) when `sub_10017940("ReBirth DLL->Panel")` (0x1001B5F0) finds that file mapping already there (`CreateFileMappingA`, `GetLastError() == 183`, 0x10017940): the app holds the engine [G for "the app"]. Else it creates a 0x430-byte engine object (`operator new(0x430)`, 0x1001B60C; ctor `sub_1001C7B0(46)`, 0x1001B62A), stores it at `dword_1011D0B0+0x370` (0x1001B638) and returns 1; 5 on an exception (0x1001B668). The ctor creates the transport (0x174 bytes, `sub_1001BB80(46)`, stored in `dword_1011D0B4` at 0x1001C884), whose mode `+0x15C` = 46 the loop setters test (`sub_1001C000`) [C] |
| `TopHatIsCloseOKProc` | 0x1001B6C0 | Not read for this document |
| `TopHatCloseDeviceProc` | 0x1001B770 | Not read for this document |
| `TopHatDriveAudioProc` | 0x1001B800 | One audio call (section 3) [C] |
| `TopHatIdleProc` | 0x1001B910 | Not read for this document |

## 2. The device info (0x8D8 bytes)

`TopHatGetDeviceInfoProc(info)` zeroes 0x236 dwords (`rep stosd`, 0x1001B37E) and sets:

| Offset | Value | Meaning |
|---|---|---|
| `+0x000` | "ReBirth Audio" (`sprintf`, 0x1001B39F) | Device name, 32 bytes [C] value, [G] size |
| `+0x020` | 18 (0x1001B3B4) | Channel count [C] |
| `+0x024 + 32i` | The channel names, i = 0..17 (strings 0x101534E0..0x101535F0, copied 0x1001B3AC..0x1001B54B) | 32-byte names [C]; room for 64 channels (to `+0x824`) [G] |
| `+0x824 + i` | 1 for i = 0..17 (loop 0x1001B562, `cmp ebx, 12h`) | One byte per channel, 64 bytes: the channel exists / may be enabled [G] |
| `+0x864`, `+0x865` | 1 (0x1001B57A, 0x1001B586) | One byte per channel, 64 bytes: channels 0 and 1 (Mix-L, Mix-R) on by default [G] |
| `+0x8A4` | 1 (0x1001B580) | One byte per channel pair, 32 bytes: pair 0 (Mix-L/Mix-R) is a stereo pair [G] |
| `+0x8C4` | 8 (dword, 0x1001B394) | The version; `TopHatGetDeviceNameAndVersionProc` returns the same 8 [G] |
| `+0x8C8..+0x8D7` | 0 | Unused by the DLL |

The 18 channels and their indices (the same index is the entry of the 18-pointer output array
that the mixer block `sub_10013444` takes, section 4):

| i | Name | i | Name | i | Name |
|---|---|---|---|---|---|
| 0 | Mix-L | 6 | 808-SD | 12 | 909-BD |
| 1 | Mix-R | 7 | 808-Tom | 13 | 909-SD |
| 2 | 303-1 | 8 | 808-Per | 14 | 909-Tom |
| 3 | 303-2 | 9 | 808-HH | 15 | 909-Per |
| 4 | 808-Mix | 10 | 808-CY | 16 | 909-HH |
| 5 | 808-BD | 11 | 909-Mix | 17 | 909-CY |

Only pair 0 is flagged as stereo: every other channel is mono, and there is no delay channel.

## 3. The drive-audio call

`TopHatDriveAudioProc(drive)` (0x1001B800) [C]:
1. `byte_10121ED8 = (drive+0x95C == 0)` (0x1001B82C..0x1001B83F). With it 0 (`+0x95C` non-zero),
   `sub_1001C0F4` takes the host's loop (0x1001C12F..0x1001C16A) and the panel's loop setters in
   mode 46 (`sub_1001C000` the loop start, `sub_1001C0A0` the loop switch) store into the
   transport only, which goes back to the host ([C]). [G] reading: `+0x95C` says the host owns the loop.
2. Clears the 64 bytes `drive+0x1188..+0x11C7` and the dword `drive+0x1184` (0x1001B846..0x1001B865).
3. Try-locks the engine (`sub_10009450` on `dword_1011D0B0+0x370`); if that fails, returns with
   nothing written (every channel's "written" byte stays 0).
4. `sub_1001C0F4(drive)`: the host's transport in (upstream's plugin document (removed, task-231) 8.1).
5. `sub_1001CC40(drive+0x944)`: drains the message port, time-budgeted by the frame count (as
   `sub_100139E0` does in the export path, `docs/export_state.md` 2.3).
6. `sub_1001C204(drive)`: renders (`sub_10013240(song, drive)`, 0x1001C22B), then the transport out.
7. Unlocks.

The fields the DLL reads and writes (offsets in bytes):

| Offset | Dir | Read/written at | Meaning |
|---|---|---|---|
| `+0x804 + i` | in | 0x1001328F | Channel i enabled by the host (64 bytes) [C] |
| `+0x844 + 4i` | in | 0x100132B8 | Channel i's buffer, `float *` (64 pointers) [C] |
| `+0x944` | in | 0x10013258, 0x1001B8A0 | Frame count [C] |
| `+0x94C` | in | 0x1001C1DA, 0x1001C1F3 | The host's position in ticks [C] |
| `+0x950` | in | 0x1001C118 | Tempo (song globals `+4`, transport `+0x144`) [C] |
| `+0x954` | in | 0x1001C178 | Play state; a change plays or stops the song [C] |
| `+0x958` | in | 0x1001C1C4 | Reposition: locate instead of follow [C] |
| `+0x95C` | in | 0x1001B82C | Non-zero: the host's loop is applied and the panel's loop edits go to the host (step 1) [C]; "the host owns the loop" [G] |
| `+0x960`, `+0x964`, `+0x968` | in | 0x1001C137..0x1001C16A | Loop start, loop end (ticks), loop on (byte); applied to the song (globals `+0xC`, `+0x10`, byte 3) and the transport (`+0x148`, `+0x14C`, `+0x150`) only when `+0x95C` is non-zero (`byte_10121ED8` = 0, tested at 0x1001C12F) [C] |
| `+0x96A`, `+0x96B` | in | 0x1001C264, 0x1001C25E | Copied to `+0x982`, `+0x983` (rows below) [C], meaning unknown |
| `+0x96C`, `+0x96D` | out | 0x1001C2B5, 0x1001C2C7 | Transport bytes `+0x151` and `+0x152`, cleared after the copy: the device's requests to the host (`+0x151` is dropped when it asks for the state the host already has, 0x1001C284..0x1001C2A2) [C] copy, [G] meaning (play/stop requests) |
| `+0x970` | out | 0x1001C2D3 | Transport `+0x154` [C] copy, [G] the requested position |
| `+0x974` | out | 0x1001C23A | Transport `+0x144`, the tempo [C] |
| `+0x978`, `+0x97C`, `+0x980` | out | 0x1001C240..0x1001C26A | Transport `+0x148`, `+0x14C`, `+0x150`: the loop as the device has it. With `+0x95C` non-zero, the panel's loop edits go only there (`sub_1001C000`, `sub_1001C0A0` in mode 46 with `byte_10121ED8` = 0), so the host gets them [C]; [G] as a request |
| `+0x982` | out | 0x1001C276 | The byte `+0x96A` of the same call [C], meaning unknown |
| `+0x983` | out | 0x1001C270 | The byte `+0x96B` of the same call [C], meaning unknown |
| `+0x1184` | out | 0x1001B865 | Cleared, never set by the DLL [C]; [G] an event count |
| `+0x1188 + i` | out | 0x100132AF, 0x100132D9 | Channel i written: 1 for each enabled channel once the song lock is held, else 0 [C] |

**The render, `sub_10013240(song, drive)` [C].** After the song's try-lock (`song+0x154`), for each
channel i < 18: if `drive+0x804+i` is 0, the output entry `dword_100DEF08[i]` and the leftover entry
`dword_100DEEC0[i]` are null and the written byte 0; else `dword_100DEF08[i]` is the host's buffer,
`dword_100DEEC0[i]` the song's leftover block `song+0x20C + 256i`, and the written byte 1. Then it
runs the export path's chunking (`sub_100139E0`, `docs/export_state.md` 5): the leftover samples of
the last call first, whole 64-frame blocks straight into the host's buffers
(`sub_10013444(dword_100DEF08)`, each non-null pointer advanced by 0x100 after a block), and a
partial block rendered into the leftover blocks and copied out (refilled with 1e-6). It differs
from `sub_100139E0` in that it has no frame clamp (`sub_100139E0` clamps to 0x4000), no int16
conversion (the host gets the f32 samples, full scale 1.0), no preview-sample player, and the
enabled channels are the host's instead of `byte_100DF754` (only 0 and 1 in the app). A host
calling with multiples of 64 frames never uses the leftover.

## 4. Where each channel is tapped [C]

The mixer block `sub_10013444` (`docs/initial_investigation.md` section 4) takes the 18-pointer
array. Entries 0 and 1 are the master buses: a non-null one is filled with the 1e-6 guard at the
start of the block; a null one is replaced by the scratch `song+0x140C` / `+0x150C`, which is never
refilled. The four mixer channels use the pointers `&arr[2]`, `&arr[3]`, `&arr[4]`, `&arr[11]`
(0x10013524..0x10013546). For channel c:

- **Device.** If its entry is non-null the device's audio slot (slot 14, vtable `+0x38`) gets
  `&arr[k]` and renders into the host's buffer (0x1001356E). If it is null, the block stores the
  shared scratch `unk_1010012C` into `arr[k]`, calls the device and restores the null
  (0x10013584..0x100135AD). The device so sees its own entry and, for the drum machines, the six
  entries after it as they are.
- **Inserts.** The distortion, the PCF and the compressor routed to the channel run in place on
  that buffer (0x100135AF..0x100135F1), so a channel's ReWire buffer is after its inserts.
- **Meter.** The channel's peak takes `|bus[i]|` (0x100135FA), the bus before level and pan.
- **Delay send.** `send[j] = f32(bus[j]*sendGain + send[j])` (0x1001364A..0x1001369A) runs in both
  cases. The send gain is `T[lev]*T[send] + 1e-6` (or the Vintage pan form), with `lev = on ? level
  : 0`, so a channel on its ReWire output still feeds the delay, scaled by its level fader and on
  switch.
- **Level, pan, master.** `mov eax, offset unk_1010012C` (0x100136A2), `cmp eax, ebp` (0x100136AB,
  ebp the channel's buffer), `jnz loc_10013808` (0x100136B1): with a host buffer the block
  skips both the gain smoothing (0x100136B7) and the pan-and-mix loop (0x10013742). The channel
  is not in Mix-L/R; its level, pan and on switch do not reach its ReWire output.
- **Master section.** The delay (slot 14 with the send bus and entries 0 and 1, 0x10013844), the
  master compressor (entries 0 and 1, 0x10013883) and the master gain (0x100138AE) act on the
  master buses only. The delay's return reaches Mix-L/R and nothing else.
- **Routing.** ReWire uses the app's mixer and its routing unchanged (mixer update 0x10012C14,
  `docs/initial_investigation.md` section 4): one PCF, on one channel (`pcfId == c+1`), one
  compressor, on one channel or the master (`compId`), and the distortion with a switch per
  channel and one shared parameter set. Several channels cannot have their own PCF or
  compressor.

**The 303s** (slot 14 0x10023F10, Vintage; the 2.0 engine 0x100248C4 takes the same argument)
read only entry 0 of what they get (`ebx = [edi]`, 0x10023FEA) and overwrite it (`out = f32(o)`,
`docs/initial_investigation.md` 6.2).

**The 808** (slot 14 0x100280B0) zeroes `arr[4]` (the 808-Mix buffer, or the scratch), then
renders six groups. For each group, a non-null entry is zeroed and the group's voices add into it
instead of into `arr[4]`:

| Entry | Channel | Voices, in this order (fields of the 808) |
|---|---|---|
| 5 | 808-BD | FILTER `+0x18FC` (BD A), `+0x1900` (BD B) |
| 6 | 808-SD | DUAL `+0x1908` (SD) |
| 7 | 808-Tom | PITCHED `+0x18F0`, `+0x18F4`, `+0x18F8` (LT/LC, MT/MC, HT/HC: the toms or congas) |
| 8 | 808-Per | SIMPLE `+0x18DC`, `+0x18E0`, `+0x18E4` (RS/CL, CP/MA, CB) |
| 9 | 808-HH | SIMPLE `+0x18E8`, `+0x18EC` (OH, CH) |
| 10 | 808-CY | FILTER `+0x1904` (CY) |

**The 909** (slot 14 0x1002B6F0) the same with `arr[11]`:

| Entry | Channel | Voices, in this order (fields of the 909) |
|---|---|---|
| 12 | 909-BD | FILTER `+0x1904`, `+0x1908` (BD A, BD B) |
| 13 | 909-SD | DUAL `+0x190C` (SD) |
| 14 | 909-Tom | PITCHED `+0x18F0`, `+0x18F4`, `+0x18F8` (LT, MT, HT) |
| 15 | 909-Per | SIMPLE `+0x18E0`, `+0x18E4` (RS, CP) |
| 16 | 909-HH | SIMPLE `+0x18E8`, `+0x18EC` (CH, OH) |
| 17 | 909-CY | PITCHED `+0x18FC`, `+0x1900` (crash, ride) |

A per-instrument buffer holds the raw voice sum: each voice's trigger gain already has the
instrument's level knob, the accent and the voice's envelope (`docs/initial_investigation.md` 7.1
to 7.3), and nothing after it. A group on its own output is taken out of its machine's bus, so
it is also out of the machine's inserts, meter, delay send, level, pan and Mix-L/R. The voices
add into a zeroed buffer, so an idle instrument channel is exactly 0 (no 1e-6 guard). The
remaining voices of the bus keep their order, so the bus's f32 sum is the same sum without the
removed terms.

**Summary per channel.**

| Ch | Source | Tap | Inserts | Level, pan, on | Delay | Master comp, master level | Mono | Removed from | Addresses |
|---|---|---|---|---|---|---|---|---|---|
| 0, 1 Mix-L/R | The master buses | After the master gain | Through the channels | Yes | Return mixed in | Yes | Stereo pair | - | Guard fill 0x1001349D..0x10013522; mix loop 0x10013742; delay 0x10013844; master comp 0x10013883; master gain 0x100138AE |
| 2 303-1, 3 303-2 | The 303's output | After the channel's inserts, before level and pan | Yes | No | Still sent (send x level) | No | Mono | Mix-L/R | Entries 0x10013528, 0x1001352B; device 0x1001356E (303 0x10023FEA); inserts 0x100135AF..0x100135F1; send 0x1001364A; bypass 0x100136AB/0x100136B1 |
| 4 808-Mix, 11 909-Mix | The machine's bus: the voices whose group has no output of its own | Same | Yes | No | Still sent | No | Mono | Mix-L/R | Entries 0x1001352E, 0x10013533; bus zeroed 0x100280BD (808), 0x1002B6FD (909); then as the 303s |
| 5..10, 12..17 instruments | The group's voices | The voice sum, before the machine's bus | No | No | No | No | Mono | The machine's bus, so its Mix channel and Mix-L/R | 808 groups 0x100280D8..0x10028273, 909 groups 0x1002B718..0x1002B8AF (the tables above) |

With a device channel on its own output and Mix-L/R enabled, Mix-L/R is the rest of the song: the
other channels, plus the delay return of every channel's send.

**Quirks [C]** (`docs/original_bugs.md`, quirks Q-6 and Q-7).
- A channel on its ReWire output skips the gain smoothing (0x100136B7), so its current L and R
  gains (the channel block's `+0xC` and `+8`, song `+0xB0 + 44c` and `+0xAC + 44c`) freeze;
  when the host disables the channel again, the smoothing starts from the frozen gains, so after
  a level or pan change made meanwhile the channel glides back in from its old gains (Q-6).
- With Mix-L or Mix-R disabled, that master bus is the never-refilled scratch: each block adds
  into the last block's value times the master gain. Nothing is output from it, but the master
  compressor (when on the master) and the master peak meter see it (Q-7).

## 5. The export path and orb's renders

The export (`SafeSoundCallback` → `sub_100139E0`) and ReWire run the same `sub_10013444`; the
export enables only channels 0 and 1, so entries 2..17 are null, every device renders into the
scratch, every drum voice sums into its machine's bus, and every channel mixes into the master
(this settles the [G] of `docs/initial_investigation.md` 7.1). A ReWire host enabling only Mix-L/R
gets the export's f32 samples before the int16 conversion (and without the preview player). orb
models the export path only, so nothing here changes its bit-exact renders. A replication of a
ReWire tap in orb would reuse the same block: the per-instrument outputs are sums the 808/909
models already form (orb-engine `drums.rs`, `tr808.rs`, `tr909.rs`) before they reach the bus.

## 6. Compared with the plugin's per-device outputs (task-256)

task-256 gives the plugin a stereo Mix and four stereo aux outputs, 303-1, 303-2, 808 and 909
(upstream's plugin document (removed, task-231) 13). The differences from ReWire:

| | ReWire (original) | Plugin (task-256) |
|---|---|---|
| Channels | 18: the Mix-L/R stereo pair and 16 mono channels | 5 stereo buses |
| Device tap | After the inserts, before level, pan and on | The same point (0x100135AF..0x100135F1, before the meter at 0x100135FA) |
| Device output stereo | Mono | The mono signal on both sides |
| Effect on the Mix | Enabling a device channel removes it from Mix-L/R (its delay send stays) | None: the Mix is bit-identical whatever is connected; the buses are copies |
| Delay send of a routed channel | Kept, so its echo stays in Mix-L/R | Not on the buses (the delay return is in the Mix only) |
| Per-instrument drum outputs | 6 per machine (BD, SD, Tom, Per, HH, CY), raw voice sums before the machine's bus, removed from it | None |
| Delay return output | None | None |
| Mix format | f32, unclamped, 1e-6 guard times the master gain | The original's int16 output over 32768 (clamped, truncated), plus the preview player |
| Mix content | Master after delay, master comp and master gain | Same |
| Buses format | f32, not clamped | f32, not clamped, full scale 1.0 |
| Host control | Per channel enable bytes; an enabled channel is always written | An unconnected bus is not written; the engine computes every tap |
| Rate | The engine's 44.1 kHz [G] (the host's rate is not read by `TopHatDriveAudioProc`) | Resampled to the host's rate like the Mix |

The user's decision (task-259, 2026-11-21): the plugin keeps task-256's four device outputs and
adds no per-instrument drum outputs (revised by task-300, below); the device outputs are at
ReWire's tap point, after the channel's inserts and before its level, pan and on switch, as
stereo buses carrying the mono signal on both sides. No delay-return output, and nothing is removed from the Mix (the outputs
are copies). task-256 implements it.

**task-300 (the user's decisions of 2026-11-25).** The plugin now has ReWire's routing:
Options > Output Routing takes a device out of Mix-L/R as an enabled ReWire device channel does
(its send kept, its gains frozen; section 4's bypass), and a drum instrument out of its machine's
bus as an enabled instrument channel does, with one output per instrument instead of ReWire's six
groups (808: BD, SD, LT, MT, HT, RS, CP, CB, OH, CH, CY; 909: BD, SD, LT, MT, HT, RS, CP, CH, OH,
CC, RC), 22 more stereo buses after the four. With every box checked the buses stay copies and the
Mix is unchanged. upstream's plugin document (removed, task-231) section 13.1.
