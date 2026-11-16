//! The plugin's process side (task-197, task-198): the engine in 64-frame blocks for any host
//! buffer size, with the host's parameter changes, MIDI events and transport, and the
//! resampler when the host is not at 44.1 kHz (`docs/plugin.md` sections 4 and 8).
//!
//! # Timing
//!
//! The engine runs in calls of one block ([`BLOCK`] frames, `orb_player::Audio` made with
//! `Player::with_call_frames`). Host frame `j` (counted from the activation) plays engine frame
//! `j - 64`: one block is rendered ahead, and the plugin reports 64 frames of latency. The block
//! is rendered when its first frame is due, and a host event at frame `t` (a parameter change,
//! a MIDI message) is applied right after frame `t` is produced. So the block that plays at
//! host frames `[64m, 64m + 64)` (after the host's latency compensation, the block of those
//! very frames) holds exactly the events of those frames, and the output is a function of the
//! events' times alone, not of the host's buffer sizes.
//!
//! With a resampler ([`Resampler`]) host frame `j` plays the engine at
//! `j * 44100 / rate - (64 + W)`, and the block a host frame needs is again rendered when that
//! frame is due, after the events before it.
//!
//! # Parameters
//!
//! Each call gets every parameter's value. One that differs from the value seen in the call
//! before is a change at the call's first frame: it is sent as the panel's message
//! ([`params::command`]) unless the engine already holds that value (a song's out-of-range
//! byte that the value stands for is kept). The mode ([`params::MODE`]) is a level: before each
//! block the engine is played or stopped to match it, which also starts a loaded song that
//! waits for Play.
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
//! - in pattern mode (the mode at Play) while the host plays, the position follows the host's, 192
//!   ticks per quarter note: when the host starts (or the mode turns to Play while it plays) the
//!   song is stopped, located to the host's tick and played again; when the position jumps (a seek,
//!   a loop) by a tick or more from where the last block's position and the tempo put it, the song
//!   is located there; otherwise only the position is set (`Command::Position`), as the original's
//!   TopHat sync does in every audio call (`sub_1001C2F4`, `docs/plugin.md` section 8);
//! - while the host is stopped the song runs on from where it is.
//!
//! # MIDI
//!
//! A note-on 0..31 on channel 1..4 (303 #1, 303 #2, 808, 909) in pattern mode selects that
//! section's pattern at once: the pattern controller (as the panel's message), then slot 2 on
//! that device (`Command::Relocate`), which takes a pending pattern at once where the panel's
//! waits for the bar line. A note-off does nothing. In note mode (the mode at Stop, task-200)
//! a note-on or note-off on those channels plays the device (`Command::Note`, `docs/plugin.md`
//! section 9). Every other message goes, when the Standard
//! Mapping is on (off by default), to it on the remote channel (default 16,
//! [`crate::instance::Shared`]); of its actions the controls are
//! applied, the transport ones (Play, Stop, Record, Locate) are not: the mode is the host's
//! parameter and the transport the host's.
//!
//! Nothing here allocates, frees or locks: the queues and the snapshot are orb-player's
//! lock-free ones, the parameter table is a constant, and the buffers are made at activation.

use std::sync::Arc;

use orb_engine::seq::TEMPO_STEP;
use orb_engine::song::Device;
use orb_player::midi::{Action, Message, StandardMapping, Timed};
use orb_player::{Audio, Command};

use crate::instance::{ROWS, Shared};
use crate::params::{self, COUNT, MODE, SPECS};
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

/// The engine and its block source.
struct Engine {
    audio: Audio,
    /// The mode parameter as last seen: the engine plays while it is set.
    playing: bool,
    /// The host events that could not be applied (the garbage queue was full); counted, not
    /// retried.
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

    /// The engine's song plays or stops as the mode says. A song loaded over a playing one
    /// waits (pending) until Play, which switches to it.
    fn hold_mode(&mut self) {
        let a = &self.audio;
        let c = if self.playing {
            (!a.current().playing() || a.has_pending()).then_some(Command::Play)
        } else {
            a.current().playing().then_some(Command::Stop)
        };
        if let Some(c) = c {
            self.send(c);
        }
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

    /// Before block `blocks` renders: the tempo, and the position while the host plays.
    fn sync(&mut self) {
        let id = Some(self.audio.loads());
        if id != self.song {
            self.song = id;
            self.song_tempo = self.audio.newest().frame.seq.globals.tempo;
        }
        // the block's host timeline frame
        let h = (self.blocks * BLOCK as u64) as f64 * self.rate as f64 / ENGINE_RATE as f64;
        let rec = self.record_at(h);
        let host_tempo = rec.t.tempo.filter(|t| t.is_finite() && *t > 0.0);
        let tempo = match host_tempo {
            Some(t) => (t * 1000.0).round().clamp(1.0, u32::MAX as f64) as u32,
            None => self.song_tempo,
        };
        if self.audio.newest().frame.seq.globals.tempo != tempo {
            self.send(Command::Tempo(tempo));
        }
        let pos = match rec.t.pos_beats {
            Some(p) if self.playing && rec.t.playing && p.is_finite() => p,
            _ => {
                self.following = false;
                self.audio.set_host_sync(false);
                return;
            }
        };
        let bpm = host_tempo.unwrap_or(tempo as f64 / 1000.0);
        let beats = pos + (h - rec.start as f64) * bpm / 60.0 / self.rate as f64;
        let tick = (beats * TICKS_PER_BEAT).clamp(0.0, MAX_TICK);
        let at = tick as u32;
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

    /// One host MIDI message.
    fn message(&mut self, m: &Message) {
        self.audio.midi_received(1);
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
            if !self.playing {
                // note mode (task-200): a Stop the mode still owes the engine comes first, so
                // that it does not silence the note
                self.hold_mode();
                self.send(Command::Note {
                    device,
                    note,
                    velocity,
                });
            } else if velocity > 0 && note < 32 {
                // pattern mode: the note selects a pattern
                self.send(Command::DeviceParam {
                    device,
                    ctrl: 1,
                    value: note,
                });
                self.send(Command::Relocate(device));
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
    /// One engine call: the UI side's queued commands, the mode, the host sync, then the block.
    fn next_block(&mut self) -> &[i16] {
        self.audio.drain();
        self.hold_mode();
        self.sync();
        self.audio.render_call();
        self.blocks += 1;
        self.mapping.follow(self.audio.last_snapshot());
        self.audio.output()
    }
}

/// How engine frames become host frames.
enum Out {
    /// At 44.1 kHz: the current block and the next frame of it to hand out.
    Direct {
        block: [i16; 2 * BLOCK],
        pos: usize,
    },
    Resampled(Resampler),
}

/// The process side of one plugin instance.
pub struct Core {
    engine: Engine,
    out: Out,
    /// The parameter values of the last call.
    last: [i32; COUNT],
    rate: u32,
    /// The host frames produced.
    frames: u64,
}

impl Core {
    /// The process side for host rate `rate`, with the parameter values `values` as the ones
    /// already seen (the engine was built with them, `crate::instance`) and the main thread's
    /// settings `shared`. `audio` must run [`BLOCK`]-frame calls.
    pub fn new(audio: Audio, rate: u32, values: &[i32; COUNT], shared: Arc<Shared>) -> Core {
        assert_eq!(audio.call_frames(), BLOCK);
        let out = if rate == ENGINE_RATE {
            // the block ahead: the first 64 host frames are silence
            Out::Direct {
                block: [0; 2 * BLOCK],
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
                playing: values[MODE] != 0,
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
            last: *values,
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

    /// Host events that could not be applied because the garbage queue was full.
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
        // the changes come after their frame (see the module doc)
        self.produce(&mut l[..1], &mut r[..1]);
        self.apply(values);
        let mut done = 1;
        for e in events {
            let t = (e.frame as usize).min(n - 1);
            if t + 1 > done {
                self.produce(&mut l[done..=t], &mut r[done..=t]);
                done = t + 1;
            }
            self.engine.message(&e.message);
        }
        self.produce(&mut l[done..n], &mut r[done..n]);
        self.frames += n as u64;
    }

    /// Applies the parameter values that changed since the last call.
    fn apply(&mut self, values: &[i32; COUNT]) {
        for (i, (&v, last)) in values.iter().zip(self.last.iter_mut()).enumerate() {
            if v == *last {
                continue;
            }
            *last = v;
            let s = &SPECS[i];
            if i == MODE {
                self.engine.playing = v != 0;
                continue;
            }
            let holds = params::engine_byte(self.engine.audio.newest(), s.target)
                .is_some_and(|b| params::value_of(s, b) == v);
            if holds {
                continue;
            }
            if let Some(c) = params::command(s, v) {
                self.engine.send(c);
            }
        }
    }

    fn produce(&mut self, l: &mut [f32], r: &mut [f32]) {
        match &mut self.out {
            Out::Direct { block, pos } => {
                for (ol, or) in l.iter_mut().zip(r.iter_mut()) {
                    if *pos == BLOCK {
                        block.copy_from_slice(self.engine.next_block());
                        *pos = 0;
                    }
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
