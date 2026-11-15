# MIDI in ReBirth 2.01 and in orb

MIDI came into scope on 2026-11-13 (milestone m-7). This is the general MIDI document: the
original's MIDI input, its settings, and what orb does with them. Tasks extend it: task-182 (this
document and the Standard Mapping), task-183 (notes played live: the original has none, section
4), task-184 (MIDI clock), task-188 (the standalone app's ports, Quick Mapping, the MIDI LED,
recording by MIDI).

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
  and task-188 decide when to feed it).
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
