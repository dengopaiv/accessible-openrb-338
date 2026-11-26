//! Note mode (task-200): host notes through the step machinery, checked against the step path
//! with an equivalent pattern.

use super::*;
use crate::frame::BLOCK;

/// A msg 39 payload with pattern 0 = `steps` (length 16, no shuffle), the rest rests.
fn payload(params: Params, steps: &[[u8; 6]]) -> Vec<u8> {
    let mut b = params.to_vec();
    for p in 0..PATTERNS {
        b.extend([0, 16]);
        for s in 0..16 {
            let st = if p == 0 { steps.get(s).copied() } else { None };
            b.extend(st.unwrap_or([0, 0, 0, 0, 0, 1]));
        }
    }
    b
}

/// A step `{note, slide, accent, up, down, rest}` that plays.
fn step(note: u8, slide: bool, accent: bool, up: bool, down: bool) -> [u8; 6] {
    [note, slide as u8, accent as u8, up as u8, down as u8, 0]
}

/// The ticks per block: a step (48 ticks) is 4 blocks, its gate-off (24) at the third.
const TICKS: i32 = 12;
const BLOCKS_PER_STEP: usize = 4;

/// `blocks` blocks of the step path playing `steps` (from the bar's start) and of note mode
/// with `events` (block, MIDI note, velocity; 0 a note-off), each applied before its block's
/// audio as the sequencer's step is; both at Tune `tune`. Returns both outputs.
fn both(
    steps: &[[u8; 6]],
    events: &[(usize, u8, u8)],
    blocks: usize,
    tune: u8,
    vintage: bool,
) -> (Vec<f32>, Vec<f32>) {
    let t = Tables::new();
    let ctx = BlockContext {
        tables: &t,
        meter_index: 0,
        tempo: 120_000,
        tick: 0,
        stack_word: None,
    };
    let mut params = DEFAULT_PARAMS;
    params[TUNE] = tune;
    let mut a = Tb303::new();
    a.data(&payload(params, steps), false);
    a.set_sound_model(vintage);
    a.relocate(0, 0);
    a.start(0);
    let mut b = Tb303::new();
    b.data(&payload(params, &[]), false);
    b.set_sound_model(vintage);
    let (mut oa, mut ob) = (Vec::new(), Vec::new());
    for k in 0..blocks {
        a.step(k as i32 * TICKS, k as i32 * TICKS, 120_000);
        for &(at, note, velocity) in events {
            if at == k {
                if velocity == 0 {
                    b.live_note_off(note);
                } else {
                    b.live_note_on(note, velocity);
                }
            }
        }
        let mut bus = [0.0f32; BLOCK];
        a.process(&mut bus, &ctx);
        oa.extend(bus);
        let mut bus = [0.0f32; BLOCK];
        b.process(&mut bus, &ctx);
        ob.extend(bus);
    }
    (oa, ob)
}

/// The step path and note mode give the same samples, which are not silence.
fn check(steps: &[[u8; 6]], events: &[(usize, u8, u8)], tune: u8) {
    let blocks = steps.len() * BLOCKS_PER_STEP;
    for vintage in [false, true] {
        let (a, b) = both(steps, events, blocks, tune, vintage);
        assert!(a.iter().any(|&x| x != 0.0));
        let first = a
            .iter()
            .zip(&b)
            .position(|(x, y)| x.to_bits() != y.to_bits());
        assert_eq!(first, None, "vintage {vintage}: first difference");
    }
}

/// The note-on and note-off of each step `(block of the step, MIDI note, velocity)` with the
/// note-off at the step's gate-off (the step's third block).
fn notes(steps: &[(u8, u8)]) -> Vec<(usize, u8, u8)> {
    let mut e = Vec::new();
    for (k, &(note, velocity)) in steps.iter().enumerate() {
        let at = k * BLOCKS_PER_STEP;
        e.push((at, note, velocity));
        e.push((at + 2, note, 0));
    }
    e
}

#[test]
fn pitch_accent_and_gate_as_the_steps() {
    // C, E with accent, G an octave up, C' an octave down: MIDI 36 + note + 12*up - 12*down
    let steps = [
        step(0, false, false, false, false),
        step(4, false, true, false, false),
        step(7, false, false, true, false),
        step(12, false, true, false, true),
    ];
    let events = notes(&[(36, 80), (40, 100), (55, 99), (36, 127)]);
    check(&steps, &events, 64);
    // the Tune knob adds its semitones at the note-on, in both paths
    check(&steps, &events, 0);
    check(&steps, &events, 100);
}

#[test]
fn an_overlapping_note_slides_as_a_sliding_step() {
    // C slides into G (no attack, the glide), G slides down into D#
    let steps = [
        step(0, true, false, false, false),
        step(7, true, true, false, false),
        step(3, false, false, false, false),
    ];
    let events = [
        (0, 36, 64),
        // the next note comes while the held one sounds; the held one is let go after it
        (4, 43, 110),
        (4, 36, 0),
        (8, 39, 64),
        (9, 43, 0),
        (10, 39, 0),
    ];
    check(&steps, &events, 64);
}

#[test]
fn a_same_pitch_overlap_is_a_tie() {
    // C slides into C with accent: one note, the accent changing at the second step
    let steps = [
        step(0, true, false, false, false),
        step(0, false, true, false, false),
    ];
    // the key pressed again while held, then one note-off: a key is held once (task-258)
    let events = [(0, 36, 80), (4, 36, 120), (6, 36, 0)];
    check(&steps, &events, 64);
}

/// The pitch target of MIDI note `m` at Tune center.
fn target(m: i32) -> u32 {
    ((64 * (m - 36) + 3072) * 16) as u32
}

#[test]
fn a_key_is_held_once() {
    let mut d = Tb303::new();
    d.key_on(36, 64);
    d.key_on(40, 64);
    // 36 again: its old entry goes and it is the newest, slid to from 40 (no attack)
    d.retrig = false;
    d.key_on(36, 64);
    assert_eq!(
        (d.held_len, d.retrig, d.pitch_target),
        (2, false, target(36))
    );
    // one note-off lets it go: back to 40
    d.key_off(36);
    assert_eq!((d.held_len, d.pitch_target, d.gate), (1, target(40), 1.0));
    // a second note-off of it is stray: nothing
    d.key_off(36);
    assert_eq!((d.held_len, d.pitch_target, d.gate), (1, target(40), 1.0));
    d.key_off(40);
    assert_eq!((d.held_len, d.gate), (0, 0.0));
}

#[test]
fn a_lost_note_off_heals_at_the_keys_next_note_off() {
    // 36's note-off is lost: 40 slides from it, and 40's release slides back to it
    let mut d = Tb303::new();
    d.key_on(36, 64);
    d.retrig = false;
    d.key_on(40, 64);
    assert!(!d.retrig);
    d.key_off(40);
    assert_eq!((d.held_len, d.gate, d.pitch_target), (1, 1.0, target(36)));
    // 36 played and let go again: nothing is held, the gate closes
    d.key_on(36, 64);
    d.key_off(36);
    assert_eq!((d.held_len, d.gate), (0, 0.0));
    // the next note attacks
    d.key_on(43, 64);
    assert!(d.retrig);
}

#[test]
fn all_off_lets_every_key_go() {
    let mut d = Tb303::new();
    // nothing held: nothing changes
    d.key_all_off();
    assert_eq!((d.held_len, d.gate, d.amp_in), (0, 0.0, 0.0));
    d.key_on(36, 64);
    d.key_on(40, 127);
    let mut e = d.clone();
    d.key_all_off();
    assert_eq!((d.held_len, d.gate, d.slide), (0, 0.0, false));
    // as the note-offs of both keys
    e.key_off(36);
    e.key_off(40);
    assert_eq!(
        (d.gate, d.amp_in, d.amp_fb, d.pitch_target),
        (e.gate, e.amp_in, e.amp_fb, e.pitch_target)
    );
    // the next note attacks
    d.retrig = false;
    d.key_on(43, 64);
    assert!(d.retrig);
}

#[test]
fn the_pitch_of_a_midi_note() {
    let mut d = Tb303::new();
    // MIDI 36 is the pattern's C at Tune center: index 3072, times 16
    d.key_on(36, 64);
    assert_eq!((d.pitch_target, d.pitch, d.gate), (49152, 49152, 1.0));
    d.key_off(36);
    assert_eq!(d.gate, 0.0);
    // A4 (69) is 440 Hz: the phase runs from -1 to 1, so f = tab * 22050
    d.key_on(69, 64);
    let f = Tables::new().tb303_pitch[(d.pitch_target >> 4) as usize] as f64 * 22050.0;
    assert!((f - 440.0).abs() < 1e-3, "{f}");
}

#[test]
fn notes_outside_the_pitch_table_are_ignored() {
    let mut d = Tb303::new();
    // Tune center: MIDI 0..83
    for (note, plays) in [(0, true), (83, true), (84, false), (127, false)] {
        d.key_on(note, 64);
        assert_eq!(d.held_len == 1, plays, "note {note}");
        assert_eq!(d.gate, plays as u8 as f32, "note {note}");
        d.key_off(note);
        assert_eq!((d.held_len, d.gate), (0, 0.0));
    }
    // Tune -12 (knob 0): up to 95; Tune +12 (127): up to 71
    d.live[TUNE] = 0;
    d.key_on(95, 64);
    assert_eq!(d.pitch_target, ((64 * 47 + 3072) * 16) as u32);
    d.key_off(95);
    d.key_on(96, 64);
    assert_eq!(d.held_len, 0);
    d.live[TUNE] = 127;
    d.key_on(72, 64);
    assert_eq!(d.held_len, 0);
    d.key_on(71, 64);
    assert_eq!(d.held_len, 1);
}

#[test]
fn accent_from_velocity_100() {
    let mut d = Tb303::new();
    d.key_on(36, 99);
    assert_eq!(d.acc_amount, 0.0);
    d.key_off(36);
    d.key_on(36, 100);
    assert_eq!(d.acc_amount, st(64.0 * ACC_AMOUNT_SCALE));
}

#[test]
fn last_note_priority_and_legato() {
    let mut d = Tb303::new();
    let target = |m: i32| ((64 * (m - 36) + 3072) * 16) as u32;
    d.key_on(36, 64);
    assert!(d.retrig);
    d.retrig = false;
    d.env = 0.5;
    d.key_on(43, 64);
    d.key_on(48, 127);
    // the newest sounds, slid to: no new attack
    assert_eq!((d.pitch_target, d.retrig, d.env), (target(48), false, 0.5));
    assert_ne!(d.pitch, d.pitch_target);
    // releasing a note that does not sound changes nothing
    d.key_off(43);
    assert_eq!((d.pitch_target, d.held_len), (target(48), 2));
    // releasing the sounding note slides back to the newest still held, with its accent
    d.key_off(48);
    assert_eq!(
        (d.pitch_target, d.acc_amount, d.gate),
        (target(36), 0.0, 1.0)
    );
    assert_eq!((d.env, d.retrig), (0.5, false));
    // the last release closes the gate
    d.key_off(36);
    assert_eq!((d.gate, d.held_len), (0.0, 0));
    // a note-off of a note not held does nothing
    d.key_off(50);
    assert_eq!(d.gate, 0.0);
}

#[test]
fn play_and_silence_forget_the_held_notes() {
    let mut d = Tb303::new();
    d.key_on(36, 64);
    d.start(0);
    assert_eq!(d.held_len, 0);
    d.key_on(36, 64);
    d.silence();
    assert_eq!((d.held_len, d.gate), (0, 0.0));
    // a fresh note after it attacks
    d.retrig = false;
    d.key_on(40, 64);
    assert!(d.retrig);
}
