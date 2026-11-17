# The VST® 3/CLAP plugin (`crates/orb-plugin`)

VST is a registered trademark of Steinberg Media Technologies GmbH.

OpenRB-338 as an instrument plugin, milestone m-7. The product decisions are task-185's final
summary (2026-11-15/16); this document describes how the plugin implements them. task-197 built
the core described here; task-198 the host sync, the pattern selection by notes and the Standard
Mapping (section 8); task-200 note mode (section 9); task-201 the editor (section 10); task-203
the pattern drag-out (section 12); packaging and the validators are task-204. task-217 (the
user's design of 2026-11-16, task-216) replaced the mode parameter `g1` with the pattern/note
mode setting and made the panel's Play/Stop the pattern preview (section 4.2). task-218 renamed
pattern mode Latch Pattern Mode and added Gate Pattern Mode (section 11).

## 1. What runs

One instance is the whole ReBirth: the two 303s, the 808, the 909, the effects and the mixer,
the engine of `orb-engine` driven as orb-player drives it in the standalone app. The plugin is
built on nice-plug 0.4.2 (codeberg.org/RustAudio/nice-plug, ISC), the community fork of
nih-plug, with its `vst3` and `editor` features (no logger). Its VST3 bindings are the `vst3`
crate 0.3.0 (MIT OR Apache-2.0); the editor is on nice-plug-egui 0.6.0 (ISC, egui-baseview
0.9 with wgpu 30, section 10). These licenses are compatible with the repository's
GPL-3.0-only.

| File | Thread | What |
|---|---|---|
| `src/lib.rs` | both | the nice-plug `Plugin`, `ClapPlugin` and `Vst3Plugin`, the exports |
| `src/instance.rs` | main | the parameters, the document the state saves, the editor's calls, the engine build |
| `src/core.rs` | audio | the 64-frame block scheduler, the parameter changes, the mode and the preview |
| `src/resample.rs` | audio | the resampler for host rates other than 44.1 kHz |
| `src/params.rs` | both | the parameter table (a constant) |
| `src/editor.rs` | main | the editor: orb-ui's panel in the host's window (section 10) |

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
| CLAP id | `io.github.pingw33n.openrb-338`; features instrument, synthesizer (task-185's Instrument\|Synth) and stereo (the port layout) |
| VST3 class id | `EFAB08AF-B39D-4399-8A80-3DC1F84ED3D6` (made once with `uuidgen`, 2026-11-16; `VST3_CLASS_ID` in `src/lib.rs`); category Instrument\|Synth |
| URL | https://github.com/pingw33n/OpenRB-338 |
| Ports | stereo out, no audio in, one MIDI input (`MidiConfig::MidiCCs`: notes and control changes, section 8) |

`tests/exports.rs` loads the built library and checks them through the CLAP and VST3 C APIs.

## 3. Parameters

One stepped integer parameter per control the plugin's panel shows (119): every controller of
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
  samples (`tests/timing.rs`). A call of no frames takes no changes; they are seen at the next
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
  as the same note later (`tests/first_note.rs`; through the CLAP and VST3 wrappers,
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
| `editor_size` | the editor window's size in logical points (section 10); absent: 624 x 532 |
| `remote_enabled` | whether the Standard Mapping (Remote MIDI) is on; false by default and when absent (section 8) |
| `remote_channel` | the Standard Mapping's MIDI channel, 0-based (15: channel 16, the default and when absent); a section's channel (0..3, MIDI 1..4) restores as 15 with `last_error` set, the rest of the state kept (section 8) |
| `mode` | `"latch"`, `"gate"` or `"note"` (section 4; a new instance: note; task-217's `"pattern"` is no longer read) |
| `preview` | the preview set (section 4.2): an object of parameter id to value, or null before the first preview; an id it lacks takes the parameter's value |

The plugin is not released: no state of an earlier build is migrated, recognized or tested
(the user, 2026-11-16: no compatibility work with old plugin states until the plugin is
released on GitHub Releases; the format changes freely). A state from before task-217 lacks
`mode` and does not parse (the restore gives a new song and `last_error`).

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
and `OpenRB-338.clap` under `target/bundled/` (`xtask/` on nice-plug-xtask 0.1.1, `bundler.toml`
for the bundle name, the alias in `.cargo/config.toml`). The xtask finds the workspace from its
own manifest: nice-plug-xtask's own entry point takes the topmost ancestor directory with a
`Cargo.toml` as the workspace, which is the wrong one for a checkout inside another (the
worktrees under `.claude/worktrees/`). Signing, the packages and the validators
(clap-validator on the CLAP bundle, pluginval on the VST3 bundle) are task-204: docs/building.md,
"The plugin".

The tests (`cargo test -p orb-plugin`; `tests/` and the resampler's unit tests): the buffer
sizes and the 48 kHz path (`timing.rs`), the parameter list and defaults (`params.rs`), the state
(`state.rs`, with a test mod made from the standard mod: renamed, the 808's and 909's bass drum
and first snare swapped), no allocation in `process` (`no_alloc.rs`, a counting allocator, and
the process side's sources checked for lock types), and the exports (`exports.rs`): it builds
the library, loads it with the dynamic loader, checks the CLAP descriptor and the VST3 factory's
class info, and runs a minimal CLAP host that restores a state with a song in pattern mode,
activates at 44.1 kHz, processes with a CLAP transport that plays (no tempo, no position), and
gets the same samples as the process side run directly. Loading in a real host (Reaper) has not
been tried by the tests. `sync.rs` (task-198) runs a simulated host transport (section 8),
`note_mode.rs` (task-200) note mode (section 9), `editor.rs` (task-201) the editor headless
(section 10), `preview.rs` (task-217) the preview and its set through the process path
(section 4.2).

## 8. Host sync, pattern selection by notes, the Standard Mapping (task-198)

In this section "pattern mode" is the plugin's pattern modes (Latch and Gate, task-218): both
follow the host the same way; the section notes differ (Latch: below; Gate: section 11).

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

## 10. The editor (task-201)

The editor is orb-ui's panel, drawn by the same code as the standalone app
(`crates/orb-ui/src/app.rs`), in the host's plugin window: nice-plug-egui's `EguiEditor` on
egui-baseview with wgpu (the wgpu of eframe, 30.x, and egui 0.37 as orb-ui). orb-ui takes eframe
only in its default `app` feature, so the plugin links no eframe and no winit. `src/editor.rs`
has the pieces; orb-ui's side is `crates/orb-ui/src/embed.rs`.

**The window.** The standalone app's menu bar (File, Mods, Options, Help; egui-drawn, 22
points high, `MENU_H`) on top, not scaled, and the panel under it, scaled to the window's width:
the window is these two and nothing else (no desk around the panel). Its height is therefore
`round(width * 510 / 624) + 22` points (`embed::window_height`), which no fixed aspect ratio
expresses. So `Editor::resize_hint()` says resizable both ways with a minimum of half the panel
(312 x 277) and no aspect ratio, and the editor's handle (`OrbHandle`, egui's handle with its
own `EditorHandle::adjust_size`) gives the host the closest size of the rule: nice-plug calls
`adjust_size` from CLAP `gui.adjust_size` and from VST3 `IPlugView::checkSizeConstraint`
(nice-plug 0.4.2, `wrapper/clap/wrapper.rs` and `wrapper/vst3/view.rs`), so no patch of
nice-plug is needed. The side the host changed more (relatively) decides the width
(`editor::adjust_logical`). `EditorHandle::set_size` (egui's) applies the host's size.

**The resize corner.** An invisible 14 x 14 point corner at the window's bottom right (the
right wood's foot, no control there): the cursor turns to the diagonal resize arrow over it, a
faint grip of three diagonal strokes (`embed::paint_grip`) shows while it is hovered or dragged,
and a drag asks for the size of the rule at the larger of the two scales the drag gives
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
  notes can). The VST3 plugin's page ends with Steinberg's terms (assets/README.md, task-233);
  the file marks that section with `<!-- vst3-only -->` and `<!-- /vst3-only -->` and the CLAP
  plugin's page leaves it out (the editor knows its format, `Embed::format`; the file keeps it,
  the user's decision of 2026-11-17). The section: the official VST
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
  CLAP logo (MIT) in the CLAP plugin, and the plain text "VST3" in the panel's label font
  (Inter bold, 28 px) in the VST3 plugin (parts `plugin_logo_vst3`, `plugin_logo_clap`, 112 x
  50 at transport x 344; the CLAP data in `orb-ui-art/src/plugin_logos.rs`; sources and terms
  in `assets/README.md`). The user's decision of 2026-11-17 (task-232): Steinberg's official
  VST Compatible Logo does not fit the panel, so the text names the format (the guidelines'
  section 2 allow "VST" in a standard font to denote compatibility); the logo and the
  attribution go to the README and Help > Plugin Usage (task-233). VST® is a registered
  trademark of Steinberg Media Technologies GmbH. The editor knows its format from the wrapper in use: nice-plug's
  `GuiContext::plugin_api()` is `Vst3` or `Clap` (`Embed::format`, `EditorEmbed::format`); the
  stand-in host of `editor_preview` takes `--format vst3|clap`.
- The Sync LED is lit while the snapshot's `host_sync` is; the MIDI LED for 0.1 s after the
  snapshot's `midi_in` count changes (art part `led_transport_on`), in the preview too. The editor redraws at least
  every 50 ms while open so that the LEDs follow.
- File has New, Open and Info (no Save, Save As, Export); nothing asks to save (the song lives
  in the host's project). File > Open loads the file into the plugin (`OrbParams::load_song`),
  sets the song's control values as host parameters; File > Open and File > New keep the
  mode (task-228). The file browser is orb-ui's own, egui-drawn (task-046),
  so no native dialog is opened from the plugin window; its folders and the Mods folder are the
  standalone's settings file (`OpenRB-338/ui.toml`), shared; a save merges only its own
  change into the file as it is then, so open instances do not overwrite each other
  (task-229).
- Mods: a mod chosen (the Mods menu, a song's mod, Choose file) is read and given to the plugin
  (`OrbParams::set_mod`, embedded in the state). An editor that opens on a state's mod names a
  copy of its bytes in the temporary folder (`OpenRB-338/mods/<sha256>.rbm`), or the file an
  editor of the process chose, for the 808's mod window and Mods > Info.
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
image of the plugin in one process (Reaper loads the `.vst3` and the `.clap`, two copies of the
same binary, when both formats are used; any other plugin linking raw-window-metal does the
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
name, so two dynamic libraries that each link raw-window-metal (audio plugins, e.g. a VST3 and
a CLAP build of one plugin, or two wgpu plugins) cannot both create a layer in one process: the
second `define_class!` registration panics. Please register the class under a name unique to
the image (e.g. with a random or address suffix through `ClassBuilder`), as baseview does for
its NSView subclass."

**No panic reaches the host.** Every method of `OrbEditor` (`spawn`, `size`, `resize_hint`)
and of `OrbHandle` (`run_until_closed`, `set_parent`, `show`, `hide`, `set_size`,
`adjust_size`, `set_fallback_scale_factor`, the main-thread, key, state and parameter
callbacks), the egui app's `build`, frame, `resized` and `editor_closed`, and the posted
resize request run under `editor::guard` (`catch_unwind`): a panic is logged (`nice_error!`)
and becomes an error or a safe fallback (no adjustment, the default size, the default resize
hint, the state's size unchanged), so the host gets no editor (`gui.create` false) or an
empty window instead of an abort. nice-plug has no hook of its own for this. `tests/clap_gui.rs`
(macOS, no test harness, on the main thread) loads the library twice as two files, as Reaper
does, and in each makes instances that open their editor through CLAP `gui.create`, process
audio while it is open and close it; with the old bundle's binary (`ORB_TEST_CLAP_LIBRARY`) it
aborts on the class name as Reaper did. The VST3 path opens the same `OrbEditor`; a VST3 host
with an `IPlugFrame` and a parent `NSView` is left to the Reaper check.

**The audio thread's SIGSEGV in the first crash report.** The first report
(`REAPER-2026-11-16-140356.ips`) also had Reaper's audio worker crashed at a byte write to
`0x50` in `orb_engine::delay::Delay as SendEffect::process` (from the CLAP instance's
`process`), while the main thread was still in the panic hook printing the backtrace before
the abort. Delay's code is safe and the second report (`REAPER-2026-11-16-140655.ips`, same
trigger) has no audio-thread fault, only the abort. Taken as a consequence of the process
being torn down by the non-unwinding panic, not a defect of its own; not reproduced
(`tests/clap_gui.rs` processes 200 blocks per instance with the editor open in both copies).

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
[--format vst3|clap] [--click X,Y]... [--scroll Y] [--screenshot OUT.png]` shows the editor in a window of its own with a
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
size. `tests/fonts.rs`: the plugin's normal build has egui's fonts. `tests/clap_gui.rs`: two
copies of the library each open CLAP editors (above), and the corner's request posted to the
main queue reaches the host once, after the event. The real-host checks (Reaper, VST3 and
CLAP) are the user's (task-201's notes).

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
    and the `.clap` may be loaded in one process.
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
