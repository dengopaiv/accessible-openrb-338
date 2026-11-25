//! The host parameters (task-197): one per control the plugin's panel shows, mirroring the u8
//! controller it sets (`docs/plugin.md` section 3): continuous for the host over 0..127 and
//! rounded to the integer ([`Spec::continuous`], [`round`]; task-275), stepped otherwise.
//!
//! The table [`SPECS`] is a constant, so the process side reads it without allocating: what each
//! parameter sets ([`Target`]), its range and its default, the device constructors' value
//! (`docs/song_load.md` section 4.9). The ids ([`id`]) are derived from the device and controller
//! numbers and never change: the host saves automation and state under them.

use orb_engine::frame::Engine;
use orb_engine::song::Device;
use orb_engine::tr909;
use orb_player::Command;

/// What a parameter sets.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Target {
    /// The shuffle amount, song msg 35 id 20.
    Shuffle,
    /// The mixer's master level: msg 145 `{0, v}`.
    Master,
    /// The compressor's place (0 off, 1 the master, 2..5 channel 1..4): msg 145 `{1, v}`.
    CompId,
    /// The PCF's place (0 off, 1..4 channel 1..4): msg 145 `{2, v}`.
    PcfId,
    /// Mixer channel `c`'s field `k` (0 on, 1 level, 2 pan, 3 delay send, 4 distortion):
    /// msg 147 `{c, k, v}`.
    Channel(u8, u8),
    /// Controller `ctrl` of a device other than the mixer: its `{ctrl, v}` message
    /// (`docs/song_load.md` section 4.10).
    Ctrl(Device, u8),
}

/// How a parameter's value relates to the engine's byte.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// The byte itself, within the range.
    Value,
    /// A two-position switch: off at 0, on at any other byte (the devices read it so).
    Switch,
    /// The PCF's mode: low-pass only at 1, band-pass at any other byte (`cmp dl, 1` at
    /// 0x10016921, `crates/orb-ui/src/behavior.rs` `switch_position`).
    PcfMode,
}

/// One parameter.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Spec {
    pub target: Target,
    pub kind: Kind,
    pub min: u8,
    pub max: u8,
    pub default: u8,
}

impl Spec {
    /// Whether the host sees the parameter as continuous (task-275, `docs/plugin.md` section 3):
    /// a [`Kind::Value`] over 0..=127 (the knobs, faders, levels, pans, sends and the shuffle).
    /// A host draws and plays a stepped parameter's automation as steps (Studio One, GitHub
    /// issue 21); the engine still gets the integer ([`round`]). The switches, the patterns,
    /// the delay steps, the PCF wave and the inserts stay stepped.
    pub const fn continuous(&self) -> bool {
        matches!(self.kind, Kind::Value) && self.min == 0 && self.max == 127
    }
}

/// The integer a continuous parameter's host value `x` stands for: the nearest one, a half
/// rounded away from zero (`f32::round`), within the range; NaN gives the minimum. The engine,
/// the panel and the display get this integer.
pub fn round(s: &Spec, x: f32) -> i32 {
    if x.is_nan() {
        return s.min as i32;
    }
    (x.round() as i32).clamp(s.min as i32, s.max as i32)
}

/// The number of parameters. The mode (`g1`, orb's Play/Stop) was one until task-217: it is the
/// plugin state's [`crate::instance::Mode`] now, and the panel's Play/Stop is the preview.
pub const COUNT: usize = 119;

const fn value(target: Target, min: u8, max: u8, default: u8) -> Spec {
    Spec {
        target,
        kind: Kind::Value,
        min,
        max,
        default,
    }
}

const fn switch(target: Target, default: u8) -> Spec {
    Spec {
        target,
        kind: Kind::Switch,
        min: 0,
        max: 1,
        default,
    }
}

/// The 303's controllers 0..8 (`docs/song_load.md` sections 4.6, 4.9).
const fn tb303(d: Device, c: u8) -> Spec {
    let def = orb_engine::tb303::DEFAULT_PARAMS[c as usize];
    match c {
        0 | 8 => switch(Target::Ctrl(d, c), def),
        1 => value(Target::Ctrl(d, c), 0, 31, def),
        _ => value(Target::Ctrl(d, c), 0, 127, def),
    }
}

/// The 808's controllers 0..28; 11, 14, 17, 19 and 21 are the voice switches.
const fn tr808(c: u8) -> Spec {
    let t = Target::Ctrl(Device::Tr808, c);
    let def = orb_engine::tr808::DEFAULT_PARAMS[c as usize];
    match c {
        0 | 11 | 14 | 17 | 19 | 21 => switch(t, def),
        1 => value(t, 0, 31, def),
        _ => value(t, 0, 127, def),
    }
}

/// The 909's controllers 0..29 (in RBS order; the engine's block permutes 20..22).
const fn tr909(c: u8) -> Spec {
    let t = Target::Ctrl(Device::Tr909, c);
    let def = orb_engine::tr909::DEFAULT_PARAMS[tr909::internal_index(c as usize)];
    match c {
        0 => switch(t, def),
        1 => value(t, 0, 31, def),
        _ => value(t, 0, 127, def),
    }
}

/// The effects' controllers.
const fn effect(d: Device, c: u8) -> Spec {
    let t = Target::Ctrl(d, c);
    match d {
        Device::Delay => {
            let def = orb_engine::delay::DEFAULTS[c as usize];
            match c {
                0 | 2 => switch(t, def),
                1 => value(t, 1, 32, def),
                _ => value(t, 0, 127, def),
            }
        }
        Device::Distortion => {
            let def = orb_engine::dist::DEFAULTS[c as usize];
            match c {
                0 => switch(t, def),
                _ => value(t, 0, 127, def),
            }
        }
        Device::Pcf => {
            let def = orb_engine::pcf::DEFAULTS[c as usize];
            match c {
                0 => switch(t, def),
                4 => value(t, 0, 54, def),
                6 => Spec {
                    target: t,
                    kind: Kind::PcfMode,
                    min: 0,
                    max: 1,
                    default: def,
                },
                _ => value(t, 0, 127, def),
            }
        }
        _ => {
            let def = orb_engine::comp::DEFAULTS[c as usize];
            match c {
                0 => switch(t, def),
                _ => value(t, 0, 127, def),
            }
        }
    }
}

const fn build() -> [Spec; COUNT] {
    // index 0 the shuffle, the song constructor's value (the sequencer's ctor state); the fill
    // is overwritten by every other entry below
    let mut s = [value(Target::Shuffle, 0, 127, 64); COUNT];
    let mut i = 1;
    // the mixer: the constructor's values (`sub_10019CC0`, `sub_10019D50`)
    s[i] = value(Target::Master, 0, 127, 100);
    s[i + 1] = value(Target::CompId, 0, 5, 0);
    s[i + 2] = value(Target::PcfId, 0, 4, 0);
    i += 3;
    let mut c = 0;
    while c < 4 {
        s[i] = switch(Target::Channel(c, 0), 1);
        s[i + 1] = value(Target::Channel(c, 1), 0, 127, 100);
        s[i + 2] = value(Target::Channel(c, 2), 0, 127, 64);
        s[i + 3] = value(Target::Channel(c, 3), 0, 127, 0);
        s[i + 4] = switch(Target::Channel(c, 4), 0);
        i += 5;
        c += 1;
    }
    let mut c = 0;
    while c <= 8 {
        s[i] = tb303(Device::Tb303_1, c);
        i += 1;
        c += 1;
    }
    let mut c = 0;
    while c <= 8 {
        s[i] = tb303(Device::Tb303_2, c);
        i += 1;
        c += 1;
    }
    let mut c = 0;
    while c <= 28 {
        s[i] = tr808(c);
        i += 1;
        c += 1;
    }
    let mut c = 0;
    while c <= 29 {
        s[i] = tr909(c);
        i += 1;
        c += 1;
    }
    let effects = [
        (Device::Delay, 5),
        (Device::Distortion, 3),
        (Device::Pcf, 7),
        (Device::Compressor, 3),
    ];
    let mut e = 0;
    while e < effects.len() {
        let (d, n) = effects[e];
        let mut c = 0;
        while c < n {
            s[i] = effect(d, c);
            i += 1;
            c += 1;
        }
        e += 1;
    }
    assert!(i == COUNT);
    s
}

/// Every parameter, in the host's display order.
pub const SPECS: [Spec; COUNT] = build();

/// The parameter's stable id: `g20` the shuffle (its song msg 35 id; `g1`, the mode's until
/// task-217, is not used), `d<device>c<controller>` the others, with the device's id
/// (`orb_engine::song::Device`) and for the mixer the byte's offset in its msg-41 block (0 master,
/// 1 compressor, 2 PCF, `5 + 6c + k` channel c's field k), which is the controller number where the
/// byte has one.
pub fn id(t: Target) -> String {
    match t {
        Target::Shuffle => "g20".into(),
        Target::Master => "d0c0".into(),
        Target::CompId => "d0c1".into(),
        Target::PcfId => "d0c2".into(),
        Target::Channel(c, k) => format!("d0c{}", 5 + 6 * c + k),
        Target::Ctrl(d, c) => format!("d{}c{c}", d.id()),
    }
}

/// The host's group of a parameter.
pub fn group(t: Target) -> &'static str {
    match t {
        Target::Shuffle => "Transport",
        Target::Master | Target::CompId | Target::PcfId | Target::Channel(..) => "Mixer",
        Target::Ctrl(d, _) => device_name(d),
    }
}

fn device_name(d: Device) -> &'static str {
    match d {
        Device::Mixer => "Mixer",
        Device::Tb303_1 => "303-1",
        Device::Tb303_2 => "303-2",
        Device::Tr808 => "808",
        Device::Tr909 => "909",
        Device::Delay => "Delay",
        Device::Distortion => "Distortion",
        Device::Pcf => "PCF",
        Device::Compressor => "Compressor",
    }
}

const TB303: [&str; 9] = [
    "On",
    "Pattern",
    "Tune",
    "Cutoff",
    "Resonance",
    "Env Mod",
    "Decay",
    "Accent",
    "Waveform",
];

const TR808: [&str; 29] = [
    "On",
    "Pattern",
    "Accent",
    "BD Level",
    "BD Tone",
    "BD Decay",
    "SD Level",
    "SD Tone",
    "SD Snappy",
    "LT Level",
    "LT Tuning",
    "LT/LC Switch",
    "MT Level",
    "MT Tuning",
    "MT/MC Switch",
    "HT Level",
    "HT Tuning",
    "HT/HC Switch",
    "RS Level",
    "RS/CL Switch",
    "CP Level",
    "CP/MA Switch",
    "CB Level",
    "CY Level",
    "CY Tone",
    "CY Decay",
    "OH Level",
    "OH Decay",
    "CH Level",
];

const TR909: [&str; 30] = [
    "On",
    "Pattern",
    "Accent",
    "BD Level",
    "BD Tune",
    "BD Attack",
    "BD Decay",
    "SD Level",
    "SD Tune",
    "SD Tone",
    "SD Snappy",
    "LT Level",
    "LT Tune",
    "LT Decay",
    "MT Level",
    "MT Tune",
    "MT Decay",
    "HT Level",
    "HT Tune",
    "HT Decay",
    "RS Level",
    "CP Level",
    "HH Level",
    "CH Decay",
    "OH Decay",
    "Crash Level",
    "Crash Tune",
    "Ride Level",
    "Ride Tune",
    "Flam",
];

/// The parameter's name, with its device: "303-1 Cutoff".
pub fn name(t: Target) -> String {
    let ctrl = |d: Device, c: u8| -> &'static str {
        let c = c as usize;
        match d {
            Device::Tb303_1 | Device::Tb303_2 => TB303[c],
            Device::Tr808 => TR808[c],
            Device::Tr909 => TR909[c],
            Device::Delay => ["On", "Steps", "Sixteenths", "Feedback", "Pan"][c],
            Device::Distortion => ["On", "Amount", "Shape"][c],
            Device::Pcf => [
                "On",
                "Frequency",
                "Resonance",
                "Amount",
                "Wave",
                "Decay",
                "Low-Pass",
            ][c],
            Device::Compressor => ["On", "Ratio", "Threshold"][c],
            Device::Mixer => "",
        }
    };
    match t {
        Target::Shuffle => "Shuffle".into(),
        Target::Master => "Master Level".into(),
        Target::CompId => "Compressor Insert".into(),
        Target::PcfId => "PCF Insert".into(),
        Target::Channel(c, k) => {
            let f = ["On", "Level", "Pan", "Delay Send", "Distortion"][k as usize];
            format!("Channel {} {f}", c + 1)
        }
        Target::Ctrl(d, c) => format!("{} {}", device_name(d), ctrl(d, c)),
    }
}

/// The value's text in the host: the switches' positions by name, the patterns as A1..D8, the
/// inserts by place; the plain number otherwise.
pub fn display(s: &Spec, v: i32) -> String {
    let two = |off: &str, on: &str| if v != 0 { on.into() } else { off.into() };
    match s.target {
        Target::CompId => match v {
            0 => "Off".into(),
            1 => "Master".into(),
            n => format!("Channel {}", n - 1),
        },
        Target::PcfId => match v {
            0 => "Off".into(),
            n => format!("Channel {n}"),
        },
        Target::Ctrl(Device::Tb303_1 | Device::Tb303_2, 8) => two("Saw", "Square"),
        Target::Ctrl(Device::Tr808, 11 | 14 | 17) => two("Conga", "Tom"),
        Target::Ctrl(Device::Tr808, 19) => two("Claves", "Rim Shot"),
        Target::Ctrl(Device::Tr808, 21) => two("Maracas", "Clap"),
        Target::Ctrl(Device::Delay, 2) => two("Triplets", "Sixteenths"),
        Target::Ctrl(Device::Pcf, 6) => two("Band-Pass", "Low-Pass"),
        Target::Ctrl(Device::Tb303_1 | Device::Tb303_2 | Device::Tr808 | Device::Tr909, 1) => {
            format!("{}{}", (b'A' + (v / 8) as u8) as char, v % 8 + 1)
        }
        _ if s.kind != Kind::Value => two("Off", "On"),
        _ => v.to_string(),
    }
}

/// The value of a parameter's text ([`display`]'s, or a number).
pub fn parse(s: &Spec, text: &str) -> Option<i32> {
    let t = text.trim();
    if let Ok(n) = t.parse::<i32>() {
        return Some(n.clamp(s.min as i32, s.max as i32));
    }
    (s.min as i32..=s.max as i32).find(|&v| display(s, v).eq_ignore_ascii_case(t))
}

/// The engine's byte for a parameter in `e`: the live value the panel shows (`None` for a
/// drum machine without samples, which has no live block).
pub fn engine_byte(e: &Engine, t: Target) -> Option<u8> {
    let d = &e.frame.devices;
    let m = &d.mixer.live;
    Some(match t {
        Target::Shuffle => e.frame.seq.globals.shuffle,
        Target::Master => m.master,
        Target::CompId => m.comp_id,
        Target::PcfId => m.pcf_id,
        Target::Channel(c, k) => {
            let ch = &m.channels[c as usize];
            [ch.enabled, ch.level, ch.pan, ch.send, ch.dist][k as usize]
        }
        Target::Ctrl(dev, c) => {
            let c = c as usize;
            let (block, i) = match dev {
                Device::Tb303_1 | Device::Tb303_2 | Device::Tr808 => {
                    (d.channels[dev as usize - 1].live(), c)
                }
                Device::Tr909 => (d.channels[3].live(), tr909::internal_index(c)),
                Device::Delay => (d.delay.live(), c),
                Device::Distortion => (d.dist.live(), c),
                Device::Pcf => (d.pcf.live(), c),
                Device::Compressor => (d.comp.live(), c),
                Device::Mixer => return None,
            };
            *block.get(i)?
        }
    })
}

/// The parameter value that stands for the engine's byte `b`: the switch position it plays,
/// or the byte within the range.
pub fn value_of(s: &Spec, b: u8) -> i32 {
    match s.kind {
        Kind::Switch => (b != 0) as i32,
        Kind::PcfMode => (b == 1) as i32,
        Kind::Value => b.clamp(s.min, s.max) as i32,
    }
}

/// The parameter a panel message sets and its value (the editor, task-201): the index in
/// [`SPECS`] and [`value_of`] the message's byte. `None` for a message that is no parameter's.
pub fn of_command(c: &Command) -> Option<(usize, i32)> {
    let (target, b) = match *c {
        Command::Shuffle(v) => (Target::Shuffle, v),
        Command::MixerGlobal { id: 0, value } => (Target::Master, value),
        Command::MixerGlobal { id: 1, value } => (Target::CompId, value),
        Command::MixerGlobal { id: 2, value } => (Target::PcfId, value),
        Command::MixerChannel {
            channel,
            field,
            value,
        } => (Target::Channel(channel, field), value),
        Command::DeviceParam {
            device,
            ctrl,
            value,
        } => (Target::Ctrl(device, ctrl), value),
        _ => return None,
    };
    let i = SPECS.iter().position(|s| s.target == target)?;
    Some((i, value_of(&SPECS[i], b)))
}

/// The panel's message for a parameter value, as a player command.
pub fn command(s: &Spec, v: i32) -> Option<Command> {
    let value = v.clamp(s.min as i32, s.max as i32) as u8;
    Some(match s.target {
        Target::Shuffle => Command::Shuffle(value),
        Target::Master => Command::MixerGlobal { id: 0, value },
        Target::CompId => Command::MixerGlobal { id: 1, value },
        Target::PcfId => Command::MixerGlobal { id: 2, value },
        Target::Channel(channel, field) => Command::MixerChannel {
            channel,
            field,
            value,
        },
        Target::Ctrl(device, ctrl) => Command::DeviceParam {
            device,
            ctrl,
            value,
        },
    })
}
