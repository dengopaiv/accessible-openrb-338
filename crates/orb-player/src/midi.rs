//! MIDI remote control (task-182): the original's **Standard Mapping**, as a layer that turns
//! timed MIDI bytes into the player's commands, whatever the bytes come from.
//!
//! The mapping, with the original's addresses and where it differs from the manual, is
//! `docs/standard_midi_mapping.md`; `docs/midi.md` is the original's MIDI input as a whole.
//!
//! # Use
//!
//! A source (a MIDI port of the standalone app, task-188; the host's events of the VST3/CLAP
//! plugin) hands [`StandardMapping::feed`] its bytes with a time each, in the source's own unit
//! (the plugin: the frame of the event; a port: its timestamp). The layer gives back
//! [`Timed`] [`Action`]s carrying the same time, in input order; [`Action::command`] is the
//! player's [`Command`] of an action, and the actions without one change only the panel's view
//! (the shown bank, the focus, the selected drum instrument, the keyboard mode). When to apply
//! an action (at once, or at the 64-frame block its time falls in) is the caller's choice: the
//! original applies its MIDI input at the UI's idle time, not at an exact sample
//! (`docs/midi.md`).
//!
//! The toggles (a mute, an effect's on switch, Record) and the stop's locate depend on the song's
//! state: the layer keeps a [`State`] that the caller refreshes from the player
//! ([`StandardMapping::follow`], with a [`Snapshot`]) and that each action the layer gives updates
//! at once, so two toggles in a row work before the engine has run.
//!
//! The parser ([`Parser`]) takes any byte stream: running status, system real-time bytes in the
//! middle of a message, system exclusive (skipped), and stray data bytes (dropped).
//!
//! # What it does not do
//!
//! Only the Standard Mapping, on one channel ([`StandardMapping::channel`], default channel 1
//! as the original's preferences). Quick Mapping, ports, the MIDI LED and recording by MIDI in
//! the app are task-188's; notes as notes are task-183's; MIDI clock is task-184's. The notes
//! below 64 in "Program Synth from Keyboard" mode (the original's step programming by MIDI) are
//! not mapped (`docs/standard_midi_mapping.md` section 7; task-189).

use orb_engine::song::Device;

use crate::{Command, Snapshot};

/// The last bar start in ticks (bar 999), the original's locate clamp (`sub_429DBF`).
pub const MAX_POSITION: u32 = 766_464;
/// Ticks per bar.
const BAR: u32 = 768;

/// A value with the time the source gave it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Timed<T> {
    pub time: u64,
    pub event: T,
}

/// A complete MIDI message (channels 0..=15).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Message {
    NoteOff {
        channel: u8,
        note: u8,
        velocity: u8,
    },
    NoteOn {
        channel: u8,
        note: u8,
        velocity: u8,
    },
    PolyPressure {
        channel: u8,
        note: u8,
        pressure: u8,
    },
    ControlChange {
        channel: u8,
        controller: u8,
        value: u8,
    },
    ProgramChange {
        channel: u8,
        program: u8,
    },
    ChannelPressure {
        channel: u8,
        pressure: u8,
    },
    /// The 14-bit value, 0x2000 the centre.
    PitchBend {
        channel: u8,
        value: u16,
    },
    /// A system common message (0xF1..0xF6; the status and up to two data bytes). A system
    /// exclusive message is skipped and gives nothing.
    System {
        status: u8,
        data: [u8; 2],
    },
    /// A system real-time byte (0xF8..0xFF: clock, start, continue, stop...).
    RealTime(u8),
}

/// A MIDI byte-stream parser: running status, real-time bytes anywhere, system exclusive skipped,
/// data bytes without a status dropped.
#[derive(Clone, Debug, Default)]
pub struct Parser {
    /// The running status (a channel voice status), or the system common status being read.
    status: Option<u8>,
    data: [u8; 2],
    have: usize,
    sysex: bool,
}

/// The data bytes a status takes.
fn data_len(status: u8) -> usize {
    match status & 0xF0 {
        0xC0 | 0xD0 => 1,
        0xF0 => match status {
            0xF1 | 0xF3 => 1,
            0xF2 => 2,
            _ => 0,
        },
        _ => 2,
    }
}

impl Parser {
    pub fn new() -> Self {
        Self::default()
    }

    /// Parses `bytes`, all at `time`, and appends the complete messages to `out`. A message can
    /// span calls.
    pub fn feed(&mut self, time: u64, bytes: &[u8], out: &mut Vec<Timed<Message>>) {
        for &b in bytes {
            if let Some(m) = self.byte(b) {
                out.push(Timed { time, event: m });
            }
        }
    }

    /// One byte; the message it completes.
    pub fn byte(&mut self, b: u8) -> Option<Message> {
        if b >= 0xF8 {
            // real-time: does not touch the message being read
            return Some(Message::RealTime(b));
        }
        if b >= 0x80 {
            self.have = 0;
            self.sysex = b == 0xF0;
            if b == 0xF0 || b == 0xF7 {
                self.status = None;
                return None;
            }
            self.status = Some(b);
            if b >= 0xF0 {
                // system common: cancels the running status
                if data_len(b) == 0 {
                    self.status = None;
                    return Some(Message::System {
                        status: b,
                        data: [0; 2],
                    });
                }
            }
            return None;
        }
        if self.sysex {
            return None;
        }
        let status = self.status?;
        self.data[self.have] = b;
        self.have += 1;
        if self.have < data_len(status) {
            return None;
        }
        self.have = 0;
        let [d0, d1] = self.data;
        let channel = status & 0x0F;
        Some(match status & 0xF0 {
            0x80 => Message::NoteOff {
                channel,
                note: d0,
                velocity: d1,
            },
            0x90 => Message::NoteOn {
                channel,
                note: d0,
                velocity: d1,
            },
            0xA0 => Message::PolyPressure {
                channel,
                note: d0,
                pressure: d1,
            },
            0xB0 => Message::ControlChange {
                channel,
                controller: d0,
                value: d1,
            },
            0xC0 => Message::ProgramChange {
                channel,
                program: d0,
            },
            0xD0 => Message::ChannelPressure {
                channel,
                pressure: d0,
            },
            0xE0 => Message::PitchBend {
                channel,
                value: d0 as u16 | (d1 as u16) << 7,
            },
            _ => {
                // system common with data: no running status
                self.status = None;
                Message::System {
                    status,
                    data: [d0, d1],
                }
            }
        })
    }
}

/// The Options menu's keyboard mode, which MIDI notes 64, 95 and 96 set
/// (`docs/standard_midi_mapping.md` section 6).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum KeyMode {
    /// "Program Synth from Keyboard" (the original's mode 10).
    Synth,
    /// "Select Patterns from Keyboard" (mode 11, the original's default).
    #[default]
    Patterns,
}

/// What a MIDI message does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    /// A knob or switch of a 303, the 808, the 909 or an effect, a pattern section's on/off
    /// switch (controller 0) or pattern (controller 1): [`Command::DeviceParam`].
    DeviceParam {
        device: Device,
        ctrl: u8,
        value: u8,
    },
    /// [`Command::MixerGlobal`]: 0 the master level, 1 the compressor id, 2 the PCF id.
    MixerGlobal {
        id: u8,
        value: u8,
    },
    /// [`Command::MixerChannel`]: channel 0..=3's field (0 on, 1 level, 2 pan, 3 delay send,
    /// 4 dist).
    MixerChannel {
        channel: u8,
        field: u8,
        value: u8,
    },
    /// [`Command::Shuffle`].
    Shuffle(u8),
    Play,
    Stop,
    Record(bool),
    Locate(u32),
    /// A pattern section shows bank 0..=3 (A..D); no command.
    Bank {
        row: u8,
        bank: u8,
    },
    /// The focus moves to row 0..=3 (303 #1, 303 #2, 808, 909); no command (the original also
    /// tells the DLL, song msg 35 `{22, row + 1}`, which the engine of orb has no use for yet).
    Focus(u8),
    /// The 808 or the 909 selects instrument 0..=11 for its step buttons; no command.
    Instrument {
        device: Device,
        index: u8,
    },
    /// The keyboard mode changes; no command.
    KeyMode(KeyMode),
}

impl Action {
    /// The player's command, `None` for a view-only action.
    pub fn command(&self) -> Option<Command> {
        Some(match *self {
            Action::DeviceParam {
                device,
                ctrl,
                value,
            } => Command::DeviceParam {
                device,
                ctrl,
                value,
            },
            Action::MixerGlobal { id, value } => Command::MixerGlobal { id, value },
            Action::MixerChannel {
                channel,
                field,
                value,
            } => Command::MixerChannel {
                channel,
                field,
                value,
            },
            Action::Shuffle(v) => Command::Shuffle(v),
            Action::Play => Command::Play,
            Action::Stop => Command::Stop,
            Action::Record(on) => Command::Record(on),
            Action::Locate(t) => Command::Locate(t),
            Action::Bank { .. }
            | Action::Focus(_)
            | Action::Instrument { .. }
            | Action::KeyMode(_) => {
                return None;
            }
        })
    }
}

/// The song's state the toggles read: the values the original's views hold.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct State {
    pub playing: bool,
    pub recording: bool,
    pub song_mode: bool,
    /// Ticks.
    pub position: u32,
    pub loop_start: u32,
    /// 0 off, 1 the master, 2..=5 channel 0..=3.
    pub comp_id: u8,
    /// 0 off, 1..=4 channel 0..=3.
    pub pcf_id: u8,
    pub channel_on: [bool; 4],
    pub channel_dist: [bool; 4],
    /// The on switches of the delay, the distortion, the PCF and the compressor.
    pub effect_on: [bool; 4],
    /// The pattern sections' on switches (303 #1, 303 #2, 808, 909).
    pub row_on: [bool; 4],
    /// The pattern each section has selected (0..=31).
    pub pattern: [u8; 4],
}

impl Default for State {
    /// A new song's: everything off, the channels on.
    fn default() -> Self {
        State {
            playing: false,
            recording: false,
            song_mode: false,
            position: 0,
            loop_start: 0,
            comp_id: 0,
            pcf_id: 0,
            channel_on: [true; 4],
            channel_dist: [false; 4],
            effect_on: [false; 4],
            row_on: [true; 4],
            pattern: [0; 4],
        }
    }
}

/// The pattern sections' devices, by row.
pub const ROWS: [Device; 4] = [
    Device::Tb303_1,
    Device::Tb303_2,
    Device::Tr808,
    Device::Tr909,
];
/// The effects, in [`State::effect_on`] order.
const EFFECTS: [Device; 4] = [
    Device::Delay,
    Device::Distortion,
    Device::Pcf,
    Device::Compressor,
];

impl State {
    /// The state a player snapshot shows (the live blocks and the mixer).
    pub fn from_snapshot(s: &Snapshot) -> State {
        let live = |d: Device, i: usize| s.live[d as usize].as_slice().get(i).copied();
        let mut st = State {
            playing: s.playing,
            recording: s.recording,
            song_mode: s.song_mode,
            position: s.tick.max(0) as u32,
            loop_start: s.loop_start,
            comp_id: s.mixer.comp_id,
            pcf_id: s.mixer.pcf_id,
            ..State::default()
        };
        for (c, ch) in s.mixer.channels.iter().enumerate() {
            st.channel_on[c] = ch.enabled != 0;
            st.channel_dist[c] = ch.dist != 0;
        }
        for (k, d) in EFFECTS.into_iter().enumerate() {
            st.effect_on[k] = live(d, 0).is_some_and(|v| v != 0);
        }
        // controllers 0 and 1 are not permuted on the 909 (`tr909::internal_index`)
        for (r, d) in ROWS.into_iter().enumerate() {
            st.row_on[r] = live(d, 0).is_none_or(|v| v != 0);
            st.pattern[r] = live(d, 1).unwrap_or(0).min(31);
        }
        st
    }
}

/// The Standard Mapping (`docs/standard_midi_mapping.md`).
#[derive(Clone, Debug, Default)]
pub struct StandardMapping {
    /// The channel it listens to, 0..=15 (MIDI channel 1..16); the others are ignored, as the
    /// original's input callback drops them (`sub_408FD4`).
    pub channel: u8,
    /// The keyboard mode: notes 12..=63 select patterns in [`KeyMode::Patterns`] only.
    pub mode: KeyMode,
    /// The state the toggles read; [`StandardMapping::follow`] refreshes it.
    pub state: State,
    /// The bank each pattern section shows.
    bank: [u8; 4],
    parser: Parser,
    messages: Vec<Timed<Message>>,
}

impl StandardMapping {
    pub fn new() -> Self {
        Self::default()
    }

    /// The bank row `r` shows: the one a bank note chose, or the selected pattern's.
    pub fn bank(&self, r: usize) -> u8 {
        self.bank[r]
    }

    /// Takes the song's state from a player snapshot. A section whose pattern changed shows that
    /// pattern's bank, as the panel does.
    pub fn follow(&mut self, s: &Snapshot) {
        self.set_state(State::from_snapshot(s));
    }

    /// [`StandardMapping::follow`] with a state the caller made.
    pub fn set_state(&mut self, st: State) {
        for r in 0..4 {
            if st.pattern[r] != self.state.pattern[r] {
                self.bank[r] = st.pattern[r] / 8;
            }
        }
        self.state = st;
    }

    /// Parses `bytes` (at `time`) and appends what they do to `out`.
    pub fn feed(&mut self, time: u64, bytes: &[u8], out: &mut Vec<Timed<Action>>) {
        let mut messages = std::mem::take(&mut self.messages);
        messages.clear();
        self.parser.feed(time, bytes, &mut messages);
        for m in &messages {
            self.message(m.time, &m.event, out);
        }
        self.messages = messages;
    }

    /// What one message does, appended to `out`.
    pub fn message(&mut self, time: u64, m: &Message, out: &mut Vec<Timed<Action>>) {
        let mut push = |a: Action| out.push(Timed { time, event: a });
        match *m {
            Message::ControlChange {
                channel,
                controller,
                value,
            } if channel == self.channel => {
                if let Some(a) = control_change(controller, value) {
                    push(a);
                }
            }
            // A note-on of velocity 0 is a note-off (the MIDI convention). The original takes
            // it as a note-on (orb differs: docs/standard_midi_mapping.md section 8, OB-24).
            Message::NoteOn {
                channel,
                note,
                velocity,
            } if channel == self.channel && velocity > 0 => self.note_on(note, &mut push),
            _ => {}
        }
    }

    /// A note-on (`sub_42EAAF`). No mapped function acts on a note-off.
    fn note_on(&mut self, note: u8, push: &mut impl FnMut(Action)) {
        let st = &mut self.state;
        match note {
            64 => {
                self.mode = match self.mode {
                    KeyMode::Synth => KeyMode::Patterns,
                    KeyMode::Patterns => KeyMode::Synth,
                };
                push(Action::KeyMode(self.mode));
            }
            95 => {
                self.mode = KeyMode::Synth;
                push(Action::KeyMode(self.mode));
            }
            96 => {
                self.mode = KeyMode::Patterns;
                push(Action::KeyMode(self.mode));
            }
            // Program Synth from Keyboard: the step programming, not mapped
            0..64 if self.mode == KeyMode::Synth => {}
            12..=63 => {
                let r = ((note - 12) / 13) as usize;
                let k = (note - 12) % 13;
                let device = ROWS[r];
                match k {
                    0 => {
                        st.row_on[r] = !st.row_on[r];
                        push(Action::DeviceParam {
                            device,
                            ctrl: 0,
                            value: st.row_on[r] as u8,
                        });
                    }
                    1..=4 => {
                        self.bank[r] = k - 1;
                        push(Action::Bank {
                            row: r as u8,
                            bank: k - 1,
                        });
                    }
                    _ => {
                        let p = (k - 5) + 8 * self.bank[r];
                        st.pattern[r] = p;
                        push(Action::DeviceParam {
                            device,
                            ctrl: 1,
                            value: p,
                        });
                    }
                }
            }
            65..=68 => push(Action::Focus(note - 65)),
            69 => {
                st.playing = true;
                push(Action::Play);
            }
            70 => {
                // `sub_429FE6`
                if st.recording {
                    st.recording = false;
                    push(Action::Record(false));
                }
                if st.playing {
                    st.playing = false;
                    push(Action::Stop);
                } else {
                    st.position = if st.loop_start >= st.position {
                        0
                    } else {
                        st.loop_start
                    };
                    push(Action::Locate(st.position));
                }
            }
            71 if st.song_mode => {
                st.recording = !st.recording;
                push(Action::Record(st.recording));
            }
            72 | 73 if st.song_mode => {
                // `sub_429DF2(-+7680)`: ten bars from the start of the current bar
                let start = BAR * (st.position / BAR);
                st.position = if note == 72 {
                    start.saturating_sub(10 * BAR)
                } else {
                    (start + 10 * BAR).min(MAX_POSITION)
                };
                push(Action::Locate(st.position));
            }
            74..=77 => {
                // PCF, delay, distortion, compressor
                let k = [2, 0, 1, 3][(note - 74) as usize];
                st.effect_on[k] = !st.effect_on[k];
                push(Action::DeviceParam {
                    device: EFFECTS[k],
                    ctrl: 0,
                    value: st.effect_on[k] as u8,
                });
            }
            78..=93 => {
                // `sub_413256`: per channel mix (on), dist, PCF, compressor
                let c = (note - 78) / 4;
                match (note - 78) % 4 {
                    0 => {
                        let on = &mut st.channel_on[c as usize];
                        *on = !*on;
                        push(Action::MixerChannel {
                            channel: c,
                            field: 0,
                            value: *on as u8,
                        });
                    }
                    1 => {
                        let dist = &mut st.channel_dist[c as usize];
                        *dist = !*dist;
                        push(Action::MixerChannel {
                            channel: c,
                            field: 4,
                            value: *dist as u8,
                        });
                    }
                    2 => {
                        st.pcf_id = if st.pcf_id == c + 1 { 0 } else { c + 1 };
                        push(Action::MixerGlobal {
                            id: 2,
                            value: st.pcf_id,
                        });
                    }
                    _ => {
                        st.comp_id = if st.comp_id == c + 2 { 0 } else { c + 2 };
                        push(Action::MixerGlobal {
                            id: 1,
                            value: st.comp_id,
                        });
                    }
                }
            }
            94 => {
                st.comp_id = if st.comp_id == 1 { 0 } else { 1 };
                push(Action::MixerGlobal {
                    id: 1,
                    value: st.comp_id,
                });
            }
            _ => {}
        }
    }
}

/// The 808's controllers that are two-position switches (the shared-slot selectors).
const TR808_SWITCHES: [u8; 5] = [11, 14, 17, 19, 21];

/// What a control change does (`sub_42EAAF` from 0x42EE8B and the handlers it calls).
pub fn control_change(cc: u8, v: u8) -> Option<Action> {
    let param = |device: Device, ctrl: u8, value: u8| {
        Some(Action::DeviceParam {
            device,
            ctrl,
            value,
        })
    };
    let switch = |v: u8| (v >= 64) as u8;
    match cc {
        // `sub_4137E2`
        7 => Some(Action::MixerGlobal { id: 0, value: v }),
        11..=22 => Some(Action::MixerChannel {
            channel: (cc - 11) / 3,
            field: 1 + (cc - 11) % 3,
            value: v,
        }),
        // `sub_4405B5`: the waveform is a switch, then tune .. accent
        23..=36 => {
            let device = if cc < 30 {
                Device::Tb303_1
            } else {
                Device::Tb303_2
            };
            match (cc - 23) % 7 {
                0 => param(device, 8, switch(v)),
                k => param(device, 1 + k, v),
            }
        }
        // `sub_443C92`
        37..=63 => {
            let ctrl = cc - 35;
            let value = if TR808_SWITCHES.contains(&ctrl) {
                switch(v)
            } else {
                v
            };
            param(Device::Tr808, ctrl, value)
        }
        64 => Some(Action::Instrument {
            device: Device::Tr808,
            index: instrument(v),
        }),
        // `sub_447D9D`: the hi-hat, RS and CP levels are controllers 22, 20, 21
        65..=92 => {
            let ctrl = match cc {
                83 => 22,
                84 => 20,
                85 => 21,
                _ => cc - 63,
            };
            param(Device::Tr909, ctrl, v)
        }
        93 => Some(Action::Instrument {
            device: Device::Tr909,
            index: instrument(v),
        }),
        // `sub_41BE94`
        94 => param(Device::Pcf, 4, v.min(54)),
        95 => param(Device::Pcf, 6, switch(v)),
        96..=98 => param(Device::Pcf, cc - 95, v),
        99 => param(Device::Pcf, 5, v),
        // `sub_40827C`: steps 1..=32
        100 => param(Device::Delay, 1, (32 * v as u32 / 128 + 1) as u8),
        101 => param(Device::Delay, 2, switch(v)),
        102 => param(Device::Delay, 4, v),
        103 => param(Device::Delay, 3, v),
        // `sub_408C98`
        104 => param(Device::Distortion, 2, v),
        105 => param(Device::Distortion, 1, v),
        // `sub_404798`
        106 | 107 => param(Device::Compressor, cc - 105, v),
        // `sub_429CF7`
        108 => Some(Action::Shuffle(v)),
        _ => None,
    }
}

/// The instrument 0..=11 of an instrument-selection value (`12 * v / 128`).
fn instrument(v: u8) -> u8 {
    (12 * v as u32 / 128) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The actions of `bytes` fed at time 0 to a new mapping.
    fn run(m: &mut StandardMapping, bytes: &[u8]) -> Vec<Action> {
        let mut out = vec![];
        m.feed(0, bytes, &mut out);
        out.into_iter().map(|t| t.event).collect()
    }

    fn one(bytes: &[u8]) -> Vec<Action> {
        run(&mut StandardMapping::new(), bytes)
    }

    fn dp(device: Device, ctrl: u8, value: u8) -> Action {
        Action::DeviceParam {
            device,
            ctrl,
            value,
        }
    }

    #[test]
    fn parser_running_status_realtime_and_sysex() {
        let mut p = Parser::new();
        let mut out = vec![];
        // CC, running status, a clock inside a message, a sysex with data, a stray data byte
        p.feed(
            5,
            &[
                0xB0, 7, 100, 11, 0xF8, 20, 0xF0, 1, 2, 3, 0xF7, 9, 0x91, 60, 0,
            ],
            &mut out,
        );
        let ev: Vec<_> = out.iter().map(|t| t.event).collect();
        assert_eq!(
            ev,
            [
                Message::ControlChange {
                    channel: 0,
                    controller: 7,
                    value: 100
                },
                Message::RealTime(0xF8),
                Message::ControlChange {
                    channel: 0,
                    controller: 11,
                    value: 20
                },
                Message::NoteOn {
                    channel: 1,
                    note: 60,
                    velocity: 0
                },
            ]
        );
        assert!(out.iter().all(|t| t.time == 5));
    }

    #[test]
    fn parser_message_split_across_calls_and_status_interrupts() {
        let mut p = Parser::new();
        let mut out = vec![];
        p.feed(1, &[0xB2, 25], &mut out);
        assert!(out.is_empty());
        p.feed(2, &[90], &mut out);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].time, 2);
        // a new status abandons the half message; a system common message ends running status
        out.clear();
        p.feed(3, &[0x90, 60, 0xC3, 5, 0xF6, 1, 2, 0xE0, 0, 64], &mut out);
        let ev: Vec<_> = out.iter().map(|t| t.event).collect();
        assert_eq!(
            ev,
            [
                Message::ProgramChange {
                    channel: 3,
                    program: 5
                },
                Message::System {
                    status: 0xF6,
                    data: [0, 0]
                },
                Message::PitchBend {
                    channel: 0,
                    value: 0x2000
                },
            ]
        );
    }

    #[test]
    fn times_are_kept_per_feed() {
        let mut m = StandardMapping::new();
        let mut out = vec![];
        m.feed(10, &[0xB0, 25, 1], &mut out);
        m.feed(70, &[25, 2], &mut out);
        assert_eq!(out[0].time, 10);
        assert_eq!(out[1].time, 70);
        assert_eq!(out[1].event, dp(Device::Tb303_1, 3, 2));
    }

    #[test]
    fn mixer_controls() {
        assert_eq!(
            one(&[0xB0, 7, 99]),
            [Action::MixerGlobal { id: 0, value: 99 }]
        );
        let all: Vec<_> = (11..=22).flat_map(|cc| one(&[0xB0, cc, cc])).collect();
        let want: Vec<_> = (0..4)
            .flat_map(|c| {
                (1..=3).map(move |f| Action::MixerChannel {
                    channel: c,
                    field: f,
                    value: 11 + 3 * c + f - 1,
                })
            })
            .collect();
        assert_eq!(all, want);
        for cc in [8, 9, 10] {
            assert!(one(&[0xB0, cc, 5]).is_empty());
        }
    }

    #[test]
    fn synth_controls_and_the_waveform_switch() {
        assert_eq!(one(&[0xB0, 23, 63]), [dp(Device::Tb303_1, 8, 0)]);
        assert_eq!(one(&[0xB0, 23, 64]), [dp(Device::Tb303_1, 8, 1)]);
        assert_eq!(one(&[0xB0, 30, 127]), [dp(Device::Tb303_2, 8, 1)]);
        for (k, ctrl) in (2..=7).enumerate() {
            let k = k as u8;
            assert_eq!(one(&[0xB0, 24 + k, 77]), [dp(Device::Tb303_1, ctrl, 77)]);
            assert_eq!(one(&[0xB0, 31 + k, 78]), [dp(Device::Tb303_2, ctrl, 78)]);
        }
    }

    #[test]
    fn drum_controls_switches_and_instrument_selection() {
        for cc in 37..=63u8 {
            let ctrl = cc - 35;
            let want = if TR808_SWITCHES.contains(&ctrl) {
                1
            } else {
                70
            };
            assert_eq!(one(&[0xB0, cc, 70]), [dp(Device::Tr808, ctrl, want)]);
        }
        assert_eq!(one(&[0xB0, 46, 63]), [dp(Device::Tr808, 11, 0)]);
        let tr909: Vec<u8> = (65..=92u8)
            .map(|cc| match one(&[0xB0, cc, 1])[0] {
                Action::DeviceParam { ctrl, .. } => ctrl,
                _ => unreachable!(),
            })
            .collect();
        let mut want: Vec<u8> = (2..=19).collect();
        want.extend([22, 20, 21]);
        want.extend(23..=29);
        assert_eq!(tr909, want);
        for (cc, device) in [(64, Device::Tr808), (93, Device::Tr909)] {
            for (v, index) in [(0, 0), (10, 0), (11, 1), (64, 6), (127, 11)] {
                assert_eq!(one(&[0xB0, cc, v]), [Action::Instrument { device, index }]);
            }
        }
    }

    #[test]
    fn effect_controls_and_their_scaling() {
        assert_eq!(one(&[0xB0, 94, 54]), [dp(Device::Pcf, 4, 54)]);
        assert_eq!(one(&[0xB0, 94, 127]), [dp(Device::Pcf, 4, 54)]);
        assert_eq!(one(&[0xB0, 95, 64]), [dp(Device::Pcf, 6, 1)]);
        assert_eq!(one(&[0xB0, 96, 3]), [dp(Device::Pcf, 1, 3)]);
        assert_eq!(one(&[0xB0, 97, 4]), [dp(Device::Pcf, 2, 4)]);
        assert_eq!(one(&[0xB0, 98, 5]), [dp(Device::Pcf, 3, 5)]);
        assert_eq!(one(&[0xB0, 99, 6]), [dp(Device::Pcf, 5, 6)]);
        for (v, steps) in [(0, 1), (3, 1), (4, 2), (64, 17), (127, 32)] {
            assert_eq!(one(&[0xB0, 100, v]), [dp(Device::Delay, 1, steps)]);
        }
        assert_eq!(one(&[0xB0, 101, 63]), [dp(Device::Delay, 2, 0)]);
        assert_eq!(one(&[0xB0, 102, 7]), [dp(Device::Delay, 4, 7)]);
        assert_eq!(one(&[0xB0, 103, 8]), [dp(Device::Delay, 3, 8)]);
        assert_eq!(one(&[0xB0, 104, 9]), [dp(Device::Distortion, 2, 9)]);
        assert_eq!(one(&[0xB0, 105, 10]), [dp(Device::Distortion, 1, 10)]);
        assert_eq!(one(&[0xB0, 106, 11]), [dp(Device::Compressor, 1, 11)]);
        assert_eq!(one(&[0xB0, 107, 12]), [dp(Device::Compressor, 2, 12)]);
        assert_eq!(one(&[0xB0, 108, 13]), [Action::Shuffle(13)]);
    }

    #[test]
    fn unmapped_controllers_and_messages_do_nothing() {
        let mut bytes = vec![];
        for cc in (0..7).chain(8..=10).chain(109..=127) {
            bytes.extend([0xB0, cc, 64]);
        }
        // pitch bend, aftertouch, program change, real-time, sysex, unmapped notes
        bytes.extend([
            0xE0, 0, 64, 0xD0, 9, 0xA0, 60, 9, 0xC0, 3, 0xFA, 0xF0, 0x7E, 0xF7,
        ]);
        bytes.extend([0x90, 0, 100, 0x90, 11, 100, 0x90, 97, 100, 0x90, 127, 100]);
        assert!(one(&bytes).is_empty());
    }

    #[test]
    fn other_channels_are_ignored() {
        let mut m = StandardMapping::new();
        assert!(run(&mut m, &[0xB1, 25, 9, 0x9F, 69, 100]).is_empty());
        m.channel = 15;
        assert_eq!(run(&mut m, &[0xBF, 25, 9]), [dp(Device::Tb303_1, 3, 9)]);
        assert!(run(&mut m, &[0xB0, 25, 9]).is_empty());
    }

    #[test]
    fn pattern_selection_by_bank_and_pattern_notes() {
        let mut m = StandardMapping::new();
        // 303 #1: pattern 3 of bank A, then bank C (note 15), pattern 8 (note 24) = 23
        assert_eq!(
            run(&mut m, &[0x90, 19, 100, 0x80, 19, 0]),
            [dp(Device::Tb303_1, 1, 2)]
        );
        assert_eq!(
            run(&mut m, &[0x90, 15, 100, 0x90, 24, 100]),
            [Action::Bank { row: 0, bank: 2 }, dp(Device::Tb303_1, 1, 23)]
        );
        // the 909: bank D (55), pattern 1 (56); 808 bank B (40), pattern 5 (47)
        assert_eq!(
            run(
                &mut m,
                &[0x90, 55, 1, 0x90, 56, 1, 0x90, 40, 1, 0x90, 47, 1]
            ),
            [
                Action::Bank { row: 3, bank: 3 },
                dp(Device::Tr909, 1, 24),
                Action::Bank { row: 2, bank: 1 },
                dp(Device::Tr808, 1, 12),
            ]
        );
        // on/off toggles on the note-on only (303 #2: note 25)
        assert_eq!(
            run(&mut m, &[0x90, 25, 1, 0x80, 25, 0, 0x90, 25, 1]),
            [dp(Device::Tb303_2, 0, 0), dp(Device::Tb303_2, 0, 1)]
        );
        // a song's pattern change shows its bank
        let mut st = m.state.clone();
        st.pattern[1] = 17;
        m.set_state(st);
        assert_eq!(m.bank(1), 2);
        // pattern 2 (note 31) of the shown bank C
        assert_eq!(run(&mut m, &[0x90, 31, 1]), [dp(Device::Tb303_2, 1, 17)]);
    }

    #[test]
    fn keyboard_mode_notes_gate_pattern_selection() {
        let mut m = StandardMapping::new();
        assert_eq!(
            run(&mut m, &[0x90, 95, 1]),
            [Action::KeyMode(KeyMode::Synth)]
        );
        // Program Synth mode: notes below 64 are the step programming, not mapped
        assert!(run(&mut m, &[0x90, 19, 1, 0x90, 12, 1]).is_empty());
        // the other notes work in both modes
        assert_eq!(run(&mut m, &[0x90, 66, 1]), [Action::Focus(1)]);
        assert_eq!(
            run(&mut m, &[0x90, 64, 1, 0x80, 64, 0]),
            [Action::KeyMode(KeyMode::Patterns)]
        );
        assert_eq!(
            run(&mut m, &[0x90, 64, 1]),
            [Action::KeyMode(KeyMode::Synth)]
        );
        assert_eq!(
            run(&mut m, &[0x90, 96, 1]),
            [Action::KeyMode(KeyMode::Patterns)]
        );
        assert_eq!(run(&mut m, &[0x90, 19, 1]), [dp(Device::Tb303_1, 1, 2)]);
    }

    #[test]
    fn velocity_zero_note_on_is_a_note_off() {
        let mut m = StandardMapping::new();
        // a keyboard sending note-on 0 for its release toggles once per press
        assert_eq!(
            run(&mut m, &[0x90, 78, 100, 78, 0]),
            [Action::MixerChannel {
                channel: 0,
                field: 0,
                value: 0
            }]
        );
    }

    #[test]
    fn transport_notes() {
        let mut m = StandardMapping::new();
        assert_eq!(run(&mut m, &[0x90, 69, 1]), [Action::Play]);
        assert_eq!(run(&mut m, &[0x90, 70, 1]), [Action::Stop]);
        // stopped: locate to 0 (the loop start is at or after the position)
        assert_eq!(run(&mut m, &[0x90, 70, 1]), [Action::Locate(0)]);
        // record and ten-bar steps need song mode
        assert!(run(&mut m, &[0x90, 71, 1, 0x90, 72, 1, 0x90, 73, 1]).is_empty());
        let mut st = m.state.clone();
        st.song_mode = true;
        st.position = 5 * BAR + 100;
        st.loop_start = 2 * BAR;
        m.set_state(st);
        assert_eq!(
            run(&mut m, &[0x90, 73, 1, 0x90, 72, 1, 0x90, 72, 1]),
            [
                Action::Locate(15 * BAR),
                Action::Locate(5 * BAR),
                Action::Locate(0)
            ]
        );
        m.state.position = MAX_POSITION - 3 * BAR;
        assert_eq!(run(&mut m, &[0x90, 73, 1]), [Action::Locate(MAX_POSITION)]);
        m.state.position = 6 * BAR;
        m.state.playing = true;
        assert_eq!(
            run(&mut m, &[0x90, 71, 1, 0x90, 70, 1, 0x90, 70, 1]),
            [
                Action::Record(true),
                Action::Record(false),
                Action::Stop,
                Action::Locate(2 * BAR)
            ]
        );
    }

    #[test]
    fn effect_and_mixer_switch_notes_toggle() {
        let mut m = StandardMapping::new();
        assert_eq!(
            run(
                &mut m,
                &[
                    0x90, 74, 1, 0x90, 75, 1, 0x90, 76, 1, 0x90, 77, 1, 0x90, 74, 1
                ]
            ),
            [
                dp(Device::Pcf, 0, 1),
                dp(Device::Delay, 0, 1),
                dp(Device::Distortion, 0, 1),
                dp(Device::Compressor, 0, 1),
                dp(Device::Pcf, 0, 0),
            ]
        );
        let mixer: Vec<_> = (78..=94).flat_map(|n| run(&mut m, &[0x90, n, 1])).collect();
        let mut want = vec![];
        for c in 0..4u8 {
            want.push(Action::MixerChannel {
                channel: c,
                field: 0,
                value: 0,
            });
            want.push(Action::MixerChannel {
                channel: c,
                field: 4,
                value: 1,
            });
            want.push(Action::MixerGlobal {
                id: 2,
                value: c + 1,
            });
            want.push(Action::MixerGlobal {
                id: 1,
                value: c + 2,
            });
        }
        want.push(Action::MixerGlobal { id: 1, value: 1 });
        assert_eq!(mixer, want);
        // the PCF and compressor notes are radio buttons that also switch off
        assert_eq!(
            run(&mut m, &[0x90, 92, 1, 0x90, 94, 1]),
            [
                Action::MixerGlobal { id: 2, value: 0 },
                Action::MixerGlobal { id: 1, value: 0 }
            ]
        );
    }

    #[test]
    fn state_from_a_snapshot() {
        let mut s = Snapshot {
            tick: 1536,
            song_mode: true,
            ..Default::default()
        };
        s.mixer.pcf_id = 3;
        s.mixer.channels[2].enabled = 0;
        s.mixer.channels[1].dist = 1;
        s.live[Device::Delay as usize].set(&[1, 3, 0, 0, 0, 0]);
        s.live[Device::Tr909 as usize].set(&[0, 20, 0]);
        let mut m = StandardMapping::new();
        m.follow(&s);
        let st = &m.state;
        assert!(st.song_mode && st.effect_on[0] && !st.effect_on[1]);
        assert_eq!((st.position, st.pcf_id), (1536, 3));
        assert_eq!(st.channel_on, [true, true, false, true]);
        assert_eq!(st.channel_dist, [false, true, false, false]);
        assert_eq!(st.row_on, [true, true, true, false]);
        assert_eq!(st.pattern[3], 20);
        assert_eq!(m.bank(3), 2);
        // toggles read it
        assert_eq!(
            run(&mut m, &[0x90, 75, 1, 0x90, 51, 1]),
            [dp(Device::Delay, 0, 0), dp(Device::Tr909, 0, 1)]
        );
    }

    #[test]
    fn every_action_with_a_command_has_one() {
        for a in [
            dp(Device::Pcf, 1, 2),
            Action::MixerGlobal { id: 0, value: 1 },
            Action::MixerChannel {
                channel: 0,
                field: 1,
                value: 1,
            },
            Action::Shuffle(1),
            Action::Play,
            Action::Stop,
            Action::Record(true),
            Action::Locate(0),
        ] {
            assert!(a.command().is_some(), "{a:?}");
        }
        for a in [
            Action::Bank { row: 0, bank: 1 },
            Action::Focus(0),
            Action::Instrument {
                device: Device::Tr808,
                index: 0,
            },
            Action::KeyMode(KeyMode::Synth),
        ] {
            assert!(a.command().is_none(), "{a:?}");
        }
    }
}
