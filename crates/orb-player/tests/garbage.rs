//! task-225: a full garbage queue holds back only the commands that hand a box back. A
//! command that carries none is applied at once (`Audio::apply_now`) or in its turn
//! (`Audio::drain`), and a song's Play while it is pending always finds the slot kept for it.

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
fn commands_without_a_box_pass_a_full_queue() {
    let (mut p, mut a) = Player::new(tables(), Some(samples()));
    edits(&mut p);
    send(&mut p, Command::Tempo(99_000));
    a.drain();
    // the edits that fit were applied, the rest wait with the Tempo queued after them
    assert_eq!(a.garbage_room(), GARBAGE_PER_COMMAND - 1);
    assert_ne!(a.current().frame.seq.globals.tempo, 99_000);
    // a command without a box is applied at once
    assert!(a.apply_now(Command::Tempo(123_000)).is_ok());
    assert_eq!(a.current().frame.seq.globals.tempo, 123_000);
    for c in [
        Command::Play,
        Command::Stop,
        Command::Locate(0),
        Command::Note {
            device: Device::Tr808,
            note: 36,
            velocity: 100,
        },
        Command::DeviceParam {
            device: Device::Tb303_1,
            ctrl: 1,
            value: 5,
        },
        Command::Relocate(Device::Tb303_1),
    ] {
        assert!(a.room_for(&c));
        assert!(a.apply_now(c).is_ok());
    }
    assert_eq!(a.garbage_room(), GARBAGE_PER_COMMAND - 1);
    // one with a box is not
    assert!(a.apply_now(edit(0)).is_err());
    // the queued ones follow once the UI side collects
    assert_eq!(p.collect_garbage(), GARBAGE - 1);
    a.drain();
    p.collect_garbage();
    a.drain();
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
    a.drain();
    assert!(a.has_pending());
    // an edit takes GARBAGE_PER_COMMAND slots and the one kept for the Play
    assert_eq!(a.garbage_room(), GARBAGE_PER_COMMAND);
    assert!(!a.room_for(&edit(0)));
    assert!(a.apply_now(Command::Play).is_ok());
    assert!(!a.has_pending());
    assert!(a.current().playing());
    assert_eq!(a.garbage_room(), GARBAGE_PER_COMMAND - 1);
    // still applied: commands without a box
    assert!(a.apply_now(Command::Stop).is_ok());
    assert!(!a.current().playing());
}
