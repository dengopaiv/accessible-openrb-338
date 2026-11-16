//! The plugin's process side (task-197): the engine in 64-frame blocks for any host buffer
//! size, with the host's parameter changes, and the resampler when the host is not at 44.1 kHz
//! (`docs/plugin.md` section 4).
//!
//! # Timing
//!
//! The engine runs in calls of one block ([`BLOCK`] frames, `orb_player::Audio` made with
//! `Player::with_call_frames`). Host frame `j` (counted from the activation) plays engine frame
//! `j - 64`: one block is rendered ahead, and the plugin reports 64 frames of latency. The block
//! is rendered when its first frame is due, and a host event at frame `t` (a parameter change)
//! is applied right after frame `t` is produced. So the block that plays at host frames
//! `[64m, 64m + 64)` (after the host's latency compensation, the block of those very frames)
//! holds exactly the events of those frames, and the output is a function of the events' times
//! alone, not of the host's buffer sizes.
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
//! Nothing here allocates, frees or locks: the queues and the snapshot are orb-player's
//! lock-free ones, the parameter table is a constant, and the buffers are made at activation.

use orb_player::{Audio, Command};

use crate::params::{self, COUNT, MODE, SPECS};
use crate::resample::{Blocks, Resampler};

/// The frames of one engine block, and of the reported latency at 44.1 kHz.
pub const BLOCK: usize = 64;

/// The engine and its block source.
struct Engine {
    audio: Audio,
    /// The mode parameter as last seen: the engine plays while it is set.
    playing: bool,
    /// The host events that could not be applied (the garbage queue was full); counted, not
    /// retried.
    dropped: u64,
}

impl Engine {
    /// The engine's song plays or stops as the mode says. A song loaded over a playing one
    /// waits (pending) until Play, which switches to it.
    fn hold_mode(&mut self) {
        let a = &mut self.audio;
        let c = if self.playing {
            (!a.current().playing() || a.has_pending()).then_some(Command::Play)
        } else {
            a.current().playing().then_some(Command::Stop)
        };
        if let Some(c) = c
            && a.apply_now(c).is_err()
        {
            self.dropped += 1;
        }
    }
}

impl Blocks for Engine {
    /// One engine call: the UI side's queued commands, the mode, then the block.
    fn next_block(&mut self) -> &[i16] {
        self.audio.drain();
        self.hold_mode();
        self.audio.render_call()
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
}

impl Core {
    /// The process side for host rate `rate`, with the parameter values `values` as the ones
    /// already seen (the engine was built with them, `crate::instance`). `audio` must run
    /// [`BLOCK`]-frame calls.
    pub fn new(audio: Audio, rate: u32, values: &[i32; COUNT]) -> Core {
        assert_eq!(audio.call_frames(), BLOCK);
        let out = if rate == crate::resample::ENGINE_RATE {
            // the block ahead: the first 64 host frames are silence
            Out::Direct {
                block: [0; 2 * BLOCK],
                pos: 0,
            }
        } else {
            Out::Resampled(Resampler::new(rate))
        };
        Core {
            engine: Engine {
                audio,
                playing: values[MODE] != 0,
                dropped: 0,
            },
            out,
            last: *values,
            rate,
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

    /// Fills `l` and `r` (the same length) with the next host frames; `values` are the
    /// parameters' values for this call. A call of no frames takes no changes: they are seen at
    /// the next call's first frame, where they belong.
    pub fn process(&mut self, l: &mut [f32], r: &mut [f32], values: &[i32; COUNT]) {
        let n = l.len().min(r.len());
        if n == 0 {
            return;
        }
        // the changes come after the first frame (see the module doc)
        self.produce(&mut l[..1], &mut r[..1]);
        self.apply(values);
        self.produce(&mut l[1..n], &mut r[1..n]);
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
            if let Some(c) = params::command(s, v)
                && self.engine.audio.apply_now(c).is_err()
            {
                self.engine.dropped += 1;
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
