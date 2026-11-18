//! The plugin's process side (task-197, task-198): the engine in 64-frame blocks for any host
//! buffer size, with the host's parameter changes, MIDI events and transport, and the
//! resampler when the host is not at 44.1 kHz (`docs/plugin.md` sections 4 and 8).
//!
//! # Timing
//!
//! The engine runs in calls of one block ([`BLOCK`] frames, `orb_player::Audio` made with
//! `Player::with_call_frames`). Host frame `j` (counted from the activation) plays engine frame
//! `j - 64`: one block is rendered ahead, and the plugin reports 64 frames of latency. After the
//! host's latency compensation block `m` plays at host frames `[64m, 64m + 64)`. A host event at
//! frame `t` (a parameter change, a MIDI message) reaches the first block with `64m >= t`, never
//! an earlier one (task-227, the user's decision of 2026-11-16): before applying it the blocks
//! that start before `t` are rendered, ahead of their time if need be (at most one beyond the
//! one being handed out); a block is otherwise rendered when its first frame is due. An event
//! sounds 0 to 63 frames late, never early (an early one lost its start at a realtime start off
//! the block grid, where the host drops what comes before the play position), and the output is
//! a function of the events' times alone, not of the host's buffer sizes.
//!
//! With a resampler ([`Resampler`]) host frame `j` plays the engine at
//! `j * 44100 / rate - (64 + W)`; the blocks whose first frame plays at or before host frame
//! `t - 1` after the compensation are rendered before an event at `t`
//! ([`Resampler::render_before`]), the others when a host frame needs them.
//!
//! # Parameters
//!
//! Each call gets every parameter's value. One that differs from the value seen in the call
//! before is a change at the call's first frame: it is sent as the panel's message
//! ([`params::command`]) unless the engine already holds that value (a song's out-of-range
//! byte that the value stands for is kept).
//!
//! # The mode and the preview (task-217)
//!
//! The mode ([`crate::instance::Mode`]) and the preview are the main thread's settings
//! ([`Shared`]), read before each block. Outside the preview, the pattern modes (Latch and
//! Gate) play the song only while the host plays and stop it when the host stops (Gate also
//! while a gate note is held); note mode keeps it stopped. The
//! preview (the panel's Play) plays the song free-running (its step does not follow the host's
//! position) at the host's tempo (the song's when the host reports none), with the preview
//! set's values: entering it applies them, a change of the set is applied at the next block,
//! and leaving it applies the parameters' latest values. While it runs, host notes, the
//! Standard Mapping and parameter changes are ignored (the changes are kept for its end; the
//! MIDI LED still counts the messages).
//!
//! # Host sync (task-198)
//!
//! Each call records the host's transport at its first frame ([`HostTransport`]). Engine block
//! `m` belongs to host timeline frame `h = 64m * rate / 44100`; before it renders, the host's
//! tempo and position at `h` are taken from the last record that starts at or before `h`
//! (extrapolated at its tempo while the host plays):
//!
//! - the tempo (song msg 35 `{4, v}`) is the host's, rounded to BPM*1000, or the song's when the
//!   host reports none;
//! - in pattern mode while the host plays, the position follows the host's, 192 ticks per quarter
//!   note: when the host starts (or pattern mode or the preview's end comes while it plays) the
//!   song is stopped, located to the host's tick and played again; when the position jumps (a seek,
//!   a loop) by a tick or more from where the last block's position and the tempo put it, the song
//!   is located there; otherwise only the position is set (`Command::Position`), as the original's
//!   TopHat sync does in every audio call (`sub_1001C2F4`, `docs/plugin.md` section 8);
//! - while the host is stopped the song is stopped (no sound in pattern mode);
//! - in note mode the song is stopped, and the note-mode clock (`Command::NoteClock`, task-242)
//!   gives the PCF its tick: the host's, set before each block, while the host plays with a
//!   position, and free-running at the tempo from where it was otherwise.
//!
//! # MIDI
//!
//! A note-on 0..31 on channel 1..4 (303 #1, 303 #2, 808, 909) in Latch Pattern Mode selects
//! that section's pattern: the pattern controller (as the panel's message), then, while the song
//! plays, slot 2 on that device (`Command::Relocate`), which takes a pending pattern at once
//! where the panel's waits for the bar line; with the host stopped only the selection changes.
//! A note-off does nothing. In Gate Pattern Mode (task-218, `docs/plugin.md` section 11) the
//! notes 0..31 held on a channel open that section's pattern gate (`Command::Gate`) on the last
//! one's pattern, and the section is silent while none is held; with the host stopped the first
//! open gate plays the song from step 1, free-running. In note mode (task-200) a note-on or
//! note-off on those channels
//! plays the device (`Command::Note`, `docs/plugin.md` section 9), whatever the host's
//! transport does. Every other message goes, when the Standard Mapping is on (off by default),
//! to it on the remote channel (default 16, [`crate::instance::Shared`]); of its actions the
//! controls are applied, the transport ones (Play, Stop, Record, Locate) are not: the
//! transport is the host's. The preview ignores every message.
//!
//! # A full garbage queue (task-225)
//!
//! Every host event is applied at once (`Audio::apply_now`), even while the garbage queue is
//! full of what the main thread's edits, loads and mods handed back and has not collected yet:
//! none of the process side's commands carries a box, and orb-player refuses only those that
//! do (`Audio::room_for`; a song's Play while it is pending finds the slot kept for it). The
//! main thread's commands wait in their queue, in order, from the first one with a box that
//! does not fit.
//!
//! Nothing here allocates, frees or locks: the queues and the snapshot are orb-player's
//! lock-free ones, the parameter table is a constant, and the buffers are made at activation.

use std::sync::Arc;

use orb_engine::frame::{Gate, NoteClock};
use orb_engine::seq::TEMPO_STEP;
use orb_engine::song::Device;
use orb_player::midi::{Action, Message, StandardMapping, Timed};
use orb_player::{Audio, Command};

use crate::instance::{Mode, ROWS, Shared};
use crate::params::{self, COUNT, SPECS};
use crate::resample::{Blocks, ENGINE_RATE, Resampler};

/// The frames of one engine block, and of the reported latency at 44.1 kHz.
pub const BLOCK: usize = 64;
/// The engine's ticks per quarter note (`docs/initial_investigation.md` section 5).
pub const TICKS_PER_BEAT: f64 = 192.0;
/// The transport records kept: more than the calls one block can lie back (a call has at least
/// one frame, and a block is at most the latency behind the call that renders it).
const RECORDS: usize = 1024;
/// The largest tick the plugin locates to.
const MAX_TICK: f64 = 1.0e9;
/// The MIDI actions one message can give (the Standard Mapping gives at most two).
const ACTIONS: usize = 8;

/// What the host says about its transport at a call's first frame.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct HostTransport {
    pub playing: bool,
    /// BPM; `None` when the host reports none.
    pub tempo: Option<f64>,
    /// The position in quarter notes; `None` when the host reports none.
    pub pos_beats: Option<f64>,
}

/// A host MIDI message at a frame of the call.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HostEvent {
    pub frame: u32,
    pub message: Message,
}

/// The transport of the call that starts at host frame `start`.
#[derive(Clone, Copy, Debug, Default)]
struct Record {
    start: u64,
    t: HostTransport,
}

/// The notes 0..31 held on one section's channel in Gate Pattern Mode, oldest first.
#[derive(Clone, Copy, Debug, Default)]
struct Held {
    notes: [u8; 32],
    len: usize,
}

impl Held {
    /// The note that selects the pattern: the last one pressed of those still held.
    fn top(&self) -> Option<u8> {
        self.len.checked_sub(1).map(|i| self.notes[i])
    }

    /// Removes `note`; whether it was held.
    fn remove(&mut self, note: u8) -> bool {
        let Some(i) = self.notes[..self.len].iter().position(|&n| n == note) else {
            return false;
        };
        self.notes.copy_within(i + 1..self.len, i);
        self.len -= 1;
        true
    }

    /// `note` (0..31) is pressed: it becomes the last one.
    fn press(&mut self, note: u8) {
        self.remove(note);
        self.notes[self.len] = note;
        self.len += 1;
    }
}

/// The engine and its block source.
struct Engine {
    audio: Audio,
    /// The mode, as of the last block or message.
    mode: Mode,
    /// The pattern gates are in use (Gate Pattern Mode outside the preview).
    gating: bool,
    /// The notes held on each section's channel while gating.
    held: [Held; 4],
    /// The preview ran in the last block.
    preview: bool,
    /// The parameters' values as last seen (the host's).
    host: [i32; COUNT],
    /// The preview set's values as last applied.
    shown: [i32; COUNT],
    /// The commands `Audio::apply_now` refused. It refuses only a command that carries a box
    /// and finds the garbage queue full (`Audio::room_for`), and the process side sends none,
    /// so this stays 0 (task-225); a diagnostic.
    dropped: u64,
    /// The host rate.
    rate: u32,
    /// The blocks rendered.
    blocks: u64,
    /// The transport records, a ring; `next` the next slot, `len` the records held.
    records: Box<[Record]>,
    next: usize,
    len: usize,
    /// The block before followed the host's position, which was `last_tick`.
    following: bool,
    last_tick: f64,
    /// The newest song seen (the player's load count when it came) and its own tempo,
    /// BPM*1000.
    song: Option<u64>,
    song_tempo: u32,
    mapping: StandardMapping,
    actions: Vec<Timed<Action>>,
    shared: Arc<Shared>,
}

impl Engine {
    fn send(&mut self, c: Command) {
        if self.audio.apply_now(c).is_err() {
            self.dropped += 1;
        }
    }

    /// The song plays: Play when it is stopped, or when a song loaded over the playing one
    /// waits (pending) for Play, which switches to it.
    fn ensure_playing(&mut self) {
        let a = &self.audio;
        if !a.current().playing() || a.has_pending() {
            self.send(Command::Play);
        }
    }

    /// The song is stopped (song msg 35 `{1,0}`, which locates to the bar start). A song
    /// loaded over the playing one is switched to first, so that the newest song is the one
    /// stopped.
    fn ensure_stopped(&mut self) {
        if self.audio.has_pending() {
            self.send(Command::Play);
        }
        if self.audio.current().playing() {
            self.send(Command::Stop);
        }
    }

    /// Whether the preview runs or is about to (the main thread has turned it on).
    fn previewing(&self) -> bool {
        self.preview || self.shared.previewing()
    }

    /// Parameter `i` at `v`: its message unless the engine already plays it.
    fn apply_one(&mut self, i: usize, v: i32) {
        let s = &SPECS[i];
        let holds = params::engine_byte(self.audio.newest(), s.target)
            .is_some_and(|b| params::value_of(s, b) == v);
        if holds {
            return;
        }
        if let Some(c) = params::command(s, v) {
            self.send(c);
        }
    }

    /// Every parameter at `values` (entering or leaving the preview).
    fn apply_all(&mut self, values: &[i32; COUNT]) {
        for (i, &v) in values.iter().enumerate() {
            self.apply_one(i, v);
        }
    }

    /// The gate of section `c` as its held notes say (gating), else off.
    fn gate_of(&self, c: usize) -> Gate {
        if !self.gating {
            return Gate::default();
        }
        let top = self.held[c].top();
        Gate {
            on: true,
            open: top.is_some(),
            pattern: top.unwrap_or(0),
        }
    }

    /// Sends section `c`'s gate; an open one first shows its pattern on the panel (the pattern
    /// controller's message, as a Latch Pattern Mode note; the parameter does not change).
    fn send_gate(&mut self, c: usize) {
        let device = ROWS[c];
        let gate = self.gate_of(c);
        if gate.open {
            self.send(Command::DeviceParam {
                device,
                ctrl: 1,
                value: gate.pattern,
            });
        }
        self.send(Command::Gate { device, gate });
    }

    /// The pattern gates follow the mode and the preview: Gate Pattern Mode outside the preview
    /// closes the four gates (no note is held yet), anything else turns them off (the original's
    /// path); either change forgets the held notes. Any engine gate that is not as it should
    /// be is sent again: a newly loaded song's (its gates are off).
    fn gates(&mut self) {
        let want = self.mode == Mode::Gate && !self.previewing();
        if want != self.gating {
            self.held = Default::default();
            self.gating = want;
        }
        for c in 0..ROWS.len() {
            let g = if want {
                self.gate_of(c)
            } else {
                Gate::default()
            };
            if self.audio.newest().frame.devices.channels[c].gate() != g {
                self.send_gate(c);
            }
        }
    }

    /// Whether a section's gate is open.
    fn any_open(&self) -> bool {
        self.gating && self.held.iter().any(|h| h.len > 0)
    }

    /// A gate note on section `c`'s channel: a note-on (`on`) opens the gate on its pattern or
    /// switches it there; a note-off of the last note pressed falls back to the newest note
    /// still held, or closes the gate. With the song stopped (the host stopped), the first open
    /// gate starts it at step 1.
    fn gate_note(&mut self, c: usize, note: u8, on: bool) {
        let top = self.held[c].top();
        if on {
            self.held[c].press(note);
        } else if !self.held[c].remove(note) {
            return;
        }
        if self.held[c].top() == top {
            return;
        }
        self.send_gate(c);
        if self.held[c].len > 0
            && !self.host_playing()
            && !self.audio.has_pending()
            && !self.audio.current().playing()
        {
            // the host is stopped and nothing plays: a held note plays its pattern from step 1,
            // free-running at the host's tempo (`sync`); a gate that opens while another
            // section plays joins it in step (the user, 2026-11-16)
            self.send(Command::Locate(0));
            self.send(Command::Play);
        }
    }

    /// Before a block: the mode, and the preview's entry, exit and changes (task-217).
    fn modes(&mut self) {
        self.mode = self.shared.mode();
        let preview = self.shared.previewing();
        if preview != self.preview {
            self.preview = preview;
            self.following = false;
            if preview {
                for (i, v) in self.shown.iter_mut().enumerate() {
                    *v = self.shared.preview_value(i);
                }
                let v = self.shown;
                self.apply_all(&v);
            } else {
                // back to the host's values, the latest ones
                let v = self.host;
                self.apply_all(&v);
            }
        } else if preview {
            for i in 0..COUNT {
                let v = self.shared.preview_value(i);
                if v != self.shown[i] {
                    self.shown[i] = v;
                    self.apply_one(i, v);
                }
            }
        }
        self.gates();
    }

    fn record(&mut self, start: u64, t: HostTransport) {
        self.records[self.next] = Record { start, t };
        self.next = (self.next + 1) % RECORDS;
        self.len = (self.len + 1).min(RECORDS);
    }

    /// The record in force at host frame `h`: the newest that starts at or before it, else the
    /// oldest held.
    fn record_at(&self, h: f64) -> Record {
        let mut oldest = Record::default();
        for k in 1..=self.len {
            let r = self.records[(self.next + RECORDS - k) % RECORDS];
            if r.start as f64 <= h {
                return r;
            }
            oldest = r;
        }
        oldest
    }

    /// Whether the host plays in the current call (its newest record): the next block's
    /// [`Engine::sync`] then starts the song itself, at the host's position.
    fn host_playing(&self) -> bool {
        self.len > 0 && self.records[(self.next + RECORDS - 1) % RECORDS].t.playing
    }

    /// Before block `blocks` renders: the tempo, whether the song plays, and the position while
    /// it follows the host.
    fn sync(&mut self) {
        let id = Some(self.audio.loads());
        if id != self.song {
            self.song = id;
            self.song_tempo = self.audio.newest().frame.seq.globals.tempo;
        }
        // the block's host timeline frame
        let h = (self.blocks * BLOCK as u64) as f64 * self.rate as f64 / ENGINE_RATE as f64;
        let rec = self.record_at(h);
        // the host's tempo also in the preview and while the host is stopped (hosts report it
        // then; the user, 2026-11-16); the song's only when the host reports none
        let host_tempo = rec.t.tempo.filter(|t| t.is_finite() && *t > 0.0);
        let tempo = match host_tempo {
            Some(t) => (t * 1000.0).round().clamp(1.0, u32::MAX as f64) as u32,
            None => self.song_tempo,
        };
        if self.audio.newest().frame.seq.globals.tempo != tempo {
            self.send(Command::Tempo(tempo));
        }
        let host_plays = self.mode.patterns() && rec.t.playing;
        // the host's tick at `h` while it plays with a position, 192 a quarter note
        let host_tick = match rec.t.pos_beats {
            Some(p) if rec.t.playing && p.is_finite() => {
                let bpm = host_tempo.unwrap_or(tempo as f64 / 1000.0);
                let beats = p + (h - rec.start as f64) * bpm / 60.0 / self.rate as f64;
                Some((beats * TICKS_PER_BEAT).clamp(0.0, MAX_TICK))
            }
            _ => None,
        };
        let tick = match host_tick {
            Some(t) if !self.preview && host_plays => t,
            _ => {
                self.following = false;
                self.audio.set_host_sync(false);
                // before the Stop note mode owes: a clock that starts takes the position the
                // playing song (the preview's) has reached, not the bar start the Stop locates to
                self.note_clock(host_tick);
                let fading = self.audio.newest().frame.gate_fade.contains(&true);
                if self.preview || host_plays || self.any_open() || fading {
                    // the preview, a host that plays without a position, or a held gate note
                    // with the host stopped (or the fade-out of the last one's close, which a
                    // Stop's slot 6 would cut): free-running
                    self.ensure_playing();
                } else {
                    // note mode, or a pattern mode with the host stopped: no pattern sounds
                    self.ensure_stopped();
                }
                return;
            }
        };
        self.note_clock(None);
        let at = tick as u32;
        if self.audio.has_pending() || !self.audio.current().playing() {
            // a song loaded over the playing one: switch to it, then start it cleanly
            if self.audio.has_pending() {
                self.send(Command::Play);
            }
            self.following = false;
        }
        if !self.following {
            // the host starts: a clean start at its position
            self.send(Command::Stop);
            self.send(Command::Locate(at));
            self.send(Command::Play);
        } else {
            let expected = self.last_tick + tempo as f64 * TEMPO_STEP;
            if (tick - expected).abs() >= 1.0 {
                self.send(Command::Locate(at));
            }
        }
        self.send(Command::Position(tick));
        self.following = true;
        self.last_tick = tick;
        self.audio.set_host_sync(true);
    }

    /// The note-mode clock (task-242, `docs/plugin.md` 9.5), before the song's Stop the mode
    /// owes: in note mode outside the preview the PCF's tick is the host's (`host_tick`) while
    /// the host plays with a position, set before the block as pattern mode's
    /// `Command::Position`, and free-running at the tempo from where it was otherwise (a clock
    /// that starts takes the song's position, which a song still playing, the preview's, has
    /// not yet lost to the Stop's locate); in the pattern modes and the preview the clock is
    /// off.
    fn note_clock(&mut self, host_tick: Option<f64>) {
        let on = self.mode == Mode::Note && !self.preview;
        let running = self.audio.newest().frame.note_clock.is_some();
        let c = match (on, host_tick) {
            (true, Some(t)) => NoteClock::At(t),
            (true, None) if !running => NoteClock::Run,
            (false, _) if running => NoteClock::Off,
            _ => return,
        };
        self.send(Command::NoteClock(c));
    }

    /// One host MIDI message.
    fn message(&mut self, m: &Message) {
        // the MIDI LED counts it even in the preview, which ignores it
        self.audio.midi_received(1);
        if self.previewing() {
            return;
        }
        let (c, note, velocity) = match *m {
            Message::NoteOn {
                channel: c @ 0..=3,
                note,
                velocity,
            } => (c, note, velocity),
            Message::NoteOff {
                channel: c @ 0..=3,
                note,
                ..
            } => (c, note, 0),
            _ => (u8::MAX, 0, 0),
        };
        if c != u8::MAX {
            // a section's channel
            let device: Device = ROWS[c as usize];
            // the mode as the main thread has it now, not as of the last block
            self.mode = self.shared.mode();
            self.gates();
            match self.mode {
                Mode::Note => {
                    // note mode (task-200), whatever the host's transport does: a Stop the mode
                    // still owes the engine comes first, so that it does not silence the note
                    self.ensure_stopped();
                    self.send(Command::Note {
                        device,
                        note,
                        velocity,
                    });
                }
                Mode::Latch if velocity > 0 && note < 32 => {
                    // the note selects a pattern, at once while the song plays; with the song
                    // stopped (the host stopped) only the selection changes, and the next
                    // start takes it (its locate relocates every device)
                    self.send(Command::DeviceParam {
                        device,
                        ctrl: 1,
                        value: note,
                    });
                    if self.audio.current().playing() {
                        self.send(Command::Relocate(device));
                    }
                }
                Mode::Gate if note < 32 => self.gate_note(c as usize, note, velocity > 0),
                _ => {}
            }
            return;
        }
        if !self.shared.remote_enabled() {
            return;
        }
        self.mapping.channel = self.shared.remote_channel();
        self.actions.clear();
        self.mapping.message(0, m, &mut self.actions);
        for k in 0..self.actions.len() {
            let a = self.actions[k].event;
            if matches!(
                a,
                Action::Play | Action::Stop | Action::Record(_) | Action::Locate(_)
            ) {
                continue;
            }
            if let Some(c) = a.command() {
                self.send(c);
            }
        }
    }
}

impl Blocks for Engine {
    /// One engine call: the UI side's queued commands, the mode and the preview, the host
    /// sync, then the block.
    fn next_block(&mut self) -> &[i16] {
        self.audio.drain();
        self.modes();
        self.sync();
        self.audio.render_call();
        self.blocks += 1;
        self.mapping.follow(self.audio.last_snapshot());
        self.audio.output()
    }
}

/// The silent blocks [`settle`] renders: enough for the mixer's per-block gain smoothing
/// (`g = f32(g*0.93 + t*0.07)`, 0x100136B7) to reach its f32 fixed point from the new song's
/// 1e-6 (`0.93^k` below `2^-24` takes 230 blocks, after the first mixer update at block 6),
/// and a multiple of the mixer update's 16-block period.
pub const SETTLE_BLOCKS: usize = 256;

/// Readies a new player's callback side at the activation, on the main thread: applies the
/// queued commands (the activation's `Command::Load`), then renders [`SETTLE_BLOCKS`] silent
/// blocks (the song is stopped, nothing sounds) and drops them, so that the mixer's gains have
/// risen from the new song's 1e-6 to their targets.
///
/// Without it a host event of the first host frames was applied to `Player`'s placeholder
/// song and lost when the first block's `Audio::drain` loaded the activation's song over it
/// (a note at the start of an offline render), and a note in the first blocks after the load
/// sounded softer, under the fade-in of a song load (task-226).
pub fn settle(audio: &mut Audio) {
    audio.drain();
    for _ in 0..SETTLE_BLOCKS {
        audio.render_call();
    }
}

/// The blocks [`Out::Direct`] holds: the one being handed out and the ones rendered ahead.
const AHEAD: usize = 3;

/// How engine frames become host frames.
enum Out {
    /// At 44.1 kHz: the blocks rendered and not yet handed out, `ring[(front + k) % AHEAD]` for
    /// `k` in `0..=queued` (the front one is being handed out, its next frame `pos`).
    Direct {
        ring: Box<[[i16; 2 * BLOCK]; AHEAD]>,
        front: usize,
        queued: usize,
        pos: usize,
    },
    Resampled(Resampler),
}

/// The process side of one plugin instance.
pub struct Core {
    engine: Engine,
    out: Out,
    rate: u32,
    /// The host frames produced.
    frames: u64,
}

impl Core {
    /// The process side for host rate `rate`, with the parameter values `values` as the ones
    /// already seen (the engine was built with them, `crate::instance`) and the main thread's
    /// settings `shared`. `audio` must run [`BLOCK`]-frame calls.
    ///
    /// The activation's song is put in place here, on the main thread, before the first call
    /// ([`settle`]): a host event of the first call reaches it, and its first block plays at
    /// the mixer's full gain (task-226).
    pub fn new(mut audio: Audio, rate: u32, values: &[i32; COUNT], shared: Arc<Shared>) -> Core {
        assert_eq!(audio.call_frames(), BLOCK);
        settle(&mut audio);
        let out = if rate == ENGINE_RATE {
            // the block ahead: the first 64 host frames are silence
            Out::Direct {
                // made here, on the main thread
                ring: Box::new([[0; 2 * BLOCK]; AHEAD]),
                front: 0,
                queued: 0,
                pos: 0,
            }
        } else {
            Out::Resampled(Resampler::new(rate))
        };
        let mut mapping = StandardMapping::new();
        mapping.channel = shared.remote_channel();
        Core {
            engine: Engine {
                audio,
                mode: shared.mode(),
                // the first block turns the gates on in Gate Pattern Mode
                gating: false,
                held: Default::default(),
                // a preview that runs is entered at the first block
                preview: false,
                host: *values,
                shown: *values,
                dropped: 0,
                rate,
                blocks: 0,
                records: vec![Record::default(); RECORDS].into_boxed_slice(),
                next: 0,
                len: 0,
                following: false,
                last_tick: 0.0,
                song: None,
                song_tempo: 0,
                mapping,
                actions: Vec::with_capacity(ACTIONS),
                shared,
            },
            out,
            rate,
            frames: 0,
        }
    }

    /// The host rate.
    pub fn rate(&self) -> u32 {
        self.rate
    }

    /// The latency the plugin reports, in host frames.
    pub fn latency(&self) -> u32 {
        match &self.out {
            Out::Direct { .. } => BLOCK as u32,
            Out::Resampled(r) => r.latency(),
        }
    }

    /// The callback side of the player (for tests and diagnostics).
    pub fn audio(&self) -> &Audio {
        &self.engine.audio
    }

    /// The process side's commands `Audio::apply_now` refused: 0, as none of them carries a box,
    /// and only such a command is refused when the garbage queue is full (task-225).
    pub fn dropped_events(&self) -> u64 {
        self.engine.dropped
    }

    /// Whether the last block followed the host's position (the Sync LED).
    pub fn following(&self) -> bool {
        self.engine.following
    }

    /// The engine blocks rendered since the activation; block `m` belongs to host timeline
    /// frame `64m * rate / 44100`.
    pub fn blocks(&self) -> u64 {
        self.engine.blocks
    }

    /// [`Core::process_with`] for a host that reports no transport and sends no MIDI.
    pub fn process(&mut self, l: &mut [f32], r: &mut [f32], values: &[i32; COUNT]) {
        self.process_with(l, r, values, &HostTransport::default(), &[]);
    }

    /// Fills `l` and `r` (the same length) with the next host frames; `values` are the
    /// parameters' values for this call, `transport` the host's transport at its first frame
    /// and `events` its MIDI messages, by frame (a frame past the call is its last). A call of
    /// no frames takes nothing: its changes are seen at the next call's first frame, where they
    /// belong.
    pub fn process_with(
        &mut self,
        l: &mut [f32],
        r: &mut [f32],
        values: &[i32; COUNT],
        transport: &HostTransport,
        events: &[HostEvent],
    ) {
        let n = l.len().min(r.len());
        if n == 0 {
            return;
        }
        self.engine.record(self.frames, *transport);
        // the changes reach the blocks at or after their frame, never one before it (see the
        // module doc)
        self.render_before(self.frames);
        self.apply(values);
        let mut done = 0;
        for e in events {
            let t = (e.frame as usize).min(n - 1);
            if t > done {
                self.produce(&mut l[done..t], &mut r[done..t]);
                done = t;
            }
            self.render_before(self.frames + t as u64);
            self.engine.message(&e.message);
        }
        self.produce(&mut l[done..n], &mut r[done..n]);
        self.frames += n as u64;
    }

    /// Applies the parameter values that changed since the last call; during the preview they
    /// are only kept, for its end.
    fn apply(&mut self, values: &[i32; COUNT]) {
        let preview = self.engine.previewing();
        for (i, &v) in values.iter().enumerate() {
            if v == self.engine.host[i] {
                continue;
            }
            self.engine.host[i] = v;
            if !preview {
                self.engine.apply_one(i, v);
            }
        }
    }

    /// Whether the last block played the preview.
    pub fn previewing(&self) -> bool {
        self.engine.preview
    }

    /// Renders, ahead of their time, the engine blocks whose first frame plays before host frame
    /// `host` after the host's latency compensation, so that an event at `host` reaches only the
    /// blocks at or after it (task-226). At 44.1 kHz block `m` plays from host frame
    /// `64m + 64`, `64m` compensated: the blocks with `64m < host`.
    fn render_before(&mut self, host: u64) {
        match &mut self.out {
            Out::Direct {
                ring,
                front,
                queued,
                ..
            } => {
                while self.engine.blocks * (BLOCK as u64) < host {
                    // at most the playing block and one more (`host` is the next frame to play)
                    assert!(*queued + 1 < AHEAD, "the blocks ahead fit");
                    let slot = (*front + 1 + *queued) % AHEAD;
                    ring[slot].copy_from_slice(self.engine.next_block());
                    *queued += 1;
                }
            }
            Out::Resampled(rs) => rs.render_before(host, &mut self.engine),
        }
    }

    fn produce(&mut self, l: &mut [f32], r: &mut [f32]) {
        match &mut self.out {
            Out::Direct {
                ring,
                front,
                queued,
                pos,
            } => {
                for (ol, or) in l.iter_mut().zip(r.iter_mut()) {
                    if *pos == BLOCK {
                        if *queued == 0 {
                            let slot = (*front + 1) % AHEAD;
                            ring[slot].copy_from_slice(self.engine.next_block());
                            *queued = 1;
                        }
                        *front = (*front + 1) % AHEAD;
                        *queued -= 1;
                        *pos = 0;
                    }
                    let block = &ring[*front];
                    // exact: an int16 over 32768
                    *ol = block[2 * *pos] as f32 / 32768.0;
                    *or = block[2 * *pos + 1] as f32 / 32768.0;
                    *pos += 1;
                }
            }
            Out::Resampled(rs) => rs.fill(l, r, &mut self.engine),
        }
    }
}
