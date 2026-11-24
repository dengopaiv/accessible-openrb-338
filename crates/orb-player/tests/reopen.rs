//! The Audio Output controller on a real device (task-191): the engine survives a reopen with
//! another buffer and rate, and a request the device refuses falls back. Skips (with a
//! message) where there is no output device that opens, as on a CI runner.

mod common;

use common::{samples, tables};
use orb_player::control::{Controller, Event};
use orb_player::devices::{self, Request};
use orb_player::{Command, Player};

#[test]
fn a_reopen_keeps_the_engine_running() {
    let (mut player, audio) = Player::new(tables(), Some(samples()));
    let (mut c, events) = Controller::start(audio, Request::default());
    if !c.is_open() {
        eprintln!("skipped: no audio output opens here ({events:?})");
        return;
    }
    let u = c.in_use().unwrap();
    assert_eq!(c.request(), &Request::default());
    assert!(u.follows_default);
    eprintln!("opened: {u:?}");
    player.send(Command::Play).ok().unwrap();
    std::thread::sleep(std::time::Duration::from_millis(200));
    let before = player.snapshot().tick;
    assert!(before > 0, "the engine does not run in the stream");

    // another buffer on the same device: the engine goes on from where it was
    let r = Request {
        buffer: 512,
        ..Request::default()
    };
    let ev = c.apply(r.clone());
    assert_eq!(ev, vec![], "{ev:?}");
    let u = c.in_use().unwrap();
    assert_eq!(c.request(), &r);
    assert!(u.granted.is_some() || u.requested == Some(512), "{u:?}");
    std::thread::sleep(std::time::Duration::from_millis(200));
    assert!(
        player.snapshot().tick > before,
        "the song stopped across the reopen"
    );

    // a rate the device lists
    let caps = devices::caps(None, None).unwrap();
    if let Some(&rate) = caps.rates.iter().find(|r| **r != 44100) {
        let r2 = Request {
            rate: Some(rate),
            ..r
        };
        let ev = c.apply(r2.clone());
        assert_eq!(ev, vec![], "{ev:?}");
        assert_eq!(c.in_use().unwrap().rate, rate);
        assert!(c.in_use().unwrap().resampled);
    }

    // a rate no device lists is refused and the default request opens instead, with an error
    let bad = Request {
        rate: Some(12345),
        ..Request::default()
    };
    let ev = c.apply(bad);
    assert!(
        matches!(ev.as_slice(), [Event::Error(e)] if e.contains("12345")),
        "{ev:?}"
    );
    assert!(c.is_open());
    assert_eq!(c.request(), &Request::default());
    // nothing to poll: the default device did not change
    assert_eq!(c.poll(), vec![]);
}
