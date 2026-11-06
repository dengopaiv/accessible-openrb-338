//! The dialogs and windows in screen reader mode (task-193, A11Y-8, docs/ui_spec.md section
//! 20.4): a role and a name, the focus moving in as they open and back as they close, the
//! error box as an alert, and the texts that can be read line by line.

#[path = "common/a11y.rs"]
mod a11y;

use a11y::AppTree;
use eframe::egui::accesskit::{Action, NodeId, Role};
use eframe::egui::{Id, Key};
use orb_ui::app::App;
use orb_ui::screen_reader::ScreenReaderSetting;

fn app(screen_reader: bool) -> AppTree {
    let mut h = AppTree::new();
    h.app.settings.screen_reader = if screen_reader {
        ScreenReaderSetting::On
    } else {
        ScreenReaderSetting::Off
    };
    h.frame(vec![]);
    h
}

fn settle(a: &mut AppTree) {
    for _ in 0..3 {
        a.frame(vec![]);
    }
}

fn focus_play(a: &mut AppTree) -> Id {
    let play = a11y::hit_id("trigger", "transport.play");
    a.send(play, Action::Focus, None);
    assert_eq!(a.tree().focused_id(), play.accesskit_id());
    play
}

fn focused(a: &AppTree) -> NodeId {
    a.tree().focused_id()
}

/// The text a screen reader reads for a node: its name, else its value.
fn spoken(a: &AppTree, id: NodeId) -> String {
    let n = a.tree().node(id).expect("the node");
    a11y::name(n)
        .or(a11y::value(n))
        .unwrap_or_default()
        .to_string()
}

fn role_of(a: &AppTree, id: NodeId) -> Role {
    a11y::role(a.tree().node(id).expect("the node"))
}

/// The named Dialog (or AlertDialog) nodes of the tree.
fn dialogs(a: &AppTree, role: Role) -> Vec<String> {
    a.tree()
        .by_role(role)
        .map(|(_, n)| a11y::name(n).unwrap_or_default().to_string())
        .collect()
}

type Open = fn(&mut App);

const WINDOWS: [(&str, &str, Open); 4] = [
    ("keys", "Keyboard Shortcuts", |a| a.show_key_help()),
    ("info", "Song Information", |a| a.show_info()),
    ("about", "About OpenRB-338", |a| a.show_about()),
    ("mod info", "Mod Information", |a| a.show_mod_info()),
];

/// Each window is a Dialog named by its title; the focus goes into it and comes back to the
/// panel control that had it; its close button has a name.
#[test]
fn each_window_is_a_named_dialog_with_the_focus_in_and_back() {
    for (what, title, open) in WINDOWS {
        let mut a = app(true);
        let play = focus_play(&mut a);
        open(&mut a.app);
        settle(&mut a);
        assert_eq!(dialogs(&a, Role::Dialog), vec![title], "{what}");
        assert!(
            a.tree()
                .by_role(Role::Window)
                .all(|(_, n)| a11y::name(n) != Some(title)),
            "{what}: the window node is the dialog"
        );
        let f = focused(&a);
        assert_ne!(f, play.accesskit_id(), "{what}: the focus left the panel");
        // the focus is inside the dialog
        let mut at = f;
        let mut inside = false;
        while let Some((p, n)) = a.tree().parent(at) {
            if a11y::role(n) == Role::Dialog {
                inside = true;
                break;
            }
            at = p;
        }
        assert!(inside, "{what}: the focus is inside the dialog");
        let close: Vec<_> = a.tree().by_name("Close window").collect();
        assert_eq!(close.len(), 1, "{what}: a named close button");
        assert_eq!(a11y::role(close[0].1), Role::Button, "{what}");
        a.key(Key::Escape);
        settle(&mut a);
        assert!(dialogs(&a, Role::Dialog).is_empty(), "{what}: closed");
        assert_eq!(
            focused(&a),
            play.accesskit_id(),
            "{what}: the focus is back"
        );
    }
}

/// With the mode off the windows are what egui makes of them: a Window with its title, no
/// Dialog, and the focus is not moved.
#[test]
fn with_the_mode_off_the_windows_are_as_before() {
    for (what, title, open) in WINDOWS {
        let mut a = app(false);
        open(&mut a.app);
        settle(&mut a);
        assert!(dialogs(&a, Role::Dialog).is_empty(), "{what}");
        assert!(
            a.tree()
                .by_role(Role::Window)
                .any(|(_, n)| a11y::name(n) == Some(title)),
            "{what}"
        );
    }
}

/// The lines of Keyboard Shortcuts take the focus one after the other and are read as
/// "keys: what".
#[test]
fn the_keyboard_shortcuts_can_be_read_line_by_line() {
    let mut a = app(true);
    focus_play(&mut a);
    a.app.show_key_help();
    settle(&mut a);
    assert!(spoken(&a, focused(&a)).starts_with("Keyboard mode now"));
    let mut seen = vec![];
    for _ in 0..8 {
        a.key(Key::Tab);
        let f = focused(&a);
        assert_eq!(role_of(&a, f), Role::Label, "a text line");
        seen.push(spoken(&a, f));
    }
    println!("{seen:#?}");
    assert!(seen.iter().all(|s| !s.is_empty()));
    assert!(
        seen.iter().any(|s| s.contains(": ")),
        "a row reads as keys: what"
    );
    // every line differs from the one before: the focus moves
    assert!(seen.windows(2).all(|w| w[0] != w[1]));
}

/// About: its lines (the version, the labels) and its links take the focus.
#[test]
fn about_can_be_read_line_by_line() {
    let mut a = app(true);
    focus_play(&mut a);
    a.app.show_about();
    settle(&mut a);
    let first = spoken(&a, focused(&a));
    assert!(first.starts_with("Version "), "{first}");
    let mut seen = vec![first];
    for _ in 0..4 {
        a.key(Key::Tab);
        seen.push(spoken(&a, focused(&a)));
    }
    println!("{seen:#?}");
    assert!(seen.iter().any(|s| s == "Song collection:"));
    assert!(seen.iter().any(|s| s == "Mod collection:"));
    assert!(seen.iter().any(|s| s.starts_with("https://")));
}

/// Mod Information: a row reads as "Name: value".
#[test]
fn mod_info_can_be_read_line_by_line() {
    let mut a = app(true);
    focus_play(&mut a);
    a.app.show_mod_info();
    settle(&mut a);
    let mut seen = vec![spoken(&a, focused(&a))];
    for _ in 0..6 {
        a.key(Key::Tab);
        seen.push(spoken(&a, focused(&a)));
    }
    println!("{seen:#?}");
    assert!(seen.iter().any(|s| s.starts_with("Name: ")), "{seen:?}");
}

/// The unsaved-changes prompt is a named Dialog, the focus is on Save, and it comes back.
#[test]
fn the_prompt_takes_the_focus_on_save_and_gives_it_back() {
    let mut a = app(true);
    let play = focus_play(&mut a);
    a.app.doc.modified = true;
    assert!(!a.app.request_close());
    settle(&mut a);
    assert!(a.app.prompting());
    let d = dialogs(&a, Role::Dialog);
    assert_eq!(d.len(), 1);
    assert!(d[0].starts_with("Save changes to"), "{d:?}");
    let f = focused(&a);
    assert_eq!(spoken(&a, f), "Save");
    assert_eq!(role_of(&a, f), Role::Button);
    a.key(Key::Escape);
    settle(&mut a);
    assert!(!a.app.prompting());
    assert_eq!(focused(&a), play.accesskit_id(), "the focus is back");
    // with the mode off the focus is not moved
    let mut b = app(false);
    b.app.doc.modified = true;
    assert!(!b.app.request_close());
    settle(&mut b);
    assert!(b.app.prompting());
    assert_ne!(spoken(&b, focused(&b)), "Save");
}

/// The error box is an AlertDialog named Error whose description is the message, the focus is
/// on OK, the close button is named, and the focus comes back to the panel.
#[test]
fn the_error_box_is_an_alert_with_the_focus_on_ok() {
    let mut a = app(true);
    let play = focus_play(&mut a);
    a.app.fail("The song could not be read");
    settle(&mut a);
    assert!(a.app.error_open());
    let alerts: Vec<_> = a.tree().by_role(Role::AlertDialog).collect();
    assert_eq!(alerts.len(), 1, "one alert");
    assert_eq!(alerts[0].1.label(), Some("Error"));
    assert_eq!(
        alerts[0].1.description(),
        Some("The song could not be read")
    );
    let f = focused(&a);
    assert_eq!(spoken(&a, f), "OK");
    assert!(
        a.tree().by_name("Close").count() == 1,
        "the close button is named"
    );
    // Escape closes it, and the focus is back on the panel control
    a.key(Key::Escape);
    settle(&mut a);
    assert!(!a.app.error_open());
    assert_eq!(focused(&a), play.accesskit_id(), "the focus is back");
    // two errors: the second box takes the focus on OK too
    a.app.fail("first");
    a.app.fail("second");
    settle(&mut a);
    a.key(Key::Enter);
    settle(&mut a);
    assert!(a.app.error_open());
    assert_eq!(spoken(&a, focused(&a)), "OK");
}

/// A window open under the error box gets the focus back when the box closes.
#[test]
fn the_focus_returns_to_the_window_under_the_error_box() {
    let mut a = app(true);
    focus_play(&mut a);
    a.app.show_about();
    settle(&mut a);
    a.app.fail("an error");
    settle(&mut a);
    assert_eq!(spoken(&a, focused(&a)), "OK");
    a.app.dismiss_error();
    settle(&mut a);
    assert!(a.app.about_is_open());
    let f = focused(&a);
    assert!(spoken(&a, f).starts_with("Version "), "{}", spoken(&a, f));
}

/// With the mode off the error box does not take the focus from the panel.
#[test]
fn with_the_mode_off_the_error_box_is_as_before() {
    let mut a = app(false);
    a.app.fail("an error");
    settle(&mut a);
    assert!(a.tree().by_role(Role::AlertDialog).next().is_none());
}

/// The focus comes back two frames after a modal's last open frame, and the frame in between
/// asks for a repaint, so that the second one comes with no input at all.
#[test]
fn the_focus_return_asks_for_the_frames_it_needs() {
    let mut a = app(true);
    let play = focus_play(&mut a);
    a.app.fail("an error");
    settle(&mut a);
    // an open modal does not keep the app repainting
    for _ in 0..3 {
        a.frame(vec![]);
        assert!(
            !a.frames.ctx.has_requested_repaint(),
            "idle with the box open"
        );
    }
    a.app.dismiss_error();
    // the frame after the last open one: the focus is not back yet, a repaint is requested
    a.frame(vec![]);
    assert!(
        a.frames.ctx.has_requested_repaint(),
        "a repaint is requested"
    );
    assert_ne!(focused(&a), play.accesskit_id());
    // the frame that request brings restores the focus; the tree shows it a frame later
    a.frame(vec![]);
    a.frame(vec![]);
    assert_eq!(focused(&a), play.accesskit_id(), "back with no input");
}
