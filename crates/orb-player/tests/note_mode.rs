//! Note mode through the player (task-200): `Command::Note` plays the devices of a stopped
//! song, is ignored while it plays, and the snapshot carries the drum machines' sounding masks
//! and AC flags.

mod common;

use common::{run, samples, send, tables};
use orb_engine::song::Device;
use orb_engine::tr808::sound as s808;
use orb_engine::tr909::sound as s909;
use orb_player::{Audio, Command, Player};

fn note(p: &mut Player, device: Device, note: u8, velocity: u8) {
    send(
        p,
        Command::Note {
            device,
            note,
            velocity,
        },
    );
}

fn peak(x: &[i16]) -> i32 {
    x.iter().map(|v| (*v as i32).abs()).max().unwrap_or(0)
}

fn player() -> (Player, Audio) {
    let (p, mut a) = Player::new(tables(), Some(samples()));
    run(&mut a, 1);
    (p, a)
}

#[test]
fn notes_sound_on_a_stopped_song() {
    let (mut p, mut a) = player();
    assert_eq!(peak(&run(&mut a, 2)), 0);
    note(&mut p, Device::Tb303_1, 36, 100);
    let held = run(&mut a, 4);
    assert!(peak(&held) > 1000, "{}", peak(&held));
    note(&mut p, Device::Tb303_1, 36, 0);
    // the release: the amp falls off within a few calls
    run(&mut a, 20);
    assert!(peak(&run(&mut a, 1)) < 50);
    // notes never change the parameters
    let live = p.snapshot().live;
    note(&mut p, Device::Tr808, 64, 127);
    run(&mut a, 1);
    assert_eq!(p.snapshot().live, live);
}

#[test]
fn the_snapshot_has_the_sounding_masks_and_ac_flags() {
    let (mut p, mut a) = player();
    note(&mut p, Device::Tr808, 64, 127);
    note(&mut p, Device::Tr808, 56, 64);
    note(&mut p, Device::Tr909, 46, 120);
    note(&mut p, Device::Tr909, 38, 60);
    run(&mut a, 1);
    let snap = *p.snapshot();
    let [a1, a2, d8, d9] = snap.sounding;
    assert_eq!((a1.mask, a2.mask), (0, 0));
    assert_eq!(d8.mask, 1 << s808::LC | 1 << s808::CB);
    assert_eq!(d8.ac & d8.mask, 1 << s808::LC);
    assert_eq!(d9.mask, 1 << s909::OH | 1 << s909::SD);
    assert_eq!(d9.ac & d9.mask, 1 << s909::OH);
    // the cowbell's sample ends (it plays at a constant gain: the voice ends with it)
    run(&mut a, 200);
    assert_eq!(p.snapshot().sounding[2].mask & 1 << s808::CB, 0);
}

#[test]
fn notes_are_ignored_while_the_song_plays() {
    let (mut p, mut a) = player();
    send(&mut p, Command::Play);
    run(&mut a, 1);
    assert!(p.snapshot().playing);
    note(&mut p, Device::Tr909, 36, 127);
    note(&mut p, Device::Tb303_1, 36, 127);
    let out = run(&mut a, 2);
    assert_eq!(p.snapshot().sounding[3].mask, 0);
    // the empty song's patterns are silent: so is everything
    assert_eq!(peak(&out), 0);
}
