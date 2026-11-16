# MIDI in ReBirth 2.01 and in orb

MIDI came into scope on 2026-11-13 (milestone m-7). This is the general MIDI document: the
original's MIDI input, its settings, and what orb does with them. Tasks extend it: task-182 (this
document and the Standard Mapping), task-183 (notes played live: the original has none, section
4), task-184 (MIDI clock), task-188 (the standalone app's ports, Quick Mapping, the MIDI LED,
recording by MIDI), task-199 (the plugin's own note mapping, section 5).

**The mapping itself** (which CC and which note moves which control, with the addresses and the
differences from the manual) is `docs/standard_midi_mapping.md`; this document does not repeat it.

Sources: the manual (`rebirth_2.01_manual.pdf`, "Remote MIDI Control", printed pages 127-134;
"Synchronization", pp. 113-125; Appendix C, pp. 195-204) and a read-only headless copy of
`Rebirth.exe.i64` (2026-11-15). Marks: [C] read in the disassembly, [G] guessed.

## 1. The original's MIDI input

- **One input port, receive only** (manual p. 134). The input is a `CReBirthMIDIInputParser`
  (vtable 0x4BA624) opened on the port the preferences name (`sub_4092C0`; the port list
  `sub_4A2432`, matched by name at start, 0x40E0E5) [C]. ReBirth sends no MIDI (p. 127).
- **The callback** `sub_408FD4(time_ms, status, d1, d2)` (the parser's slot 1) [C]:
  1. when Sync to MIDI Clock is on (`sub_409FD0`), passes every message to the DLL's sync handler
     `dword_4DA5D4` with `time * 1000` (0x40901B; task-184);
  2. when Remote MIDI Control is on (`sub_409855`), the channel matches the preferences' and the
     type is note off, note on or control change (0x409071..0x4090A6), it maps the message through
     the Quick Mapping table when Quick Mapping is chosen (`sub_409531`, `sub_410F74`; task-188),
     queues it in a ring of 512 16-byte records (`+0x538`, write index `+0x2538`, 0x409104), and
     stores the first data byte at `+0x532` (0x40913D; [G] the controller number the Quick
     Mapping dialog's Learn field shows, task-188), then calls `sub_47FE66` (not read).
- **The dispatch.** The playing song's window drains the ring at idle time, up to 100 messages
  per call (`sub_42AC5D` → `sub_42EAAF`; `docs/standard_midi_mapping.md` section 1) [C]. A MIDI
  change so takes effect like a mouse change: the panel's message, then the next engine call that
  drains the message queue (`docs/export_state.md` 2.3). Its time inside the stream is not kept.
- **The panel follows.** Every handler stores the value in its view and redraws (`sub_42590B`),
  so "the controls move on the screen when they receive MIDI messages" (p. 132) [C].
- **Recording.** The handlers send the panel's own messages, which call the DLL's record hook,
  so in song mode with record on a MIDI move is recorded like a mouse move (p. 133;
  `docs/initial_investigation.md` 5.3) [C: the messages; G: not observed].

## 2. The original's MIDI settings (Preferences, "MIDI" section; manual p. 128)

| Setting | Where | Default |
|---|---|---|
| MIDI Input (port) | name at `+0x18C` of the stored preferences; `sub_4092C0` | the first port that opens (0x40E16F..0x40E1C3) [C] |
| Sync to MIDI Clock | Options menu and the dialog; `sub_4094FA` | [G] off |
| Remote MIDI Control on/off | `+0x530` (`sub_40983C` / `sub_409855`) | [G] off (`sub_41C210` 0x41C293) |
| Channel 1..16 (stored 0..15) | `sub_409869` sets, `sub_409882` reads; the dialog's popup (0x41EBA9) | [G] channel 1 (0x41C2B8) |
| Standard / Quick Mapping | `+0x528`: 40 Standard, 41 Quick (`sub_40964B`, `sub_409545`) | Standard (0x41C29D) [G: the field shift below] |
| Quick Mapping table | `PERSISTENT_QUICK_MIDI_MAP` (vtable 0x4BA634): 20 controller numbers and 20 functions (`sub_410808`) | task-188 |
| Keyboard mode | Options menu (Program Synth / Select Patterns from Keyboard); `+0x4F8` | Select Patterns, mode 11 (0x41C272) [C] |

The defaults are the preference constructor's (`sub_41C210`); the loader (`sub_40DEF2`) reads
the MIDI fields 44 bytes lower in the stored layout than the constructor writes them (remote at
`+0x22C` against `+0x258`, mapping `+0x230` against `+0x25C`, channel `+0x270` against
`+0x29C`), while the keyboard mode is at `+0x214` in both; the two layouts were not reconciled, so
the defaults of the three shifted fields are [G]. On the test machine the MIDI section is greyed
(no port: `docs/test_machine.md` section 5).

## 3. What orb does (task-182)

- **The layer.** `crates/orb-player/src/midi.rs`: a MIDI byte parser (`Parser`: running status,
  real-time bytes inside messages, system exclusive skipped, stray data bytes dropped) and the
  Standard Mapping (`StandardMapping`), which turns timed MIDI bytes into timed `Action`s, each
  with the player's `Command` (`Action::command`) or a view change (the shown bank, the focus,
  the selected drum instrument, the keyboard mode). It does not know where the bytes come from:
  the VST3/CLAP plugin will feed it the host's events with their sample offsets, task-188 a MIDI
  port. Applying an action at the 64-frame block its time falls in is the caller's choice; the
  layer only keeps the time.
- **The state.** The toggles read a `State` (the mixer's switches, the effects' on switches, the
  sections' on switches and patterns, the transport) that the caller refreshes from the player's
  snapshot (`StandardMapping::follow`); the layer updates it with each action it gives, so two
  toggles in a row work before the engine has run.
- **The panel.** `orb_ui::link::Link::send` sends the commands as the panel's own, and the panel
  shows the new values from the snapshot (`Link::follow`), as for a mouse change
  (`crates/orb-ui/tests/midi.rs`).
- **Settings.** One channel (`StandardMapping::channel`, default channel 1), the keyboard mode
  (`StandardMapping::mode`, default Select Patterns); no menu item, setting, port, LED or Quick
  Mapping (task-188). No Remote MIDI Control on/off: the original's is off by default [G]
  (section 2), orb's layer acts whenever a source feeds it (a deliberate difference; the plugin
  and task-188 decide when to feed it). The layer's default channel is 1; the plugin
  (`docs/plugin.md` section 8) feeds it only when its own Standard Mapping setting is on (off by
  default) and sets the channel to 16 by default, since channels 1..4 are the plugin's device
  channels (303 #1, 303 #2, 808, 909).
- **Not mapped yet.** The Program Synth notes (step programming;
  `docs/standard_midi_mapping.md` section 7): task-189.
- **Differences from the original**: `docs/standard_midi_mapping.md` section 10.

## 4. Live play: notes that sound a device (task-183)

**The original plays no device from MIDI notes**: not the two 303s, not the 808, not the 909.
A note on the remote channel is always a panel function of the Standard Mapping (pattern
selection, step programming, switches; `docs/standard_midi_mapping.md` sections 4..7), and a
note on another channel is dropped (section 1). The manual says so: "you can not play the sounds
in ReBirth RB-338 via MIDI. It would feel just awful." ("About Latency", printed page 183). The
DLL facts below were read in a read-only headless copy of `ReBirth.dll.i64` (2026-11-15); the
exe's note dispatch is task-182's reading of `Rebirth.exe.i64` (`docs/standard_midi_mapping.md`),
not re-read here.

### 4.1 The evidence

- **The 303's voice has one caller** [C]. Its note-on `0x10023CD4` is called only by the step
  function `0x100253A0` (at 0x10025589), its note-off `0x10023C74` only by the step function
  (0x10025442, 0x10025499, 0x1002556B) and by slot 6 (0x1002563D, stop). Nothing outside the
  sequencer starts or ends a 303 note.
- **The exe's MIDI input** sends notes to `sub_42EAAF` only (section 1), whose every case is a
  Standard Mapping function (task-182; `docs/standard_midi_mapping.md`). None reaches a voice.
- **The drums have no MIDI handler** [C]. The device vtable's slot 12 (offset 0x30, the slot of
  `sub_100261F0`, 4.2) is `nullsub_4` (0x10010380, `retn 0Ch`) in every device class but the
  303: 24 vtable references, among them the 808's at 0x10034184 and the 909's at 0x100342C4.
- **The only MIDI status test in the DLL** is in `sub_100261F0` (0x1002621C `cmp edx, 90h`,
  0x10026228 `cmp edx, 0B0h`); a search of `.text` for compares with 0x80, 0x90, 0xB0 and
  0xF0..0xFF finds no other MIDI one.

### 4.2 `sub_100261F0`: a copy of the 303's remote control, apparently unreachable

`docs/initial_investigation.md` 5.3 called `sub_100261F0` "the 303's MIDI input". It is slot 12
of the `CTB303Device` vtable (base 0x10033FD4, the entry at 0x10034004, between setParam
0x10026180 and the panel update 0x10026700), `(msg*, on, programMode)`, `retn 0Ch`; it reads the
status at `msg+8`, the data bytes at `+0xC` and `+0xD`. It plays nothing: it is the 303's part of
the Standard Mapping, done on the device's own bytes (`dev` = `sub_10010350`, 1 or 2):

- **0xB0**: CC `7*dev + 16 + k` (23..29, 30..36): k = 0 the waveform, `v >= 64` (0x100262B9);
  k = 1..6 tune, cutoff, reso, env mod, decay, accent, raw. Each stores the live byte
  (`+0xC7E..+0xC84`), sets its changed flag (`+0xC88` + controller) and calls the record hook
  `sub_10012BC0(dev, ctrl, v)`. The same as the exe's `sub_4405B5`.
- **0x90, programMode 0**: note `13*dev - 1 + k` (12..24, 25..37): k = 0 toggles the enable
  `+0xC7C` (`setz`, 0x100264DD) and records controller 0; k = 1..4 set the shown bank `+0xC86`;
  k = 5..12 set the pattern `8*bank + k - 5` (`+0xC7D`, 0x1002654B) and record controller 1. The
  `on` argument is not tested here.
- **0x90, programMode set, on**: edits the step under the cursor `+0xC87` of the live pattern
  (`+0x3C + 98*pattern + 2 + 6*step`): notes 12..24 set the pitch `n - 12` (0x100265D3); 25 toggles
  octave down, 26 octave up, 27 accent, 28 slide, 29 the rest flag; 30 moves the cursor back
  (0 wraps to 15, 0x1002669E), 31 forward (15 wraps to 0, 0x100266B8); 32 does nothing (the jump
  table 0x10153D38 sends it to the default).
- **0x80** is never tested: a note-off does nothing. No velocity byte is read.

What it does (the list above) is [C], read in the disassembly.

**It seems to have no caller** [G]. What was checked: IDA's cross-references to 0x100261F0 are
only the vtable entry (data, 0x10034004); the DLL's virtual calls load the slot into a register
and call it (`mov reg, [reg+N]` / `call reg`), and none of the 57 `+30h]` operands in `.text` is
such a load of slot 12. Not ruled out: a dispatch that reaches the slot by an indexed, `add` or
`lea` address. Whether it is called or not, it plays nothing, and the "no live play" conclusion
rests on the voice's callers (4.1), which are [C]. In 2.01 the exe does all of this itself
(`docs/standard_midi_mapping.md`). [G] A leftover of an earlier DLL-side remote control: its note
31 steps forward, as the manual's "G0 31 Step" (p. 202), where the exe's presses Back (OB-25).

### 4.3 What orb does

orb follows the original: its MIDI layer (`crates/orb-player/src/midi.rs`) gives no action that
sounds a device, on any channel, in either keyboard mode (tested there). Live play as the plugin
would use it (pitch = note, accent = velocity, slide = overlapping notes, the conventions of the
303 clones) is not in the original, so adding it is a decision: task-194 (Decide), implemented
by task-195 if chosen. `sub_100261F0` is not modelled: no caller of it was found ([G], 4.2), and the Standard
Mapping already does what it does (task-182; its step editing is task-189's, following the
exe).

**Differences from the manual**: none on this point (p. 183 and the binary agree).

## 5. The plugin's note mapping (task-185, task-199)

orb's own mapping of MIDI notes for the VST3/CLAP plugin (`docs/plugin.md`), decided in task-185
(2026-11-15/16; channels changed on 2026-11-16, task-207). It is not the original's: the
original plays no device from notes (section 4), and its Standard Mapping
(`docs/standard_midi_mapping.md`) stays as it is, on a channel of its own. Three features use
this mapping: pattern mode's pattern selection (task-198), note mode's live play (task-200) and
the pattern drag-out (task-203). The engine facts below carry the DLL's addresses; [C] means read
in the disassembly of a read-only headless copy of `ReBirth.dll.i64` (2026-11-16), and the engine
code named matches the corpus byte for byte.

### 5.1 Channels

| MIDI channel (1-based) | Device |
|---|---|
| 1 | 303 #1 |
| 2 | 303 #2 |
| 3 | TR-808 |
| 4 | TR-909 |

The same channels serve both modes and the drag-out. Channels 5..16 carry no device. The
Standard Mapping is off by default in the plugin; turned on, it listens on one channel outside
1..4, 16 by default, and a device channel is refused for it (task-207, which replaced the
earlier 2..5 layout; `docs/plugin.md` 5 and 8.2). A keyboard on its default channel 1 plays
303 #1 without setup.

### 5.2 Pattern mode (orb playing): pattern selection

Notes 0..31 on a device's channel select that device's pattern n + 1, i.e. banks A-D x patterns
1..8 (note 0 = A1, 7 = A8, 8 = B1, 31 = D8). Latched: the pattern changes at once and stays until
the next note on that channel (or a panel change; the last write wins). A note-off, a velocity-0
note-on and notes 32..127 do nothing. `docs/plugin.md` 8.2 has how it reaches the engine.

### 5.3 The 303 in note mode (orb stopped)

**Pitch = note.** The MIDI note of ReBirth's C at the Tune knob's center is **36** (C2 in the
convention where middle C, 60, is C4), and ReBirth's tuning is **concert pitch**: A4 = 440 Hz,
equal temperament, exact to the f32 rounding of the pitch table (about 1e-4 cent). The
derivation [C]:

- The note-on 0x10023CD4 sets the pitch target at 0x10023E61..0x10023EBF:
  `target = ((tuneTab[tune >> 2] << 6) + ((i16)(note + 12*up - 12*down) << 6) + 0xC00) << 4`,
  `tuneTab` = `word_10033F90` (bytes read: -12, -12, -11..-1, 0 x6, 1..12, 12), tune the live
  byte `+0xC7E`. Without a slide the gliding pitch `+0xCB4` takes the target at once.
- The audio (2.0 sound `sub_100248C4`, 0x100248EB..0x1002499E; Vintage 0x10023F9F) glides
  `p = (11*p + target) / 12` (`lea`, `add`, unsigned `div` by 12) and loads the phase increment
  `flt_10125364[p >> 4]`: index `i = 64*(tuneTab + semitones) + 3072`, 64 steps a semitone. No
  bound is checked.
- The pitch table (6144 f32, ctor `sub_10023920`; `docs/initial_investigation.md` 6.1) is
  `tab[i] = f32(2^(i/768) * k)` with `k = (2^(3/12)*110/44100) / 2^4`. The oscillator's phase
  runs from -1 to 1 and wraps once per sample (`crates/orb-engine/src/tb303.rs`, `Voice::sample`),
  so the frequency is `tab[i] * 22050` = `65.406 Hz * 2^((i - 3072)/768)`. Index 3072 is
  `55 * 2^(3/12)` = 65.406 Hz, the C two octaves below middle C when A is 440 Hz.
- Pattern note 0 is C (the 303's pitch keys C..C' are notes 0..12, `docs/ui_spec.md`), and the
  Tune knob gives 0 semitones over 52..75 (`tune >> 2` = 13..18), so its center 64 is untransposed.

So note mode plays MIDI note `m` as `semitones = m - 36`, with the Tune knob's semitones added as
the original adds them; `target = (64*(tuneTab[tune >> 2] + m - 36) + 3072) * 16`. The tune is
read at the note-on only (0x10023E88), as in the original: turning the knob does not bend a held
note.

**Range.** The table covers `tuneTab + m - 36` from -48 to 47 (index 0..6080 of 0..6143); a
higher or lower note reads past the table (the next bytes are the filter table at 0x1012B364),
so note mode must not play it.

| | MIDI notes | Frequencies |
|---|---|---|
| The table, Tune at center | 0..83 (B5) | 8.18 Hz .. 987.8 Hz (index 768..6080) |
| The table, any Tune setting | 0..71 always; up to 95 with Tune at -12 | |
| A pattern step (notes 0..12, Up, Down), Tune at center | 24..60 | 32.7 .. 261.6 Hz |
| A pattern step, Tune -12..+12 | 12..72 | 16.4 .. 523.3 Hz |

Recommended note-mode range: every MIDI note whose index is in the table (`-48 <= tuneTab + m -
36 <= 47`); other notes are ignored. All of it is computed sensibly: the saw and the square are
not band-limited, so even the subsonic fundamentals below MIDI 12 (16 Hz) sound, as a click
train; the DC blocker (`o = (y - y1) + o*0.985853374` per sample, `flt_10153BD0`) is a one-pole
high-pass at about 100 Hz, so fundamentals below about MIDI 43 are already weaker in the
original's own range. The top note, 988 Hz, aliases little (its increment is 0.045 phase units a
sample, 0.022 of the 2-wide span).

**Glide.** A slide rising in pitch ends up to 11 units of `p` short of the target: `(11p + t)/12`
truncates to `p` once `t - p < 12`, so the index stays one below (1/64 semitone, 1.56 cents flat)
until the next note. A falling slide reaches the target exactly. Note mode inherits this by using
the same glide ([Q-5](original_bugs.md)).

**Velocity, overlap.** Accent = velocity >= 100 (a fixed threshold; a setting later). A note
that overlaps the held one slides to it (no new attack; the glide above); an overlapping note of
the same pitch is a tie; last-note priority, legato. Velocity changes nothing else.

### 5.4 The drums (General MIDI notes)

| Note | 808 | 909 |
|---|---|---|
| 36 | BD | BD |
| 37 | RS (the rim shot of the RS/CL slot) | RS |
| 38 | SD | SD |
| 39 | CP (the clap of the CP/MA slot) | CP |
| 42 | CH | CH |
| 46 | OH | OH |
| 43, 47, 50 | LT, MT, HT (tom of the LT/LC, MT/MC, HT/HC slot) | LT, MT, HT |
| 64, 63, 62 | the 808's low, mid, high conga (LC, MC, HC; orb's labels: GM names 64 low conga, 63 open high conga, 62 mute high conga) | - |
| 75 | claves (CL) | - |
| 70 | maracas (MA) | - |
| 56 | cowbell (CB) | - |
| 49 | cymbal (CY) | crash (CC) |
| 51 | - | ride (RC) |

A note on a shared 808 slot plays the voice it names for that hit only; the slot's switch
parameter (controllers 11, 14, 17, 19, 21) is not changed and governs pattern mode. The slot has
one voice, so its two sounds cannot sound at once. Notes not in the table do nothing. A note-off
does nothing (the drum voices run their own decay).

**909 flam: no note.** Two notes in quick succession play as two hits.

### 5.5 Accent and velocity

The engine facts, the same in `crates/orb-engine/src/tr808.rs` and `tr909.rs`; the accent and
gain formulas and the 909 snare are [C] in the disassembly, which instruments the 808's accent
scales is from the engine code:

- **808** (step 0x10027821..0x10027848): `accent = f32((AC*knob) * 0.03125f + 1.0)` at
  `+0x190C` (`flt_10153F48`, `dbl_10153F40`), AC the step's 0/1 AC byte, knob the Accent knob
  (live byte 2, default 40). Every instrument's level is multiplied by it (`level`, `snare` g1,
  `tom`); the snare's snappy level g2 is not.
- **909** (step 0x1002ACAF..0x1002ACD1): the same accent at `+0x1918` (`flt_10154368` = 0.03125,
  `dbl_10154348` = 1.0), with the AC byte the panel writes: **2**, not 1 (`CTR909PatternView`'s
  AC row stores 2, `docs/ui_spec.md`; the shipped songs hold 0 or 2), so the factor is
  `1 + knob/16`. Then each instrument's gain `+0x191C` is `f32(accent * 0.8)` for step value 1
  or 3 (0x1002B011, `dbl_10154358` = 0x3FE999999999999A) and `f32(accent * 2.0)` for 2 or 4
  (0x1002AD02, `dbl_10154360`), and multiplies the instrument's level. The snare's snappy level
  is not scaled by it (`sub_1002DC54`: g2 = `snappy * flt_10154648` at 0x1002DC77, g1 times
  `+0x191C` at 0x1002DCDB). Knob default 32.

**808 in note mode:** velocity 1..99 = normal (x1), 100..127 = AC (x`(1 + knob/32)`, AC = 1 as
the 808's row). AC is louder for any knob above 0 and equal at 0.

**909 in note mode:** four bands, the hit's gain relative to the level (k = the Accent knob,
AC = 2 as the panel's row):

| Velocity | Step | Gain | k = 0 | k = 16 | k = 24 | k = 32 (default) | k = 127 |
|---|---|---|---|---|---|---|---|
| 1..47 | low (value 1) | 0.8 | 0.8 | 0.8 | 0.8 | 0.8 | 0.8 |
| 48..95 | high (value 2) | 2.0 | 2.0 | 2.0 | 2.0 | 2.0 | 2.0 |
| 96..111 | low + AC | 0.8 (1 + k/16) | 0.8 | 1.6 | 2.0 | 2.4 | 7.15 |
| 112..127 | high + AC | 2.0 (1 + k/16) | 2.0 | 4.0 | 5.0 | 6.0 | 17.875 |

**The loudness order.** Low < high and low + AC < high + AC always; low + AC is at least low
and high + AC at least high (equal at k = 0). The bands rise with velocity (low < high < low +
AC < high + AC) only when `0.8 (1 + k/16) > 2.0`, i.e. **from Accent knob 25 up** (at 24 low +
AC equals high: `f32(2.5 * 0.8)` = 2.0); the default 32 is inside it. Below 25 the 96..111 band is
quieter than the 48..95 band. (task-185 wrote "from 16 up", which holds for neither AC value;
with AC = 1, as an 808-style row or a hand-made file could hold, the order would hold from 49 up.
The band edges are task-185's, to be tuned by ear.)

### 5.6 Export velocities (the pattern drag-out, task-203)

The clip writes fixed velocities that fall in the bands above, so it plays back in note mode as
the pattern sounds:

| Device | Case | Velocity |
|---|---|---|
| 303 | normal / accent | 80 / 127 |
| 808 | step without / with AC (every instrument of the step) | 80 / 127 |
| 909 | value 1 (low), value 2 (high) | 40, 80 |
| 909 | value 1 + AC, value 2 + AC | 104, 127 |

80 stays below any accent threshold above 80 and 127 above any threshold, should the threshold
become a setting. A 909 AC byte other than 0 counts as AC (a value of 1 from a hand-made file
plays back as 2: louder). The 909's flam pass retriggers every step value above 2 at accent x1.5
(0x1002B320; `docs/initial_investigation.md` 5, 7.2), and a flam has no note (5.4), so the flam
is lost in the clip: a value 3 (low + flam) is exported as value 1 (40 or 104), a value 4 (high
+ flam) as value 2 (80 or 127), and a value 5..255 (a hand-made file: no step hit, the flam hit
alone at x1.5) as value 2 (80 or 127), the band whose gain (x2.0) is nearest, at the step's start.
The AC row writes no note. The 808's shared slots (LT/LC, MT/MC, HT/HC, RS/CL, CP/MA) write the
note of the voice the slot's switch selects when the clip is made (the LT/LC slot: 43 when it
says tom, 64 when conga; RS/CL: 37 or 75; CP/MA: 39 or 70), as the pattern plays it. The 303's note is `36 + note + 12*up - 12*down` (Tune is
applied at playback, as in the pattern), a rest step writes no note, and a sliding step's note
overlaps the next one (the same pitch: a tie).
