//! The Audio Output fallbacks of `Controller::start` (task-191): what a missing saved device
//! and a refused request fall back to, and that the engine comes back. They work with no output
//! device: where one opens, the stream and the exact fallback are checked; where none does (CI),
//! every request falls on to the default request, which fails too, and the tests check that.

mod common;

use common::{samples, tables};
use orb_player::Player;
use orb_player::control::{Controller, Event};
use orb_player::devices::{DeviceRef, Request};

fn errors(events: &[Event]) -> Vec<&str> {
    events
        .iter()
        .filter_map(|e| match e {
            Event::Error(m) => Some(m.as_str()),
            Event::Note(_) => None,
        })
        .collect()
}

#[test]
fn a_missing_saved_device_opens_system_default_with_a_note() {
    let (_p, audio) = Player::new(tables(), Some(samples()));
    let saved = Request {
        device: Some(DeviceRef {
            id: "no-such-host:nothing".into(),
            name: "Gone DAC".into(),
        }),
        rate: Some(48000),
        buffer: 512,
        driver: None,
    };
    let (mut c, events) = Controller::start(audio, saved.clone());
    // the note comes first, whatever the machine has
    assert!(
        matches!(events.first(), Some(Event::Note(n)) if n.contains("Gone DAC")
            && n.contains("System default")),
        "{events:?}"
    );
    // the request now runs on System default, with the rate and the buffer kept; the saved
    // choice is the settings' and Controller::start never writes it. Where that refuses (no
    // output device, or no 48 kHz at 512 frames) it falls on to the default request, with one
    // error.
    assert!(c.request().device.is_none());
    if errors(&events).is_empty() {
        assert_eq!(c.request(), &saved.on_system_default());
        assert!(c.is_open());
        assert!(c.in_use().unwrap().follows_default);
    } else {
        assert_eq!(errors(&events).len(), 1, "{events:?}");
        assert_eq!(c.request(), &Request::default());
        eprintln!("System default refused the saved rate and buffer here ({events:?})");
    }
    // the engine is not lost either way: the default request can be applied
    let ev = c.apply(Request::default());
    assert!(
        errors(&ev).iter().all(|e| !e.contains("engine is lost")),
        "{ev:?}"
    );
}

#[test]
fn a_refused_request_gives_the_defaults_with_an_error_and_the_engine_back() {
    for bad in [
        Request {
            driver: Some("NoSuchDriver".into()),
            ..Request::default()
        },
        Request {
            rate: Some(12345),
            ..Request::default()
        },
    ] {
        let (_p, audio) = Player::new(tables(), Some(samples()));
        let (mut c, events) = Controller::start(audio, bad.clone());
        let errs = errors(&events);
        assert_eq!(errs.len(), 1, "{events:?}");
        // with an output device the error names the refused driver or rate; with none (CI) the
        // first refusal may be the missing device, and the default fails too
        if c.is_open() {
            assert!(
                errs[0].contains("NoSuchDriver") || errs[0].contains("12345"),
                "{}",
                errs[0]
            );
        } else {
            assert!(errs[0].contains("runs without sound"), "{}", errs[0]);
        }
        // the request now is the default one, and says what it fell back to
        assert_eq!(c.request(), &Request::default());
        // the engine came back: nothing says it is lost, and a new request is tried
        assert!(!errs[0].contains("engine is lost"));
        let ev = c.apply(Request::default());
        assert!(
            errors(&ev).iter().all(|e| !e.contains("engine is lost")),
            "{ev:?}"
        );
    }
}

#[test]
fn the_default_request_refused_is_one_error_and_no_fallback_loop() {
    // a request that is already the default is not retried with itself: with no output device
    // it is one error, and the engine is kept for the retry
    let (_p, audio) = Player::new(tables(), Some(samples()));
    let (c, events) = Controller::start(audio, Request::default());
    if c.is_open() {
        assert!(errors(&events).is_empty(), "{events:?}");
    } else {
        assert_eq!(errors(&events).len(), 1, "{events:?}");
        assert!(errors(&events)[0].contains("runs without sound"));
    }
}

/// By hand only (`cargo test --release -p orb-player --test fallbacks -- --ignored --nocapture`,
/// task-251): ASIO with System default opens the first installed driver that opens and says
/// which ones did not. The engine is idle, so nothing is heard.
#[test]
#[ignore = "needs ASIO drivers; run by hand"]
fn asio_system_default_opens_the_first_driver_that_opens() {
    let (_p, audio) = Player::new(tables(), Some(samples()));
    let req = Request {
        driver: Some("ASIO".into()),
        ..Request::default()
    };
    eprintln!(
        "ASIO devices: {:?}",
        orb_player::devices::list_devices(Some("ASIO"))
    );
    let (mut c, events) = Controller::start(audio, req);
    eprintln!("events: {events:?}");
    eprintln!("request: {:?}", c.request());
    if let Some(u) = c.in_use() {
        eprintln!("in use: {} / {}", u.driver, u.device);
    }
    std::thread::sleep(std::time::Duration::from_secs(2));
    drop(c.apply(Request::default()));
}
