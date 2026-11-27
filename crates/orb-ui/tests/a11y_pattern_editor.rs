//! The 303 pattern editor windows in screen reader mode (task-235, decision task-242 A;
//! docs/ui_spec.md section 20.8), headless through the AccessKit tree: the window is a named
//! Dialog that takes the focus as it opens and gives it back as it closes; its controls are
//! named, have their values and work from the keyboard; the Step editor has the panel step
//! editor's keys and speech; every edit goes through the window's one path (heard, the song
//! modified, one Undo); playback says nothing; with the mode off nothing of this is there.

#[path = "common/a11y.rs"]
mod a11y;

use std::sync::{Arc, OnceLock};

use a11y::{AppTree, Frames};
use eframe::egui::accesskit::{Action, Role, Toggled};
use eframe::egui::{Event, Id, Key, Modifiers, Vec2};
use orb_engine::tables::Tables;
use orb_player::{Audio, CALL_FRAMES};
use orb_ui::a11y::editor303::{self, Part};
use orb_ui::a11y::step303;
use orb_ui::app::App;
use orb_ui::layout::{PANEL_H, PANEL_W};
use orb_ui::link::Link;
use orb_ui::model::Panel;
use orb_ui::pattern_editor::{self, Flag};
use orb_ui::screen_reader::ScreenReaderSetting;

fn tables() -> Arc<Tables> {
    static T: OnceLock<Arc<Tables>> = OnceLock::new();
    T.get_or_init(|| Arc::new(Tables::new())).clone()
}

/// The app with a new song (an engine behind it), screen reader mode on or off, no menu bar.
struct H {
    t: AppTree,
    audio: Audio,
}

impl H {
    fn new(screen_reader: bool) -> Self {
        let samples = rbm::standard::builtin_samples();
        let (link, audio) = Link::new(tables(), Some(samples.clone()));
        let mut app = App::with_link(Panel::new(), link, None);
        app.samples = Some(samples);
        app.menu = false;
        app.settings.screen_reader = if screen_reader {
            ScreenReaderSetting::On
        } else {
            ScreenReaderSetting::Off
        };
        let mut h = Self {
            t: AppTree {
                frames: Frames::new(Vec2::new(PANEL_W, PANEL_H)),
                app,
            },
            audio,
        };
        h.calls(1);
        for _ in 0..4 {
            h.t.frame(vec![]);
        }
        h
    }

    /// `n` engine calls, then a frame.
    fn calls(&mut self, n: usize) {
        let mut out = vec![0i16; 2 * CALL_FRAMES];
        for _ in 0..n {
            self.audio.render_i16(&mut out);
        }
        self.t.frame(vec![]);
    }

    fn frames(&mut self, n: usize) {
        for _ in 0..n {
            self.t.frame(vec![]);
        }
    }

    fn key_with(&mut self, key: Key, modifiers: Modifiers) {
        let e = |pressed| Event::Key {
            key,
            physical_key: Some(key),
            pressed,
            repeat: false,
            modifiers,
        };
        self.t.frame(vec![e(true)]);
        self.t.frame(vec![e(false)]);
    }

    /// Presses `k` and returns what the announcer said, if it spoke.
    fn press(&mut self, k: Key) -> Option<String> {
        let n = self.t.app.announcer.serial();
        self.t.key(k);
        (self.t.app.announcer.serial() != n).then(|| self.said())
    }

    fn said(&self) -> String {
        self.t.app.announcer.text().unwrap_or_default().to_string()
    }

    fn focus(&mut self, id: Id) {
        self.t.send(id, Action::Focus, None);
        assert_eq!(self.t.tree().focused_id(), id.accesskit_id());
    }

    fn focused(&self) -> eframe::egui::accesskit::NodeId {
        self.t.tree().focused_id()
    }

    /// The value of control `part` of the window of 303 `k` in the last tree.
    fn value(&self, k: usize, part: Part) -> String {
        let n = self
            .t
            .tree()
            .by_id(editor303::id(k, part))
            .unwrap_or_else(|| panic!("{part:?}"));
        a11y::value(n).unwrap_or_default().to_string()
    }

    /// The pattern of the editor of 303 `k` as the link holds it.
    fn bytes(&self, k: usize) -> Vec<u8> {
        let p = self.t.app.pattern_editor(k).pattern;
        self.t.app.link.as_ref().unwrap().pattern(k, p).to_vec()
    }

    /// Opens the editor of 303 `k` as the Edit menu does, and lets the focus go in.
    fn open(&mut self, k: usize) {
        self.t.app.open_pattern_editor(k);
        self.frames(4);
    }
}

#[test]
fn the_window_is_a_named_dialog_with_the_focus_in_and_back() {
    let mut h = H::new(true);
    let back = step303::editor_id(0);
    h.focus(back);
    // Ctrl+E with the 303 focused opens its editor; the focus goes to its first control
    h.key_with(Key::E, Modifiers::COMMAND);
    h.frames(3);
    assert!(h.t.app.pattern_editor(0).open);
    assert_eq!(
        h.focused(),
        editor303::id(0, Part::Pattern).accesskit_id(),
        "the focus went into the window"
    );
    let t = h.t.tree();
    let dialogs: Vec<_> = t
        .by_role(Role::Dialog)
        .filter_map(|(_, n)| a11y::name(n))
        .collect();
    assert!(dialogs.contains(&"303-1 Pattern Editor"), "{dialogs:?}");
    // its controls are inside the dialog
    let (dialog, _) = t
        .by_role(Role::Dialog)
        .find(|(_, n)| a11y::name(n) == Some("303-1 Pattern Editor"))
        .unwrap();
    let mut up = t.parent(editor303::id(0, Part::Steps).accesskit_id());
    let mut inside = false;
    while let Some((id, _)) = up {
        if id == dialog {
            inside = true;
            break;
        }
        up = t.parent(id);
    }
    assert!(inside, "the Step editor is in the dialog");

    // Tab walks the controls in their order
    let mut order = vec![h.focused()];
    for _ in 1..Part::ALL.len() {
        h.t.key(Key::Tab);
        order.push(h.focused());
    }
    let want: Vec<_> = Part::ALL
        .iter()
        .map(|&p| editor303::id(0, p).accesskit_id())
        .collect();
    assert_eq!(order, want);

    // Escape closes it and the focus goes back to the panel's step editor
    h.t.key(Key::Escape);
    h.frames(3);
    assert!(!h.t.app.pattern_editor(0).open);
    assert_eq!(h.focused(), back.accesskit_id(), "the focus came back");

    // the Edit menu's item on an open window brings the focus in again
    h.open(1);
    assert_eq!(h.focused(), editor303::id(1, Part::Pattern).accesskit_id());
    h.focus(back);
    h.open(1);
    assert_eq!(
        h.focused(),
        editor303::id(1, Part::Pattern).accesskit_id(),
        "reopened: the focus is in again"
    );
}

#[test]
fn every_control_is_named_with_its_value() {
    let mut h = H::new(true);
    h.open(0);
    let t = h.t.tree();
    let role = |p: Part| a11y::role(t.by_id(editor303::id(0, p)).unwrap());
    let name = |p: Part| a11y::name(t.by_id(editor303::id(0, p)).unwrap()).map(str::to_string);
    for p in Part::ALL {
        assert_eq!(name(p).as_deref(), Some(p.name()), "{p:?}");
        assert!(a11y::supports(
            t.by_id(editor303::id(0, p)).unwrap(),
            Action::Focus
        ));
    }
    assert_eq!(role(Part::Pattern), Role::SpinButton);
    assert_eq!(role(Part::Steps), Role::Slider);
    assert_eq!(role(Part::Length), Role::SpinButton);
    assert_eq!(role(Part::Shuffle), Role::CheckBox);
    for p in [
        Part::TransposeUp,
        Part::TransposeDown,
        Part::ShiftLeft,
        Part::ShiftRight,
        Part::Copy,
        Part::Paste,
        Part::Clear,
    ] {
        assert_eq!(role(p), Role::Button, "{p:?}");
    }
    assert_eq!(h.value(0, Part::Pattern), "A1");
    assert_eq!(h.value(0, Part::Steps), "Step 1 of 16: rest, C");
    assert_eq!(h.value(0, Part::Length), "16 steps");
    let shuffle = t.by_id(editor303::id(0, Part::Shuffle)).unwrap();
    assert_eq!(a11y::toggled(shuffle), Some(Toggled::False));
    assert_eq!(step303::EDITOR_NAME, Part::Steps.name());
}

/// The Step editor: the panel step editor's keys, its words, the edits through the window's
/// path (one Undo each, the song modified), F2.
#[test]
fn the_step_editor_programs_the_pattern_with_the_panel_s_keys() {
    let mut h = H::new(true);
    h.open(0);
    h.focus(editor303::id(0, Part::Steps));
    let empty = h.bytes(0);
    assert!(!h.t.app.doc.modified);

    // a pitch key on a rest sets the note only; the same note again is said by the announcer
    assert_eq!(h.press(Key::C).as_deref(), Some("Step 1: rest, C"));
    // the others by the value: - makes it a note, P an accent
    assert_eq!(h.press(Key::Minus), None);
    assert_eq!(h.value(0, Part::Steps), "Step 1 of 16: C");
    assert_eq!(h.press(Key::P), None);
    assert_eq!(h.value(0, Part::Steps), "Step 1 of 16: C, accent");
    assert!(h.t.app.doc.modified, "an edit marks the song modified");
    assert_eq!(h.t.app.undo_name(), Some("Edit Pattern"));
    let b = h.bytes(0);
    assert_eq!(pattern_editor::note(&b, 0), Some(0));
    assert!(pattern_editor::flag(&b, 0, Flag::Accent));

    // Down to step 2: V is D, still a rest; [ slide, ' octave up, ; octave down
    assert_eq!(h.press(Key::ArrowDown), None);
    assert_eq!(h.value(0, Part::Steps), "Step 2 of 16: rest, C");
    h.press(Key::V);
    assert_eq!(h.value(0, Part::Steps), "Step 2 of 16: rest, D");
    h.press(Key::OpenBracket);
    h.press(Key::Quote);
    h.press(Key::Semicolon);
    assert_eq!(
        h.value(0, Part::Steps),
        "Step 2 of 16: rest, D, octave down, octave up, slide"
    );
    // Options > Pitch Keys Make Notes: a pitch key on a rest makes it a note
    h.t.app.settings.pitch_keys_make_notes = true;
    h.press(Key::G);
    assert_eq!(
        h.value(0, Part::Steps),
        "Step 2 of 16: D sharp, octave down, octave up, slide"
    );
    // the one-level Undo takes the last edit back
    h.key_with(Key::Z, Modifiers::COMMAND);
    h.frames(1);
    assert_eq!(
        h.value(0, Part::Steps),
        "Step 2 of 16: rest, D, octave down, octave up, slide"
    );

    // Left and Right do nothing; Up, Home and End move; Enter and Backspace as Step and Back
    h.press(Key::ArrowLeft);
    h.press(Key::ArrowRight);
    assert!(h.value(0, Part::Steps).starts_with("Step 2 of 16"));
    h.press(Key::ArrowUp);
    assert!(h.value(0, Part::Steps).starts_with("Step 1 of 16"));
    h.press(Key::ArrowUp);
    assert!(
        h.value(0, Part::Steps).starts_with("Step 16 of 16"),
        "Up from step 1 wraps to 16"
    );
    h.press(Key::Home);
    assert!(h.value(0, Part::Steps).starts_with("Step 1 of 16"));
    h.press(Key::Enter);
    assert!(h.value(0, Part::Steps).starts_with("Step 2 of 16"));
    h.press(Key::Backspace);
    assert!(h.value(0, Part::Steps).starts_with("Step 1 of 16"));
    h.press(Key::End);
    assert!(h.value(0, Part::Steps).starts_with("Step 16 of 16"));

    // F2 reads the whole pattern
    assert_eq!(
        h.press(Key::F2).as_deref(),
        Some(editor303::readout(0, 0, &h.bytes(0)).as_str())
    );
    assert!(
        h.said()
            .starts_with("303-1, pattern A1, 16 steps, shuffle off. 1 C accent, 2 rest")
    );
    // the panel's 303 has the edits too (one pattern, the panel's step views)
    assert!(h.t.app.panel.synth[0].patterns[0][0].accent);
    assert_ne!(h.bytes(0), empty);
}

#[test]
fn the_header_controls_work_from_the_keyboard_and_say_what_they_did() {
    let mut h = H::new(true);
    h.open(0);
    // Pattern: Up is the next pattern, selected on the panel too; Page Up a bank
    h.focus(editor303::id(0, Part::Pattern));
    h.press(Key::ArrowUp);
    h.frames(1);
    assert_eq!(h.t.app.pattern_editor(0).pattern, 1);
    assert_eq!(h.value(0, Part::Pattern), "A2");
    assert_eq!(h.t.app.panel.rows[0].pattern, 1);
    h.press(Key::PageUp);
    h.frames(1);
    assert_eq!(h.value(0, Part::Pattern), "B2");
    h.press(Key::Home);
    h.frames(1);
    assert_eq!(h.value(0, Part::Pattern), "A1");
    // Length: Down one, Home 1, End 16; an Undo each
    h.focus(editor303::id(0, Part::Length));
    h.press(Key::ArrowDown);
    assert_eq!(h.value(0, Part::Length), "15 steps");
    assert_eq!(pattern_editor::length(&h.bytes(0)), 15);
    assert_eq!(h.t.app.undo_name(), Some("Pattern Length"));
    h.press(Key::Home);
    assert_eq!(h.value(0, Part::Length), "1 step");
    h.press(Key::End);
    assert_eq!(h.value(0, Part::Length), "16 steps");
    // Shuffle: Space toggles it
    h.focus(editor303::id(0, Part::Shuffle));
    h.press(Key::Space);
    let n = h.t.tree().by_id(editor303::id(0, Part::Shuffle)).unwrap();
    assert_eq!(a11y::toggled(n), Some(Toggled::True));
    assert_eq!(h.bytes(0)[0], 1);
    // a Shift that changes nothing (every step alike) says so, and writes nothing
    let undo = h.t.app.undo_name();
    h.t.send(editor303::id(0, Part::ShiftRight), Action::Click, None);
    assert_eq!(h.said(), "Pattern A1 unchanged");
    assert_eq!(h.t.app.undo_name(), undo, "no write, the Undo stays");
    // Transpose and Shift say what they did
    h.focus(editor303::id(0, Part::Steps));
    h.press(Key::Minus);
    h.focus(editor303::id(0, Part::TransposeUp));
    assert_eq!(
        h.press(Key::Space).as_deref(),
        Some("Pattern A1 transposed up")
    );
    assert_eq!(pattern_editor::note(&h.bytes(0), 0), Some(1));
    assert_eq!(h.t.app.undo_name(), Some("Transpose Pattern"));
    h.t.send(editor303::id(0, Part::ShiftRight), Action::Click, None);
    assert_eq!(h.said(), "Pattern A1 shifted right");
    assert_eq!(pattern_editor::note(&h.bytes(0), 1), Some(1));
    h.t.send(editor303::id(0, Part::TransposeDown), Action::Click, None);
    assert_eq!(h.said(), "Pattern A1 transposed down");
    h.t.send(editor303::id(0, Part::ShiftLeft), Action::Click, None);
    assert_eq!(h.said(), "Pattern A1 shifted left");
    // Copy and Clear speak through the status line
    h.t.send(editor303::id(0, Part::Copy), Action::Click, None);
    assert_eq!(h.said(), "303-1: A1 copied");
    h.t.send(editor303::id(0, Part::Clear), Action::Click, None);
    assert_eq!(h.said(), "303-1: A1 cleared");
    assert_eq!(pattern_editor::note(&h.bytes(0), 0), None);
}

/// Move-along doesn't speak: while the song plays, and when playback moves the 303's row to
/// another pattern, the focused window's values do not change and nothing is said.
#[test]
fn playback_says_nothing_and_does_not_move_the_window() {
    let mut h = H::new(true);
    h.open(0);
    h.focus(editor303::id(0, Part::Steps));
    h.press(Key::ArrowDown);
    h.press(Key::F5);
    assert!(h.t.app.panel.transport.playing);
    let n = h.t.app.announcer.serial();
    let (steps, pattern) = (h.value(0, Part::Steps), h.value(0, Part::Pattern));
    for _ in 0..60 {
        h.calls(4);
    }
    // the engine's playback moving the row to another pattern (`Link::follow_values`)
    h.t.app.panel.adopt_pattern(0, 5);
    h.frames(5);
    assert_eq!(h.t.app.announcer.serial(), n, "nothing was said");
    assert_eq!(h.value(0, Part::Steps), steps);
    assert_eq!(h.value(0, Part::Pattern), pattern, "the window stays on A1");
    assert_eq!(h.t.app.pattern_editor(0).pattern, 0);
}

#[test]
fn with_the_mode_off_the_window_is_upstream_s() {
    let mut h = H::new(false);
    h.open(0);
    assert!(h.t.app.pattern_editor(0).open);
    let t = h.t.tree();
    for p in Part::ALL {
        assert!(t.by_id(editor303::id(0, p)).is_none(), "{p:?}");
    }
    assert!(
        t.by_role(Role::Dialog).next().is_none(),
        "egui's own Window node"
    );
    // the painted grid is there, and the header's egui controls
    assert!(h.t.app.pattern_editor_note_cell(0, 0, 0).is_some());
    assert!(h.t.app.pattern_editor_hit(0, "Copy").is_some());
    assert!(h.t.app.pattern_editor_hit(0, "number").is_some());
}
