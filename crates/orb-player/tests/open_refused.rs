//! A stream that cannot be opened hands the engine back (task-191), so the Audio Output window
//! can fall back and the song goes on. No audio device is needed: the refusals come before one.

mod common;

use common::{samples, tables};
use orb_player::Player;
use orb_player::devices::{DeviceRef, Request};
use orb_player::output::open;

#[test]
fn an_unknown_driver_is_refused_and_the_engine_comes_back() {
    let (_p, audio) = Player::new(tables(), Some(samples()));
    let req = Request {
        driver: Some("NoSuchDriver".into()),
        ..Request::default()
    };
    let Err(r) = open(audio, &req) else {
        panic!("opened a stream on a driver that does not exist");
    };
    assert!(r.error.0.contains("NoSuchDriver"), "{}", r.error);
    assert!(r.audio.is_some(), "the engine was not handed back");
}

#[test]
fn an_unknown_device_is_refused_and_the_engine_comes_back() {
    let (_p, audio) = Player::new(tables(), Some(samples()));
    let req = Request {
        device: Some(DeviceRef {
            id: "no-such-host:nothing".into(),
            name: "Gone".into(),
        }),
        ..Request::default()
    };
    let Err(r) = open(audio, &req) else {
        panic!("opened a stream on a device that does not exist");
    };
    assert!(r.error.0.contains("Gone"), "{}", r.error);
    assert!(r.audio.is_some());
}
