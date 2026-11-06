//! The value controls a screen reader drives (task-187, A11Y-3, docs/accessibility.md 3.3):
//! sliders, spin buttons, the pattern selectors and the buttons, toggles and switches.
//!
//! Each change is made twice, on two apps run headless from the same state: by mouse, as
//! `tests/interaction.rs` does it (the panel at 1:1, synthetic pointer events), and by the
//! screen reader's request or key. The engine events (`Panel::events`) must be equal.

#[path = "common/a11y.rs"]
mod a11y;

use a11y::AppTree;
use eframe::egui::accesskit::{Action, Role};
use eframe::egui::{Event, Key, Modifiers, PointerButton, Pos2, Vec2};
use orb_ui::a11y::{drive, names};
use orb_ui::behavior;
use orb_ui::layout::{self, Ctl};
use orb_ui::model::Event as Out;
use orb_ui::screen_reader::ScreenReaderSetting;

fn ctl(kind: &str, id: &str) -> Ctl {
    layout::controls()
        .into_iter()
        .find(|c| c.id == id && c.kind == kind)
        .unwrap_or_else(|| panic!("{kind} {id}"))
}

fn center(c: &Ctl) -> Pos2 {
    Pos2::new(c.x + c.w / 2.0, c.y + c.h / 2.0)
}

/// The app without its menu bar, so that panel pixels are points (as interaction.rs), with
/// screen reader mode set.
fn app(screen_reader: bool) -> AppTree {
    let mut h = AppTree::new();
    h.app.menu = false;
    h.app.settings.screen_reader = if screen_reader {
        ScreenReaderSetting::On
    } else {
        ScreenReaderSetting::Off
    };
    h.frame(vec![]);
    h.app.panel.events.clear();
    h
}

fn take(h: &mut AppTree) -> Vec<Out> {
    std::mem::take(&mut h.app.panel.events)
}

fn button(pos: Pos2, pressed: bool) -> Event {
    Event::PointerButton {
        pos,
        button: PointerButton::Primary,
        pressed,
        modifiers: Modifiers::NONE,
    }
}

fn click_at(h: &mut AppTree, p: Pos2) {
    h.frame(vec![Event::PointerMoved(p)]);
    h.frame(vec![button(p, true)]);
    h.frame(vec![button(p, false)]);
}

fn click(h: &mut AppTree, kind: &str, id: &str) {
    click_at(h, center(&ctl(kind, id)));
}

/// Drags from `p` up (`dy` < 0) or down in steps of 0.05 px, a frame each, until `done`;
/// the mouse is released there.
fn drag_until(h: &mut AppTree, p: Pos2, dy: f32, done: impl Fn(&AppTree) -> bool) {
    h.frame(vec![Event::PointerMoved(p)]);
    h.frame(vec![button(p, true)]);
    let mut q = p;
    for _ in 0..4000 {
        if done(h) {
            break;
        }
        q.y += dy.signum() * 0.05;
        h.frame(vec![Event::PointerMoved(q)]);
    }
    assert!(done(h), "the drag reached its value");
    h.frame(vec![button(q, false)]);
}

/// One jump of the pointer, `dy` pixels, between mouse down and up.
fn drag_by(h: &mut AppTree, p: Pos2, dy: f32) {
    let q = p + Vec2::new(0.0, dy);
    h.frame(vec![Event::PointerMoved(p)]);
    h.frame(vec![button(p, true)]);
    h.frame(vec![Event::PointerMoved(q)]);
    h.frame(vec![button(q, false)]);
}

/// Where the mouse grabs a slider: a knob's center, a fader's handle.
fn grab(h: &AppTree, c: &Ctl) -> Pos2 {
    if c.kind == "fader" {
        let top = behavior::fader_handle_top(h.app.panel.fader(&c.id));
        Pos2::new(c.x + c.w / 2.0, c.y + top + 5.5)
    } else {
        center(c)
    }
}

fn set_position(h: &mut AppTree, c: &Ctl, v: i32) {
    if c.kind == "fader" {
        h.app.panel.set_fader(&c.id, behavior::fader_from_engine(v));
    } else {
        h.app.panel.set_knob(&c.id, behavior::knob_from_engine(v));
    }
    h.frame(vec![]);
    h.app.panel.events.clear();
}

fn value(h: &AppTree, id: &str) -> i32 {
    i32::from(h.app.panel.value(id).unwrap())
}

#[test]
fn sliders_increment_decrement_and_set_as_the_mouse() {
    for (kind, id) in [
        ("knob", "tb303_1.cutoff"),
        ("knob", "tr808.bd_level"),
        ("knob", "mixer2.pan"),
        ("fader", "mixer3.level"),
        ("fader", "pcf.freq"),
    ] {
        let c = ctl(kind, id);
        let node = a11y::hit_id(kind, id);
        let (mut m, mut a) = (app(false), app(false));
        for (action, d) in [(Action::Increment, 1), (Action::Decrement, -1)] {
            set_position(&mut m, &c, 60);
            set_position(&mut a, &c, 60);
            let p = grab(&m, &c);
            drag_until(&mut m, p, -d as f32, |h| value(h, id) == 60 + d);
            a.send(node, action, None);
            let (em, ea) = (take(&mut m), take(&mut a));
            assert_eq!(
                em,
                vec![Out::Set {
                    id: id.into(),
                    value: 60 + d
                }],
                "{id}"
            );
            assert_eq!(ea, em, "{id}: {action:?}");
        }
        // a mouse jump to some value, and SetValue to the same value
        set_position(&mut m, &c, 30);
        set_position(&mut a, &c, 30);
        let p = grab(&m, &c);
        drag_by(&mut m, p, -20.0);
        let v = value(&m, id);
        assert!(v > 30, "{id}: {v}");
        a.frame(vec![a11y::set_value_request(node, f64::from(v))]);
        a.frame(vec![]);
        let (em, ea) = (take(&mut m), take(&mut a));
        assert_eq!(em.len(), 1, "{id}: {em:?}");
        assert_eq!(ea, em, "{id}: SetValue");
        // the node reads the new value
        let n = a.tree().by_layout(kind, id).unwrap();
        assert_eq!(a11y::numeric_value(n), Some(f64::from(v)));
        // the harness is in mode off: the plain 0..127 text
        let text = names::control_value_text(&a.app.panel, id, v as u8, names::Speech::Plain);
        assert_eq!(a11y::value(n), Some(text.as_str()));
        for action in [Action::Increment, Action::Decrement, Action::SetValue] {
            assert!(a11y::supports(n, action), "{id}: {action:?}");
        }
    }
}

#[test]
fn spin_buttons_click_their_arrows() {
    let (mut m, mut a) = (app(false), app(false));
    // the transport's bar and loop arrows work in song mode only
    for h in [&mut m, &mut a] {
        h.app.panel.press("transport.mode", 0.0);
        h.frame(vec![]);
        h.app.panel.events.clear();
    }
    for display in drive::SPIN_DISPLAYS {
        let node = a11y::hit_id("display", display);
        for (action, arrow) in [(Action::Increment, "up"), (Action::Decrement, "down")] {
            let id = format!("{display}_{arrow}");
            let kind = if display == "transport.bar" {
                "wind"
            } else {
                "repeat"
            };
            click(&mut m, kind, &id);
            a.send(node, action, None);
            let (em, ea) = (take(&mut m), take(&mut a));
            assert!(!em.is_empty(), "{id} sends");
            assert_eq!(ea, em, "{id}");
        }
        assert_eq!(
            m.app.panel.display(display),
            a.app.panel.display(display),
            "{display}"
        );
    }
}

#[test]
fn the_pattern_selector_clicks_bank_and_pattern() {
    let (mut m, mut a) = (app(false), app(false));
    for r in 0..4 {
        let n = r + 1;
        let node = drive::selector_id(r);
        let start = a.app.panel.rows[r].pattern;
        // up to A8 one by one, then B1: the bank button, then the pattern button
        for p in start + 1..=8 {
            let bank = char::from(b'A' + p / 8);
            if m.app.panel.rows[r].bank != p / 8 {
                click(&mut m, "grid", &format!("pattern{n}.bank{bank}"));
            }
            click(&mut m, "grid", &format!("pattern{n}.pattern{}", p % 8 + 1));
            a.send(node, Action::Increment, None);
            let (em, ea) = (take(&mut m), take(&mut a));
            assert_eq!(em, vec![Out::Pattern { row: r, pattern: p }], "row {n}");
            assert_eq!(ea, em, "row {n} pattern {p}");
            assert_eq!(a.app.panel.focus, m.app.panel.focus);
        }
        // and back to A8
        click(&mut m, "grid", &format!("pattern{n}.bankA"));
        click(&mut m, "grid", &format!("pattern{n}.pattern8"));
        a.send(node, Action::Decrement, None);
        assert_eq!(take(&mut a), take(&mut m), "row {n} back");
        let sel = a.tree().by_id(node).unwrap();
        assert_eq!(a11y::value(sel), Some("A8"));
    }
}

#[test]
fn buttons_toggles_and_switches_take_the_click_request() {
    let (mut m, mut a) = (app(false), app(false));
    for (kind, id) in [
        ("toggle", "transport.loop"),
        ("toggle", "pattern1.shuffle"),
        ("switch", "transport.mode"),
        ("hswitch", "tb303_1.wave"),
        ("vswitch", "pcf.mode"),
        ("radio", "mixer2.pcf"),
        ("trigger", "tb303_2.key3"),
        ("trigger", "transport.play"),
        ("trigger", "transport.stop"),
        ("wind", "transport.ff"),
    ] {
        click(&mut m, kind, id);
        a.send(a11y::hit_id(kind, id), Action::Click, None);
        let (em, ea) = (take(&mut m), take(&mut a));
        assert!(!em.is_empty(), "{id} sends");
        assert_eq!(ea, em, "{kind} {id}");
    }
}

#[test]
fn one_node_per_spin_button_and_pattern_selector() {
    let a = app(false);
    let t = a.tree();
    for c in layout::controls() {
        if drive::joined(&c) {
            assert!(t.by_layout(&c.kind, &c.id).is_none(), "{} has a node", c.id);
        }
    }
    let spins: Vec<_> = t.by_role(Role::SpinButton).collect();
    assert_eq!(spins.len(), 10 + 4);
    for display in drive::SPIN_DISPLAYS {
        let n = t.by_layout("display", display).unwrap();
        assert_eq!(a11y::role(n), Role::SpinButton, "{display}");
        assert!(a11y::supports(n, Action::Focus), "{display}");
        assert!(!a11y::supports(n, Action::Click), "{display}");
        assert!(a11y::supports(n, Action::Increment), "{display}");
    }
    for r in 0..4 {
        let n = t.by_id(drive::selector_id(r)).unwrap();
        assert_eq!(a11y::name(n), Some("Pattern"));
        assert!(a11y::supports(n, Action::Focus));
    }
    // the 303's edit step has no arrows: still a label
    let n = t.by_layout("display", "tb303_1.edit_step").unwrap();
    assert_eq!(a11y::role(n), Role::Label);
}

#[test]
fn values_read_as_words_and_numbers() {
    let mut a = app(false);
    a.app
        .panel
        .set_knob("mixer1.pan", behavior::knob_from_engine(64));
    a.app
        .panel
        .set_knob("tb303_1.cutoff", behavior::knob_from_engine(64));
    while a.app.panel.transport.tempo < 128_000 {
        a.app.panel.arrow("transport.tempo_up");
    }
    while a.app.panel.transport.tempo > 128_000 {
        a.app.panel.arrow("transport.tempo_down");
    }
    for _ in 0..10 {
        a.send(drive::selector_id(2), Action::Increment, None);
    }
    let t = a.frame(vec![]);
    let read = |kind: &str, id: &str| {
        let n = t.by_layout(kind, id).unwrap();
        (a11y::name(n).unwrap(), a11y::value(n).unwrap())
    };
    assert_eq!(read("knob", "tb303_1.cutoff"), ("Cutoff", "64"));
    assert_eq!(read("knob", "mixer1.pan"), ("Channel 1 pan", "center"));
    assert_eq!(read("display", "transport.tempo"), ("Tempo", "128 BPM"));
    assert_eq!(
        read("display", "pattern2.steps"),
        ("Pattern length", "16 steps")
    );
    // the transport's bar is off in pattern mode
    assert_eq!(read("display", "transport.bar"), ("Bar", "off"));
    let sel = t.by_id(drive::selector_id(2)).unwrap();
    assert_eq!(a11y::value(sel), Some("B3"));
}

fn key_with(key: Key, pressed: bool, repeat: bool) -> Event {
    Event::Key {
        key,
        physical_key: Some(key),
        pressed,
        repeat,
        modifiers: Modifiers::NONE,
    }
}

#[test]
fn keys_change_the_focused_control_in_screen_reader_mode() {
    let mut a = app(true);
    let focus = |a: &mut AppTree, id| {
        a.send(id, Action::Focus, None);
        assert_eq!(a.tree().focused_id(), id.accesskit_id());
    };
    let cutoff = a11y::hit_id("knob", "tb303_1.cutoff");
    focus(&mut a, cutoff);
    let row = a.app.panel.focus;
    a.app
        .panel
        .set_knob("tb303_1.cutoff", behavior::knob_from_engine(60));
    for (key, v) in [
        (Key::ArrowUp, 61),
        (Key::ArrowDown, 60),
        (Key::PageUp, 68),
        (Key::PageDown, 60),
        (Key::End, 127),
        (Key::Home, 0),
    ] {
        a.key(key);
        assert_eq!(value(&a, "tb303_1.cutoff"), v, "{key:?}");
    }
    // auto-repeat counts; the focus and the row focus stay
    a.frame(vec![key_with(Key::ArrowUp, true, false)]);
    a.frame(vec![key_with(Key::ArrowUp, true, true)]);
    a.frame(vec![key_with(Key::ArrowUp, false, false)]);
    assert_eq!(value(&a, "tb303_1.cutoff"), 2);
    assert_eq!(a.tree().focused_id(), cutoff.accesskit_id());
    assert_eq!(
        a.app.panel.focus, row,
        "Up and Down do not move the row focus"
    );

    let fader = a11y::hit_id("fader", "master.fader");
    focus(&mut a, fader);
    a.key(Key::Home);
    a.key(Key::PageUp);
    assert_eq!(value(&a, "master.fader"), 8);

    let tempo = a11y::hit_id("display", "transport.tempo");
    focus(&mut a, tempo);
    let bpm = a.app.panel.display("transport.tempo").unwrap();
    a.key(Key::ArrowUp);
    a.key(Key::ArrowUp);
    a.key(Key::ArrowDown);
    assert_eq!(a.app.panel.display("transport.tempo"), Some(bpm + 1));
    // Home and End do nothing on tempo (TASK-205)
    a.app.panel.events.clear();
    a.key(Key::End);
    a.key(Key::Home);
    assert!(a.app.panel.events.is_empty());

    let sel = drive::selector_id(3);
    focus(&mut a, sel);
    a.key(Key::PageUp);
    a.key(Key::ArrowUp);
    assert_eq!(a.app.panel.rows[3].pattern, 9);
    a.key(Key::End);
    assert_eq!(a.app.panel.rows[3].pattern, 31);
    let t = a.key(Key::Home);
    assert_eq!(a11y::value(t.by_id(sel).unwrap()), Some("A1"));
}

/// TASK-205 (the user, 2026-11-06): on a spin button Page Up and Page Down are 10 clicks of its
/// arrow, Home and End click to the end of a short range and do nothing on tempo, bar, loop
/// start and loop length.
#[test]
fn page_home_and_end_on_a_spin_button() {
    let (mut m, mut a) = (app(false), app(true));
    for h in [&mut m, &mut a] {
        h.app.panel.press("transport.mode", 0.0);
        h.frame(vec![]);
        h.app.panel.events.clear();
    }
    for display in drive::SPIN_DISPLAYS {
        let node = a11y::hit_id("display", display);
        a.send(node, Action::Focus, None);
        for (key, arrow) in [(Key::PageUp, "up"), (Key::PageDown, "down")] {
            let kind = if display == "transport.bar" {
                "wind"
            } else {
                "repeat"
            };
            for _ in 0..10 {
                click(&mut m, kind, &format!("{display}_{arrow}"));
            }
            a.key(key);
            let (em, ea) = (take(&mut m), take(&mut a));
            // one event per click (two for loop start: start and end)
            let per_click = if display == "transport.loop_start" {
                2
            } else {
                1
            };
            assert_eq!(em.len(), 10 * per_click, "{display} {key:?}: {em:?}");
            assert_eq!(ea, em, "{display} {key:?}");
            assert_eq!(a.app.panel.display(display), m.app.panel.display(display));
        }
        let before = a.app.panel.display(display);
        match drive::short_range(display) {
            Some((lo, hi)) => {
                a.key(Key::End);
                assert_eq!(a.app.panel.display(display), Some(hi), "{display} End");
                // as many clicks as values crossed, each sent
                let sent = take(&mut a).len() as i32;
                assert_eq!(sent, hi - before.unwrap(), "{display} End");
                a.key(Key::End);
                assert!(take(&mut a).is_empty(), "{display}: End at the end");
                a.key(Key::Home);
                assert_eq!(a.app.panel.display(display), Some(lo), "{display} Home");
                assert_eq!(take(&mut a).len() as i32, hi - lo, "{display} Home");
                // back to where the mouse app is
                while a.app.panel.display(display) != before {
                    a.app.panel.arrow(&format!("{display}_up"));
                }
                a.app.panel.events.clear();
            }
            None => {
                a.key(Key::End);
                a.key(Key::Home);
                assert!(
                    take(&mut a).is_empty(),
                    "{display}: Home and End do nothing"
                );
                assert_eq!(a.app.panel.display(display), before);
            }
        }
    }
}

#[test]
fn with_the_mode_off_the_keys_are_the_originals() {
    let mut a = app(false);
    let cutoff = a11y::hit_id("knob", "tb303_1.cutoff");
    a.send(cutoff, Action::Focus, None);
    let v = value(&a, "tb303_1.cutoff");
    let row = a.app.panel.focus;
    a.key(Key::ArrowDown);
    assert_eq!(value(&a, "tb303_1.cutoff"), v);
    // Down is the original's row focus
    assert_eq!(a.app.panel.focus, (row + 1) % 4);
    a.key(Key::End);
    a.key(Key::PageUp);
    assert_eq!(value(&a, "tb303_1.cutoff"), v);
}

/// With the mode off, a focused slider, spin button or pattern selector changes nothing: every
/// value key does what it does with nothing focused (main's behavior: Up and Down move the row
/// focus, the others nothing), compared on the engine events and the panel's state.
#[test]
fn with_the_mode_off_a_focused_value_control_takes_no_keys() {
    let keys = [
        Key::ArrowUp,
        Key::ArrowDown,
        Key::PageUp,
        Key::PageDown,
        Key::Home,
        Key::End,
    ];
    let state = |h: &AppTree| {
        let p = &h.app.panel;
        (
            p.focus,
            p.value("tb303_1.cutoff"),
            p.transport.tempo,
            p.display("pattern2.steps"),
            p.rows
                .iter()
                .map(|r| (r.pattern, r.bank))
                .collect::<Vec<_>>(),
        )
    };
    for node in [
        a11y::hit_id("knob", "tb303_1.cutoff"),
        a11y::hit_id("display", "transport.tempo"),
        a11y::hit_id("display", "pattern2.steps"),
        drive::selector_id(1),
    ] {
        let (mut focused, mut plain) = (app(false), app(false));
        focused.send(node, Action::Focus, None);
        assert_eq!(focused.tree().focused_id(), node.accesskit_id());
        focused.app.panel.events.clear();
        for key in keys {
            focused.key(key);
            plain.key(key);
            assert_eq!(take(&mut focused), take(&mut plain), "{key:?}");
            assert_eq!(state(&focused), state(&plain), "{key:?}");
        }
    }
    // and Up alone is the original's row focus up (row 1 wraps to row 4)
    let mut a = app(false);
    a.send(drive::selector_id(0), Action::Focus, None);
    let row = a.app.panel.focus;
    a.key(Key::ArrowUp);
    assert_eq!(a.app.panel.focus, (row + 3) % 4);
}
