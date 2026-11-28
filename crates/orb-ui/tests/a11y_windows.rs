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
    // one more stop since task-233: upstream's copy-version button after the version
    for _ in 0..6 {
        a.key(Key::Tab);
        seen.push(spoken(&a, focused(&a)));
    }
    println!("{seen:#?}");
    assert_eq!(seen[1], "Copy version");
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

/// Initialize Song's question (task-237) is a named Dialog whose description is the original's
/// text, the focus is on Initialize, Tab reaches Cancel, and the focus comes back after either
/// answer. With the mode off the focus is not moved.
#[test]
fn the_init_song_question_takes_the_focus_on_initialize_and_gives_it_back() {
    let mut a = app(true);
    let play = focus_play(&mut a);
    a.app.ask_init_for_test();
    settle(&mut a);
    assert!(a.app.confirming_init());
    let dlg: Vec<_> = a.tree().by_role(Role::Dialog).collect();
    assert_eq!(dlg.len(), 1, "one dialog");
    assert_eq!(dlg[0].1.label(), Some("Initialize Song from Pattern Mode"));
    assert_eq!(dlg[0].1.description(), Some(orb_ui::menu::INIT_SONG_TEXT));
    let f = focused(&a);
    assert_eq!(spoken(&a, f), "Initialize");
    assert_eq!(role_of(&a, f), Role::Button);
    // the button lies inside the Dialog that carries the text, so it is read with it
    let mut up = f;
    let mut found = false;
    while let Some((p, n)) = a.tree().parent(up) {
        if n.description() == Some(orb_ui::menu::INIT_SONG_TEXT) {
            found = true;
            break;
        }
        up = p;
    }
    assert!(found, "Initialize is not under the Dialog with the text");
    // Tab reaches Cancel (and stays in the question)
    let mut seen = vec![];
    for _ in 0..2 {
        a.key(Key::Tab);
        seen.push(spoken(&a, focused(&a)));
    }
    assert!(seen.iter().any(|s| s == "Cancel"), "{seen:?}");
    assert!(seen.iter().any(|s| s == "Initialize"), "{seen:?}");
    // Escape cancels; the focus is back on the panel control
    a.key(Key::Escape);
    settle(&mut a);
    assert!(!a.app.confirming_init());
    assert_eq!(focused(&a), play.accesskit_id(), "the focus is back");
    // the second time it takes the focus again; Enter (Initialize) gives it back too
    a.app.ask_init_for_test();
    settle(&mut a);
    assert_eq!(spoken(&a, focused(&a)), "Initialize");
    a.key(Key::Enter);
    settle(&mut a);
    assert!(!a.app.confirming_init());
    assert_eq!(focused(&a), play.accesskit_id(), "the focus is back");
    // with the mode off the focus is not moved
    let mut b = app(false);
    b.app.ask_init_for_test();
    settle(&mut b);
    assert!(b.app.confirming_init());
    assert_ne!(spoken(&b, focused(&b)), "Initialize");
    // (egui's own modal node stays; the fork's name and description are not set)
    assert!(
        b.tree()
            .by_role(Role::Dialog)
            .all(|(_, n)| n.description().is_none())
    );
}

/// Opens Audio Output with fixed lists: no driver or device is asked.
fn open_audio(a: &mut AppTree) {
    use orb_player::devices::{Caps, DeviceInfo, SupportedBufferSize};
    a.app.audio_open_for_test(
        vec!["WASAPI".into(), "ASIO".into()],
        vec![DeviceInfo {
            id: "a".into(),
            name: "Speakers".into(),
        }],
        Some(Caps {
            rates: vec![44100, 48000],
            buffers: vec![256, 512, 1024],
            range: SupportedBufferSize::Range {
                min: 256,
                max: 1024,
            },
        }),
    );
}

/// Options > Audio Output (task-234): a named Dialog, the focus goes in on the Driver combo box
/// and comes back on close (Escape), every combo box says what it chooses and holds the chosen
/// value, and the In use line can be reached.
#[test]
fn audio_output_takes_the_focus_and_names_its_combo_boxes() {
    let mut a = app(true);
    let play = focus_play(&mut a);
    open_audio(&mut a);
    settle(&mut a);
    assert!(a.app.audio_is_open());
    assert_eq!(dialogs(&a, Role::Dialog), vec!["Audio Output"]);
    // the four combo boxes, by name, with the chosen value
    let mut combos: Vec<(String, String)> = a
        .tree()
        .by_role(Role::ComboBox)
        .map(|(_, n)| {
            (
                a11y::name(n).unwrap_or_default().to_string(),
                a11y::value(n).unwrap_or_default().to_string(),
            )
        })
        .collect();
    combos.sort();
    let names: Vec<&str> = combos.iter().map(|(n, _)| n.as_str()).collect();
    assert_eq!(
        names,
        ["Buffer size", "Device", "Driver", "Sample rate"],
        "{combos:?}"
    );
    // the 2048 asked for is clamped into the device's range (up to 1024)
    assert!(combos[0].1.starts_with("1024 frames"), "{combos:?}");
    assert_eq!(combos[1].1, "System default");
    assert_eq!(combos[2].1, "WASAPI");
    assert_eq!(combos[3].1, "Device default");
    // the focus is in the window, on the first combo box
    let f = focused(&a);
    assert_eq!(role_of(&a, f), Role::ComboBox);
    assert_eq!(a11y::name(a.tree().node(f).unwrap()), Some("Driver"));
    // Tab walks the combo boxes and then the In use line
    let mut seen = vec![];
    for _ in 0..6 {
        a.key(Key::Tab);
        let n = a.tree().node(focused(&a)).unwrap();
        seen.push(spoken(&a, focused(&a)));
        let _ = n;
    }
    for want in [
        "Device",
        "Sample rate",
        "Buffer size",
        "Output: no audio output",
    ] {
        assert!(seen.iter().any(|s| s == want), "{want}: {seen:?}");
    }
    // Escape closes it and the focus is back
    a.key(Key::Escape);
    settle(&mut a);
    assert!(!a.app.audio_is_open());
    assert!(dialogs(&a, Role::Dialog).is_empty());
    assert_eq!(focused(&a), play.accesskit_id(), "the focus is back");
}

/// The window open for seconds (the 500 ms In use refresh, the repaints) says nothing; a change
/// is one message with what changed.
#[test]
fn audio_output_says_a_change_once_and_never_the_refresh() {
    let mut a = app(true);
    open_audio(&mut a);
    settle(&mut a);
    let before = a.app.announcer.serial();
    // many frames (the In use sampling is timed by the wall clock and has its own unit
    // tests; here no repaint or input says anything)
    for _ in 0..130 {
        a.frame(vec![]);
    }
    assert_eq!(a.app.announcer.serial(), before, "the refresh is silent");
    let old = orb_ui::settings::AudioChoice::default();
    let new = orb_ui::settings::AudioChoice {
        rate: Some(48000),
        ..old.clone()
    };
    let t = orb_ui::audio_window::change_text(&old, &new, &["WASAPI".to_string()]);
    assert_eq!(t.as_deref(), Some("Sample rate: 48000 Hz"));
}

/// After the controller fell back from a lost device (task-245) the Device combo box shows
/// System default, and only the controller's note is said.
#[test]
fn audio_output_follows_a_fallback_and_says_only_the_note() {
    use orb_player::control::{Controller, Event};
    for sr in [true, false] {
        let mut a = app(sr);
        a.app.settings.audio = orb_ui::settings::AudioChoice {
            device: Some("a".into()),
            device_name: Some("Speakers".into()),
            rate: Some(48000),
            ..Default::default()
        };
        open_audio(&mut a);
        settle(&mut a);
        let device_value = |a: &AppTree| {
            a.tree()
                .by_name("Device")
                .filter_map(|(_, n)| a11y::value(n).map(str::to_string))
                .next()
        };
        if sr {
            assert_eq!(device_value(&a).as_deref(), Some("Speakers"));
        }
        // the controller ran the same request on the system default (rate kept)
        let req = a.app.settings.audio.request().on_system_default();
        assert_eq!(req.rate, Some(48000));
        a.app.audio = Some(Controller::for_test(req));
        let before = a.app.announcer.serial();
        let note = "Audio device Speakers is not available; using System default";
        a.app.audio_events(vec![Event::Note(note.into())]);
        // one announcement, the controller's own note
        assert_eq!(
            a.app.announcer.serial() - before,
            if sr { 1 } else { 0 },
            "sr {sr}"
        );
        assert_eq!(a.app.status.as_deref(), Some(note));
        // the window shows what runs; the saved choice is the user's
        assert_eq!(a.app.audio_choice().device, None);
        assert_eq!(a.app.audio_choice().rate, Some(48000));
        assert_eq!(a.app.settings.audio.device.as_deref(), Some("a"));
        settle(&mut a);
        if sr {
            assert_eq!(device_value(&a).as_deref(), Some("System default"));
        }
        // a later poll that reports nothing says nothing
        let n = a.app.announcer.serial();
        a.app.audio_events(vec![]);
        assert_eq!(a.app.announcer.serial(), n);
    }
}

/// The fallback is never saved: a later save of another setting keeps the saved device; a
/// saved device missing at the start shows as System default (the real order of the start); the
/// next start that finds the device shows it, with nothing written (task-245).
#[test]
fn a_fallback_is_shown_and_never_saved() {
    use orb_player::control::Controller;
    use orb_ui::settings::{AudioChoice, Settings, Theme};
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("OpenRB-338.toml");
    let saved = AudioChoice {
        device: Some("a".into()),
        device_name: Some("Speakers".into()),
        ..Default::default()
    };
    let mut s = Settings::load_from(&file);
    s.set_audio(saved.clone());
    s.save().unwrap();
    // the start, as main.rs does it: App::new has the default settings, then the controller and
    // the saved settings are adopted; the saved device was missing, the controller runs the
    // system default
    let mut a = AppTree::new();
    let mut loaded = Settings::load_from(&file);
    loaded.screen_reader = ScreenReaderSetting::On;
    assert_eq!(loaded.audio, saved);
    a.app.adopt_audio_and_settings(
        Some(Controller::for_test(saved.request().on_system_default())),
        loaded,
    );
    a.frame(vec![]);
    open_audio(&mut a);
    settle(&mut a);
    let value = |a: &AppTree| {
        a.tree()
            .by_name("Device")
            .filter_map(|(_, n)| a11y::value(n).map(str::to_string))
            .next()
    };
    assert_eq!(value(&a).as_deref(), Some("System default"));
    // another setting is saved: the file keeps the device
    a.app.settings.set_theme(Theme::Classic);
    a.app.settings.save().unwrap();
    let back = Settings::load_from(&file);
    assert_eq!(back.audio.device.as_deref(), Some("a"));
    assert_eq!(back.theme, Theme::Classic);
    // the next start finds the saved device: the window shows it, nothing was written
    let mut b = AppTree::new();
    let mut loaded = Settings::load_from(&file);
    loaded.screen_reader = ScreenReaderSetting::On;
    b.app
        .adopt_audio_and_settings(Some(Controller::for_test(saved.request())), loaded);
    b.frame(vec![]);
    open_audio(&mut b);
    settle(&mut b);
    assert_eq!(value(&b).as_deref(), Some("Speakers"));
    assert_eq!(
        Settings::load_from(&file).audio.device.as_deref(),
        Some("a")
    );
}

/// A fallback while a list is open changes the chosen value but the list is read again only
/// when it closes (task-245).
#[test]
fn a_fallback_waits_for_the_open_list() {
    use orb_player::control::{Controller, Event};
    let mut a = app(true);
    a.app.settings.audio = orb_ui::settings::AudioChoice {
        device: Some("a".into()),
        device_name: Some("Speakers".into()),
        ..Default::default()
    };
    open_audio(&mut a);
    settle(&mut a);
    // the focus is on the Driver combo box: Enter opens its list
    a.key(Key::Enter);
    settle(&mut a);
    let req = a.app.settings.audio.request().on_system_default();
    a.app.audio = Some(Controller::for_test(req));
    a.app.audio_events(vec![Event::Note("x".into())]);
    settle(&mut a);
    assert!(
        a.app.audio_refresh_pending_for_test(),
        "the list is not replaced"
    );
    a.key(Key::Escape);
    settle(&mut a);
    assert!(a.app.audio_is_open(), "Escape closed only the list");
    assert!(
        !a.app.audio_refresh_pending_for_test(),
        "read once the list closed"
    );
}

/// With the mode off the window is egui's: a Window, no Dialog, no named combo boxes, and Escape
/// closes it.
#[test]
fn with_the_mode_off_audio_output_is_as_before() {
    let mut a = app(false);
    open_audio(&mut a);
    settle(&mut a);
    assert!(dialogs(&a, Role::Dialog).is_empty());
    assert!(
        a.tree()
            .by_role(Role::Window)
            .any(|(_, n)| a11y::name(n) == Some("Audio Output"))
    );
    assert!(a.tree().by_name("Driver").next().is_none());
    a.key(Key::Escape);
    settle(&mut a);
    assert!(!a.app.audio_is_open());
}

/// Escape in screen reader mode closes an open list first; the window stays. The second Escape
/// closes the window and the focus returns.
#[test]
fn audio_output_escape_closes_the_list_then_the_window() {
    let mut a = app(true);
    let play = focus_play(&mut a);
    open_audio(&mut a);
    settle(&mut a);
    let driver = a
        .tree()
        .by_name("Driver")
        .next()
        .map(|(i, _)| i)
        .expect("Driver");
    assert_eq!(focused(&a), driver);
    a.key(Key::Enter);
    settle(&mut a);
    assert!(
        a.tree().by_role(Role::ListBox).next().is_some(),
        "the list is open"
    );
    a.key(Key::Escape);
    settle(&mut a);
    assert!(
        a.tree().by_role(Role::ListBox).next().is_none(),
        "the list closed"
    );
    assert!(a.app.audio_is_open(), "the window stays");
    a.key(Key::Escape);
    settle(&mut a);
    assert!(!a.app.audio_is_open());
    assert_eq!(focused(&a), play.accesskit_id(), "the focus is back");
}
