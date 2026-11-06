//! The physical units of the knobs and faders (task-192, A11Y-14, decision A11Y-D5): what each
//! 0..127 value does in the engine, said the way a musician wants to hear it ("1.24 kilohertz",
//! "350 milliseconds", "minus 6.5 decibels").
//!
//! The engine reproduces the original bit for bit, so each mapping is computed in f64 from the
//! engine's own tables and public functions (`orb_engine::tables`, `seq::shuffled`,
//! `pcf::smooth_cutoff`, `comp::params`, `Tb303::note_on` and `process`), not from copies. A few
//! scalar constants the engine keeps private are restated here with the original's addresses;
//! the tests check each one against the engine's behavior. Every mapping, its formula, its
//! source and how exact it is ([C], [D] or [G]) is listed in docs/control_units.md, which a test
//! keeps in step with this file and with `assets/ui/layout.tsv`.

use std::collections::HashMap;
use std::f64::consts::PI;
use std::sync::{LazyLock, Mutex};

use orb_engine::frame::{BLOCK, BlockContext, ChannelDevice};
use orb_engine::tables::Tables;
use orb_engine::tables::data::{PCF_DECAY, TB303_TUNE, TR808_DECAY, TR808_TONE, TR909_BD_ATTACK};
use orb_engine::tb303::{self, Tb303};
use orb_engine::{comp, pcf, seq};

/// The engine's sample rate.
const RATE: f64 = 44100.0;
/// The level a decay time is measured to: 60 dB below the start.
const DOWN_60: f64 = 0.001;

/// The 303's cutoff knob: `cut = v * CUT_SCALE + CUT_BASE` (`dbl_10153B80`/`dbl_10153BE8`,
/// `dbl_10153B88`/`dbl_10153BF0`; `Tb303::smooth_cutoff`, 0x10023F73 and 0x100248E7;
/// orb-engine `tb303.rs:89` `CUT_SCALE`, `:91` `CUT_BASE`).
pub const TB303_CUT_SCALE: f64 = f64::from_bits(0x3F62_B4C5_B45E_7F8E);
pub const TB303_CUT_BASE: f64 = f64::from_bits(0x3FDD_70A3_D70A_3D71);
/// The 303's envelope mod knob: `em = v * EM_SCALE + EM_BASE` (`dbl_10153B90`/`dbl_10153BF8`,
/// `dbl_10153B98`/`dbl_10153C00`; `tb303.rs:93` `EM_SCALE`, `:95` `EM_BASE`).
pub const TB303_EM_SCALE: f64 = f64::from_bits(0x3F68_82F1_67EE_957D);
pub const TB303_EM_BASE: f64 = f64::from_bits(0x3FB4_7AE1_47AE_147B);
/// The envelope value at which the envelope mod adds nothing to the cutoff
/// (`dbl_10153BB0`/`dbl_10153C20`, 0.4: `c = (e - 0.4)*em + cut`; `tb303.rs:101`).
pub const TB303_ENV_BIAS: f64 = 0.4;
/// The 2.0 engine's resonance row scale (`flt_10153C3C`) and the rows per unit
/// (`flt_10153A70`, `sub_10023800`; `tb303.rs:109` `RESO_ROW`, `:111` `ROWS`).
const TB303_RESO_ROW: f32 = f32::from_bits(0x3C00_FEB7);
const TB303_ROWS: f64 = 7.0;
/// The cutoff index scale (`flt_10153BC8`, `flt_10153A6C`; `tb303.rs:116` `C_SCALE`).
const TB303_INDEX: f64 = 1023.0;
/// The span of the 303's filter table: `pow(1602, i/1023)` (0x100234BC), so one unit of the
/// cutoff value moves the frequency by `log2(1602)` octaves (`tables/mod.rs:174` `FILT_SPAN`).
const TB303_SPAN: f64 = 1602.0;
/// The drum accent step `flt_10153F48` (808) and `flt_10154368` (909): `accent = AC*acc*0.03125
/// + 1`; the 808's AC column holds 1, the 909's 2 (docs/song_load.md 4.7, 4.8; `tr808.rs:104`
/// and `tr909.rs:127` `ACC_STEP`).
pub const DRUM_ACCENT_STEP: f64 = 0.03125;
/// The 808's open hi-hat multiplier per pair (`mov esi, 3F7F7CEEh` at 0x10027E4F) and the 909
/// snare's noise multiplier (0x1002DC6B), both f32 0.998 (`tr808.rs:118` `OH_DECAY_M`,
/// `tr909.rs:160` `SD_DECAY_M`).
pub const DRUM_NOISE_DECAY: f32 = f32::from_bits(0x3F7F_7CEE);
/// The 909 snare's hold: `tone + 30` blocks (`add ebx, 1Eh` at 0x1002DD1F; `tr909.rs:162`).
pub const TR909_SD_HOLD: u32 = 30;
/// The 909 flam divisor (`mov ebx, 0AAE60h` at 0x1002ABBC): `flamTick = tick +
/// (flam*tempo)/700000 + 1` (`tr909.rs:135` `FLAM_DIV`).
pub const TR909_FLAM_DIV: i32 = 700_000;
/// The master target `2*T[master] + 1e-6` (`flt_10151F40`, mixer update 0x10012C14;
/// `frame/mod.rs:72` `MASTER_SCALE`).
pub const MASTER_SCALE: f64 = 2.0;
/// A drum voice's gain multiplier applies once per pair of samples; the PCF's envelope once per
/// group of four; the 303's 2.0 envelope once per 64-sample block, the Vintage one per four.
const PAIR: f64 = 2.0;
const GROUP: f64 = 4.0;
/// Ticks per sixteenth (192 per quarter note, docs/initial_investigation.md section 5).
const TICKS_16TH: i32 = 48;

static TABLES: LazyLock<Tables> = LazyLock::new(Tables::new);

fn tables() -> &'static Tables {
    &TABLES
}

/// How exact a mapping is (docs/control_units.md): `C` closed form from the engine's own
/// formula, `D` derived numerically from its tables or by running it, `G` estimated.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mark {
    C,
    D,
    G,
}

impl Mark {
    /// The mark as the document writes it.
    pub fn as_str(self) -> &'static str {
        match self {
            Mark::C => "[C]",
            Mark::D => "[D]",
            Mark::G => "[G]",
        }
    }
}

/// The other values some mappings depend on: the 303 cutoff and accent on the synth's
/// resonance and the sound model, the PCF frequency on its Q fader, the compressor's ratio on
/// its threshold, the flam on the tempo.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Context {
    /// The 303's resonance (of the synth whose knob is read).
    pub reso: u8,
    /// The song's "Vintage ReBirth Synth Sound" (`song+0x16D`).
    pub vintage: bool,
    /// The PCF's Q fader.
    pub pcf_q: u8,
    /// The compressor's threshold knob.
    pub comp_threshold: u8,
    /// The tempo, BPM * 1000.
    pub tempo: i32,
}

impl Default for Context {
    /// The engine's constructor values: resonance 90 (`sub_10018C20`), PCF Q 64
    /// (`sub_10019C10`), threshold 64 (`sub_10019B90`), the 2.0 sound, 140 BPM (the song ctor).
    fn default() -> Self {
        Self {
            reso: tb303::DEFAULT_PARAMS[tb303::RESONANCE],
            vintage: false,
            pcf_q: pcf::DEFAULTS[pcf::RESONANCE],
            comp_threshold: comp::DEFAULTS[comp::THRESHOLD],
            tempo: 140_000,
        }
    }
}

/// The mapping of a knob or fader.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    Shuffle,
    Percent,
    Tune303,
    Cutoff303,
    EnvMod303,
    Decay303,
    Accent303,
    /// A drum's accent level; the AC column's value (808 1, 909 2).
    AccentDrum(u8),
    /// A drum level with the gain `~ L^2`; the effective level `L` of the knob value.
    Level(Eff),
    Tone808,
    Decay808,
    OhDecay808,
    Snappy808,
    Tuning808,
    /// A knob that switches among `n` samples: the sample is `(v*n) >> 7` (5 snare samples)
    /// or `v >> 5` (4 bass drums), both `v*n/128` truncated.
    Sample(u8),
    Attack909,
    Decay909,
    ChDecay909,
    SdTone909,
    Snappy909,
    Tune909,
    Flam,
    Pan,
    Send,
    ChannelLevel,
    Master,
    PcfFreq,
    PcfDecay,
    Feedback,
    DistShape,
    CompRatio,
    CompThreshold,
}

/// The effective level `L` a drum level knob gives.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Eff {
    /// The knob's value.
    Same,
    /// The 808's open hi-hat, `lvl >> 1` (0x10027DDB).
    Half,
    /// The 808's closed hi-hat, `(2*lvl)/3` (0x10027E90).
    TwoThirds,
}

impl Eff {
    pub fn level(self, v: u8) -> u8 {
        match self {
            Eff::Same => v,
            Eff::Half => v >> 1,
            Eff::TwoThirds => ((2 * u32::from(v)) / 3) as u8,
        }
    }
}

/// The mapping of the layout id `id`; None for an id that is not a knob or fader.
fn kind(id: &str) -> Option<Kind> {
    let (sec, local) = id.split_once('.')?;
    // `tb303_1` -> `tb303`, `mixer2` -> `mixer`
    let family = if sec.starts_with("tb303_") {
        "tb303"
    } else if sec.starts_with("mixer") {
        "mixer"
    } else {
        sec
    };
    use Kind::*;
    Some(match (family, local) {
        ("transport", "shuffle") => Shuffle,
        ("tb303", "tune") => Tune303,
        ("tb303", "cutoff") => Cutoff303,
        ("tb303", "reso") => Percent,
        ("tb303", "envmod") => EnvMod303,
        ("tb303", "decay") => Decay303,
        ("tb303", "accent") => Accent303,
        ("tr808", "ac_level") => AccentDrum(1),
        ("tr808", "oh_level") => Level(Eff::Half),
        ("tr808", "ch_level") => Level(Eff::TwoThirds),
        ("tr808", "bd_tone" | "cy_tone") => Tone808,
        ("tr808", "bd_decay" | "cy_decay") => Decay808,
        ("tr808", "oh_decay") => OhDecay808,
        ("tr808", "sd_tone") => Sample(5),
        ("tr808", "sd_snappy") => Snappy808,
        ("tr808", "lt_tuning" | "mt_tuning" | "ht_tuning") => Tuning808,
        ("tr909", "ac_level") => AccentDrum(2),
        ("tr909", "bd_tune") => Sample(4),
        ("tr909", "bd_attack") => Attack909,
        ("tr909", "bd_decay" | "lt_decay" | "mt_decay" | "ht_decay" | "oh_decay") => Decay909,
        ("tr909", "ch_decay") => ChDecay909,
        ("tr909", "sd_tune") => Sample(5),
        ("tr909", "sd_tone") => SdTone909,
        ("tr909", "sd_snappy") => Snappy909,
        ("tr909", "lt_tune" | "mt_tune" | "ht_tune" | "cc_tune" | "rc_tune") => Tune909,
        ("tr909", "flam") => Flam,
        ("tr808" | "tr909", l) if l.ends_with("_level") => Level(Eff::Same),
        ("mixer", "pan") | ("delay", "pan") => Pan,
        ("mixer", "delay") => Send,
        ("mixer", "level") => ChannelLevel,
        ("master", "fader") => Master,
        ("pcf", "freq") => PcfFreq,
        ("pcf", "q" | "amt") | ("dist", "amount") => Percent,
        ("pcf", "dec") => PcfDecay,
        ("delay", "feedback") => Feedback,
        ("dist", "shape") => DistShape,
        ("comp", "ratio") => CompRatio,
        ("comp", "threshold") => CompThreshold,
        _ => return None,
    })
}

/// How exact the mapping of `id` is; None for an id that is not a knob or fader.
pub fn mark(id: &str) -> Option<Mark> {
    Some(match kind(id)? {
        Kind::Cutoff303 | Kind::Accent303 | Kind::PcfFreq => Mark::D,
        _ => Mark::C,
    })
}

/// The spoken value of the knob or fader `id` at the value `v` (0..127) in the context `ctx`;
/// None for an id that is not a knob or fader.
pub fn text(id: &str, v: u8, ctx: &Context) -> Option<String> {
    use Kind::*;
    Some(match kind(id)? {
        Shuffle => format!("{} percent of a sixteenth", sig3(shuffle_percent(v))),
        Percent => percent(v),
        Tune303 => semitones(f64::from(tb303_tune(v)) * 100.0),
        Cutoff303 => hz(tb303_cutoff_hz(v, ctx.reso, ctx.vintage)),
        EnvMod303 => octaves(tb303_envmod_octaves(v)),
        Decay303 => decay(tb303_decay_s(v, ctx.vintage)),
        Accent303 => db(tb303_accent_db(v, ctx.reso, ctx.vintage)),
        AccentDrum(ac) => db(drum_accent_db(v, ac)),
        Level(eff) => level(drum_level_db(eff, v)),
        Tone808 => tone(tr808_tone_hz(v)),
        Decay808 => decay(tr808_decay_s(v)),
        OhDecay808 => decay(Some(tr808_oh_decay_s(v))),
        Snappy808 => level(tr808_snappy_db(v)),
        Tuning808 => semitones(tr808_tuning_cents(v)),
        Sample(n) => {
            let k = u32::from(v) * u32::from(n) / 128 + 1;
            format!("sample {k} of {n}")
        }
        Attack909 => tone(tr909_attack_hz(v)),
        Decay909 => decay(tr909_decay_s(v)),
        ChDecay909 => decay(tr909_ch_decay_s(v)),
        SdTone909 => decay(Some(tr909_sd_tone_s(v))),
        Snappy909 => level(tr909_snappy_db(v)),
        Tune909 => semitones(tr909_tune_cents(v)),
        Flam => time(tr909_flam_s(v, ctx.tempo)),
        Pan => pan(v),
        Send => match send_db(v) {
            Some(d) => db(d),
            None => "off".into(),
        },
        ChannelLevel => level(channel_level_db(v)),
        Master => level(master_db(v)),
        PcfFreq => hz(pcf_freq_hz(v, ctx.pcf_q)),
        PcfDecay => decay(pcf_decay_s(v)),
        Feedback => match feedback_db(v) {
            Some(d) => format!("{} per repeat", db(d)),
            None => "no repeats".into(),
        },
        DistShape if v == 0 => "0 percent, hard clip".into(),
        DistShape => percent(v),
        CompRatio => format!("{} to 1", sig2(comp_ratio(v, ctx.comp_threshold))),
        CompThreshold => db(comp_threshold_db(v)),
    })
}

// ------------------------------------------------------------------ the numbers

/// The pole frequency in Hz of a 2-pole section `y = c0*y1 - c1*y2 + ...`: the poles are
/// `r*e^(+-i*theta)` with `r = sqrt(c1)` and `cos(theta) = c0 / (2r)`; the tables build
/// `c0 = 2*exp(-pi*w/d)*cos(2*pi*w)` and `c1 = exp(-2*pi*w/d)`, so `theta = 2*pi*w`.
pub fn pole_hz(c0: f32, c1: f32) -> f64 {
    let r = f64::from(c1).sqrt();
    let cos = (f64::from(c0) / (2.0 * r)).clamp(-1.0, 1.0);
    cos.acos() / (2.0 * PI) * RATE
}

/// The 303's cutoff value of the knob, `v*CUT_SCALE + CUT_BASE` (the smoother's steady state).
pub fn tb303_cut(v: u8) -> f64 {
    f64::from(v) * TB303_CUT_SCALE + TB303_CUT_BASE
}

/// The 303's envelope mod depth, `v*EM_SCALE + EM_BASE`.
pub fn tb303_em(v: u8) -> f64 {
    f64::from(v) * TB303_EM_SCALE + TB303_EM_BASE
}

/// The 303's filter coefficients `(c0, c1)` at the cutoff value `c` and resonance `reso`, as the
/// sound engine looks them up: the 2.0 engine interpolates between the rows `trunc(rf*7)` and
/// the next (`sub_10023800`), the Vintage engine takes the row `reso >> 4` (0x10023F10).
pub fn tb303_coefs(c: f64, reso: u8, vintage: bool) -> (f32, f32) {
    let t = tables();
    let c = c.clamp(0.0, 1.0);
    if vintage {
        let idx = (c * TB303_INDEX) as usize;
        let e = t.tb303_filter[usize::from(reso >> 4)][idx];
        return (e[0], e[1]);
    }
    let idx = (f64::from(c as f32) * TB303_INDEX) as usize;
    let r7 = f64::from(reso) * f64::from(TB303_RESO_ROW) * TB303_ROWS;
    let l = r7 as usize;
    let frac = r7 - l as f64;
    let (lo, hi) = (t.tb303_filter[l][idx], t.tb303_filter[l + 1][idx]);
    let at = |k: usize| (f64::from(lo[k]) + (f64::from(hi[k]) - f64::from(lo[k])) * frac) as f32;
    (at(0), at(1))
}

/// The 303's cutoff in Hz: the pole frequency of the filter at the knob's cutoff value with the
/// envelope at its neutral point 0.4 (where the envelope mod adds nothing) and no accent.
pub fn tb303_cutoff_hz(v: u8, reso: u8, vintage: bool) -> f64 {
    let (c0, c1) = tb303_coefs(tb303_cut(v), reso, vintage);
    pole_hz(c0, c1)
}

/// The 303's tune in semitones (`word_10033F90[tune >> 2]`, note-on 0x10023CD4).
pub fn tb303_tune(v: u8) -> i16 {
    TB303_TUNE[usize::from(v >> 2)]
}

/// The span of the envelope's sweep in octaves: the envelope moves the cutoff value from
/// `-0.4*em` (decayed) to `+0.6*em` (at a note's start), and the table's frequency is
/// `1602^(c)` times a constant, so the span is `em*log2(1602)` octaves (before the clamp).
pub fn tb303_envmod_octaves(v: u8) -> f64 {
    tb303_em(v) * TB303_SPAN.log2()
}

/// The time a 303 envelope takes to fall 60 dB after an unaccented note, from the engine's
/// note-on (0x10023CD4): the multiplier per 64-sample block (2.0) or per 4 samples (Vintage).
pub fn tb303_decay_s(v: u8, vintage: bool) -> Option<f64> {
    let mut s = Tb303::new();
    s.vintage = vintage;
    s.live[tb303::DECAY] = v;
    s.note_on([0, 0, 0, 0, 0, 0]);
    let per = if vintage { GROUP } else { BLOCK as f64 };
    fall_60_s(s.env_decay, per)
}

/// The peak gain an accented 303 note gets, `1 + acc` at the accent envelope's peak, in dB;
/// measured by running the engine's voice (note-on, then the audio slot, block by block) and
/// remembered per (value, resonance, model).
pub fn tb303_accent_db(v: u8, reso: u8, vintage: bool) -> f64 {
    type Seen = Mutex<HashMap<(u8, u8, bool), f64>>;
    static SEEN: LazyLock<Seen> = LazyLock::new(Default::default);
    if let Some(&d) = SEEN.lock().unwrap().get(&(v, reso, vintage)) {
        return d;
    }
    let mut s = Tb303::new();
    s.vintage = vintage;
    s.live[tb303::ACCENT] = v;
    s.live[tb303::RESONANCE] = reso;
    s.note_on([0, 0, 1, 0, 0, 0]);
    let ctx = BlockContext {
        tables: tables(),
        meter_index: 0,
        tempo: 140_000,
        tick: 0,
        stack_word: None,
    };
    let mut bus = [0.0f32; BLOCK];
    let mut peak = 0.0f64;
    for _ in 0..4000 {
        s.process(&mut bus, &ctx);
        let a = f64::from(s.acc_env);
        if a > peak {
            peak = a;
        } else if a < peak * 0.5 {
            break;
        }
    }
    let d = 20.0 * (1.0 + peak).log10();
    SEEN.lock().unwrap().insert((v, reso, vintage), d);
    d
}

/// The gain of an accented drum step over an unaccented one, `accent = AC*acc*0.03125 + 1`
/// (808 0x10027821, 909 0x1002ACAF), in dB.
pub fn drum_accent_db(v: u8, ac: u8) -> f64 {
    let a = (f64::from(ac) * f64::from(v)) * DRUM_ACCENT_STEP + 1.0;
    20.0 * f64::from(a as f32).log10()
}

/// A drum level in dB relative to the knob's top: every drum's gain is `L^2` times constants
/// (docs/initial_investigation.md 7.3), with `L` the effective level of `v`. None for silence.
pub fn drum_level_db(eff: Eff, v: u8) -> Option<f64> {
    let (l, top) = (eff.level(v), eff.level(127));
    (l > 0).then(|| 40.0 * (f64::from(l) / f64::from(top)).log10())
}

/// The -3 dB frequency of a one-pole low-pass `y = c*y + g*x`: `|H(w)|^2/|H(0)|^2 = 1/2` at
/// `cos(w) = (1 + c^2 - 2(1 - c)^2) / (2c)`. None when there is none below half the sample
/// rate (the filter is open).
pub fn one_pole_hz(c: f32) -> Option<f64> {
    let c = f64::from(c);
    if c <= 0.0 {
        return None;
    }
    let cos = (1.0 + c * c - 2.0 * (1.0 - c) * (1.0 - c)) / (2.0 * c);
    (cos >= -1.0).then(|| cos.min(1.0).acos() / (2.0 * PI) * RATE)
}

/// The 808 bass drum's and cymbal's tone, `TONE8[tone >> 4]` (0x1002A171).
pub fn tr808_tone_hz(v: u8) -> Option<f64> {
    one_pole_hz(TR808_TONE[usize::from(v >> 4)])
}

/// The time to fall 60 dB with the multiplier `m` per `per` samples; None when `m` is 1 (no
/// decay: the voice ends with its sample).
pub fn fall_60_s(m: f32, per: f64) -> Option<f64> {
    let m = f64::from(m);
    (m < 1.0).then(|| DOWN_60.ln() / m.ln() * per / RATE)
}

/// The 808 bass drum's and cymbal's decay, `DEC16[decay >> 3]` per pair (0x1002A18A).
pub fn tr808_decay_s(v: u8) -> Option<f64> {
    fall_60_s(TR808_DECAY[usize::from(v >> 3)], PAIR)
}

/// The 808 open hi-hat: `4*decay` blocks held, then 0.998 per pair (0x10027DDB).
pub fn tr808_oh_decay_s(v: u8) -> f64 {
    let hold = 4.0 * f64::from(v) * BLOCK as f64 / RATE;
    hold + fall_60_s(DRUM_NOISE_DECAY, PAIR).unwrap_or(0.0)
}

/// The 808 snare's noise gain, `(snappy*k)^2` (`sub_1002A1E4`), in dB.
pub fn tr808_snappy_db(v: u8) -> Option<f64> {
    drum_level_db(Eff::Same, v)
}

/// Cents from a tom step table entry (256 = the sample's own pitch).
fn step_cents(step: u16) -> f64 {
    1200.0 * (f64::from(step) / 256.0).log2()
}

/// The 808 toms' and congas' tuning, `T808[tune]` (`sub_1002A004`), in cents.
pub fn tr808_tuning_cents(v: u8) -> f64 {
    step_cents(tables().tr808_tom_step[usize::from(v & 127)])
}

/// The 909 bass drum's attack, `C8[attack >> 4]` (`sub_1002DB24`, 0x1002DBBC).
pub fn tr909_attack_hz(v: u8) -> Option<f64> {
    one_pole_hz(TR909_BD_ATTACK[(usize::from(v) << 3) >> 7])
}

/// `DT[(d << 4) >> 7]` (`Tr909::dt`).
fn tr909_dt(v: u8) -> f32 {
    tables().tr909_decay[(usize::from(v) << 4) >> 7]
}

/// The 909 decays (bass drum, toms, open hi-hat): `DT[d >> 3]` per pair.
pub fn tr909_decay_s(v: u8) -> Option<f64> {
    fall_60_s(tr909_dt(v), PAIR)
}

/// The 909 closed hi-hat: `f32(DT[d >> 3]^8)` per pair (0x1002D935).
pub fn tr909_ch_decay_s(v: u8) -> Option<f64> {
    let x = f64::from(tr909_dt(v));
    let x2 = x * x;
    let x4 = x2 * x2;
    fall_60_s((x4 * x4) as f32, PAIR)
}

/// The 909 snare's noise: `tone + 30` blocks held, then 0.998 per pair (`sub_1002DC54`).
pub fn tr909_sd_tone_s(v: u8) -> f64 {
    let hold = f64::from(u32::from(v) + TR909_SD_HOLD) * BLOCK as f64 / RATE;
    hold + fall_60_s(DRUM_NOISE_DECAY, PAIR).unwrap_or(0.0)
}

/// The 909 snare's noise gain, `snappy*0.01` (`sub_1002DC54`), in dB relative to the top.
pub fn tr909_snappy_db(v: u8) -> Option<f64> {
    (v > 0).then(|| 20.0 * (f64::from(v) / 127.0).log10())
}

/// The 909 toms' and cymbals' tuning, `T909[tune]` (`sub_1002DA24`), in cents.
pub fn tr909_tune_cents(v: u8) -> f64 {
    step_cents(tables().tr909_tom_step[usize::from(v & 127)])
}

/// The 909 flam: the retrigger waits for the first block whose tick passes `tick +
/// (flam*tempo)/700000 + 1` (0x1002ABBC), so it comes `(flam*tempo)/700000 + 2` ticks after the
/// step (to one 64-sample block).
pub fn tr909_flam_s(v: u8, tempo: i32) -> f64 {
    let ticks = i32::from(v).wrapping_mul(tempo) / TR909_FLAM_DIV + 2;
    f64::from(ticks) * 60_000.0 / (f64::from(tempo) * 192.0)
}

/// The delay of the off-beat sixteenths in percent of a sixteenth: the first tick of the
/// second sixteenth of an eighth whose shuffled tick (`seq::shuffled`, 0x1001F3D0) reaches it.
pub fn shuffle_percent(v: u8) -> f64 {
    let late = (TICKS_16TH..2 * TICKS_16TH)
        .find(|&t| seq::shuffled(t, v) >= TICKS_16TH)
        .map_or(TICKS_16TH, |t| t - TICKS_16TH);
    f64::from(late) * 100.0 / f64::from(TICKS_16TH)
}

/// `T[v]` in dB (the mixer's level curve, `sub_100046B0`); None for 0.
fn curve_db(v: u8) -> Option<f64> {
    let g = f64::from(tables().level[usize::from(v)]);
    (g > 0.0).then(|| 20.0 * g.log10())
}

/// A channel's send to the delay, `T[send]` (2.0 sound; mixer update 0x10012C14).
pub fn send_db(v: u8) -> Option<f64> {
    curve_db(v)
}

/// A channel's level fader, `T[level]` (`PAN[lev][64]` at the center).
pub fn channel_level_db(v: u8) -> Option<f64> {
    curve_db(v)
}

/// The master fader, `2*T[master]`.
pub fn master_db(v: u8) -> Option<f64> {
    curve_db(v).map(|d| d + 20.0 * MASTER_SCALE.log10())
}

/// The PCF's cutoff base for the frequency fader: the steady state of the cutoff smoother
/// (`pcf::smooth_cutoff`, stored as f32 each block), for every value.
fn pcf_bases() -> &'static [f32; 128] {
    static BASES: LazyLock<[f32; 128]> = LazyLock::new(|| {
        let k = &tables().pcf_smoother;
        std::array::from_fn(|v| {
            let (mut y1, mut y2) = (0.0f32, 0.0f32);
            for _ in 0..3000 {
                let s = pcf::smooth_cutoff(k, v as u8, y1, y2);
                y2 = y1;
                y1 = s as f32;
            }
            y1
        })
    });
    &BASES
}

/// The PCF's frequency: the pole frequency of its table at the cutoff base (the envelope at
/// rest), in the row `(q*8) >> 7` of the Q fader. The low-pass and band-pass tables have the
/// same poles.
pub fn pcf_freq_hz(v: u8, q: u8) -> f64 {
    let idx = pcf::env_index(f64::from(pcf_bases()[usize::from(v & 127)]));
    let e = tables().pcf_lowpass[(usize::from(q) * 8) >> 7][idx];
    pole_hz(e[0], e[1])
}

/// The PCF envelope's decay, `dec = PCF_DECAY[decay >> 3]` per group of 4 samples: the time
/// the trigger level E falls 60 dB.
pub fn pcf_decay_s(v: u8) -> Option<f64> {
    fall_60_s(PCF_DECAY[usize::from(v >> 3)], GROUP)
}

/// The delay's feedback per repeat, `fb = f32(v * (1/127))` (update 0x1000F754); None for 0.
pub fn feedback_db(v: u8) -> Option<f64> {
    (v > 0).then(|| 20.0 * f64::from((f64::from(v) * (1.0 / 127.0)) as f32).log10())
}

/// The compressor's ratio: the slope of its static curve just above the knee. The gain is
/// `X/env + off` with the detector `env` held at least at `floor` (`comp::params`), so the
/// output level is `X + off*env` above the knee: the ratio there is `1 + X/(off*floor)`.
pub fn comp_ratio(v: u8, threshold: u8) -> f64 {
    let (x, off, floor) = comp::params(v, threshold);
    1.0 + f64::from(x) / (f64::from(off) * f64::from(floor))
}

/// The compressor's threshold: the knee, where the detector reaches `floor`, in dB of full
/// scale. The detector is `|x1 + x3|` of a group of 4, twice the level for a signal well below
/// 11 kHz, so the knee level is `floor/2`.
pub fn comp_threshold_db(v: u8) -> f64 {
    let (_, _, floor) = comp::params(0, v);
    20.0 * (f64::from(floor) / 2.0).log10()
}

// ------------------------------------------------------------------ the words

/// `x` rounded to `n` significant digits.
fn round_sig(x: f64, n: i32) -> f64 {
    if x == 0.0 || !x.is_finite() {
        return x;
    }
    let p = n - 1 - x.abs().log10().floor() as i32;
    let k = 10f64.powi(p);
    (x * k).round() / k
}

/// A number already rounded, written with at most `decimals` decimals and no trailing zeros.
fn plain(x: f64, decimals: usize) -> String {
    let s = format!("{x:.decimals$}");
    if s.contains('.') {
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    } else {
        s
    }
}

/// `x` to `n` significant digits, without trailing zeros.
fn sig(x: f64, n: i32) -> String {
    let r = round_sig(x, n);
    let decimals = if r == 0.0 {
        0
    } else {
        (n - 1 - r.abs().log10().floor() as i32).max(0) as usize
    };
    plain(r, decimals)
}

fn sig3(x: f64) -> String {
    sig(x, 3)
}

fn sig2(x: f64) -> String {
    sig(x, 2)
}

/// "196 hertz", "1.24 kilohertz".
fn hz(f: f64) -> String {
    let r = round_sig(f, 3);
    if r >= 1000.0 {
        format!("{} kilohertz", sig3(r / 1000.0))
    } else {
        format!("{} hertz", sig3(r))
    }
}

/// "350 milliseconds", "1.25 seconds".
fn time(s: f64) -> String {
    let ms = round_sig(s * 1000.0, 3);
    if ms >= 1000.0 {
        let t = sig3(s);
        format!("{t} {}", if t == "1" { "second" } else { "seconds" })
    } else {
        let t = sig3(ms);
        format!(
            "{t} {}",
            if t == "1" {
                "millisecond"
            } else {
                "milliseconds"
            }
        )
    }
}

fn decay(s: Option<f64>) -> String {
    s.map_or_else(|| "no decay".into(), time)
}

fn tone(f: Option<f64>) -> String {
    f.map_or_else(|| "open".into(), hz)
}

/// "minus 6.5 decibels", "plus 13.9 decibels", "0 decibels" (to 0.1 dB).
fn db(d: f64) -> String {
    let r = (d * 10.0).round() / 10.0;
    if r == 0.0 {
        return "0 decibels".into();
    }
    let sign = if r < 0.0 { "minus" } else { "plus" };
    format!("{sign} {} decibels", plain(r.abs(), 1))
}

/// A level: its dB, or "silent".
fn level(d: Option<f64>) -> String {
    d.map_or_else(|| "silent".into(), db)
}

fn percent(v: u8) -> String {
    format!("{} percent", (f64::from(v) * 100.0 / 127.0).round())
}

/// "2.35 octaves".
fn octaves(o: f64) -> String {
    let t = sig3(o);
    format!("{t} {}", if t == "1" { "octave" } else { "octaves" })
}

/// "plus 3 semitones 25 cents", "minus 50 cents", "0 semitones".
fn semitones(cents: f64) -> String {
    let c = cents.round() as i64;
    if c == 0 {
        return "0 semitones".into();
    }
    let sign = if c < 0 { "minus" } else { "plus" };
    let (s, k) = (c.abs() / 100, c.abs() % 100);
    let mut words = vec![sign.to_string()];
    if s > 0 {
        words.push(format!(
            "{s} {}",
            if s == 1 { "semitone" } else { "semitones" }
        ));
    }
    if k > 0 {
        words.push(format!("{k} {}", if k == 1 { "cent" } else { "cents" }));
    }
    words.join(" ")
}

/// Pan as the percent of the knob's travel to each side: "center", "left 31", "right 100" (the
/// middle sends 64; 0 is full left, 127 full right).
fn pan(v: u8) -> String {
    let d = i32::from(v) - 64;
    match d {
        0 => "center".into(),
        d if d < 0 => format!("left {}", (f64::from(-d) * 100.0 / 64.0).round()),
        d => format!("right {}", (f64::from(d) * 100.0 / 63.0).round()),
    }
}

#[cfg(test)]
mod tests;
