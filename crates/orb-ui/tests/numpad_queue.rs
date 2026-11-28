//! App-level tests of the numpad hook's queue (task-227, task-225's design): which frames claim
//! or drop the entries the hook (`numpad::note_message`) queued, numpad `*` as Record, and the
//! numpad's Enter in screen reader mode.
//!
//! The queue is a process-global static, so these tests are in their own file (their own
//! process) and take `LOCK`, so that no test's `end_frame` can drop another's entries.

#[path = "common/a11y.rs"]
mod a11y;

use std::sync::{Mutex, MutexGuard};

use a11y::AppTree;
use eframe::egui::accesskit::Action;
use eframe::egui::{Event, Id, Key, Modifiers};
use orb_ui::browser::Mode;
use orb_ui::model::Event as Out;
use orb_ui::numpad;
use orb_ui::screen_reader::ScreenReaderSetting;

static LOCK: Mutex<()> = Mutex::new(());

fn lock() -> MutexGuard<'static, ()> {
    let g = LOCK.lock().unwrap_or_else(|e| e.into_inner());
    numpad::end_frame();
    g
}

const WM_KEYDOWN: u32 = 0x0100;
const WM_KEYUP: u32 = 0x0101;
const VK_MULTIPLY: usize = 0x6A;
const VK_SUBTRACT: usize = 0x6D;
const VK_RETURN: usize = 0x0D;
const EXTENDED: isize = 1 << 24;

/// An app in song mode (where Record works), screen reader mode `sr`, the menu bar shown.
fn app(sr: bool) -> AppTree {
    let mut h = AppTree::new();
    h.app.settings.screen_reader = if sr {
        ScreenReaderSetting::On
    } else {
        ScreenReaderSetting::Off
    };
    h.app.panel.set_switch("transport.mode", 1);
    h.frame(vec![]);
    h.frame(vec![]);
    take(&mut h);
    h
}

fn take(h: &mut AppTree) -> Vec<Out> {
    std::mem::take(&mut h.app.panel.events)
}

fn key_event(key: Key, pressed: bool) -> Event {
    Event::Key {
        key,
        physical_key: Some(key),
        pressed,
        repeat: false,
        modifiers: Modifiers::NONE,
    }
}

/// The hook's entry of numpad `*` (it has no egui event), then a frame.
fn star(h: &mut AppTree, pressed: bool) {
    let message = if pressed { WM_KEYDOWN } else { WM_KEYUP };
    numpad::note_message(message, VK_MULTIPLY, 0);
    h.frame(vec![]);
}

fn set_record(v: i32) -> Out {
    Out::Set {
        id: "transport.record".into(),
        value: v,
    }
}

fn focus(h: &mut AppTree, id: Id) {
    h.send(id, Action::Focus, None);
    assert_eq!(h.tree().focused_id(), id.accesskit_id());
}

fn bar_has_keys(h: &AppTree) -> bool {
    orb_ui::menu::menu_has_keys(&h.frames.ctx)
}

#[test]
fn star_is_record_as_insert_is() {
    let _g = lock();
    let mut a = app(false);
    let mut b = app(false);
    // the same press and release by Insert, on a twin
    b.key(Key::Insert);
    let by_insert = take(&mut b);
    assert_eq!(by_insert, vec![set_record(1)]);
    // a press records, the release only lets go of the Record button
    star(&mut a, true);
    assert_eq!(take(&mut a), by_insert);
    assert!(a.app.panel.transport.recording);
    star(&mut a, false);
    assert!(take(&mut a).is_empty());
    assert!(a.app.panel.transport.recording);
    // the next tap toggles it off, as Insert does
    star(&mut a, true);
    star(&mut a, false);
    assert_eq!(take(&mut a), vec![set_record(0)]);
    assert!(!a.app.panel.transport.recording);
    // the entries are all used up
    a.frame(vec![]);
    assert!(take(&mut a).is_empty());
}

#[test]
fn star_pressed_while_a_dialog_has_the_keys_does_not_fire_later() {
    let _g = lock();
    let mut a = app(false);
    a.app.open_browser(Mode::OpenSong);
    a.frame(vec![]);
    // the browser has the keys: the press and release of `*` are dropped by that frame
    star(&mut a, true);
    star(&mut a, false);
    assert!(take(&mut a).is_empty());
    a.key(Key::Escape);
    a.frame(vec![]);
    a.frame(vec![]);
    assert!(take(&mut a).is_empty(), "no Record when the dialog closes");
    assert!(!a.app.panel.transport.recording);
    // the panel works again: a new press records
    star(&mut a, true);
    assert_eq!(take(&mut a), vec![set_record(1)]);
}

#[test]
fn star_pressed_while_the_menu_bar_has_the_keys_does_not_fire_later() {
    let _g = lock();
    let mut a = app(false);
    a.key(Key::F10);
    a.frame(vec![]);
    assert!(bar_has_keys(&a), "the menu bar has the keys");
    star(&mut a, true);
    star(&mut a, false);
    assert!(take(&mut a).is_empty());
    a.key(Key::Escape);
    a.frame(vec![]);
    a.frame(vec![]);
    assert!(!bar_has_keys(&a), "the menu bar let go");
    assert!(
        take(&mut a).is_empty(),
        "no Record when the menu bar closes"
    );
    assert!(!a.app.panel.transport.recording);
}

#[test]
fn an_entry_left_in_a_gated_frame_does_not_tag_a_later_key() {
    let _g = lock();
    let mut a = app(false);
    a.app.open_browser(Mode::OpenSong);
    a.frame(vec![]);
    // a numpad `-` that no event of the browser's frame claims
    numpad::note_message(WM_KEYDOWN, VK_SUBTRACT, 0);
    a.frame(vec![]);
    assert!(!numpad::claim(Key::Minus, true, true), "the entry is gone");
}

#[test]
fn numpad_enter_plays_with_a_303_section_focused_in_screen_reader_mode() {
    let _g = lock();
    let mut a = app(true);
    let mut b = app(true);
    for h in [&mut a, &mut b] {
        h.app.panel.set_switch("transport.mode", 0);
        focus(h, orb_ui::a11y::hit_id_of("knob", "tb303_2.cutoff"));
        take(h);
    }
    let step = |h: &AppTree| h.app.panel.display("tb303_2.edit_step");
    let before = step(&a);
    // the main Enter is the 303's Step there
    a.frame(vec![key_event(Key::Enter, true)]);
    a.frame(vec![key_event(Key::Enter, false)]);
    assert_ne!(step(&a), before, "the main Enter is the 303's Step");
    assert!(!a.app.panel.transport.playing);
    // the numpad's Enter (the hook saw the extended-key bit) is Play
    numpad::note_message(WM_KEYDOWN, VK_RETURN, EXTENDED);
    b.frame(vec![key_event(Key::Enter, true)]);
    numpad::note_message(WM_KEYUP, VK_RETURN, EXTENDED);
    b.frame(vec![key_event(Key::Enter, false)]);
    assert_eq!(step(&b), before, "no Step");
    assert!(b.app.panel.transport.playing, "the numpad's Enter plays");
    assert!(take(&mut b).contains(&Out::Press {
        id: "transport.play".into()
    }));
}
