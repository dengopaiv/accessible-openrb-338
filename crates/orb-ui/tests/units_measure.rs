//! The spoken units checked against renders of the engine (task-192, A11Y-14, AC 2): the 303's
//! cutoff against the resonance peak of a rendered note, and the 808 bass drum's decay against
//! the fall of a rendered hit. docs/control_units.md states the tolerances.
//!
//! Both render a device of `orb-engine` through its public API (the device, its note-on or step
//! function and its audio slot), as the engine's frame does.

use std::f64::consts::PI;
use std::sync::Arc;

use orb_engine::frame::{BLOCK, BlockContext, ChannelDevice, SongDevice};
use orb_engine::process::ProcessState;
use orb_engine::samples::{COUNT, Sample, SampleSet};
use orb_engine::tables::Tables;
use orb_engine::tb303::{self, Tb303};
use orb_engine::tr808::{self, Tr808};
use orb_ui::a11y::units;

const RATE: f64 = 44100.0;

fn ctx(t: &Tables) -> BlockContext<'_> {
    BlockContext {
        tables: t,
        meter_index: 0,
        tempo: 140_000,
        tick: 0,
        stack_word: None,
    }
}

/// A 303 note (saw, 2.0 sound) at its lowest pitch (tune 0 and octave down: C at 16.35 Hz),
/// with the envelope pinned at the neutral point 0.4 in every block, so the filter stays at the
/// knob's cutoff. Returns `n` samples after it settled, and the note's frequency.
fn render_303(t: &Tables, cutoff: u8, reso: u8, n: usize) -> (Vec<f64>, f64, f32) {
    let mut s = Tb303::new();
    s.live[tb303::TUNE] = 0;
    s.live[tb303::CUTOFF] = cutoff;
    s.live[tb303::RESONANCE] = reso;
    s.live[tb303::ENVMOD] = 0;
    s.live[tb303::ACCENT] = 0;
    s.live[tb303::WAVEFORM] = 0;
    s.note_on([0, 0, 0, 0, 1, 0]);
    s.env_decay = 1.0;
    let c = ctx(t);
    let mut bus = [0.0f32; BLOCK];
    let mut out = Vec::with_capacity(n);
    for b in 0.. {
        // the block adds the 1e-6 offset to the envelope first
        s.env = (0.4 - 1e-6) as f32;
        s.process(&mut bus, &c);
        if b >= 2000 {
            out.extend(bus.iter().map(|&x| f64::from(x)));
            if out.len() >= n {
                break;
            }
        }
    }
    out.truncate(n);
    let f0 = f64::from(t.tb303_pitch[(s.pitch >> 4) as usize]) * RATE / 2.0;
    (out, f0, s.coefs[3])
}

/// The magnitude at `f` Hz of `y` under a Hann window.
fn magnitude(y: &[f64], f: f64) -> f64 {
    let n = y.len() as f64;
    let w = 2.0 * PI * f / RATE;
    let (mut re, mut im) = (0.0, 0.0);
    for (i, &v) in y.iter().enumerate() {
        let i = i as f64;
        let h = 0.5 - 0.5 * (2.0 * PI * i / n).cos();
        re += h * v * (w * i).cos();
        im -= h * v * (w * i).sin();
    }
    (re * re + im * im).sqrt()
}

/// `|1/(1 - a e^-iw)|` and `|(1 - e^-iw)/(1 - a e^-iw)|`: the voice's one-pole pre-filter
/// (`s = c3*s + x`) and its DC blocker (`o = (y - y1) + o*0.985853374`, `flt_10153BD0`).
fn one_pole(a: f64, w: f64) -> f64 {
    1.0 / (1.0 - 2.0 * a * w.cos() + a * a).sqrt()
}

fn dc_blocker(w: f64) -> f64 {
    let a = f64::from(f32::from_bits(0x3F7C_60E3));
    (2.0 - 2.0 * w.cos()).sqrt() * one_pole(a, w)
}

/// The frequency of the 2-pole's resonance peak in a render: the saw's harmonics (amplitude
/// `1/k`) divided out, and the pre-filter and DC blocker, then the largest harmonic and a
/// parabola through the logarithms of it and its neighbors.
fn measured_peak(y: &[f64], f0: f64, c3: f32, around: f64) -> f64 {
    let lo = ((around / 2.0) / f0).floor().max(1.0) as usize;
    let hi = ((around * 2.0) / f0).ceil() as usize;
    let h: Vec<(usize, f64)> = (lo..=hi)
        .map(|k| {
            let f = k as f64 * f0;
            let w = 2.0 * PI * f / RATE;
            let m = magnitude(y, f) * k as f64 / one_pole(f64::from(c3), w) / dc_blocker(w);
            (k, m.ln())
        })
        .collect();
    let i = (1..h.len() - 1)
        .max_by(|&a, &b| h[a].1.total_cmp(&h[b].1))
        .unwrap();
    let (a, b, c) = (h[i - 1].1, h[i].1, h[i + 1].1);
    let d = 0.5 * (a - c) / (a - 2.0 * b + c);
    (h[i].0 as f64 + d) * f0
}

/// The spoken 303 cutoff (the pole frequency of the filter table's entry) against the resonance
/// peak of a render, within 2% (docs/control_units.md).
#[test]
fn the_303_cutoff_matches_the_resonance_peak_of_a_render() {
    let t = Tables::new();
    for (cutoff, reso) in [
        (0u8, 127u8),
        (40, 127),
        (64, 127),
        (100, 127),
        (127, 127),
        (64, 112),
    ] {
        let spoken = units::tb303_cutoff_hz(cutoff, reso, false);
        let (y, f0, c3) = render_303(&t, cutoff, reso, 1 << 16);
        let peak = measured_peak(&y, f0, c3, spoken);
        let err = (peak - spoken).abs() / spoken;
        println!("cutoff {cutoff} reso {reso}: spoken {spoken:.1} Hz, peak {peak:.1} Hz, {err:.4}");
        assert!(
            err < 0.02,
            "cutoff {cutoff} reso {reso}: {spoken} Hz vs {peak} Hz"
        );
    }
}

/// An 808 with a constant bass drum sample (the bass drum is slot 1, resource 1000) that plays
/// one bass drum hit at full level with the tone open (TONE8[7] = 0.001) and the decay `v`;
/// the absolute output, sample by sample.
fn render_808_bd(t: &Tables, v: u8, n: usize) -> Vec<f64> {
    let mut s: Vec<Sample> = vec![Sample::from_audio(&vec![16000; 12 * 44100])];
    s.extend((1..COUNT).map(|_| Sample::from_audio(&[0; 64])));
    let mut d = Tr808::new(Arc::new(SampleSet::from_samples(s)), ProcessState::new());
    d.live[tr808::BD_LEVEL] = 127;
    d.live[tr808::BD_TONE] = 127;
    d.live[tr808::BD_DECAY] = v;
    d.patterns[2 + tr808::BD] = 1;
    d.wrapped = true;
    d.step(0, 0, 140_000);
    let c = ctx(t);
    let mut bus = [0.0f32; BLOCK];
    let mut out = Vec::with_capacity(n);
    while out.len() < n {
        d.process(&mut bus, &c);
        out.extend(bus.iter().map(|&x| f64::from(x).abs()));
    }
    out
}

/// The spoken 808 bass drum decay (the time to fall 60 dB from `DEC16[decay >> 3]` per pair)
/// against a render: the hit falls 40 dB in two thirds of it, to within one 64-sample block.
/// (The render cannot show the 60 dB point itself: a FILTER voice stops when its gain falls
/// below 3e-8, about 56 dB under a full-level bass drum.)
#[test]
fn the_808_bass_drum_decay_matches_a_render() {
    let t = Tables::new();
    for v in [0u8, 40, 64, 100] {
        let spoken = units::tr808_decay_s(v).unwrap();
        let y = render_808_bd(&t, v, (spoken * RATE) as usize + 4 * BLOCK);
        let top = y.iter().cloned().fold(0.0, f64::max);
        let at = y
            .iter()
            .position(|&x| x < top * 0.01)
            .expect("the hit falls 40 dB");
        let measured = at as f64 / RATE * 1.5;
        let err = (measured - spoken).abs();
        println!("decay {v}: spoken {spoken:.4} s, measured {measured:.4} s");
        assert!(
            err <= 1.5 * BLOCK as f64 / RATE,
            "decay {v}: {spoken} s vs {measured} s"
        );
    }
    assert_eq!(units::tr808_decay_s(127), None, "DEC16[15] = 1: no decay");
}
