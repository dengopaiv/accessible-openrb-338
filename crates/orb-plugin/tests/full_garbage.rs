//! task-225: host events while the player's garbage queue is full. The main thread's 128
//! pattern edits, sent before the first block and never collected, fill the queue at the first
//! block (63 of them fit; the rest wait). The host's events of every kind still take effect at
//! the next block: the output is the same, frame for frame, as an instance's whose main thread
//! collects after every call, and nothing is refused (`Core::dropped_events`).

mod common;

use common::with_song;
use orb_engine::song::Device;
use orb_player::midi::Message;
use orb_player::{Command, GARBAGE_PER_COMMAND};
use orb_plugin::core::{BLOCK, Core, HostEvent, HostTransport};
use orb_plugin::instance::{Mode, OrbParams};

const RATE: u32 = 44_100;

fn on(channel: u8, note: u8) -> HostEvent {
    HostEvent {
        frame: 0,
        message: Message::NoteOn {
            channel,
            note,
            velocity: 100,
        },
    }
}

fn off(channel: u8, note: u8) -> HostEvent {
    HostEvent {
        frame: 0,
        message: Message::NoteOff {
            channel,
            note,
            velocity: 64,
        },
    }
}

/// The 808's pattern 32 (the last), 16 steps of instrument 1: the same bytes each time, so
/// that the instance whose edits wait has the same patterns as the one that applied them all.
fn edit() -> Command {
    let mut p = vec![0u8; 194];
    p[1] = 16;
    for s in 0..16 {
        p[2 + 12 * s + 1] = 1;
    }
    Command::Pattern {
        device: Device::Tr808,
        pattern: 31,
        bytes: p.into_boxed_slice(),
    }
}

/// An instance of the song `full_mix` in `mode` with the 128 edits queued.
fn instance(mode: Mode) -> (OrbParams, Core) {
    let p = with_song("full_mix");
    p.set_mode(mode);
    let core = common::activate(&p, RATE);
    for _ in 0..128 {
        p.edit(edit()).unwrap();
    }
    (p, core)
}

/// What the host does in call `k` (one block each): its transport and events.
type Script = fn(usize) -> (HostTransport, Vec<HostEvent>);

/// The host at call `k`, playing in the ranges of calls `plays`, each from beat 0, at 120 BPM.
fn transport(k: usize, plays: &[std::ops::Range<usize>]) -> HostTransport {
    let from = plays.iter().find(|r| r.contains(&k)).map(|r| r.start);
    HostTransport {
        playing: from.is_some(),
        tempo: Some(120.0),
        pos_beats: Some(from.map_or(0.0, |f| (k - f) as f64 * BLOCK as f64 * 2.0 / RATE as f64)),
    }
}

/// Runs `calls` calls of one block in two instances, the one whose garbage queue stays full
/// and one whose main thread collects after each call; `check(k, core)` runs after call `k`
/// on the full one. Returns both outputs.
fn session(
    mode: Mode,
    calls: usize,
    script: Script,
    mut check: impl FnMut(usize, &Core),
) -> (Vec<f32>, Vec<f32>) {
    let (full_p, mut full) = instance(mode);
    let (free_p, mut free) = instance(mode);
    let (mut a, mut b) = (Vec::new(), Vec::new());
    for k in 0..calls {
        let (t, ev) = script(k);
        let (mut l, mut r) = ([0f32; BLOCK], [0f32; BLOCK]);
        full.process_with(&mut l, &mut r, &full_p.values(), &t, &ev);
        a.extend(l.iter().zip(&r).flat_map(|(x, y)| [*x, *y]));
        free.process_with(&mut l, &mut r, &free_p.values(), &t, &ev);
        b.extend(l.iter().zip(&r).flat_map(|(x, y)| [*x, *y]));
        free_p.collect_garbage();
        // full from the first block (rendered in call 1; call 0 hands out the latency's
        // silence): no edit fits any more
        if k > 0 {
            assert!(
                full.audio().garbage_room() < GARBAGE_PER_COMMAND,
                "call {k}"
            );
        }
        check(k, &full);
    }
    assert_eq!(full.dropped_events(), 0);
    assert_eq!(free.dropped_events(), 0);
    (a, b)
}

fn peak(x: &[f32]) -> f32 {
    x.iter().fold(0f32, |m, v| m.max(v.abs()))
}

#[test]
fn latch_notes_and_the_host_transport_pass_a_full_queue() {
    // the host plays from call 4 to 40 and from 50 on; a Latch note selects 303 #1's pattern
    // 6 at call 20
    let script: Script = |k| {
        let ev = if k == 20 { vec![on(0, 5)] } else { vec![] };
        (transport(k, &[4..40, 50..1000]), ev)
    };
    let playing = |core: &Core| core.audio().current().playing();
    let (full, free) = session(Mode::Latch, 80, script, |k, core| {
        let ch = &core.audio().current().frame.devices.channels[0];
        match k {
            // the host starts at call 4: the block of call 5 plays
            4 => assert!(!playing(core)),
            5 => assert!(playing(core)),
            // the note of call 20 is applied at once (it reaches the block of call 21): shown
            // and playing
            19 => assert_ne!(ch.playing_pattern(), Some(5)),
            21 => {
                assert_eq!(core.audio().last_snapshot().live[1].as_slice()[1], 5);
                assert_eq!(ch.playing_pattern(), Some(5));
            }
            // stop at call 40, start again at 50
            40 => assert!(playing(core)),
            41 => assert!(!playing(core)),
            50 => assert!(!playing(core)),
            51 => assert!(playing(core)),
            _ => {}
        }
    });
    assert!(peak(&full) > 0.01);
    assert_eq!(full, free);
}

#[test]
fn note_mode_notes_pass_a_full_queue() {
    // a 303 note from call 10 to 30, an 808 bass drum at call 50
    let script: Script = |k| {
        let ev = match k {
            10 => vec![on(0, 36)],
            30 => vec![off(0, 36)],
            50 => vec![on(2, 36)],
            _ => vec![],
        };
        (HostTransport::default(), ev)
    };
    let (full, free) = session(Mode::Note, CALLS, script, |k, core| {
        let s = core.audio().last_snapshot();
        assert!(!s.playing);
        match k {
            10 => assert_eq!(s.sounding[2].mask, 0),
            51 => assert_eq!(
                s.sounding[2].mask & 1 << orb_engine::tr808::sound::BD,
                1 << orb_engine::tr808::sound::BD
            ),
            _ => {}
        }
    });
    let at = |k: usize| 2 * BLOCK * k;
    assert_eq!(peak(&full[..at(11)]), 0.0, "nothing before the note");
    assert!(peak(&full[at(11)..at(31)]) > 0.05, "the 303 note sounds");
    // a lost note-off would hold the 303's note (as `note_mode.rs`, 20000 frames and more
    // after it)
    assert!(
        peak(&full[at(CALLS - 32)..]) < 0.002,
        "the note-off ended it"
    );
    assert_eq!(full, free);
}

/// The calls of the note-mode session: the 303's note-off at call 30 has faded by the end.
const CALLS: usize = 800;
