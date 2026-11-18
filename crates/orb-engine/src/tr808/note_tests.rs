//! Note mode (task-200): General MIDI drum notes trigger the instruments as a step would, and
//! the sounding mask.

use super::*;
use crate::frame::{AUDIBLE, BLOCK};
use crate::samples::{COUNT, FIRST_ID};

/// A sample set whose resource `id` holds `frames` frames of the value `id - 999` (the
/// built-in kit's slot number), so every voice's sample can be told apart.
fn set(frames: usize) -> Arc<SampleSet> {
    Arc::new(SampleSet::from_samples(
        (0..COUNT)
            .map(|i| {
                let id = FIRST_ID as usize + i;
                Sample::from_audio(&vec![(id - 999) as i16; frames])
            })
            .collect(),
    ))
}

fn slot_of(s: &Option<Arc<Sample>>) -> i16 {
    s.as_ref().unwrap().audio()[0]
}

fn device(frames: usize) -> Tr808 {
    let mut d = Tr808::new(set(frames), ProcessState::new());
    d.relocate(0, 0);
    d.start(0);
    d
}

/// Every voice's gain, multiplier, hold, activity and sample slot.
fn voices(d: &Tr808) -> Vec<(u32, u32, u32, bool, i16)> {
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

/// Every General MIDI note of `docs/midi.md` section 5.4: the note, its column, the switch value
/// that makes a step play the same sound, the voice's sample slot.
const NOTES: [(u8, usize, Option<u8>, usize); 16] = [
    (36, BD, None, slot::BD),
    (37, RS, Some(1), slot::RS),
    (38, SD, None, 0),
    (39, CP, Some(1), slot::CP),
    (42, CH, None, slot::CH),
    (43, LT, Some(1), slot::LT),
    (46, OH, None, slot::OH),
    (47, MT, Some(1), slot::MT),
    (49, CY, None, slot::CY),
    (50, HT, Some(1), slot::HT),
    (56, CB, None, slot::CB),
    (62, HT, Some(0), slot::HC),
    (63, MT, Some(0), slot::MC),
    (64, LT, Some(0), slot::LC),
    (70, CP, Some(0), slot::MA),
    (75, RS, Some(0), slot::CL),
];

/// The live byte of the switch of column `c`'s shared slot.
fn switch(c: usize) -> usize {
    match c {
        LT => LT_LEVEL + 2,
        MT => MT_LEVEL + 2,
        HT => HT_LEVEL + 2,
        RS => RS_SELECT,
        _ => CP_SELECT,
    }
}

#[test]
fn each_gm_note_plays_as_a_step_of_its_instrument() {
    for (note, col, sel, sample) in NOTES {
        for (velocity, ac) in [(1, 0), (99, 0), (100, 1), (127, 1)] {
            // the step path: one step with the instrument (and AC), the switch as the note says
            let mut a = device(100);
            a.live[ACCENT] = 77;
            if let Some(s) = sel {
                a.live[switch(col)] = s;
            }
            let pat = &mut a.patterns[..PATTERN_LEN];
            pat[2 + col] = 1;
            pat[2 + AC] = ac;
            a.step(0, 0, 0);
            // note mode, the switch the other way round: the note names the sound
            let mut b = device(100);
            b.live[ACCENT] = 77;
            if let Some(s) = sel {
                b.live[switch(col)] = 1 - s;
            }
            b.key_on(note, velocity);
            assert_eq!(b.accent, a.accent, "note {note} velocity {velocity}");
            assert_eq!(voices(&b), voices(&a), "note {note} velocity {velocity}");
            // the switch is not changed
            if let Some(s) = sel {
                assert_eq!(b.live[switch(col)], 1 - s);
            }
            let bits = b.sounding();
            if col != SD {
                let played = match col {
                    BD => slot_of(&b.filter[FILTER_BD_A].v.sample),
                    CY => slot_of(&b.filter[FILTER_CY].v.sample),
                    LT | MT | HT => slot_of(&b.pitched[col - LT].v.sample),
                    RS => slot_of(&b.simple[SIMPLE_RS].v.sample),
                    CP => slot_of(&b.simple[SIMPLE_CP].v.sample),
                    CB => slot_of(&b.simple[SIMPLE_CB].v.sample),
                    OH => slot_of(&b.simple[SIMPLE_OH].v.sample),
                    _ => slot_of(&b.simple[SIMPLE_CH].v.sample),
                };
                assert_eq!(played, sample as i16, "note {note}");
            }
            assert_eq!(bits.mask.count_ones(), 1, "note {note}");
            assert_eq!(bits.ac, if ac != 0 { bits.mask } else { 0 }, "note {note}");
        }
    }
}

#[test]
fn the_accent_band_gains() {
    // AC at velocity 100 and up: x(1 + knob/32); below it x1
    let mut d = device(100);
    for (knob, velocity, accent) in [
        (40, 99, 1.0f32),
        (40, 100, 2.25),
        (32, 127, 2.0),
        (0, 127, 1.0),
        (127, 127, 4.96875),
    ] {
        d.live[ACCENT] = knob;
        d.key_on(56, velocity);
        assert_eq!(d.accent, accent);
        let l = d.live[CB_LEVEL] as f64;
        let g = st((((K as f64 * l) * l) * LEVEL_SCALE as f64) * accent as f64);
        assert_eq!(d.simple[SIMPLE_CB].v.g, st(g as f64 / 32768.0));
    }
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

#[test]
fn the_bass_drum_alternates_and_the_closed_hihat_chokes() {
    let mut d = device(44_100);
    d.key_on(36, 64);
    d.key_on(36, 64);
    // the two voices, the first fading
    assert!(d.filter[FILTER_BD_A].v.active && d.filter[FILTER_BD_B].v.active);
    assert_eq!(d.filter[FILTER_BD_A].v.m, BD_FADE);
    // OH holds; CH clears its hold
    d.live[OH_DECAY] = 100;
    d.key_on(46, 64);
    assert_eq!(d.simple[SIMPLE_OH].v.hold, 400);
    d.key_on(42, 64);
    assert_eq!(d.simple[SIMPLE_OH].v.hold, 0);
}

/// Blocks of audio until `sound` stops sounding (at most `max`).
fn sounding_blocks(d: &mut Tr808, sound: u32, max: usize) -> usize {
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
fn the_sounding_mask_follows_the_voices() {
    // a short sample: sounding until its end (100 frames: two blocks)
    let mut d = device(100);
    d.key_on(64, 127);
    let s = d.sounding();
    assert_eq!((s.mask, s.ac), (1 << sound::LC, 1 << sound::LC));
    assert_eq!(sounding_blocks(&mut d, sound::LC, 10), 2);
    // the AC flag stays with the sound's last hit; the tom of the same slot takes it
    d.key_on(43, 50);
    let s = d.sounding();
    assert_eq!((s.mask, s.ac), (1 << sound::LT, 1 << sound::LC));
    // a decaying voice: the OH (m 0.998 a pair after its hold) until -60 dB
    let mut d = device(44_100);
    d.live[OH_DECAY] = 0;
    d.key_on(46, 64);
    let n = sounding_blocks(&mut d, sound::OH, 1000);
    assert!(n > 10 && n < 1000, "{n}");
    // -60 dB is about where the voice's own end test (g < 3e-8 after the 2^-15) stops it
    assert!(d.simple[SIMPLE_OH].v.peak() < AUDIBLE);
    assert!(d.simple[SIMPLE_OH].v.peak() > crate::drums::SILENT as f64 * 32768.0 * 0.9);
    // either bass drum voice lights the BD
    let mut d = device(44_100);
    d.key_on(36, 127);
    d.key_on(36, 64);
    d.filter[FILTER_BD_B].silence();
    let s = d.sounding();
    assert_eq!((s.mask, s.ac), (1 << sound::BD, 0));
    // a level of 0 does not sound
    let mut d = device(44_100);
    d.live[CB_LEVEL] = 0;
    d.key_on(56, 64);
    assert!(d.simple[SIMPLE_CB].v.active);
    assert_eq!(d.sounding().mask, 0);
}

#[test]
fn every_hit_is_counted_retriggers_too() {
    let mut d = device(100);
    assert_eq!(d.sounding().hits, [0; 16]);
    d.key_on(64, 127);
    d.key_on(64, 127);
    d.key_on(36, 64);
    // the voice ends, the count stays: a reader that missed the mask still sees the hits
    assert_eq!(sounding_blocks(&mut d, sound::LC, 10), 2);
    let h = d.sounding().hits;
    assert_eq!((h[sound::LC as usize], h[sound::BD as usize]), (2, 1));
    assert_eq!(h.iter().sum::<u32>(), 3);
}

#[test]
fn pattern_steps_set_the_mask_and_the_ac_flags_too() {
    let mut d = device(44_100);
    let pat = &mut d.patterns[..PATTERN_LEN];
    pat[2 + AC] = 1;
    pat[2 + RS] = 1;
    pat[2 + BD] = 1;
    d.step(0, 0, 0);
    // RS_SELECT is 0 by default: the claves
    let s = d.sounding();
    assert_eq!(s.mask, 1 << sound::BD | 1 << sound::CL);
    assert_eq!(s.ac, s.mask);
}
