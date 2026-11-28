//! The 808 and 909 pattern editor windows in screen reader mode (task-236, decision task-243 A;
//! docs/ui_spec.md section 20.9), headless through the AccessKit tree: the window is a named
//! Dialog that takes the focus as it opens and gives it back as it closes; its controls are
//! named, have their values and work from the keyboard; the grid has the panel grid's keys and
//! words; every edit goes through the window's one path (heard, the song modified, one Undo);
//! playback says nothing; with the mode off nothing of this is there.

#[path = "common/a11y.rs"]
mod a11y;

use std::sync::{Arc, OnceLock};

use a11y::{AppTree, Frames};
use eframe::egui::accesskit::{Action, Role, Toggled};
use eframe::egui::{Event, Id, Key, Modifiers, Vec2};
use orb_engine::tables::Tables;
use orb_player::{Audio, CALL_FRAMES};
use orb_ui::a11y::editor_drum::{self, Part};
use orb_ui::a11y::grid;
use orb_ui::app::App;
use orb_ui::drum_editor::{self, Machine};
use orb_ui::layout::{PANEL_H, PANEL_W};
use orb_ui::link::Link;
use orb_ui::model::Panel;
use orb_ui::screen_reader::ScreenReaderSetting;

const M8: Machine = Machine::Tr808;
const M9: Machine = Machine::Tr909;

/// Ctrl as Windows reports it (Ctrl is also the command key there).
const CTRL: Modifiers = Modifiers {
    alt: false,
    ctrl: true,
    shift: false,
    mac_cmd: false,
    command: true,
};

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
        h.frames(4);
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

    /// Presses and releases `key` with `modifiers` held, in two frames.
    fn key_with(&mut self, key: Key, modifiers: Modifiers) {
        let e = |pressed| Event::Key {
            key,
            physical_key: Some(key),
            pressed,
            repeat: false,
            modifiers,
        };
        self.t
            .frame(vec![Event::ModifiersChanged(modifiers), e(true)]);
        self.t
            .frame(vec![e(false), Event::ModifiersChanged(Modifiers::NONE)]);
    }

    /// Presses `k` and returns what the announcer said, if it spoke.
    fn press(&mut self, k: Key) -> Option<String> {
        self.press_with(k, Modifiers::NONE)
    }

    fn press_with(&mut self, k: Key, modifiers: Modifiers) -> Option<String> {
        let n = self.t.app.announcer.serial();
        self.key_with(k, modifiers);
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

    /// The value of control `part` of machine `m`'s window in the last tree.
    fn value(&self, m: Machine, part: Part) -> String {
        let n = self
            .t
            .tree()
            .by_id(editor_drum::id(m, part))
            .unwrap_or_else(|| panic!("{part:?}"));
        a11y::value(n).unwrap_or_default().to_string()
    }

    /// The name of the grid's active cell of machine `m`'s window.
    fn cell(&self, m: Machine) -> String {
        let t = self.t.tree();
        let g = t.by_id(editor_drum::id(m, Part::Steps)).expect("the grid");
        let c = t
            .node(g.active_descendant().expect("an active cell"))
            .unwrap();
        a11y::name(c).unwrap_or_default().to_string()
    }

    /// The pattern of machine `m`'s editor as the link holds it.
    fn bytes(&self, m: Machine) -> Vec<u8> {
        let p = self.t.app.drum_editor(m).pattern;
        self.t
            .app
            .link
            .as_ref()
            .unwrap()
            .pattern(m.row(), p)
            .to_vec()
    }

    /// Opens machine `m`'s editor as the Edit menu does, and lets the focus go in.
    fn open(&mut self, m: Machine) {
        self.t.app.open_drum_editor(m);
        self.frames(4);
    }
}

#[test]
fn the_window_is_a_named_dialog_with_the_focus_in_and_back() {
    let mut h = H::new(true);
    let back = grid::grid_id(false);
    h.focus(back);
    // Ctrl+E with the 909 focused opens its editor; the focus goes to its first control
    h.key_with(Key::E, Modifiers::COMMAND);
    h.frames(3);
    assert!(h.t.app.drum_editor(M9).open);
    assert!(!h.t.app.drum_editor(M8).open);
    assert_eq!(
        h.focused(),
        editor_drum::id(M9, Part::Pattern).accesskit_id(),
        "the focus went into the window"
    );
    let t = h.t.tree();
    let (dialog, _) = t
        .by_role(Role::Dialog)
        .find(|(_, n)| a11y::name(n) == Some("909 Pattern Editor"))
        .expect("the dialog");
    // its grid is inside the dialog
    let mut up = t.parent(editor_drum::id(M9, Part::Steps).accesskit_id());
    let mut inside = false;
    while let Some((id, _)) = up {
        if id == dialog {
            inside = true;
            break;
        }
        up = t.parent(id);
    }
    assert!(inside, "the grid is in the dialog");

    // Tab walks the controls in their order
    let mut order = vec![h.focused()];
    for _ in 1..Part::of(M9).len() {
        h.t.key(Key::Tab);
        order.push(h.focused());
    }
    let want: Vec<_> = Part::of(M9)
        .iter()
        .map(|&p| editor_drum::id(M9, p).accesskit_id())
        .collect();
    assert_eq!(order, want);

    // Escape closes it and the focus goes back to the panel's 909 grid
    h.t.key(Key::Escape);
    h.frames(3);
    assert!(!h.t.app.drum_editor(M9).open);
    assert_eq!(h.focused(), back.accesskit_id(), "the focus came back");

    // the 808's: the Edit menu's item, and again on an open window brings the focus in
    h.open(M8);
    assert_eq!(
        h.focused(),
        editor_drum::id(M8, Part::Pattern).accesskit_id()
    );
    h.focus(back);
    h.open(M8);
    assert_eq!(
        h.focused(),
        editor_drum::id(M8, Part::Pattern).accesskit_id(),
        "reopened: the focus is in again"
    );
    let dialogs: Vec<_> =
        h.t.tree()
            .by_role(Role::Dialog)
            .filter_map(|(_, n)| a11y::name(n))
            .collect();
    assert!(dialogs.contains(&"808 Pattern Editor"), "{dialogs:?}");
}

#[test]
fn every_control_is_named_with_its_value() {
    let mut h = H::new(true);
    h.open(M8);
    h.open(M9);
    let t = h.t.tree();
    for m in Machine::ALL {
        let node = |p: Part| t.by_id(editor_drum::id(m, p)).unwrap();
        for p in Part::of(m) {
            assert_eq!(a11y::name(node(p)), Some(p.name()), "{m:?} {p:?}");
            assert!(a11y::supports(node(p), Action::Focus), "{m:?} {p:?}");
        }
        assert_eq!(a11y::role(node(Part::Pattern)), Role::SpinButton);
        assert_eq!(a11y::role(node(Part::Steps)), Role::Grid);
        assert_eq!(a11y::role(node(Part::Length)), Role::SpinButton);
        assert_eq!(a11y::role(node(Part::Shuffle)), Role::CheckBox);
        for p in [
            Part::ShiftLeft,
            Part::ShiftRight,
            Part::Copy,
            Part::Paste,
            Part::Clear,
        ] {
            assert_eq!(a11y::role(node(p)), Role::Button, "{p:?}");
        }
        let g = node(Part::Steps);
        assert_eq!(g.row_count(), Some(12));
        assert_eq!(g.column_count(), Some(16));
        assert_eq!(a11y::toggled(node(Part::Shuffle)), Some(Toggled::False));
    }
    assert!(
        t.by_id(editor_drum::id(M8, Part::Entry)).is_none(),
        "the 808 has no entry"
    );
    let entry = t.by_id(editor_drum::id(M9, Part::Entry)).unwrap();
    assert_eq!(a11y::role(entry), Role::SpinButton);
    assert_eq!(h.value(M9, Part::Entry), "Cycle");
    assert_eq!(h.value(M8, Part::Pattern), "A1");
    assert_eq!(h.value(M9, Part::Length), "16 steps");
    // the grid starts on the first drum row, step 1, the 192 cells under it
    assert_eq!(h.cell(M9), "Bass drum, step 1, off");
    assert_eq!(h.cell(M8), "Bass drum, step 1, off");
    let gid = editor_drum::id(M9, Part::Steps).accesskit_id();
    let cells = t
        .by_role(Role::GridCell)
        .filter(|(id, _)| t.parent(*id).is_some_and(|(up, _)| up == gid))
        .count();
    assert_eq!(cells, 12 * 16);
}

/// The 909's grid: the panel grid's keys and words, the edits through the window's path (one
/// Undo each, the song modified), the Entry, the hi-hats, F2.
#[test]
fn the_909_grid_programs_the_pattern_with_the_panel_grid_s_keys() {
    let mut h = H::new(true);
    h.open(M9);
    let gid = editor_drum::id(M9, Part::Steps);
    h.focus(gid);
    assert!(!h.t.app.doc.modified);
    // Space: the click cycle, off, on, accent, off; the active cell says it (nothing announced)
    assert_eq!(h.press(Key::Space), None);
    assert_eq!(h.cell(M9), "Bass drum, step 1, on");
    assert!(h.t.app.doc.modified, "an edit marks the song modified");
    assert_eq!(h.t.app.undo_name(), Some("Edit Pattern"));
    assert_eq!(drum_editor::cell(&h.bytes(M9), 0, 1), 1);
    h.press(Key::Space);
    assert_eq!(h.cell(M9), "Bass drum, step 1, accent");
    h.press(Key::Space);
    assert_eq!(h.cell(M9), "Bass drum, step 1, off");
    // Ctrl+Space a note on / off, Shift+Space (and Alt+Space) an accent, Ctrl+Alt+Space a flam
    h.press_with(Key::Space, CTRL);
    assert_eq!(h.cell(M9), "Bass drum, step 1, on");
    h.press_with(Key::Space, Modifiers::SHIFT);
    assert_eq!(h.cell(M9), "Bass drum, step 1, accent");
    h.press_with(Key::Space, Modifiers::ALT);
    assert_eq!(h.cell(M9), "Bass drum, step 1, off");
    h.press_with(Key::Space, Modifiers { alt: true, ..CTRL });
    assert_eq!(h.cell(M9), "Bass drum, step 1, flam");
    assert_eq!(drum_editor::cell(&h.bytes(M9), 0, 1), 3);
    // the one-level Undo takes the last back
    h.key_with(Key::Z, Modifiers::COMMAND);
    h.frames(1);
    assert_eq!(h.cell(M9), "Bass drum, step 1, off");

    // the moves: Up to Accent (and no further), Right, End, Home, Down; egui's focus stays
    h.press(Key::ArrowUp);
    assert_eq!(h.cell(M9), "Accent, step 1, off");
    h.press(Key::ArrowUp);
    assert_eq!(h.cell(M9), "Accent, step 1, off");
    h.press(Key::Space);
    assert_eq!(h.cell(M9), "Accent, step 1, on");
    assert_eq!(
        drum_editor::cell(&h.bytes(M9), 0, 0),
        2,
        "the AC byte's on is 2"
    );
    h.press(Key::ArrowRight);
    assert_eq!(h.cell(M9), "Accent, step 2, off");
    h.press(Key::End);
    assert_eq!(h.cell(M9), "Accent, step 16, off");
    h.press(Key::ArrowRight);
    assert_eq!(h.cell(M9), "Accent, step 16, off", "stops at the last step");
    h.press(Key::Home);
    assert_eq!(h.cell(M9), "Accent, step 1, on");
    assert_eq!(
        h.focused(),
        gid.accesskit_id(),
        "the arrows did not move the focus"
    );

    // the Entry: High, then a plain Space enters an accent
    h.focus(editor_drum::id(M9, Part::Entry));
    h.press(Key::ArrowUp);
    h.press(Key::ArrowUp);
    assert_eq!(h.value(M9, Part::Entry), "High (accent)");
    assert_eq!(h.t.app.drum_editor(M9).entry, drum_editor::Entry::High);
    h.focus(gid);
    for _ in 0..9 {
        h.press(Key::ArrowDown);
    }
    assert_eq!(h.cell(M9), "Open hi-hat, step 1, off");
    h.press(Key::Space);
    assert_eq!(h.cell(M9), "Open hi-hat, step 1, accent");
    // entering the closed hi-hat clears the open one on that step, as the panel does
    h.press(Key::ArrowUp);
    h.press_with(Key::Space, CTRL);
    assert_eq!(h.cell(M9), "Closed hi-hat, step 1, on");
    let b = h.bytes(M9);
    assert_eq!(
        (drum_editor::cell(&b, 0, 8), drum_editor::cell(&b, 0, 9)),
        (1, 0)
    );

    // F2 reads the whole pattern, in the panel's words
    assert_eq!(
        h.press(Key::F2).as_deref(),
        Some("909, pattern A1, 16 steps, shuffle off. Accent: 1. Closed hi-hat: 1")
    );
    // the panel's 909 has the edits too (one pattern, the panel's grid)
    assert_eq!(h.t.app.panel.steps909[8][0], 1);
    assert_eq!(h.t.app.panel.steps909[0][0], 2);
}

#[test]
fn the_808_grid_toggles_and_names_its_slots() {
    let mut h = H::new(true);
    h.open(M8);
    h.focus(editor_drum::id(M8, Part::Steps));
    h.press(Key::Space);
    assert_eq!(h.cell(M8), "Bass drum, step 1, on");
    // the 808 reads no modifier
    h.press_with(Key::Space, CTRL);
    assert_eq!(h.cell(M8), "Bass drum, step 1, off");
    h.press_with(Key::Space, Modifiers::SHIFT);
    assert_eq!(h.cell(M8), "Bass drum, step 1, on");
    assert_eq!(drum_editor::cell(&h.bytes(M8), 0, 1), 1);
    // the slots by their switches, as the panel's grid
    h.press(Key::ArrowDown);
    h.press(Key::ArrowDown);
    let want = grid::cell_text(&h.t.app.panel, true, 3, 0);
    assert_eq!(h.cell(M8), want);
    for _ in 0..20 {
        h.press(Key::ArrowDown);
    }
    assert_eq!(h.cell(M8), "Closed hi-hat, step 1, off");
    assert!(h.t.app.panel.steps808[1][0]);
}

#[test]
fn the_header_controls_work_from_the_keyboard_and_say_what_they_did() {
    let mut h = H::new(true);
    h.open(M9);
    // Pattern: Up is the next pattern, selected on the panel too; Page Up a bank
    h.focus(editor_drum::id(M9, Part::Pattern));
    h.press(Key::ArrowUp);
    h.frames(1);
    assert_eq!(h.t.app.drum_editor(M9).pattern, 1);
    assert_eq!(h.value(M9, Part::Pattern), "A2");
    assert_eq!(h.t.app.panel.rows[3].pattern, 1);
    h.press(Key::PageUp);
    h.frames(1);
    assert_eq!(h.value(M9, Part::Pattern), "B2");
    h.press(Key::Home);
    h.frames(1);
    assert_eq!(h.value(M9, Part::Pattern), "A1");
    // Length: Down one, Home 1, End 16; an Undo each; the grid follows the length
    h.focus(editor_drum::id(M9, Part::Length));
    h.press(Key::ArrowDown);
    assert_eq!(h.value(M9, Part::Length), "15 steps");
    assert_eq!(h.bytes(M9)[1], 15);
    assert_eq!(h.t.app.undo_name(), Some("Pattern Length"));
    let g = h.t.tree().by_id(editor_drum::id(M9, Part::Steps)).unwrap();
    assert_eq!(g.column_count(), Some(15));
    h.press(Key::Home);
    assert_eq!(h.value(M9, Part::Length), "1 step");
    h.press(Key::End);
    assert_eq!(h.value(M9, Part::Length), "16 steps");
    // Shuffle: Space toggles it
    h.focus(editor_drum::id(M9, Part::Shuffle));
    h.press(Key::Space);
    let n =
        h.t.tree()
            .by_id(editor_drum::id(M9, Part::Shuffle))
            .unwrap();
    assert_eq!(a11y::toggled(n), Some(Toggled::True));
    assert_eq!(h.bytes(M9)[0], 1);
    // a Shift that changes nothing (an empty pattern) says so, and writes nothing
    let undo = h.t.app.undo_name();
    h.t.send(editor_drum::id(M9, Part::ShiftRight), Action::Click, None);
    assert_eq!(h.said(), "Pattern A1 unchanged");
    assert_eq!(h.t.app.undo_name(), undo, "no write, the Undo stays");
    // with a step set, Shift says what it did
    h.focus(editor_drum::id(M9, Part::Steps));
    h.press(Key::Space);
    h.focus(editor_drum::id(M9, Part::ShiftRight));
    assert_eq!(
        h.press(Key::Space).as_deref(),
        Some("Pattern A1 shifted right")
    );
    assert_eq!(drum_editor::cell(&h.bytes(M9), 1, 1), 1);
    assert_eq!(h.t.app.undo_name(), Some("Shift Pattern"));
    h.t.send(editor_drum::id(M9, Part::ShiftLeft), Action::Click, None);
    assert_eq!(h.said(), "Pattern A1 shifted left");
    assert_eq!(drum_editor::cell(&h.bytes(M9), 0, 1), 1);
    // Copy and Clear speak through the status line
    h.t.send(editor_drum::id(M9, Part::Copy), Action::Click, None);
    assert_eq!(h.said(), "909: A1 copied");
    h.t.send(editor_drum::id(M9, Part::Clear), Action::Click, None);
    assert_eq!(h.said(), "909: A1 cleared");
    assert_eq!(drum_editor::cell(&h.bytes(M9), 0, 1), 0);
    h.t.send(editor_drum::id(M9, Part::Paste), Action::Click, None);
    assert_eq!(drum_editor::cell(&h.bytes(M9), 0, 1), 1, "pasted back");
}

/// Move-along doesn't speak: while the song plays, and when playback moves the 909's row to
/// another pattern, the focused window's values do not change and nothing is said.
#[test]
fn playback_says_nothing_and_does_not_move_the_window() {
    let mut h = H::new(true);
    h.open(M9);
    h.focus(editor_drum::id(M9, Part::Steps));
    h.press(Key::ArrowRight);
    h.press(Key::F5);
    assert!(h.t.app.panel.transport.playing);
    let n = h.t.app.announcer.serial();
    let (cell, pattern) = (h.cell(M9), h.value(M9, Part::Pattern));
    for _ in 0..60 {
        h.calls(4);
    }
    // the engine's playback moving the row to another pattern (`Link::follow_values`)
    h.t.app.panel.adopt_pattern(3, 5);
    h.frames(5);
    assert_eq!(h.t.app.announcer.serial(), n, "nothing was said");
    assert_eq!(h.cell(M9), cell);
    assert_eq!(
        h.value(M9, Part::Pattern),
        pattern,
        "the window stays on A1"
    );
    assert_eq!(h.t.app.drum_editor(M9).pattern, 0);
}

#[test]
fn with_the_mode_off_the_windows_are_upstream_s() {
    let mut h = H::new(false);
    h.open(M8);
    h.open(M9);
    let t = h.t.tree();
    for m in Machine::ALL {
        assert!(h.t.app.drum_editor(m).open);
        for p in Part::ALL {
            assert!(t.by_id(editor_drum::id(m, p)).is_none(), "{m:?} {p:?}");
        }
        // the painted grid is there, and the header's egui controls
        assert!(h.t.app.drum_editor_cell(m, 1, 0).is_some());
        assert!(h.t.app.drum_editor_hit(m, "Copy").is_some());
        assert!(h.t.app.drum_editor_hit(m, "number").is_some());
    }
    assert!(
        t.by_role(Role::Dialog).next().is_none(),
        "egui's own Window node"
    );
    assert!(
        h.t.app.drum_editor_hit(M9, "Cycle").is_some(),
        "the entry switch"
    );
}
