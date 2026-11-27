//! task-225: a full garbage queue holds back only the commands that hand a box back, with the
//! ones queued after them, until the UI side collects; a song's Play while it is pending always
//! finds the slot kept for it.

mod common;

use common::{manifest, run, samples, send, song, tables};
use orb_engine::song::Device;
use orb_player::{Command, GARBAGE, GARBAGE_PER_COMMAND, Player};

fn edit(pattern: u8) -> Command {
    Command::Pattern {
        device: Device::Tr808,
        pattern,
        bytes: vec![pattern; 194].into_boxed_slice(),
    }
}

/// 128 pattern edits, sent before the callback runs (nothing collected in between).
fn edits(p: &mut Player) {
    for k in 0..128 {
        send(p, edit(k as u8 % 32));
    }
}

#[test]
fn a_full_queue_holds_back_the_commands_after_a_box() {
    let (mut p, mut a) = Player::new(tables(), Some(samples()));
    edits(&mut p);
    send(&mut p, Command::Tempo(99_000));
    run(&mut a, 1);
    // the edits that fit were applied, the rest wait with the Tempo queued after them
    assert_eq!(a.garbage_room(), GARBAGE_PER_COMMAND - 1);
    assert_ne!(a.current().frame.seq.globals.tempo, 99_000);
    // a command without a box would fit; one with a box does not
    for c in [
        Command::Play,
        Command::Stop,
        Command::Locate(0),
        Command::DeviceParam {
            device: Device::Tb303_1,
            ctrl: 1,
            value: 5,
        },
    ] {
        assert!(a.room_for(&c));
    }
    assert!(!a.room_for(&edit(0)));
    // the queued ones follow once the UI side collects
    assert_eq!(p.collect_garbage(), GARBAGE - 1);
    run(&mut a, 1);
    p.collect_garbage();
    run(&mut a, 1);
    assert_eq!(a.current().frame.seq.globals.tempo, 99_000);
}

#[test]
fn a_pending_song_keeps_a_slot_for_its_play() {
    let mix = song(&manifest()["diag"]["full_mix"]);
    let (mut p, mut a) = Player::new(tables(), Some(samples()));
    assert!(p.load(&mix, Some(samples())).is_ok());
    send(&mut p, Command::Play);
    run(&mut a, 1);
    assert!(a.current().playing());
    p.collect_garbage();
    // a song loaded over the playing one waits, then the edits fill the queue
    assert!(p.load(&mix, Some(samples())).is_ok());
    edits(&mut p);
    run(&mut a, 1);
    assert!(a.has_pending());
    // an edit takes GARBAGE_PER_COMMAND slots and the one kept for the Play
    assert_eq!(a.garbage_room(), GARBAGE_PER_COMMAND);
    assert!(!a.room_for(&edit(0)));
    assert!(a.room_for(&Command::Play));
    assert!(a.room_for(&Command::Stop));
    // the Play queued after the edits is applied in its turn: the pending song becomes current
    // and plays
    send(&mut p, Command::Play);
    for _ in 0..8 {
        if !a.has_pending() {
            break;
        }
        p.collect_garbage();
        run(&mut a, 1);
    }
    assert!(!a.has_pending());
    assert!(a.current().playing());
    // and a Stop after it
    send(&mut p, Command::Stop);
    run(&mut a, 1);
    assert!(!a.current().playing());
}
