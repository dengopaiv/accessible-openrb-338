//! task-197 AC #2, #3 and #4: the plugin plays a loaded song's patterns, its output does not
//! depend on the host's buffer sizes, it reports 64 frames of latency at 44.1 kHz, and at 48 kHz
//! it resamples deterministically with the resampler's latency added.

mod common;

use std::sync::Arc;

use common::{Change, activate, host, host_with, peak, song_bytes, with_song};
use orb_player::{Command, Player};
use orb_plugin::core::{BLOCK, HostTransport, settle};
use orb_plugin::instance::Mode;

/// Changes of every kind of target: a 303 knob, a pattern selection, an 808 voice switch, the
/// master, a mixer mute, the shuffle, an effect, and the mute off again.
const CHANGES: [Change; 8] = [
    Change {
        at: 9_000,
        id: "d1c3",
        value: 110,
    },
    Change {
        at: 15_017,
        id: "d3c1",
        value: 3,
    },
    Change {
        at: 21_000,
        id: "d3c11",
        value: 1,
    },
    Change {
        at: 26_001,
        id: "d0c0",
        value: 70,
    },
    Change {
        at: 31_999,
        id: "d0c5",
        value: 0,
    },
    Change {
        at: 40_000,
        id: "g20",
        value: 90,
    },
    Change {
        at: 47_111,
        id: "d5c3",
        value: 120,
    },
    Change {
        at: 70_003,
        id: "d0c5",
        value: 1,
    },
];

const FRAMES: usize = 100_000;

fn run(rate: u32, buf: usize, changes: &[Change]) -> Vec<f32> {
    let p = with_song("full_mix");
    let mut core = activate(&p, rate);
    host(&p, &mut core, FRAMES, buf, changes)
}

#[test]
fn the_latency_is_one_block_at_44100_and_more_when_resampled() {
    let p = with_song("full_mix");
    assert_eq!(activate(&p, 44_100).latency(), 64);
    // (64 + 32) engine frames at 48 kHz: 104.49 host frames, rounded up
    assert_eq!(activate(&p, 48_000).latency(), 105);
    assert_eq!(activate(&p, 96_000).latency(), 209);
}

#[test]
fn buffer_sizes_give_the_same_output_at_44100() {
    let want = run(44_100, 512, &CHANGES);
    assert!(peak(&want) > 0.05, "the song plays");
    for buf in [1, 63, 64, 4096] {
        let got = run(44_100, buf, &CHANGES);
        let first = want.iter().zip(&got).position(|(a, b)| a != b);
        assert_eq!(
            first, None,
            "buffer size {buf}: first difference at sample {first:?}"
        );
    }
    // the changes are heard: the same run without them differs
    assert_ne!(run(44_100, 512, &[]), want);
}

/// Pattern mode follows the host's transport (task-217): the host's stop stops the patterns,
/// its start starts them again; note mode never plays them.
#[test]
fn pattern_mode_stops_and_starts_with_the_host() {
    let p = with_song("full_mix");
    let mut core = activate(&p, 44_100);
    let mut out = host(&p, &mut core, 55_000, 512, &[]);
    out.extend(host_with(
        &p,
        &mut core,
        9_000,
        512,
        &[],
        &HostTransport::default(),
    ));
    out.extend(host(&p, &mut core, 6_000, 512, &[]));
    assert!(peak(&out[..2 * 55_000]) > 0.1, "playing");
    // the host stops at 55000: from the release on (the delay's tail) quiet until it plays
    // again at 64000
    let stopped = &out[2 * 60_000..2 * 64_000];
    assert!(peak(stopped) < 0.01, "stopped: peak {}", peak(stopped));
    assert!(peak(&out[2 * 64_000..2 * 70_000]) > 0.1, "playing again");
    // note mode: never played, the host playing
    let p = with_song("full_mix");
    p.set_mode(Mode::Note);
    let mut core = activate(&p, 44_100);
    assert_eq!(peak(&host(&p, &mut core, 20_000, 512, &[])), 0.0);
    // pattern mode with the host stopped: no sound
    let p = with_song("full_mix");
    let mut core = activate(&p, 44_100);
    let t = HostTransport::default();
    assert_eq!(peak(&host_with(&p, &mut core, 20_000, 512, &[], &t)), 0.0);
}

/// Without changes the plugin plays what orb-player plays for the same song in pattern mode,
/// one block later: the same engine, the same messages, after the same settling of the mixer's
/// gains at the activation (task-226).
#[test]
fn the_plugin_plays_as_the_player_one_block_later() {
    let plugin = run(44_100, 333, &[]);
    let song = orb_plugin::instance::parse_song(&song_bytes("full_mix")).unwrap();
    let tables = Arc::new(orb_engine::tables::Tables::new());
    let (mut p, mut a) = Player::with_call_frames(tables, None, BLOCK);
    assert!(
        p.load(&song, Some(rbm::standard::builtin_samples()))
            .is_ok()
    );
    settle(&mut a);
    assert!(p.send(Command::SongMode(false)).is_ok());
    assert!(p.send(Command::Play).is_ok());
    // the player's calls as the plugin makes them (render_call after settle, not render_f32)
    let mut player = Vec::with_capacity(2 * FRAMES);
    while player.len() < 2 * FRAMES {
        a.drain();
        player.extend(a.render_call().iter().map(|&s| s as f32 / 32768.0));
    }
    assert!(
        plugin[..128].iter().all(|&x| x == 0.0),
        "the block ahead is silence"
    );
    let first = plugin[128..].iter().zip(&player).position(|(a, b)| a != b);
    assert_eq!(first, None, "first difference at sample {first:?}");
}

#[test]
fn at_48000_the_output_is_resampled_and_deterministic() {
    let want = run(48_000, 512, &CHANGES);
    assert!(peak(&want) > 0.05, "the song plays");
    // two offline renders of the same state are identical, whatever the buffer size
    assert_eq!(run(48_000, 512, &CHANGES), want);
    for buf in [1, 63, 4096] {
        let got = run(48_000, buf, &CHANGES);
        let first = want.iter().zip(&got).position(|(a, b)| a != b);
        assert_eq!(
            first, None,
            "buffer size {buf}: first difference at sample {first:?}"
        );
    }
    // the first (64 + 32) * 48000 / 44100 frames are the silence before the engine's first
    // frame, then the song: about as loud as at 44.1 kHz
    let at44 = run(44_100, 512, &[]);
    let at48 = run(48_000, 512, &[]);
    let (p44, p48) = (peak(&at44), peak(&at48));
    assert!((p48 / p44 - 1.0).abs() < 0.1, "peaks {p44} and {p48}");
}
