# The VST3/CLAP plugin (`crates/orb-plugin`)

OpenRB-338 as an instrument plugin, milestone m-7. The product decisions are task-185's final
summary (2026-11-15/16); this document describes how the plugin implements them. task-197 built
the core described here; task-198 the host sync, the pattern selection by notes and the Standard
Mapping (section 8); task-200 note mode (section 9); the editor is task-201, packaging and the
validators task-204.

## 1. What runs

One instance is the whole ReBirth: the two 303s, the 808, the 909, the effects and the mixer,
the engine of `orb-engine` driven as orb-player drives it in the standalone app. The plugin is
built on nice-plug 0.4.2 (codeberg.org/RustAudio/nice-plug, ISC), the community fork of
nih-plug, with only its `vst3` feature (no editor and no logger yet). Its VST3 bindings are the
`vst3` crate 0.3.0 (MIT OR Apache-2.0); both licenses are compatible with the repository's
GPL-3.0-only.

| File | Thread | What |
|---|---|---|
| `src/lib.rs` | both | the nice-plug `Plugin`, `ClapPlugin` and `Vst3Plugin`, the exports |
| `src/instance.rs` | main | the parameters, the document the state saves, the editor's calls, the engine build |
| `src/core.rs` | audio | the 64-frame block scheduler, the parameter changes, the mode |
| `src/resample.rs` | audio | the resampler for host rates other than 44.1 kHz |
| `src/params.rs` | both | the parameter table (a constant) |

The process side reuses orb-player's lock-free split (`crates/orb-player/src/lib.rs`): the
UI half (`Player`) stays on the main thread behind a mutex that `process` never takes, the
callback half (`Audio`) is the process side's. `Player::with_call_frames(.., 64)` makes calls of
one engine block (the standalone app keeps its 1024-frame calls, `CALL_FRAMES`); the plugin
renders through `Audio::drain`, `Audio::apply_now` and `Audio::render_call`. Commands, songs and
mods travel in orb-player's command queue, and what the audio thread lets go of goes back in
its garbage queue, so `process` allocates and frees nothing.

## 2. Identity

Permanent after the first release (task-185):

| | |
|---|---|
| Name, vendor | `OpenRB-338`, `OpenRB-338` |
| CLAP id | `io.github.pingw33n.openrb-338`; features instrument, synthesizer (task-185's Instrument\|Synth) and stereo (the port layout) |
| VST3 class id | `EFAB08AF-B39D-4399-8A80-3DC1F84ED3D6` (made once with `uuidgen`, 2026-11-16; `VST3_CLASS_ID` in `src/lib.rs`); category Instrument\|Synth |
| URL | https://github.com/pingw33n/OpenRB-338 |
| Ports | stereo out, no audio in, one MIDI input (`MidiConfig::MidiCCs`: notes and control changes, section 8) |

`tests/exports.rs` loads the built library and checks them through the CLAP and VST3 C APIs.

## 3. Parameters

One stepped integer parameter per control the plugin's panel shows (120): every controller of
the 303s (0..8 each: on/off, pattern, tune, cutoff, resonance, env mod, decay, accent,
waveform), the 808 (0..28, the voice switches 11, 14, 17, 19, 21 included), the 909 (0..29),
the delay (0..4), the distortion (0..2), the PCF (0..6) and the compressor (0..2); the mixer's
master, compressor and PCF inserts, and per channel on, level, pan, delay send and distortion;
the shuffle; and the mode, orb's Play/Stop. Hidden in the plugin and not parameters: the
pattern/song switch, the loop switch and range, Rewind, Record, the tempo buttons (task-185).
The pattern edits (steps, a pattern's length and shuffle flag) are pattern data, kept in the
state (section 5), not parameters.

- **Ids.** `g1` the mode and `g20` the shuffle (their song msg 35 ids, `docs/song_load.md`
  section 3); `d<device>c<n>` the others, with the device id (0 mixer, 1 and 2 the 303s, 3 the
  808, 4 the 909, 5 delay, 6 distortion, 7 PCF, 8 compressor) and the controller number
  (`docs/song_load.md` section 5; the 909's in RBS order). For the mixer `n` is the byte's
  offset in its msg-41 block (section 4.1: 0 master, 1 compressor id, 2 PCF id, `5 + 6c + k`
  channel c's field k, 0 on .. 4 distortion), which equals the controller number where the byte
  has one (`6 + 6c` .. `9 + 6c`). `crates/orb-plugin/tests/param_ids.txt` lists every id with
  its target, range and default; `tests/params.rs` fails when the list changes.
- **Ranges.** 0..127 for the knobs and faders; 0..1 for the switches; 0..31 the patterns (shown
  A1..D8), 1..32 the delay steps, 0..54 the PCF wave, 0..5 the compressor insert (off, master,
  channel 1..4), 0..4 the PCF insert (off, channel 1..4).
- **Defaults** are the song constructor's values (`docs/song_load.md` section 4.9; the shuffle
  64, the sequencer's ctor value): a new instance's parameters change nothing in a new song
  (`the_defaults_are_the_new_songs_values`).
- **Into the engine.** A changed value is sent as the panel's message
  (`docs/song_load.md` section 4.10): `{ctrl, v}` to the device (`Command::DeviceParam`), mixer
  msg 145 `{id, v}` or 147 `{c, k, v}`, song msg 35 `{20, v}` for the shuffle. The mode is a
  level (section 4).
- **A song's out-of-range bytes.** A song can hold a byte outside the control's range (a 303
  waveform of 127, task-156). Its parameter shows what the engine plays (a switch is on for any
  byte but 0, the PCF's mode is low-pass only at 1, a number is clamped), and a value is sent
  only when the engine does not already play it, so loading or restoring does not rewrite such a
  byte until the control is moved.

## 4. Timing

The engine runs only at 44.1 kHz, in blocks of 64 frames; the host's buffers can be any size.

- **One block ahead.** Host frame `j` (counted from the activation) plays engine frame
  `j - 64`; the first 64 frames after an activation are silence, and the plugin reports 64
  frames of latency. A block is rendered when its first frame is due.
- **Events.** A host event at frame `t` (a parameter change; with
  `SAMPLE_ACCURATE_AUTOMATION` nice-plug splits the host's buffer at each change, so a change is
  always at a call's first frame) is applied right after frame `t` is produced. The block that
  plays at host frames `[64m, 64m + 64)`, which after the host's latency compensation is the
  block of those very frames, is rendered after exactly the events of frames before `64m`.
  The output is therefore a function of the events' frames alone: buffer sizes 1, 63, 64, 512
  and 4096 give the same samples (`tests/timing.rs`). A call of no frames takes no changes;
  they are seen at the next call.
- **The mode** (`g1`) is a level: before each block the engine is played (song msg 35 `{1,1}`)
  when the mode is Play and the song does not play or a loaded song waits for Play, and stopped
  (`{1,0}`, which locates to the bar start) when it is Stop and the song plays. Its default is
  Stop (0): the original starts stopped (a new or opened song does not play until Play), and a
  plugin just inserted should make no sound of its own; task-185 does not fix the default. Play
  is pattern mode: the patterns play whether the host plays or not, following its position
  while it plays (section 8); Stop is note mode (task-200).
- **Activation.** Every activation makes a new player and builds the engine from the state and
  the parameters (section 5), so an offline bounce does not depend on what played before it and
  reads no clock. The UI side's commands are applied at the start of the next block, as the
  standalone player's are at the start of its call.
- **The same engine as the app.** Without changes the plugin's output equals orb-player's for the
  same song in pattern mode, 64 frames later (`the_plugin_plays_as_the_player_one_block_later`).

### 4.1 Another host rate

When the host does not run at 44.1 kHz the engine's output is resampled (`src/resample.rs`): a
fixed-ratio windowed-sinc interpolator. The ratio `44100 / rate` is kept as a reduced fraction
`num / den`, and host frame `j` is the engine signal at engine position
`j * num / den - (64 + W)`, an integer plus a remainder over `den`, from the `2W` engine frames
around it. The weights are a sinc with a cutoff of 0.94 of the lower Nyquist frequency under a
4-term Blackman-Harris window, normalized to a sum of 1 per phase; `W` is 32 engine frames when
the host rate is at least 44.1 kHz and `ceil(32 * 44100 / rate)` below it. The weights of the
`den` phases are tabulated when `den` is at most 4096 (48 kHz: 160 phases; 96 kHz: 320) and
computed per frame otherwise. Everything is f64 and depends on the frame index only, so the
output is again independent of the buffer sizes and identical between runs.

The block a host frame needs ends 64 engine frames before `j * 44100 / rate`, so it is rendered
after the events before that host frame, as at 44.1 kHz. The reported latency is the delay
`64 + W` engine frames in host frames, rounded up so that it is never shorter than the delay:
105 at 48 kHz (104.49), 209 at 96 kHz (208.98), 64 at 22.05 kHz (exact).
The resampled output is not bit-exact with the engine's (task-185: resample and report the
latency).

## 5. State

nice-plug saves the parameters (the controls and the mode) by id. The rest is one persisted
field, `orb`, a JSON object (version 1):

| Key | What |
|---|---|
| `version` | 1 |
| `song` | the song file as loaded (`.rbs` or a ReBirth 1.x song), base64; null for a new song. File > Info reads its tempo and text from it |
| `patterns` | the four pattern arrays as edited (303 #1 and #2: 32 x 98 bytes; 808 and 909: 32 x 194), base64 |
| `vintage` | the Vintage sound model |
| `mod` | null for the built-in kit (never embedded); else `{name, sha256, data}`: the mod's INFO name, the SHA-256 of its bytes and the recrypted `.rbm`, base64 (the mods are 2.6 to 9.4 MB, so about 3.5 to 12.5 MB of text) |
| `editor_size` | the editor's size, for task-201 |
| `remote_enabled` | whether the Standard Mapping (Remote MIDI) is on; false by default and when absent (section 8) |
| `remote_channel` | the Standard Mapping's MIDI channel, 0-based (15: channel 16, the default and when absent); a section's channel (0..3, MIDI 1..4) restores as 15 with `last_error` set, the rest of the state kept (section 8) |

The plugin is not released: no state from before task-207 (section channels 2..5, the mapping
always on, channel 1) is migrated or recognized as such; such a state's `remote_channel` is read
as it is.

A restore (nice-plug sets the parameters, then calls `deserialize_fields`) replaces the
document and, when the instance runs, builds the engine and sends it; otherwise the next
activation builds it. The build (`instance::build`): the song (or a new song) with the
document's mod, in pattern mode (slot 9 makes the pattern-mode copies live, as the original's
switch does) and stopped; each of the 32 patterns of the four sections as the panel's whole
pattern message (sub 44, 61, 78); the sound model; then every parameter whose value the engine
does not already play, through its message. A state that does not parse, or a song or mod that
does not load, gives a new song or the built-in kit, and `Main::last_error` says why. A mod's
SHA-256 is checked on restore. Instances share nothing but constant data (the tables and the
built-in samples): each has its own player and process globals.

## 6. The main thread's calls

For the editor (task-201), on `OrbParams` (`OrbPlugin::orb_params`):

- `load_song(bytes)` (File > Open) and `new_song()` (File > New): the song, with its own patterns
  and controls and the instance's mod, is built and sent (it plays at once when the mode is
  Play). It returns the song's control values by parameter id; the caller sets them through the
  host (`ParamSetter`), so that the host, its automation and the state follow. Until then the
  parameters keep their old values, and nothing is sent for them: a parameter is sent only when
  it changes.
- `set_mod(Some(rbm) | None)` (the Mods menu): the drum machines take the recrypted mod (song
  msg 19, `Command::Mod`); `None` or the standard mod is the built-in kit.
- `edit(command)`: a pattern edit (`PatternField`, `PatternHeader`, `Pattern`, `StepField`,
  `Step`), sent and kept in the document; any other command is refused.
- `set_vintage(on)`, `editor_size()`, `set_editor_size(size)`.
- `snapshot()`: orb-player's `Snapshot` after the last engine block: the engine's state (the
  patterns playing, the tempo orb plays at for the read-only tempo display), `host_sync` (the
  Sync LED) and `midi_in` (a count of the host's MIDI messages: the MIDI LED lights when it
  changes), and `sounding` (the 808's and 909's sounding sounds and their AC flags, section 9.3).
- `remote_enabled()`, `set_remote_enabled(on)`: the Standard Mapping's on/off (default off).
- `remote_channel()`, `set_remote_channel(c)`: the Standard Mapping's channel (section 8); a
  section's channel is refused.
- `select_pattern(row, pattern)`: a pattern button's click as the panel's message (it plays from
  the next bar line). For a click on the pattern the parameter already holds, when a host note
  has selected another since: setting the parameter to its own value is no change the host or
  the process side sees. Otherwise the editor sets the parameter.

## 7. Building and testing

`cargo xtask bundle orb-plugin --release` builds the library and lays out `OpenRB-338.vst3`
and `OpenRB-338.clap` under `target/bundled/` (`xtask/` on nice-plug-xtask 0.1.1, `bundler.toml`
for the bundle name, the alias in `.cargo/config.toml`). The xtask finds the workspace from its
own manifest: nice-plug-xtask's own entry point takes the topmost ancestor directory with a
`Cargo.toml` as the workspace, which is the wrong one for a checkout inside another (the
worktrees under `.claude/worktrees/`). Signing, the packages and the validators
(clap-validator, the VST3 validator, pluginval) are task-204.

The tests (`cargo test -p orb-plugin`; `tests/` and the resampler's unit tests): the buffer
sizes and the 48 kHz path (`timing.rs`), the parameter list and defaults (`params.rs`), the state
(`state.rs`, with a test mod made from the standard mod: renamed, the 808's and 909's bass drum
and first snare swapped), no allocation in `process` (`no_alloc.rs`, a counting allocator, and
the process side's sources checked for lock types), and the exports (`exports.rs`): it builds
the library, loads it with the dynamic loader, checks the CLAP descriptor and the VST3 factory's
class info, and runs a minimal CLAP host that restores a state with a song and the mode at
Play, activates at 44.1 kHz, processes, and gets the same samples as the process side run
directly. Loading in a real host (Reaper) has not been tried by the tests. `sync.rs` (task-198)
runs a simulated host transport (section 8), `note_mode.rs` (task-200) note mode (section 9).

## 8. Host sync, pattern selection by notes, the Standard Mapping (task-198)

### 8.1 How the original follows an external position [C]

The DLL has a host-sync path of its own, for TopHat (ReWire's precursor; out of orb's scope
otherwise): the export `TopHatDriveAudioProc` (0x1001B800) calls `sub_1001C0F4` (0x1001B891)
with the host's block in each audio call.

- The host's tempo (`+0x950`) becomes the song's tempo (globals `+4`, 0x1001C118) and
  `transport+0x144`.
- A change of the host's play state (`+0x954`, 0x1001C17E) plays or halts the song through
  `sub_1001F7C0` (play `sub_1001F7E4`, or `sub_100218A0`: the stop without the bar locate).
- With the reposition flag `+0x958` set, `sub_1001C3C4(+0x94C)`, the host's tick: unless the
  countdown `transport+0x158` runs, a change of bar first merges every track's recording
  (`sub_1001FD10` → `sub_1001F520`), then the song is located there (`sub_1001F980` → the
  locate `sub_1001F984`, 0x1001C454).
- Otherwise `sub_1001C2F4(tick)`: with the host stopped (`transport+0x164` = 0) the countdown is
  cleared; with the host playing, loop on (`sub_100215D0`, globals byte 3) and the tick within
  192 ticks (`cmp eax, 0C0h`) of the loop start (`sub_1001F8C0`, globals `+0xC`) or the loop end
  (`sub_1001F8E0`, `+0x10`) it is set to 10. Then, while the countdown is 0, the position is set
  to the host's tick, `fild; fstp qword [song+8]` at 0x1001C38F (the store of `sub_1001F8B0`):
  no chase, no relocate, the last tick kept; else the countdown goes down by one.
- The MIDI Song Position Pointer also locates: `EngineHandleMIDI` (0x1001B277) turns it into
  `(msb << 7 | lsb) * 48` ticks and `sub_1001C5C4` locates the current song there when MIDI sync
  is on (`transport+0x160`, 0x1001C5EA).

So the original has no "jump to step": the position is set (`sub_1001F8B0`) each call, the
sequencer's clock (`sub_1001EF20`) then advances it and steps the devices from the absolute tick
(pattern mode: `(t mod 48*len) / 48`, `docs/initial_investigation.md` section 5), and a jump is a
locate.

### 8.2 What the plugin does

The engine gains two operations for it (`crates/orb-engine`; no existing path changes, the
corpus renders as before): `Sequencer::set_pos` (`sub_1001F8B0`'s store, but of a fractional
tick) and `Sequencer::relocate_device` (slot 2 on one device at the current tick, not a path of
the original's); orb-player sends them as `Command::Position` and `Command::Relocate`.

Each `process` call records the host's transport at its first frame (playing, tempo, position in
quarter notes). Engine block `m` belongs to host timeline frame `h = 64m * rate / 44100` (section
4: the host compensates the latency); before it renders, the tempo and position at `h` come from
the last record that starts at or before `h`, extrapolated at its tempo while the host plays
(`src/core.rs`, `Engine::sync`):

- **Tempo**: the host's, rounded to BPM*1000 (song msg 35 `{4, v}`, the TopHat path's store),
  even while the host is stopped (hosts report it then); the song's own tempo when the host
  reports none (the newest song's tempo when it was first seen: a loaded song brings its own).
  The snapshot's tempo is what orb plays at.
- **Position** (pattern mode, the mode at Play, and the host playing): the host's tick is
  `beats * 192` (bar 1 beat 1 is tick 0, step 1; quarter notes, whatever the host's time
  signature; other than 4/4 is task-206's decision). The first block that follows (the host starts, or the mode turns to Play while it
  plays) stops the song (song msg 35 `{1,0}`), locates it to `trunc(tick)` and plays it: the
  devices start clean at the host's position (slot 4, then slot 3). A later block whose tick is
  a tick or more away from the last one plus the clock's advance (`tempo * TEMPO_STEP`) is a seek
  or a loop jump: a locate, as the TopHat reposition. Then, in every block, the position is set
  to the host's fractional tick, and the clock adds the block's advance, so after the block it
  equals the host's position at the block's end. The TopHat path's loop-point hold-off (the
  countdown) is not taken: the host's loop is not the song's, and pattern mode has none.
- **Host stopped**: the song runs on from where it is (the clock alone).
- An offline bounce reads nothing but the calls' frames and transports: two renders of the same
  timeline are identical, and with a host whose positions do not depend on where its buffers
  start, so are different buffer sizes (`sync.rs`).

**Notes.** A note-on (velocity above 0) 0..31 on MIDI channel 1..4 (303 #1 = 1, 303 #2 = 2,
808 = 3, 909 = 4; a keyboard or DAW track on its default channel 1 reaches 303 #1) in pattern
mode selects pattern n+1 of that section (A1..D8): the pattern controller's panel message (it
becomes the pending pattern), then slot 2 on that device, which takes it at once
(`relocate`: `0x100255E0`, `0x10028030`, `0x1002B670`) where the panel's would wait for the bar
line; the next step is the new pattern's step at the same position. It latches until the next
note on that channel or a panel change (a parameter change: sent when the value changes, and
taken at the next bar line, as the original's panel); the last write wins. A note-off and a
note above 31 do nothing; in note mode (Stop) every note on these channels plays the device
(section 9). Notes never change
parameters: the parameter keeps the panel's pattern, which a state and every activation (a
bounce) start from.

**The Standard Mapping** (`docs/standard_midi_mapping.md`, orb-player's `midi::StandardMapping`)
is **off by default** (the original's Remote MIDI Control defaults to off too [G], 0x41C293,
`docs/midi.md`; `remote_enabled` in the state, no host parameter). When on, it listens on the remote channel,
MIDI channel 16 by default (`remote_channel`); any channel but 1..4 is allowed. A section's
channel (1..4) is refused by `set_remote_channel`; in a restored state it becomes channel 16
with `last_error` set, and the rest of the state (song, patterns, controls) is kept. Channels
5..16 never select patterns. While it is off, every message but the section notes is ignored
(it still counts for the MIDI LED).

**The main way to map a controller** is the host's MIDI learn of the plugin's parameters, not
the Standard Mapping: in Reaper the FX window's Param button > Learn (move the control, then
pick the parameter). The mode parameter (Play/Stop, `g1`) is a two-valued parameter, so it
learns as a play/stop button; the host's transport is the other way to start and stop.

The Standard Mapping's control actions (the devices' and the
mixer's knobs and switches, the pattern notes 12..63, the shuffle) are applied to the engine at
the message's frame, like a host note: they do not change the parameters, and the last write
wins. Its transport actions (Play, Stop, Record, the locates) are dropped: the mode is a host
parameter and the transport the host's. Its view actions (bank, focus, instrument, keyboard
mode) are dropped too until the editor (task-201) shows them. Its toggle state follows the
snapshot after every block.

**LEDs.** `Snapshot::host_sync` is set while a block followed the host's position (pattern mode,
orb and the host playing): the Sync LED. `Snapshot::midi_in` counts every MIDI message the host
sent (also those nothing maps): the MIDI LED.

## 9. Note mode (task-200)

With the mode at Stop the song is stopped, and a note-on or note-off on MIDI channel 1..4 (303
#1, 303 #2, 808, 909) plays that device by the note mapping of `docs/midi.md` section 5, at the
block holding the event (section 4), whether the host plays or not. The original plays no
device from a note (`docs/midi.md` section 4), so this is new engine behavior; it does not run
while the song plays, so pattern playback is unchanged (the corpus renders as before).

- **The path.** `src/core.rs` sends `Command::Note {device, note, velocity}` (velocity 0, or a
  note-off, is the note-off). If the mode has just turned to Stop and the song still plays,
  the Stop is sent first (`hold_mode`), so that its silence (slot 4 → slot 6) does not cut the
  note. orb-player applies it as `Engine::live_note`, which calls the channel device's
  `live_note_on` / `live_note_off` (`crates/orb-engine/src/frame/mod.rs`) and does nothing
  while the song plays. Notes change no parameter: the 808's voice switches, the Tune and the
  patterns stay as the panel and the host set them.
- **The section's On/off switch** (device controller 0) is not read: notes play on a section
  that is switched off (the switch is the pattern sequencer's; the mixer channel's On/off mutes
  in both modes). Provisional, task-210 decides.

### 9.1 The 303

`Tb303::key_on` / `key_off` (`crates/orb-engine/src/tb303.rs`) drive the step machinery's
note-on 0x10023CD4 and note-off 0x10023C74 as a pattern would, so the sound is the 303's:

- A note-on is the step note-on with the note's semitones `m - 36` in place of
  `note + 12*up - 12*down`, accent at velocity 100 and up, no rest; the Tune is added at the
  note-on as there (`docs/midi.md` 5.3). A note whose index `tuneTab[tune >> 2] + m - 36` is
  outside -48..47 is ignored (the table has no bound check in the DLL); at the Tune's center
  that is every note above 83.
- **Overlap = slide.** A note-on while another note is held first sets the slide flag `+0xCA1`,
  as a sliding step before it leaves it: no new attack (the envelope and the 2.0 sound's
  coefficient restart are skipped), and the pitch glides `(11p + t)/12`. The same pitch is a
  tie; its accent still changes the envelope decay and the accent amount, as a tied accented
  step does.
- **Held notes** (up to 16, a fixed array; a 17th forgets the oldest): the newest sounds
  (last-note priority). Releasing it slides back to the newest note still held, with that
  note's accent (legato; a held note the Tune has since put outside the table is dropped);
  releasing a note that does not sound changes nothing; releasing the last one is the note-off
  (the gate is the note's length; the release is the note-off's amp constants). A note-off
  releases the oldest held instance of its note, so two overlapping notes of one pitch (a tie
  in the pattern drag-out, task-203) sound as one until the second ends.
- Play (slot 3) and slot 6 (Stop, a song switch, a reset) forget the held notes.

The unit tests (`src/tb303/note_tests.rs`) play patterns through the step function and the same
notes through note mode, 12 ticks a block, and require the same samples bit for bit in both
sound models: plain notes with accent, octave up and down and three Tune settings, a slide
chain (an overlapping note let go after the next one starts), and a tie.

### 9.2 The 808 and the 909

`Tr808::key_on` and `Tr909::key_on` play the General MIDI note's instrument as a step with that
instrument alone would, through the same trigger code (`sub_10027894`; the 909's switches at
0x1002AD2D): the BD's two alternating voices (the process-global toggle), the 808's CH clearing
every SIMPLE voice's hold and the 909's CH setting the OH's multiplier to 0.996, the snare's
noise sequence counter, one voice per slot (a second hit restarts it). A note-off does
nothing. The accent is the step's expression with the note's AC byte:

- 808: `accent = f32((AC * knob) * 0.03125f + 1.0)` with AC = 1 at velocity 100 and up, else 0.
- 909: the band's step value and AC byte (`tr909::velocity_band`: 1..47 value 1, 48..95 value 2,
  96..111 value 1 + AC 2, 112..127 value 2 + AC 2), then `gain = f32(accent * 0.8)` or
  `f32(accent * 2.0)`. A note arms no flam.

A note on an 808 shared slot plays the sound it names through an override of the trigger's
switch read (`trigger_as`); the switch byte is not changed, and the step function passes no
override. The tests (`src/tr808/note_tests.rs`, `src/tr909/note_tests.rs`) compare every note's
voices (gain, multiplier, hold, sample) and the accent and gain with a one-instrument step at
each band's velocities.

### 9.3 The sounding mask (for task-202)

`ChannelDevice::sounding()` gives, after each block, a `Sounding {mask, ac}` for the 808 and
the 909 (empty for the 303s); orb-player's `Snapshot::sounding` holds the four channel devices'.
Bit `n` is sound `n` of `tr808::sound` / `tr909::sound`: the instrument column minus 1 (the 808's
shared slots by their first sound: LT, MT, HT, RS, CP), then for the 808 LC, MC, HC, CL, MA at
11..15.

- `mask`: the sound's voice is active and its peak for a full-scale sample is at least
  `AUDIBLE` = 0.001 (-60 dB): `g * 2^15` for a SIMPLE or PITCHED voice, `g1 * (1 + g2) * 2^15`
  for the snare's DUAL voice, `g * 2^15 / (1 - c)` for a FILTER voice (its one-pole's DC gain).
  An 808 shared slot's voice lights the sound it last played; the BD is lit while either of its
  two voices sounds. So a CH choke darkens the OH as its gain falls, and a level of 0 lights
  nothing. The threshold is close to the voices' own end test (the original's 3e-8 gain test,
  `docs/initial_investigation.md` section 7.1): `3e-8 * 2^15` = 9.8e-4 = -60.2 dB of full
  scale, so a decaying SIMPLE or FILTER voice goes dark about when it ends.
- `ac`: bit `n` set when sound `n`'s last hit had accent (a step with a non-zero AC byte, or a
  note in an AC band), in pattern mode too. The panel's AC light is "any lit sound whose bit is
  set": `mask & ac != 0`.

The minimum on-time and the drawing are the editor's (task-202).
