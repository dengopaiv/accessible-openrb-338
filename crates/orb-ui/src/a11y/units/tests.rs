use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use orb_engine::frame::{BLOCK, BlockContext, ChannelDevice, SongDevice};
use orb_engine::process::ProcessState;
use orb_engine::samples::{COUNT, Sample, SampleSet};
use orb_engine::tb303::Tb303;
use orb_engine::tr808::{self, Tr808};
use orb_engine::tr909::{self, Tr909};

use super::*;
use crate::layout;

/// docs/control_units.md.
const DOC: &str = include_str!("../../../../../docs/control_units.md");

/// The values the document gives the spoken text of.
const SAMPLES: [u8; 5] = [0, 32, 64, 96, 127];

/// The rows of the table under the heading `heading` (cells trimmed, header and rule left out).
fn table(heading: &str) -> Vec<Vec<String>> {
    let mut rows = Vec::new();
    let mut inside = false;
    for line in DOC.lines() {
        if line.starts_with("## ") {
            inside = line.trim() == heading;
            continue;
        }
        if inside && line.starts_with('|') {
            let cells: Vec<String> = line
                .trim()
                .trim_matches('|')
                .split('|')
                .map(|c| c.trim().to_string())
                .collect();
            rows.push(cells);
        }
    }
    assert!(rows.len() > 2, "no table under {heading:?}");
    rows.drain(..2);
    rows
}

/// The `backticked` words of a cell.
fn ids(cell: &str) -> Vec<String> {
    cell.split('`')
        .skip(1)
        .step_by(2)
        .map(str::to_string)
        .collect()
}

/// The knobs and faders of `layout.tsv`.
fn layout_ids() -> BTreeSet<String> {
    layout::controls()
        .into_iter()
        .filter(|c| c.kind == "knob" || c.kind == "fader")
        .map(|c| c.id)
        .collect()
}

#[test]
fn the_document_lists_every_knob_and_fader_with_its_mark() {
    let mut listed = BTreeMap::new();
    for row in table("## The mappings") {
        let mark = row.last().unwrap().clone();
        for id in ids(&row[0]) {
            assert!(
                listed.insert(id.clone(), mark.clone()).is_none(),
                "{id} twice"
            );
        }
    }
    let want = layout_ids();
    assert_eq!(want.len(), 86, "77 knobs and 9 faders");
    let got: BTreeSet<String> = listed.keys().cloned().collect();
    assert_eq!(got, want, "docs/control_units.md and layout.tsv");
    for (id, m) in &listed {
        let code = mark(id).unwrap_or_else(|| panic!("{id} has no mapping"));
        assert_eq!(m, code.as_str(), "{id}: the document's mark and the code's");
    }
}

#[test]
fn the_documented_sample_values_are_spoken() {
    let ctx = Context::default();
    let mut kinds = BTreeSet::new();
    for row in table("## Sample values") {
        let id = &ids(&row[0])[0];
        kinds.insert(format!("{:?}", kind(id).unwrap()));
        for (k, v) in SAMPLES.iter().enumerate() {
            let want = &row[1 + k];
            let got = text(id, *v, &ctx).unwrap();
            assert_eq!(&got, want, "{id} at {v}");
        }
    }
    // every mapping has its sample row
    let all: BTreeSet<String> = layout_ids()
        .iter()
        .map(|id| format!("{:?}", kind(id).unwrap()))
        .collect();
    assert_eq!(kinds, all, "a sample row per mapping");
}

/// Prints the sample table for docs/control_units.md:
/// `cargo test -p orb-ui --lib units::tests::print_sample_table -- --ignored --nocapture`.
#[test]
#[ignore]
fn print_sample_table() {
    let ctx = Context::default();
    let mut seen = BTreeSet::new();
    for c in layout::controls() {
        if !(c.kind == "knob" || c.kind == "fader") {
            continue;
        }
        if !seen.insert(format!("{:?}", kind(&c.id).unwrap())) {
            continue;
        }
        let cells: Vec<String> = SAMPLES
            .iter()
            .map(|v| text(&c.id, *v, &ctx).unwrap())
            .collect();
        println!("| `{}` | {} |", c.id, cells.join(" | "));
    }
}

#[test]
fn the_words() {
    assert_eq!(hz(196.04), "196 hertz");
    assert_eq!(hz(1237.0), "1.24 kilohertz");
    assert_eq!(hz(999.7), "1 kilohertz");
    assert_eq!(hz(45.26), "45.3 hertz");
    assert_eq!(time(0.35), "350 milliseconds");
    assert_eq!(time(0.005_21), "5.21 milliseconds");
    assert_eq!(time(15.66), "15.7 seconds");
    assert_eq!(time(1.0), "1 second");
    assert_eq!(db(-6.54), "minus 6.5 decibels");
    assert_eq!(db(13.94), "plus 13.9 decibels");
    assert_eq!(db(-6.0), "minus 6 decibels");
    assert_eq!(db(0.04), "0 decibels");
    assert_eq!(semitones(325.0), "plus 3 semitones 25 cents");
    assert_eq!(semitones(-50.0), "minus 50 cents");
    assert_eq!(semitones(-1200.0), "minus 12 semitones");
    assert_eq!(semitones(100.0), "plus 1 semitone");
    assert_eq!(semitones(0.2), "0 semitones");
    assert_eq!(pan(64), "center");
    assert_eq!(pan(0), "left 100");
    assert_eq!(pan(44), "left 31");
    assert_eq!(pan(127), "right 100");
    assert_eq!(percent(64), "50 percent");
    assert_eq!(sig2(89.44), "89");
    assert_eq!(sig2(1.894), "1.9");
}

// ------------------------------------------------------------------ the restated constants

fn ctx(t: &Tables) -> BlockContext<'_> {
    BlockContext {
        tables: t,
        meter_index: 0,
        tempo: 140_000,
        tick: 0,
        stack_word: None,
    }
}

/// The index of the 303's filter table row 0 whose entry is `(c0, c1)`.
fn row0_index(c0: f32, c1: f32) -> usize {
    tables().tb303_filter[0]
        .iter()
        .position(|e| e[0] == c0 && e[1] == c1)
        .expect("a row 0 entry")
}

/// The 303 at resonance 0 (one table row, no interpolation) with the envelope pinned so that
/// it is `e` in every block; the cutoff index its 2.0 engine looks up after it settled.
fn tb303_index(cutoff: u8, envmod: u8, e: f64) -> usize {
    let t = tables();
    let mut s = Tb303::new();
    s.live[tb303::CUTOFF] = cutoff;
    s.live[tb303::ENVMOD] = envmod;
    s.live[tb303::RESONANCE] = 0;
    s.note_on([0, 0, 0, 0, 0, 0]);
    s.env_decay = 1.0;
    let mut bus = [0.0f32; BLOCK];
    for _ in 0..1500 {
        // the block adds the 1e-6 offset `flt_10153C30` to the envelope first
        s.env = (e - 1e-6) as f32;
        s.process(&mut bus, &ctx(t));
    }
    row0_index(s.coefs[0], s.coefs[1])
}

#[test]
fn the_303_cutoff_and_envmod_constants_are_the_engines() {
    for v in [0u8, 20, 64, 101, 127] {
        let idx = tb303_index(v, 0, TB303_ENV_BIAS);
        let want = (f64::from(tb303_cut(v) as f32) * TB303_INDEX) as usize;
        assert!(
            idx.abs_diff(want) <= 1,
            "cutoff {v}: engine {idx}, here {want}"
        );
    }
    for em in [0u8, 50, 127] {
        let lo = tb303_index(30, em, TB303_ENV_BIAS);
        let hi = tb303_index(30, em, 1.0);
        let measured = (hi - lo) as f64 / TB303_INDEX / (1.0 - TB303_ENV_BIAS);
        let d = (measured - tb303_em(em)).abs();
        assert!(
            d < 3.0 / TB303_INDEX,
            "envmod {em}: engine {measured}, here {}",
            tb303_em(em)
        );
    }
}

fn samples() -> Arc<SampleSet> {
    let s = (0..COUNT)
        .map(|_| Sample::from_audio(&[1000; 256]))
        .collect();
    Arc::new(SampleSet::from_samples(s))
}

/// An 808 that plays step 0 of pattern 0 with the columns `cols` set.
fn tr808_step(live: &[(usize, u8)], cols: &[usize]) -> Tr808 {
    let mut d = Tr808::new(samples(), ProcessState::new());
    for &(k, v) in live {
        d.live[k] = v;
    }
    for &c in cols {
        d.patterns[2 + c] = 1;
    }
    d.wrapped = true;
    d.step(0, 0, 140_000);
    d
}

fn tr909_step(live: &[(usize, u8)], cols: &[(usize, u8)], tempo: i32) -> Tr909 {
    let mut d = Tr909::new(samples(), ProcessState::new());
    for &(k, v) in live {
        d.live[k] = v;
    }
    for &(c, v) in cols {
        d.patterns[2 + c] = v;
    }
    d.wrapped = true;
    d.step(0, 0, tempo);
    d
}

#[test]
fn the_drum_accents_are_the_engines() {
    for v in [0u8, 40, 127] {
        let d = tr808_step(&[(tr808::ACCENT, v)], &[tr808::AC]);
        let a = 1.0 + f64::from(v) * DRUM_ACCENT_STEP;
        assert_eq!(d.accent, a as f32, "808 {v}");
        assert!((drum_accent_db(v, 1) - 20.0 * f64::from(d.accent).log10()).abs() < 1e-9);
        let d = tr909_step(&[(tr909::ACCENT, v)], &[(tr909::AC, 2)], 140_000);
        assert!((drum_accent_db(v, 2) - 20.0 * f64::from(d.accent).log10()).abs() < 1e-9);
    }
}

#[test]
fn the_drum_holds_and_multipliers_are_the_engines() {
    let d = tr808_step(&[(tr808::OH_DECAY, 37)], &[tr808::OH]);
    // the 808's SIMPLE voices: RS/CL, CP/MA, CB, OH, CH
    assert_eq!(d.simple[3].v.m, DRUM_NOISE_DECAY);
    assert_eq!(d.simple[3].v.hold, 4 * 37);
    let d = tr909_step(&[(tr909::SD_TONE, 37)], &[(tr909::SD, 1)], 140_000);
    assert_eq!(d.dual.v.m, DRUM_NOISE_DECAY);
    assert_eq!(d.dual.v.hold, 37 + TR909_SD_HOLD);
}

#[test]
fn the_flam_divisor_is_the_engines() {
    for (v, tempo) in [(0u8, 140_000), (64, 120_000), (127, 87_500), (100, 300_000)] {
        let d = tr909_step(&[(tr909::FLAM, v)], &[(tr909::BD, 3)], tempo);
        let ticks = tr909_flam_s(v, tempo) * f64::from(tempo) * 192.0 / 60_000.0;
        // the flam fires at the first tick above `flam_tick`
        assert_eq!(ticks.round() as i32, d.flam_tick + 1, "flam {v} at {tempo}");
    }
}

#[test]
fn drum_gains_go_with_the_square_of_the_effective_level() {
    // the 808's bass drum (FILTER voice A first), open and closed hi-hat
    let gain = |k: usize, v: u8, col: usize, voice: fn(&Tr808) -> f32| {
        voice(&tr808_step(&[(k, v)], &[col]))
    };
    let bd = |d: &Tr808| d.filter[0].v.g;
    let oh = |d: &Tr808| d.simple[3].v.g;
    let ch = |d: &Tr808| d.simple[4].v.g;
    for v in [10u8, 64, 100] {
        for (k, col, voice, eff) in [
            (
                tr808::BD_LEVEL,
                tr808::BD,
                bd as fn(&Tr808) -> f32,
                Eff::Same,
            ),
            (tr808::OH_LEVEL, tr808::OH, oh, Eff::Half),
            (tr808::CH_LEVEL, tr808::CH, ch, Eff::TwoThirds),
        ] {
            let r = f64::from(gain(k, v, col, voice)) / f64::from(gain(k, 127, col, voice));
            let db = 20.0 * r.log10();
            let want = drum_level_db(eff, v).unwrap();
            assert!(
                (db - want).abs() < 1e-4,
                "808 control {k} at {v}: {db} vs {want}"
            );
        }
    }
}

#[test]
fn some_numbers() {
    // shuffle: 127/6 = 21 ticks of 48
    assert!((shuffle_percent(127) - 43.75).abs() < 1e-12);
    assert_eq!(shuffle_percent(5), 0.0);
    // the 808 BD decay at 0: 0.999 per pair
    let t = tr808_decay_s(0).unwrap();
    assert!((t - 2.0 * 0.001f64.ln() / f64::from(0.999f32).ln() / 44100.0).abs() < 1e-12);
    assert_eq!(tr808_decay_s(127), None);
    // the 808 tunings span +-4 semitones (T808 = 256*2^((i-64)/192), truncated)
    assert!(tr808_tuning_cents(64).abs() < 1e-9);
    // (truncated steps: T808[0] = trunc(256 * 2^(-1/3) = 203.2) = 203, so -402 cents)
    assert!((tr808_tuning_cents(0) + 400.0).abs() < 3.0);
    // the tones: TONE8[7] = 0.001 has no -3 dB point below 22.05 kHz
    assert_eq!(tr808_tone_hz(127), None);
    // the mixer: T[127] = 1, the master doubles it
    assert_eq!(channel_level_db(127), Some(0.0));
    assert!((master_db(127).unwrap() - 6.0206).abs() < 1e-3);
    assert_eq!(send_db(0), None);
    // the compressor: amount 0 is no compression
    assert!((comp_ratio(0, 64) - 1.0).abs() < 1e-12);
    // the PCF's base settles one f32 step below 4*freq for every frequency 1..127, so its
    // resting index is 4*freq - 1; frequency 0 rests at 0 (docs/control_units.md)
    assert_eq!(pcf::env_index(f64::from(pcf_bases()[0])), 0);
    for v in 1..128usize {
        let b = pcf_bases()[v];
        assert!(b < (4 * v) as f32, "{v}: {b}");
        assert_eq!(pcf::env_index(f64::from(b)), 4 * v - 1, "{v}: {b}");
    }
}

/// The 303's filter table spans `1602^(i/1023)` in `w`: its pole frequencies at two indices of
/// a row are in that ratio (`TB303_SPAN`, behind the envelope mod's octaves).
#[test]
fn the_303_table_spans_1602() {
    let row = &tables().tb303_filter[0];
    for (a, b) in [(400usize, 1000usize), (512, 1023), (300, 700)] {
        let f = |i: usize| pole_hz(row[i][0], row[i][1]);
        let ratio = f(b) / f(a);
        let want = TB303_SPAN.powf((b - a) as f64 / TB303_INDEX);
        assert!(
            (ratio / want - 1.0).abs() < 1e-3,
            "{a}..{b}: {ratio} vs {want}"
        );
    }
}

/// The 2.0 engine's coefficients at a resonance between two table rows are the ones
/// [`tb303_coefs`] interpolates.
#[test]
fn the_303_interpolation_is_the_engines() {
    let t = tables();
    for (cutoff, reso) in [(64u8, 64u8), (100, 100), (20, 37)] {
        let mut s = Tb303::new();
        s.live[tb303::CUTOFF] = cutoff;
        s.live[tb303::ENVMOD] = 0;
        s.live[tb303::RESONANCE] = reso;
        s.note_on([0, 0, 0, 0, 0, 0]);
        s.env_decay = 1.0;
        let mut bus = [0.0f32; BLOCK];
        for _ in 0..1500 {
            s.env = (TB303_ENV_BIAS - 1e-6) as f32;
            s.process(&mut bus, &ctx(t));
        }
        let (c0, c1) = tb303_coefs(tb303_cut(cutoff), reso, false);
        let d = (f64::from(s.coefs[0]) - f64::from(c0)).abs()
            + (f64::from(s.coefs[1]) - f64::from(c1)).abs();
        assert!(
            d < 1e-6,
            "cutoff {cutoff} reso {reso}: {:?} vs {c0} {c1}",
            s.coefs
        );
    }
}

/// The pole frequency from the f32 coefficients against the tables' closed form `w*44100`
/// (`w = K^i * 0.4/K^511 * (0.6 + 0.4*row/7)`, docs/initial_investigation.md 8.2) at the PCF's
/// low end, where `cos(theta)` is closest to 1.
#[test]
fn the_pole_frequency_holds_at_the_low_end() {
    let k = 1.0145453214645386f64;
    let mut worst = (0.0f64, 0usize);
    for row in [0usize, 4, 7] {
        let aw = row as f64 * f64::from(f32::from_bits(0x3E12_4925)) * 0.4 + 0.6;
        for i in (3..512).step_by(4) {
            let w = k.powf(i as f64) * (0.4 / k.powf(511.0)) * aw;
            let e = tables().pcf_lowpass[row][i];
            let err = (pole_hz(e[0], e[1]) / (w * RATE) - 1.0).abs();
            if err > worst.0 {
                worst = (err, i);
            }
            // 1.9 % at index 3 (about 9 Hz), docs/control_units.md
            assert!(err < 0.02, "row {row} index {i}: {err}");
        }
    }
    println!("worst relative error of the PCF's pole frequency: {worst:?}");
}

#[test]
fn every_knob_and_fader_has_a_unit() {
    let ctx = Context::default();
    for id in layout_ids() {
        for v in 0..=127u8 {
            let t = text(&id, v, &ctx).unwrap_or_else(|| panic!("{id}"));
            assert!(
                !t.is_empty() && !t.contains("NaN") && !t.contains("inf"),
                "{id} {v}: {t}"
            );
        }
    }
}
