# The VST® 3 plugin (`crates/orb-plugin`)

VST is a registered trademark of Steinberg Media Technologies GmbH.

**CLAP was dropped** (the user's decision of 2026-11-22, task-278). The plugin ships as VST3
only, from the release after v2026.1118.0919 (the last with a CLAP bundle). The plugin uses no
CLAP-only feature, and the CLAP format cost a forked nice-plug (task-275, task-276: its wrapper
gave a continuous `FloatParam` the CLAP range 0..1, so a patched copy kept the CLAP range 0..127
of the released stepped parameters) and half the bundle tests. The workspace takes nice-plug
0.4.2 from crates.io again. Sections 3 to 10 mention CLAP where they report what was found with
the CLAP bundle (REAPER, Bitwig); that is history. What a user of the CLAP version does is in
section 7.2: delete the old `OpenRB-338.clap` (the Windows installer and the macOS plugin
installer do it on an upgrade) and replace the plugin by its VST3 by hand in each project.

OpenRB-338 as an instrument plugin, milestone m-7. The product decisions are task-185's final
summary (2026-11-15/16); this document describes how the plugin implements them. task-197 built
the core described here; task-198 the host sync, the pattern selection by notes and the Standard
Mapping (section 8); task-200 note mode (section 9); task-201 the editor (section 10); task-203
the pattern drag-out (section 12); packaging and the validators are task-204; task-256 the
per-device outputs (section 13). task-217 (the
user's design of 2026-11-16, task-216) replaced the mode parameter `g1` with the pattern/note
mode setting and made the panel's Play/Stop the pattern preview (section 4.2). task-218 renamed
pattern mode Latch Pattern Mode and added Gate Pattern Mode (section 11).

## 1. What runs

One instance is the whole ReBirth: the two 303s, the 808, the 909, the effects and the mixer,
the engine of `orb-engine` driven as orb-player drives it in the standalone app. The plugin is
built on nice-plug 0.4.2 (codeberg.org/RustAudio/nice-plug, ISC), the community fork of
nih-plug, with its `vst3` and `editor` features (no logger). Its VST3 bindings are the `vst3`
crate 0.3.0 (MIT OR Apache-2.0); the editor is on nice-plug-egui 0.6.0 (ISC) and egui-baseview
0.9 with wgpu 30, egui-baseview from the user's fork that frees textures after the submit
(MIT OR Apache-2.0, as upstream; section 10). These licenses are compatible with the
repository's GPL-3.0-only.

| File | Thread | What |
|---|---|---|
| `src/lib.rs` | both | the nice-plug `Plugin` and `Vst3Plugin`, the export |
| `src/instance.rs` | main | the parameters, the document the state saves, the editor's calls, the engine build |
| `src/core.rs` | audio | the 64-frame block scheduler, the parameter changes, the mode and the preview |
| `src/resample.rs` | audio | the resampler for host rates other than 44.1 kHz |
| `src/params.rs` | both | the parameter table (a constant) |
| `src/editor.rs` | main | the editor: orb-ui's panel in the host's window (section 10) |
| `src/plugin_log.rs` | main, the editor's | the diagnostic log (`ORB_PLUGIN_LOG`, section 10), whose activation lines name the library that runs (section 7.1), and the editor's wgpu adapter |

The process side reuses orb-player's lock-free split (`crates/orb-player/src/lib.rs`): the
UI half (`Player`) stays on the main thread behind a mutex that `process` never takes, the
callback half (`Audio`) is the process side's. `Player::with_call_frames(.., 64)` makes calls of
one engine block (the standalone app keeps its 1024-frame calls, `CALL_FRAMES`); the plugin
renders through `Audio::drain`, `Audio::apply_now` and `Audio::render_call`. Commands, songs and
mods travel in orb-player's command queue, and what the audio thread lets go of goes back in
its garbage queue, so `process` allocates and frees nothing.

**When the garbage queue is full (task-225).** The garbage queue (`GARBAGE` = 64 slots) fills
when the main thread sends commands that hand a box back (a pattern edit, a data block, a song,
a mod, a preview) faster than it collects them (`OrbParams::collect_garbage`, at every
main-thread command and every editor frame; the editor may be closed). orb-player then holds
back only what does not fit, `Audio::room_for`:
- a command that carries no box and replaces none lets go of nothing and is always applied:
  everything `process` sends for the host (the parameters' messages, the Standard Mapping's
  controls, notes, gates, pattern selections, Play, Stop, Locate, Position, Tempo);
- `Play` while a song loaded over the playing one is pending lets go of the current song: it
  needs one slot, and that slot is kept free for it, since a command with a box takes
  `GARBAGE_PER_COMMAND` (2) free slots plus one more while a song is pending;
- the main thread's commands are applied in their order at the start of each block
  (`Audio::drain`) as long as the next one fits: the first box command that does not fit
  waits, with everything queued after it, until the main thread collects.

So no host event is lost or delayed by a full queue: it reaches the block at or after its
frame as usual. Host events are not ordered after the main-thread commands still queued: while
a box command waits, the box-free commands queued behind it (the editor's pattern selection,
the panel's Play or Stop) wait too, and a host event applied in that time comes before them.
The window lasts until the main thread collects, about one editor frame (or the next
main-thread command). `Core::dropped_events` counts the commands `Audio::apply_now` refused, which can
only be one with a box; `process` sends none, so it stays 0 (`tests/full_garbage.rs`: 128
uncollected edits, a Latch note, note-mode notes and the host's start and stop, frame for frame
equal to an instance that collects). Before task-225 any command was refused with fewer than
two free slots and only counted: a Latch note's pattern, a note-mode note-on or note-off, the
host's start or stop and a parameter change could be lost (task-218's gates alone were resent).

## 2. Identity

Permanent after the first release (task-185):

| | |
|---|---|
| Name, vendor | `OpenRB-338`, `OpenRB-338` |
| VST3 class id | `EFAB08AF-B39D-4399-8A80-3DC1F84ED3D6` (made once with `uuidgen`, 2026-11-16; `VST3_CLASS_ID` in `src/lib.rs`); category Instrument\|Synth |
| URL | https://github.com/pingw33n/OpenRB-338 |
| Ports | stereo out, no audio in, one MIDI input (`MidiConfig::MidiCCs`: notes and control changes, section 8) |

`tests/exports.rs` loads the built library and checks them through the VST3 C API.

**Version (task-291).** `Plugin::VERSION`, which the VST3 class info's `version` and the state's
`version` carry, is orb-ui's `about::VERSION`: exactly the About box's string
`YYYY.MMDD.HHMM (<[dev-]hash[-dirty]>)` (set by `crates/orb-ui/build.rs`, `src/version.rs`;
docs/building.md), not Cargo's `0.1.0`. Nothing compares the state's `version` (nice-plug only
writes it; `STATE_VERSION` in `instance.rs` is the plugin's own, separate), so a state of a
build that reported `0.1.0` loads unchanged (`tests/continuous.rs`).

## 3. Parameters

One parameter per control the plugin's panel shows (119; continuous or stepped, below): every controller of
the 303s (0..8 each: on/off, pattern, tune, cutoff, resonance, env mod, decay, accent,
waveform), the 808 (0..28, the voice switches 11, 14, 17, 19, 21 included), the 909 (0..29),
the delay (0..4), the distortion (0..2), the PCF (0..6) and the compressor (0..2); the mixer's
master, compressor and PCF inserts, and per channel on, level, pan, delay send and distortion;
and the shuffle. Hidden in the plugin and not parameters: the
pattern/song switch, the loop switch and range, Rewind, Record, the tempo buttons (task-185).
Not parameters either since task-217: orb's Play/Stop (the preview, section 4.2) and the
pattern/note mode (a setting of the state, section 5), so automation cannot change them; the
mode parameter `g1` of task-197 is gone, and states saved with it are not migrated (the user,
2026-11-16: no compatibility with plugin states before the first release).
The pattern edits (steps, a pattern's length and shuffle flag) are pattern data, kept in the
state (section 5), not parameters.

- **Ids.** `g20` the shuffle (its song msg 35 id, `docs/song_load.md` section 3; `g1` is
  unused since task-217); `d<device>c<n>` the others, with the device id (0 mixer, 1 and 2 the 303s, 3 the
  808, 4 the 909, 5 delay, 6 distortion, 7 PCF, 8 compressor) and the controller number
  (`docs/song_load.md` section 5; the 909's in RBS order). For the mixer `n` is the byte's
  offset in its msg-41 block (section 4.1: 0 master, 1 compressor id, 2 PCF id, `5 + 6c + k`
  channel c's field k, 0 on .. 4 distortion), which equals the controller number where the byte
  has one (`6 + 6c` .. `9 + 6c`). `crates/orb-plugin/tests/param_ids.txt` lists every id with
  its target, range and default; `tests/params.rs` fails when the list changes.
- **Ranges.** 0..127 for the knobs and faders; 0..1 for the switches; 0..31 the patterns (shown
  A1..D8), 1..32 the delay steps, 0..54 the PCF wave, 0..5 the compressor insert (off, master,
  channel 1..4), 0..4 the PCF insert (off, channel 1..4).
- **Continuous and stepped** (task-275, GitHub issue 21: Studio One 7 drew and played the
  automation of a stepped parameter, "ORB-303 2 Cutoff", as a staircase and would not draw a
  ramp; REAPER and Ableton do not, which is why no test had caught it). Until task-275 every
  parameter was a nice-plug `IntParam`, which gives VST3 a `stepCount` of `max - min`.
  - **Continuous** (86): every value over 0..127 (`params::Spec::continuous`: `Kind::Value`,
    0..=127): the shuffle, the master level, the four channels' level, pan and delay send, the
    303s' tune, cutoff, resonance, env mod, decay and accent, the 808's and 909's knobs and
    levels (controllers 2.. other than the 808's voice switches), the delay's feedback and pan,
    the distortion's amount and shape, the PCF's frequency, resonance, amount and decay, the
    compressor's ratio and threshold. Each is a nice-plug `FloatParam`, linear over 0..127, with
    no step size: VST3 `stepCount` 0.
  - **Stepped** (33), `IntParam`s as before: the switches (`Kind::Switch`: the devices' and
    channels' on, the 303s' waveform, the 808's voice switches, the delay's sixteenths, the
    distortion switches of the channels; `Kind::PcfMode`), the patterns 0..31, the delay steps
    1..32, the PCF wave 0..54, the compressor insert 0..5 and the PCF insert 0..4.
  - **Into integers.** The engine, the panel, the display and `OrbParams::values` get a
    continuous parameter's host value rounded to the nearest integer, a half away from zero
    (`f32::round`), clamped to the range (NaN: the minimum; `params::round`,
    `instance::HostParam::value`). The process side compares those integers with the ones of
    the call before (`Core::apply`), so a host's ramp sends an engine message only when the
    rounded integer changes, one per integer crossed (`tests/continuous.rs`). The text is the
    integer's (`params::display` of the rounded value; `params::parse` gives the integer).
  - **The same meaning as the released build's** (v2026.1118.0919, all `IntParam`s): the ids
    are unchanged; VST3's normalized value of an integer `v` is `(v - min) / (max - min)` in
    f32 for both (nice-plug's `IntRange::Linear` and `FloatRange::Linear::normalize`, bit for
    bit with min 0), and a host's normalized value `n` gives the integer the released build
    made of it (`round(n * 127)`); stock nice-plug 0.4.2 does this for VST3 (its VST3 wrapper
    works in normalized values; its CLAP wrapper, no longer built, would have given the CLAP
    range 0..1). A plugin state of the released build
    restores (section 5). Checked in REAPER 7.86 with the released bundles (task-275's notes):
    a project REAPER saved through the released build, with an envelope ramp on "ORB-303 1
    Cutoff" from 0 to its maximum, renders bit for bit the same through the new build, VST3
    (REAPER saves the envelope normalized, `<PARMENV 27:3029309 0 1 0.5`; 3 renders with each
    build, all 9 pairs equal). (CLAP, history: REAPER saved the CLAP value, `<PARMENV
    27:3029309 0 127 63.5`, and its CLAP render of that project was not repeatable.)
  - **The panel** sets a continuous parameter through `ParamSetter` to the float of its
    integer, and only when the parameter's rounded value differs (`editor::set_param`).
- **Defaults** are the song constructor's values (`docs/song_load.md` section 4.9; the shuffle
  64, the sequencer's ctor value): a new instance's parameters change nothing in a new song
  (`the_defaults_are_the_new_songs_values`).
- **Into the engine.** A changed value is sent as the panel's message
  (`docs/song_load.md` section 4.10): `{ctrl, v}` to the device (`Command::DeviceParam`), mixer
  msg 145 `{id, v}` or 147 `{c, k, v}`, song msg 35 `{20, v}` for the shuffle. During the
  preview a change is only kept, and sent when the preview ends (section 4.2).
- **A song's out-of-range bytes.** A song can hold a byte outside the control's range (a 303
  waveform of 127, task-156). Its parameter shows what the engine plays (a switch is on for any
  byte but 0, the PCF's mode is low-pass only at 1, a number is clamped), and a value is sent
  only when the engine does not already play it, so loading or restoring does not rewrite such a
  byte until the control is moved.

## 4. Timing

The engine runs only at 44.1 kHz, in blocks of 64 frames; the host's buffers can be any size.

- **One block ahead.** Host frame `j` (counted from the activation) plays engine frame
  `j - 64`; the first 64 frames after an activation are silence, and the plugin reports 64
  frames of latency. A block is rendered when its first frame is due, or earlier when an event
  after its start comes first (below).
- **Events.** A host event at frame `t` (a parameter change; with
  `SAMPLE_ACCURATE_AUTOMATION` nice-plug splits the host's buffer at each change, so a change is
  always at a call's first frame; a MIDI message) reaches the first block that starts at or
  after `t`, never one before it (the user's decision of task-227, 2026-11-16): after the
  host's latency compensation block `m` plays at host frames `[64m, 64m + 64)`, and an event
  at `t` is applied before the first block with `64m >= t`, so it sounds 0 to 63 frames late
  and never early. Before each event the plugin renders, ahead of their time, the blocks that
  start before it (at 44.1 kHz at most one beyond the one being played; resampled, the
  blocks whose first frame plays at or before host frame `t - 1` after the compensation:
  engine frame `b` with `(b + 64 + W) * den <= (t - 1 + latency) * num`, which allows the
  half frame the rounded-up latency adds). The rule was the reverse before (the block that
  held `t`, up to 63 frames early): at a realtime start off the block grid the early part of
  a note fell before the play position, where the host drops it (task-226 round 3: a 909
  bass drum at frame 0 after 7777 stopped frames came 33 frames early). The output is a
  function of the events' frames alone: buffer sizes 1, 63, 64, 512 and 4096 give the same
  samples (`tests/timing.rs`); not when a block starts exactly on a step's tick while the
  plugin follows the host's position (section 7.1, task-263). A call of no frames takes no changes; they are seen at the next
  call.
- **The mode** (task-217, task-218; a setting of the state, not a parameter, section 5) is
  read before each block and at each section note. In **Latch Pattern Mode** (task-217's
  pattern mode, renamed) the song plays (song msg 35 `{1,1}`) only while the host plays, its
  step following the host's position (section 8), and is stopped (`{1,0}`, which locates to the
  bar start) when the host stops: no sound with the host stopped. **Gate Pattern Mode**
  follows the host the same way, but a section sounds only while a note on its channel is
  held, and a held note also plays with the host stopped (section 11). In **note mode** the
  song is stopped and host notes play the devices whatever the host's transport does (section
  9). The default is note mode: a plugin just inserted makes no sound of its own, as the old
  mode parameter's Stop default (the original starts stopped). File > Open and File > New
  keep the mode (task-228; File > Open used to turn note mode into Latch Pattern Mode). The preview (section
  4.2) overrides all three while it runs. A song loaded over the playing one waits (pending)
  until the next block, which switches to it (Play) and then plays or stops it as the mode
  says.
- **Activation.** Every activation makes a new player and builds the engine from the state and
  the parameters (section 5), so an offline bounce does not depend on what played before it and
  reads no clock. The UI side's commands are applied at the start of the next block, as the
  standalone player's are at the start of its call.
- **The activation's song is in place before the first call** (task-226, `core::settle`, on the
  main thread in `Core::new`): the player's queue (the activation's `Command::Load`) is drained
  there, and 256 silent blocks are rendered and dropped. Before, the load was applied by the
  first block's drain, 64 host frames after the activation (more when resampled), so a host
  event of those frames went to the player's placeholder song and was lost (a note at position
  0 of an offline render, for every device; a parameter change, the cause of task-214). And a
  new song's mixer gains start at 1e-6 and reach their targets through the per-block
  smoothing `g = f32(g*0.93 + t*0.07)` (0x100136B7) after the first mixer update (block 6), a
  fade-in of about 50 ms that made a drum note of the first blocks softer (an 808 bass drum's
  peak 0.097 against 0.135). 256 blocks (`0.93^230 < 2^-24`, and a multiple of the update's 16-block
  period) reach the gains' f32 fixed point; the song is stopped at the activation, so the
  settle plays nothing (state that runs free in silence, such as the 303s' voices, moves on as
  it would in any silent blocks). A note at frame 0 of the first call now sounds
  as the same note later (`tests/first_note.rs`; through the VST3 wrapper and, until task-278, the CLAP one,
  `tests/exports.rs`): for the drums at 44.1 kHz within one LSB, as two later notes differ
  from each other (engine state that runs on in silence shifts the int16 truncation of a
  sample now and then; the 303s' voices, for one, run free, their amp adding 1e-6 every block
  at 0x1002494C; which state it is was not isolated), and for the 303s as loud, their waveform
  depending on when the note starts (two later 303 notes differ by up to 0.33).
- **The same engine as the app.** Without changes the plugin's output equals orb-player's for the
  same song in pattern mode, 64 frames later, with a host that plays and reports no tempo or
  position, once the player has settled the same way
  (`the_plugin_plays_as_the_player_one_block_later`). The app's own song load keeps the
  fade-in, as the original's does.

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

### 4.2 The preview (task-217)

The panel's Play enters the **pattern preview** and its Stop leaves it (`OrbParams::set_preview`,
section 6). The preview is the standalone app's pattern mode inside the plugin, a stable state
for editing patterns:

- It plays the patterns free-running, whatever the host's transport does (stopped, playing,
  jumping) and in either mode: its step does not follow the host's position. On entry the
  song plays from where it is (a stopped song from its bar start; a song that followed the
  host plays on from its position). The tempo is the host's, also while the host is stopped
  (hosts report it then), and the song's only when the host reports none (the user's Reaper
  check, 2026-11-16; the first build played the song's tempo); the tempo display shows it. No
  host sync: the Sync LED and the Bar display are dark.
- Host input is ignored: notes on every channel, the Standard Mapping, parameter changes and
  automation change neither the sound nor the panel. The parameters' values still follow the
  host in the plugin's model (the process side keeps the latest), and the MIDI LED still blinks
  on every message.
- **The preview set.** The controls the panel changes during the preview (knobs, switches,
  pattern selections) go into a set of their own, by parameter id, kept in the state
  (`preview`, section 5), and not to the host: no begin/set/end gesture. The first entry starts
  the set from the parameters' values; later entries restore it; a value is clamped to its
  parameter's range. The main thread writes the set into atomics (`Shared`), and the process
  side applies the set on entry and its changes before each block (no lock, no allocation).
- Leaving the preview applies every parameter's latest value where the engine does not
  already hold it, so the panel and the engine are the host's again (a pattern the set had
  selected is replaced at the next bar line, or at once by the clean start when the host
  plays in pattern mode); then the mode rules again.
- Pattern step edits (pattern data, section 5) are the same in and out of the preview.
- File > Open and File > New during the preview load the song, set its values as parameters
  (through the host, as always) and make them the preview set, which the preview plays at
  once; outside the preview a load forgets the set (the next entry starts from the new song's
  values).
- The preview is not saved: a state restore ends it. It survives a re-activation (the first
  block enters it again).

## 5. State

nice-plug saves the parameters (the controls) by id. The rest is one persisted
field, `orb`, a JSON object (version 1):

| Key | What |
|---|---|
| `version` | 1 |
| `song` | the song file as loaded (`.rbs` or a ReBirth 1.x song), base64; null for a new song. File > Info reads its tempo and text from it |
| `patterns` | the four pattern arrays as edited (303 #1 and #2: 32 x 98 bytes; 808 and 909: 32 x 194), base64 |
| `vintage` | the Vintage sound model |
| `mod` | null for the built-in kit (never embedded); else `{name, sha256, data}`: the mod's INFO name, the SHA-256 of its bytes and the recrypted `.rbm`, base64 (the mods are 2.6 to 9.4 MB, so about 3.5 to 12.5 MB of text) |
| `editor_size` | the editor window's size in logical points (section 10); absent: 656 x 532 (624 x 532 before task-234's side margins) |
| `remote_enabled` | whether the Standard Mapping (Remote MIDI) is on; false by default and when absent (section 8) |
| `remote_channel` | the Standard Mapping's MIDI channel, 0-based (15: channel 16, the default and when absent); a section's channel (0..3, MIDI 1..4) restores as 15 with `last_error` set, the rest of the state kept (section 8) |
| `mode` | `"latch"`, `"gate"` or `"note"` (section 4; a new instance: note; task-217's `"pattern"` is no longer read) |
| `preview` | the preview set (section 4.2): an object of parameter id to value, or null before the first preview; an id it lacks takes the parameter's value |

Until the first release (v2026.1118.0919) no state of an earlier build was migrated,
recognized or tested (the user, 2026-11-16: no compatibility work with old plugin states until
the plugin is released on GitHub Releases). A state from before task-217 lacks `mode` and does
not parse (the restore gives a new song and `last_error`).

**The parameters in the state** (nice-plug's `PluginState.params`, by id) are nice-plug's
externally tagged values: `{"i32": v}` for a stepped parameter, `{"f32": x}` for a continuous
one (section 3; `x` the host's value, which may lie between two integers). The released build
saved every parameter as `{"i32": v}`, and nice-plug restores only an `F32` into a `FloatParam`
(`deserialize_object`, `src/wrapper/state.rs`: a mismatched pair is skipped, so the parameter
would keep its default). The plugin's `Plugin::filter_state`, which nice-plug calls before the
restore, turns a continuous id's `I32` into the `F32` of the same value
(`instance::migrate_state`; task-275), so a released build's state restores the same values
(`tests/continuous.rs`, and through the VST3 export `tests/exports.rs`). A state of this build
loaded by the released build loses its continuous values (they keep their defaults): no
downgrade.

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

### 5.1 The logo animation (task-219)

The panel's logo animation (the standalone app's startup animation) plays at most once per
instance: at the first editor open, and only if no state was ever loaded into the instance.
`OrbParams` holds two runtime flags, saved nowhere: `restored`, set by every state load
(`deserialize_fields`: CLAP `state.load`, VST3 `setState`; so a reopened project, a duplicated
or pasted FX or track, undo/redo and a loaded preset all count), and `intro_played`, set when
an editor asks `OrbParams::take_intro` (the editor's build). A restored instance never plays
it, and neither does a later editor open of an instance that played it. The editor calls
`App::enable_logo_anim` only when `take_intro` is true; the standalone app is unchanged.

Caveat: a host that loads a default state on insertion makes the new instance count as
restored, so it never animates. Reaper does this when the user has saved a default preset for
the plugin.

Residual ordering risk: a host that opens the editor before its `state.load` on a project
reopen would play the animation once for an instance that is then restored; the flags cannot
know a load that has not happened yet.

## 6. The main thread's calls

For the editor (task-201), on `OrbParams` (`OrbPlugin::orb_params`):

- `load_song(bytes)` (File > Open) and `new_song()` (File > New): the song, with its own patterns
  and controls and the instance's mod, is built and sent (it plays at once in the preview, or
  in pattern mode while the host plays). During the preview its values become the preview set;
  outside it the set is forgotten (section 4.2). It returns the song's control values by parameter id; the caller sets them through the
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
- `mode()`, `set_mode(m)` (task-217): pattern or note mode, kept in the state (`mode`); the
  process side reads it before each block (and at each note).
- `previewing()`, `set_preview(on)`: the panel's Play and Stop (section 4.2); not saved.
- `set_preview_value(i, v)`: a control the panel changed during the preview, into the preview
  set (refused outside the preview); `preview_values()` the set's values (`None` before the
  first preview); `panel_values()` the values the panel and the engine play (the set's during
  the preview, else the parameters').

## 7. Building and testing

`cargo xtask bundle orb-plugin --release` builds the library and lays out `OpenRB-338.vst3`
under `target/bundled/` (`xtask/` on nice-plug-xtask 0.1.1, `bundler.toml`
for the bundle name, the alias in `.cargo/config.toml`). The xtask finds the workspace from its
own manifest: nice-plug-xtask's own entry point takes the topmost ancestor directory with a
`Cargo.toml` as the workspace, which is the wrong one for a checkout inside another (the
worktrees under `.claude/worktrees/`). Signing, the packages and the validators
(pluginval on the VST3 bundle) are task-204: docs/building.md,
"The plugin".

The tests (`cargo test -p orb-plugin`; `tests/` and the resampler's unit tests): the buffer
sizes and the 48 kHz path (`timing.rs`), the parameter list and defaults (`params.rs`), the
continuous parameters, their rounding and the released build's mapping and states
(`continuous.rs`, task-275; their VST3 step counts and normalized values through the export in
`exports.rs`), the state
(`state.rs`, with a test mod made from the standard mod: renamed, the 808's and 909's bass drum
and first snare swapped), no allocation in `process` (`no_alloc.rs`, a counting allocator, and
the process side's sources checked for lock types), and the exports (`exports.rs`): it builds
the library, loads it with the dynamic loader, checks the VST3 factory's class info and
plays a note at frame 0 of the first playing block (task-226) through the VST3 C API. A real host, REAPER, is the opt-in
`reaper.rs` (task-244, section 7.1). `sync.rs` (task-198) runs a simulated host transport
(section 8), `note_mode.rs` (task-200) note mode (section 9), `held_notes.rs` (task-258) the
303's held notes with lost and stray note-offs, CC 123/120 and the host's stop (section 9.1),
`editor.rs` (task-201) the editor headless (section 10), `preview.rs` (task-217) the preview
and its set through the process path (section 4.2).

### 7.1 The REAPER render test (task-244)

`cargo test -p orb-plugin --test reaper` renders committed REAPER projects through REAPER's
command line and compares the WAVs with the plugin's process side driven in-process as REAPER
drives it. Pinned to **REAPER 7.86** (7.86.0_f6d0464u, macOS arm64); the RPP format, the
`reaper.ini` keys and the command line differ between versions.

- **Opt-in.** The render test runs only with `ORB_REAPER` set to the REAPER executable
  (`/Applications/REAPER.app/Contents/MacOS/REAPER`); otherwise it prints `skipped: ...` and
  passes, as the `ORB_CD_ISO` tests do. CI does not run it (REAPER is licensed software), and
  it knows REAPER's settings for macOS only (elsewhere it skips with a message). It takes about
  a minute: 11 renders of a few seconds each and the in-process references. The fixtures' freshness
  test (`the_fixtures_are_current`) always runs.
- **The fresh build only, nothing installed** (the user's decision, 2026-11-21). The test
  builds the bundles (`cargo xtask bundle orb-plugin --release`, into `target/bundled`) and
  runs each render (through `open`, see "Hidden" below) as `REAPER -newinst -nosplash -ignoreerrors -cfgfile
  target/reaper/<case>.<format>/resources/reaper.ini -renderproject <copy of the fixture>`:
  - `-newinst` starts an instance of its own: a REAPER the user has open is neither used nor
    disturbed (without it the render could go to the running instance).
  - `-cfgfile` moves REAPER's whole resource folder to the ini's folder: the plugin caches
    (`reaper-vstplugins_arm64.ini`), `reaper-fxtags.ini`,
    `Scripts/` and the rest are written there, so the user's REAPER configuration and caches
    are never read or written and no other resource folder is needed.
  - The scratch `reaper.ini`: `vstpath_arm64` (on x86_64 `vstpath64`; the key is a string of
    the executable) lists only
    `target/bundled`. `vstfullstate=33605`: without it REAPER's first start appends
    `/Library/Audio/Plug-Ins/VST3;~/Library/Audio/Plug-Ins/VST3` to the VST path and scans
    them (seen on an exploratory first try, which read the user's installed copies into its
    scratch cache; nothing was written outside it). So the protection is these path settings
    plus the test's check, after every render, that the scratch `reaper.ini` still lists only
    `target/bundled` in both keys: the test fails if REAPER added its default folders.
    `coreaudiobs=512`, `coreaudioindevnew=<none>`, `coreaudiooutdevnew=<none>`: REAPER's
    dummy audio device; without them a new resource folder stops at the modal "You have not
    yet selected an audio device" (in REAPER's arm64 code, `OSXhasAudioBeenConfigured`
    0x1006EF298 wants `coreaudiobs` and both device names, `audioConfigInit` 0x1006EF33C reads
    them, and the streamer choice 0x1006EF538 takes `audioStreamer_dummy` when both are
    `<none>`). The evaluation license's nag does not stop `-renderproject`.
  - **Which copy ran.** The test sets `ORB_PLUGIN_LOG` (section 10) to a scratch file, and the
    plugin log has a line per activation, `activate: loaded from <path> at <rate> Hz`, with the
    path of the library that runs (`dladdr` of the plugin's own code;
    `plugin_log::activated`, `parse_loaded_from`; task-244's `ORB_PLUGIN_LOAD_LOG` was folded
    into it by task-260). The test fails unless every activation line names `target/bundled/OpenRB-338.<format>/
    Contents/MacOS/OpenRB-338` at the project's rate, REAPER kept the two paths, and each
    scratch cache lists one OpenRB-338.
  - **Hidden, no focus change** (task-277, macOS only; the test is macOS-only anyway, and
    nothing here is done for Windows, where the test skips). The test starts REAPER through
    `open -n -g -j -W -a REAPER.app --env ORB_PLUGIN_LOG=<log> --args <the arguments above>`:
    `-n` a new instance, `-g` not activated, `-j` hidden, `-W` waits for exit, `--env` because
    `open` does not pass the test's environment. Measured on REAPER 7.86 (arm64, macOS 15.6)
    with a 20 ms sampler of the on-screen windows (CGWindowList) and the front app
    (NSWorkspace) while REAPER renders `latch_48k`: started directly, REAPER showed its main
    window and the render dialog in 52 of 155 samples; through `open -g -j` still in 15 of
    117, so **`-j` does not keep the windows hidden** (REAPER unhides itself when it opens
    them; the front app stayed the user's in all runs, so `-g` works). The scratch `reaper.ini`
    with `wnd_x`/`wnd_y` far off-screen was no fallback: macOS clamps the window back
    (`wnd_x=68`, `wnd_y=0` were written back). So the test also runs a hider, a JXA loop of
    `osascript` (started before REAPER, as its start-up takes 0.3 s) that waits for the pid on
    its standard input and then calls `NSRunningApplication.hide` whenever the app is not
    hidden (every 5 ms, until the process is gone; no Accessibility permission needed, unlike
    System Events). The hider keeps looping while the lookup is nil (REAPER not yet known to
    NSWorkspace) and ends when the app is terminated, or gone after it was seen. With it,
    over the whole test (11 renders plus the hang test, about 2100 samples in 55 s) REAPER was
    never the front app in any of 12 runs, and no REAPER window was on screen in 8 of them;
    the other 4 saw windows in 14, 58, 59 and 131 samples (the last: 3 s in one render). So
    it is a race, not a guarantee, and a window can show for a few hundred ms. The 3 s case
    was not reproduced in 4 later runs with the hider logging (each: the hider had the pid
    and saw REAPER within about 70 ms, and the 48 hiders hid REAPER 160 to 210 times).
  - **Lifecycle.** `Started` (a `Drop` guard) stops the hider, `open` and the REAPERs found
    by the scratch `-cfgfile` on every exit, a panic included, and after a timeout looks again
    three times over 1 s for a REAPER LaunchServices starts late. A Ctrl-C kills the test
    process without running it, and REAPER (started by launchd) survives. Clean up by the
    scratch path, never by name. The scratch folder is `<target dir>/reaper/<case>.vst3/`
    (the target dir is `CARGO_TARGET_DIR` if set, else the workspace's `target`; the test
    prints it in a failure, and a REAPER's command line shows it). List the processes with a
    fixed-string match (not `pkill -f`, whose pattern is a regex, so a path with regex
    characters would need escaping): `ps -axww -o pid=,command= | grep -F -- '-cfgfile
    <target dir>/reaper/'`, check that each line is a REAPER of the test (or its `open`), and
    `kill -KILL <pid>` them. The hider is an `osascript -l JavaScript -e ObjC.import(...)`
    process (its command line has no path: `grep -F 'ObjC.import(' ` on the same list); it ends by itself once its REAPER is gone. The plugin
    variables `ORB_PLUGIN_LOG_LEVEL` and `ORB_PLUGIN_SIZE_LOG` are passed on by `--env` when
    set (checked with several `--env` flags: all reach REAPER, task-277 notes).
  - **The exit status.** `open -W` exits 0 whatever REAPER's status, and REAPER's own is not
    available, so a render counts only when `project/out/Master.wav` exists, and the plugin-log
    and length checks below follow as before.
  - **The timeout.** A render that does not finish in 120 s (a dialog) fails the test. The
    child is `open`, not REAPER, so the test finds the REAPER it started by its command line in
    `ps`: the exact pair `-cfgfile <its scratch reaper.ini> -renderproject ` on a process that
    is an app's executable (not `open`, which carries the same arguments), and kills only it
    (`reapers_of`, unit-tested on sample `ps` lines with another REAPER, `open`, and a path
    that only ends like the scratch path; `a_hung_reaper_is_killed_and_fails_the_run` hangs a
    real REAPER with an unreadable project for 15 s, and checks that it fails and that no
    process of that scratch configuration is left). A REAPER on any other configuration, the
    user's included, never matches.
- **Fixtures.** `crates/orb-plugin/tests/reaper/<case>.vst3.RPP`, made by `cargo run -p orb-plugin --example reaper_fixtures` from
  `tests/reaper/template.RPP` and the cases of `tests/reaper/cases.rs`. Rerun it when the
  state format, a case or the template changes; the freshness test fails on a stale fixture.
  A fixture holds the plugin's block as REAPER 7.86 saves it (from a project REAPER saved
  through a ReaScript): for VST3 a header line (the class's REAPER id 1476801187 as a
  little-endian u32, 0xFEED5EEE, 0 inputs, 10 outputs and their u64 pin masks), a line of
  (data size, 1, 0xFFFF), the data (the component state's size and 1, nice-plug's JSON state,
  then 8 zero bytes) and 6 zero bytes; base64 lines of 96 bytes. (The CLAP fixtures went with
  the format, task-278.)
  The state's JSON holds the song, the patterns, the controls and the mode (section 5). A
  MIDI item counts in frames (its division is a quarter note's frames), so a note plays at the
  frame the case gives it. Renders are 32-bit float WAVs (`RENDER_CFG` `ZXZhdyAAAA==`);
  `RENDER_STEMS 1` is the master mix and a stem per selected track.
- **REAPER drives the plugin as the in-process host.** At 44.1 kHz REAPER's frame `j` is the
  plugin's output frame `j + 64` (it compensates the reported latency), at 48 kHz `j + 105`;
  with the in-process host rolling from beat 0 at the project's tempo and the item's notes at
  their frames, the master render (the Mix) of every case, in both formats, is bit for bit the
  in-process one (the in-process render is also checked bit for bit for calls of 64 and 1024
  frames). The device outputs' tracks are compared with two REAPER exemptions (multi-output,
  below).
- **The cases**, all at 120 BPM (below), each also compared as follows:
  - the 303's slide pattern (`303_slide`), and the 808's and 909's accent patterns
    (`808_accent_level127`, `909_accent_level127`) with the PCF on their channel (wave 0),
    dragged out (`clip::notes`, two loops) and played in note mode, against orb-player's
    pattern-mode render (Latch Pattern Mode through the same process side, the host rolling
    from beat 0), up to the block where the pattern starts the loop the clip does not have.
    With each note moved to the first frame of the block where the pattern plays its step
    (and the 303's first note-off to tick 25, where the pattern's clean start, stepping step 1
    at tick 1, closes its gate), all three are within one LSB (task-243): the real check. The
    moved note-off covers a real discrepancy: the clip closes the first gate at tick 24, the
    pattern at 25, up to 0.0045 apart (task-265). At
    the clip's own times the difference is task-227's gap (the same hits a block later, and
    the 303's waveform, which depends on when a note starts); the bounds there are measured
    ceilings, not a model, catching only gross changes: 0.1497 (303), 1.5874 (808), 1.7431
    (909), the drums' equal to `tests/note_clock.rs`'s;
  - `full_mix` at 48 kHz (the resampler, the 105-frame latency): bit-identical to the
    in-process plugin (`Core`) at 48 kHz, not to orb-player's 44.1 kHz render;
  - multi-output (section 13): the four device outputs routed (`NCHAN 10`, each device track
    `AUXRECV` of channels 3/4 .. 9/10 before its fader, no master send) and rendered as stems.
    The Mix is bit for bit the same project's with no outputs routed. Each device track is the
    in-process plugin's (`Core`) bus bit for bit, except that REAPER 7.86's VST3 host writes 0
    for a sample below about 1e-16 (the 909's decay tail, 3e-17; allowed for the VST3 tracks
    only);
  - `loop_ignored`, a limitation pin and not a matching case: Latch Pattern Mode of
    `full_mix` with a looped time selection (bar 2 to 3, `LOOP 1`). **REAPER's command-line
    render does not loop**: it renders the linear timeline, bit for bit the project's without
    the loop (the master renders compared bit for bit), and the test fails once that changes. A looped host transport is not
    covered (task-264);
  - `tb303_stuck_note` (task-258, section 9.1): the 303's slide clip (two loops, note mode, the
    clip's times) with a note-on of key 39 at tick 0 whose note-off never comes, the item
    running 1.5 s past the clips. The orphan holds the gate (the clip's notes slide) until the
    clip plays key 39 and lets it go; from half a second after the clips to the item's end the
    master is silent (peak 0 in both formats). Before task-258 the in-process render kept the
    303 sounding to the end, and REAPER's differed from it from the item's end (frame 242569,
    up to 0.092). The inference, not an observation of REAPER's messages: **REAPER most likely
    closes a note an item left open at the item's end** with a note-off, because the pre-fix
    plugin changed its output there and reacted to nothing at that point but a note-off of the
    held key (the item's all-notes-off it ignored); the in-process host sends no such note-off.
    The in-process events carry the all-notes-off REAPER writes at each item's
    end (`E ... b0 7b 00`, on channel 1), a no-op in every case since nothing is held there.
- **120 BPM, not 125** (found by this test): at 125 BPM a block starts exactly on bar 2's tick
  768 (the host's tick at block `m` is `m * tempo * 512 / 110250`; block 1323), and the
  plugin's position there, extrapolated from the start of the host's call, comes out as 768.0
  or 767.9999999999999 by the call's start. The output then depends on the host's buffer size
  from that bar line on: in-process, Latch Pattern Mode at 48 kHz differs between calls of 256
  and 512 frames from output frame 92161 (up to 0.16), and the note-mode clock's PCF at
  44.1 kHz (the 808 case) between calls of 64 and 1024 frames from frame 84608 (up to 0.12);
  REAPER's VST3 and CLAP renders at 48 kHz matched different call sizes. Section 4's claim that
  the output does not depend on the buffer sizes holds only away from such exact ticks
  (task-263). At 120 BPM the first block that starts on a whole tick that is a multiple of 24
  is 11025 (16 s), after every render's end.

### 7.2 CLAP was dropped: removing the old bundle (task-278)

The user decided on 2026-11-22 to ship VST3 only. A user who installed the CLAP version
(releases up to v2026.1118.0919) has an `OpenRB-338.clap` that a new release no longer
replaces, so each way of installing deals with it:

| Install | The old CLAP |
|---|---|
| Windows installer (`scripts/installer/openrb.nsi`) | deletes `%COMMONPROGRAMFILES%\CLAP\OpenRB-338.clap` (and the one in the folder the earlier install recorded as `ClapDir`) on every install, whether or not the VST3 component is picked; the uninstaller deletes it too; nothing else in a CLAP folder is touched; the options `/CLAP=` and `/CLAPDIR=` are ignored |
| macOS plugin `.pkg` (`scripts/pkg/preinstall`) | removes only `/Library/Audio/Plug-Ins/CLAP/OpenRB-338.clap` (never the folder) and forgets the old package's receipt (`pkgutil --forget net.emphased.openrb338.plugin.clap`); a copy in `~/Library/Audio/Plug-Ins/CLAP` is not touched; checked by `scripts/test-pkg-scripts.sh` against a scratch folder (no `/Library`) |
| `.dmg`, the archives | nothing can run: the `.dmg`'s README and the archives' README.txt say to delete `OpenRB-338.clap` from `/Library/Audio/Plug-Ins/CLAP` or `~/Library/Audio/Plug-Ins/CLAP` (macOS), `~/.clap` or `/usr/lib/clap` (Linux), `%COMMONPROGRAMFILES%\CLAP` or `%LOCALAPPDATA%\Programs\Common\CLAP` (Windows) |

**A project saved with the CLAP plugin** keeps a reference to the CLAP plugin id, which no
longer exists: the host shows it as missing. Add the VST3 `OpenRB-338` in its place on the same
track (the song, the patterns and the controls are plugin state: copy them by loading the song
again, or keep the old project's plugin state by hand in a host that moves it). The two formats
share the state's document (section 5), but a host does not move state between formats.

## 8. Host sync, pattern selection by notes, the Standard Mapping (task-198)

In this section "pattern mode" is the plugin's pattern modes (Latch and Gate, task-218): both
follow the host the same way; the section notes differ (Latch: below; Gate: section 11).

### 8.1 How the original follows an external position [C]

The DLL has a host-sync path of its own, for TopHat (ReWire's precursor; out of orb's scope
otherwise; its 18 audio outputs are in `docs/rewire.md`): the export `TopHatDriveAudioProc` (0x1001B800) calls `sub_1001C0F4` (0x1001B891)
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
  The snapshot's tempo is what orb plays at. The preview plays at the same tempo, without
  following the position (section 4.2).
- **Position** (pattern mode, no preview, and the host playing): the host's tick is
  `beats * 192` (bar 1 beat 1 is tick 0, step 1; quarter notes, whatever the host's time
  signature; other than 4/4 is task-206's decision). The first block that follows (the host
  starts, pattern mode is chosen or the preview ends while it plays, or a song loaded over the
  playing one is switched to) stops the song (song msg 35 `{1,0}`), locates it to `trunc(tick)` and plays it: the
  devices start clean at the host's position (slot 4, then slot 3). A later block whose tick is
  a tick or more away from the last one plus the clock's advance (`tempo * TEMPO_STEP`) is a seek
  or a loop jump: a locate, as the TopHat reposition. Then, in every block, the position is set
  to the host's fractional tick, and the clock adds the block's advance, so after the block it
  equals the host's position at the block's end. The TopHat path's loop-point hold-off (the
  countdown) is not taken: the host's loop is not the song's, and pattern mode has none.
- **Host stopped** (task-217; before it, the song ran on): the song is stopped (song msg 35
  `{1,0}`), so pattern mode makes no sound, except in Gate Pattern Mode while a gate note is
  held (section 11: free-running from step 1). A host that plays but reports no position plays
  the song free-running from where it is (the clock alone).
- An offline bounce reads nothing but the calls' frames and transports: two renders of the same
  timeline are identical, and with a host whose positions do not depend on where its buffers
  start, so are different buffer sizes (`sync.rs`).

**Notes.** A note-on (velocity above 0) 0..31 on MIDI channel 1..4 (303 #1 = 1, 303 #2 = 2,
808 = 3, 909 = 4; a keyboard or DAW track on its default channel 1 reaches 303 #1) in Latch
Pattern Mode selects pattern n+1 of that section (A1..D8): the pattern controller's panel message (it
becomes the pending pattern), then, while the song plays, slot 2 on that device, which takes it
at once (`relocate`: `0x100255E0`, `0x10028030`, `0x1002B670`) where the panel's would wait for
the bar line; the next step is the new pattern's step at the same position. With the host
stopped (the song stopped, task-217) only the selection changes: the panel shows it, nothing
sounds, and the host's start plays it (the clean start's locate relocates every device). It
latches until the next note on that channel or a panel change (a parameter change: sent when
the value changes, and taken at the next bar line, as the original's panel); the last write
wins. A note-off and a note above 31 do nothing; in Gate Pattern Mode the notes 0..31 gate the
sections (section 11); in note mode every note on these channels plays the device (section 9). The preview ignores every note (section 4.2). Notes never change
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
pick the parameter). Play/Stop and the mode are no parameters (task-217): in pattern mode the
host's transport starts and stops orb, and stopping from MIDI is the host's stop.

The Standard Mapping's control actions (the devices' and the
mixer's knobs and switches, the pattern notes 12..63, the shuffle) are applied to the engine at
the message's frame, like a host note: they do not change the parameters, and the last write
wins. Its transport actions (Play, Stop, Record, the locates) are dropped: the transport is the
host's. The preview ignores the mapping altogether. Its view actions (bank, focus, instrument, keyboard
mode) are dropped too until the editor (task-201) shows them. Its toggle state follows the
snapshot after every block.

**LEDs.** `Snapshot::host_sync` is set while a block followed the host's position (pattern mode,
no preview, the host playing): the Sync LED. `Snapshot::midi_in` counts every MIDI message the
host sent (also those nothing maps, and in the preview, which ignores them): the MIDI LED.

## 9. Note mode (task-200)

In note mode (outside the preview) the song is stopped, and a note-on or note-off on MIDI
channel 1..4 (303 #1, 303 #2, 808, 909) plays that device by the note mapping of
`docs/midi.md` section 5, at the block holding the event (section 4), whether the host plays or
not (the no-sound-while-stopped rule is pattern mode's only; the user, 2026-11-16). The original plays no
device from a note (`docs/midi.md` section 4), so this is new engine behavior; it does not run
while the song plays, so pattern playback is unchanged (the corpus renders as before).

- **The path.** `src/core.rs` sends `Command::Note {device, note, velocity}` (velocity 0, or a
  note-off, is the note-off). If the mode has just turned to note mode and the song still
  plays, the Stop is sent first (`ensure_stopped`), so that its silence (slot 4 → slot 6) does not cut the
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
- **Held notes** (up to 16 keys, a fixed array; a 17th forgets the oldest): the newest sounds
  (last-note priority). Releasing it slides back to the newest note still held, with that
  note's accent (legato; a held note the Tune has since put outside the table is dropped);
  releasing a note that does not sound changes nothing; releasing the last one is the note-off
  (the gate is the note's length; the release is the note-off's amp constants).
- **A key is held once** (task-258; MIDI's own model, and Gate Pattern Mode's, section 11): a
  note-on of a key already held presses it again (its old entry goes and it becomes the
  newest; with another key or itself held it is legato, a tie for itself), and one note-off
  lets it go. Before task-258 a key was held once per note-on and a note-off let go of the
  oldest instance; the drag-out never overlaps two notes of one pitch (a tie is one longer
  note, section 12), so only a host's lost or doubled note gave two.
- **Missing and stray note-offs** (task-258). A note-off that never comes (the user's report,
  2026-11-21: a REAPER send to the plugin removed while a 303 note played; REAPER most likely
  closes the notes of an item at its end, inferred in section 7.1, but the removed send no
  longer carries them) leaves
  its key held: every later note slides from it (no attack) and every release slides back to
  it, so the 303 sounds without a break. It heals at the key's next note-off (a looped clip
  plays the key again), at an all-notes-off or all-sound-off on the section's channel, and at
  the host's stop; before task-258 only Play or Stop of the song (a mode switch) cleared it,
  as the held instances outnumbered the note-offs for good and the plugin ignored both
  controllers and the stop. A note-off of a key not held (a send added during a note) changes
  nothing.
- **A note whose MIDI source is gone** (the user's check, 2026-11-21: a send deleted while a
  303 note was held). The plugin then gets no more MIDI from that source, so the held key
  sounds on, as in any MIDI instrument, until the host's transport stops or an all-notes-off
  or all-sound-off arrives on the section's channel. The user decided on 2026-11-21 to keep
  this (option A, task-270); neither clearing held notes at a host loop wrap or position jump (B) nor a
  timeout (C). In REAPER 7.86 the action "Send all-notes-off and all-sounds-off to all MIDI
  outputs/plug-ins" sends both at once, and its MIDI reset preferences ("Reset on:", "On stop
  (default, recommended)" or "Reset on playback start"; strings of its executable) choose when
  it sends reset messages; the plugin also lets go at the stop itself.
- **All notes off.** In note mode CC 123 (All Notes Off) and CC 120 (All Sound Off) on channel
  1..4 (`src/core.rs`, `Command::NotesOff`, `Engine::live_notes_off`, `Tb303::key_all_off`)
  and the host's stop (the first block whose host record stops after one that played; also
  with the host's position) let go of every held key at once: the slide flag cleared, then the
  note-off, as the last key's note-off (bit for bit, `tests/held_notes.rs`). All Sound Off is
  no hard cut: the 303 decays by its release. The reason: the engine has no way to silence a
  voice but the original's note-off (slot 6, the song's Stop, releases the same way), a cut
  mid-waveform would click, and hosts send All Sound Off together with All Notes Off at a
  stop or reset (REAPER's action above), where a release is what a stop sounds like. Nothing when no key is held; the 808 and the 909
  hold no notes. The messages then reach the Standard Mapping as before, which maps neither
  (section 8).
- Play (slot 3) and slot 6 (Stop, a song switch, a reset) forget the held notes. This is what
  a mode switch reset while the host played: to Latch or Gate the song plays (Play), back to
  note mode it stops (Stop). With the host stopped a switch to Latch and back sends neither
  (the song stays stopped).

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
- 909: the band's step value and AC byte (`tr909::velocity_band`: 1..7 value 3, 8..47 value 1,
  48..55 value 4, 56..95 value 2, 96..99 value 3 + AC 2, 100..111 value 1 + AC 2, 112..115
  value 4 + AC 2, 116..127 value 2 + AC 2), then `gain = f32(accent * 0.8)` (values 1, 3) or
  `f32(accent * 2.0)` (2, 4).
- **The 909's flam** (task-245, decided in task-246). Values 3 and 4 (the flam slices) arm the
  instrument's own flam (`Tr909::note_flam`, not the step function's `+0x1910`), and
  `Tr909::note_flams`, run by the 909's audio slot before each block, fires it as the step
  function's flam pass: the tick and the retrigger of `docs/midi.md` 5.5, on the note-mode
  clock's tick (section 9.5). A plain note of the instrument, slot 6 and Play forget it; only
  notes arm one, so pattern playback and the corpus are unchanged.
  - **Why** (the user's report, 2026-11-18: the clap of a dragged-out 909 pattern A6 sounded
    shorter than the preview's). The clap's step value there is 3 (low + flam). In the pattern
    the flam pass (0x1002AAC0, flamTick 0x1002ABBC, the value test at 0x1002B324) restarts
    the clap's SIMPLE voice (decay byte 127: multiplier 1.0, the sample plays out) 11 ticks
    (28.6 ms at Flam 54 and 120 BPM) after the step at `accent * 1.5` instead of
    `accent * 0.8`, so the pattern's clap ended 28.6 ms later and 5.5 dB louder. The drag-out
    wrote value 3 as value 1 and a note armed no flam: the notes' output differed from the
    pattern's by up to 0.94 from the flam's block on; with the clap's value 3 set to 1 in both,
    within one LSB. The bands, the following CH/OH/BD steps and the trigger code gave no
    difference.

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

The minimum on-time and the drawing are the editor's (section 9.4).

### 9.4 The note-mode lights (task-202)

In the plugin's note mode (the mode selector's Note, not the preview, which plays the patterns) the
panel lights the 808's and the 909's sounding voices (`embed::Lights`, `App::note_lights`,
`App::instrument_selection`). The 303s show nothing; pattern mode and the preview are as
before (the standalone app has no lights and is unchanged).

- **808:** a green ring of the selection ring's shape on the chip of the sound played: the lower
  legend (`LT`, `RS`, ...) or the upper chip (`LC`, `CL`, ...) by the sound's bit, whatever the
  slot's switch says. A chip that is also the selected one gets the green ring instead of the
  orange (the `litsel808_*` part: the selected chip's lighter fill and the green ring); the
  selected chip when the switch shows the other label of the slot keeps its orange ring. AC's
  chip is lit while AC is.
- **909:** the instrument's LED in the name row (and the AC box for AC) is green; on the
  selected instrument the green LED replaces the red one (the selected name's label stays lit).
- **Not by hue alone:** the green ring is wider (1.6 against 1.1 panel pixels) with a white core
  and a wide, strong glow, the orange ring is thin with a faint glow; the green LED has a wider
  halo and a white core, the red selection LED has neither (checked in a deuteranopia
  simulation of the screenshots).
- **When lit:** a sound is lit while the latest block's mask has it, and for at least 100 ms
  (`MIN_ON`, tuned by eye) from every hit, so several voices light at once. A choke darkens a
  voice when its minimum is over. AC is lit while a lit sound's `ac` bit is set. The engine
  counts the hits of each sound (`Sounding::hits`, wrapping, in the snapshot: every trigger,
  retriggers of a sounding voice too); the editor keeps the counts it saw and a count that
  changed since its last frame is an onset that restarts the minimum. The counts are state,
  not events, so no hit is lost between two snapshots or two frames (a hit shorter than a frame
  lights for 100 ms, a retrigger restarts the light) with no atomics or locks. The first frame
  after note mode starts takes the counts without lighting. The panel is drawn every 16 ms
  (`LIGHT_POLL`) only while a light is lit or the engine plays; otherwise the 50 ms `POLL`
  carries it (idle note mode costs about the same as pattern mode, within the harness's noise: one run
  of each gave 7.3% and 5.0% of a core), and a hit is seen by its count
  at the next frame.
- **The art:** `lit808_<name>` and `litsel808_<name>` (17 chips each), `led909_lit`,
  `led909ac_lit` (assets/ui/README.md). `editor_preview --note CH,NOTE,VEL` sends host notes
  for screenshots.

### 9.5 The PCF in note mode: the note-mode clock (task-242)

The PCF picks its wave step from the block's tick: `quotient(ctx.tick, res)`
(`crates/orb-engine/src/pcf.rs`, the original's 0x100166D0; `tick / 24`, `/ 48` or `/ 96` at
resolution 32, 16 or 8, 0x10016792..0x100167AE), and a new step sets the decay and triggers the
envelope (0x10016845). The tick is the song's position (`sub_1001F870`). In note mode the song
is stopped, so it stood at the bar start: the PCF stayed on one step, with no triggers, and a
dragged-out pattern (section 12) sounded unlike its preview. The original behaves so with its
song stopped: the PCF block 0x100166D0 takes the tick from the song's position (`sub_1001F870`,
read at 0x10016756), and the clock `sub_1001EF20`, the only code that advances the position
each block, returns at once while the play flag is clear (0x1001EF34,
`docs/initial_investigation.md` section 5 [C]), so the tick stays where Stop's locate put it.
Note mode is new engine behavior, and the user decided (2026-11-18) that the PCF's tick in
note mode comes from a clock of its own:

- **The engine** (not in the original): `Frame::note_clock`, a fractional tick, off (`None`)
  in every new song, so the corpus renders, the app and the pattern modes never see it. While
  it runs and the song is stopped, each block first adds the song clock's advance to it
  (`tempo * TEMPO_STEP`, as `sub_1001EF20` adds it to the position at 0x1001EF51; the tempo is
  the song's, which the plugin sets to the host's in every mode, section 8.2), and the block's
  `BlockContext::tick` is `trunc` of it (`Frame::block_tick`). While the song plays the song's
  tick wins and the clock stands. `Engine::set_note_clock` (orb-player's
  `Command::NoteClock`): `Off`; `Run`, which starts a clock that is off at the song's position
  and keeps a running one; `At(tick)`, which sets it.
- **The plugin** (`src/core.rs`, `Engine::note_clock`, in `sync` before the Stop the mode owes):
  in note mode outside the preview, while the host plays and reports a position, the clock is
  set before every block to the host's tick at the block's host frame, `beats * 192`
  extrapolated at its tempo exactly as pattern mode's position (section 8.2), so that after the
  block's advance it equals pattern mode's position after the same block. While the host is
  stopped or reports no position the clock runs free at the tempo from where it was: from the
  last host position when the host stops; from the song's position when note mode starts with
  the host stopped. The clock is started before the Stop that the mode owes, so a song that
  played stops after the clock has taken its position: leaving the preview (or Gate Pattern
  Mode's free-running gate, or a host that played without a position) with the host stopped
  continues the PCF from the tick the playing song had reached, not from the bar start that
  the Stop's locate (`sub_1001E100`) moves the song to. When the song was already stopped (an
  activation, Latch Pattern Mode with the host stopped) that is its bar start. In the pattern
  modes and the preview the clock is off. The song stays stopped and the sequencer steps no
  device; only the PCF and the 909's note flams read the tick.
- **What reads the tick.** Of the devices and effects, the PCF reads `BlockContext::tick`
  (`pcf.rs`, `control`), and the 909's audio slot reads it and `ctx.tempo` for the flams that
  note-mode notes armed (`Tr909::note_flams`, section 9.2, task-245; only notes arm one, so a
  playing song never runs it). The delay reads `ctx.tempo` (its length, slot 16),
  which is the host's tempo in note mode too, so the delay's time is the preview's. The mixer
  update's context keeps the song's tick (the PCF's slot 16 does not read it).

**The drag-out against the pattern** (`tests/note_clock.rs`): an 808 pattern (the diag song
`808_accent_level127`: BD and SD with AC), a 909 one (`909_accent_level127`) and both with
shuffle (`shuffle_drums_g64`, 808 and 909 together), with the PCF on the section's channel
(wave 0), are played for two loops at 120 BPM with the host rolling from bar 1 in Latch Pattern
Mode, and their drag-out (`clip::notes`) is played into note mode with the same host:

- The PCF's step quotient after every block is the same in both, block for block, from the
  first block (the PCF triggers on the same blocks).
- The notes played at the host's frames (the first frame whose position reaches the note's
  tick, as a host plays a MIDI item) sound up to a block later than the steps: a step plays in
  the first block whose position after the advance reaches it, a note in the first block at or
  after its frame (section 4, task-227), so the note comes one block (64 frames, 1.45 ms) later
  for most ticks; and the pattern's step at tick 0 after the clean start (Stop, Locate 0, Play)
  plays in the second block, as the clock steps the devices from tick 1, while a note at 0
  plays in the first. The outputs differ by up to 1.6 (808), 1.7 (909) and 0.21 (shuffled),
  the same hits shifted by a block.
- The same notes put in the blocks where the pattern plays its steps give the pattern's output
  within one LSB of the int16 output (1/32768): the first difference at frame 3394 (808), 3396
  (909) and 3438 (shuffled), in about 3400 of the 176400 frames. It is there without the PCF
  too, so it is the drum voices', not the clock's; the cause was not isolated (the notes and the
  steps run the same trigger code, section 9.2; as section 4's later notes, some state that
  runs on in silence, here likely what the pattern mode's clean start, a Stop with its slot 4
  and 6, resets; task-243).
- So the accent mapping (808 velocity 127 = AC 1; the 909's bands), the shuffle baked into the
  clip (the shuffled ticks, `seq::shuffled`) and the delay's tempo give no difference of their
  own. Not compared: the 303s (their own comparison is section 9.1's).
- **The 909's flam** (task-245, section 9.2): the user's pattern A6 (the clap at value 3 on
  steps 5 and 13, Flam 54, Accent 127) written into `909_accent_level127` gives, in the steps'
  blocks, the pattern's output within one LSB, and differences up to 2.0 at the host's frames
  (the same loud hits a block apart, as above). With the clip's flam velocities replaced by the plain
  ones the aligned outputs differ by up to 0.51.

## 10. The editor (task-201)

The editor is orb-ui's panel, drawn by the same code as the standalone app
(`crates/orb-ui/src/app.rs`), in the host's plugin window: nice-plug-egui's `EguiEditor` on
egui-baseview with wgpu (the wgpu of eframe, 30.x, and egui 0.37 as orb-ui). orb-ui takes eframe
only in its default `app` feature, so the plugin links no eframe and no winit. `src/editor.rs`
has the pieces; orb-ui's side is `crates/orb-ui/src/embed.rs`.

**The window.** The standalone app's menu bar (File, Mods, Options, Help; egui-drawn, 22
points high, `MENU_H`) on top, not scaled, across the window's whole width, and the panel
under it, **letterboxed on the standalone's desk**: the rest of the window less a margin of
16 points left and right and none above and below (`embed::MARGIN_X`, `embed::MARGIN_Y`, not
scaled; `embed::panel_area`) is the area the panel is fitted in, the largest panel-shaped area
centered in it (`orb_ui::app::View::fit`, the standalone app's own fit), and the whole area
under the bar is the standalone's desk (`backdrop::paint`: the dark gradient, the tile, the
vignette and the panel's shadow). At the exact fit the panel touches the menu bar and the
window's bottom edge, with a strip of desk on each side (the user's design of 2026-11-17: first
8 points on all four sides, after the grip on the window's corner was hard to tell from the
host's own grip; then, after the check of 02eb46e, twice that left and right and none above
and below). The panel's exact fit, height = `round((width - 32) * 510 / 624) + 22` points
(`embed::window_height`), leaves only the side strips; the default size is 656 x 532 (the
panel at 1:1, `editor::DEFAULT_SIZE`). `editor_preview` screenshots of 656 x 532 (exact
fit), 1100 x 532 (wide) and 656 x 800 (tall) were checked. **Project dirty state:** REAPER and
Live do not mark the project modified when only the plugin window is resized (the user's
check of 2026-11-17; AudioRealm ABL3 behaves the same); the size (`editor_size`) is saved with
the plugin's state at the next save. The menu strip stays out of the scaled area: it keeps its height at every size,
as in the standalone app. The standalone app is unchanged (its `--screenshot` is byte for byte
the same before and after task-234).

**The host may give the editor any size** (task-234, the user's decision of 2026-11-17),
in both formats and with no format detection. `Editor::resize_hint()` (`editor::resize_hint`;
nice-plug 0.4.2 answers CLAP `gui.can_resize` and `gui.get_resize_hints` from it,
`wrapper/clap/wrapper.rs` 2907-2937, and VST3 `IPlugView::canResize`, `wrapper/vst3/view.rs`
721-732; no patch of nice-plug) is resizable both ways, with a minimum of 488 x 373
(`MIN_WINDOW_W/H`, task-297: every window the user cannot resize fits in it; it was half the
panel and the bar, 312 x 277), no maximum and no aspect ratio. CLAP `gui.adjust_size` and VST3
`checkSizeConstraint` (nice-plug routes both to `OrbHandle`'s `EditorHandle::adjust_size`,
`view.rs` 734-772) give the host's size back unchanged, raised to the minimum only
(`editor::adjust_logical`); `EditorHandle::set_size` (egui's) applies it. The editor's own
baseview window (nice-plug-egui's settings) has the same hint: a non-resizable baseview window
is fixed (baseview 0.4.2 `utils.rs` `SizingStrategy::Fixed`), and the host's `set_size` must
take effect. Sizes on macOS are points (nice-plug-core 0.4.2 `editor/dpi.rs`: `NativeSize` is
logical there), so the backing scale does not enter.

**Host quirks (task-234).** Found with the user's checks of 2026-11-16 and 2026-11-17 and
the size log (below):
- **Ableton Live 12 Lite (VST3, macOS).** Live hosts the view in a window of its own and, on
  each mouse event of a drag of its window's corner, calls `checkSizeConstraint` with the
  cursor's size (absolute, from the window's top left), sizes its window to the answer and
  calls `onSize` with it. nice-plug's `onSize` is `set_size`, which makes it the window's size
  synchronously (baseview's `resize` calls egui's `resized`, `platform/macos/view.rs` 243-273
  and 371-405), so the answer becomes the current size before the next event.
- **Bitwig (CLAP).** The same sequence through the CLAP API: `gui.adjust_size` with the
  cursor's size on each mouse event, then `gui.set_size` with the answer.
- **REAPER (CLAP, macOS).** On each mouse event REAPER calls `gui.adjust_size` and then
  `gui.set_size` with the answer, but keeps its own FX window at the size it asked for. The
  user's log `~/orb-sizes.log` (2026-11-17, with the contain rule below): `adjust_size 721x532
  (scale 2, window 720x610) -> 721x611`, `set_size 721x611`, and REAPER's window stayed 721 x
  532, so the editor, 79 points taller than its window, was cropped at the bottom; with the
  contain rule every edge and corner drag cropped.
- **REAPER (VST3, macOS).** REAPER's FX window frame and the editor are sized separately:
  with the host's resizing off (variant 3 below, the panel's corner the one way) a resize
  took two steps, REAPER's frame first and then the corner (the user's check of
  2026-11-17).
- **The host's grip and the panel's (VST3).** With the panel's grip in the window's own
  bottom right corner (until the margin), a host-frame drag of the VST3 (the user's check of
  f315da3, 2026-11-17) stuck at one width or height for a while and kept it on release, a
  snap no host-only drag gives: the editor answers every host size unchanged. The fake VST3
  host (`a_live_corner_drag_follows_the_cursor`, with an `IPlugFrame` that records
  `resizeView`) shows that the view asks nothing of the host during a host drag, with the
  main run loop run after each event and a backing change sent to the editor's view
  (baseview's own request path, `platform/macos/view.rs` 371-405). The remaining source is
  the panel's grip: it was the window's bottom right 14 points, the same spot as the host's
  resize grip, so a press there could start the panel's corner drag, which asks for the
  panel's exact fit at the larger of the two scales (`corner_size`): moving along the smaller
  scale's axis leaves the size unchanged, the "sticking" seen. [G] (no VST3 size log of that
  check.) The grip now sits on the panel's corner, `MARGIN_X` (16 points) in from the
  window's right edge, and is always drawn. Its bottom is at the window's bottom edge (no
  margin there), but its 14-point area spans 30 to 16 points from the right edge, clear of the
  host's corner grip, which on macOS takes only the last few points at the window's corner;
  the host's corner is the host's alone.

Any answer that depends on the window's current size flips in Live and Bitwig, and any
answer other than the host's size is cropped in REAPER's CLAP; taking the host's size as it
is and letterboxing the panel is the one answer that is stable in all four.

**Variants tried (task-234).** Live's recording of 2026-11-17 showed the panel's scale
jumping back and forth during a corner drag: the rule before task-234 (height =
`window_height(width)`, the width taken from the side the host changed more relative to the
window's current size) chose by the current size, and with the cursor off the rule's
diagonal the width's answer left the height far from the cursor's, so the next event got
the height's answer, and so on (the recording's widths, 4.23 to 4.50 s, frame pixels: 646,
636, 688, 601, 577, 629, 535, 513, 561; a replay growing 8 x 4 points per event gave 664,
653, 680, 663, 696, 673, ...). Then, in turn: (1) a rule of the host's size alone, fitted
inside it: no flip, but an outward drag of one edge no longer grew the window; (2) the
smallest size of the rule that contains the host's size: no flip and either edge grows it,
but REAPER's CLAP crops (above); (3) not resizable by the host (`canResize` false, as
AudioRealm ABL3), the panel's corner the one way: Live worked, REAPER's VST3 needed two
resizes, and REAPER's CLAP resizing jumped; (4) per format, (3) for the VST3 and the CLAP
resizable with the old rule: Bitwig's CLAP jumped as Live's VST3; (5) per format, (3) for the
VST3 and the CLAP with rule (2): REAPER's CLAP cropped. (6) The decision: any size, both
formats, letterboxed. CLAP's aspect hints (`preserve_aspect_ratio` with
`aspect_ratio_width`/`height` in `clap_gui_resize_hints`) were not used: the 22-point bar and
the 16-point side margins do not scale, so no fixed ratio is the panel's shape (a ratio of
the default 656 x 532 gives 1012 where the fit, round((1248 - 32) x 510 / 624) + 22, gives
1016 at width 1248), and VST3 has no such hint.

**Tests.** `tests/editor.rs` `the_size_is_kept_in_the_state_and_the_host_gets_sizes_of_the_rule`
checks the hint (resizable both ways, no aspect ratio), `adjust_logical` giving the host's
size back (721 x 532, 630 x 1042, ...; the minimum below it) and, on Live's and Bitwig's
drag, answer == request, where the rule before task-234 goes back on the same drag;
`the_panel_is_letterboxed_in_a_window_of_any_shape` fits the panel in wide (1100 x 532,
721 x 532, 1600 x 1042) and tall (656 x 800) windows, checks that it lies inside the
window's margins (no crop) and that a click on the letterboxed panel hits the control under
it (the panel's hit-testing maps the pointer through the fitted `View`, `View::to_panel`; the
menus, windows and the file browser use the whole window);
`the_grip_is_on_the_panels_corner_clear_of_the_windows` checks that the panel's corner
resizes and the window's own corner does not; orb-ui's `embed` unit test checks the exact fit
(the panel fills the area within the margins and touches the bar and the bottom edge) and the
grip `MARGIN_X` in from the window's right edge;
`the_corner_drag_resizes_steadily_through_the_host` drags the panel's corner through
`HostRoute` with a host that sizes the window inside the request (REAPER's way): every size
asked for is the panel's exact fit, passes `adjust_logical` unchanged and follows the drag.
`tests/clap_gui.rs` `a_live_corner_drag_follows_the_cursor` loads the library as a VST3 host
(`createView`, `attached` to an `NSView`): `canResize` is `kResultOk`; Live's sequence along a
drag off the diagonal (growing to 1000 x 700, shrinking to 400 x 450) gets the cursor's size
from `checkSizeConstraint` and the view calls no `resizeView` on the test's `IPlugFrame`
during that drag, and the corner's sizes (computed by the test with `corner_size`; the test
cannot drag the egui corner of the loaded library) pass the round trip unchanged.
`open_editor` checks `gui.can_resize`, `gui.get_resize_hints` and `gui.adjust_size` (1000 x 540,
721 x 532 and 630 x 1042 unchanged, 10 x 10 to 488 x 373); `the_host_sizes_the_editor` replays
Bitwig's drag through the CLAP API (`gui.adjust_size`, `gui.set_size`, `gui.get_size`): every
answer is the host's size.

**The size log.** With `ORB_PLUGIN_SIZE_LOG=FILE` the editor appends each host size call and
request to `FILE` (`editor::size_log`: `resize_hint` for `canResize`, `adjust_size` for
`checkSizeConstraint` with the window's size, `set_size` for `onSize`, `size` for `getSize`,
`set_parent`, egui's `resized`, baseview's and the corner's requests and their answers), each
line with the ms since the library's first line. To capture a host's real sequence, start it
from a terminal with the variable, e.g. `ORB_PLUGIN_SIZE_LOG=$HOME/orb-sizes.log
"/Applications/Ableton Live 12 Lite.app/Contents/MacOS/Live"`, or `open -a "Ableton Live 12
Lite" --env ORB_PLUGIN_SIZE_LOG=$HOME/orb-sizes.log` with Live not running. The variable is
read once per loaded copy of the library.

**The diagnostic log (task-260).** `src/plugin_log.rs`, in release builds too. nice-plug's
`NICE_LOG` does nothing in this plugin: nice-plug is built without its `tracing-subscriber`
feature and the plugin's `Plugin::setup_logger` is nice-plug's default (`None`), so no logger
exists (`nice-plug-0.4.2/src/wrapper/util.rs` `setup_logger`), whatever the host passes on;
the `nice_warn!`/`nice_error!` lines of the plugin go nowhere. The plugin's own log is on when:
- `ORB_PLUGIN_LOG` names a file (appended to), or is `stderr` (Bitwig copies its plugin host's
  stderr into `~/.BitwigStudio/log/engine.log`: the `libEGL warning` lines of the user's first
  Linux run are there);
- or, with the variable unset or empty, the switch file `plugin.log` exists in orb-ui's log
  folder (`orb_ui::crash::log_dir`, the crash logs' folder; `ORB_UI_LOG_DIR` moves it). Creating
  the empty file turns the log on for a host that does not pass the environment on; the log is
  appended to it. Delete the file to turn it off. The switch file is exactly:

  | System | Switch file |
  |---|---|
  | Linux | `$XDG_STATE_HOME/OpenRB-338/plugin.log`, or `~/.local/state/OpenRB-338/plugin.log` |
  | macOS | `~/Library/Logs/OpenRB-338/plugin.log` |
  | Windows | `%LOCALAPPDATA%\OpenRB-338\logs\plugin.log`; in portable mode (task-289, task-295, docs/building.md) `logs\plugin.log` next to the `OpenRB-338.vst3` file (the zip); the installer's bundle in `%COMMONPROGRAMFILES%\VST3` is portable only when an `OpenRB-338.portable` marker is put beside it |

  It is not in the settings folder (the settings file's: `~/.config/OpenRB-338`, `~/Library/Application
  Support/OpenRB-338`, `%APPDATA%\OpenRB-338`): a `plugin.log` there does nothing.

It is read once per loaded copy of the library, at the first activation or editor open. It has
a first line with the version, the system, the process and the variables that choose the
graphics (`XDG_SESSION_TYPE`, `DISPLAY`, `WAYLAND_DISPLAY`, `WGPU_BACKEND`, `WGPU_POWER_PREF`);
a line per activation (`Plugin::activate`, main thread, never the audio thread): `activate:
loaded from <library path> at <rate> Hz` (`dladdr` of the plugin's own code, `?` on Windows;
the REAPER test reads it, section 7.1; it replaces task-244's `ORB_PLUGIN_LOAD_LOG`); the `log`
records of wgpu and egui-wgpu at info and above (wgpu-core's `acquire error` and `present
error`, egui-wgpu's software-rasterizer warning), egui-baseview at debug, and the other crates
at info; every panic (a chained panic hook: wgpu's uncaptured validation errors panic); and the
editor's lines: the adapter choice below, `editor: spawn` with the size and the parent,
`editor: window built` with the window handle, the first five frames and each frame after a
pause of over a second (size in pixels, the time since the last), the frame count 1, 2, 4, 8,
16 and 32 s after the window was built, and `editor: closed after N frames`. At most 20000
lines per process. `ORB_PLUGIN_LOG_LEVEL=debug` adds wgpu's and egui-wgpu's debug records (the
instance's backends, each adapter's details, the swapchain configured at each resize, wgpu-core
device resources, Metal's shader sources), which is why they are off by default: the macOS
`clap_gui` test (seven editor opens) wrote 3,420 lines with them and 61 at the default level
(task-260). What it cannot show: egui-baseview's renderer drops a surface acquire of
`Timeout`, `Occluded`, `Outdated`, `Suboptimal` or `Lost` without a log of its own
(`egui-baseview-0.9.0/src/renderer/wgpu/renderer.rs` 207-232); a frame counted but never
presented shows as frames in the log and nothing on the screen.

**The wgpu adapter (task-260).** egui-baseview makes the editor's wgpu instance itself, with
all backends and no environment (`renderer.rs` 57, `InstanceDescriptor::new_with_display_handle`);
the plugin cannot change it without patching egui-baseview. It chooses the adapter:
`plugin_log::graphics_config` gives egui-baseview egui-wgpu's `WgpuSetup::CreateNew` with a
`native_adapter_selector` and, as the backends to enumerate, those of `WGPU_BACKEND` when it
is set (wgpu's names, comma-separated: `vulkan`, `gl`, `metal`, `dx12`), else all. The
selector picks as wgpu-core 30's `request_adapter` does (`instance.rs` `get_order`): the
adapters that can present to the window's surface, in wgpu's backend order (Vulkan, Metal,
DX12, GL), stably sorted by device type for egui-wgpu's power preference (`WGPU_POWER_PREF`,
default high performance: discrete, integrated, other, virtual, CPU), the first wins. So the
default is the same as before task-260: all backends, wgpu's own choice (on a Linux VM with
Mesa, the software Vulkan llvmpipe before the software GL llvmpipe, both CPU devices). Unlike
the standalone app (docs/building.md, Linux runtime), the plugin's default on Windows is not
DX12 only. The selector logs each adapter, whether it presents to the window, the one taken,
the surface's formats, present modes and alpha modes, and the format and present mode egui
takes (`AutoVsync`, frame latency 2: egui-wgpu's `SurfaceConfig::HIGH_THROUGHPUT`). The
`libEGL warning: DRI3 error` lines when the editor opens come from wgpu making its GL backend
(EGL) at instance creation, which it does for every backend; they do not show that GL was
taken.

**Linux: baseview paces every frame on the X server's Present extension (task-260).**
baseview 0.4.2 on X11 draws (`handler.draw`, egui-baseview's frame and present) only when
`PresentState::draw_now` is set (`src/platform/x11/present.rs` 72), and with the Present
extension (`x11_connection.rs` 56) only a Present `CompleteNotify` sets it
(`handle_present_complete_notify`, 134-158): it must answer baseview's last `PresentNotifyMSC`
serial and its MSC must be past the last one received (a completion with an MSC that did not
advance is recorded and draws nothing, 150-153). The next request asks for the last MSC + 2
and is sent only after the previous one was answered (`handle_present_notify`, 192-226);
a redraw request (egui's repaint, a mouse event: egui-baseview `window.rs` 520, 938) only asks
for that notification. The 15 ms fallback timer (`setup_fallback_frame_timer`, 160) runs only
when the server lacks the extension or refuses the request. So when the X server's MSC for
the window does not advance (no vblank: a VM's display driver), or a notification is never
answered, the editor draws its first frame and then no more, and no input brings it back.
The log's Present probe (X11 only, 1 s after the window is built) asks the same of the X
server from a connection of its own (six `PresentNotifyMSC` on baseview's window, the first
for MSC 0, each next for the last MSC + 2, serials `0x4f524200` + n, which baseview ignores)
and logs each answer's MSC, UST, kind, mode and delay, or that none came in 2 s.

This is the cause of the black (VST3) or stale (CLAP) editor in Bitwig on the user's VM
(task-260, confirmed 2026-11-21 with build 2026.1121.0857): Ubuntu, kernel 7.0.0-30, a VMware
SVGA II adapter (VirtualBox VMSVGA), Xorg on modesetting with glamor refused on llvmpipe
("Refusing to try glamor on llvmpipe"), Present 1.2 initialized, Xfce under gdm. The editor
took Vulkan llvmpipe (Cpu) over GL llvmpipe, `WGPU_BACKEND` unset. Every probe
`PresentNotifyMSC` on the editor's window, for MSC 0 and then for MSC 2, completed at once (0
to 6 ms) with MSC 0 and UST 0 (kind `NOTIFY_MSC`, mode `COPY`): the MSC never advances, and
the editor drew exactly one frame (the count stayed 1 at 1, 2, 4, 8 and 16 s, "closed after 1
frames"), for the VST3 (parent none, waiting for it) and the CLAP (parent given) alike. The fix
is a baseview patch (task-262, after Decide task-261). On closing, the same run logged
baseview's `Error while closing window: Request to X11 thread failed: Could not receive
response (X11 thread disconnected)` and then X11 `BadWindow` errors for Present
`SelectInput` and `DestroyWindow` on the editor's window (destroyed already); whether that
path causes another user's crash on closing the CLAP editor (Debian 13, Bitwig 6.1) is task-262's
to check.

**The panel's own resize.** The corner's request does not depend on `can_resize`: CLAP
`gui.request_resize` and VST3 `IPlugFrame::resizeView` are the plugin's own resize in both APIs
(`can_resize` / `canResize` is about the host's user dragging), and nice-plug passes the
request on without looking at the hint (CLAP `ClapHostCallbacks::request_resize`,
`wrapper/clap/wrapper.rs` 2713-2737; VST3 `Task::RequestResize` -> `WrapperView::request_resize`
-> `resizeView`, `wrapper/vst3/inner.rs` 735-747, `view.rs` 248-290). It asks for the panel's
exact fit, so a host that checks it (`adjust_size`) gets it back unchanged and the window
snaps to the panel with no band; the task-224 guards (`HostRoute`, below) keep a host that
sizes the window inside the request from looping.

**The resize corner.** A 14 x 14 point corner at the panel's bottom right (`embed::corner_rect`,
the right wood's foot, no control there), at least `MARGIN_X` (16 points) in from the
window's right edge (task-234): a grip of three diagonal strokes (`embed::paint_grip`, 2 points
in from the wood's right side, down to the wood's foot) is always drawn there, lighter while
it is hovered or dragged, the cursor turns to the diagonal resize arrow
over it, and a drag asks for the panel's exact fit (`window_height` of the width, the margins
included) at the larger of the two scales the drag gives
(`embed::corner_size`), once per new size. egui's `ViewportCommand::InnerSize` would not do:
egui-baseview 0.9 turns it into baseview's `Window::resize`, which resizes the plugin's view
without telling the host (baseview 0.4.2 `src/window.rs`: "does *not* trigger the
`HostCallbacks::request_resize` callback"; macOS `notify_host = false`), so the host's window
would not follow. nice-plug 0.4.2 has no `GuiContext::request_resize` (nih-plug's), but it
hands the host's callbacks (`HostMethods`, whose `request_resize` is VST3
`IPlugFrame::resizeView` or CLAP `gui.request_resize`) to `Editor::spawn` for baseview.
`OrbEditor::spawn` puts a proxy in between (`HostRoute`, `HostProxy`) that keeps them
reachable from the window's thread, and the corner's size (`Embed::request_size`) goes to
them; the host then sizes the window through `EditorHandle::set_size`. The request is made
after the egui frame, not in it: egui-baseview 0.9 holds its window state borrowed for the
whole frame (`src/window.rs` 584-592), and a host that answers with `set_size` at once
(Reaper does) resizes the window, whose `resized` borrows that state again (`window.rs` 736)
and panics ("already borrowed"; baseview's own handler borrow is shared and nests). So
`HostRoute::request` only records the size (the newest wins) and posts one flush to the
window's thread, which asks the host once the current event is over: libdispatch's main
queue on macOS (`dispatch_async_f`, from the main thread only), a thread timer on Windows
(`SetTimer` without a window, 0 ms). Where the callbacks are not reachable or nothing can be
posted (no host callbacks; X11, whose baseview window runs on a thread of its own) the window
resizes itself with `InnerSize`. The posted callback is code of the plugin's image. When the
editor closes, Windows kills the route's pending timers (`cancel_posts`; a `WM_TIMER` already
in the queue is not removed by `KillTimer`). macOS cannot take back a block on the main queue;
it finds no route and does nothing. The residual risk: a host that closes the editor and
unloads the bundle within the same turn of its event loop as a corner drag, before the posted
callback runs, would have it run unmapped code. Hosts keep a bundle loaded well past an
editor's close (Reaper unloads none while the project is open), and pinning the image for each
post (`dlopen` of itself, released by a block outside the image) was not worth the code. The size of each resize (egui's `resized`) goes into
the state (`editor_size`, logical points); the editor opens at it. No zoom presets (task-185).

**baseview's own requests (task-224).** Reaper crashed (stack overflow on the main thread,
the user's check of 2026-11-16, CLAP) when its FX window switched from the parameter list to
the plugin UI. nice-plug 0.4.2 spawns a CLAP editor in `gui.create` with no parent
(`wrapper/clap/wrapper.rs`, `spawn(None, true, ...)`; VST3 `attached` gives the parent), and
baseview 0.4.2 on macOS ignores `wait_for_parent`: with no parent it makes a window of its own
(`platform/macos/window.rs`, `create_window_standalone`), its view the window's content and
delegate, until `gui.set_parent` closes that window and moves the view into the host's. A
resize of that window calls the view's `window_did_resize` (`platform/macos/view.rs` 362-369),
which asks the host for the new size (`view_did_change_backing_properties` with `notify_host`,
371-405); when the host refuses, baseview sets the previous size back (`resize(previous)`, and
for its own window `setContentSize`, 243-273), which posts the window's resize notification
again, with the sizes swapped. So a host that sets a size between `gui.create` and
`gui.set_parent` (`gui.set_size`) and refuses `gui.request_resize` there gets the two sizes
asked and set back in turn until the stack ends: the crash report's cycle `window_did_resize`,
`view_did_change_backing_properties`, `BaseviewView::resize`, `-[NSWindow
_setFrameCommon:display:fromServer:]`, `NSWindowDidResizeNotification`. `tests/clap_gui.rs`
reproduces it (a host that refuses, `gui.set_size` before `gui.set_parent`: the requests
repeat until the test aborts at the 101st). baseview's Windows backend skips the request for
a size the host set (`platform/win/window.rs`, `resize_host_originated`); the macOS backend
has no such check. The fix is in the editor's proxy of the host's callbacks (`HostProxy`), not in
baseview: baseview's own requests are answered `Ok` without asking the host while the window
has no parent (no host window to resize; `HostRoute::set_parented` at `set_parent`, or at
spawn with VST3's parent), while the host's `set_size` runs (`HostRoute::host_sizing`), and
while another request is in progress (a host may answer one with `set_size` at once). baseview
then never sets a size back where that re-enters it; in the host's view, setting it back only
resizes baseview's view (`setFrameSize`), which posts nothing. The corner's request
(`HostRoute::flush`) still goes to the host and may be refused. The in-progress case accepts
a baseview request nested in any host callback that holds the callbacks, not only in a
`set_size`: were a host to resize the editor's view inside a request by other means than
`set_size`, baseview would keep that size and the host would not be told, so the two sizes
could differ until the next `set_size` (not seen in the hosts tried). VST3 cannot loop: its view is in
the host's from the start (nice-plug's `attached` spawns with the parent; no window of
baseview's own, no `windowDidResize`), so baseview's setting back is only `setFrameSize` on
its view, which posts nothing. (nice-plug's VST3 request can still fail: `wrapper/vst3/view.rs`
508-519 returns `Err` when the task cannot be posted or the plugin is gone.) The test also
covers the other transitions: `gui.set_scale`, sizes before and after `set_parent`, the
backing-properties notification with no change (a test cannot change the display's scale),
the host's window resized (which reaches no baseview code by itself: the view is not that
window's delegate and does not autoresize), hide and show, and the editor closed and opened
again; the host is never asked for a size it set.

**The panel's differences** (`embed.rs`; the standalone app draws as before, its screenshots
are pixel-identical):
- Hidden (`PLUGIN_HIDDEN`, and no parameters, section 3): the Pattern/Song switch, Rewind,
  Fast forward, Record, the bar display and its arrows, the loop switch and range, the tempo's
  arrows. Fast forward and the bar display are hidden beyond task-185's list because they act
  only in song mode; task-213 decides. The transport's background is the art part
  `bg_transport_plugin` (the standalone's without those, a well for Play and Stop).
- The tempo display shows the tempo orb plays at (the snapshot's: the host's, or the song's when
  the host reports none), read-only; in the preview too.
- The host position: a Bar display (3 digits) where the song position was
  (`embed::host_position`): while orb follows the host (the snapshot's `host_sync`: pattern
  mode, orb and the host playing) the engine's position is the host's (section 8.2), shown as
  bar `tick / 768 + 1` (bars of four quarter notes, whatever the host's time signature,
  task-206). In note mode, with the host stopped, and in the preview (orb free-runs and has no
  host position) it is dark, as the standalone's bar display in pattern mode (the standalone
  shows no position of its own in pattern mode, so the preview shows none). No Beat display: the first
  version had one beside the Bar (1 digit, `tick % 768 / 192 + 1`); the user removed it after
  the Reaper check (2026-11-16).
- Play and Stop enter and leave the preview (task-217, section 4.2). The Play light shows only
  whether the preview runs (`Embed::previewing`, set over the engine's play state after each
  sync in `App::show`): the host's start and stop do not change Play and Stop, and pattern
  mode following a playing host does not light Play (the user's Reaper check, 2026-11-16; the
  first build lit it from the engine).
- Options has only the standalone's two checkboxes. It had the mode as three radio items
  (task-217, task-218: "Latch Pattern Mode", "Gate Pattern Mode", "Note Mode"); the user
  removed them on 2026-11-17 when task-231 added the mode selector below, the only place to
  change the mode (orb-ui `embed::MODE_ITEMS` names the modes, `Embed::mode` / `set_mode`,
  `Cmd::SetPluginMode`).
- **Help > Plugin Usage** (task-221): the plugin's Help menu has Plugin Usage... first, then
  About; no Keyboard Shortcuts. It opens a scrollable window (`orb-ui/src/plugin_usage.rs`,
  resizable, 580 x 520 points at most and 200 x 150 at least, closed by its button or Escape)
  with the guide to the modes, channels, pattern selection, note mappings, host parameters and
  recipes. The text is `docs/plugin_usage.md`, embedded with `include_str!` and drawn from its
  Markdown subset, so the page and the file are one (the README links the file, the release
  notes can). The page ends with Steinberg's terms (assets/README.md, task-233; until task-278
  the file marked that section for the VST3 plugin only, the CLAP plugin's page left it out).
  The section: the official VST
  Compatible Logo, the variant for the dark window (`VST_Compatible_Logo_Steinberg_negative.svg`,
  drawn unaltered from the file's bytes, 96 x 61.7 points: above 15 x 10 mm, 57 x 38 px at 1x),
  "VST®" at the first VST of the page and "VST is a registered trademark of Steinberg Media
  Technologies GmbH.", all checked by tests (`plugin_usage::tests`, `tests/editor.rs`). The
  first release has no Standard Mapping on the page (the user, 2026-11-16: task-208/209
  deferred): it says only that the sections listen on channels 1..4. The standalone's Help menu
  is unchanged.
- **The mode selector** (task-231; the user's design of 2026-11-17): in the transport, after the
  Bar display. A gray knob (Shuffle's design at 1.6 times its size, no ticks) at transport
  (250, 26) with an engraved line per position running to a yellow LED (the mixer routing keys'
  lamp) and a label: "Latch Pattern" (row y 13, pointer at 45 degrees), "Gate Pattern" (26,
  90) and "Note" (39, 135). The selected mode's LED is lit, its label bold and bright, the
  others as Sync and MIDI. Art (orb-ui-art `parts.rs`, constants `PLUGIN_MODE_*`, mirrored and
  checked in orb-ui `embed.rs`): the body, lines and dark LEDs are in `bg_transport_plugin`;
  orb-ui draws the pointer (`knob_gray_pointer` turned), `led_mode_on` and the parts
  `mode_label_{0,1,2}_{off,on}` (`mode_selector`). Behavior: a click on a row (its line, LED or
  label) selects that mode; a vertical drag on the knob steps through the modes, one per
  `MODE_DRAG_STEP` (12) panel pixels from the mode the drag started in, up toward Latch and
  down toward Note; a click on the knob does nothing; no keys; no tooltips (the
  user's decision of 2026-11-17). It calls `Embed::set_mode` (`Cmd::SetPluginMode`), which sets `OrbParams::mode`,
  and the state keeps it (section 5). Nothing is hit while
  the file browser, a prompt or the error box is open. The standalone app has none of it.
- **The plugin format's mark**: one mark right of the selector, before the wordmark, right-aligned
  6 px before it, engraved (a dark copy at 0.38 opacity over a light copy 0.6 px lower): the
  plain text "VST3" in the panel's label font (Inter bold, 28 px; part `plugin_logo_vst3`,
  112 x 50 at transport x 344; the CLAP logo of the first releases went with the format,
  task-278). The user's decision of 2026-11-17 (task-232): Steinberg's official
  VST Compatible Logo does not fit the panel, so the text names the format (the guidelines'
  section 2 allow "VST" in a standard font to denote compatibility); the logo and the
  attribution go to Help > Plugin Usage only (task-233; the README and the package texts dropped
  them, task-239). VST® is a registered trademark of Steinberg Media Technologies GmbH. The editor
  knows its format from the wrapper in use: nice-plug's `GuiContext::plugin_api()` is `Vst3`
  (`Embed::format`, `EditorEmbed::format`); the stand-in host of `editor_preview` reports `Vst3`.
- The Sync LED is lit while the snapshot's `host_sync` is; the MIDI LED for 0.1 s after the
  snapshot's `midi_in` count changes (art part `led_transport_on`), in the preview too. The editor redraws at least
  every 50 ms while open so that the LEDs follow.
- File has New, Open and Info (no Save, Save As, Export); nothing asks to save (the song lives
  in the host's project). File > Open loads the file into the plugin (`OrbParams::load_song`),
  sets the song's control values as host parameters; File > Open and File > New keep the
  mode (task-228). The file browser is orb-ui's own, egui-drawn (task-046),
  so no native dialog is opened from the plugin window; its folders and the Mods folder are the
  standalone's settings file (`OpenRB-338/OpenRB-338.toml`, or the portable one next to the plugin file or bundle on Windows while `OpenRB-338.portable` is there, docs/building.md), shared; a save merges only its own
  change into the file as it is then, so open instances do not overwrite each other
  (task-229).
- Mods: a mod chosen (the Mods menu, a song's mod, Choose file) is read and given to the plugin
  (`OrbParams::set_mod`, embedded in the state). An editor that opens on a state's mod names a
  copy of its bytes in the temporary folder (`OpenRB-338/mods/<sha256>.rbm`), or the file an
  editor of the process chose, for the 808's mod window and Mods > Info. The editor has no
  View menu and no classic skin: View > Classic / Modern is the standalone app's only (task-267,
  docs/ui_spec.md section 21; the Theme Viewer of task-187 is gone).
- **No keyboard** (the user, 2026-11-16): hosts take keys from plugin windows, and the 303
  plugins surveyed (task-185) have no shortcuts. The editor acts on no key: no menu
  accelerators (no shortcut texts in the menus), no Program Synth / Select Patterns modes in
  Options, no Help > Keyboard Shortcuts, no computer-keyboard keys of the panel. The window
  passes every key on to the host (egui-baseview `KeyCapture::IgnoreAll`) except while a text
  field (the file browser's) has the focus; so the browser's arrows, Enter and Escape also
  reach the host while no text field has the focus (egui still sees them). Mouse gestures
  (drags, Shift-drags, double-click reset, the 909's Ctrl/Alt clicks) are the standalone's.
  The key forwarding itself is egui-baseview's and is checked in a host only.

**Repainting.** The panel asks egui for its repaints as in the standalone app (orb-ui's
repaint rule, docs/ui_spec.md section 16: the meter rate while the engine's side moves; the
plugin's LEDs and Bar every 50 ms, `embed::POLL`), but egui-baseview 0.9 passes those requests
to baseview's waker, and baseview 0.4.2 on macOS loses one made while the window draws (a zero
delay sets the display link running at once, `platform/macos/waker.rs` 20-26 and
`wrappers/appkit/main_thread.rs` 73-77, and the frame's end pauses it again,
`platform/macos/view.rs` 280-296). egui makes such a request at the start of the frame after
any immediate one (egui 0.37 `context.rs` 113-125), and the delayed requests of that frame then
skip the callback (`context.rs` 164-172), so the editor stopped drawing until the mouse moved:
LEDs, step lights, Sync/MIDI and the Bar froze (the user's Reaper check, 2026-11-16). The
editor replaces egui's repaint callback (`EditorApp::build`) with `editor::RepaintClock`: a
thread per open editor that keeps the earliest requested time and, at it, sets nice-plug's
`RepaintNotifier`, which egui-baseview's window timer polls every 1/64 s outside the frame and
turns into a redraw. With no request the thread sleeps and the window does not draw, so a
still panel costs no CPU; playing, it draws at the panel's meter rate. The thread ends when the
editor closes (`editor_closed`). Whether a window the host hides but keeps open still draws
is the display link's (it is paused while nothing is requested).

**Fonts.** egui 0.37 has no default features; the standalone app gets `default_fonts` (and the
system fonts as a fallback, `egui_system_font_provider`) from eframe's defaults. orb-ui asks
egui for `default_fonts` itself, and the editor adds the system font provider as eframe does.
Without them every egui text of the editor was drawn as boxes (the user's first Reaper try,
2026-11-16), which the screenshots did not show: the crate's dev-dependency eframe turns the
feature on for its tests and examples. `tests/fonts.rs` asks `cargo tree` for the features of
the plugin's normal dependencies and requires `default_fonts`; the editor debug-asserts that
egui has fonts.

**The vendored raw-window-metal.** wgpu's Metal surface (wgpu-hal 30 `metal/mod.rs`,
`raw_window_metal::Layer::from_ns_view`) puts a `CAMetalLayer` subclass into the editor's view.
raw-window-metal 1.1.0 registers that class with objc2's `define_class!` under the fixed name
`RawWindowMetalLayer` (`src/observer.rs`), and Objective-C classes are process-wide: the second
image of the plugin in one process (Reaper loaded the `.vst3` and the `.clap`, two copies of the
same binary, when both formats were used; any other plugin linking raw-window-metal does the
same) panicked opening its editor ("could not create new class \"RawWindowMetalLayer\",
perhaps a class with that name already exists?"). The panic is inside the host's call (CLAP
`gui.create`, VST3 `IPlugView::attached`), an `extern "C"` function of nice-plug, so it
aborted Reaper (the user's check of 2026-11-16, macOS arm64, Reaper 7.86, either order of
the two formats). The repository carries raw-window-metal 1.1.0 under `vendor/` (MIT OR
Apache-2.0, its licence files kept), used through `[patch.crates-io]` of the workspace
`Cargo.toml`, so it applies to the whole workspace: the standalone OpenRB-338 app (eframe on
wgpu) uses the patched crate too. The patch is not a minimal diff: `src/observer.rs` (marked
"OpenRB-338 patch") is rewritten from `define_class!` to `ClassBuilder` (about 300 lines),
and `src/lib.rs` changes in 3 places (`extern crate std`, the two uses of `ObserverLayer::new`).
It registers the class at run time with `ClassBuilder` under
`RawWindowMetalLayer_<address of a static of the image>_<n>`, taking the next `n` when a name
is taken (an image unloaded from the same address); the instance variable is a boxed
`Weak<CALayer>` and `dealloc` removes the observers as the original's `Drop` did. One behavior
differs: the key-value observer ignores an unexpected change (no change dictionary, no new
value, a value of the wrong type, an unknown key path) where the original panicked, since a
panic there would unwind into AppKit. Otherwise the layer behaves as the original (the
crate's own tests pass on the copy). Without a name, `define_class!` would
reuse the class of the first image (objc2's rule for generated names), whose code goes away
when that image is unloaded, so the patch uses a unique name instead. baseview names its view
class with a UUID already. The rest of the plugin's macOS dependency tree registers no class
under a fixed name (searched for `define_class!`, `declare_class!`, `ClassBuilder::new` and
`#[name = ...]` in the 277 crates; objc2-app-kit's `#[name]`s bind AppKit's own classes).
Upstream issue text (not posted): "raw-window-metal: `RawWindowMetalLayer` has a fixed class
name, so two dynamic libraries that each link raw-window-metal (audio plugins, e.g. two wgpu plugins) cannot both create a layer in one process: the
second `define_class!` registration panics. Please register the class under a name unique to
the image (e.g. with a random or address suffix through `ClassBuilder`), as baseview does for
its NSView subclass."

**The egui-baseview fork (task-292).** The workspace takes egui-baseview from the user's fork
github.com/pingw33n/egui-baseview, branch `OpenRB-338`, commit
e3db19695653fb05dead3916a553c514c1c88c23, through `[patch.crates-io]` of the workspace
`Cargo.toml` pinned to that commit (Cargo.lock's source is the fork's git URL at that rev; its
one user is nice-plug-egui 0.6.0). The fork is upstream commit 1f645a3
(codeberg.org/RustAudio/egui-baseview, the published 0.9.0) plus one change to
`src/renderer/wgpu/renderer.rs` `render`. 0.9.0 freed the frame's `textures_delta.free`
(egui-wgpu 0.37's `Renderer::free_texture`, which calls `wgpu::Texture::destroy` at once,
`renderer.rs` 790) after recording the frame's render pass but before `queue.submit` (0.9.0
`renderer.rs` 278-289), so a texture freed in a frame that drew it was destroyed under the
submitted draw: wgpu's "Validation Error ... In Queue::submit ... In a draw command, kind:
Draw ... Texture with 'egui_texid_Managed(1)' label has been destroyed", which panics in
wgpu's default error handler inside the host's call and aborted REAPER 7.8.6 on Windows
(0xc0000409, FAST_FAIL_FATAL_APP_EXIT, the user's report of 2026-11-24). The trigger was File
> Open, another folder, "Use as mods folder" with the editor open: `App::refresh_mods`
(`crates/orb-ui/src/menu.rs`) drops the mod window's splash picture (`self.splash = None`) in
a frame that has drawn it; any texture the editor drops in a frame that drew it did the same.
Choosing the current folder again drops nothing, and the standalone app never crashed: eframe's
egui-wgpu integration frees after the submit. The fork submits, then frees, then presents. Not
changed by the fork: a frame whose surface acquire returns `Occluded`, `Timeout`, `Outdated`,
`Suboptimal` or `Lost` returns before the free (in 0.9.0 too), so the textures freed in that
frame stay allocated until the editor closes. `tests/texture_free.rs` replays the renderer's
frame on a headless wgpu device (the old order fails validation with "destroyed", the fork's
passes; skipped with no adapter) and requires the resolved egui-baseview's source to call
`free_texture` after `.submit(` and before `.present(`. No upstream pull request has been
opened.

**No panic reaches the host.** Every method of `OrbEditor` (`spawn`, `size`, `resize_hint`)
and of `OrbHandle` (`run_until_closed`, `set_parent`, `show`, `hide`, `set_size`,
`adjust_size`, `set_fallback_scale_factor`, the main-thread, key, state and parameter
callbacks), the egui app's `build`, frame, `resized` and `editor_closed`, and the posted
resize request run under `editor::guard` (`catch_unwind`): a panic is logged (`nice_error!`)
and becomes an error or a safe fallback (no adjustment, the default size, the default resize
hint, the state's size unchanged), so the host gets no editor (`gui.create` false) or an
empty window instead of an abort. nice-plug has no hook of its own for this. (The test of
this, `tests/clap_gui.rs`, went with the CLAP format, task-278.)

**The panel's commands** (`PluginPort`, orb-ui's `link::Port`): a control's message becomes its
host parameter (`params::of_command`; `ParamSetter` begin, set, end), so the host records and
saves it and the process side sends it to the engine (section 3); during the preview it goes
to the preview set instead (`set_preview_value`, no gesture, section 4.2); Play and Stop are
`set_preview(true | false)`; a pattern button on the
pattern its parameter already holds sends the message itself (`select_pattern`, section 6); a
pattern edit goes to `OrbParams::edit`, the Vintage flag to `set_vintage`, the mod's startup
sound to `OrbParams::preview`; the song mode, record, loop, locate and tempo messages are
dropped. The panel takes the engine's values again (the snapshot after the last engine call,
which shows host automation, notes' pattern choices and the Standard Mapping's changes) only
8192 engine frames after its last command (`FOLLOW_FRAMES`, longer than a host buffer), since a
parameter reaches the engine through the host's next `process`.

**The song in the editor.** `OrbParams::revision` counts loads, File > New and state restores;
when it changes the editor shows the song anew from `OrbParams::view`: the patterns and a
snapshot of a throwaway engine built from the document and the parameter values (one stopped
call of 64 frames, not heard), so an editor opened before the first activation, or right after
a load or a restore the host has not run yet, shows the song. The plugin's garbage queue is now
emptied on the main thread at every command and every editor frame (`OrbParams::collect_garbage`).

**Screenshots without a host.** `cargo run --release -p orb-plugin --example editor_preview --
[SONG.rbs] [--size WxH] [--ppp S] [--host-playing] [--midi] [--mode latch|gate|note] [--preview]
[--click X,Y]... [--scroll Y] [--screenshot OUT.png]` shows the editor in a window of its own with a
stand-in host that runs the process side (no sound) and applies parameter changes at once;
`--preview` enters the preview, `--click` clicks a point of the window (one every 6 frames from
frame 10: `--click 104,11` opens Options at 1:1; `--click 150,11 --click 190,39` opens Help >
Plugin Usage); `--scroll Y` scrolls the window under the pointer down by Y points after the
clicks (task-221); the screenshot is taken 24 frames and 1.5 s after the last click (16 frames
more after a scroll), so a window opened by it has faded in and the scroll has settled, also on
a loaded machine. `--ppp 1` is the system's scale (2 physical pixels a point on a Retina
Mac), `--ppp 2` twice that.

**Tests** (`tests/editor.rs`, headless egui with a stand-in `GuiContext`): the editor shows the
plugin's song; a knob drag and a pattern button set their parameters through the host; Play
and Stop enter and leave the preview, and in it a knob drag and a pattern button go to the
preview set with no host gesture; the Play light is the preview's, not the host transport's;
the Options menu has no mode items; the Help menu has Plugin Usage, which opens the
guide with the trademark terms and fits the smallest window (task-221); a click on each mode selector row sets the mode, a vertical drag on its knob steps the modes, and the
selector's mode is saved with the state (task-231); the editor reports its plugin format; the hidden
controls take no clicks; no key acts; the LEDs and the Bar display follow the snapshot (dark
Sync and Bar in the preview, the MIDI LED still lit by input); File > Open keeps the mode and New
asks nothing; a mod from the Mods menu goes into the state; the File menu opens from the bar;
the corner asks for sizes of the rule, through the host's callbacks after the frame (a host
that sizes the window inside its request finds the window's state free); a window
1.5 times larger still hits the controls; the size rule of the host's resizing and the state's
size. `tests/fonts.rs`: the plugin's normal build has egui's fonts. The real-host checks
(Reaper, VST3) are the user's (task-201's notes).

## 11. Gate Pattern Mode (task-218)

The user's design of 2026-11-16, after Phoscyon's Host Note mode and ABL3's MIDI Trigger mode:
a section plays its pattern only while a note 0..31 on its channel is held, so a DAW clip's
note lengths are the arrangement and a gap is silence. Latch Pattern Mode (section 8.2) is
unchanged; the preview overrides Gate as it overrides the other modes (section 4.2).

**What it does** (`src/core.rs`; tests `tests/gate_mode.rs`):

- **A gate per section.** Each section (303 #1, 303 #2, 808, 909 on MIDI channels 1..4) has a
  gate, closed until a note 0..31 on its channel is held; the sections gate independently. A
  closed section is silent and does not step; the song itself plays as in Latch (following
  the host while it plays).
- **The note-on opens it at once**, at the first 64-frame block at or after the event's frame
  (section 4,
  the same resolution as every host event), on pattern n+1 (A1..D8), from the step at the
  song's position: with the host playing the host-position step, mid-pattern in step with the
  song, following loops and seeks as Latch does. The step at the position sounds at once,
  wherever in the step the note comes (a 303 note of a step entered late ends by the step's
  end, so the next step comes on time), **except near a step boundary** (the user's choice of
  2026-11-16): a note-on within 5 ms before the next step, at most 12 ticks (a quarter step),
  takes that step, which then plays on time. Under task-218's timing a clip note on the grid
  arrived while the engine's position was up to one block (1.45 ms) before its step, because
  the event was applied to the block that held it; without the snap it would play the
  previous step's last instant (and a 303 note of it that swallows the next step's start).
  Since task-227 an event reaches the first block at or after its frame, so a grid note finds
  the position at its step or up to a block after it; the snap stays for a host whose notes
  come a little before the grid. 5 ms covers a block with room for host timing jitter and
  stays below the ~10 ms at which an early note is heard as a separate event; the 12-tick cap
  keeps the snap near the boundary at fast tempos (5 ms is 16 ticks at 999 BPM, where a step
  lasts 15 ms). The window is in time, not ticks, so it means the same at any tempo below 750
  BPM.
- **The note-off closes it at once**: the section fades out linearly over the first 64-frame
  block at or after the note-off (1.45 ms; gain `(63 - i) / 64` on sample `i` of the
  section's channel bus, before the inserts) and is silent from the block after (the device's
  voices get slot 6 at the fade's end, and its bus is zeroed; a delay or insert tail of what it
  played before continues). The fade is the user's decision of task-222 (2026-11-16): the
  first build's hard cut stepped to silence by up to 0.036 (303), 0.047 (808), 0.037 (909) of
  full scale at the default levels,
  an audible click; with the fade the step is below 0.002 (measured 0.0016, 0.0007, 0.0008).
  The bound depends on the level: the ramp's last non-zero sample has gain 1/64, so the step
  is about the section's peak / 64, below 0.002 only for a peak below about 0.128 (the 303's
  default peak, 0.125, passes with little margin); a full-scale section gives about 0.016.
  With the host stopped the song is stopped only after the fade (a Stop's slot 6 would cut
  it). The fade is the Gate stop's only: entering Gate fades the sections out too, nothing else
  changes.
- **The last note pressed wins.** Of several notes held on one channel the last note-on
  selects the pattern (a switch takes effect at once, at the position's step, as an opening
  does); releasing it falls back to the newest note still held, releasing another changes
  nothing, releasing the last one closes the gate. A velocity-0 note-on is a note-off; notes
  above 31 do nothing. The held notes are a set: a note pressed twice is released by its first
  note-off.
- **With the host stopped** a held note plays too: the first gate that opens locates the song
  to 0 and plays it (step 1), free-running at the host's tempo (the song's when the host reports
  none); the song stops when no gate is open, and the next note starts at step 1 again. A gate
  that opens while another section plays joins it in step, at the running position (the
  user's choice of 2026-11-16: the sections stay in step; the song is located to 0 only when
  nothing plays). A host that stops while notes are held leaves the song playing on from where
  it is. "Stopped" is the transport of the note's own call: a note in the call where the host
  starts, before the first block that sees it play (a note at the host's position 0), leaves
  the start to that block's sync, at the host's position; it does not start the song
  free-running first, which the sync's Stop (slot 6) would cut (task-226).
- **The On/off switch is the user's mute** (the supervisor's clarification of task-185's rule
  that notes never change parameters): the gate never changes it, nor any parameter, and sends
  no host gesture. A section sounds only when its switch is on and its gate is open. Opening a
  gate takes the switch's state at once (slot 2 reads it); a switch changed while the gate is
  open acts at the next bar line, as in the original. The panel's On button does not follow
  the gate; a closed section shows no step light (orb-player `Snapshot::gate_closed`, orb-ui
  `link.rs`).
- **The panel's pattern** (the user's option A): a held note shows its pattern on the panel
  through the pattern controller's message, as a Latch note does, without changing the
  pattern parameter, and after the note-off the panel keeps showing it. What plays is always
  the held note's pattern: a panel click or automation of the pattern parameter sets the
  parameter and what the panel shows, but plays only through a gate note.
- **The Standard Mapping** (on, on its own channel): its pattern notes only select (the
  pattern controller), in both pattern modes; in Gate they open no gate and do not change the
  held note's pattern. Its other actions are as in Latch.
- **Mode changes.** Entering Gate (or leaving the preview in Gate) closes the four gates;
  leaving it (or entering the preview) opens every section again; either forgets the held
  notes (the preview ignores note-offs, so a note held through it sounds again only when
  pressed again). A song loaded or restored while gating gets the gates as they are.

**How** (not in the original; songs, Latch and Note mode keep the original's code path, and the
full corpus renders as before, 175 of 175 equal):

- orb-engine `frame::Gate {on, open, pattern}` on each step device (`ChannelDevice::gate`,
  `set_gate`; the 303's, 808's and 909's `pattern_gate`), off by default. On and closed: slot 1
  (`step`) returns at once and `Frame::block`, right after slot 14, fades the channel's bus
  over the first block (`Frame::gate_fade`, `gate_fade_gain`, f64 products stored as f32) and
  runs slot 6, then zeroes it.
  On and open: the bar-line latch in slot 1 (`0x100254D3` 303, `0x10027772` 808, `0x1002ABB0`
  909: `cur_pattern = controller 1` at `(tick/48)&15 == 0`) and slot 2 (`0x100255E0`,
  `0x10028030`, `0x1002B670`) take the gate's pattern instead of controller 1. Opening or
  switching runs slot 2 at the current tick, ends a 303's pending note (its pending gate-off,
  as a pattern wrap in slot 1 does), and decides the step (`frame::gate_arm` with
  `GateClock::lead`, the fractional tick plus the window `GATE_SNAP_MS`, `GATE_SNAP_MAX_TICKS`):
  at once, `wrapped` is set and, while the song plays, slot 1 runs at once with `(t,
  shuffled(t), tempo)`, so the step sounds from the next block, not from the next tick change
  (a tick is 1.8 blocks at 120 BPM); a 303's gate-off is then capped at the step's last tick.
  Snapped, the current step is recorded as the last one, so that the next triggers on time.
- orb-player `Command::Gate {device, gate}`. The plugin keeps each channel's held notes in a
  fixed array (no allocation in `process`) and, before every block and every section note,
  sends any engine gate that differs from what the held notes say: this covers a newly loaded
  song (its gates are off). (A full garbage queue no longer refuses a `Gate`, task-225,
  section 1.)

## 12. The pattern drag-out (task-203)

A press on one of a section's eight pattern buttons that moves 6 points or more (`embed::DRAG_OUT`,
egui's own click tolerance) starts the operating system's drag of a MIDI file of that pattern
(the button's pattern in the bank shown). The user drops it on a track of the host, which
makes a MIDI item of it, as a MIDI file dragged from the file manager. The details:

- The press has already selected the pattern at mouse down, as a click does (the original's
  pattern buttons act at mouse down), so a drag-out also selects the pattern it drags (the
  user kept this, task-223, 2026-11-16). A click without movement selects the pattern and drags
  nothing. The bank buttons do not drag.
- Only the plugin's panel drags (`Embed::drag_pattern`, `App::drag_out` in orb-ui's `embed.rs`).
  The standalone app is unchanged.
- There is no "Save MIDI file", no song or all-patterns export and no import (task-185).

**The clip** (`src/clip.rs`) is a Standard MIDI File:
- Type 0, one track, 192 ticks a quarter note, so a MIDI tick is an engine tick.
- The track is named after the section and the pattern (`303 #1 A1`, `808 C2`), with a 4/4
  time signature and no tempo, so the host's tempo plays it.
- The end of the track is the pattern's length: 48 ticks a step, the length byte cut to 1..16.
- The notes are on the section's channel (1..4: 303 #1, 303 #2, 808, 909) in the note mapping
  of `docs/midi.md` section 5. The 303's notes follow its gate, the drums' are 24 ticks long,
  the velocities are the export velocities, and shuffle is baked in (all in `docs/midi.md` 5.6).
- The data is the plugin's, at the moment of the drag:
  - its pattern arrays: the state's, or the song's own before the first build;
  - the values the panel plays (the preview set during the preview): the Shuffle knob and the
    808's five shared-slot switches.

**The clip is meant for note mode.** Played on a track into an instance in note mode (the mode
selector's Note), it plays the pattern as the pattern plays: for the 303, bit for bit the step path,
except for three things the clip cannot carry (`docs/midi.md` 5.6): a tie's own accent, a
rest's effect on the release tail, and a played step after a rest that slid. A slide out of
the last step into the next loop is lost too, as the clip is one loop. In the pattern modes,
notes 0..31 on a section's channel are patterns: Latch Pattern Mode selects them (section 8.2)
and Gate Pattern Mode plays them while held (section 11). A 303 clip's notes 24..31 would select
or gate patterns D1..D8, and its other notes and every drum note (36 and up) would do nothing.

**The file.** The clip is written to `OpenRB-338/clips/<pid>-<ms>-<n>/` in the temporary folder,
under the name the host gives its item: `303-1 A1.mid`, `303-2 B4.mid`, `808 C2.mid`, `909 D8.mid`.
Each drag-out has a folder of its own, so the name can repeat. The folders are swept
(`clip::sweep`): one older than 10 minutes (`clip::KEEP`) is removed at the next drag-out and at
every editor close, from any process's editors. Hosts read a dropped MIDI file at the drop, a
few seconds at most. On Windows, a folder whose file a host still holds open stays until a
later sweep. A host that kept a reference to the dropped file instead of importing its notes
would lose the clip at the sweep; Reaper is to be checked (task-203's review).

**The native drag** (`src/drag.rs`). egui and baseview cannot start one:
- The panel's frame only records the request. The editor's embed writes the clip and leaves its
  path in `drag::Pending`; after the frame, `EditorApp::ui` calls `drag::start` with
  baseview's window handle (raw-window-handle 0.6).
- Each platform begins the drag once the current event is over, as the resize corner's request
  (section 10): baseview's window state is borrowed for the whole frame, and a drag runs the
  event loop.
- Every callback runs under `editor::guard`, so no Rust panic reaches the host.

Per platform:
- **macOS** (`drag/macos.rs`), a block on the main queue (`dispatch_async_f`):
  - `-[NSView beginDraggingSessionWithItems:event:source:]` on baseview's view, with one
    `NSDraggingItem` whose pasteboard writer is the file's `NSURL`, and the file's icon
    (`NSWorkspace iconForFile:`) at the pointer.
  - The event is a left-mouse-down made at the pointer (`+[NSEvent mouseEventWithType:...]`), as
    JUCE does for external drags: the event that the frame saw is gone by then. Nothing starts
    when the button is up already (`+[NSEvent pressedMouseButtons]`).
  - The source answers copy (`NSDragOperationCopy`) for any destination. Its class is registered
    at run time under a name unique to the loaded image (`OrbClipDragSource_<address>_<n>`,
    `ClassBuilder`), for the reason of the vendored raw-window-metal (section 10): the `.vst3`
    and another wgpu plugin may be loaded in one process.
  - The session takes the button's release from the view. When it ends
    (`draggingSession:endedAtPoint:operation:`), the source sends the view a made-up `mouseUp:`
    at the pointer, so that egui sees the release.
  - The residual risk of a posted block of an unloaded image is the resize corner's.
- **Windows** (`drag/windows.rs`), a thread timer of 0 ms (`SetTimer` without a window):
  - OLE's `DoDragDrop` (after `OleInitialize`, balanced) with a data object and a drop source,
    COM objects made by hand: vtables of `extern "system"` functions and reference counts.
  - The data object offers the file as `CF_HDROP` in global memory (`DROPFILES` with the wide
    path), and its `EnumFormatEtc` is the shell's `SHCreateStdEnumFmtEtc`. `SetData`, the advise
    sink and `GetDataHere` are not supported.
  - The drop source drops at the release of the left button (the right one when the buttons are
    swapped) and cancels on Escape.
  - Nothing starts when the window is gone or the button is up already. The editor's close
    kills the pending timers.
  - `DoDragDrop` takes the button's release: after it the window is posted a `WM_LBUTTONUP` at
    the pointer.
- **Linux, X11** (`drag/x11.rs`): the source side of XDND version 5, offering `text/uri-list`
  (the `file://` URI, percent-encoded) and `TARGETS`, on an X connection (x11rb's
  `RustConnection`) and a thread of its own:
  - baseview's window runs its own event loop, which would swallow the target's replies, so the
    drag has an invisible input-only window as its source window and the `XdndSelection` owner.
    A property change gives it a server time.
  - The pointer is not grabbed. The press keeps it with baseview's window (an implicit grab),
    which therefore also gets the release.
  - The drag polls the pointer every 10 ms (`QueryPointer`). The target is the first window
    with `XdndAware` from the root down through the children under the pointer.
  - `XdndPosition` waits for the previous `XdndStatus`. The drop is made when the first button
    is up and the target has accepted; otherwise the target is left.
  - The drag waits up to 5 s for `XdndFinished`. Escape does not cancel, and a drag ends after
    two minutes in any case.

**What was verified** (2026-11-16):
- The clip and the panel's gesture: by the tests below, on macOS.
- macOS: the bundles build and validate. The drag into a host is the user's check in Reaper
  (task-203 AC #2 and #4).
- Windows: `cargo xwin clippy --target x86_64-pc-windows-msvc -p orb-plugin --all-targets -D
  warnings` passes. The data object's test (Windows only; CI runs it) passed on the Windows 11
  test machine, with the clip tests (`cargo xwin test --no-run`, the exe run there: 12
  passed). No drag into a Windows host was tried: `DoDragDrop`, the drop source in a real drag
  and the posted `WM_LBUTTONUP` are unverified.
- Linux: the X11 module passes `cargo clippy --target x86_64-unknown-linux-gnu -D warnings` in a
  scratch crate of its own (the workspace's Linux check needs a C cross toolchain). It was not
  run against an X server or a host.

**Tests:**
- `src/clip/tests.rs`: the written file read back for a 303 pattern (pitch with Up and Down, the
  gate, a slide, a tie, accent, a rest, a rest after a slide, a slide out of the last step), an
  808 pattern (both positions of the five switches, AC, the AC row alone) and a 909 pattern
  (the four levels, the flam values 3, 4, 5..255, AC bytes 1 and 2).
- Also there: shuffle; the step and gate times against `seq::shuffled` for every shuffle value;
  the 303 clip played back in note mode against the step path, bit for bit; the three limits;
  names, lengths and the temporary files' sweep.
- `tests/editor.rs`: a press-and-move gives the pattern's clip once per press, and the press
  selects the pattern; a click without movement selects it and drags nothing; a bank button
  drags nothing.
- `drag/windows.rs` (Windows): the data object read as a drop target reads it, and the drop
  source's answers.
- `drag/x11.rs`: the URI list.

## 13. The per-device outputs (task-256)

Besides the main stereo output, named **Mix**, the plugin has four stereo aux outputs named
**303-1**, **303-2**, **808** and **909** (VST3 aux buses 1 to 4 after the main bus 0; one layout, `AUDIO_IO_LAYOUTS` in `src/lib.rs`,
names `OUTPUT_NAMES`). Requested by the user on 2026-11-20 (item 7, "multi-out", of task-196's
deferred features). The tap point is the user's decision of 2026-11-21 (task-259): ReWire's.

**The tap.** Each bus is its mixer channel's signal where ReBirth's own ReWire output for the
channel takes it (`docs/rewire.md` section 4, [C]): the device's bus after the channel's
inserts (the distortion, the PCF and the compressor where the mixer routes them to the
channel, in that order, 0x100135AF..0x100135F1), before the channel's peak meter (0x100135FA),
its delay send (0x1001364A), its level, pan and on switch (the gain smoothing 0x100136B7 and
the pan-and-mix loop 0x10013742). The signal is mono; the plugin puts it on both sides of the
bus. The bus is a stored f32 block (`unk_1010012C`), so the tap is an exact copy
(`Frame::block_taps`, `crates/orb-engine/src/frame/mod.rs`). Not on the buses: the channel's
level, pan and on switch, the delay (a send whose return goes only into the master,
0x10013844), the master compressor (0x10013883) and the master gain (0x100138AE).

Unlike ReWire, the buses are copies: a connected bus does not take its channel out of the Mix,
and the Mix is the same whatever the host connects. The panel's channel on switch (the
mixer's ON) does it instead: a channel turned off is out of the Mix (its gains go to the 1e-6
guard, `lev = on ? level : 0`) but still on its own bus, so a device can be taken out of the
Mix and processed on its own track. Its delay send goes too (`T[lev]*T[send]`), as ReWire's
does not: an off channel has no echo in the Mix.

With the send off and no master compressor the Mix is
`int16(f32(f32(1e-6 + bus*g) * m))` for a device playing alone, `g` the channel's ramped gain
(level and pan) and `m` the master gain (0.6338 at the default master level 100). The buses
are f32 at full scale 1.0 and not clamped (the Mix is the original's int16 output over 32768).
A channel whose pattern gate is closed (Gate Pattern Mode, section 11) has a silent bus; the
preview player (section 4.2) is mixed into the Mix only.

**The Mix does not change.** The main output is bit-identical whether the host connects none,
some or all of the buses: `block_taps` runs `block`'s code and only copies each channel's bus,
and the main output's path through the player and the 44.1 kHz ring or the resampler is
untouched. `cargo test -p orb-engine --test corpus_golden` and `orb compare --manifest
tests/refs/manifest.json` stay EQUAL (the corpus renders through `Engine::render`, which
renders no taps).

**How they flow.** `Engine::render_taps` (a sibling of `render`, with its own leftover tap
block for a partial block) writes the taps frame-interleaved, `TAP_WIDTH` = 4 values per frame
(channel c at `[4i + c]`). `Audio::enable_taps` (orb-player) allocates the call's tap buffer
on the main thread (`Core::new`, at the activation); every `render_call` then fills it
(zero for a faulting call), `Audio::taps`. The process side keeps each block's taps beside its
int16 block: at 44.1 kHz in a ring slot of their own (copied with the block), resampled in a
ring indexed as the int16 frames and interpolated with the same weights. So the buses have
the Mix's latency and timing. `Core::process_aux` takes the host's buses; `process_with` is
`process_aux` with none.

**Unconnected buses.** nice-plug gives an empty slice for a bus without a host buffer
(`wrapper/util/buffer_management.rs`); `process` passes such a bus as `None`, and nothing is
written for it: no copy at 44.1 kHz, no interpolation when resampled (the per-frame cost is the
copy of the block's taps into the ring, 1 KiB a block). The engine copies the taps whatever is
connected (one block copy per channel). Nothing allocates in `process` (`tests/no_alloc.rs`
connects the buses in every combination). One caveat of nice-plug 0.4.2: it calls the
plugin's `process` only when every output bus has a buffer (`buffer_is_valid` in the VST3
wrapper), so a host that passes a null buffer for an unconnected bus would get no
processing at all, the Mix included; and it reads one bus past the count a host passes when
that count is lower than the plugin's (`aux_output_idx > numOutputs`, an off-by-one). Hosts
pass every bus they are told of; whether REAPER and Ableton do is the user's check (task-256
AC 5). Also of nice-plug 0.4.2: the main output is named after `names.main_input`
(`AudioIOLayout::main_output_name`), so the layout sets both to "Mix".

**Tests.**
- `crates/orb-engine/src/frame/tests.rs`: the taps are each channel's bus after an insert,
  exactly, the same for different gains (one channel at the guard, as when turned off), and
  the master is bit-identical to `block`'s; `render_taps` across the leftover block gives
  `render`'s int16 and the taps of one whole-call render.
- `crates/orb-plugin/tests/outputs.rs`: the Mix of `full_mix` (every device, the delay, the PCF
  and the master compressor) is bit-identical with none, some or all of the buses connected,
  at 44.1 and 48 kHz and buffers of 1 to 4096 frames, and a bus does not depend on the buffer
  size; a device alone with an insert (the distortion on the 303-1 in `dist_a100_s64`, the PCF
  on the 909 in `pcf_on_909`, the compressor on the 808 in `comp_ch_808`) reproduces the Mix
  from its bus within one LSB, with the same signal on both sides and the other buses silent
  (an idle 303 outputs about 3e-6); the 303-1's level, pan and on switch change the Mix and
  leave its bus bit-identical, and with the channel off the Mix is silent.
- `src/resample.rs`: the buses are resampled as the main output, independent of the chunks.
- `tests/exports.rs`: the VST3 buses (count, names, stereo, which is main), and the VST3 host
  passes all five buses.
