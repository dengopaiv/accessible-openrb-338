//! Note mode (task-200): General MIDI drum notes trigger the instruments as a step would, at
//! the velocity band's step value and AC, and the sounding mask.

use super::*;
use crate::frame::BLOCK;
use crate::samples::{COUNT, FIRST_ID};

/// A sample set whose resource `id` holds `frames` frames of the value `id - 1000` (the 909
/// kit's slot number).
fn set(frames: usize) -> Arc<SampleSet> {
    Arc::new(SampleSet::from_samples(
        (0..COUNT)
            .map(|i| {
                let id = FIRST_ID as usize + i;
                Sample::from_audio(&vec![id as i16 - 1000; frames])
            })
            .collect(),
    ))
}

fn slot_of(s: &Option<Arc<Sample>>) -> i16 {
    s.as_ref().unwrap().audio()[0]
}

fn device(frames: usize) -> Tr909 {
    let mut d = Tr909::new(set(frames), ProcessState::new());
    d.relocate(0, 0);
    d.start(0);
    d
}

/// Every voice's gain, multiplier, hold, activity and sample slot.
fn voices(d: &Tr909) -> Vec<(u32, u32, u32, bool, i16)> {
    let v = |v: &crate::drums::Voice| {
        let s = v.sample.as_ref().map_or(-1, |_| slot_of(&v.sample));
        (v.g.to_bits(), v.m.to_bits(), v.hold, v.active, s)
    };
    let mut out: Vec<_> = d.simple.iter().map(|x| v(&x.v)).collect();
    out.extend(d.pitched.iter().map(|x| v(&x.v)));
    out.extend(d.filter.iter().map(|x| v(&x.v)));
    out.push(v(&d.dual.v));
    out.push((d.dual.g2.to_bits(), 0, 0, false, 0));
    out
}

/// The notes of `docs/midi.md` section 5.4 and their columns.
const NOTES: [(u8, usize); 11] = [
    (36, BD),
    (37, RS),
    (38, SD),
    (39, CP),
    (42, CH),
    (43, LT),
    (46, OH),
    (47, MT),
    (49, CC),
    (50, HT),
    (51, RC),
];

#[test]
fn each_gm_note_plays_as_a_step_of_its_band() {
    for (note, col) in NOTES {
        for (velocity, value, ac) in [
            (1, 1, 0),
            (47, 1, 0),
            (48, 2, 0),
            (95, 2, 0),
            (96, 1, 2),
            (111, 1, 2),
            (112, 2, 2),
            (127, 2, 2),
        ] {
            assert_eq!(velocity_band(velocity), (value, ac));
            let mut a = device(100);
            a.live[ACCENT] = 45;
            let pat = &mut a.patterns[..PATTERN_LEN];
            pat[2 + col] = value;
            pat[2 + AC] = ac;
            a.step(0, 0, 120_000);
            let mut b = device(100);
            b.live[ACCENT] = 45;
            b.key_on(note, velocity);
            let what = format!("note {note} velocity {velocity}");
            assert_eq!((b.accent, b.gain), (a.accent, a.gain), "{what}");
            assert_eq!(voices(&b), voices(&a), "{what}");
            // a note arms no flam
            assert!(!b.flam_pending);
            let s = b.sounding();
            assert_eq!(s.mask, 1 << (col - 1), "{what}");
            assert_eq!(s.ac, if ac != 0 { s.mask } else { 0 }, "{what}");
        }
    }
}

#[test]
fn the_band_gains_at_the_default_accent() {
    // knob 32, AC = 2: accent 3.0; gains 0.8, 2.0, 2.4, 6.0 (docs/midi.md 5.5)
    let mut d = device(100);
    assert_eq!(d.live[ACCENT], 32);
    for (velocity, gain) in [(30, 0.8f32), (60, 2.0), (100, 2.4), (120, 6.0)] {
        d.key_on(36, velocity);
        assert_eq!(d.gain, gain, "velocity {velocity}");
    }
    // at knob 24 low + AC equals high: f32(2.5 * 0.8) = 2.0
    d.live[ACCENT] = 24;
    d.key_on(36, 100);
    assert_eq!(d.gain, 2.0);
}

#[test]
fn other_notes_do_nothing() {
    let mut d = device(100);
    let before = voices(&d);
    for note in (0..=127).filter(|n| gm_note(*n).is_none()) {
        d.key_on(note, 127);
        d.live_note_off(note);
    }
    assert_eq!(voices(&d), before);
    assert_eq!(d.sounding(), Sounding::default());
}

/// Blocks of audio until `sound` stops sounding (at most `max`).
fn sounding_blocks(d: &mut Tr909, sound: u32, max: usize) -> usize {
    let t = crate::tables::Tables::new();
    let ctx = BlockContext {
        tables: &t,
        meter_index: 0,
        tempo: 120_000,
        tick: 0,
        stack_word: None,
    };
    for k in 0..max {
        if d.sounding().mask & 1 << sound == 0 {
            return k;
        }
        let mut bus = [0.0f32; BLOCK];
        d.process(&mut bus, &ctx);
    }
    max
}

#[test]
fn the_closed_hihat_darkens_the_open_one() {
    // OH decay 127: multiplier DT[15] = 1, so it sounds until its sample ends
    let mut d = device(44_100);
    d.key_on(46, 64);
    assert_eq!(sounding_blocks(&mut d, sound::OH, 100), 100);
    // CH chokes it (m 0.996 a pair): below -60 dB long before the sample ends
    let mut d = device(44_100);
    d.key_on(46, 64);
    d.key_on(42, 64);
    let s = d.sounding();
    assert_eq!(s.mask, 1 << sound::OH | 1 << sound::CH);
    let n = sounding_blocks(&mut d, sound::OH, 1000);
    assert!(n > 10 && n < 100, "{n}");
}

#[test]
fn every_hit_is_counted_retriggers_too() {
    let mut d = device(44_100);
    d.key_on(46, 64);
    d.key_on(46, 64);
    d.key_on(42, 64);
    let h = d.sounding().hits;
    assert_eq!((h[sound::OH as usize], h[sound::CH as usize]), (2, 1));
    assert_eq!(h.iter().sum::<u32>(), 3);
}

#[test]
fn either_bass_drum_voice_lights_the_bd() {
    let mut d = device(44_100);
    d.key_on(36, 127);
    d.key_on(36, 30);
    let s = d.sounding();
    assert_eq!((s.mask, s.ac), (1 << sound::BD, 0));
    d.filter[FILTER_BD_B].silence();
    assert_eq!(d.sounding().mask, 1 << sound::BD);
    d.filter[FILTER_BD_A].silence();
    assert_eq!(d.sounding().mask, 0);
}
