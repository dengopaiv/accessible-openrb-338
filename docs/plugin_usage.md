# Plugin Usage

How to drive the OpenRB-338 instrument plugin from a DAW. It is a VST® 3 plugin: one instance is the whole ReBirth (two 303 bass lines, the 808, the 909, the effects and the mixer) with a stereo Mix output, four stereo outputs of the devices, 22 stereo outputs of the drum instruments (section 6) and one MIDI input. This page is also in the plugin: Help > Plugin Usage.

## 1. Modes

The mode decides what the plugin does with the MIDI notes and with the host's transport. Choose it with the mode selector on the panel, after the Bar display: click a row (Latch Pattern, Gate Pattern or Note), or drag the knob up and down. It is the only place to change the mode. The mode is saved with the project. A new instance starts in **Note** mode, so a plugin you have just inserted makes no sound of its own. File > New and File > Open keep the mode.

- **Note.** The notes play the devices like a synthesizer, whatever the host's transport does: playing or stopped. The song does not run.
- **Latch Pattern.** The plugin plays its patterns while the host plays and follows the host's position, so the pattern steps are in time with your project. With the host stopped there is no sound. A note selects a pattern for a section and the pattern stays until the next note on that channel.
- **Gate Pattern.** As Latch Pattern, but a section plays its pattern only while a note on its channel is held: the notes of a MIDI clip are the arrangement, and a gap is silence. A held note also plays with the host stopped.

**Preview mode.** The panel's Play button starts Preview mode and Stop ends it, in any mode. Preview mode is for editing patterns: it plays the patterns of the sections free-running, ignores the host's transport and every MIDI message and parameter change from the host, and keeps its own values of the controls you turn meanwhile (they do not go to the host; leaving Preview mode restores the host's values). It plays at the host's tempo (the song's when the host reports none). Preview mode is not saved with the project. The host's start and stop do not press Play and Stop.

## 2. Channels

The four sections listen on fixed MIDI channels:

| Channel | Section |
|---|---|
| 1 | 303-1 |
| 2 | 303-2 |
| 3 | 808 |
| 4 | 909 |

A keyboard on its default channel 1 plays 303-1. Messages on other channels do nothing.

## 3. Selecting patterns (Latch and Gate)

In both pattern modes the notes 0 to 31 select the pattern of the section on the note's channel: 0 is A1, 7 is A8, 8 is B1 and 31 is D8. The note names on this page call MIDI note 60 C4 (middle C). Some DAWs call note 60 C3, and then every name they show is one octave lower than the names here, so the tables give the MIDI note numbers too: go by the numbers.

| Bank | Notes | Names |
|---|---|---|
| A1 to A8 | 0 to 7 | C-1 to G-1 |
| B1 to B8 | 8 to 15 | G#-1 to D#0 |
| C1 to C8 | 16 to 23 | E0 to B0 |
| D1 to D8 | 24 to 31 | C1 to G1 |

- **Latch.** A note selects the pattern at once, in step with the song, and it stays until the next note on that channel or a change on the panel. With the host stopped only the selection changes (the panel shows it) and the host's start plays it. A note-off does nothing. A pattern change from the panel or from automation takes effect at the next bar line, as in ReBirth.
- **Gate.** The note-on starts the section at once and the note-off stops it at once (a 1.45 ms fade, one 64-frame block at 44.1 kHz), with no bar-line wait. With the host playing, the pattern plays from the step at the host's position, in step with the song, and follows loops and seeks. With the host stopped, a held note plays the pattern from step 1 at the host's tempo. When several notes are held on a channel the last one pressed wins; releasing it falls back to the newest one still held. The four sections gate independently.
- A velocity 0 note-on is a note-off. Notes above 31 do nothing in the pattern modes.
- Notes never change parameters, so the host does not record pattern changes made with notes as automation: the MIDI clip is the record. The pattern parameter keeps the panel's pattern, which is what a saved project starts from.
- A section's On/off switch is your mute: in Gate a section sounds only while its switch is on and its note is held.

## 4. Note mode: what each note plays

**303 (channels 1 and 2).** The note is the pitch: MIDI note 36 (C2) is the C at the Tune knob's center, in concert pitch (A4 = 440 Hz). With the Tune at center the range is notes 0 to 83 (C-1 to B5); a note outside the range of the pitch table is ignored. The Tune knob transposes as in ReBirth, read at each note-on.
- Accent: velocity 100 and up.
- Slide: a note that starts while another is still held slides to it, with no new attack. The same pitch is a tie.
- Several held notes: the newest sounds. Releasing it slides back to the newest note still held; releasing the last one ends the note.
- Stuck notes: if the host loses a note-off (for example a MIDI send removed while a note plays), the 303 keeps sliding from that note. It recovers when that key is played and released again, on All Notes Off (CC 123) or All Sound Off (CC 120) on its channel, and when the host stops. If the MIDI source is removed while a note is held, no more MIDI arrives from it, so that note sounds until the transport stops or an all-notes-off arrives. In REAPER the action "Send all-notes-off and all-sounds-off to all MIDI outputs/plug-ins" clears it at once, and REAPER's MIDI reset preferences ("Reset on:") choose whether it also resets on stop or on playback start.

**808 (channel 3) and 909 (channel 4).** The notes are the General MIDI drum notes:

| Note | Name | 808 | 909 |
|---|---|---|---|
| 36 | C2 | BD | BD |
| 37 | C#2 | RS | RS |
| 38 | D2 | SD | SD |
| 39 | D#2 | CP | CP |
| 42 | F#2 | CH | CH |
| 43 | G2 | LT | LT |
| 46 | A#2 | OH | OH |
| 47 | B2 | MT | MT |
| 49 | C#3 | CY | CC |
| 50 | D3 | HT | HT |
| 51 | D#3 | no sound | RC |
| 56 | G#3 | CB | no sound |
| 62 | D4 | HC | no sound |
| 63 | D#4 | MC | no sound |
| 64 | E4 | LC | no sound |
| 70 | A#4 | MA | no sound |
| 75 | D#5 | CL | no sound |

- **Shared slots of the 808.** Five slots hold two sounds each (LT/LC, MT/MC, HT/HC, RS/CL, CP/MA). The note picks the sound for that hit: 43, 47, 50, 37 and 39 the first of each pair, 64, 63, 62, 75 and 70 the second. The slot's switch on the panel is not changed and still decides the sound in the patterns. A slot has one voice, so its two sounds cannot sound together.
- **Velocity, 808:** 100 and up is accented, below is not.
- **Velocity, 909:** four bands: 1 to 47 low, 48 to 95 high, 96 to 111 low with accent, 112 to 127 high with accent; the bottom of each band adds a flam (next item). From Accent knob 25 up the bands get louder in that order; below 25 the 96 to 111 band is quieter than the 48 to 95 band (the same at 24).
- **Flam, 909:** the bottom of each band plays the hit with a flam, as a flam step does (timed by the Flam knob): 1 to 7, 48 to 55, 96 to 99 and 112 to 115. A dragged-out pattern uses them for its flam steps.
- A note-off does nothing: the drum voices decay on their own. There is no separate flam note; two quick notes are two hits.
- The section's On/off switch does not mute notes; the mixer channel's On/off does.
- **Feedback.** In Note mode the panel lights what sounds: a green ring on the 808's chip of the sound played (and on AC), a green LED in the 909's instrument row. A hit lights for at least 0.1 s.

## 5. In the host

- **Parameters.** Every control you see on the panel is a host parameter, named after its section: 303-1 Cutoff, 808 BD Level, the mixer's faders and so on, 119 in all. Automate them, or map a hardware control with the host's own MIDI learn. Turning a control on the panel is recorded by the host as automation as usual.
- **Not parameters.** The mode, Preview mode and the control changes made in it, and the pattern data (the steps you edit). The pattern data is saved in the project, not automated.
- **Latency.** At 44.1 kHz the plugin reports 64 samples. At other host rates it reports the delay of its resampler, rounded up: 105 samples at 48 kHz, 209 at 96 kHz. At 44.1 kHz a note or parameter change sounds 0 to 63 samples after its position and never before it. At other rates it sounds up to one 64-frame engine block (1.45 ms) late, and, because the reported latency is rounded up, up to a fraction of a sample early. Renders do not depend on the buffer size.
- **Sample rate.** The engine runs at 44.1 kHz and the output is resampled for other host rates.
- **State.** The project holds the song and the patterns as you edited them, the mod, the controls, the mode and the window size. File > Open loads a song into the plugin; Mods loads a mod, which is then saved in the project too.
- **Drag a pattern out as MIDI.** Press a pattern button of a section and drag it out of the window onto a host track: it drops a MIDI clip of that pattern, on the section's channel, 192 ticks to the quarter note, with the shuffle baked in. The press also selects the pattern. The clip is made for Note mode: played into Note mode it plays the pattern. In the pattern modes its notes 0 to 31 would be pattern selects, so use it in Note mode.
- **Keys.** The plugin window takes no keys: the host keeps them. Everything is done with the mouse. (On Windows, Escape cancels a pattern drag-out in progress.)

## 6. One output per device and per drum

Besides its main output, **Mix**, the plugin has four stereo outputs, one per device, and 22 for the drum instruments (below), so that each can be recorded and processed on a track of its own:

| Output | Mixer channel |
|---|---|
| Mix | the master: everything |
| 303-1 | 1 |
| 303-2 | 2 |
| 808 | 3 |
| 909 | 4 |

- **What a device output carries.** The device through the distortion, PCF and compressor when the mixer routes them to its channel, as ReBirth's own ReWire outputs do. It is mono, the same signal on both sides.
- **What it does not carry.** The channel's level, pan and on/off switch, the delay (a send: its echoes are only in the Mix), the compressor when it is on the master, and the master level. Those change the Mix only: set the device's level and pan on its own track.
- **The drum outputs.** After the four device outputs come one output per drum instrument: outputs 5 to 15 are the 808's BD, SD, LT, MT, HT, RS, CP, CB, OH, CH and CY, and outputs 16 to 26 the 909's BD, SD, LT, MT, HT, RS, CP, CH, OH, CC and RC (the host names them "808 BD" to "909 RC"). The 808's LT/LC, MT/MC, HT/HC, RS/CL and CP/MA share one output each, as they share one sound. A drum output carries the instrument's sound with its level knob, accent and envelope, and nothing after it: no distortion, PCF or compressor, no mixer channel, no delay.
- **Options > Output Routing...** chooses what stays in the mix. Under "To Mix-L/R", the four devices: unchecked, a device leaves the Mix (its delay send stays, so its echoes stay in the Mix, as in ReBirth's ReWire). Under "To 808" and "To 909", each instrument: unchecked, it leaves its machine, so it is out of the machine's output (3 or 4), its effects, its mixer channel, its delay and the Mix, and only on its own output. Every box is checked at first, and the boxes are saved with the project. A box's tooltip shows its output's number, #1 for 303-1 to #26 for 909 RC (the Mix is output 0).
- **The Mix is unchanged** while every box is checked: it is the same sound whether or not you use the outputs. Uncheck what you take to its own track, or you hear it twice. Turning a channel off in the panel's mixer also removes the device from the Mix (and from the delay) but not from its own output.
- **Unused outputs** cost almost nothing. They have the same latency as the Mix.

## 7. Recipes

**One track per device (Reaper style).**
1. Insert OpenRB-338 on one track.
2. Make four MIDI tracks, one per section, and send each to the plugin's track (MIDI only, no audio).
3. Give each send the channel of its section: 303-1 on channel 1, 303-2 on 2, 808 on 3, 909 on 4 (in Reaper: the send's MIDI settings, source All, destination the channel).
4. Choose Note mode and play or draw notes (section 4).

**Arranging with Gate Pattern.**
1. Choose Gate Pattern with the mode selector.
2. On each section's track draw one note per clip region, on the note that is the pattern: A1 is note 0 (C-1), B3 is note 10 and so on (section 3).
3. The length of the note is the length of the section's part: a gap is silence, and two sections can start at different bars and play in step. Move the clips to arrange the song. The host's loop and seeks are followed.

**Live pattern switching with Latch Pattern.**
1. Choose Latch Pattern and start the host.
2. Play the pattern notes of a section, 0 to 31, from a keyboard on that section's channel (a MIDI-learned pad row also works). The pattern changes at once, in time, and stays until the next note.
3. Record the notes into a MIDI clip to keep the performance: the clip, not automation, is the record.

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="../assets/steinberg/VST_Compatible_Logo_Steinberg_negative.svg">
  <img src="../assets/steinberg/VST_Compatible_Logo_Steinberg.svg" alt="VST Compatible" width="120">
</picture>

VST is a registered trademark of Steinberg Media Technologies GmbH.
