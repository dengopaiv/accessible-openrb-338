---
id: TASK-185
title: 'Decide: what the VST3/CLAP plugin offers and how it works with the host'
status: Done
assignee:
  - '@pingw33n'
created_date: '2026-11-14T07:36:00Z'
updated_date: '2026-11-16T10:49:00Z'
labels:
  - decision
  - ui
  - player
milestone: m-7
dependencies: []
references:
  - docs/ui_spec.md
  - docs/song_load.md
  - docs/initial_investigation.md
  - crates/orb-player/src/lib.rs
  - crates/orb-ui/src/menu.rs
  - https://github.com/robbert-vdh/nih-plug
  - https://codeberg.org/RustAudio/nice-plug
  - https://docs.rs/nice-plug/latest/nice_plug/
priority: medium
type: spike
ordinal: 201000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
orb is to become a VST3/CLAP instrument plugin (nih-plug, both formats from one codebase) in m-7, next to MIDI input (task-182 remote control, task-183 live play, task-184 clock sync for the standalone app only). This task is the user's survey of similar plugins and the product decisions the plugin's implementation tasks need. Below: what the discussion of 2026-11-13/14 found, then each decision with options and a recommendation.

## Facts from the discussion
- Engine: runs only at 44.1 kHz, in 64-frame blocks; the player applies commands per 1024-frame call (the original's MME buffer). orb-player's `Player`/`Audio` split (lock-free, no allocation in the callback, a garbage queue) fits a plugin; commands should apply per 64-frame block at event offsets.
- A song (`.rbs`) = patterns (device data, edited live, read live in both modes) + tracks of controller events (knobs, pattern select ctrl 1, on/off ctrl 0). Song-mode record records controller moves only: never pattern contents, mixer mutes, the master, tempo, shuffle or loop points (docs/song_load.md 4.10, task-057). So patterns are like a DAW's shared clips, the song like its arrangement.
- The original has a slave mode: with MIDI sync active, Stop does not relocate (`transport+0x160`); `transport+0x15C == 46` (ReWire, [G]) leaves loop and tempo to the host.
- Neither VST3 nor CLAP lets a plugin edit host clips or automation lanes. Precedent for exchanging patterns: MIDI drag-out (D16, AudioRealism, XO, Toontrack), dropping MIDI files on the plugin, plugin MIDI output recorded by the host (Bitwig, Reaper, FL, Cubase; Ableton limited), host-specific scripting (Max for Live, Bitwig API, ReaScript).
- DAWs already model patterns: FL Studio (patterns + playlist, slide notes), Studio One (Pattern events with variations), Logic (Step Sequencer rows incl. tie and octave, aliases), Cubase shared copies, Reaper pooled items; Ableton/Bitwig clip launchers with bar launch quantization = ReBirth's bar-line latch.
- 303 note conventions of every clone: pitch = note, accent = velocity at or above a threshold, slide = overlapping notes (causal, no lookahead), gate = note length.
- The original plays no device from MIDI notes (task-183, docs/midi.md section 4): `sub_100261F0` is an unused copy of the 303's remote control with no caller, not a live input. Live play is new in orb (task-194).
- Mods: recrypted `.rbm`, 2.6 MB (Standard, embedded in orb) to 9.4 MB, about 3.4 MB on average over the user's 320 third-party mods.
- UI: orb-ui is eframe 0.37 + wgpu; nih_plug_egui runs an older egui on baseview/OpenGL, so the editor needs an updated egui-baseview or our own baseview+wgpu glue. No native menu bar in a plugin window, rfd dialogs risky, hosts eat keyboard shortcuts. The panel already fits any size with the aspect kept.

## Survey (to do)
Starting list, from memory, to check: Reason Rack Plugin (whole rack in one instance, same company as ReBirth); D16 Phoscyon 2 / Nepheton / Drumazon / Nithonat (one device per instance, patterns synced to host, pattern switch by low MIDI notes, MIDI drag-out, multi-out); AudioRealism ABL3 / ADM; Roland Cloud TB-303 / TR-808 / TR-909; Native Instruments Maschine (patterns and its own arrangement inside a DAW); XLN XO; Open303 / JC303 (open source, note conventions). For each: instance model, sequencer vs host notes, pattern switching, transport/song handling, MIDI export/import, multi-out, how samples/kits are stored.

## Decisions
1. **What the panel shows and does in plugin mode.** Transport strip (docs/ui_spec.md 3). Recommended:
   - keep Shuffle (plugin state, automatable) and the Pattern/Song switch;
   - tempo display shows the host tempo, arrows disabled;
   - Play/Stop/Rewind/FF, bar up/down and the loop controls disabled (dimmed): the host owns the transport;
   - bar display shows the host position;
   - Record disabled (see 2);
   - MIDI LED lights on host MIDI; Sync LED lit while following the host;
   - logo click (startup sound) kept, the startup sound off by default.

   Menus (in the panel, no native bar):
   - File: New, Open (load an .rbs into the instance), Save As (write an .rbs), Info; no Save (state lives in the project); no Export loop as audio file (the host bounces).
   - Mods: as in the app, Demo Song included.
   - Options: Vintage, Play Mod Startup Sound, key mode (best effort), plus zoom (75–200 %) and the MIDI settings (channel map, accent threshold, per-device mode).
   - Help: as is.
2. **Patterns vs host notes.** Options: (a) internal sequencer only, host gives transport; (b) host notes only; (c) per device a mode switch, Pattern (internal sequencer, host transport) or Notes (host notes, that device's pattern silent), mixable in one instance. Recommended: (c), Pattern by default.
   - Pattern selection from the host: an automatable parameter per device, plus MIDI notes or program changes per task-182's map, latched at the bar as in the original.
   - Editor: always edits the device's stored patterns. It follows the selected pattern when host automation changes it, as song mode does in the original. Edits are heard on the next pass. In Notes mode it still edits (not heard) and says so.
   - Notes on a device in Pattern mode: whatever task-183 finds the original did, else ignored.
   - Song mode follows the host position (ppq × 192 = ticks, locate and chase on jumps); a host-automated controller holds that controller against the song's track; no stop at the song end; ReBirth's own loop ignored.
   - Recording into song tracks: not in the plugin; the host records automation.
3. **Live note input.** Options: MIDI channels per device on one note input; CLAP note ports / VST3 event buses per device (poor host support for several); one device per instance. Recommended: one note input, the original's channel map, CLAP note events treated like MIDI, no MPE or note expressions; accent threshold configurable, slide by overlap. One-device-per-instance only if users ask.
4. **Export and import with the host.** Recommended:
   - drag a pattern out as a MIDI clip (accent = high velocity, slide = overlap), and "export all patterns" (one clip per pattern);
   - drag the song out as a MIDI arrangement (patterns placed as clips, knobs as CCs);
   - "Save MIDI file…" as the fallback where drag-out fails;
   - import: drop a MIDI clip on a device, quantized to 16ths, into the selected pattern; drop an .rbs to load it.
   - Not now: playing the song into the host's automation recorder.
5. **No direct MIDI devices.** Recommended: the plugin takes MIDI only from the host. Private ports mean the host does not record or replay the input, no sample-accurate timing, exclusive ports on classic Windows MIDI, and every instance receiving everything. Port access and clock sync stay in the standalone app. A possible later exception: LED feedback to a known controller.
6. **Mods in the plugin state.** Options: (a) a reference only (name + SHA-256), resolved from a mod folder setting; (b) the recrypted mod embedded in the state (3–10 MB per instance); (c) both. Recommended: (c). Always store the reference. Embed by default for a third-party mod, with a per-instance toggle, so projects open on another machine. Never embed the built-in Standard kit. Use the embedded copy when the folder lacks the mod. Note that embedding redistributes the mod with the project.

## Also open from the discussion (decide here or as follow-ups)
- Formats: VST3 + CLAP via nih-plug; AU later via clap-wrapper or never. nih-plug is git-only and slow-moving.
- Sample rate: resample to the host rate and report the latency (not bit-exact), or 44.1 kHz only.
- Parameters: stepped integers mirroring the u8 controllers, stable ids.
- Outputs: stereo first; per-device multi-out later (needs engine taps).
- Editor: zoom steps saved in the state, plus aspect-locked resize where the host allows; minimum 100 %; ScrollArea fallback below that.
- Packaging: .vst3/.clap bundles in scripts/package.sh; macOS signing and notarization; validation with clap-validator, the VST3 validator and pluginval.

The outcome (2026-11-15) is in the final summary; it replaces the recommendations above where they differ.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 The survey of similar plugins is in the task notes, each with its instance model, sequencer/notes handling, pattern switching, transport and song handling, MIDI export/import, outputs and sample storage
- [x] #2 Each of the six decisions, and each item of 'Also open', has an outcome (or an explicit deferral) with the date in the final summary
- [x] #3 The plugin's implementation tasks are created in m-7 from the outcomes, each depending on this task where it needs a decision
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Survey 2026-11-14/15 (the user in Reaper, with the manuals):
- D16 Phoscyon 2: one 303 per instance. EXT mode = mono synth from host notes, accent by a velocity threshold (Cog > MIDI > Note Velocity Settings), overlap = slide. SEQ mode with Pattern Trigger Modes: Live (GUI only, change at the end of the pattern, 48 patterns A-D x 12) and Host Note (48 notes from the C of the second-lowest octave; the pattern plays only while its note is held; the DAW's Start is ignored); Host Link Active option. MIDI output. Export: 'Drag MIDI file' (a MIDI clip, velocities settable) or 'Drag pattern' (native format, to disk or another Phoscyon; Reaper cannot take it). GUI size presets / system scale (manual p. 56); does not follow the host window's resize.
- AudioRealism ABL3: one 303 per instance. MIDI modes Note (synth, accent at velocity > 100, overlap = slide) and Pattern (each key = one of 128 patterns). Sync modes Host Sync (runs with the transport; MIDI can change the pattern) and MIDI Trigger (gate). Imports ReBirth 338 patterns; Export Pattern as MIDI (padded to 16 steps); pattern lock; step recording via MIDI. GUI Small/Default/Large plus its own resize corner (it asks the host to resize; Reaper honors it); does not follow the host window's resize.
- The rest of the starting list (Reason Rack Plugin, D16 drum machines, ADM, Roland Cloud, Maschine, XO, Open303/JC303) was not surveyed: the user found the two 303 plugins enough (2026-11-15).
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Decided by the user, 2026-11-15/16. Where this differs from the description's recommendations, this wins.

**Framework.** nice-plug (https://codeberg.org/RustAudio/nice-plug, docs.rs/nice-plug; 0.4.2 of 2026-11-08, ISC license; the community fork of nih-plug), not nih-plug. Its `nice-plug-egui` adapter uses egui 0.37 (as orb-ui) on egui-baseview with a `wgpu` feature; VST3 and CLAP exports; host-driven editor resizing through `Editor::resize_hint()` (`ResizeHint`, with aspect-ratio options; the wrapper answers CLAP `gui.can_resize`/`get_resize_hints` and VST3 `canResize`) and `EditorHandle::set_size()`; MIDI and note output; optional sample-accurate automation. Experimental, with limited testing on macOS (README, 2026-11-16).

**Instance and modes.** One instance = the whole ReBirth (two 303s, 808, 909, effects, mixer). No song playback in the plugin for now (song mode deferred). orb's Play/Stop is the switch between two modes and is an automatable host parameter:
- **Pattern mode (orb playing).** orb's sequencer plays the patterns. Host notes on a section's channel select that section's pattern: notes 0..31 = patterns 1..32 (banks A-D x 8). Latched: a note changes the pattern at once and it stays until the next note; note-off is ignored. A section that has had no note plays the pattern the plugin state has (the panel's), and the panel can change it live, as in standalone song playback: the last write wins. Plays whether or not the host plays. Tempo: the host's (hosts report it while stopped), the song's if the host reports none. While the host plays, the step comes from the host position (bar 1 beat 1 = step 1), so host start, seek and loop jump realign it and bounces are deterministic; with the host stopped orb free-runs.
- **Note mode (orb stopped).** Host notes play the devices (the original has no live play, task-183; this decides task-194 for the plugin). Works whether or not the host plays.
  - 303: pitch = note over the whole audible and sensible range (the MIDI note of ReBirth's C at Tune center is for the implementation to find), accent = velocity at or above a fixed threshold (100 to start; a setting later), overlap = slide, same pitch overlap = tie, last-note priority, legato.
  - Drums on General MIDI notes: BD 36, RS 37, SD 38, CP 39, CH 42, OH 46, LT/MT/HT 43/47/50, 808 congas L/M/H 64/63/62, claves 75, maracas 70, cowbell 56, 808 cymbal and 909 crash 49, 909 ride 51. 909 flam: no note (two quick notes sound as one).
  - 808 shared slots (tom/conga, rim/claves, clap/maracas): a note plays the voice it names for that hit only. The slot's switch parameter is not changed by notes (it governs pattern mode). One voice per shared slot (the engine's), so the slot's two voices cannot sound at once; lifting that is a later item.
  - Accent per hit (in note mode the AC row is not global: its factor applies to each hit). 808 (steps on/off + AC row): velocity below 100 = normal, at or above = AC. 909 (step level low x0.8 / high x2.0, times the AC row's 1 + AC*knob/32): four velocity bands in the loudness order that holds from Accent knob 16 up: 1-47 low, 48-95 high, 96-111 low+AC, 112-127 high+AC (edges to tune by ear).
- **Channels: a note mapping separate from the Standard Mapping.** Devices on channels 2..5 (303 #1, 303 #2, 808, 909) for both modes; the Standard Mapping stays exactly the original's on its own channel (default 1) and keeps working; a remote channel equal to a device channel is refused or warned.

**Timing and latency.** The standalone player's 1024-frame call (`CALL_FRAMES`, the original's MME buffer) does not apply. The plugin renders 64-frame engine blocks for any host buffer size, keeps the leftover frames, applies each note and parameter change at the block holding its timestamp, renders one block ahead and reports a constant 64-frame latency (plus the resampler's when the host is not at 44.1 kHz).

**Visual feedback.**
- Note mode, separate from the editor's selection, which notes never change:
  - 303s: none for now (maybe later).
  - 808: a green outline of the same shape as the orange selection outline, on the chip of the voice played (the LT legend or the LC upper chip, whatever the switch says). On an instrument that is also selected only the green shows. A hit with AC also outlines AC green.
  - 909: the instrument's LED glows green instead of red; AC's LED green for a hit with AC.
  - Lit while the voice sounds: active and its gain above a threshold (about -60 dB), with a minimum on-time (about 100 ms, tuned by eye). So several voices can be lit at once; a CH choke darkens OH; a retrigger restarts the light; the 808/909 BD is lit while either of its two alternating voices sounds. AC is lit while any voice whose last hit had AC is lit. The engine exposes a per-voice sounding mask to the UI per block.
  - Green must stay distinguishable from red/orange for color-blind users (brightness or glow, not hue alone).
- MIDI LED: lights on incoming host MIDI, as in the standalone app.
- Sync LED: lit while orb follows the host position (pattern mode, orb playing, host playing); off while orb free-runs (host stopped) and in note mode.

**Panel in the plugin.** Everything shows and works as in the standalone app except: hidden pattern/song switch, loop switch and range, Rewind, Record, File > Save; the tempo display stays, read-only (its up/down buttons hidden), showing the tempo orb plays at (the host's, or the song's when the host reports none). File > Open always switches to pattern mode. Edits live in the DAW project (no .rbs export for now). The whole plugin window is the panel: no metallic filler background. Sizing, aspect-locked to the panel, saved in the state: (1) host-driven resizing (nice-plug `ResizeHint` with the panel's aspect ratio); (2) the plugin's own resize corner, invisible: only the mouse cursor changes over it (the plugin asks the host: VST3 `IPlugFrame::resizeView`, CLAP `gui.request_resize`). No zoom presets for now.

**Host parameters.** Every visible control, including the five 808 voice switches (ctrl 11, 14, 17, 19, 21); controls hidden in the plugin (e.g. Record, tempo buttons) are not parameters. Stable ids. Notes never change parameters.

**State.** The loaded .rbs (so File > Info shows its tempo etc.), the patterns and controls, the mode, the editor size, and a non-default mod embedded (mods are loadable in the plugin).

**Pattern drag-out.** Dragging a section's pattern button onto the DAW gives that section's pattern as a one-track MIDI clip in the note mapping (its channel, GM drum notes, slide as overlap; the 808 shared slots use the note of the switch's current position). Accent as velocity by the same bands in reverse, at fixed values: 303 and 808 normal/accent; 909 low/high/low+AC/high+AC (e.g. 40/80/104/127), so the clip played back in note mode sounds like the pattern. Drag only: no 'Save MIDI file' fallback, no song or all-patterns export, no import. Needs native drag-out per platform (egui/winit/baseview cannot start an OS drag).

**Identity and formats.** Name `OpenRB-338`, vendor `OpenRB-338`, CLAP id `io.github.pingw33n.openrb-338`, VST3 class id a random GUID made once, category Instrument|Synth, URL the GitHub repo; all permanent after the first release. VST3 primary, CLAP too (free with nice-plug, needs testing in a CLAP host). No AU.

**Recorded as recommended (no objection, 2026-11-15).** MIDI only from the host, no private ports (decision 5). Resample to the host rate and report the latency; deterministic offline bounce; independent instances. Stereo output. Packaging: .vst3/.clap bundles, clap-validator, the VST3 validator, pluginval; macOS signing later.

**Standalone app.** No changes for now.

**Later (to become tasks).** Song mode in the plugin; recording as import with save as .rbs; per-device pattern/note override; both voices of an 808 shared slot at once; 303 visual feedback in note mode; plugin MIDI output; multi-out (stems); accent threshold setting; .rbs export from the plugin; zoom presets.

**Follow-up tasks (2026-11-16).** task-197 plugin core, task-198 host sync and pattern mode, task-199 note mapping doc, task-200 note mode, task-201 editor, task-202 note-mode visual feedback, task-203 pattern drag-out, task-204 packaging; task-196 decides the 'Later' items. The user reviews task-201, task-202 and task-203 before merge. task-194/195 (live play in general) are untouched: for the plugin, task-200 implements what this task decided.

**Change of 2026-11-16 (the user):** the device channels are 1..4 (303 #1 = 1, 303 #2 = 2, 808 = 3, 909 = 4), not 2..5, and the Standard Mapping is off by default in the plugin (channel 16 when on, any channel but 1..4; its transport notes ignored; the host's MIDI learn is the main controller mapping): task-207. Its UI: task-208. Pattern alignment under other time signatures: task-206, option A.

**Correction of 2026-11-16 (task-199, checked in the DLL at 0x1002ACAF..0x1002ACD1 and in tr909.rs):** the 909's AC step value is 2, so the AC factor is 1 + knob/16; the four velocity bands (low, high, low+AC, high+AC) get louder in that order only from Accent knob **25** up (at 24 low+AC equals high), not from 16 as written above. docs/midi.md section 5.5 has the gain table.
<!-- SECTION:FINAL_SUMMARY:END -->
