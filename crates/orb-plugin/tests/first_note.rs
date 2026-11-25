//! task-226: a note at the very start of the host's timeline. Reaper's offline render activates
//! the plugin and plays from sample 0, so a note at position 0 is an event of the first call
//! after the activation; a realtime start is the transport starting in a call with a note at its
//! frame 0. Either way the note sounds as the same note later does, for every device, in every
//! mode (`docs/plugin.md` section 4).
//!
//! The comparison is "as the same note later", not "bit for bit": engine state runs on in
//! silence (the 303s' voices run free, their amp adding 1e-6 every block at 0x1002494C), so a
//! 303's waveform depends on when its note starts, and a drum sample's int16 truncation flips by
//! one LSB now and then. Two later notes differ from each other the same way; the note at 0 must
//! not differ more.

mod common;

use common::{activate, peak, with_song};
use orb_player::midi::Message;
use orb_plugin::core::{Core, HostEvent, HostTransport};
use orb_plugin::instance::{Mode, OrbParams};

/// The shift between the notes compared: a whole number of engine blocks at 44.1 kHz (160)
/// and at 48 kHz (147: 10240 * 147 / 160 = 9408 engine frames), so that the resampler's phase
/// is the same.
const S: usize = 10_240;
/// The frames compared after each note.
const LEN: usize = 16_384;
/// The host's buffer size.
const BUF: usize = 512;
const BPM: f64 = 120.0;
/// The host is stopped this long before it plays, in a realtime start (a whole number of
/// shifts, so that the blocks keep their phase).
const WARM: usize = 2 * S;
/// One LSB of the int16 output.
const LSB: f32 = 1.0 / 32768.0;

fn note_on(channel: u8, note: u8) -> Message {
    Message::NoteOn {
        channel,
        note,
        velocity: 100,
    }
}

fn note_off(channel: u8, note: u8) -> Message {
    Message::NoteOff {
        channel,
        note,
        velocity: 64,
    }
}

/// A host that plays from beat 0 at [`BPM`] after `warm` frames stopped, in calls of [`BUF`]
/// frames split at the events, which are at frames counted from the start of the play.
/// Returns the played frames, interleaved.
fn host(
    p: &OrbParams,
    core: &mut Core,
    rate: u32,
    warm: usize,
    frames: usize,
    events: &[(usize, Message)],
) -> Vec<f32> {
    let stopped = HostTransport {
        playing: false,
        tempo: Some(BPM),
        pos_beats: Some(0.0),
    };
    let mut pos = 0;
    while pos < warm {
        let n = BUF.min(warm - pos);
        let (mut l, mut r) = (vec![0f32; n], vec![0f32; n]);
        core.process_with(&mut l, &mut r, &p.values(), &stopped, &[]);
        pos += n;
    }
    let (mut l, mut r) = (vec![0f32; frames], vec![0f32; frames]);
    let mut pos = 0;
    while pos < frames {
        let n = BUF.min(frames - pos);
        let t = HostTransport {
            playing: true,
            tempo: Some(BPM),
            pos_beats: Some(pos as f64 * BPM / 60.0 / rate as f64),
        };
        let ev: Vec<HostEvent> = events
            .iter()
            .filter(|(at, _)| (pos..pos + n).contains(at))
            .map(|&(at, message)| HostEvent {
                frame: (at - pos) as u32,
                message,
            })
            .collect();
        core.process_with(
            &mut l[pos..pos + n],
            &mut r[pos..pos + n],
            &p.values(),
            &t,
            &ev,
        );
        pos += n;
    }
    l.into_iter().zip(r).flat_map(|(a, b)| [a, b]).collect()
}

/// The largest difference of two outputs.
fn diff(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b).fold(0.0, |m, (x, y)| m.max((x - y).abs()))
}

fn rms(x: &[f32]) -> f64 {
    (x.iter().map(|&v| v as f64 * v as f64).sum::<f64>() / x.len() as f64).sqrt()
}

/// The [`LEN`] frames after host frame `at`.
fn after(out: &[f32], at: usize) -> &[f32] {
    &out[2 * at..2 * (at + LEN)]
}

/// A device's note at host frame 0, `S` and `2S` of the play, each in a new instance in
/// `mode` (`with`: a song in it, else a new instance), after `warm` frames of a stopped host;
/// the 303's note ends after 0.2 s. Returns the [`LEN`] frames after each note.
fn three(
    mode: Mode,
    song: Option<&str>,
    rate: u32,
    warm: usize,
    channel: u8,
    note: u8,
) -> [Vec<f32>; 3] {
    [0, S, 2 * S].map(|at| {
        let p = match song {
            Some(s) => with_song(s),
            None => OrbParams::new(),
        };
        p.set_mode(mode);
        let mut core = activate(&p, rate);
        let events = [
            (at, note_on(channel, note)),
            (at + 8_820, note_off(channel, note)),
        ];
        let out = host(&p, &mut core, rate, warm, at + LEN, &events);
        after(&out, at).to_vec()
    })
}

/// The note at 0 sounds as the later ones: as loud (within 3%), and no further from the note at `S`
/// than the note at `2S` is (for a drum at 44.1 kHz: at most one LSB).
fn check(what: &str, x: &[Vec<f32>; 3], drum_at_44100: bool) {
    let (p0, p1) = (peak(&x[0]), peak(&x[1]));
    let (r0, r1, r2) = (rms(&x[0]), rms(&x[1]), rms(&x[2]));
    let (d01, d12) = (diff(&x[0], &x[1]), diff(&x[1], &x[2]));
    let msg = format!(
        "{what}: peaks {p0} {p1}, rms {r0:.6} {r1:.6} {r2:.6}, differences {d01} (0 and S) {d12} \
         (S and 2S)"
    );
    assert!(p1 > 0.05, "{msg}: the later note sounds");
    // the 303's level varies by about 1% with its waveform's phase at the note-on; the fade-in
    // of a song load took a quarter, a lost note all
    assert!((r0 / r1 - 1.0).abs() < 0.03, "{msg}");
    if drum_at_44100 {
        assert!(d01 <= LSB, "{msg}");
    } else {
        assert!(d01 <= d12.max(4.0 * LSB) * 1.5, "{msg}");
    }
}

/// The devices: 303-1, 303-2, the 808's and the 909's bass drum, the 808's snare, the 909's
/// closed hi-hat (channel, note, a drum).
const DEVICES: [(u8, u8, bool); 6] = [
    (0, 36, false),
    (1, 48, false),
    (2, 36, true),
    (3, 36, true),
    (2, 38, true),
    (3, 42, true),
];

/// The pattern parameters of the four sections.
const PATTERN_IDS: [&str; 4] = ["d1c1", "d2c1", "d3c1", "d4c1"];

/// The pattern each section plays (the snapshot's).
fn playing(core: &Core) -> [Option<u8>; 4] {
    core.audio().last_snapshot().patterns
}

/// The song's own patterns, the parameters' values (a section's notes 0..31 select them).
fn song_patterns(p: &OrbParams) -> [u8; 4] {
    PATTERN_IDS.map(|id| p.param(id).unwrap().value() as u8)
}

/// The notes at frame 0 that select `picks`, one per section's channel.
fn selections(picks: [u8; 4]) -> Vec<(usize, Message)> {
    (0..4).map(|c| (0, note_on(c as u8, picks[c]))).collect()
}

/// Latch Pattern Mode: notes at frame 0 of the first call (the host plays from 0) select the
/// patterns the song starts with: an instance whose pattern parameters are at pattern 32 (empty
/// in the song) plays, after notes that select the song's patterns, what the song plays with
/// its own selections, sample for sample. With the host playing from a while after the
/// activation as well.
#[test]
fn latch_notes_at_the_start_select_the_first_patterns() {
    for rate in [44_100, 48_000] {
        for warm in [0, WARM] {
            let want = with_song("full_mix");
            let picks = song_patterns(&want);
            let mut core = activate(&want, rate);
            let song = host(&want, &mut core, rate, warm, 3 * S, &[]);
            assert!(peak(&song) > 0.05);

            let p = with_song("full_mix");
            for id in PATTERN_IDS {
                common::set(&p, id, 31);
            }
            let mut core = activate(&p, rate);
            let notes = host(&p, &mut core, rate, warm, 3 * S, &selections(picks));
            assert_eq!(playing(&core), picks.map(Some), "rate {rate}, warm {warm}");
            let first = notes.iter().zip(&song).position(|(a, b)| a != b);
            assert_eq!(first, None, "rate {rate}, warm {warm}: first difference");
        }
    }
}

/// Gate Pattern Mode: notes held from frame 0 of the first call open the four gates on the
/// song's patterns from the first block, and the song sounds as in Latch Pattern Mode, sample
/// for sample (after a stopped while, within one LSB). The notes come before the first block
/// that sees the host play: they must not start the song free-running as with the host stopped
/// (the block's sync would stop it again, slot 6 cutting the first step).
#[test]
fn gate_notes_at_the_start_open_the_gates_at_once() {
    for rate in [44_100, 48_000] {
        for warm in [0, WARM] {
            let want = with_song("full_mix");
            let picks = song_patterns(&want);
            let mut core = activate(&want, rate);
            let latch = host(&want, &mut core, rate, warm, 3 * S, &[]);

            let p = with_song("full_mix");
            p.set_mode(Mode::Gate);
            let mut core = activate(&p, rate);
            let gate = host(&p, &mut core, rate, warm, 3 * S, &selections(picks));
            assert_eq!(playing(&core), picks.map(Some), "rate {rate}, warm {warm}");
            assert_eq!(core.audio().last_snapshot().gate_closed, [false; 4]);
            if warm == 0 {
                let first = gate.iter().zip(&latch).position(|(a, b)| a != b);
                assert_eq!(first, None, "rate {rate}: first difference");
            } else {
                // the gates closed while the host was stopped, and slot 6 ran on the sections
                // (`docs/plugin.md` section 11): the 303s' voices leak differently
                let d = diff(&gate, &latch);
                assert!(d <= LSB, "rate {rate}, warm {warm}: difference {d}");
            }
        }
    }
}

/// The user's live case (round 3, 2026-11-16: Reaper at 48 kHz, the 909's bass drum at 0, the
/// first hit after pressing play "muffled", the same hit at the loop wrap right): the host
/// starts after a stopped stretch that is not a whole number of engine blocks, with the note at
/// frame 0 of its first playing call. The host compensates the reported latency and drops what
/// the plugin outputs before the play start (the first `latency` frames of the play), so no
/// part of the note may come out there: an event used to land in the block that held its frame,
/// up to 63 engine frames before it, and that early attack was cut. The note now sounds whole,
/// at or after its frame.
#[test]
fn a_note_at_an_unaligned_realtime_start_sounds_whole() {
    for rate in [44_100, 48_000] {
        // the hit in an activation that starts with it: block-aligned, nothing early
        let p = OrbParams::new();
        let mut core = activate(&p, rate);
        let latency = core.latency() as usize;
        let want = host(&p, &mut core, rate, 0, LEN, &[(0, note_on(3, 36))]);
        let full = peak(&want);
        assert!(full > 0.1, "rate {rate}: {full}");
        for warm in [1_000, 7_777, 20_000, 20_001, 20_480] {
            let p = OrbParams::new();
            let mut core = activate(&p, rate);
            let out = host(&p, &mut core, rate, warm, LEN, &[(0, note_on(3, 36))]);
            let early = peak(&out[..2 * latency]);
            // what the host plays: from the play start on, after its latency compensation
            let got = peak(&out[2 * latency..]);
            let onset = |x: &[f32]| x.iter().position(|v| v.abs() > 1e-3).unwrap() / 2;
            let late = onset(&out) as i64 - onset(&want) as i64;
            let what = format!(
                "rate {rate}, warm {warm}: early peak {early}, peak {got} of {full}, onset {late} \
                 frames after the aligned one"
            );
            // at 44.1 kHz nothing before the note; resampled, only the kernel's pre-ringing
            let most = if rate == 44_100 { 0.0 } else { 0.1 * full };
            assert!(early <= most, "{what}");
            assert!((got / full - 1.0).abs() < 0.01, "{what}");
            // late by less than an engine block, never early
            let block = (64 * rate as usize).div_ceil(44_100) as i64;
            assert!((0..block).contains(&late), "{what}");
        }
    }
}

/// Note mode: the offline render (the note in the first call after the activation) and the
/// realtime start (the host plays after a while stopped, the note in the first playing call),
/// at 44.1 kHz and resampled at 48 kHz.
#[test]
fn a_note_at_the_start_sounds_as_later_in_note_mode() {
    for rate in [44_100, 48_000] {
        for warm in [0, WARM] {
            for (channel, note, drum) in DEVICES {
                let x = three(Mode::Note, None, rate, warm, channel, note);
                let what = format!("rate {rate}, warm {warm}, channel {channel}, note {note}");
                check(&what, &x, drum && rate == 44_100);
            }
        }
    }
}
