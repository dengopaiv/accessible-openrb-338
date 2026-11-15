# MIDI in ReBirth 2.01 and in orb

MIDI came into scope on 2026-11-13 (milestone m-7). This is the general MIDI document: the
original's MIDI input, its settings, and what orb does with them. Tasks extend it: task-182 (this
document and the Standard Mapping), task-183 (notes played live), task-184 (MIDI clock), task-188
(the standalone app's ports, Quick Mapping, the MIDI LED, recording by MIDI).

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
