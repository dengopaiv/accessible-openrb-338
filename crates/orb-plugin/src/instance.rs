//! The plugin's main-thread side (task-197): the parameters, the document the state saves, and
//! the calls the editor uses (`docs/plugin.md` sections 3, 5 and 6).
//!
//! [`OrbParams`] is the plugin's `Params` object: the [`HostParam`]s of [`crate::params::SPECS`]
//! and, behind a mutex that only main-thread calls take (never `process`), the UI half of the
//! player ([`orb_player::Player`]) and the [`Doc`]: the song as loaded, the patterns as edited,
//! the Vintage sound model, the mod and the editor size. The controls are the parameters
//! themselves, which the host saves.
//!
//! A song for the engine is built here ([`build`]): the song, in pattern mode and stopped, the
//! document's patterns and sound model, then every parameter whose value the engine does not
//! already hold, through the panel's messages; it reaches the process side as
//! `Command::Load`.

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicU8, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as B64;
use nice_plug::plugin::ParamValue;
use nice_plug::prelude::*;
use orb_engine::frame::Engine;
use orb_engine::frame::edit::{PATTERNS, PatternShape};
use orb_engine::samples::SampleSet;
use orb_engine::song::{Device, Song};
use orb_engine::tables::Tables;
use orb_player::{Audio, Command, Player};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::core::BLOCK;
use crate::params::{self, COUNT, SPECS, Spec};

/// The devices of the four pattern sections.
pub const ROWS: [Device; 4] = [
    Device::Tb303_1,
    Device::Tb303_2,
    Device::Tr808,
    Device::Tr909,
];

/// The persisted field the document is saved under.
pub const STATE_KEY: &str = "orb";
/// The version of the saved document's format.
pub const STATE_VERSION: u32 = 1;

/// The engine's tables, shared by every instance (they are constant).
pub fn tables() -> Arc<Tables> {
    static T: OnceLock<Arc<Tables>> = OnceLock::new();
    T.get_or_init(|| Arc::new(Tables::new())).clone()
}

/// The drum samples of an instance.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum Kit {
    /// The built-in kit (`assets/Standard.rbm`), never embedded in the state.
    #[default]
    Standard,
    /// A recrypted mod, embedded in the state: its INFO name, the SHA-256 of its bytes (hex)
    /// and the bytes.
    Mod {
        name: String,
        sha256: String,
        bytes: Arc<Vec<u8>>,
    },
}

/// What host notes on the section channels do and when the patterns play, outside the
/// preview (task-217, task-218, `docs/plugin.md` section 4): a setting of the plugin's state,
/// not a host parameter, so automation cannot change it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    /// Latch Pattern Mode (task-217's pattern mode): the patterns play while the host plays,
    /// the step following the host position; with the host stopped there is no sound, and
    /// notes 0..31 on channels 1..4 select patterns until the next note.
    Latch,
    /// Gate Pattern Mode (task-218): a section plays only while a note 0..31 on its channel is
    /// held, the held note's pattern, from the note-on to the note-off; with the host playing
    /// the step follows the host position, with the host stopped it starts at step 1.
    Gate,
    /// The song is stopped; notes on channels 1..4 play the devices (task-200). The default: a
    /// plugin just inserted makes no sound of its own, as the mode parameter's Stop before.
    #[default]
    Note,
}

impl Mode {
    /// Latch or Gate Pattern Mode: the patterns play while the host plays.
    pub fn patterns(self) -> bool {
        self != Mode::Note
    }

    fn to_u8(self) -> u8 {
        match self {
            Mode::Latch => 0,
            Mode::Gate => 1,
            Mode::Note => 2,
        }
    }

    fn from_u8(v: u8) -> Mode {
        match v {
            0 => Mode::Latch,
            1 => Mode::Gate,
            _ => Mode::Note,
        }
    }
}

/// What the state saves besides the parameters.
#[derive(Clone, Debug, PartialEq)]
pub struct Doc {
    /// The song file as loaded (`None`: a new song).
    pub song: Option<Vec<u8>>,
    /// The four pattern arrays as edited (303 #1, 303 #2, 808, 909; 32 patterns each), as the
    /// engine holds them; empty before the first build (the song's own).
    pub patterns: [Vec<u8>; 4],
    /// The Vintage sound model (song msg 35 id 21; `None` before the first build: the song's).
    pub vintage: Option<bool>,
    pub kit: Kit,
    /// The editor's size, kept for the editor (task-201).
    pub editor_size: Option<(u32, u32)>,
    /// Whether the Standard Mapping (Remote MIDI) is on; off by default, as the original's.
    pub remote_enabled: bool,
    /// The Standard Mapping's channel, 0..=15 (MIDI channel 1..16; never 0..=3, the sections'
    /// channels 1..4: [`check_remote_channel`]); [`DEFAULT_REMOTE_CHANNEL`].
    pub remote_channel: u8,
    /// Latch Pattern, Gate Pattern or Note Mode.
    pub mode: Mode,
    /// The preview set (task-217): the controls as the panel set them during the preview, by
    /// parameter id; `None` until the first preview, which starts from the parameters' values.
    /// An id it lacks takes the parameter's value.
    pub preview: Option<BTreeMap<String, i32>>,
}

impl Default for Doc {
    fn default() -> Doc {
        Doc {
            song: None,
            patterns: Default::default(),
            vintage: None,
            kit: Kit::default(),
            editor_size: None,
            remote_enabled: false,
            remote_channel: DEFAULT_REMOTE_CHANNEL,
            mode: Mode::default(),
            preview: None,
        }
    }
}

/// The preview set `set` as values in [`SPECS`] order: a parameter it lacks takes its value in
/// `values`, a value out of range is clamped.
pub fn preview_values(set: &BTreeMap<String, i32>, values: &[i32; COUNT]) -> [i32; COUNT] {
    let mut out = *values;
    for (o, s) in out.iter_mut().zip(&SPECS) {
        if let Some(&v) = set.get(&params::id(s.target)) {
            *o = v.clamp(s.min as i32, s.max as i32);
        }
    }
    out
}

/// `values` as a preview set, by parameter id.
pub fn preview_set(values: &[i32; COUNT]) -> BTreeMap<String, i32> {
    SPECS
        .iter()
        .zip(values)
        .map(|(s, &v)| (params::id(s.target), v))
        .collect()
}

/// The Standard Mapping's default channel (0-based): MIDI channel 16.
pub const DEFAULT_REMOTE_CHANNEL: u8 = 15;

fn default_remote_channel() -> u8 {
    DEFAULT_REMOTE_CHANNEL
}

/// The MIDI channels (0-based) of the four pattern sections, 303 #1, 303 #2, 808 and 909: MIDI
/// channels 1..4.
pub const SECTION_CHANNELS: std::ops::RangeInclusive<u8> = 0..=3;

/// Refuses a remote channel (0-based) that is a section's channel or not a MIDI channel.
pub fn check_remote_channel(c: u8) -> Result<(), String> {
    if SECTION_CHANNELS.contains(&c) {
        return Err(format!(
            "MIDI channel {} is a pattern section's channel (1..4); the remote channel must be \
             another",
            c + 1
        ));
    }
    if c > 15 {
        return Err(format!("MIDI channel {} does not exist", c as u32 + 1));
    }
    Ok(())
}

/// The settings the main thread shares with the process side (atomics only).
#[derive(Debug)]
pub struct Shared {
    remote_enabled: AtomicBool,
    remote_channel: AtomicU8,
    /// The [`Mode`] ([`Mode::to_u8`]).
    mode: AtomicU8,
    /// The preview runs (the panel's Play).
    preview: AtomicBool,
    /// The preview set's values in [`SPECS`] order, written before `preview` is set.
    preview_values: [AtomicI32; COUNT],
}

impl Default for Shared {
    fn default() -> Shared {
        Shared {
            remote_enabled: AtomicBool::new(false),
            remote_channel: AtomicU8::new(DEFAULT_REMOTE_CHANNEL),
            mode: AtomicU8::new(Mode::default().to_u8()),
            preview: AtomicBool::new(false),
            preview_values: std::array::from_fn(|i| AtomicI32::new(SPECS[i].default as i32)),
        }
    }
}

impl Shared {
    /// The mode.
    pub fn mode(&self) -> Mode {
        Mode::from_u8(self.mode.load(Ordering::Relaxed))
    }

    fn set_mode(&self, m: Mode) {
        self.mode.store(m.to_u8(), Ordering::Relaxed);
    }

    /// Whether the preview runs.
    pub fn previewing(&self) -> bool {
        self.preview.load(Ordering::Acquire)
    }

    fn set_previewing(&self, on: bool) {
        self.preview.store(on, Ordering::Release);
    }

    /// The preview set's value of parameter `i` ([`SPECS`] order).
    pub fn preview_value(&self, i: usize) -> i32 {
        self.preview_values[i].load(Ordering::Relaxed)
    }

    fn set_preview_values(&self, v: &[i32; COUNT]) {
        for (a, &x) in self.preview_values.iter().zip(v) {
            a.store(x, Ordering::Relaxed);
        }
    }

    /// Whether the Standard Mapping is on.
    pub fn remote_enabled(&self) -> bool {
        self.remote_enabled.load(Ordering::Relaxed)
    }

    fn set_remote_enabled(&self, on: bool) {
        self.remote_enabled.store(on, Ordering::Relaxed);
    }

    /// The Standard Mapping's channel, 0..=15.
    pub fn remote_channel(&self) -> u8 {
        self.remote_channel.load(Ordering::Relaxed)
    }

    fn set_remote_channel(&self, c: u8) {
        self.remote_channel.store(c, Ordering::Relaxed);
    }
}

#[derive(Serialize, Deserialize)]
struct SavedMod {
    name: String,
    sha256: String,
    data: String,
}

#[derive(Serialize, Deserialize)]
struct Saved {
    version: u32,
    song: Option<String>,
    patterns: Vec<String>,
    vintage: Option<bool>,
    #[serde(rename = "mod")]
    kit: Option<SavedMod>,
    editor_size: Option<(u32, u32)>,
    /// Absent: off.
    #[serde(default)]
    remote_enabled: bool,
    /// Absent: channel 16 (0-based 15).
    #[serde(default = "default_remote_channel")]
    remote_channel: u8,
    mode: Mode,
    /// Null: no preview yet.
    preview: Option<BTreeMap<String, i32>>,
}

impl Doc {
    /// The document as the state's JSON.
    pub fn to_json(&self) -> String {
        let s = Saved {
            version: STATE_VERSION,
            song: self.song.as_ref().map(|b| B64.encode(b)),
            patterns: self.patterns.iter().map(|p| B64.encode(p)).collect(),
            vintage: self.vintage,
            kit: match &self.kit {
                Kit::Standard => None,
                Kit::Mod {
                    name,
                    sha256,
                    bytes,
                } => Some(SavedMod {
                    name: name.clone(),
                    sha256: sha256.clone(),
                    data: B64.encode(bytes.as_slice()),
                }),
            },
            editor_size: self.editor_size,
            remote_enabled: self.remote_enabled,
            remote_channel: self.remote_channel,
            mode: self.mode,
            preview: self.preview.clone(),
        };
        serde_json::to_string(&s).expect("the document serializes")
    }

    /// The document of a state's JSON. States of earlier builds are not supported (the user,
    /// 2026-11-16: no compatibility until the plugin's first release).
    pub fn from_json(json: &str) -> Result<Doc, String> {
        let s: Saved = serde_json::from_str(json).map_err(|e| e.to_string())?;
        if s.version != STATE_VERSION {
            return Err(format!(
                "state version {} is not {STATE_VERSION}",
                s.version
            ));
        }
        let bytes = |t: &str| B64.decode(t).map_err(|e| e.to_string());
        let mut patterns: [Vec<u8>; 4] = Default::default();
        if s.patterns.len() != 4 {
            return Err(format!("{} pattern arrays, not 4", s.patterns.len()));
        }
        for (r, (p, t)) in patterns.iter_mut().zip(&s.patterns).enumerate() {
            *p = bytes(t)?;
            if !p.is_empty() && p.len() != pattern_len(r) * PATTERNS {
                return Err(format!("pattern array {r} is {} bytes", p.len()));
            }
        }
        let kit = match s.kit {
            None => Kit::Standard,
            Some(m) => {
                let data = bytes(&m.data)?;
                if sha256_hex(&data) != m.sha256 {
                    return Err(format!("the mod {:?} does not match its SHA-256", m.name));
                }
                Kit::Mod {
                    name: m.name,
                    sha256: m.sha256,
                    bytes: Arc::new(data),
                }
            }
        };
        // a section's channel as the remote channel is checked by the restore, which keeps the
        // rest of the document (`OrbParams::deserialize_fields`)
        Ok(Doc {
            song: s.song.as_deref().map(bytes).transpose()?,
            patterns,
            vintage: s.vintage,
            kit,
            editor_size: s.editor_size,
            remote_enabled: s.remote_enabled,
            remote_channel: s.remote_channel,
            mode: s.mode,
            preview: s.preview,
        })
    }
}

/// The SHA-256 of `b`, lower-case hex.
pub fn sha256_hex(b: &[u8]) -> String {
    Sha256::digest(b)
        .iter()
        .map(|x| format!("{x:02x}"))
        .collect()
}

/// The length of one pattern of section `r`.
fn pattern_len(r: usize) -> usize {
    PatternShape::of(ROWS[r]).expect("a pattern device").len
}

/// A song file's engine model: an RBS 4.x song or a ReBirth 1.x MIDI-file song.
pub fn parse_song(data: &[u8]) -> Result<Song, String> {
    if rbs::rb1x::is_1x(data) {
        let s = rbs::rb1x::Song1x::parse(data)
            .map_err(|e| format!("not a valid ReBirth 1.x song: {e}"))?;
        return orb_engine::load::convert_1x(&s).map_err(|e| format!("cannot load the song: {e}"));
    }
    let s = rbs::Song::parse(data).map_err(|e| format!("not a valid .rbs song: {e}"))?;
    orb_engine::load::convert(&s).map_err(|e| format!("cannot load the song: {e}"))
}

/// The kit of a recrypted mod's bytes: the built-in kit for the standard mod, else the mod,
/// with the built-in samples in the slots it lacks.
pub fn load_kit(bytes: Vec<u8>) -> Result<(Kit, Arc<SampleSet>), String> {
    let sha256 = sha256_hex(&bytes);
    let rbm = rbm::Rbm::parse(bytes.clone()).map_err(|e| format!("not a mod: {e}"))?;
    if rbm.is_standard() {
        return Ok((Kit::Standard, rbm::standard::builtin_samples()));
    }
    let table = rbm::KeyTable::embedded(&rbm)
        .filter(|t| rbm.slots_covered(t))
        .ok_or("the mod is not recrypted (run `orb mod recrypt` on it once)")?;
    let set = rbm
        .load_samples(&table, &rbm::standard::builtin_samples())
        .map_err(|e| e.to_string())?
        .set;
    let name = rbm.info().map(|i| i.name.clone()).unwrap_or_default();
    let bytes = Arc::new(bytes);
    Ok((
        Kit::Mod {
            name,
            sha256,
            bytes,
        },
        Arc::new(set),
    ))
}

/// The engine bytes `e` holds for every parameter, as parameter values.
pub fn values_of(e: &Engine) -> [i32; COUNT] {
    let mut v = [0; COUNT];
    for (x, s) in v.iter_mut().zip(&SPECS) {
        *x = params::engine_byte(e, s.target).map_or(s.default as i32, |b| params::value_of(s, b));
    }
    v
}

/// The engine's four pattern arrays.
fn read_patterns(e: &mut Engine) -> [Vec<u8>; 4] {
    let mut out: [Vec<u8>; 4] = Default::default();
    for (r, o) in out.iter_mut().enumerate() {
        if let Some(p) = e.frame.devices.channels[r].patterns_mut() {
            *o = p[..pattern_len(r) * PATTERNS].to_vec();
        }
    }
    out
}

/// Applies a panel message to an engine that no callback runs yet.
fn apply_to(e: &mut Engine, c: Command) {
    match c {
        Command::DeviceParam {
            device,
            ctrl,
            value,
        } => e.device_param(device, ctrl, value),
        Command::MixerGlobal { id, value } => e.mixer_global(id, value),
        Command::MixerChannel {
            channel,
            field,
            value,
        } => e.mixer_channel(channel, field, value),
        Command::Shuffle(v) => e.set_shuffle(v),
        _ => {}
    }
}

/// Builds the engine of `doc`: its song (or a new one) on `player`'s process globals with
/// `samples`, in pattern mode and stopped, with the document's patterns (when it has them) and
/// sound model, then `values` where the engine does not already hold them. The document's
/// patterns and sound model are then the engine's.
pub fn build(
    player: &Player,
    doc: &mut Doc,
    samples: &Arc<SampleSet>,
    values: Option<&[i32; COUNT]>,
) -> Result<Box<Engine>, String> {
    let mut e = match &doc.song {
        Some(bytes) => {
            let song = parse_song(bytes)?;
            player
                .build(&song, Some(samples.clone()))
                .map_err(|r| r.to_string())?
        }
        None => player.build_new(Some(samples.clone())),
    };
    if e.frame.seq.globals.song_mode != 0 {
        e.set_song_mode(0);
    }
    if e.playing() {
        e.stop();
    }
    for (r, &d) in ROWS.iter().enumerate() {
        let len = pattern_len(r);
        if doc.patterns[r].len() == len * PATTERNS {
            for (p, bytes) in doc.patterns[r].chunks(len).enumerate() {
                e.pattern(d, p as u8, bytes);
            }
        }
    }
    if let Some(v) = doc.vintage {
        e.set_vintage(v as u8);
    }
    if let Some(values) = values {
        for (s, &v) in SPECS.iter().zip(values) {
            set_value(&mut e, s, v);
        }
    }
    doc.patterns = read_patterns(&mut e);
    doc.vintage = Some(e.frame.vintage);
    Ok(e)
}

/// Sets parameter `s` to `v` in `e` through the panel's message, unless `e` holds it.
fn set_value(e: &mut Engine, s: &Spec, v: i32) {
    if params::engine_byte(e, s.target).is_some_and(|b| params::value_of(s, b) == v) {
        return;
    }
    if let Some(c) = params::command(s, v) {
        apply_to(e, c);
    }
}

/// The main-thread state of an instance.
pub struct Main {
    pub doc: Doc,
    /// The UI half of the player, from the first activation on.
    player: Option<Player>,
    /// The samples of `doc.kit`.
    samples: Option<Arc<SampleSet>>,
    /// The last error of a build from the state (the song or the mod could not be loaded).
    pub last_error: Option<String>,
}

impl Main {
    fn samples(&mut self) -> Arc<SampleSet> {
        if let Some(s) = &self.samples {
            return s.clone();
        }
        let s = match &self.doc.kit {
            Kit::Standard => rbm::standard::builtin_samples(),
            Kit::Mod { bytes, .. } => match load_kit(bytes.as_ref().clone()) {
                Ok((_, s)) => s,
                Err(e) => {
                    self.last_error = Some(format!("the mod: {e}"));
                    rbm::standard::builtin_samples()
                }
            },
        };
        self.samples = Some(s.clone());
        s
    }

    /// The engine of the document with `values`; a song that does not load gives a new song
    /// (and [`Main::last_error`]).
    fn build_doc(&mut self, player: &Player, values: &[i32; COUNT]) -> Box<Engine> {
        let samples = self.samples();
        match build(player, &mut self.doc, &samples, Some(values)) {
            Ok(e) => e,
            Err(err) => {
                self.last_error = Some(err);
                self.doc.song = None;
                self.doc.patterns = Default::default();
                build(player, &mut self.doc, &samples, Some(values)).expect("a new song builds")
            }
        }
    }

    /// A new player for an activation, its engine built from the document and `values`: the
    /// callback half, with the song queued (it becomes current at the first block).
    pub fn activate(&mut self, values: &[i32; COUNT]) -> Audio {
        let (mut player, audio) = Player::with_call_frames(tables(), None, BLOCK);
        let e = self.build_doc(&player, values);
        assert!(player.send(Command::Load(e)).is_ok(), "an empty queue");
        self.player = Some(player);
        audio
    }

    /// Sends a command; a full queue is an error.
    fn send(&mut self, c: Command) -> Result<(), String> {
        match &mut self.player {
            Some(p) => {
                // what the process side let go of (an old song, a mod) is dropped here, on the
                // main thread, so that the garbage queue has room for the command
                p.collect_garbage();
                p.send(c)
                    .map_err(|_| "the plugin's command queue is full".to_string())
            }
            None => Ok(()),
        }
    }

    /// Rebuilds the engine from the document after a state restore.
    fn reload(&mut self, values: &[i32; COUNT]) {
        self.samples = None;
        if let Some(player) = self.player.take() {
            let e = self.build_doc(&player, values);
            self.player = Some(player);
            if let Err(err) = self.send(Command::Load(e)) {
                self.last_error = Some(err);
            }
        }
    }

    /// Builds a song (`None`: a new song) for loading, in a throwaway player before the first
    /// activation; returns the engine and its parameter values.
    fn build_song(&mut self, song: Option<Vec<u8>>) -> Result<(Box<Engine>, Doc), String> {
        let mut doc = Doc {
            song,
            patterns: Default::default(),
            vintage: None,
            kit: self.doc.kit.clone(),
            editor_size: self.doc.editor_size,
            remote_enabled: self.doc.remote_enabled,
            remote_channel: self.doc.remote_channel,
            mode: self.doc.mode,
            preview: None,
        };
        let samples = self.samples();
        let e = match &self.player {
            Some(p) => build(p, &mut doc, &samples, None)?,
            None => {
                let (p, _) = Player::new(tables(), None);
                build(&p, &mut doc, &samples, None)?
            }
        };
        Ok((e, doc))
    }
}

/// One host parameter (task-275, `docs/plugin.md` section 3): a nice-plug `FloatParam` for a
/// continuous one ([`Spec::continuous`]: linear over min..max, no step size, so no VST3 step
/// count), an `IntParam` for a discrete one. Both normalize a value `v` as
/// `(v - min) / (max - min)`, as the released build's `IntParam`s did, so a host's saved
/// automation and values keep their meaning. [`HostParam::value`] is the integer the engine
/// gets.
pub enum HostParam {
    Stepped(IntParam),
    Continuous(FloatParam, Spec),
}

impl HostParam {
    fn new(s: &Spec) -> HostParam {
        let spec = *s;
        let parse_spec = *s;
        let name = params::name(s.target);
        if s.continuous() {
            HostParam::Continuous(
                FloatParam::new(
                    name,
                    s.default as f32,
                    FloatRange::Linear {
                        min: s.min as f32,
                        max: s.max as f32,
                    },
                )
                .with_value_to_string(Arc::new(move |x| {
                    params::display(&spec, params::round(&spec, x))
                }))
                .with_string_to_value(Arc::new(move |t| {
                    params::parse(&parse_spec, t).map(|v| v as f32)
                })),
                *s,
            )
        } else {
            HostParam::Stepped(
                IntParam::new(
                    name,
                    s.default as i32,
                    IntRange::Linear {
                        min: s.min as i32,
                        max: s.max as i32,
                    },
                )
                .with_value_to_string(Arc::new(move |v| params::display(&spec, v)))
                .with_string_to_value(Arc::new(move |t| params::parse(&parse_spec, t))),
            )
        }
    }

    /// The integer the parameter's value stands for: a continuous one's host value rounded
    /// ([`params::round`]), a stepped one's value. Atomics only.
    pub fn value(&self) -> i32 {
        match self {
            HostParam::Stepped(p) => p.value(),
            HostParam::Continuous(p, s) => params::round(s, p.value()),
        }
    }

    /// Whether the host sees the parameter as continuous.
    pub fn is_continuous(&self) -> bool {
        matches!(self, HostParam::Continuous(..))
    }

    /// The host's (normalized) value of the integer `v`.
    pub fn preview_normalized(&self, v: i32) -> f32 {
        match self {
            HostParam::Stepped(p) => p.preview_normalized(v),
            HostParam::Continuous(p, _) => p.preview_normalized(v as f32),
        }
    }

    /// The parameter's current normalized value, as the host sees it.
    pub fn normalized_value(&self) -> f32 {
        match self {
            HostParam::Stepped(p) => p.unmodulated_normalized_value(),
            HostParam::Continuous(p, _) => p.unmodulated_normalized_value(),
        }
    }

    pub fn as_ptr(&self) -> ParamPtr {
        match self {
            HostParam::Stepped(p) => p.as_ptr(),
            HostParam::Continuous(p, _) => p.as_ptr(),
        }
    }
}

/// The plugin's parameters and its main-thread state.
pub struct OrbParams {
    params: Vec<HostParam>,
    ids: Vec<String>,
    main: Mutex<Main>,
    shared: Arc<Shared>,
    /// Counts the changes of the song (a load, File > New, a state restore): the editor shows
    /// the song anew when it changes ([`OrbParams::revision`]).
    revision: AtomicU64,
    /// A state was loaded into this instance (runtime only, never saved; task-219).
    restored: AtomicBool,
    /// An editor has started the logo animation (runtime only, never saved; task-219).
    intro_played: AtomicBool,
}

impl Default for OrbParams {
    fn default() -> Self {
        Self::new()
    }
}

impl OrbParams {
    pub fn new() -> Self {
        let params = SPECS.iter().map(HostParam::new).collect();
        OrbParams {
            params,
            ids: SPECS.iter().map(|s| params::id(s.target)).collect(),
            main: Mutex::new(Main {
                doc: Doc::default(),
                player: None,
                samples: None,
                last_error: None,
            }),
            shared: Arc::new(Shared::default()),
            revision: AtomicU64::new(0),
            restored: AtomicBool::new(false),
            intro_played: AtomicBool::new(false),
        }
    }

    /// Whether the editor that opens now plays the logo animation (task-219, docs/plugin.md
    /// section 5.1): once per instance, at its first editor open, and only if no state was
    /// ever loaded into it. Calling it counts as the editor starting the animation.
    pub fn take_intro(&self) -> bool {
        !self.intro_played.swap(true, Ordering::AcqRel) && !self.restored.load(Ordering::Acquire)
    }

    /// The song's revision: it changes at each load, File > New and state restore.
    pub fn revision(&self) -> u64 {
        self.revision.load(Ordering::Acquire)
    }

    /// What the editor shows of the song (task-201), as the engine would hold it with the
    /// parameter values `values`: a snapshot of a throwaway engine built from the document
    /// (one stopped 64-frame call, which is not heard) and the four pattern arrays. The
    /// editor opened before an activation, or after a load or a restore whose engine the host
    /// has not run yet, shows the song this way.
    pub fn view(&self, values: &[i32; COUNT]) -> (orb_player::Snapshot, [Vec<u8>; 4]) {
        let mut m = self.main();
        let samples = m.samples();
        let mut doc = m.doc.clone();
        drop(m);
        let (mut player, mut audio) = Player::with_call_frames(tables(), None, BLOCK);
        let e = match build(&player, &mut doc, &samples, Some(values)) {
            Ok(e) => e,
            Err(_) => {
                doc.song = None;
                doc.patterns = Default::default();
                build(&player, &mut doc, &samples, Some(values)).expect("a new song builds")
            }
        };
        assert!(player.send(Command::Load(e)).is_ok(), "an empty queue");
        audio.drain();
        audio.render_call();
        let snapshot = *audio.last_snapshot();
        drop(audio);
        player.collect_garbage();
        (snapshot, doc.patterns)
    }

    /// The preview player's commands (the mod's startup sound: `Command::Preview`,
    /// `Command::PreviewStop`); others are refused.
    pub fn preview(&self, c: Command) -> Result<(), String> {
        if !matches!(c, Command::Preview(_) | Command::PreviewStop) {
            return Err("not a preview command".into());
        }
        self.main().send(c)
    }

    /// The last error of a build from the state, once (the editor shows it).
    pub fn take_error(&self) -> Option<String> {
        self.main().last_error.take()
    }

    /// The instance's kit (the built-in kit or the embedded mod).
    pub fn kit(&self) -> Kit {
        self.main().doc.kit.clone()
    }

    /// The song file as loaded (`None`: a new song).
    pub fn song(&self) -> Option<Vec<u8>> {
        self.main().doc.song.clone()
    }

    /// Drops what the process side let go of (the editor calls it every frame).
    pub fn collect_garbage(&self) {
        if let Some(p) = &mut self.main().player {
            p.collect_garbage();
        }
    }

    fn bump(&self) {
        self.revision.fetch_add(1, Ordering::AcqRel);
    }

    /// The settings shared with the process side.
    pub fn shared(&self) -> &Arc<Shared> {
        &self.shared
    }

    /// Whether the Standard Mapping (Remote MIDI) is on (default off).
    pub fn remote_enabled(&self) -> bool {
        self.main().doc.remote_enabled
    }

    /// Turns the Standard Mapping on or off, kept in the state; on, it listens on the remote
    /// channel (default MIDI channel 16).
    pub fn set_remote_enabled(&self, on: bool) {
        let mut m = self.main();
        m.doc.remote_enabled = on;
        self.shared.set_remote_enabled(on);
    }

    /// The Standard Mapping's channel (0-based; default 15, MIDI channel 16).
    pub fn remote_channel(&self) -> u8 {
        self.main().doc.remote_channel
    }

    /// Sets the Standard Mapping's channel (0-based), kept in the state. A pattern section's
    /// channel (0..=3, MIDI channels 1..4) is refused: its notes select patterns.
    pub fn set_remote_channel(&self, c: u8) -> Result<(), String> {
        check_remote_channel(c)?;
        let mut m = self.main();
        m.doc.remote_channel = c;
        self.shared.set_remote_channel(c);
        Ok(())
    }

    /// The mode (default Note Mode), kept in the state.
    pub fn mode(&self) -> Mode {
        self.main().doc.mode
    }

    /// Sets the mode (the panel's mode selector; File > Open and File > New keep it, task-228);
    /// the process side takes it at its next engine block.
    pub fn set_mode(&self, mode: Mode) {
        let mut m = self.main();
        m.doc.mode = mode;
        self.shared.set_mode(mode);
    }

    /// Whether the preview runs (the panel's Play, task-217). Not saved: a restore ends it.
    pub fn previewing(&self) -> bool {
        self.shared.previewing()
    }

    /// The panel's Play (`true`) and Stop (`false`): enters or leaves the preview. The first
    /// entry makes the preview set from the parameters' values; later entries restore it. The
    /// process side takes the change at its next engine block: entering, it plays the preview
    /// set free-running; leaving, it returns to the parameters' values.
    pub fn set_preview(&self, on: bool) {
        if on {
            let values = self.values();
            let mut m = self.main();
            let set = m.doc.preview.get_or_insert_with(|| preview_set(&values));
            self.shared
                .set_preview_values(&preview_values(set, &values));
        }
        self.shared.set_previewing(on);
    }

    /// A control the panel changed during the preview: parameter `i` ([`SPECS`] order) of the
    /// preview set becomes `v` (clamped to its range). The host sees nothing of it. Refused
    /// when the preview does not run.
    pub fn set_preview_value(&self, i: usize, v: i32) -> Result<(), String> {
        if !self.previewing() {
            return Err("the preview does not run".into());
        }
        let s = SPECS.get(i).ok_or("no such parameter")?;
        let v = v.clamp(s.min as i32, s.max as i32);
        let values = self.values();
        let mut m = self.main();
        let set = m.doc.preview.get_or_insert_with(|| preview_set(&values));
        set.insert(params::id(s.target), v);
        self.shared.preview_values[i].store(v, Ordering::Relaxed);
        Ok(())
    }

    /// The preview set's values ([`SPECS`] order; a parameter it lacks takes its value), or
    /// `None` before the first preview.
    pub fn preview_values(&self) -> Option<[i32; COUNT]> {
        let values = self.values();
        let m = self.main();
        m.doc.preview.as_ref().map(|s| preview_values(s, &values))
    }

    /// The values the panel and the engine play: the preview set's while the preview runs,
    /// else the parameters'.
    pub fn panel_values(&self) -> [i32; COUNT] {
        if self.previewing() {
            std::array::from_fn(|i| self.shared.preview_value(i))
        } else {
            self.values()
        }
    }

    /// The player's snapshot after the last engine block (`None` before the first activation):
    /// the engine's state, the Sync LED ([`orb_player::Snapshot::host_sync`]) and the MIDI
    /// activity ([`orb_player::Snapshot::midi_in`]).
    pub fn snapshot(&self) -> Option<orb_player::Snapshot> {
        self.main().player.as_mut().map(|p| *p.snapshot())
    }

    /// The panel's pattern button of section `row` (0..=3): the pattern controller's message
    /// to the engine, which plays it from the next bar line, as the original's panel does. For
    /// the editor when the pattern parameter already holds `pattern` (a host note selected
    /// another since), so that the host sees no change; otherwise the editor sets the
    /// parameter.
    pub fn select_pattern(&self, row: usize, pattern: u8) -> Result<(), String> {
        let device = *ROWS.get(row).ok_or("no such pattern section")?;
        if pattern > 31 {
            return Err("no such pattern".into());
        }
        self.main().send(Command::DeviceParam {
            device,
            ctrl: 1,
            value: pattern,
        })
    }

    /// Every parameter's current value, in [`SPECS`] order (atomics only): the integers, a
    /// continuous parameter's rounded ([`HostParam::value`]).
    pub fn values(&self) -> [i32; COUNT] {
        let mut v = [0; COUNT];
        for (x, p) in v.iter_mut().zip(&self.params) {
            *x = p.value();
        }
        v
    }

    /// The parameter at index `i` of [`SPECS`].
    pub fn param_at(&self, i: usize) -> &HostParam {
        &self.params[i]
    }

    /// The parameter of `id`.
    pub fn param(&self, id: &str) -> Option<&HostParam> {
        self.ids
            .iter()
            .position(|i| i == id)
            .map(|i| &self.params[i])
    }

    /// The parameters' ids in [`SPECS`] order.
    pub fn ids(&self) -> &[String] {
        &self.ids
    }

    /// The main-thread state. Never taken by `process`.
    pub fn main(&self) -> MutexGuard<'_, Main> {
        self.main.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// The process side's player half for a new activation (see [`Main::activate`]).
    pub fn activate(&self) -> Audio {
        let values = self.values();
        self.main().activate(&values)
    }

    /// File > Open: makes `data` (an `.rbs` or a ReBirth 1.x song) the instance's song, with its
    /// own patterns and controls and the instance's mod. The song is sent to the engine (it
    /// plays at once when the mode is Play); the controls are the song's from then on, and the
    /// caller sets the parameters to the values returned (parameter id, value) through the
    /// host, so that the host and the state follow (the mode is not among them).
    pub fn load_song(&self, data: Vec<u8>) -> Result<Vec<(String, i32)>, String> {
        self.load(Some(data))
    }

    /// File > New: an empty song, as [`OrbParams::load_song`].
    pub fn new_song(&self) -> Result<Vec<(String, i32)>, String> {
        self.load(None)
    }

    fn load(&self, song: Option<Vec<u8>>) -> Result<Vec<(String, i32)>, String> {
        let mut m = self.main();
        let (e, doc) = m.build_song(song)?;
        let values = values_of(&e);
        m.send(Command::Load(e))?;
        let mut doc = doc;
        // the old song's preview set is not this song's: the next preview starts from the new
        // song's values, and a running preview plays them now
        if self.previewing() {
            doc.preview = Some(preview_set(&values));
            self.shared.set_preview_values(&values);
        }
        m.doc = doc;
        drop(m);
        self.bump();
        Ok(self.ids.iter().cloned().zip(values).collect())
    }

    /// The Mods menu: the instance's drum machines take the mod `rbm` (recrypted; `None` or the
    /// standard mod: the built-in kit), as song msg 19 does; the mod is embedded in the state
    /// unless it is the built-in kit.
    pub fn set_mod(&self, rbm: Option<Vec<u8>>) -> Result<(), String> {
        let (kit, samples) = match rbm {
            None => (Kit::Standard, rbm::standard::builtin_samples()),
            Some(b) => load_kit(b)?,
        };
        let mut m = self.main();
        if let Some(p) = &mut m.player {
            p.set_mod(samples.clone())
                .map_err(|_| "the plugin's command queue is full".to_string())?;
        }
        m.doc.kit = kit;
        m.samples = Some(samples);
        Ok(())
    }

    /// A pattern edit of the editor (`Command::PatternField`, `PatternHeader`, `Pattern`,
    /// `StepField`, `Step`): sent to the engine and kept in the document. Other commands are
    /// refused: the controls are parameters.
    pub fn edit(&self, c: Command) -> Result<(), String> {
        let mut m = self.main();
        if !mirror(&mut m.doc.patterns, &c) {
            return Err("not a pattern edit".into());
        }
        m.send(c)
    }

    /// The Vintage sound model (song msg 35 id 21).
    pub fn set_vintage(&self, on: bool) -> Result<(), String> {
        let mut m = self.main();
        m.send(Command::Vintage(on))?;
        m.doc.vintage = Some(on);
        Ok(())
    }

    /// The editor's size as the state keeps it.
    pub fn editor_size(&self) -> Option<(u32, u32)> {
        self.main().doc.editor_size
    }

    pub fn set_editor_size(&self, size: (u32, u32)) {
        self.main().doc.editor_size = Some(size);
    }
}

/// Applies a pattern edit to the document's pattern arrays as the engine applies it
/// (`orb_engine::frame::edit`); false for a command that is not one. An address past the
/// array is dropped, as the engine drops it.
fn mirror(patterns: &mut [Vec<u8>; 4], c: &Command) -> bool {
    let (device, at, bytes): (Device, usize, &[u8]) = match c {
        Command::PatternField {
            device,
            field,
            value,
            ..
        } => {
            if *field > 1 {
                return true;
            }
            (*device, *field as usize, std::slice::from_ref(value))
        }
        Command::PatternHeader { device, header, .. } => (*device, 0, &header[..]),
        Command::Pattern { device, bytes, .. } => (*device, 0, &bytes[..]),
        Command::StepField {
            device,
            step,
            field,
            value,
            ..
        } => {
            let Some(s) = PatternShape::of(*device) else {
                return false;
            };
            if matches!(device, Device::Tb303_1 | Device::Tb303_2) && *field > 5 {
                return true;
            }
            (
                *device,
                2 + s.step * *step as usize + *field as usize,
                std::slice::from_ref(value),
            )
        }
        Command::Step {
            device,
            step,
            bytes,
            ..
        } => {
            let Some(s) = PatternShape::of(*device) else {
                return false;
            };
            (*device, 2 + s.step * *step as usize, &bytes[..s.step])
        }
        _ => return false,
    };
    let pattern = match c {
        Command::PatternField { pattern, .. }
        | Command::PatternHeader { pattern, .. }
        | Command::Pattern { pattern, .. }
        | Command::StepField { pattern, .. }
        | Command::Step { pattern, .. } => *pattern as usize,
        _ => unreachable!(),
    };
    let Some(r) = ROWS.iter().position(|&d| d == device) else {
        return false;
    };
    let start = pattern_len(r) * pattern + at;
    if let Some(dst) = patterns[r].get_mut(start..start + bytes.len()) {
        dst.copy_from_slice(bytes);
    }
    true
}

unsafe impl Params for OrbParams {
    fn param_map(&self) -> Vec<(String, ParamPtr, String)> {
        self.params
            .iter()
            .zip(&self.ids)
            .zip(&SPECS)
            .map(|((p, id), s)| (id.clone(), p.as_ptr(), params::group(s.target).to_string()))
            .collect()
    }

    fn serialize_fields(&self) -> BTreeMap<String, String> {
        let mut m = BTreeMap::new();
        m.insert(STATE_KEY.to_string(), self.main().doc.to_json());
        m
    }

    /// A state restore: nice-plug has set the parameters already; the document is replaced and,
    /// when the instance runs, its engine rebuilt and sent.
    fn deserialize_fields(&self, serialized: &BTreeMap<String, String>) {
        self.restored.store(true, Ordering::Release);
        let values = self.values();
        let mut m = self.main();
        match serialized.get(STATE_KEY).map(|j| Doc::from_json(j)) {
            Some(Ok(mut doc)) => {
                // a remote channel that is a section's falls back to the default; the song,
                // the patterns and the controls are kept
                if let Err(e) = check_remote_channel(doc.remote_channel) {
                    m.last_error = Some(format!("the saved state: {e}; channel 16 is used"));
                    doc.remote_channel = DEFAULT_REMOTE_CHANNEL;
                }
                m.doc = doc;
            }
            Some(Err(e)) => {
                m.last_error = Some(format!("the saved state: {e}"));
                m.doc = Doc::default();
            }
            None => m.doc = Doc::default(),
        }
        self.shared.set_remote_enabled(m.doc.remote_enabled);
        self.shared.set_remote_channel(m.doc.remote_channel);
        self.shared.set_mode(m.doc.mode);
        // the preview is not saved: a restore ends it, the restored state plays as saved
        self.shared.set_previewing(false);
        if let Some(set) = &m.doc.preview {
            self.shared
                .set_preview_values(&preview_values(set, &values));
        }
        m.reload(&values);
        drop(m);
        self.bump();
    }
}

/// The process side for a host rate, after [`OrbParams::activate`].
pub fn core(params: &OrbParams, rate: u32) -> crate::core::Core {
    let audio = params.activate();
    crate::core::Core::new(audio, rate, &params.values(), params.shared.clone())
}

/// Sets a parameter's value as the host would (for tests: nice-plug sets it so before a call).
///
/// # Safety
/// Not concurrently with the host setting it.
pub unsafe fn set_param(p: &HostParam, v: i32) {
    unsafe { set_normalized(p, p.preview_normalized(v)) }
}

/// Sets a parameter's normalized value as the host would (for tests: any value of a host's
/// automation, a continuous parameter's between the integers too).
///
/// # Safety
/// Not concurrently with the host setting it.
pub unsafe fn set_normalized(p: &HostParam, normalized: f32) {
    unsafe {
        p.as_ptr()._internal_set_normalized_value(normalized);
    }
}

/// A plugin state's parameter values as the released build (v2026.1118.0919) saved them: every
/// parameter an `IntParam`, `{"i32": v}`. nice-plug restores only an `F32` into a `FloatParam`
/// (`wrapper/state.rs` `deserialize_object`), so a continuous parameter's `I32` becomes the
/// `F32` of the same value before the restore ([`crate::OrbPlugin`]'s `filter_state`; task-275).
/// A value that is neither is left to nice-plug.
pub fn migrate_state(state: &mut PluginState) {
    for s in SPECS.iter().filter(|s| s.continuous()) {
        if let Some(v) = state.params.get_mut(&params::id(s.target))
            && let ParamValue::I32(i) = *v
        {
            *v = ParamValue::F32(i as f32);
        }
    }
}
