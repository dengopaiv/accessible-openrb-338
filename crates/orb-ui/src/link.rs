//! The panel wired to the real-time player (task-045).
//!
//! [`Link`] owns the UI half of an [`orb_player::Player`] and sits between it and the
//! [`Panel`] model:
//!
//! - **Out.** The model's events (`Panel::events`) become the messages the original's panel sends
//!   ([`Msg`]: song msg 35 `{id, v}` and the device messages of msg 41, the senders listed in
//!   `docs/song_load.md` section 4.10), and each message becomes the player's [`Command`] that
//!   stands for it. The messages are kept in [`Link::log`] when asked, which is what the tests
//!   compare with the original's wire format.
//! - **In.** After each engine call the snapshot the audio callback publishes moves the panel: the
//!   transport (position, play, mode, loop, tempo), every control's live value (so the song's
//!   automation moves the knobs), the pattern each section selects, the running lights of the drum
//!   machines and the meters. The panel shows the engine's values, as the original's does
//!   (`docs/ui_spec.md` section 11).
//!
//! # Timing
//!
//! A command takes effect in the next engine call (`orb_player` crate doc), and the snapshot
//! shows it after that call. Until then the snapshot still has the old values, and taking them
//! would put the controls back: [`Link::sync`] takes the engine's values only from a snapshot
//! that is newer than the last commands it sent. A loaded song that waits for Play (the old one
//! plays on) is not followed: the panel shows the song the commands address, the newest, as the
//! original's panel does.
//!
//! # Patterns
//!
//! The panel shows the selected pattern of each section at once, and an edit goes to that
//! pattern (the original's views keep a current pattern, `docs/song_load.md` section 4.10). The
//! link keeps a copy of the four pattern arrays ([`Link::patterns`], from the song and the edits
//! it sends) to fill the 303 step editor, the drum step strips and the length and shuffle
//! displays when a pattern is selected. The Edit menu's commands read a pattern whole
//! ([`Link::pattern`]) and write one whole ([`Link::set_pattern`]).

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

use orb_engine::frame::{Engine, LoadRefused, MixerParams};
use orb_engine::samples::{STARTUP_ID, SampleSet};
use orb_engine::seq::Event as TrackEvent;
use orb_engine::song::{Device, Song};
use orb_engine::tables::Tables;
use orb_engine::tr909;
use orb_player::{Audio, CALL_FRAMES, Command, LiveBlock, Player, Preview, Snapshot, Tracks};

use crate::model::{Event, Panel, Step303};

/// What the link drives: the UI half of a player. The standalone app's is its own
/// [`orb_player::Player`]; the plugin's editor (orb-plugin, task-201) has one that turns the
/// panel's commands into the plugin's host parameters and calls.
pub trait Port: Send {
    /// Queues a command; a full queue gives it back.
    fn send(&mut self, c: Command) -> Result<(), Command>;
    /// The engine's state after the last call.
    fn snapshot(&mut self) -> &Snapshot;
    /// The open song's tracks as the callback last published them.
    fn tracks(&mut self) -> &Tracks;
    /// Whether [`Port::tracks`] are the song's (recording): false where nothing records them.
    fn records(&self) -> bool {
        true
    }
    /// Drops what the callback let go of.
    fn collect_garbage(&mut self);
    /// Builds a song's engine on the player's process globals.
    fn build(
        &self,
        song: &Song,
        samples: Option<Arc<SampleSet>>,
    ) -> Result<Box<Engine>, LoadRefused>;
    /// Builds a new empty song's engine.
    fn build_new(&self, samples: Option<Arc<SampleSet>>) -> Box<Engine>;
    /// The drum machines of the newest song take `samples` (song msg 19).
    fn set_mod(&mut self, samples: Arc<SampleSet>) -> Result<(), Command>;
    /// The engine frames after which a snapshot can show the commands just sent: one call of
    /// the player ([`CALL_FRAMES`]); more where they reach the engine by a longer way.
    fn follow_frames(&self) -> u64 {
        CALL_FRAMES as u64
    }
}

impl Port for Player {
    fn send(&mut self, c: Command) -> Result<(), Command> {
        Player::send(self, c)
    }

    fn snapshot(&mut self) -> &Snapshot {
        Player::snapshot(self)
    }

    fn tracks(&mut self) -> &Tracks {
        Player::tracks(self)
    }

    fn collect_garbage(&mut self) {
        Player::collect_garbage(self);
    }

    fn build(
        &self,
        song: &Song,
        samples: Option<Arc<SampleSet>>,
    ) -> Result<Box<Engine>, LoadRefused> {
        Player::build(self, song, samples)
    }

    fn build_new(&self, samples: Option<Arc<SampleSet>>) -> Box<Engine> {
        Player::build_new(self, samples)
    }

    fn set_mod(&mut self, samples: Arc<SampleSet>) -> Result<(), Command> {
        Player::set_mod(self, samples)
    }
}

/// The devices of the four pattern sections (rows).
pub const ROW_DEVICES: [Device; 4] = [
    Device::Tb303_1,
    Device::Tb303_2,
    Device::Tr808,
    Device::Tr909,
];

/// What the original's panel sends: a song message of the exe's `sub_423C98` family, without the
/// song and slot bytes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Msg {
    /// Song msg 35 `{id, v}` (`docs/song_load.md` section 3): 0 song mode, 1 play, 2 record,
    /// 3 loop, 4 tempo (BPM * 1000), 8 locate (ticks), 12 loop start, 16 loop end, 20 shuffle,
    /// 21 the Vintage sound model (the Edit menu's item 323, handler 0x42C32E).
    Global { id: u8, value: u32 },
    /// Song msg 41 `[device, sub, data...]`.
    Device { dev: u8, sub: u8, data: Vec<u8> },
}

impl Msg {
    /// The message's bytes after the song and slot bytes, as the exe writes them
    /// (`docs/song_load.md` sections 2 and 3): msg 35 is `[35, id, value]` with the value a
    /// big-endian u32 for the ids 4, 8, 12 and 16 and one byte for the others; msg 41 is
    /// `[41, dev, sub, data...]`.
    pub fn encode(&self) -> Vec<u8> {
        match self {
            Msg::Global { id, value } => {
                let mut b = vec![35, *id];
                if matches!(id, 4 | 8 | 12 | 16) {
                    b.extend(value.to_be_bytes());
                } else {
                    b.push(*value as u8);
                }
                b
            }
            Msg::Device { dev, sub, data } => {
                let mut b = vec![41, *dev, *sub];
                b.extend(data);
                b
            }
        }
    }
}

/// The `{ctrl, v}` sub of a device (`docs/song_load.md` section 4.10).
pub fn param_sub(d: Device) -> Option<u8> {
    Some(match d {
        Device::Tb303_1 | Device::Tb303_2 => 40,
        Device::Tr808 => 57,
        Device::Tr909 => 74,
        Device::Delay => 84,
        Device::Distortion => 92,
        Device::Compressor => 102,
        Device::Pcf => 114,
        Device::Mixer => return None,
    })
}

/// The pattern header message `{p, field, v}` of a pattern device (303 42, 808 59, 909 76); the
/// pattern (44, 61, 78), step byte (45, 62, 79) messages follow at +2 and +3.
fn header_sub(d: Device) -> u8 {
    match d {
        Device::Tb303_1 | Device::Tb303_2 => 42,
        Device::Tr808 => 59,
        _ => 76,
    }
}

/// Where a pattern array starts in the device's data block, the pattern length and the step
/// length (`docs/song_load.md` sections 4.6 to 4.8).
fn layout(row: usize) -> (usize, usize, usize) {
    match row {
        0 | 1 => (12, 98, 6),
        2 => (32, 194, 12),
        _ => (33, 194, 12),
    }
}

/// The bytes of a cleared 303 step: note 0, no flags, rest on (`Step303::default`).
const CLEAR_303_STEP: [u8; 6] = [0, 0, 0, 0, 0, 1];

/// Whether `pattern`, a whole pattern of row `row`, has no step set: every 303 step is cleared,
/// every drum step byte is 0. The length and the shuffle flag do not count.
pub fn is_empty_pattern(row: usize, pattern: &[u8]) -> bool {
    let (_, _, slen) = layout(row);
    pattern[2..].chunks(slen).all(|s| {
        if row < 2 {
            s == CLEAR_303_STEP
        } else {
            s.iter().all(|&b| b == 0)
        }
    })
}

/// `pattern` with its steps emptied ([`is_empty_pattern`]); the length and the shuffle flag stay.
pub fn emptied_pattern(row: usize, pattern: &[u8]) -> Vec<u8> {
    let (_, _, slen) = layout(row);
    let mut p = pattern.to_vec();
    for s in p[2..].chunks_mut(slen) {
        if row < 2 {
            s.copy_from_slice(&CLEAR_303_STEP);
        } else {
            s.fill(0);
        }
    }
    p
}

/// The first empty pattern of row `row` after `from` (0..=31) in `patterns`, the row's 32
/// patterns: on through pattern 31, then from 0. `None` when no other pattern is empty.
pub fn next_empty_pattern(row: usize, patterns: &[u8], from: u8) -> Option<u8> {
    let (_, plen, _) = layout(row);
    (1..32)
        .map(|d| (from + d) % 32)
        .find(|&p| is_empty_pattern(row, &patterns[plen * p as usize..][..plen]))
}

/// The index of a control's device and controller: the knobs, faders and switches of the 303s,
/// the 808, the 909 and the effects, and the pattern sections' on/off buttons (controller 0).
/// The controller numbers are those of `docs/song_load.md` section 5.
pub fn controls() -> HashMap<String, (Device, u8)> {
    let mut m = HashMap::new();
    let mut add = |id: String, d: Device, c: u8| {
        m.insert(id, (d, c));
    };
    for (k, d) in [(1, Device::Tb303_1), (2, Device::Tb303_2)] {
        for (n, c) in [
            ("tune", 2),
            ("cutoff", 3),
            ("reso", 4),
            ("envmod", 5),
            ("decay", 6),
            ("accent", 7),
            ("wave", 8),
        ] {
            add(format!("tb303_{k}.{n}"), d, c);
        }
    }
    for (n, c) in [
        ("ac_level", 2),
        ("bd_level", 3),
        ("bd_tone", 4),
        ("bd_decay", 5),
        ("sd_level", 6),
        ("sd_tone", 7),
        ("sd_snappy", 8),
        ("lt_level", 9),
        ("lt_tuning", 10),
        ("slot3_switch", 11),
        ("mt_level", 12),
        ("mt_tuning", 13),
        ("slot4_switch", 14),
        ("ht_level", 15),
        ("ht_tuning", 16),
        ("slot5_switch", 17),
        ("rs_level", 18),
        ("slot6_switch", 19),
        ("cp_level", 20),
        ("slot7_switch", 21),
        ("cb_level", 22),
        ("cy_level", 23),
        ("cy_tone", 24),
        ("cy_decay", 25),
        ("oh_level", 26),
        ("oh_decay", 27),
        ("ch_level", 28),
    ] {
        add(format!("tr808.{n}"), Device::Tr808, c);
    }
    for (n, c) in [
        ("ac_level", 2),
        ("bd_level", 3),
        ("bd_tune", 4),
        ("bd_attack", 5),
        ("bd_decay", 6),
        ("sd_level", 7),
        ("sd_tune", 8),
        ("sd_tone", 9),
        ("sd_snappy", 10),
        ("lt_level", 11),
        ("lt_tune", 12),
        ("lt_decay", 13),
        ("mt_level", 14),
        ("mt_tune", 15),
        ("mt_decay", 16),
        ("ht_level", 17),
        ("ht_tune", 18),
        ("ht_decay", 19),
        ("rs_level", 20),
        ("cp_level", 21),
        ("hh_level", 22),
        ("ch_decay", 23),
        ("oh_decay", 24),
        ("cc_level", 25),
        ("cc_tune", 26),
        ("rc_level", 27),
        ("rc_tune", 28),
        ("flam", 29),
    ] {
        add(format!("tr909.{n}"), Device::Tr909, c);
    }
    for (id, d, c) in [
        ("delay.on", Device::Delay, 0),
        ("delay.steps", Device::Delay, 1),
        ("delay.triplet", Device::Delay, 2),
        ("delay.feedback", Device::Delay, 3),
        ("delay.pan", Device::Delay, 4),
        ("dist.on", Device::Distortion, 0),
        ("dist.amount", Device::Distortion, 1),
        ("dist.shape", Device::Distortion, 2),
        ("comp.on", Device::Compressor, 0),
        ("comp.ratio", Device::Compressor, 1),
        ("comp.threshold", Device::Compressor, 2),
        ("pcf.on", Device::Pcf, 0),
        ("pcf.freq", Device::Pcf, 1),
        ("pcf.q", Device::Pcf, 2),
        ("pcf.amt", Device::Pcf, 3),
        ("pcf.pattern", Device::Pcf, 4),
        ("pcf.dec", Device::Pcf, 5),
        ("pcf.mode", Device::Pcf, 6),
    ] {
        add(id.to_string(), d, c);
    }
    for (r, d) in ROW_DEVICES.into_iter().enumerate() {
        add(format!("pattern{}.on", r + 1), d, 0);
    }
    m
}

/// The mixer message a control id sends: `Global(id)` is msg 145 `{id, v}`, `Channel(c, k)` is
/// msg 147 `{c, k, v}`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MixerTarget {
    Global(u8),
    Channel(u8, u8),
}

pub fn mixer_target(id: &str) -> Option<MixerTarget> {
    match id {
        "master.fader" => return Some(MixerTarget::Global(0)),
        "mixer.comp_id" => return Some(MixerTarget::Global(1)),
        "mixer.pcf_id" => return Some(MixerTarget::Global(2)),
        _ => {}
    }
    let (sec, name) = id.split_once('.')?;
    let c = sec
        .strip_prefix("mixer")?
        .parse::<u8>()
        .ok()?
        .checked_sub(1)?;
    let k = match name {
        "mute" => 0,
        "level" => 1,
        "pan" => 2,
        "delay" => 3,
        "dist" => 4,
        _ => return None,
    };
    (c < 4).then_some(MixerTarget::Channel(c, k))
}

/// The value of every knob of the panel in `engine`, a new empty song as the song constructor
/// made it, by layout id: the values [`Link::follow_values`] gives the panel for such a song
/// (the devices' live blocks, the mixer's pan and delay send, the shuffle). The values are the
/// device constructors' and do not depend on the kit; a drum machine built without samples is
/// a placeholder with no live block, so its knobs are left out.
fn new_song_knobs(
    engine: &Engine,
    controls: &HashMap<String, (Device, u8)>,
) -> HashMap<String, u8> {
    let f = &engine.frame;
    let d = &f.devices;
    let live = |dev: Device, c: u8| {
        let (block, i) = match dev {
            Device::Mixer => return None,
            Device::Tr909 => (d.channels[3].live(), tr909::internal_index(c as usize)),
            Device::Tb303_1 | Device::Tb303_2 | Device::Tr808 => {
                (d.channels[dev as usize - 1].live(), c as usize)
            }
            Device::Delay => (d.delay.live(), c as usize),
            Device::Distortion => (d.dist.live(), c as usize),
            Device::Pcf => (d.pcf.live(), c as usize),
            Device::Compressor => (d.comp.live(), c as usize),
        };
        block.get(i).copied()
    };
    let mut knobs = HashMap::new();
    for c in crate::layout::controls() {
        if c.kind != "knob" {
            continue;
        }
        let v = match mixer_target(&c.id) {
            Some(MixerTarget::Channel(ch, 2)) => Some(d.mixer.live.channels[ch as usize].pan),
            Some(MixerTarget::Channel(ch, 3)) => Some(d.mixer.live.channels[ch as usize].send),
            Some(_) => None,
            None if c.id == "transport.shuffle" => Some(f.seq.globals.shuffle),
            None => controls.get(&c.id).and_then(|&(dev, ctrl)| live(dev, ctrl)),
        };
        if let Some(v) = v {
            knobs.insert(c.id, v);
        }
    }
    knobs
}

/// The panel's player link.
pub struct Link {
    player: Box<dyn Port>,
    controls: HashMap<String, (Device, u8)>,
    /// The value each knob has in a new empty song ([`Link::knob_default`]).
    knob_defaults: HashMap<String, u8>,
    /// The four pattern arrays (32 patterns each), as the engine holds them.
    patterns: [Vec<u8>; 4],
    /// The pattern each section edits: the selected one.
    row_pattern: [u8; 4],
    /// The pattern whose steps the model shows, per section.
    shown: [Option<u8>; 4],
    backlog: VecDeque<Command>,
    /// The engine's frame count when the last commands were queued.
    mark: u64,
    /// The `mark` whose engine values the panel took last ([`Link::follow`]).
    taken: u64,
    /// The panel took engine values this turn: the window draws the panel before it takes them,
    /// so one more frame must show them ([`Link::moving`]).
    fresh: bool,
    /// The messages sent so far, when turned on (`Some`).
    pub log: Option<Vec<Msg>>,
    /// The drum samples of the newest song (its mod's), whose startup sound
    /// [`Link::startup_sound`] plays.
    kit: Option<Arc<SampleSet>>,
    /// The generation and revision last published after recording changed the open song.
    recorded_at: Option<(u32, u64)>,
    /// The first player generation that can be the open song: a copy of an older song's tracks
    /// is not the open song's.
    tracks_from: u32,
    /// The open song's latest tracks, initially from its built engine, then from the callback.
    loaded: [Vec<TrackEvent>; 9],
    /// The generation and revision last copied into `loaded`.
    loaded_at: Option<(u32, u64)>,
    /// The open song's saved blocks while the engine plays it in song mode
    /// ([`Link::song_mode_blocks`]).
    saved: Option<SavedBlocks>,
    /// The number of lit gain-reduction LEDs [`Link::moving`] saw last.
    shown_gr: Option<usize>,
    /// A pattern written into the mirror whose message waits until the engine no longer plays
    /// it ([`Link::set_pattern_unplayed`]).
    deferred: Option<Deferred>,
}

/// Pattern `pattern` of row `row` was written by [`Link::set_pattern_unplayed`] when the engine
/// had run up to frame `after`: its message goes from a snapshot past `after` that does not
/// play it.
#[derive(Clone, Copy, Debug)]
struct Deferred {
    row: usize,
    pattern: u8,
    after: u64,
}

/// The device parameters the original's save writes for a song in song mode, as the engine
/// last published them (task-159): each device's pattern-mode copy, and the mixer's with its
/// live master and channel enables ([`orb_player::Snapshot::saved`],
/// [`orb_player::Snapshot::saved_mixer`]; `docs/song_load.md` section 4.10). The live values
/// the panel shows then are the tracks' and the panel's, which the save does not write.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SavedBlocks {
    /// By device id, indexed as the device's live block (index 0, the mixer, is empty).
    pub blocks: [LiveBlock; 9],
    pub mixer: MixerParams,
}

impl SavedBlocks {
    /// Controller `c` of device `d` (not the mixer), by controller number.
    pub fn param(&self, d: Device, c: u8) -> Option<u8> {
        let i = if d == Device::Tr909 {
            tr909::internal_index(c as usize)
        } else {
            c as usize
        };
        self.blocks[d as usize].as_slice().get(i).copied()
    }
}

impl Link {
    /// A link to a new player with the drum samples `samples` and the audio half to run in the
    /// callback.
    pub fn new(tables: Arc<Tables>, samples: Option<Arc<SampleSet>>) -> (Link, Audio) {
        let (player, mut audio) = Player::new(tables, samples.clone());
        let link = Self::of(Box::new(player), &mut audio, samples);
        (link, audio)
    }

    /// A link to `port` (the plugin's editor, task-201), with the panel's views of a new empty
    /// song until [`Link::reset_view`] gives it the open one. `tables` and `samples` build that
    /// song, on a player of its own, for the knobs' new-song values.
    pub fn with_port(
        port: Box<dyn Port>,
        tables: Arc<Tables>,
        samples: Option<Arc<SampleSet>>,
    ) -> Link {
        let (_, mut audio) = Player::new(tables, samples.clone());
        Self::of(port, &mut audio, samples)
    }

    /// A link to `player`, whose new empty song `audio` holds.
    fn of(player: Box<dyn Port>, audio: &mut Audio, samples: Option<Arc<SampleSet>>) -> Link {
        let kit = samples;
        // the new empty song's patterns, as its constructor made them
        let patterns = std::array::from_fn(|c| {
            audio.current_mut().frame.devices.channels[c]
                .patterns_mut()
                .map(|p| p.to_vec())
                .unwrap_or_default()
        });
        let loaded = std::array::from_fn(|t| audio.current_mut().frame.seq.events(t).to_vec());
        let controls = controls();
        let knob_defaults = new_song_knobs(audio.current_mut(), &controls);
        Link {
            player,
            controls,
            knob_defaults,
            patterns,
            row_pattern: [0; 4],
            shown: [None; 4],
            backlog: VecDeque::new(),
            mark: 0,
            taken: u64::MAX,
            fresh: false,
            log: None,
            kit,
            recorded_at: None,
            tracks_from: 0,
            loaded,
            loaded_at: None,
            saved: None,
            shown_gr: None,
            deferred: None,
        }
    }

    /// Shows a song that the port has loaded by itself (the plugin's editor, task-201: opened,
    /// after File > Open or New, after the host restored a state): the panel is made new and
    /// takes `patterns` (the four pattern arrays) and the engine's values of `snapshot`, a
    /// snapshot of that song; the drum samples are `samples`. The panel's pending events are
    /// sent first. The engine's values are then taken again from the port's snapshots only
    /// after [`Port::follow_frames`], when they show the song.
    pub fn reset_view(
        &mut self,
        patterns: [Vec<u8>; 4],
        snapshot: &Snapshot,
        samples: Option<Arc<SampleSet>>,
        panel: &mut Panel,
    ) {
        self.forward(panel);
        self.kit = samples;
        self.patterns = patterns;
        self.shown = [None; 4];
        self.deferred = None;
        self.saved = None;
        self.recorded_at = None;
        *panel = Panel::new();
        let live = |d: Device, i: usize| snapshot.live[d as usize].as_slice().get(i).copied();
        self.follow_values(panel, snapshot, &live);
        self.fill_views(panel);
        self.flush_marked();
    }

    /// The value (0..=127) the knob `id` has in a new empty song, which a double click sets.
    /// `None` for an id that is not a knob, and for a drum machine's knob on a link made
    /// without samples.
    pub fn knob_default(&self, id: &str) -> Option<u8> {
        self.knob_defaults.get(id).copied()
    }

    /// The open song's tracks when recording has changed them since it was opened (the merge
    /// of the loop wrap, record off or the song end), else `None`: Save writes these, as the
    /// original's save writes the tracks it reads back from the DLL (`sub_437CFF`).
    pub fn recorded_tracks(&self) -> Option<&[Vec<TrackEvent>; 9]> {
        self.recorded_at.map(|_| &self.loaded)
    }

    /// The open song's latest known engine tracks, including constructor events and any
    /// published merges or stop prunes. Before its load runs, these are the built song's tracks.
    pub fn loaded_tracks(&self) -> &[Vec<TrackEvent>; 9] {
        &self.loaded
    }

    /// The device parameters Save writes when the engine plays the open song in song mode:
    /// `Some` from the first snapshot the panel follows that is in song mode, `None` in pattern
    /// mode (Save writes the panel's values, the live ones, as the original's queries do then)
    /// and after a load until the panel has followed the new song (the panel then holds the
    /// song's own blocks). The mode is the engine's, not the panel's: a mode switch the engine
    /// has not run yet has not moved the blocks either.
    pub fn song_mode_blocks(&self) -> Option<&SavedBlocks> {
        self.saved.as_ref()
    }

    /// Whether what the panel shows of the engine can still change by itself, after
    /// [`Link::sync`] (task-153, the repaint rule of `docs/ui_spec.md` section 16): the song
    /// plays, a meter is lit, the gain-reduction LEDs changed, the snapshot does
    /// not show the last commands sent yet, or the panel took the engine's first values since
    /// a send this turn (one more frame draws them, task-186). While this holds the window repaints
    /// at the meter rate; otherwise only input (or an animation of its own) repaints it. A
    /// loaded song that waits for Play is not followed (module doc), so it moves nothing.
    pub fn moving(&mut self, panel: &Panel) -> bool {
        let mark = self.mark;
        let s = self.player.snapshot();
        let (playing, frames) = (s.playing, s.frames);
        // the compressor's reduction holds its last value in silence: it moves only when it
        // changes
        let gr = crate::model::gr_lit(panel.gr_byte);
        let gr_moved = self.shown_gr.replace(gr) != Some(gr);
        let fresh = std::mem::take(&mut self.fresh);
        playing
            || fresh
            || frames <= mark
            || !self.backlog.is_empty()
            || self.deferred.is_some()
            || panel.meters.values().any(|&m| m > 0)
            || gr_moved
    }

    /// Which copy [`Link::recorded_tracks`] is (the player's generation and track revision): it
    /// changes when recording changes the open song's tracks.
    pub fn recorded_revision(&self) -> Option<(u32, u64)> {
        self.recorded_at
    }

    /// The engine's values are taken only from a snapshot with more frames than this: the one
    /// after the first call that can have drained the commands sent last.
    pub fn adopts_after(&self) -> u64 {
        self.mark
    }

    pub fn player(&mut self) -> &mut dyn Port {
        self.player.as_mut()
    }

    /// The pattern arrays as the link knows them (the song's, with the panel's edits).
    pub fn patterns(&self) -> &[Vec<u8>; 4] {
        &self.patterns
    }

    /// Pattern `p` (0..=31) of row `row`, whole (98 or 194 bytes), as [`Link::patterns`] holds
    /// it: the panel's events count once they are forwarded ([`Link::forward`]).
    pub fn pattern(&self, row: usize, p: u8) -> &[u8] {
        let (_, plen, _) = layout(row);
        &self.patterns[row][plen * p as usize..][..plen]
    }

    /// Writes `bytes`, a whole pattern of row `row`, into its pattern `p` (0..=31): the panel's
    /// pending events go first, then msg 41 sub 44, 61 or 78 `{p, bytes}`, and the panel shows
    /// the new pattern. False, and no pattern message, when the pattern holds `bytes` already.
    pub fn set_pattern(&mut self, row: usize, p: u8, bytes: &[u8], panel: &mut Panel) -> bool {
        if !self.write_mirror(row, p, bytes, panel) {
            return false;
        }
        // this message carries the pattern whole: a deferred one of it has nothing to add
        if self
            .deferred
            .is_some_and(|d| (d.row, d.pattern) == (row, p))
        {
            self.deferred = None;
        }
        self.push_pattern(row, p);
        self.flush_marked();
        true
    }

    /// [`Link::set_pattern`] of a pattern the engine may be playing but is about to leave (the
    /// Undo of a pattern the row no longer selects, `docs/ui_spec.md` section 18): the mirror
    /// and the panel take `bytes` at once, the engine only from the first snapshot that ran the
    /// commands sent so far and is stopped or plays another pattern of the row
    /// ([`Link::follow`]), so it never plays a half-switched pattern. A selection of the pattern
    /// or an edit of it sends it at once, before the event. One pattern waits at a time: an
    /// earlier one is sent first.
    pub fn set_pattern_unplayed(
        &mut self,
        row: usize,
        p: u8,
        bytes: &[u8],
        panel: &mut Panel,
    ) -> bool {
        self.send_deferred();
        if !self.write_mirror(row, p, bytes, panel) {
            self.flush_marked();
            return false;
        }
        self.deferred = Some(Deferred {
            row,
            pattern: p,
            after: self.mark,
        });
        true
    }

    /// The panel's pending events go first, then `bytes` into the mirror's pattern `p` of row
    /// `row`, and the panel shows it; false when it held `bytes` already.
    fn write_mirror(&mut self, row: usize, p: u8, bytes: &[u8], panel: &mut Panel) -> bool {
        self.forward(panel);
        if self.pattern(row, p) == bytes {
            return false;
        }
        let (_, plen, _) = layout(row);
        self.patterns[row][plen * p as usize..][..plen].copy_from_slice(bytes);
        // the row's lengths, shuffle flags and steps are filled again
        self.shown[row] = None;
        self.fill_views(panel);
        true
    }

    /// Queues msg 41 sub 44, 61 or 78 `{p, bytes}` with the mirror's pattern `p` of row `row`.
    fn push_pattern(&mut self, row: usize, p: u8) {
        let d = ROW_DEVICES[row];
        let mut data = vec![p];
        data.extend(self.pattern(row, p));
        self.push(Msg::Device {
            dev: d.id(),
            sub: header_sub(d) + 2,
            data,
        });
    }

    /// Queues the message of the pattern [`Link::set_pattern_unplayed`] holds back, if any
    /// (not flushed).
    fn send_deferred(&mut self) {
        if let Some(d) = self.deferred.take() {
            self.push_pattern(d.row, d.pattern);
        }
    }

    /// Whether a pattern's message waits for the engine to leave it (for tests).
    pub fn has_deferred_pattern(&self) -> bool {
        self.deferred.is_some()
    }

    // ---- loading ------------------------------------------------------------------------

    /// Opens `song`: builds it and queues it as the newest song, and resets the panel to its
    /// values (the data blocks as the pattern mode shows them). A song that is playing is
    /// stopped first: the new song is current at once, stopped, and starts when Play is
    /// pressed. The song's own drum samples are `samples`. A song the node pool refuses
    /// ([`LoadRefused`]) is not opened: the current song, the panel and the kit stay as they
    /// were.
    pub fn load(
        &mut self,
        song: &Song,
        samples: Option<Arc<SampleSet>>,
        panel: &mut Panel,
    ) -> Result<(), LoadRefused> {
        let engine = self.player.build(song, samples.clone())?;
        self.queue_song(engine, samples, panel);
        let g = &song.globals;
        *panel = Panel::new();
        let t = &mut panel.transport;
        t.tempo = g.tempo as i32;
        t.song_mode = g.song_mode != 0;
        t.looping = g.loop_on != 0;
        t.loop_start = g.loop_start as i32;
        t.loop_end = g.loop_end as i32;
        panel.adopt("transport.shuffle", g.shuffle as i32);
        panel.vintage = g.is_vintage();
        let payloads: Vec<Vec<u8>> = Device::ALL
            .iter()
            .map(|&d| song.device_payload(d))
            .collect();
        for (r, d) in ROW_DEVICES.into_iter().enumerate() {
            let payload = &payloads[d as usize];
            let (base, plen, _) = layout(r);
            self.patterns[r] = payload[base..base + 32 * plen].to_vec();
            // a song file may hold a selection past 31; the arrays have 32 patterns
            let sel = payload[1].min(31);
            self.row_pattern[r] = sel;
            panel.rows[r].pattern = sel;
            panel.rows[r].bank = sel / 8;
        }
        self.shown = [None; 4];
        self.deferred = None;
        for (id, &(d, c)) in &self.controls {
            panel.adopt(id, payloads[d as usize][c as usize] as i32);
        }
        let m = &payloads[Device::Mixer as usize];
        panel.adopt("master.fader", m[0] as i32);
        panel.adopt("mixer.comp_id", m[1] as i32);
        panel.adopt("mixer.pcf_id", m[2] as i32);
        for c in 0..4 {
            for (k, name) in ["mute", "level", "pan", "delay", "dist"]
                .into_iter()
                .enumerate()
            {
                panel.adopt(&format!("mixer{}.{name}", c + 1), m[5 + 6 * c + k] as i32);
            }
        }
        self.fill_views(panel);
        self.flush_marked();
        Ok(())
    }

    /// File > New: a new song, the song constructor's empty song with the template's four
    /// patterns in A1 ([`crate::template`], task-288), queued as [`Link::load`] queues a song: the
    /// open song stops, the panel is new and takes the engine's values at the next snapshot.
    pub fn load_new(&mut self, samples: Option<Arc<SampleSet>>, panel: &mut Panel) {
        let mut engine = self.player.build_new(samples.clone());
        crate::template::apply(&mut engine);
        let patterns = std::array::from_fn(|c| {
            engine.frame.devices.channels[c]
                .patterns_mut()
                .map(|p| p.to_vec())
                .unwrap_or_default()
        });
        // the old song's pending panel events are forwarded first, into the old mirror
        self.queue_song(engine, samples, panel);
        self.patterns = patterns;
        self.row_pattern = [0; 4];
        self.shown = [None; 4];
        self.deferred = None;
        *panel = Panel::new();
        self.fill_views(panel);
        self.flush_marked();
    }

    /// Queues `engine` as the newest song after a Stop, and forgets the old song's recording.
    fn queue_song(
        &mut self,
        engine: Box<Engine>,
        samples: Option<Arc<SampleSet>>,
        panel: &mut Panel,
    ) {
        // what the panel did before is sent first (the panel is replaced after)
        self.forward(panel);
        self.kit = samples;
        self.loaded = std::array::from_fn(|t| engine.frame.seq.events(t).to_vec());
        self.loaded_at = None;
        // the open song stops first (task-090): the new song then becomes current at once, at
        // its start, stopped, instead of waiting as the pending song behind a playing one
        self.backlog.push_back(Command::Stop);
        self.backlog.push_back(Command::Load(engine));
        self.recorded_at = None;
        self.saved = None;
        // Every Load queued here becomes current at once (the Stop before it), so each raises
        // the player's generation by exactly one; a pending song would break `tracks_from`.
        debug_assert!(
            !self.player.snapshot().pending,
            "a load waits behind a playing song"
        );
        self.tracks_from = self.tracks_from.max(self.player.snapshot().generation) + 1;
    }

    /// The Mods menu's mod change of the open song (song msg 19): the pending panel events go
    /// first, then the drum machines of the newest song take `samples`
    /// ([`Player::set_mod`], docs/rbm.md section 14).
    pub fn set_mod(&mut self, samples: Arc<SampleSet>, panel: &mut Panel) {
        self.forward(panel);
        self.kit = Some(samples.clone());
        if let Err(c) = self.player.set_mod(samples) {
            // a full queue: the change waits in the backlog, behind what is queued
            self.backlog.push_back(c);
        }
        self.flush_marked();
    }

    /// Top-level msg 58 with the newest song's mod: the preview player plays its startup
    /// sound (slot 46, [`Command::Preview`]; docs/export_state.md section 8). Nothing without
    /// drum samples.
    pub fn startup_sound(&mut self) {
        let Some(kit) = &self.kit else {
            return;
        };
        // behind what waits in the backlog
        let sample = kit.shared(STARTUP_ID);
        self.backlog
            .push_back(Command::Preview(Box::new(Preview::new(sample))));
        self.flush();
    }

    /// Fills the drum step strips of the pattern each one now edits, at once: `App` calls it
    /// when a drum grid's pin changed (task-191, screen reader mode), so the frame's input reads
    /// the steps of the pattern it edits, not those shown before ([`Link::fill_views`]).
    pub fn show_patterns(&mut self, panel: &mut Panel) {
        self.fill_views(panel);
    }

    /// Fills the model's pattern views from the pattern arrays: the 303s' steps, the lengths and
    /// shuffle flags, and the drum step strips of a section whose shown pattern changed (the
    /// row's, or a drum grid's pin, [`Panel::edit_pattern`]).
    fn fill_views(&mut self, panel: &mut Panel) {
        for r in 0..4 {
            let (_, plen, slen) = layout(r);
            let pat = &self.patterns[r];
            if self.shown[r].is_none() {
                for p in 0..32 {
                    let h = &pat[p * plen..];
                    panel.rows[r].shuffle[p] = h[0] != 0;
                    panel.rows[r].len[p] = h[1];
                    if r < 2 {
                        for s in 0..16 {
                            let b = &h[2 + slen * s..2 + slen * s + 6];
                            panel.synth[r].patterns[p][s] = Step303 {
                                note: b[0],
                                slide: b[1] != 0,
                                accent: b[2] != 0,
                                up: b[3] != 0,
                                down: b[4] != 0,
                                rest: b[5] != 0,
                            };
                        }
                    }
                }
            }
            // a drum strip shows the pattern it edits: the row's, or in screen reader mode the
            // drum grid's pin (task-191)
            let sel = if r >= 2 {
                panel.edit_pattern(r)
            } else {
                panel.rows[r].pattern
            };
            if self.shown[r] != Some(sel) {
                self.shown[r] = Some(sel);
                self.row_pattern[r] = panel.rows[r].pattern;
                let h = &pat[sel as usize * plen..];
                // only the drum rows have 12-byte steps; a 303 pattern is 98 bytes and the last
                // pattern (31) would run past the array
                if r >= 2 {
                    for s in 0..16 {
                        for i in 0..12 {
                            let b = h[2 + slen * s + i];
                            if r == 2 {
                                panel.steps808[i][s] = b != 0;
                            } else {
                                panel.steps909[i][s] = b;
                            }
                        }
                    }
                }
            }
        }
    }

    // ---- out ----------------------------------------------------------------------------

    /// The message(s) the original sends for a model event; updates the pattern arrays for the
    /// edits.
    pub fn messages(&mut self, e: &Event) -> Vec<Msg> {
        let dev = |d: Device, sub: u8, data: &[u8]| Msg::Device {
            dev: d.id(),
            sub,
            data: data.to_vec(),
        };
        match e {
            Event::Press { id } => match id.as_str() {
                "transport.play" => vec![Msg::Global { id: 1, value: 1 }],
                "transport.stop" => vec![Msg::Global { id: 1, value: 0 }],
                _ => vec![],
            },
            Event::Pattern { row, pattern } => {
                self.row_pattern[*row] = *pattern;
                let d = ROW_DEVICES[*row];
                vec![dev(d, param_sub(d).unwrap(), &[1, *pattern])]
            }
            Event::Set { id, value } => self.set_messages(id, *value),
            Event::Step808 {
                instrument,
                pattern,
                step,
                on,
            } => self.drum_step(2, *pattern, *step, *instrument, *on as u8),
            Event::Step909 {
                instrument,
                pattern,
                step,
                value,
            } => self.drum_step(3, *pattern, *step, *instrument, *value),
            Event::Step303 {
                synth,
                pattern,
                step,
                field,
                value,
            } => {
                let (_, plen, slen) = layout(*synth);
                self.patterns[*synth]
                    [plen * *pattern as usize + 2 + slen * step + *field as usize] = *value;
                vec![dev(
                    ROW_DEVICES[*synth],
                    header_sub(ROW_DEVICES[*synth]) + 3,
                    &[*pattern, *step as u8, *field, *value],
                )]
            }
            Event::Clear303 { synth, pattern } => {
                let (_, plen, _) = layout(*synth);
                let at = plen * *pattern as usize;
                let bytes = emptied_pattern(*synth, &self.patterns[*synth][at..at + plen]);
                self.patterns[*synth][at..at + plen].copy_from_slice(&bytes);
                let mut data = vec![*pattern];
                data.extend(bytes);
                vec![dev(
                    ROW_DEVICES[*synth],
                    header_sub(ROW_DEVICES[*synth]) + 2,
                    &data,
                )]
            }
        }
    }

    /// A drum step of pattern `p` of `row` (2 the 808, 3 the 909). The pattern is the event's:
    /// with screen reader mode off the row's, as `row_pattern` (task-191).
    fn drum_step(&mut self, row: usize, p: u8, step: usize, inst: usize, value: u8) -> Vec<Msg> {
        let (_, plen, slen) = layout(row);
        self.patterns[row][plen * p as usize + 2 + slen * step + inst] = value;
        let d = ROW_DEVICES[row];
        vec![Msg::Device {
            dev: d.id(),
            sub: header_sub(d) + 3,
            data: vec![p, step as u8, inst as u8, value],
        }]
    }

    fn set_messages(&mut self, id: &str, value: i32) -> Vec<Msg> {
        let global = |id: u8| {
            vec![Msg::Global {
                id,
                value: value as u32,
            }]
        };
        match id {
            "transport.mode" => return global(0),
            "transport.record" => return global(2),
            "transport.loop" => return global(3),
            "transport.tempo" => return global(4),
            "transport.position" | "transport.bar" => return global(8),
            "transport.loop_start" => return global(12),
            "transport.loop_end" => return global(16),
            "transport.shuffle" => return global(20),
            "song.vintage" => return global(21),
            _ => {}
        }
        if let Some(t) = mixer_target(id) {
            let data = match t {
                MixerTarget::Global(i) => vec![i, value as u8],
                MixerTarget::Channel(c, k) => vec![c, k, value as u8],
            };
            let sub = if matches!(t, MixerTarget::Global(_)) {
                145
            } else {
                147
            };
            return vec![Msg::Device {
                dev: Device::Mixer.id(),
                sub,
                data,
            }];
        }
        // the pattern section's Shuffle and Steps: the pattern header `{p, field, v}`
        if let Some((sec, name)) = id.split_once('.')
            && let Some(r) = sec
                .strip_prefix("pattern")
                .and_then(|n| n.parse::<usize>().ok())
                .and_then(|n| n.checked_sub(1))
                .filter(|&r| r < 4)
            && let Some(field) = match name {
                "shuffle" => Some(0u8),
                "steps" => Some(1),
                _ => None,
            }
        {
            let p = self.row_pattern[r];
            let (_, plen, _) = layout(r);
            self.patterns[r][plen * p as usize + field as usize] = value as u8;
            let d = ROW_DEVICES[r];
            return vec![Msg::Device {
                dev: d.id(),
                sub: header_sub(d),
                data: vec![p, field, value as u8],
            }];
        }
        if let Some(&(d, c)) = self.controls.get(id) {
            return vec![Msg::Device {
                dev: d.id(),
                sub: param_sub(d).unwrap(),
                data: vec![c, value as u8],
            }];
        }
        vec![]
    }

    /// The player's command for a message.
    pub fn command(m: &Msg) -> Option<Command> {
        match m {
            Msg::Global { id, value } => Some(match id {
                0 => Command::SongMode(*value != 0),
                1 if *value != 0 => Command::Play,
                1 => Command::Stop,
                2 => Command::Record(*value != 0),
                3 => Command::LoopOn(*value != 0),
                4 => Command::Tempo(*value),
                8 => Command::Locate(*value),
                12 => Command::LoopStart(*value),
                16 => Command::LoopEnd(*value),
                20 => Command::Shuffle(*value as u8),
                21 => Command::Vintage(*value != 0),
                _ => return None,
            }),
            Msg::Device { dev, sub, data } => {
                let d = Device::from_id(*dev)?;
                let b = |i: usize| data.get(i).copied();
                if Some(*sub) == param_sub(d) {
                    return Some(Command::DeviceParam {
                        device: d,
                        ctrl: b(0)?,
                        value: b(1)?,
                    });
                }
                match (d, *sub) {
                    (Device::Mixer, 145) => Some(Command::MixerGlobal {
                        id: b(0)?,
                        value: b(1)?,
                    }),
                    (Device::Mixer, 147) => Some(Command::MixerChannel {
                        channel: b(0)?,
                        field: b(1)?,
                        value: b(2)?,
                    }),
                    (d, s) if d.id() >= 1 && d.id() <= 4 && s == header_sub(d) => {
                        Some(Command::PatternField {
                            device: d,
                            pattern: b(0)?,
                            field: b(1)?,
                            value: b(2)?,
                        })
                    }
                    (d, s) if d.id() >= 1 && d.id() <= 4 && s == header_sub(d) + 2 => {
                        Some(Command::Pattern {
                            device: d,
                            pattern: b(0)?,
                            bytes: data[1..].into(),
                        })
                    }
                    (d, s) if d.id() >= 1 && d.id() <= 4 && s == header_sub(d) + 3 => {
                        Some(Command::StepField {
                            device: d,
                            pattern: b(0)?,
                            step: b(1)?,
                            field: b(2)?,
                            value: b(3)?,
                        })
                    }
                    _ => None,
                }
            }
        }
    }

    fn flush(&mut self) {
        while let Some(c) = self.backlog.pop_front() {
            if let Err(c) = self.player.send(c) {
                self.backlog.push_front(c);
                break;
            }
        }
    }

    /// Sends the panel's pending events to the player.
    pub fn forward(&mut self, panel: &mut Panel) {
        let events = std::mem::take(&mut panel.events);
        if events.is_empty() && self.backlog.is_empty() {
            return;
        }
        for e in &events {
            // a pattern held back goes before a selection or an edit of it
            if let Some(d) = self.deferred {
                let (r, p) = (d.row, d.pattern);
                let touches = match *e {
                    Event::Pattern { row, pattern } => (row, pattern) == (r, p),
                    Event::Step303 { synth, pattern, .. } | Event::Clear303 { synth, pattern } => {
                        (synth, pattern) == (r, p)
                    }
                    Event::Step808 { .. } => r == 2 && self.row_pattern[2] == p,
                    Event::Step909 { .. } => r == 3 && self.row_pattern[3] == p,
                    _ => false,
                };
                if touches {
                    self.send_deferred();
                }
            }
            for m in self.messages(e) {
                self.push(m);
            }
        }
        self.flush_marked();
    }

    /// Queues the player's command for `m` behind the backlog and logs `m`.
    fn push(&mut self, m: Msg) {
        if let Some(c) = Self::command(&m) {
            self.backlog.push_back(c);
        }
        if let Some(log) = &mut self.log {
            log.push(m);
        }
    }

    /// Sends player commands that do not come from the panel, such as MIDI remote control's
    /// (`orb_player::midi`, task-182), the way the panel's own go: the panel takes the engine's
    /// values only from a snapshot that can show them, and then shows them as it shows a mouse
    /// change ([`Link::follow`]).
    pub fn send(&mut self, commands: impl IntoIterator<Item = Command>) {
        self.backlog.extend(commands);
        self.flush_marked();
    }

    /// Flushes the backlog and sets `mark` to the first snapshot that can show the commands:
    /// the call that has already started when they are pushed does not drain them, so the
    /// snapshot after the next call is the first (`frames` of the snapshot read before the push,
    /// plus one call).
    fn flush_marked(&mut self) {
        let before = self.player.snapshot().frames;
        self.flush();
        self.mark = before + self.player.follow_frames();
    }

    // ---- in -----------------------------------------------------------------------------

    /// Refreshes the open song's tracks from the latest callback publication.
    /// A stop's prune changes `loaded` (what Save writes) but not `recorded_revision`, so it
    /// does not mark an unrecorded song modified: it never changes playback.
    pub fn refresh_tracks(&mut self) {
        if !self.player.records() {
            return;
        }
        let t = self.player.tracks();
        let at = Some((t.generation, t.revision));
        if t.generation >= self.tracks_from && at != self.loaded_at {
            self.loaded = std::array::from_fn(|i| t.events[i].to_vec());
            self.loaded_at = at;
            if t.merges > 0 {
                self.recorded_at = at;
            }
        }
    }

    /// Takes the engine's state into the panel.
    pub fn follow(&mut self, panel: &mut Panel) {
        let s = *self.player.snapshot();
        // a pattern held back goes once the engine ran the commands sent before it and does not
        // play it
        if let Some(d) = self.deferred
            && s.frames > d.after
            && (!s.playing || s.patterns[d.row] != Some(d.pattern))
        {
            self.send_deferred();
            self.flush_marked();
        }
        self.refresh_tracks();
        // the meters and the running lights follow every call; the meters keep the engine's
        // byte, which the modern skin (`model::meter_state`) and the classic skin
        // (`model::peak_meter_frame`, the original's scale) map each their way
        for c in 0..4 {
            panel
                .meters
                .insert(format!("mixer{}.meter", c + 1), s.channel_meters[c]);
        }
        panel
            .meters
            .insert("master.meter_l".into(), s.master_meters[0]);
        panel
            .meters
            .insert("master.meter_r".into(), s.master_meters[1]);
        let live = |d: Device, i: usize| s.live[d as usize].as_slice().get(i).copied();
        for (id, d, i) in [
            ("delay.meter", Device::Delay, 5),
            ("dist.meter", Device::Distortion, 3),
            ("pcf.meter", Device::Pcf, 7),
            ("comp.meter", Device::Compressor, 4),
        ] {
            panel.meters.insert(id.into(), live(d, i).unwrap_or(0));
        }
        // the compressor's gain byte, which the gain-reduction LEDs show (`model::gr_lit`)
        panel.gr_byte = live(Device::Compressor, 3).unwrap_or(0);
        // a section whose pattern gate is closed (the plugin's Gate Pattern Mode) lights no step
        let step = |d: Device, i: usize| {
            live(d, i)
                .filter(|&b| s.playing && b < 16 && !s.gate_closed[d as usize - 1])
                .map(|b| b as usize)
        };
        panel.playing_step303 = [step(Device::Tb303_1, 9), step(Device::Tb303_2, 9)];
        panel.playing_step808 = step(Device::Tr808, 29);
        panel.playing_step909 = step(Device::Tr909, 30);

        // the values only from a call that ran the commands sent, and not of a waiting song
        if s.frames > self.mark && !s.pending {
            self.follow_values(panel, &s, &live);
            // the first values after the commands sent: the frame has drawn the panel before
            // this, so it is drawn once more with them (a File > New or Open of a stopped song
            // would otherwise show the old values until the next input)
            if self.taken != self.mark {
                self.taken = self.mark;
                self.fresh = true;
            }
        }
        self.fill_views(panel);
    }

    fn follow_values(
        &mut self,
        panel: &mut Panel,
        s: &orb_player::Snapshot,
        live: &dyn Fn(Device, usize) -> Option<u8>,
    ) {
        let t = &mut panel.transport;
        t.position = s.tick;
        t.playing = s.playing;
        t.song_mode = s.song_mode;
        t.looping = s.loop_on;
        t.loop_start = s.loop_start as i32;
        t.loop_end = s.loop_end as i32;
        t.tempo = s.tempo as i32;
        // record: the engine's byte (the song end turns it off, 0x1001F48C)
        t.recording = s.recording && t.song_mode;
        panel.adopt("transport.shuffle", s.shuffle as i32);
        panel.vintage = s.vintage;
        self.saved = s.song_mode.then_some(SavedBlocks {
            blocks: s.saved,
            mixer: s.saved_mixer,
        });
        for (id, &(d, c)) in &self.controls {
            let i = if d == Device::Tr909 {
                tr909::internal_index(c as usize)
            } else {
                c as usize
            };
            if let Some(v) = live(d, i) {
                panel.adopt(id, v as i32);
            }
        }
        for (r, d) in ROW_DEVICES.into_iter().enumerate() {
            if let Some(p) = live(d, 1) {
                panel.adopt_pattern(r, p.min(31));
                self.row_pattern[r] = panel.rows[r].pattern;
            }
        }
        let m = &s.mixer;
        panel.adopt("master.fader", m.master as i32);
        panel.adopt("mixer.comp_id", m.comp_id as i32);
        panel.adopt("mixer.pcf_id", m.pcf_id as i32);
        for (c, ch) in m.channels.iter().enumerate() {
            let n = c + 1;
            panel.adopt(&format!("mixer{n}.mute"), ch.enabled as i32);
            panel.adopt(&format!("mixer{n}.level"), ch.level as i32);
            panel.adopt(&format!("mixer{n}.pan"), ch.pan as i32);
            panel.adopt(&format!("mixer{n}.delay"), ch.send as i32);
            panel.adopt(&format!("mixer{n}.dist"), ch.dist as i32);
        }
    }

    /// One turn of the UI loop: sends what the panel did, then takes what the engine did.
    pub fn sync(&mut self, panel: &mut Panel) {
        self.forward(panel);
        if !self.backlog.is_empty() {
            self.flush_marked();
        }
        self.player.collect_garbage();
        self.follow(panel);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A new song's pattern of row `row`: 16 steps, no shuffle, every step empty.
    fn empty(row: usize) -> Vec<u8> {
        let (_, plen, _) = layout(row);
        let mut p = emptied_pattern(row, &vec![0xFF; plen]);
        p[..2].copy_from_slice(&[0, 16]);
        p
    }

    #[test]
    fn a_pattern_is_empty_when_no_step_is_set() {
        for row in 0..4 {
            let e = empty(row);
            assert!(is_empty_pattern(row, &e), "row {row}");
            // the length and the shuffle flag do not count
            let mut other = e.clone();
            other[..2].copy_from_slice(&[1, 7]);
            assert!(is_empty_pattern(row, &other), "row {row}");
            // the last byte of the last step is used
            let mut used = e.clone();
            *used.last_mut().unwrap() ^= 1;
            assert!(!is_empty_pattern(row, &used), "row {row}");
            assert_eq!(emptied_pattern(row, &used), e, "row {row}");
        }
        // a 303 step of zeros is a note, not a rest
        let mut note = empty(0);
        note[2 + 5] = 0;
        assert!(!is_empty_pattern(1, &note));
        assert_eq!(&emptied_pattern(1, &note)[2..8], &CLEAR_303_STEP);
    }

    /// Row `row`'s 32 patterns, empty but those in `used`.
    fn row_with(row: usize, used: &[u8]) -> Vec<u8> {
        let mut v = Vec::new();
        for p in 0..32 {
            let mut e = empty(row);
            if used.contains(&p) {
                e[2] ^= 1;
            }
            v.extend(e);
        }
        v
    }

    #[test]
    fn the_next_empty_pattern_skips_used_ones_and_wraps() {
        for row in [0, 2, 3] {
            // the next one, crossing from bank A to bank B
            assert_eq!(next_empty_pattern(row, &row_with(row, &[0, 8]), 7), Some(9));
            assert_eq!(next_empty_pattern(row, &row_with(row, &[]), 7), Some(8));
            // from D8 on to A1, and past the used A1
            assert_eq!(next_empty_pattern(row, &row_with(row, &[31]), 31), Some(0));
            assert_eq!(
                next_empty_pattern(row, &row_with(row, &[0, 31]), 31),
                Some(1)
            );
            // the pattern it starts from does not count
            let all_but_5: Vec<u8> = (0..32).filter(|&p| p != 5).collect();
            assert_eq!(next_empty_pattern(row, &row_with(row, &all_but_5), 5), None);
            let all: Vec<u8> = (0..32).collect();
            assert_eq!(next_empty_pattern(row, &row_with(row, &all), 12), None);
            assert_eq!(
                next_empty_pattern(row, &row_with(row, &all_but_5), 30),
                Some(5)
            );
        }
    }
}
