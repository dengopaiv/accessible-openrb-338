# The Standard MIDI Mapping of ReBirth 2.01 (task-182)

The original's predefined remote-control mapping: which MIDI message moves which panel control.
The manual calls it **Standard Mapping** ("Remote MIDI Control", printed pages 127-134; the
tables of "Appendix C: Standard MIDI Mapping Tables", pages 195-204). Its other mode, **Quick
Mapping** (a user table of controller numbers), is task-188's; `docs/midi.md` is the original's
MIDI input as a whole (the input callback, the settings, the timing) and what orb does with it.

Read in a read-only headless copy of `Rebirth.exe.i64` (2026-11-15). Every address is the exe's.
Every entry was checked against the manual's tables; the disagreements are in section 9. orb
follows the binary, never the manual, and lists where it deliberately differs from the binary in
section 10. Marks: [C] read in the disassembly, [G] guessed or inferred.

orb: `crates/orb-player/src/midi.rs` (`StandardMapping`, `control_change`), tested in the module
and in `crates/orb-ui/tests/midi.rs`.

## 1. Channel, messages, dispatch [C]

- **One channel.** The input callback `sub_408FD4` (`CReBirthMIDIInputParser`, vtable 0x4BA624
  slot 1) keeps a message only when Remote MIDI Control is on (`sub_409855`: byte `+0x530` of the
  settings object), its channel `status & 0x0F` equals the preferences' channel (`sub_409882`,
  compared at 0x409071), and its type `status & 0xF0` is 0x80, 0x90 or 0xB0 (0x4090A6). The same
  channel addresses every device: there is no channel per device. Every other message (pitch bend,
  aftertouch, program change, other channels) is dropped for remote control. (MIDI clock and the
  other sync messages go to the DLL before this test, 0x40901B; task-184.)
- **Dispatch.** The song window's idle handler `sub_42AC5D` (the window class's vtable slot at
  0x4BBFD8) drains up to 100 queued messages per idle call (0x42AD62) and hands each to
  `sub_42EAAF`, only for the window of the playing song (0x42AD58). `sub_42EAAF` takes 0x90 and
  0x80 as notes (section 4..7) and 0xB0 as control changes (section 2..3).
- **Note on, note off.** For a note the handler passes "on" = `status == 0x90` (`setz` at
  0x42EAFC). The velocity byte is never read (OB-24): a note-on of velocity 0 is a note-on.
- **Value range.** A control change's value is the data byte, 0..127 by the protocol. No handler
  checks it further except where section 2 and 3 give a scaling.

## 2. Control changes: mixer, synths, drum machines [C]

`device controller n` is the panel's own knob or switch message (`docs/song_load.md` 4.10: the
303's msg 41 sub 40 `sub_43EC83`, the 808's 57 `sub_441C82`, the 909's 74 `sub_445C41`); the
mixer's are msg 145 `{id, v}` (`sub_4131BC`) and 147 `{c, k, v}` (`sub_413206`). Each handler
stores the value in its view and sends the message, the same message as a mouse move, so the
DLL's record hook records it in song mode with record on (`docs/initial_investigation.md` 5.3).
"raw" is the CC value unchanged; "switch" is `v >= 64` (0 or 1).

### 2.1 Mixer (`sub_4137E2`, from 0x42EED9; manual p. 195)

| CC | Control | Message | Value |
|---|---|---|---|
| 7 | Master level | 145 `{0, v}` | raw |
| 11, 12, 13 | Synth 1 level, pan, delay | 147 `{0, 1..3, v}` | raw |
| 14, 15, 16 | Synth 2 level, pan, delay | 147 `{1, 1..3, v}` | raw (CC 16: below) |
| 17, 18, 19 | 808 level, pan, delay | 147 `{2, 1..3, v}` | raw |
| 20, 21, 22 | 909 level, pan, delay | 147 `{3, 1..3, v}` | raw (CC 21: below) |
| 8, 9, 10 | nothing | | |

CC 16 and CC 21 compute their store offset and their message's channel from globals
(`dword_4D9610`, `dword_4DFB24 + 0x140`, `dword_4D9514`, `dword_4D9490`, `GetCurrentProcessId`
for CC 16; `dword_4D8B74` for CC 21): copy-protection arithmetic like `docs/song_load.md`
section 9. [G] Their intended results are the ones of their neighbors' pattern: CC 16 stores
`+0x101` and sends `{1, 3, v}`, CC 21 stores `+0x10C` (the byte the send reads) and sends
`{3, 2, v}`; orb uses these.

### 2.2 Synths (`sub_4405B5`, from 0x42EF0F for 303 #1 and 0x42EF46 for 303 #2; p. 195)

| CC synth 1 | CC synth 2 | Control | 303 controller | Value |
|---|---|---|---|---|
| 23 | 30 | Waveform | 8 | switch (0x4405F9) |
| 24 | 31 | Tune | 2 | raw |
| 25 | 32 | Cutoff | 3 | raw |
| 26 | 33 | Reso | 4 | raw |
| 27 | 34 | Env Mod | 5 | raw |
| 28 | 35 | Decay | 6 | raw |
| 29 | 36 | Accent | 7 | raw |

### 2.3 TR-808 (`sub_443C92`, from 0x42EF7C; p. 196)

CC 37..63 are the 808's controllers 2..28 in order (`controller = CC - 35`): AC level (37), BD
level, tone, decay (38..40), SD level, tone, snappy (41..43), LT level, tune, switch (44..46), MT
level, tune, switch (47..49), HT level, tune, switch (50..52), RS level, switch (53, 54), CP level,
switch (55, 56), CB level (57), CY level, tone, decay (58..60), OH level, decay (61, 62), CH level
(63). The five switches (the shared-slot selectors LT/LC, MT/MC, HT/HC, RS/CL, CP/MA: CC 46, 49,
52, 54, 56, controllers 11, 14, 17, 19, 21) take `v >= 64`; the others raw.

CC 64 is **Instrument Selection**: `sub_4433C4(12 * v / 128)` (0x443C92's last case), the
instrument 0..11 whose steps the step buttons show (`+0x34` of the view, a word). No message.

### 2.4 TR-909 (`sub_447D9D`, from 0x42EFB2; p. 197)

CC 65..82 are controllers 2..19 (`CC - 63`): AC level, BD level, tune, attack, decay, SD level,
tune, tone, snappy, LT level, tune, decay, MT level, tune, decay, HT level, tune, decay. Then
CC 83 Hi-hat level (controller 22), CC 84 RS level (20), CC 85 CP level (21), and CC 86..92 are
controllers 23..29 (`CC - 63`): CH decay, OH decay, CC level, CC tune, RC level, RC tune, Flam.
All raw. (The panel's numbering puts RS, CP and HH at 20, 21, 22; the DLL stores them permuted,
`docs/song_load.md` 4.8.)

CC 93 is **Instrument Selection**, `sub_4474D6(12 * v / 128)`, as the 808's.

## 3. Control changes: master section, effects, shuffle [C] (p. 198)

| CC | Section, control | Handler, message | Value |
|---|---|---|---|
| 94 | PCF Pattern | `sub_41BE94`, PCF controller 4 | `clamp(v, 0, 54)` (`sub_467EBB(v, 0, 0x36)`, 0x41BEC5) |
| 95 | PCF Mode (LP/HP) | PCF 6 | switch |
| 96 | PCF Freq | PCF 1 | raw |
| 97 | PCF Q | PCF 2 | raw |
| 98 | PCF Amount | PCF 3 | raw |
| 99 | PCF Decay | PCF 5 | raw |
| 100 | Delay Steps | `sub_40827C`, delay controller 1 | `32 * v / D + 1`, below |
| 101 | Delay Triplet mode | delay 2 | 1 when `v >= 64`, else below |
| 102 | Delay Pan | delay 4 | raw |
| 103 | Delay Feedback | delay 3 | raw |
| 104 | Dist Shape | `sub_408C98`, distortion controller 2 | raw |
| 105 | Dist Amount | distortion 1 | raw |
| 106 | Compressor Ratio | `sub_404798`, compressor controller 1 | raw |
| 107 | Compressor Threshold | compressor 2 | raw |
| 108 | Shuffle Amount | `sub_429CF7`, song msg 35 `{20, v}` | raw |
| 109..127, 0..6 | nothing | | |

The delay's two values go through copy-protection arithmetic again (0x4082BD..0x408301: the
divisor `D = ((dword_4DA0AC >> 16) & 0xFFFF) - ([dword_4DFB24 + 0x13C](.., -6) & 0x7F) + 1`;
0x40833F..0x408381 for the triplet's "off" value from `dword_4D9F2C`, `dword_4D8AF8`,
`dword_4D90BC`). [G] The intended values: `D = 128`, so Steps = `v / 4 + 1` = 1..32, the panel's
range; the triplet's off value 0. orb uses these.

## 4. Notes in both keyboard modes (`sub_42EAAF`; manual p. 199 "Various Switches") [C]

Every function below acts on the note-on only (each handler tests its "on" argument), so a note
off does nothing. "Toggle" flips the value the view holds and sends it.

| Note | Function | What it does (address) |
|---|---|---|
| 64 | Select Pattern / Program Synth | flips the keyboard mode 10 <-> 11 (0x42EB0D..0x42EB33), section 6 |
| 65..68 | Focus to Synth 1, Synth 2, 808, 909 | `sub_42A89E(1..4)`: the focus byte `+0x8C`, and song msg 35 `{22, row}` |
| 69 | Transport Play | `sub_429FCF`: song msg 35 `{1, 1}` |
| 70 | Transport Stop | `sub_429FE6`: record off `{2, 0}` if on; then stop `{1, 0}` if playing, else locate to the loop start, or to 0 when the loop start is at or after the position (the Stop button's code, `docs/ui_spec.md` 3) |
| 71 | Transport Record | in song mode only (transport `+0x2C`, 0x429C62): toggles record, `{2, v}` |
| 72, 73 | Transport "Bar -", "Bar +" | in song mode only: `sub_429DF2(-7680)` / `(+7680)`, a locate to the start of the current bar minus / plus **10 bars**, clamped to 0..766464 (`sub_429DBF`): the Rewind / Fast Forward step. [G] When the transport's `+0x4C` flag is set it moves a held position `+0x48` instead of locating |
| 74 | PCF Enable | `sub_41BE32`: toggles PCF controller 0 |
| 75 | Delay Enable | `sub_40821A`: toggles delay controller 0 |
| 76 | Dist Enable | `sub_408C36`: toggles distortion controller 0 |
| 77 | Comp Enable | `sub_404736`: toggles compressor controller 0 |
| 78, 82, 86, 90 | Synth 1, Synth 2, 808, 909 Mix On/Off | `sub_413256`: toggles the channel's on byte, 147 `{c, 0, v}` |
| 79, 83, 87, 91 | the same channels' Dist On/Off | toggles the channel's dist byte, 147 `{c, 4, v}` |
| 80, 84, 88, 92 | the same channels' PCF On/Off | PCF id = 0 if it is `c + 1`, else `c + 1`; 145 `{2, id}` |
| 81, 85, 89, 93 | the same channels' Comp On/Off | compressor id = 0 if it is `c + 2`, else `c + 2`; 145 `{1, id}` |
| 94 | Master Comp On/Off | compressor id = 0 if it is 1, else 1; 145 `{1, id}` |
| 95 | Program Synths from Keyboard | keyboard mode 10 (0x42EB5C) |
| 96 | Select Patterns from Keyboard | keyboard mode 11 (0x42EB86) |

Notes 0..11 and 97..127 do nothing; notes 12..63 are section 5 or 7 by the mode.

## 5. Pattern selection (notes 12..63 in mode 11; `sub_41AE84`; manual pp. 200-201) [C]

In "Select Patterns from Keyboard" mode (11) each pattern section has 13 notes from its base
`12 + 13r` (r = 0..3: Synth 1 at 12, Synth 2 at 25, the 808 at 38, the 909 at 51), each to the
section's view (`+0x1C4`, `+0x354`, `+0x4E4`, `+0x674` of the window, 0x42ED71..0x42EE14):

| Note | Function | What it does |
|---|---|---|
| base | Pattern On/Off | toggles the section's on button (`+0x128`, a `CToggleBitmapButton`; vtable 0x4BBC0C slot +0x60 `sub_4250D8` toggles on a note-on when the button is enabled), which sends the device's controller 0 as a click does |
| base + 1..4 | Bank A..D | the view's bank select (vtable +0x84): the shown bank, no message |
| base + 5..12 | Pattern 1..8 | pattern `i + 8 * bank` (the shown bank, vtable +0x88), the view's pattern select (vtable +0x64): the device's controller 1, as a pattern button does; it takes effect at the end of the playing pattern as any pattern change |

The 909's 13 notes end at 63. (The range tests at 0x42ED4E..0x42EE07 test 25..37, 25..50 and
25..63 after 12..24; the earlier tests catch the lower notes first, so each note reaches its own
section.)

## 6. The keyboard mode [C]

The Options menu's "Program Synth from Keyboard" and "Select Patterns from Keyboard" are one value
(`+0x4F8` of the settings object, `sub_408F4C`), mode 10 or 11; the menu commands 401 and 402
(`sub_40C163`) and the MIDI notes 95, 96 and 64 set it with `sub_408F60`, which also sends
top-level msg 76 `{6, 1}` for mode 10 and `{6, 0}` for 11 (the DLL's use of it not read). The
preferences save it (`+0x214` of the stored preferences, read at 0x40E2E0); a new preference set
has mode 11 (`sub_41C210`, 0x41C272).

## 7. Program Synth from Keyboard (notes 12..63 in mode 10; pp. 202-204) [C]

In mode 10 a note below 64 goes to the focused section (`+0x8C`; 0x42EBC4) for step programming:

- **303** (`sub_4402AB`, 303 view `+0x14B8` / `+0x2600`): notes 12..24 press the 13 pitch keys
  C..C (members `+0x18C`..`+0x1BC`), 25 and 26 Octave Down and Up (`+0x1C0`, `+0x1F0`), 27
  Accent (`+0x250`), 28 Slide (`+0x220`) ([G] these four names are the manual's; the buttons were
  not identified), 29 Note/Pause (`+0x2E0`, OB-13's button), **30 and
  31 both Back** (`+0x2B0`, the button Backspace presses: key code 30, `sub_492312` case 8; the
  jump table 0x440561 gives cases 30 and 31 the same code, 0x44050C and 0x440527) (OB-25), 32
  Pitch Mode (`+0x310`). Each passes "on" to the button (vtable +0x60): pressed on the note-on,
  released on the note-off, as the computer keyboard does.
- **808** (`sub_4435D9`): notes 12..27 toggle step 1..16 of the selected instrument in the shown
  pattern (`sub_44305C`, the step buttons' message 62); 28..39 select instrument 0..11 (AC, BD,
  SD, LT, MT, HT, RS, CP, CB, CY, OH, CH).
- **909** (`sub_447680`): the same, with the 909's instruments (AC, BD, SD, LT, MT, HT, RS, CP,
  CH, OH, CC, RC); notes 34 and 36 compute the instrument by copy-protection arithmetic ([G]
  intended 6 and 8). Note 40 toggles the view's byte `+0x6BC` ([G] the Flam button's state,
  `tr909.flam_button`), which the manual does not list.

orb does not map these notes yet (the step programming lives in orb-ui's panel model, not in a
player command); task-189 does.

## 8. Defaults and settings

The mapping depends on the preferences' MIDI channel (default channel 1 [G]: a new preference
set's byte for it is 0, `sub_41C210` 0x41C2B8, read at 0x40E267 as `+0x270` under a 44-byte shift
of the stored layout that was not traced), on Remote MIDI Control being on (default off [G],
0x41C293), on Standard (40) rather than Quick (41) mapping (default Standard, 0x41C29D; the
dialog's radio, `sub_41E959` 0x41EABE / 0x41EACA), and on the keyboard mode (section 6).
`docs/midi.md` has the settings. orb's layer listens to one channel (`StandardMapping::channel`,
default 0 = channel 1), is on when its source feeds it, and has the mode.

## 9. Differences from the manual

Each: the manual's statement and page; what the binary does; what orb does (always the binary,
except where section 10 says otherwise).

1. **Stop and Record note numbers** (p. 199, "Various Switches"): "B3 70 Transport - Record",
   "A#3 71 Transport - Stop". The binary: note 70 is Stop and 71 is Record (`sub_429BEC` cases
   0x46 → `sub_429FE6` at 0x429C46, 0x47 → the record toggle at 0x429C62..0x429C88). The manual's
   key names agree with the binary under its own numbering (C4 = 72, so A#3 = 70, B3 = 71); its
   number column is swapped. orb: 70 Stop, 71 Record.
2. **"Bar -" / "Bar +"** (p. 199): notes 72, 73 "Transport - Bar -, Bar +". The binary moves 10
   bars, from the start of the current bar (`sub_429DF2(-7680 / +7680)`, 0x429CB0 / 0x429CD8): the
   Rewind and Fast Forward step, not the Bar arrows' one bar. orb: 10 bars.
3. **Transport notes need song mode** (p. 199, "keys that can be used in all modes"): Record (71)
   and the two bar notes (72, 73) do nothing in pattern mode (transport `+0x2C` tested at
   0x429C62, 0x429CA1, 0x429CC9). orb: the same.
4. **Synth "Step" and "Back"** (p. 202, "Synth Section Switches"): "G0 31 Step", "F#0 30 Back".
   The binary sends both notes to the Back button (section 7, OB-25); Step has no note. orb: not
   mapped (section 7).
5. **909 note 40** (p. 204 lists 12..39): in Program Synth mode the binary toggles the 909 view's
   byte `+0x6BC` on note 40 (section 7). orb: not mapped.
6. **PCF Pattern range** (p. 198 gives no range; p. 206, Appendix D: "the 34 PCF patterns"): CC
   94 is clamped to 0..54, the 55 patterns the DLL has (`docs/initial_investigation.md`, the PCF).
   orb: 0..54.
7. **Value scalings the manual does not give** (pp. 195-198). The binary:
   - `v >= 64` (`cmp ..., 40h`): the waveform, CC 23 and 30 (`sub_4405B5`, 0x4405F0); the 808's
     five switches, CC 46, 49, 52, 54, 56 (`sub_443C92`, 0x443E7D, 0x443F1B, 0x443FB9, 0x444027,
     0x444094); the PCF mode, CC 95 (`sub_41BE94`, 0x41BF17); the delay's triplet mode, CC 101
     (`sub_40827C`, 0x40833A).
   - `12 * v / 128` (`imul eax, 0Ch`, `sar eax, 7`): the instrument selections, CC 64
     (0x444216..0x44421F) and CC 93 (`sub_447D9D`, 0x44830D..0x448316).
   - `32 * v / D + 1` (`shl eax, 5`, `div ecx`, `add eax, 1`; 0x4082DA..0x408301, [G] D = 128):
     the delay steps, CC 100.

   orb: the same.
8. **Velocity 0** (pp. 199-204 speak of notes only): a note-on of velocity 0 acts as a note-on.
   The binary sets "on" from the status alone (`cmp dword ptr [ecx+8], 90h` / `setz dl`,
   0x42EAF5..0x42EAFC) and never reads a note's velocity (OB-24). orb differs, section 10.
9. **Only the note-on acts** (pp. 199-201 do not say). Every toggle and selection in sections 4
   and 5 tests the "on" flag of 0x42EAFC before acting: the mode and focus notes at 0x42EB1A and
   0x42EC7B..0x42ECCE (disassembly), and inside the handlers it passes the flag to: the
   transport `sub_429BEC`, the effect enables (`sub_41BE32`, `sub_40821A`, `sub_408C36`,
   `sub_404736`), the mixer `sub_413256`, the pattern rows `sub_41AE84` and the on button
   `sub_4250D8` (each `if (on)` read in Hex-Rays only). A note-off does nothing. orb: the same.

Everything else in the manual's tables (pp. 195-204) matches the binary: every CC number and the
control it names, the mixer's, the 808's and the 909's order (the 909's 83 Hi-hat, 84 RS, 85 CP
are its controllers 22, 20, 21), every note of "Various Switches" but the ones above, and the
pattern-selection table.

## 10. Where orb differs from the binary

- **Velocity-0 note-on** is a note-off in orb (the MIDI convention: many keyboards and every
  sequencer send it for a release). The original's would toggle twice per key press on such a
  keyboard (OB-24).
- **The copy-protected values** (CC 16, CC 21, CC 100, CC 101) take their intended values [G]
  (sections 2.1, 3), not a computation from the protection globals. (The 909's program notes 34
  and 36, section 7, are not mapped yet: task-189.)
- **Always on.** The original's Remote MIDI Control is a preference, off by default [G] (section 8:
  `sub_41C210` stores 0 at 0x41C293, under the unreconciled layout shift); orb's layer has no
  on/off: it acts whenever a source feeds it. Whether a source feeds it is the plugin's and
  task-188's choice.
- **Timing.** The original applies MIDI at the song window's idle time, up to 100 messages a call
  (section 1); orb's layer keeps each message's time and leaves when to apply it to the caller
  (`docs/midi.md`).
- **The Program Synth notes** (section 7) are not mapped yet.
- **The focus note** (65..68) gives orb's view its focus; the original's song msg 35 `{22, row}`
  is not sent (orb's engine has no use for it; [G] task-183's live notes may need it).
