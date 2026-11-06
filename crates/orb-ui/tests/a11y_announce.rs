//! Announcements and the read-out keys (task-189, A11Y-5, docs/ui_spec.md section 20.3): the
//! announcer's node and text for each event, its silence while a song plays, and F2, F3 and F4
//! on a known song.

#[path = "common/a11y.rs"]
mod a11y;

use std::path::PathBuf;
use std::sync::Arc;

use a11y::{AppTree, Frames};
use eframe::egui::accesskit::{Action, Live, Role};
use eframe::egui::{Event, Key, Modifiers, Vec2};
use orb_cli::mods::Kit;
use orb_engine::tables::Tables;
use orb_player::output::Health;
use orb_player::{Audio, CALL_FRAMES};
use orb_ui::a11y::{announce, drive};
use orb_ui::app::App;
use orb_ui::layout::{PANEL_H, PANEL_W};
use orb_ui::link::Link;
use orb_ui::model::Panel;
use orb_ui::screen_reader::ScreenReaderSetting;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn trakka() -> PathBuf {
    root().join("tests/songs/shipped/Trakka Rekka.rbs")
}

/// An app on a player (the new song, the built-in kit) with its audio callback, without the
/// menu bar, settled, in screen reader mode or not.
fn player(screen_reader: bool) -> (AppTree, Audio) {
    let samples = Some(rbm::standard::builtin_samples());
    let (link, audio) = Link::new(Arc::new(Tables::new()), samples.clone());
    let mut app = App::with_link(Panel::new(), link, None);
    app.samples = samples;
    app.audio_health = Some(Health::started());
    app.settings.screen_reader = if screen_reader {
        ScreenReaderSetting::On
    } else {
        ScreenReaderSetting::Off
    };
    app.new_now();
    let mut h = AppTree {
        frames: Frames::new(Vec2::new(PANEL_W, PANEL_H)),
        app,
    };
    for _ in 0..4 {
        h.frame(vec![]);
    }
    (h, audio)
}

/// Runs `n` engine calls of the audio callback.
fn calls(audio: &mut Audio, n: usize) {
    let mut out = vec![0i16; 2 * CALL_FRAMES];
    for _ in 0..n {
        audio.render_i16(&mut out);
    }
}

fn said(h: &AppTree) -> Option<String> {
    h.app.announcer.text().map(str::to_string)
}

fn key(key: Key, pressed: bool, modifiers: Modifiers) -> Event {
    Event::Key {
        key,
        physical_key: Some(key),
        pressed,
        repeat: false,
        modifiers,
    }
}

/// Presses and releases `k` and returns what the announcer says then.
fn press(h: &mut AppTree, k: Key) -> Option<String> {
    h.key(k);
    said(h)
}

fn ctrl(h: &mut AppTree, k: Key) -> Option<String> {
    h.frame(vec![key(k, true, Modifiers::COMMAND)]);
    h.frame(vec![key(k, false, Modifiers::COMMAND)]);
    said(h)
}

/// The announcer's node in the last tree: its text, checked to be a polite live region that
/// takes no focus.
fn node_text(h: &AppTree) -> Option<String> {
    let n = h.tree().by_id(h.app.announcer.node_id())?;
    assert_eq!(n.role(), Role::Label);
    assert_eq!(n.live(), Some(Live::Polite));
    assert!(!n.supports_action(Action::Focus));
    assert_eq!(n.label(), n.value());
    n.value().map(str::to_string)
}

#[test]
fn a_status_message_is_announced_and_a_repeat_is_a_new_node() {
    let (mut h, _audio) = player(true);
    let dir = std::env::temp_dir().join(format!("orb-a11y-announce-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("saved.rbs");
    h.app.save_to(&path);
    h.frame(vec![]);
    let text = format!("Saved {}", path.display());
    assert_eq!(said(&h).as_deref(), Some(text.as_str()));
    assert_eq!(node_text(&h).as_deref(), Some(text.as_str()));
    let first = h.app.announcer.node_id();
    // the status line has its own node, with the text and no live setting
    let status = h
        .tree()
        .by_id(announce::status_id())
        .expect("the status line's node");
    assert_eq!(status.role(), Role::Status);
    assert_eq!(status.label(), Some(text.as_str()));
    assert_eq!(status.live(), None);

    // the same message again: a new node with the same text, the old one gone
    h.app.save_to(&path);
    h.frame(vec![]);
    assert_eq!(node_text(&h).as_deref(), Some(text.as_str()));
    assert_ne!(h.app.announcer.node_id(), first);
    assert!(h.tree().by_id(first).is_none());
    assert_eq!(
        h.tree()
            .by_role(Role::Label)
            .filter(|(_, n)| n.live().is_some())
            .count(),
        1,
        "one announcer node"
    );
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn a_song_opened_a_new_song_and_a_mod_chosen_are_announced() {
    let (mut h, _audio) = player(true);
    h.app.open_now(&trakka());
    h.frame(vec![]);
    let opened = format!("Opened Trakka Rekka.rbs, mod {}", h.app.doc.mod_name);
    assert_eq!(said(&h).as_deref(), Some(opened.as_str()));
    assert_eq!(node_text(&h).as_deref(), Some(opened.as_str()));

    h.app.new_now();
    assert_eq!(said(&h).as_deref(), Some("New song, mod Standard ReBirth"));

    h.app.use_kit(Kit::Mod(root().join("assets/Standard.rbm")));
    h.frame(vec![]);
    assert_eq!(said(&h).as_deref(), Some("Mod Standard ReBirth"));
}

#[test]
fn the_keyboard_mode_is_announced() {
    let (mut h, _audio) = player(true);
    assert_eq!(
        ctrl(&mut h, Key::G).as_deref(),
        Some("Select patterns from keyboard")
    );
    assert_eq!(
        ctrl(&mut h, Key::F).as_deref(),
        Some("Program synth from keyboard")
    );
    // again: said again
    let n = h.app.announcer.serial();
    ctrl(&mut h, Key::F);
    assert_eq!(h.app.announcer.serial(), n + 1);
}

#[test]
fn what_an_original_key_did_is_announced() {
    let (mut h, _audio) = player(true);
    ctrl(&mut h, Key::G);
    assert_eq!(
        press(&mut h, Key::Num2).as_deref(),
        Some("Bass Line 1 pattern A2")
    );
    assert_eq!(press(&mut h, Key::X).as_deref(), Some("TR-909 pattern A2"));
    assert_eq!(press(&mut h, Key::Plus).as_deref(), Some("Tempo 141 BPM"));
    assert_eq!(press(&mut h, Key::Minus).as_deref(), Some("Tempo 140 BPM"));
    // Record and the bar work in song mode
    h.app.panel.set_switch("transport.mode", 1);
    assert_eq!(press(&mut h, Key::Insert).as_deref(), Some("Recording on"));
    assert_eq!(press(&mut h, Key::Slash).as_deref(), Some("Loop on"));
    assert_eq!(press(&mut h, Key::Enter).as_deref(), Some("Playing"));
    assert_eq!(
        press(&mut h, Key::Num0).as_deref(),
        Some("Stopped, recording off")
    );
    // a key that changes nothing announced says nothing
    let n = h.app.announcer.serial();
    ctrl(&mut h, Key::F);
    let n = n + 1;
    h.key(Key::C); // the 303's key C (task-190 speaks the step editor)
    assert_eq!(h.app.announcer.serial(), n);
}

#[test]
fn the_focused_control_is_left_to_the_screen_reader() {
    let (mut h, _audio) = player(true);
    ctrl(&mut h, Key::G);
    // the focus on the tempo spin button: its own value change is spoken, no announcement
    let tempo = a11y::hit_id("display", "transport.tempo");
    h.send(tempo, Action::Focus, None);
    assert_eq!(h.tree().focused_id(), tempo.accesskit_id());
    let n = h.app.announcer.serial();
    h.key(Key::Plus);
    assert_eq!(h.app.panel.transport.tempo_display(), 141);
    assert_eq!(h.app.announcer.serial(), n);
    // but a pattern key still says what it did
    assert_eq!(
        press(&mut h, Key::Num3).as_deref(),
        Some("Bass Line 1 pattern A3")
    );
    // the focus on row 1's pattern selector: its pattern key is silent
    h.send(drive::selector_id(0), Action::Focus, None);
    let n = h.app.announcer.serial();
    h.key(Key::Num4);
    assert_eq!(h.app.panel.rows[0].pattern, 3);
    assert_eq!(h.app.announcer.serial(), n);
}

#[test]
fn space_on_play_and_stop_and_the_function_keys_are_announced() {
    let (mut h, _audio) = player(true);
    let play = a11y::hit_id("trigger", "transport.play");
    h.send(play, Action::Focus, None);
    assert_eq!(press(&mut h, Key::Space).as_deref(), Some("Playing"));
    assert!(h.app.panel.transport.playing);
    let stop = a11y::hit_id("trigger", "transport.stop");
    h.send(stop, Action::Focus, None);
    assert_eq!(press(&mut h, Key::Space).as_deref(), Some("Stopped"));
    // Space on a focused toggle: its own state is spoken by the reader, no announcement
    let loop_ = a11y::hit_id("toggle", "transport.loop");
    h.send(loop_, Action::Focus, None);
    let n = h.app.announcer.serial();
    h.key(Key::Space);
    assert!(h.app.panel.transport.looping);
    assert_eq!(h.app.announcer.serial(), n);
    // the transport keys of the mode (task-188), the focus on the loop toggle
    assert_eq!(press(&mut h, Key::F5).as_deref(), Some("Playing"));
    assert_eq!(press(&mut h, Key::F12).as_deref(), Some("Song mode"));
    assert_eq!(press(&mut h, Key::F9).as_deref(), Some("Recording on"));
    let n = h.app.announcer.serial();
    h.key(Key::F11); // the focused Loop toggle speaks for itself
    assert!(!h.app.panel.transport.looping);
    assert_eq!(h.app.announcer.serial(), n);
    assert_eq!(
        press(&mut h, Key::F5).as_deref(),
        Some("Stopped, recording off")
    );
}

#[test]
fn the_read_out_keys_work_with_a_window_open() {
    let (mut h, _audio) = player(true);
    h.app.show_key_help();
    for _ in 0..3 {
        h.frame(vec![]);
    }
    assert_eq!(
        press(&mut h, Key::F3).as_deref(),
        Some("Stopped, 140 BPM, pattern mode")
    );
    // F2 reads the row focus's pattern (a 303's: its steps follow, task-190)
    let f2 = press(&mut h, Key::F2).unwrap();
    assert!(
        f2.starts_with("Bass Line 1, pattern A1, 16 steps, shuffle off. 1 rest, 2 rest"),
        "{f2}"
    );
}

#[test]
fn a_held_tempo_key_announces_where_it_ended() {
    let (mut h, _audio) = player(true);
    h.frame(vec![key(Key::Plus, true, Modifiers::NONE)]);
    assert_eq!(said(&h).as_deref(), Some("Tempo 141 BPM"));
    // held about 1.1 s: the repeat schedule steps on
    for _ in 0..70 {
        h.frame(vec![]);
    }
    let n = h.app.announcer.serial();
    h.frame(vec![key(Key::Plus, false, Modifiers::NONE)]);
    let bpm = h.app.panel.transport.tempo_display();
    assert!(bpm > 142, "{bpm}");
    assert_eq!(h.app.announcer.serial(), n + 1);
    assert_eq!(said(&h), Some(format!("Tempo {bpm} BPM")));
}

#[test]
fn with_the_mode_off_nothing_is_announced() {
    let (mut h, _audio) = player(false);
    ctrl(&mut h, Key::G);
    for k in [
        Key::Num2,
        Key::Plus,
        Key::Enter,
        Key::Num0,
        Key::F2,
        Key::F3,
        Key::F4,
    ] {
        h.key(k);
    }
    h.app
        .save_to(&std::env::temp_dir().join("orb-a11y-announce-off.rbs"));
    h.frame(vec![]);
    assert_eq!(h.app.announcer.serial(), 0);
    assert!(h.tree().by_id(h.app.announcer.node_id()).is_none());
    // the status line's node is there with the mode off too
    assert!(h.tree().by_id(announce::status_id()).is_some());
    std::fs::remove_file(std::env::temp_dir().join("orb-a11y-announce-off.rbs")).ok();
}

/// Trakka Rekka opened in screen reader mode.
fn trakka_open() -> (AppTree, Audio) {
    let (mut h, mut audio) = player(true);
    h.app.open_now(&trakka());
    calls(&mut audio, 2);
    h.frame(vec![]);
    (h, audio)
}

#[test]
fn nothing_is_announced_by_itself_during_10_seconds_of_playback() {
    let (mut h, mut audio) = trakka_open();
    // Enter is Play outside Program Synth mode with a 303 focused
    ctrl(&mut h, Key::G);
    h.key(Key::Enter);
    assert_eq!(said(&h).as_deref(), Some("Playing"));
    let serial = h.app.announcer.serial();
    let node = h.app.announcer.node_id();
    // a frame each 33 ms of audio, for 10 s of engine time, with no input
    let (mut positions, mut steps, mut lit, mut grs) = (vec![], vec![], 0, vec![]);
    let total = 10.0 * 44100.0 / CALL_FRAMES as f64;
    let mut sent = 0.0;
    let mut f = 0;
    while sent < total {
        f += 1;
        let due = (f as f64 * 0.033 * 44100.0 / CALL_FRAMES as f64).min(total.ceil());
        while sent < due {
            calls(&mut audio, 1);
            sent += 1.0;
        }
        h.frame(vec![]);
        positions.push(h.app.panel.transport.position);
        steps.push(h.app.panel.playing_step909);
        lit += h.app.panel.meters.values().any(|&m| m > 0) as usize;
        grs.push(h.app.panel.gr_leds);
        assert_eq!(h.app.announcer.serial(), serial, "frame {f}");
        assert_eq!(said(&h).as_deref(), Some("Playing"));
    }
    assert!(h.app.panel.transport.playing);
    assert!(f >= 300, "{f} frames");
    // the position, the running light and the meters moved
    let bars = h.app.panel.transport.bar_display();
    assert!(positions.last() > positions.first(), "{positions:?}");
    assert!(bars >= 5, "bar {bars}");
    let changes = steps.windows(2).filter(|w| w[0] != w[1]).count();
    assert!(changes >= 60, "{changes} running light changes");
    assert!(lit > f / 2, "meters lit in {lit} of {f} frames");
    let gr_changes = grs.windows(2).filter(|w| w[0] != w[1]).count();
    assert!(gr_changes > 0, "the gain-reduction LEDs moved");
    // the node is the one of "Playing" still
    assert_eq!(h.app.announcer.node_id(), node);
    assert_eq!(node_text(&h).as_deref(), Some("Playing"));
}

#[test]
fn the_read_out_keys_on_a_known_song() {
    let (mut h, mut audio) = trakka_open();
    assert_eq!(h.app.panel.focus, 0, "the row focus");
    let f2 = press(&mut h, Key::F2).unwrap();
    let f3 = press(&mut h, Key::F3).unwrap();
    let f4 = press(&mut h, Key::F4).unwrap();
    // the song's Bass Line 1 section is switched off; its loop is off; the compressor's LEDs
    // hold the value of the load's engine calls
    // (a 303's steps follow since task-190)
    assert_eq!(
        f2,
        "Bass Line 1, pattern A6, 16 steps, shuffle off, pattern section off. \
         1 A sharp octave down, 2 A sharp octave down, 3 C octave down octave up, 4 C, \
         5 C octave down, 6 C, 7 D sharp, 8 D sharp, 9 C accent, 10 C accent, \
         11 C octave down slide, 12 C octave down, 13 C octave up slide, 14 C octave up slide, \
         15 C slide, 16 C"
    );
    assert_eq!(
        f3,
        "Stopped, bar 1, 139 BPM, song mode, loop off, recording off"
    );
    assert_eq!(
        f4,
        "Master left 0 of 10, right 0 of 10, gain reduction 1 of 4"
    );
    assert_eq!(node_text(&h).as_deref(), Some(f4.as_str()));

    // F2 reads the row of the section the focus is in: the 909's level fader
    let fader = a11y::hit_id("fader", "mixer4.level");
    h.send(fader, Action::Focus, None);
    assert_eq!(h.tree().focused_id(), fader.accesskit_id());
    let f2 = press(&mut h, Key::F2).unwrap();
    // with the drums' steps (task-191): the song's 909 pattern B5
    assert!(
        f2.starts_with("TR-909, pattern B5, 16 steps, shuffle off. Accent: 1, 3, 5, "),
        "{f2}"
    );
    assert!(f2.contains(". Bass drum: 1 accent, 9 accent. "), "{f2}");
    assert!(f2.ends_with(". Closed hi-hat: 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16"));

    // playing: the transport and the levels follow
    ctrl(&mut h, Key::G);
    h.key(Key::Enter);
    for _ in 0..60 {
        calls(&mut audio, 2);
        h.frame(vec![]);
    }
    let f3 = press(&mut h, Key::F3).unwrap();
    let f4 = press(&mut h, Key::F4).unwrap();
    assert!(f3.starts_with("Playing, bar "), "{f3}");
    assert_eq!(f4, announce::levels_readout(&h.app.panel));
    assert!(!f4.starts_with("Master left 0 "), "{f4}");
}
