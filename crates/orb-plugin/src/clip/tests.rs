//! The clip writer (task-203 AC #1): the written file read back and compared with the pattern's
//! steps.

use orb_engine::seq;

use super::*;

/// What a written file holds.
#[derive(Debug)]
struct Parsed {
    format: u16,
    tracks: u16,
    division: u16,
    name: String,
    /// The channel of every note event.
    channels: Vec<u8>,
    notes: Vec<Note>,
    /// The end of track's tick.
    end: u32,
}

fn read_vlq(b: &[u8], i: &mut usize) -> u32 {
    let mut v = 0u32;
    loop {
        let x = b[*i];
        *i += 1;
        v = (v << 7) | (x & 0x7F) as u32;
        if x & 0x80 == 0 {
            return v;
        }
    }
}

/// Reads a type-0 file as [`smf`] writes it (no running status), pairing each note-off with the
/// oldest open note of its key.
fn parse(f: &[u8]) -> Parsed {
    assert_eq!(&f[0..4], b"MThd");
    assert_eq!(u32::from_be_bytes(f[4..8].try_into().unwrap()), 6);
    let format = u16::from_be_bytes([f[8], f[9]]);
    let tracks = u16::from_be_bytes([f[10], f[11]]);
    let division = u16::from_be_bytes([f[12], f[13]]);
    assert_eq!(&f[14..18], b"MTrk");
    let len = u32::from_be_bytes(f[18..22].try_into().unwrap()) as usize;
    let t = &f[22..];
    assert_eq!(t.len(), len, "the track's length");
    let (mut i, mut now) = (0, 0u32);
    let mut name = String::new();
    let mut channels = Vec::new();
    let mut open: Vec<Note> = Vec::new();
    let mut notes = Vec::new();
    loop {
        now += read_vlq(t, &mut i);
        let status = t[i];
        i += 1;
        if status == 0xFF {
            let kind = t[i];
            i += 1;
            let n = read_vlq(t, &mut i) as usize;
            let data = &t[i..i + n];
            i += n;
            match kind {
                0x03 => name = String::from_utf8(data.to_vec()).unwrap(),
                0x58 => assert_eq!(data, [4, 2, 24, 8], "4/4"),
                0x2F => {
                    assert_eq!(i, t.len(), "the end of track is last");
                    break;
                }
                k => panic!("meta event {k:#x}"),
            }
            continue;
        }
        let (key, vel) = (t[i], t[i + 1]);
        i += 2;
        channels.push(status & 15);
        match status & 0xF0 {
            0x90 => {
                assert!(vel > 0, "a note-on has a velocity");
                open.push(Note {
                    start: now,
                    end: 0,
                    key,
                    velocity: vel,
                });
            }
            0x80 => {
                let at = open
                    .iter()
                    .position(|n| n.key == key)
                    .expect("a note-off of an open note");
                let mut n = open.remove(at);
                n.end = now;
                notes.push(n);
            }
            s => panic!("status {s:#x}"),
        }
    }
    assert!(open.is_empty(), "every note ends");
    notes.sort_by_key(|n| (n.start, n.key));
    Parsed {
        format,
        tracks,
        division,
        name,
        channels,
        notes,
        end: now,
    }
}

/// A 303 pattern of `steps` (`{note, slide, accent, up, down, rest}`), its length the steps'.
fn pattern_303(shuffle: bool, steps: &[[u8; 6]]) -> Vec<u8> {
    let mut b = vec![shuffle as u8, steps.len() as u8];
    for k in 0..16 {
        b.extend(steps.get(k).copied().unwrap_or([0, 0, 0, 0, 0, 1]));
    }
    assert_eq!(b.len(), LEN_303);
    b
}

/// A drum pattern of `steps` (12 columns, 0 the AC), 16 steps.
fn pattern_drums(shuffle: bool, steps: &[[u8; 12]]) -> Vec<u8> {
    let mut b = vec![shuffle as u8, 16];
    for k in 0..16 {
        b.extend(steps.get(k).copied().unwrap_or([0; 12]));
    }
    assert_eq!(b.len(), LEN_DRUM);
    b
}

fn n(start: u32, end: u32, key: u8, velocity: u8) -> Note {
    Note {
        start,
        end,
        key,
        velocity,
    }
}

/// `{note, slide, accent, up, down, rest}`.
fn st(note: u8, slide: bool, accent: bool, up: bool, down: bool, rest: bool) -> [u8; 6] {
    [
        note,
        slide as u8,
        accent as u8,
        up as u8,
        down as u8,
        rest as u8,
    ]
}

/// The 303 pattern of the tests: pitch with Up and Down, accent, a slide, a tie, a rest, a rest
/// after a slide and a slide out of the last step.
fn steps_303() -> Vec<[u8; 6]> {
    vec![
        // 0: C, accent: 36, the gate (24 ticks)
        st(0, false, true, false, false, false),
        // 1: D# up, slides into step 2: 36 + 3 + 12
        st(3, true, false, true, false, false),
        // 2: F down, slides into step 3 at the same pitch: a tie
        st(5, true, false, false, true, false),
        // 3: F down again, accent (the tie keeps the first step's velocity), no slide
        st(5, false, true, false, true, false),
        // 4: a rest
        st(9, false, true, false, false, true),
        // 5: G, slides into a rest
        st(7, true, false, false, false, false),
        // 6: a rest after a slide: legato to its pitch A (the gate stays open), accent
        st(9, false, true, false, false, true),
        // 7: C', slides out of the pattern
        st(12, true, false, false, false, false),
    ]
}

#[test]
fn a_303_pattern_pitch_slide_tie_accent_and_gate() {
    let bytes = pattern_303(false, &steps_303());
    let f = parse(&clip(1, 10, &bytes, &Controls::default()));
    assert_eq!((f.format, f.tracks, f.division), (0, 1, 192));
    assert_eq!(f.name, "303-2 B3");
    assert!(f.channels.iter().all(|&c| c == 1), "303-2: channel 2");
    assert_eq!(f.end, 8 * 48, "the clip ends with the pattern's 8 steps");
    assert_eq!(
        f.notes,
        [
            // the gate: half a step
            n(0, 24, 36, 127),
            // the slide: overlaps the next note by OVERLAP
            n(48, 96 + 12, 51, 80),
            // the tie: one note over steps 2 and 3, to step 3's gate
            n(96, 144 + 24, 29, 80),
            // step 4 rests; step 5 slides into the rest of step 6
            n(240, 288 + 12, 43, 80),
            // the rest after a slide: legato at its pitch and accent, to its gate
            n(288, 288 + 24, 45, 127),
            // the slide out of the last step ends with the clip
            n(336, 384, 48, 80),
        ]
    );
}

#[test]
fn a_303_pattern_with_shuffle_moves_as_the_engine() {
    let steps = steps_303();
    for shuffle in [0u8, 6, 64, 127] {
        let c = Controls {
            shuffle,
            ..Controls::default()
        };
        // the pattern's Shuffle off: straight whatever the amount
        let straight = notes(0, &pattern_303(false, &steps), &c);
        assert_eq!(
            straight,
            notes(0, &pattern_303(false, &steps), &Controls::default())
        );
        let s = shuffle as u32 / 6;
        let got = notes(0, &pattern_303(true, &steps), &c);
        let want: Vec<Note> = [
            n(0, 24 + s, 36, 127),
            n(48 + s, 96 + 12, 51, 80),
            n(96, 144 + 24 + s, 29, 80),
            n(240 + s, 288 + 12, 43, 80),
            n(288, 288 + 24 + s, 45, 127),
            n(336 + s, 384, 48, 80),
        ]
        .into();
        assert_eq!(got, want, "shuffle {shuffle}");
    }
}

/// The 303 playing pattern 0 = `bytes` through its step machinery and, in a second 303, the
/// clip's notes through note mode (task-200), 12 ticks a block, each event before its block's
/// audio as the sequencer's step; the two outputs, `blocks` blocks of each.
fn play_303(bytes: &[u8], shuffle: u8, blocks: usize, vintage: bool) -> (Vec<f32>, Vec<f32>) {
    use orb_engine::frame::{BLOCK, BlockContext, ChannelDevice, SongDevice};
    use orb_engine::tables::Tables;
    use orb_engine::tb303::{DEFAULT_PARAMS, PATTERNS, Tb303};
    const TICKS: u32 = 12;
    let t = Tables::new();
    let ctx = BlockContext {
        tables: &t,
        meter_index: 0,
        tempo: 120_000,
        tick: 0,
        stack_word: None,
    };
    let payload = |first: &[u8]| {
        let mut b = DEFAULT_PARAMS.to_vec();
        for p in 0..PATTERNS {
            if p == 0 {
                b.extend(first);
            } else {
                b.extend(pattern_303(false, &[]));
            }
        }
        b
    };
    let mut a = Tb303::new();
    a.data(&payload(bytes), false);
    a.set_sound_model(vintage);
    a.relocate(0, 0);
    a.start(0);
    let mut b = Tb303::new();
    b.data(&payload(&pattern_303(false, &[])), false);
    b.set_sound_model(vintage);
    let c = Controls {
        shuffle,
        ..Controls::default()
    };
    let f = parse(&clip(0, 0, bytes, &c));
    // (tick, 0 note-off / 1 note-on, key, velocity), as the file orders them
    let mut events: Vec<(u32, u8, u8, u8)> = Vec::new();
    for n in &f.notes {
        events.push((n.start, 1, n.key, n.velocity));
        events.push((n.end, 0, n.key, 0));
    }
    events.sort_by_key(|e| (e.0, e.1));
    assert!(
        events.iter().all(|e| e.0 % TICKS == 0),
        "the events fall on block starts"
    );
    let (mut oa, mut ob) = (Vec::new(), Vec::new());
    for k in 0..blocks as u32 {
        let tick = (k * TICKS) as i32;
        a.step(tick, seq::shuffled(tick, shuffle), 120_000);
        for &(_, on, key, vel) in events.iter().filter(|e| e.0 / TICKS == k) {
            if on == 1 {
                b.live_note_on(key, vel);
            } else {
                b.live_note_off(key);
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

/// The 303's clip played back in note mode sounds as this pattern, bit for bit, in both sound
/// models, straight and shuffled (a shuffle of 72..77 moves the steps 12 ticks, a whole block):
/// the gate, the slides, the tie, the rest after a slide. The pattern avoids the three things
/// the clip cannot carry (see [`the_clip_cannot_carry_a_rest_or_a_ties_own_accent`]): the tie's
/// second step has the first one's accent, step 4 plays instead of resting, and no rest slides
/// into a played step with the gate closed.
#[test]
fn the_303_clip_in_note_mode_sounds_as_the_pattern() {
    let mut steps = steps_303();
    steps[3][2] = 0;
    steps[4] = st(2, false, false, false, false, false);
    for (shuffle, flag) in [(0u8, false), (72, true), (77, true)] {
        let bytes = pattern_303(flag, &steps);
        for vintage in [false, true] {
            // the pattern once (the slide out of the last step goes on into the next loop)
            let (a, b) = play_303(&bytes, shuffle, 8 * 4, vintage);
            assert!(a.iter().any(|&x| x != 0.0));
            let first = a
                .iter()
                .zip(&b)
                .position(|(x, y)| x.to_bits() != y.to_bits());
            assert_eq!(
                first, None,
                "shuffle {shuffle}, vintage {vintage}: first difference"
            );
        }
    }
}

/// What the clip cannot carry, found by playing it back: the step path's note-on runs at every
/// step, a rest's too, and a tied step's. A tie written as one note loses the second step's
/// accent (its note-on sets the accent anew); a rest after a closed gate sets the pitch of the
/// release's tail at once (`pitch = target` without slide), cutting short a glide the step
/// before began, where note mode's tail glides on; a played step after a rest that slid (the
/// gate closed) opens the gate without a new attack and glides from the rest's pitch, which
/// never sounded, where the clip's note attacks (note mode attacks a note with nothing held).
/// Each difference starts at its step (block 4k of step k); the second one changes the
/// oscillator's phase from then on.
#[test]
fn the_clip_cannot_carry_a_rest_or_a_ties_own_accent() {
    let first_difference = |steps: &[[u8; 6]]| {
        let (a, b) = play_303(&pattern_303(false, steps), 0, steps.len() * 4, false);
        a.iter()
            .zip(&b)
            .position(|(x, y)| x.to_bits() != y.to_bits())
            .map(|i| i / BLOCK_FRAMES)
    };
    let steps = steps_303();
    // the tie's second step (3) has an accent of its own
    assert_eq!(first_difference(&steps[..5]), Some(3 * 4));
    // a rest (4) after the tie, whose glide from 51 to 29 has not ended, at the same pitch
    let mut s = steps.clone();
    s[3][2] = 0;
    s[4] = st(5, false, false, false, true, true);
    assert_eq!(first_difference(&s[..5]), Some(4 * 4));
    // a played step (2) after a rest that slid (1), the gate closed: the step path opens the
    // gate without a new attack and glides from the rest's pitch; the clip's note attacks
    let s = [
        st(0, false, false, false, false, false),
        st(0, true, false, false, false, true),
        st(0, false, false, false, false, false),
    ];
    assert_eq!(
        notes(0, &pattern_303(false, &s), &Controls::default()),
        [n(0, 24, 36, 80), n(96, 120, 36, 80)]
    );
    assert_eq!(first_difference(&s), Some(2 * 4));
}

/// The frames of an engine block.
const BLOCK_FRAMES: usize = orb_engine::frame::BLOCK;

/// The first tick whose shuffled tick (the engine's, [`seq::shuffled`]) reaches `target`.
fn shuffled_reaches(target: u32, shuffle: u8) -> u32 {
    (0..)
        .find(|&t| seq::shuffled(t as i32, shuffle) >= target as i32)
        .unwrap()
}

#[test]
fn the_step_and_gate_times_are_the_engines_shuffled_ticks() {
    for shuffle in 0..=127u8 {
        let s = shuffle as u32 / 6;
        for k in 0..16 {
            assert_eq!(
                step_on(k, s),
                shuffled_reaches(48 * k, shuffle),
                "{shuffle} {k}"
            );
            assert_eq!(
                gate_off(k, s),
                shuffled_reaches(48 * k + 24, shuffle),
                "{shuffle} {k}"
            );
        }
    }
}

#[test]
fn an_808_pattern_shared_slots_and_accent() {
    use tr808::*;
    let mut s0 = [0u8; 12];
    s0[AC] = 1;
    s0[BD] = 1;
    s0[LT] = 1;
    s0[RS] = 1;
    let mut s1 = [0u8; 12];
    s1[MT] = 1;
    s1[HT] = 1;
    s1[CP] = 1;
    let mut s2 = [0u8; 12];
    s2[SD] = 1;
    s2[CB] = 1;
    s2[CY] = 1;
    s2[OH] = 1;
    s2[CH] = 1;
    // the AC row alone plays nothing
    let mut s3 = [0u8; 12];
    s3[AC] = 1;
    let bytes = pattern_drums(false, &[s0, s1, s2, s3]);
    // LT/LC on tom, MT/MC on conga, HT/HC on conga, RS/CL on claves, CP/MA on clap
    let c = Controls {
        shuffle: 127,
        switches_808: [true, false, false, false, true],
    };
    let f = parse(&clip(2, 0, &bytes, &c));
    assert_eq!(f.name, "808 A1");
    assert!(f.channels.iter().all(|&c| c == 2), "the 808: channel 3");
    assert_eq!(f.end, 16 * 48);
    let mut want = vec![
        n(0, 24, 36, 127),
        n(0, 24, 43, 127),
        n(0, 24, 75, 127),
        n(48, 72, 39, 80),
        n(48, 72, 62, 80),
        n(48, 72, 63, 80),
        n(96, 120, 38, 80),
        n(96, 120, 42, 80),
        n(96, 120, 46, 80),
        n(96, 120, 49, 80),
        n(96, 120, 56, 80),
    ];
    want.sort_by_key(|n| (n.start, n.key));
    assert_eq!(f.notes, want);
    // the other positions of the switches
    let c = Controls {
        shuffle: 0,
        switches_808: [false, true, true, true, false],
    };
    let keys: Vec<u8> = notes(2, &bytes, &c).iter().map(|n| n.key).collect();
    assert_eq!(keys, [36, 37, 64, 47, 50, 70, 38, 42, 46, 49, 56]);
    // shuffled: the second step of each 8th moves
    let shuffled = notes(2, &pattern_drums(true, &[s0, s1]), &Controls::default());
    assert!(shuffled.iter().all(|n| n.start == 0 || n.start == 48));
    let c = Controls {
        shuffle: 127,
        ..Controls::default()
    };
    let shuffled = notes(2, &pattern_drums(true, &[s0, s1]), &c);
    assert!(
        shuffled
            .iter()
            .all(|n| n.start == 0 || (n.start == 48 + 21 && n.end == 48 + 21 + 24))
    );
}

#[test]
fn a_909_pattern_four_levels_and_flam() {
    use tr909::*;
    let hit = |i: usize, v: u8, ac: u8| {
        let mut s = [0u8; 12];
        s[i] = v;
        s[AC] = ac;
        s
    };
    let steps = [
        hit(BD, 1, 0),
        hit(SD, 2, 0),
        hit(LT, 1, 2),
        hit(MT, 2, 2),
        // flam: value 3 is low + flam, 4 high + flam, 5..255 the flam hit alone (high)
        hit(HT, 3, 0),
        hit(RS, 4, 0),
        hit(CP, 7, 0),
        hit(CH, 4, 2),
        // any AC byte counts as AC
        hit(OH, 1, 1),
        hit(CC, 2, 0),
        hit(RC, 255, 2),
        // the AC row alone plays nothing
        hit(BD, 0, 2),
    ];
    let bytes = pattern_drums(false, &steps);
    let f = parse(&clip(3, 31, &bytes, &Controls::default()));
    assert_eq!(f.name, "909 D8");
    assert!(f.channels.iter().all(|&c| c == 3), "the 909: channel 4");
    let got: Vec<(u32, u8, u8)> = f
        .notes
        .iter()
        .map(|n| (n.start, n.key, n.velocity))
        .collect();
    assert_eq!(
        got,
        [
            (0, 36, 40),
            (48, 38, 80),
            (96, 43, 104),
            (144, 47, 127),
            // the flam bands (task-245); the flam-only value 7 is the plain high band
            (192, 50, 4),
            (240, 37, 52),
            (288, 39, 80),
            (336, 42, 114),
            (384, 46, 104),
            (432, 49, 80),
            (480, 51, 127),
        ]
    );
    // each band plays back in note mode as the step's value (3 and 4 with flam) and AC
    // (tr909::velocity_band)
    for (flam, by_high) in VEL_909.iter().enumerate() {
        for (high, by_ac) in by_high.iter().enumerate() {
            for (&vel, ac) in by_ac.iter().zip([0, 2]) {
                assert_eq!(velocity_band(vel), (1 + high as u8 + 2 * flam as u8, ac));
            }
        }
    }
    assert!(f.notes.iter().all(|n| n.end == n.start + DRUM_LEN));
}

// The export velocities fall in note mode's bands (the 303's and the 808's accent threshold).
const _: () = assert!(VEL_NORMAL < tr808::ACCENT_VELOCITY && VEL_ACCENT >= tr808::ACCENT_VELOCITY);

#[test]
fn names_and_lengths() {
    assert_eq!(file_name(0, 0), "303-1 A1.mid");
    assert_eq!(file_name(1, 9), "303-2 B2.mid");
    assert_eq!(file_name(2, 23), "808 C8.mid");
    assert_eq!(file_name(3, 24), "909 D1.mid");
    // a length byte outside 1..16 is cut
    let mut b = pattern_drums(false, &[]);
    b[1] = 0;
    assert_eq!(length(&b), 48);
    b[1] = 40;
    assert_eq!(length(&b), 16 * 48);
    // an empty pattern is a clip of its length with no notes
    let f = parse(&clip(
        2,
        0,
        &pattern_drums(false, &[]),
        &Controls::default(),
    ));
    assert!(f.notes.is_empty());
    assert_eq!(f.end, 16 * 48);
}

#[test]
fn a_variable_length_quantity() {
    for (v, want) in [
        (0u32, vec![0x00]),
        (0x7F, vec![0x7F]),
        (0x80, vec![0x81, 0x00]),
        (0x3FFF, vec![0xFF, 0x7F]),
        (0x0FFF_FFFF, vec![0xFF, 0xFF, 0xFF, 0x7F]),
    ] {
        let mut out = Vec::new();
        vlq(&mut out, v);
        assert_eq!(out, want, "{v:#x}");
        let mut i = 0;
        assert_eq!(read_vlq(&out, &mut i), v);
    }
}

#[test]
fn a_note_off_comes_before_a_note_on_at_the_same_tick() {
    let f = smf(0, "x", &[n(0, 48, 40, 80), n(48, 96, 40, 80)], 96);
    let p = parse(&f);
    assert_eq!(p.notes, [n(0, 48, 40, 80), n(48, 96, 40, 80)]);
}

#[test]
fn the_temporary_files_are_swept() {
    let base = tempfile::tempdir().unwrap();
    let a = write_temp(base.path(), "303-1 A1.mid", b"one").unwrap();
    assert_eq!(std::fs::read(&a).unwrap(), b"one");
    assert_eq!(a.file_name().unwrap(), "303-1 A1.mid");
    // a second clip of the same name: a folder of its own
    let b = write_temp(base.path(), "303-1 A1.mid", b"two").unwrap();
    assert_ne!(a, b);
    assert!(a.is_file() && b.is_file(), "younger than KEEP: kept");
    sweep(base.path(), KEEP);
    assert!(a.is_file() && b.is_file());
    sweep(base.path(), Duration::ZERO);
    assert!(!a.exists() && !b.exists());
    assert_eq!(std::fs::read_dir(base.path()).unwrap().count(), 0);
    // no folder: nothing to do
    sweep(&base.path().join("none"), Duration::ZERO);
}
