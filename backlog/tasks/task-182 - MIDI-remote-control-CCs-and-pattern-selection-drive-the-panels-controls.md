---
id: TASK-182
title: >-
  MIDI Standard Mapping: a source-independent layer from MIDI to the panel's
  controls
status: Done
assignee:
  - '@agent-complex'
created_date: '2026-11-13T18:19:00Z'
updated_date: '2026-11-15T12:44:00Z'
labels:
  - re
  - player
  - docs
milestone: m-7
dependencies: []
references:
  - docs/initial_investigation.md
  - docs/song_load.md
  - docs/ui_spec.md
  - crates/orb-player/src/lib.rs
  - /Volumes/downloads/rebirth_2.01_manual.pdf
  - task-185
  - task-188
priority: medium
type: feature
ordinal: 198000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
The user decided (2026-11-13) to bring MIDI into scope, until now excluded by AGENTS.md ("MIDI and ReWire are out of scope for now"), in the new milestone m-7, which also holds the VST3/CLAP plugin (task-185). This task is the core of MIDI remote control: MIDI messages do what the mouse and keyboard do on the panel (turn knobs, flip switches, select patterns, switch devices on and off).

Why the core comes first (the user, 2026-11-15): MIDI input will first be used inside the plugin, as the way songs and patterns exported to the host as MIDI play back through it. The exported files use the mapping this task implements. Opening MIDI ports in the standalone app comes later, in task-188.

The original manual (`/Volumes/downloads/rebirth_2.01_manual.pdf`, or the copy on the CD) describes two MIDI mappings:
- **Standard Mapping**, predefined and unchangeable. This task implements it, and only it.
- **Quick Mapping**, a limited mapping the user changes in the UI. That is task-188's, not this task's.

What is known: the parameters `docs/song_load.md` marks as probably MIDI remote control (around line 413); the 303 waveform's CCs 23 and 30 (`sub_4405B5`, stores `v >= 64`, docs/ui_spec.md section 2); the controller table of the ReBirth 1.x MIDI-file import (`docs/song_load.md` section 12). In the DLL every panel controller message calls the record hook `sub_10012BC0` after its store (`docs/initial_investigation.md` section 5.3).

Reverse engineering is deliberately minimal: confirm the Standard Mapping (which channel addresses which device, which CC drives which controller and how its value is scaled, how patterns are selected, what other messages it reacts to), from the manual and the exe, with addresses checked against the disassembly. If the Standard Mapping depends on a setting of the original (a base channel, for example), use the original's default and add no UI for it; the setting belongs to task-188. MIDI-driven output does NOT have to match the original sample for sample; it should behave as a user of the original would expect. Write down any choice where the original is unclear.

The layer takes timed MIDI bytes and produces the player's commands, independent of where the bytes come from: the plugin will feed it the host's events with sample offsets, and task-188 will feed it a MIDI port. This task has no live MIDI source yet: the tests drive it with byte streams. No new UI: no menu items, no settings, no port handling, no MIDI LED. The user's IDA databases are read only.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 A new docs/midi.md documents the original's Standard Mapping (channels per device, the CC-to-controller map with value scaling, pattern selection, the other messages it handles, and any setting it depends on with its default), each fact marked [C] with addresses or [G]; it names Quick Mapping as task-188's and lists where orb deliberately differs
- [x] #2 The MIDI-to-command layer is independent of the MIDI source, and cargo tests drive it with timed byte streams (no MIDI port) covering every mapped controller kind, pattern selection, channel filtering and out-of-range or unmapped messages
- [x] #3 A control changed through the layer shows its new value on the panel, as a mouse change does, checked by a test in crates/orb-ui
- [x] #4 No menu item, setting, MIDI port handling or MIDI LED behavior is added to the app
- [x] #5 AGENTS.md's scope sentence is updated to say MIDI is in scope in m-7
- [x] #6 A new docs/standard_midi_mapping.md describes the whole Standard Mapping (channels per device, every CC with its controller and value scaling, pattern selection, the other messages), each entry checked against the manual's MIDI chapter, with a section "Differences from the manual" (the manual's statement and page, the binary's behavior with addresses, orb's behavior) or "none"; docs/midi.md points to it instead of repeating it; the final summary repeats the list (added 2026-11-15 at the user's request)
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. RE (minimal, exe copy in the scratchpad, read only): the MIDI input callback (CReBirthMIDIInputParser), the queue, the idle dispatcher sub_42EAAF and every handler it calls; the Preferences fields (remote on, channel, Standard/Quick) and their defaults. Check each entry against the manual (Remote MIDI Control, pp. 127-134; Appendix C, pp. 195-204).
2. docs/standard_midi_mapping.md: the whole mapping with addresses and [C]/[G], plus "Differences from the manual". docs/midi.md: the general MIDI document (input path, settings, what orb does, Quick Mapping is task-188's), pointing to the mapping doc. docs/original_bugs.md for the defects found. AGENTS.md scope sentence.
3. orb-player: new module midi (source independent): a byte parser (running status, realtime bytes inside messages, sysex skipped) taking (time, bytes) and a StandardMapping that turns messages on its channel (default 1) into timed actions: player Commands (DeviceParam, MixerGlobal, MixerChannel, Shuffle, Play/Stop/Record/Locate) plus view actions (bank shown, focus, instrument, keyboard mode). It keeps a mirror of the state toggles need, refreshed from the Snapshot. No UI, no ports, no LED.
4. Program-Synth notes (step programming) documented, not mapped: follow-up task.
5. Tests: orb-player unit tests drive the layer with byte streams (every kind, pattern selection, channel filter, out-of-range, unmapped, running status); an orb-ui test sends the layer's commands through Link to a real player and checks the panel shows the value.
6. cargo test, clippy, nightly fmt; rebase; one commit.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: worktree-agent-aa457b5b1de55493e

RE (2026-11-15, read-only headless copy of Rebirth.exe.i64 in the scratchpad, closed save=false; IDA disassembly, Hex-Rays as a second opinion):
- Input callback sub_408FD4 (CReBirthMIDIInputParser vtable 0x4BA624 slot 1): sync messages to the DLL first (0x40901B); remote control only if +0x530 on, channel == sub_409882, type 0x80/0x90/0xB0 (0x409071..0x4090A6); Quick Mapping translate sub_410F74 when sub_409531; ring of 512 x 16 bytes (0x409104).
- Idle drain sub_42AC5D: up to 100 per idle, playing window only -> dispatcher sub_42EAAF. Notes: on = status==0x90 (setz 0x42EAFC), velocity never read -> OB-24. 64 toggles mode 10/11, 95 -> 10, 96 -> 11; mode 10 and note<64 -> step programming (sub_4402AB / sub_4435D9 / sub_447680); else 12..63 pattern selection sub_41AE84 per row view (+0x1C4/+0x354/+0x4E4/+0x674); 65..68 focus sub_42A89E; 69..73 transport sub_429BEC; 74..77 effect enables; 78..94 mixer switches sub_413256. CCs: 7..22 sub_4137E2, 23..36 sub_4405B5, 37..64 sub_443C92, 65..93 sub_447D9D, 94..99 sub_41BE94, 100..103 sub_40827C, 104..105 sub_408C98, 106..107 sub_404798, 108 sub_429CF7.
- Scalings: switches v>=64 (303 wave, 808 selectors 11/14/17/19/21, PCF mode, delay triplet), instrument 12*v/128, PCF pattern clamp(v,0,54) (push 36h at 0x41BEC5), delay steps 32*v/D+1 with a copy-protected D (intended 128).
- Copy-protection arithmetic in CC 16, CC 21 (mixer), CC 100/101 (delay), 909 program notes 34/36: orb takes the intended values [G].
- Manual vs binary: notes 70/71 numbers swapped in the manual (70 Stop, 71 Record in the binary); 72/73 move 10 bars (sub_429DF2(+-7680)), song mode only (+0x2C = song mode, set at 0x42A48C); program-mode notes 30 and 31 both press Back (+0x2B0; jump table 0x440561 cases 30, 31 -> 0x44050C, 0x440527; Backspace = key code 30 by sub_492312, Enter 44 -> +0x280 Step in sub_43F673) -> OB-25; 909 note 40 toggles +0x6BC, not in the manual.
- Pattern on/off note: CToggleBitmapButton (vtable 0x4BBC0C) slot +0x60 sub_4250D8 toggles on note-on.
- Settings: Standard 40 / Quick 41 (sub_40964B), defaults from sub_41C210 (mode 11 at 0x41C272; remote off, Standard, channel byte 0 under a 44-byte layout shift [G]).
Failed hypothesis: that the manual's Various Switches table matched the binary note for note (it does not: 70/71, 72/73).
Design: orb-player::midi (Parser + StandardMapping -> Timed<Action>, Action::command() -> player Command; a State mirror refreshed from the Snapshot for toggles); orb-ui Link::send for commands not from the panel. Program Synth notes not mapped: follow-up task-189 created.
Docs: docs/standard_midi_mapping.md (new), docs/midi.md (new), docs/original_bugs.md OB-24, OB-25, docs/song_load.md 4.10 [G] -> [MIDI] handlers, docs/initial_investigation.md scope line, AGENTS.md scope sentence.
Reproduce: cargo test -p orb-player --lib midi (16 tests); cargo test -p orb-ui --test midi (2 tests).

Tests: cargo test (whole workspace, debug) 714 passed, 9 ignored, 0 failed; cargo clippy --all-targets -D warnings clean; cargo +nightly fmt --all --check clean. No engine change (orb-engine untouched; orb-player gains a module, orb-ui a Link::send), so no corpus run.
AC evidence:
#1 docs/midi.md (input path, settings with defaults [C]/[G], what orb does) + docs/standard_midi_mapping.md (the mapping; Quick Mapping named as task-188's; orb's differences in section 10).
#2 crates/orb-player/src/midi.rs: no MIDI port; 16 unit tests with byte streams: every CC kind (mixer, 303 incl. waveform switch, 808 incl. switches and instrument select, 909 incl. permuted 83..85, PCF clamp, delay steps scaling, triplet, dist, comp, shuffle), pattern selection (banks, patterns, on/off, bank following the song), channel filtering, unmapped CCs/notes/messages, running status, real-time inside a message, sysex, split messages, times kept.
#3 crates/orb-ui/tests/midi.rs: bytes -> StandardMapping -> Link::send -> real Player -> Link::sync: tb303_1.cutoff, tr808.slot3_switch, tr909.hh_level, mixer2.level, delay.steps and delay.on show their new values; pattern selection shows rows[2].pattern 19 and bank C; the 909 on/off toggles.
#4 git diff: no change in menu.rs, settings.rs, app.rs; no port code, no LED; only Link::send (a method) in orb-ui src.
#5 AGENTS.md line 5: MIDI in scope in m-7.
#6 docs/standard_midi_mapping.md section 9 lists 9 differences from the manual with pages and addresses; docs/midi.md points to it.

Review round 1 (send back), fixed:
1. Section 10 and the final summary no longer claim intended values for the 909 program notes 34/36 (not mapped yet, task-189).
2. The always-on layer (the original's Remote MIDI Control is off by default [G], 0x41C293 under the unreconciled layout shift) is listed as a deliberate difference in docs/standard_midi_mapping.md section 10, docs/midi.md section 3 and the final summary; no enable flag added.
3. Section 9 items 7-9 now carry the binary's addresses, rechecked in the disassembly this round (IDA headless copy, closed save=false): cmp ..,40h at 0x4405F0, 0x443E7D, 0x443F1B, 0x443FB9, 0x444027, 0x444094, 0x41BF17, 0x40833A; imul eax,0Ch / sar eax,7 at 0x444216..0x44421F and 0x44830D..0x448316; delay steps 0x4082DA..0x408301; note-on flag 0x42EAF5..0x42EAFC; its tests at 0x42EB1A, 0x42EC7B..0x42ECCE (the handlers' if(on) tests are named as read in Hex-Rays only). Also rechecked: 0x41C293 mov byte [eax+258h],0; 0x41C29D mov dword [ecx+25Ch],28h; 0x41C2B8 mov byte [edx+29Ch],0; 0x41C272 mov dword [eax+214h],0Bh.
4. The final summary's deliberate list now includes the focus note (msg 35 {22,row} not sent), matching section 10.
AC #1 wording: the CC-to-controller map that AC #1 asks of docs/midi.md lives, at the user's request (AC #6), in docs/standard_midi_mapping.md; docs/midi.md points to it.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
The original's MIDI Standard Mapping is reverse engineered (Rebirth.exe: input callback sub_408FD4, idle drain sub_42AC5D, dispatcher sub_42EAAF and its 20 handlers) and implemented as a source-independent layer, crates/orb-player/src/midi.rs: a MIDI byte parser and StandardMapping, which turns timed bytes on one channel (default channel 1) into timed Actions, each with the player's Command (DeviceParam, MixerGlobal, MixerChannel, Shuffle, Play, Stop, Record, Locate) or a view change (bank, focus, drum instrument, keyboard mode). Toggles read a State mirror refreshed from the player's Snapshot. orb-ui gains Link::send, and a test shows MIDI-driven values on the panel. No UI, ports, settings or LED were added.

Documents: docs/standard_midi_mapping.md (the whole mapping, checked entry by entry against the manual's pp. 127-134 and 195-204), docs/midi.md (the general MIDI document, pointing to the mapping), docs/original_bugs.md OB-24 and OB-25, docs/song_load.md 4.10 (the [G] senders are now the [C] MIDI handlers), docs/ui_spec.md and docs/initial_investigation.md scope notes, and AGENTS.md's scope sentence.

Differences from the manual (orb follows the binary):
1. p. 199: the manual numbers Record 70 and Stop 71; the binary has Stop 70 and Record 71 (sub_429BEC, 0x429C46, 0x429C62). The manual's key names A#3 and B3 agree with the binary.
2. p. 199: notes 72/73 'Bar -/+' move 10 bars, from the start of the bar (sub_429DF2(-+7680)).
3. p. 199 says 'all modes', but Record (71) and the bar notes need song mode (transport +0x2C).
4. p. 202: Program Synth notes 30 (Back) and 31 (Step) both press Back (jump table 0x440561; OB-25).
5. p. 204: 909 note 40 toggles +0x6BC (probably Flam), which is not in the manual.
6. p. 206 says 34 PCF patterns; CC 94 is clamped to 0..54 (0x41BEC5).
7. pp. 195-198 give no scalings: switches are v>=64 (0x4405F0, 0x443E7D.., 0x41BF17, 0x40833A), instrument selection is 12*v/128 (0x444216, 0x44830D), delay steps 32*v/D+1 (0x4082DA..0x408301).
8. A velocity-0 note-on acts as a note-on (0x42EAF5..0x42EAFC; OB-24).
9. Only the note-on acts (the on flag of 0x42EAFC is tested at 0x42EB1A, 0x42EC7B.. and in the handlers).

Where orb deliberately differs from the binary:
- A velocity-0 note-on is a note-off.
- The copy-protected values (CC 16, 21, 100, 101) take their intended values [G].
- orb's layer has no Remote MIDI Control on/off: it acts whenever a source feeds it. The original's preference is off by default [G].
- orb keeps each event's time; the caller decides when to apply it.
- The focus notes (65..68) change orb's view focus only; the original's song msg 35 {22, row} is not sent.
- The Program Synth notes (including the 909's notes 34 and 36) are not mapped; follow-up task-189 was created.

Branch worktree-agent-aa457b5b1de55493e, one commit (amended after review round 1). Tests: cargo test 714 passed, 9 ignored (orb-player midi 16 unit tests, orb-ui tests/midi.rs 2), clippy and nightly fmt clean. The engine is unchanged, so no corpus run.
<!-- SECTION:FINAL_SUMMARY:END -->
