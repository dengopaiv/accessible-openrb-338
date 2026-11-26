//! Song msgs 28 and 29, the Edit menu's "Initialize Song from Pattern Mode" and "Initialize Loop
//! from Pattern Mode" (task-306, `docs/ui_spec.md` section 12): each song of
//! `tests/songs/edit_init` is loaded, the command runs, and every track's list must equal, node
//! for node in list order, the TRAK chunks of the file the original saved after the same command
//! (the original's save writes each track's list in order, `sub_437CFF`).

use std::path::{Path, PathBuf};
use std::sync::Arc;

use orb_engine::frame::{Activation, Devices, Engine};
use orb_engine::seq::Pool;
use orb_engine::song::{Device, Song};
use orb_engine::tables::Tables;

fn dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/songs/edit_init")
}

fn song(name: &str) -> Song {
    let data = std::fs::read(dir().join(name)).unwrap();
    orb_engine::load::convert(&rbs::Song::parse(&data).unwrap()).unwrap()
}

fn loaded(name: &str) -> Engine {
    let mut e = Engine::new(
        Arc::new(Tables::new()),
        Devices::implemented(Some(rbm::standard::builtin_samples())),
    );
    e.load(&song(name)).unwrap();
    e
}

/// Track i's list as `(tick, ctrl, value)`.
fn lists(e: &Engine) -> Vec<Vec<(u32, u8, u8)>> {
    (0..9)
        .map(|i| {
            e.frame
                .seq
                .events(i)
                .iter()
                .map(|n| (n.tick as u32, n.ctrl, n.value))
                .collect()
        })
        .collect()
}

fn expected(name: &str) -> Vec<Vec<(u32, u8, u8)>> {
    song(name)
        .tracks
        .iter()
        .map(|t| t.iter().map(|n| (n.tick, n.ctrl, n.value)).collect())
        .collect()
}

fn check(input: &str, command: fn(&mut Engine), saved: &str) {
    let mut e = loaded(input);
    command(&mut e);
    let (got, want) = (lists(&e), expected(saved));
    for i in 0..9 {
        assert_eq!(got[i], want[i], "{input} -> {saved}: track {i}");
    }
}

#[test]
fn initialize_loop_equals_the_originals_save() {
    check(
        "init_loop.rbs",
        Engine::init_loop,
        "init_loop.after_init_loop.rbs",
    );
    check(
        "init_zero.rbs",
        Engine::init_loop,
        "init_zero.after_init_loop.rbs",
    );
}

#[test]
fn initialize_song_equals_the_originals_save() {
    check(
        "init_loop.rbs",
        Engine::init_song,
        "init_loop.after_init_song.rbs",
    );
    check(
        "init_zero.rbs",
        Engine::init_song,
        "init_zero.after_init_song.rbs",
    );
}

/// After the command the devices hold the chased values of the current position (the
/// `sub_1001FDC0` chase): at tick 0 after Initialize Song, every controller is the pattern copy's.
#[test]
fn initialize_song_applies_the_pattern_values_at_once() {
    let mut e = loaded("init_loop.rbs");
    let d = orb_engine::song::Device::Tb303_1;
    let before = e.frame.devices.device_mut(d).live().to_vec();
    e.init_song();
    let after = e.frame.devices.device_mut(d).live().to_vec();
    let pattern = e.frame.devices.device_mut(d).pattern_params().to_vec();
    assert_ne!(before[..9], pattern[..9]);
    assert_eq!(after[..9], pattern[..9]);
}

/// orb's Undo of the commands (task-306, not the original's): the lists come back as they were.
#[test]
fn undo_puts_the_tracks_back() {
    for name in ["init_loop.rbs", "init_zero.rbs"] {
        for command in [Engine::init_loop, Engine::init_song] {
            let mut e = loaded(name);
            let before = lists(&e);
            assert!(e.keep_undo(Box::new(Pool::new())).is_none());
            command(&mut e);
            assert_ne!(lists(&e), before);
            assert!(e.undo_init().is_some());
            assert_eq!(lists(&e), before, "{name}");
            assert!(e.undo_init().is_none(), "one level");
        }
    }
}

/// Every device's live parameters, the mixer's included.
fn live(e: &mut Engine) -> (orb_engine::frame::MixerParams, Vec<Vec<u8>>) {
    let mixer = e.frame.devices.mixer.live;
    let devices = Device::ALL
        .iter()
        .map(|&d| e.frame.devices.device_mut(d).live().to_vec())
        .collect();
    (mixer, devices)
}

/// Undo while playing: the song goes on, and the devices then hold what a song that never had
/// the command holds at the same place.
#[test]
fn undo_while_playing() {
    let mut buf = vec![0i16; 2 * 1024];
    let mut run = |e: &mut Engine, calls: usize| {
        for _ in 0..calls {
            e.render(&mut buf, 1024);
        }
    };
    for command in [Engine::init_loop, Engine::init_song] {
        let mut a = loaded("init_loop.rbs");
        let mut b = loaded("init_loop.rbs");
        for e in [&mut a, &mut b] {
            e.activate(Activation::ReplacesCurrent {
                current_playing: false,
            });
            e.locate(1536);
            e.play();
        }
        run(&mut a, 20);
        a.keep_undo(Box::new(Pool::new()));
        command(&mut a);
        run(&mut a, 20);
        let during = live(&mut a);
        a.undo_init();
        assert!(a.playing());
        run(&mut a, 60);
        run(&mut b, 100);
        assert_eq!(a.frame.seq.pos, b.frame.seq.pos);
        assert_eq!(lists(&a), lists(&b));
        let after = live(&mut a);
        assert_ne!(during, after, "the command's values are gone");
        assert_eq!(after, live(&mut b));
    }
}
