---
id: TASK-199
title: >-
  Docs: the plugin's note mapping (device channels, 303 key range, GM drum
  notes, velocity bands)
status: Done
assignee:
  - '@agent-complex'
created_date: '2026-11-16T07:47:00Z'
updated_date: '2026-11-16T10:44:00Z'
labels:
  - docs
  - re
milestone: m-7
dependencies:
  - TASK-185
  - TASK-207
references:
  - backlog task-185 (final summary)
  - docs/midi.md
  - docs/standard_midi_mapping.md
  - docs/initial_investigation.md
  - crates/orb-engine/src/tb303.rs
  - crates/orb-engine/src/tr808.rs
  - crates/orb-engine/src/tr909.rs
priority: medium
type: docs
ordinal: 212000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
task-185 defines orb's own note mapping for the plugin, separate from the original's Standard Mapping (which stays as it is on its own channel, off by default in the plugin: task-207). It is used by note mode (live play), pattern mode's pattern selection and the pattern drag-out. Write it down as one reference, in a new section of docs/midi.md, with what is still to be found:

- **Channels:** 303 #1 = 1, 303 #2 = 2, 808 = 3, 909 = 4 (1-based; changed from 2..5 on 2026-11-16, task-207). The Standard Mapping, when turned on, uses a channel outside 1..4 (default 16).
- **Pattern mode:** notes 0..31 = patterns 1..32 (banks A-D x 8), latched.
- **303 in note mode:** pitch = note; find the MIDI note of ReBirth's C at the Tune knob's center (is ReBirth's tuning concert pitch? the 303's pitch computation, crates/orb-engine/src/tb303.rs and the DLL), and the range: every note the 303's oscillator computes sensibly and audibly (pattern steps reach about 3 octaves with up/down; live notes may go further if the pitch computation allows). Accent = velocity >= 100; overlap = slide; same-pitch overlap = tie; last-note priority, legato.
- **Drums (General MIDI):** BD 36, RS 37, SD 38, CP 39, CH 42, OH 46, LT/MT/HT 43/47/50, 808 congas L/M/H 64/63/62, claves 75, maracas 70, cowbell 56, 808 cymbal and 909 crash 49, 909 ride 51. A shared-slot note plays the voice it names, for that hit only (the switch parameter is untouched).
- **Accent:** 808: velocity < 100 normal, >= 100 AC. 909: step level low x0.8 / high x2.0 times the AC factor 1 + AC*knob/32 (crates/orb-engine/src/tr909.rs): bands 1-47 low, 48-95 high, 96-111 low+AC, 112-127 high+AC. Export velocities for each case (303/808 normal/accent, 909 e.g. 40/80/104/127). Check the 909 gain formula and the 808's against the code and state the loudness order and from which Accent knob value it holds.
- **909 flam:** no note.

Facts about the original found on the way go into their documents with addresses ([C]/[G]).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 docs/midi.md has the note mapping section with every item above
- [x] #2 The MIDI note of ReBirth's 303 C at Tune center and the 303 range are stated with the engine code or DLL addresses they come from
- [x] #3 The 808 and 909 accent gains and the band loudness order are stated from the engine code
- [x] #4 docs/standard_midi_mapping.md points to the new section and says the two mappings use different channels
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Read the 303 pitch path (note-on 0x10023CD4, glide 0x100248C4/0x10023F10, pitch table 0x10125364) in a read-only headless copy of ReBirth.dll.i64 and in crates/orb-engine/src/tb303.rs; derive the frequency of pitch index i, the MIDI note of C at Tune center, the table's reach and the pattern's reach.
2. Read the 808 (0x10027821) and 909 (0x1002ACAF, 0x1002AD02, 0x1002B011, snare 0x1002DC54) accent and gain code against tr808.rs/tr909.rs; compute the 909 band loudness order and the Accent knob value from which it holds (AC row value 2 from the panel, docs/ui_spec.md).
3. Write docs/midi.md section 5 (channels, pattern notes, 303 note mode with pitch/range, GM drums, accent bands, export velocities, flam); pointer in docs/standard_midi_mapping.md; [C] facts into docs/initial_investigation.md (6.1 pitch, 7.2 909) and the rising-glide quirk into docs/original_bugs.md.
4. Docs only: no cargo run needed; rebase, one commit.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: worktree-agent-a09e24d38a378c2c7

Evidence (read-only headless copy of ReBirth.dll.i64, closed save=false, copy deleted):
- 303 note-on 0x10023E61..0x10023EBF: target = ((word_10033F90[tune>>2] << 6) + ((i16)(note+12up-12down) << 6) + 0xC00) << 4; tune = live byte +0xC7E read at 0x10023E88. Tune table bytes read: -12,-12,-11..-1,0 x6,1..12,12.
- Glide: 2.0 0x100248EB..0x10024940 (lea 9p, lea +2p, add target, div 12), Vintage 0x10023F9F..0x10023FC8 the same; index load flt_10125364[p>>4] at 0x1002499E, no bound check (table 6144 f32 ends at 0x1012B364 = filter table).
- Frequency = tab[i]*22050 (phase -1..1, one wrap per sample, tb303.rs Voice::sample) = 65.406 Hz * 2^((i-3072)/768); i = 3072 at note 0 (C), Tune 52..75 -> 55*2^(3/12) = 65.406 Hz = C2 = MIDI 36, A4 = 440 concert pitch.
- Range: table covers tuneTab + m - 36 in -48..47: MIDI 0..83 at Tune center (8.18..987.8 Hz), 0..71 at any tune; a pattern reaches 24..60 at center, 12..72 over the Tune range. DC blocker 0.985853374 = one-pole HPF at 100.0 Hz.
- Rising glide stalls (t-p<12): index one below, 1.5625 cents flat; falling exact. Recorded as original_bugs Q-5 (quirk, not defect).
- 808 accent 0x10027821..0x10027848: f32((AC*knob)*0.03125f + 1.0) at +0x190C (flt_10153F48 bytes 00 00 00 3D).
- 909 accent 0x1002ACAF..0x1002ACD1 at +0x1918 (flt_10154368 = 0.03125, dbl_10154348 = 1.0); value 1/3 -> f32(acc*0.8) at 0x1002B011 (dbl_10154358 = 0x3FE999999999999A), 2/4 -> f32(acc*2.0) at 0x1002AD02 (dbl_10154360 = 2.0); dbl_10154350 = 1.5 (flam). Bytes read 0x10154348..0x1015436F.
- 909 snare sub_1002DC54: g2 = snappy*flt_10154648 (0x1002DC77) without the gain; g1 times +0x191C (0x1002DCDB).
- The panel's 909 AC row stores 2 (docs/ui_spec.md CTR909PatternView; orb-ui behavior.rs tr909_step_value), so the 909 accent is 1 + knob/16.
- Loudness order of the four 909 bands holds from Accent knob 25 up (0.8(1+k/16) > 2.0; equal at 24, f32(2.5*0.8) = 2.0). task-185's 'from 16 up' is wrong for AC = 2 (and for AC = 1, where it is 49). Gains table k=0/16/24/32/127 in docs/midi.md 5.5.
- Export velocities chosen: 303/808 80/127, 909 40/80/104/127 (80 below any threshold > 80, 127 above any).
Docs only; no cargo run (no code change).

Rebased onto main 778c208 (task-207 landed): the channel text of docs/midi.md 5.1 now points at docs/plugin.md 5 and 8.2 as done. git diff main --stat lists only the task file and the four docs.
AC evidence: #1 docs/midi.md section 5 (5.1 channels, 5.2 pattern notes, 5.3 303, 5.4 GM drums incl. shared slots and flam, 5.5 accent/bands, 5.6 export velocities). #2 docs/midi.md 5.3 (MIDI 36 = C at Tune center, concert pitch; range table) with 0x10023E61..0x10023EBF, 0x100248EB..0x1002499E, 0x10023F9F, word_10033F90, flt_10125364 and tb303.rs. #3 docs/midi.md 5.5 from tr808.rs/tr909.rs and 0x10027821, 0x1002ACAF, 0x1002AD02, 0x1002B011, 0x1002DC54: order holds from knob 25. #4 docs/standard_midi_mapping.md header paragraph 'The plugin's note mapping is a different one'.

Review round 1 (send back, 7 doc defects), fixed: (1) midi.md 5.6: the flam pass retriggers values > 2 at x1.5 (0x1002B320): 3 -> value 1, 4 -> value 2, 5..255 (flam hit alone) -> value 2 band, the flam lost in each; (2) 5.6: 808 shared slots export the note of the switch's current voice (43/64, 47/63, 50/62, 37/75, 39/70); (3) Q-5 Where: ecx = target (0x1002491D, added 0x10024929), 12 loaded at 0x1002492B, div at 0x10024936; (4) Q-5 Corpus marked [G], songs not counted; (5) 5.3: 0.045 phase units a sample = 0.022 of the 2-wide span; (6) initial_investigation 6.1: -12..+24 at Tune center, -24..+36 over the Tune range; (7) 5.4: congas labelled as orb's, GM names in brackets. Rebased onto main.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
docs/midi.md gains section 5, the plugin's note mapping: channels 1..4 (303 #1, 303 #2, 808, 909; the Standard Mapping off by default, on channel 16 when on), pattern notes 0..31 = A1..D8 latched, the 303 in note mode, the GM drum notes with the 808 shared slots and no 909 flam note, the 808/909 accent gains and velocity bands, and the drag-out's export velocities (303/808 80/127; 909 40/80/104/127). docs/standard_midi_mapping.md points to it and says the two mappings use different channels.

303 tuning [C]: ReBirth's C at the Tune knob's center (52..75 all give 0) is MIDI note 36, 65.406 Hz = 55*2^(3/12), concert pitch (A4 = 440 Hz, equal temperament): note-on 0x10023E61 index = 64*(tuneTab + semitones) + 3072, frequency = flt_10125364[i]*22050 = 65.406*2^((i-3072)/768). The pitch table covers -48..+47 semitones from that C (MIDI 0..83 at Tune center, 0..71 at any Tune; the pattern reaches 24..60, 12..72 over the Tune range); the DLL checks no bound (0x1002499E), so note mode must ignore notes outside. Recommended range: the whole table.

Accent [C]: 808 x(1 + AC*knob/32) with AC 0/1; 909 accent with the panel's AC = 2 is 1 + knob/16, times 0.8 (values 1/3) or 2.0 (2/4). The four 909 bands rise with velocity only from Accent knob 25 up (equal at 24); task-185's 'from 16 up' was wrong. The snares' snappy level is not scaled by accent.

New original facts: docs/initial_investigation.md 6.1 (pitch, range, no bound) and 7.2 (909 AC = 2 order, snappy unscaled); docs/original_bugs.md Q-5 (a rising slide ends 1.56 cents flat, the integer glide). Branch worktree-agent-a09e24d38a378c2c7, one commit on main 778c208. Docs only: no tests run.
<!-- SECTION:FINAL_SUMMARY:END -->
