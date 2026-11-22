//! The 808 pattern editor (task-280, docs/ui_spec.md section 24), headless: the Edit menu item
//! and Ctrl/Cmd+E with the TR-808 focused open it; 11 drum rows whose labels follow the panel's
//! switches and the AC row; clicks toggle, drags paint along their row; Shift, Clear, Copy,
//! Paste, Length and Shuffle; every change is one Undo and goes through the panel's pattern path
//! (heard, shown); the selection rules of pattern and song mode; and every input the Keyboard
//! Shortcuts help lists for it does what the help says.

use std::sync::{Arc, OnceLock};

use eframe::egui::{self, Event, Modifiers, PointerButton, Pos2, RawInput, Rect, Vec2};
use orb_engine::tables::Tables;
use orb_player::{Audio, CALL_FRAMES};
use orb_ui::app::App;
use orb_ui::drum_editor::{AC, DrumAct, Input, Machine, cell};
use orb_ui::layout::{PANEL_H, PANEL_W};
use orb_ui::link::{self, Link};
use orb_ui::menu::{Cmd, MENU_H};
use orb_ui::model::Panel;
use orb_ui::settings::Settings;

const M: Machine = Machine::Tr808;
/// The TR-808's row.
const ROW: usize = 2;

fn tables() -> Arc<Tables> {
    static T: OnceLock<Arc<Tables>> = OnceLock::new();
    T.get_or_init(|| Arc::new(Tables::new())).clone()
}

struct Harness {
    ctx: egui::Context,
    app: App,
    audio: Audio,
    time: f64,
    /// The modifiers held in the next frames.
    mods: Modifiers,
}

impl Harness {
    /// The app with the menu bar and a new song, in a window as high as the panel and the bar.
    fn new() -> Self {
        let ctx = egui::Context::default();
        egui_extras::install_image_loaders(&ctx);
        let samples = rbm::standard::builtin_samples();
        let (link, audio) = Link::new(tables(), Some(samples.clone()));
        let mut app = App::with_link(Panel::new(), link, None);
        app.samples = Some(samples);
        app.menu = true;
        let mut h = Self {
            ctx,
            app,
            audio,
            time: 0.0,
            mods: Modifiers::NONE,
        };
        h.calls(1);
        h
    }

    fn frame(&mut self, mut events: Vec<Event>) {
        self.time += 0.016;
        events.insert(0, Event::ModifiersChanged(self.mods));
        let input = RawInput {
            screen_rect: Some(Rect::from_min_size(
                Pos2::ZERO,
                Vec2::new(PANEL_W, PANEL_H + MENU_H),
            )),
            time: Some(self.time),
            events,
            ..Default::default()
        };
        let app = &mut self.app;
        let mut out = self.ctx.run_ui(input, |ui| app.show(ui));
        out.textures_delta.clear();
    }

    /// `n` engine calls, then a UI frame that takes the snapshot.
    fn calls(&mut self, n: usize) {
        let mut out = vec![0i16; 2 * CALL_FRAMES];
        for _ in 0..n {
            self.audio.render_i16(&mut out);
        }
        self.frame(vec![]);
    }

    fn button(&self, p: Pos2, pressed: bool) -> Event {
        Event::PointerButton {
            pos: p,
            button: PointerButton::Primary,
            pressed,
            modifiers: self.mods,
        }
    }

    fn click(&mut self, p: Pos2) {
        self.frame(vec![Event::PointerMoved(p)]);
        self.frame(vec![self.button(p, true)]);
        self.frame(vec![self.button(p, false)]);
    }

    /// A press at the first point, the mouse moved through the others (a frame each), and the
    /// release at the last.
    fn drag(&mut self, points: &[Pos2]) {
        let first = points[0];
        self.frame(vec![Event::PointerMoved(first)]);
        self.frame(vec![self.button(first, true)]);
        for &p in &points[1..] {
            self.frame(vec![Event::PointerMoved(p)]);
        }
        let last = *points.last().unwrap();
        self.frame(vec![self.button(last, false)]);
        self.frame(vec![]);
    }

    fn key(&mut self, key: egui::Key, modifiers: Modifiers) {
        let e = |pressed| Event::Key {
            key,
            physical_key: Some(key),
            pressed,
            repeat: false,
            modifiers,
        };
        self.frame(vec![e(true)]);
        self.frame(vec![e(false)]);
    }

    /// Opens the editor (the command of the menu item) and draws it.
    fn open_editor(&mut self) {
        self.app.command(Cmd::DrumEditor(M));
        for _ in 0..3 {
            self.frame(vec![]);
        }
        assert!(self.app.drum_editor(M).open);
    }

    /// The editor's pattern as the link holds it.
    fn bytes(&self) -> Vec<u8> {
        let p = self.app.drum_editor(M).pattern;
        self.app.link.as_ref().unwrap().pattern(ROW, p).to_vec()
    }

    /// The center of instrument `column`'s cell (0 the AC row) of step `step`.
    fn cell(&self, column: usize, step: usize) -> Pos2 {
        self.app
            .drum_editor_cell(M, column, step)
            .expect("the grid is drawn")
            .center()
    }

    fn hit(&self, name: &str) -> Pos2 {
        self.app
            .drum_editor_hit(M, name)
            .unwrap_or_else(|| panic!("{name} is drawn"))
            .center()
    }

    fn press(&mut self, name: &str) {
        let p = self.hit(name);
        self.click(p);
    }

    fn undo(&mut self) {
        self.key(egui::Key::Z, Modifiers::COMMAND);
    }

    /// Selects pattern `p` (0..=31) of row `r` as a click on its button does.
    fn select(&mut self, r: usize, p: u8) {
        self.app.panel.rows[r].bank = p / 8;
        self.app
            .panel
            .press(&format!("pattern{}.pattern{}", r + 1, p % 8 + 1), 0.0);
        self.frame(vec![]);
    }

    /// Song mode, taken from the engine.
    fn song_mode(&mut self) {
        self.app.panel.set_switch("transport.mode", 1);
        self.calls(2);
        self.calls(2);
        assert!(self.app.panel.transport.song_mode);
    }

    /// The menu bar's Edit menu opened (found along the bar).
    fn open_edit_menu(&mut self) {
        for x in (6..300).step_by(4) {
            egui::Popup::close_all(&self.ctx);
            self.click(Pos2::new(x as f32, MENU_H / 2.0));
            self.frame(vec![]);
            if self.app.edit_item(M.menu_item()).is_some() {
                return;
            }
        }
        panic!("the Edit menu has the item");
    }
}

/// The engine's patterns of the TR-808.
fn engine_808(audio: &mut Audio, p: u8) -> Vec<u8> {
    let channels = &mut audio.current_mut().frame.devices.channels;
    channels[ROW].patterns_mut().unwrap()[194 * p as usize..][..194].to_vec()
}

#[test]
fn the_menu_item_and_cmd_e_open_it_on_the_808s_pattern_and_bring_it_to_the_front() {
    let mut h = Harness::new();
    h.select(ROW, 5);
    // the menu item: the 808's A6
    h.app.panel.focus = 0;
    h.open_edit_menu();
    let (r, enabled) = h.app.edit_item(M.menu_item()).unwrap();
    assert!(enabled);
    // the items in order: 303's, then 808's
    let (r303, _) = h.app.edit_item("303 Pattern Editor...").unwrap();
    assert!(r303.top() < r.top());
    h.click(r.center());
    h.frame(vec![]);
    let e = h.app.drum_editor(M);
    assert!(e.open);
    assert_eq!(e.pattern, 5);
    assert!(!h.app.pattern_editor().open, "the 303's stays closed");

    // File > Info opens over it; Cmd+E with the 808 focused brings it back to the front, on
    // the 808's selected pattern
    h.app.command(Cmd::Info);
    for _ in 0..3 {
        h.frame(vec![]);
    }
    let p = h.cell(3, 8);
    let id = egui::Id::unique("orb_drum_editor_808");
    let in_front = |h: &Harness| h.ctx.layer_id_at(p).map(|l| l.id) == Some(id);
    assert!(!in_front(&h), "the info window covers the editor");
    h.select(ROW, 9);
    h.app.panel.focus = ROW;
    h.key(egui::Key::E, Modifiers::COMMAND);
    h.frame(vec![]);
    assert!(in_front(&h), "the editor is in front");
    assert_eq!(h.app.drum_editor(M).pattern, 9);
    // with a 303 focused, the key opens the 303 editor; the 808's stays open
    h.app.panel.focus = 1;
    h.key(egui::Key::E, Modifiers::COMMAND);
    assert!(h.app.pattern_editor().open);
    assert_eq!(h.app.pattern_editor().synth, 1);
    assert!(h.app.drum_editor(M).open);
}

#[test]
fn the_header_the_rows_and_the_window_that_does_not_resize() {
    let mut h = Harness::new();
    h.open_editor();
    for name in [
        "A",
        "B",
        "C",
        "D",
        "number_prev",
        "number",
        "number_next",
        "Copy",
        "Paste",
        "Clear",
        "Length",
        "Shuffle",
        "shift_left",
        "shift_right",
    ] {
        assert!(h.app.drum_editor_hit(M, name).is_some(), "{name}");
    }
    // no 303 switch, no transpose
    for name in ["303-1", "303-2", "transpose_up", "transpose_down"] {
        assert!(h.app.drum_editor_hit(M, name).is_none(), "{name}");
    }
    // 11 drum rows as the panel names them, then AC
    let mut want = M.labels(&h.app.panel).to_vec();
    want.push("AC");
    assert_eq!(h.app.drum_editor_labels(M), want);
    let top = h.app.drum_editor_cell(M, 1, 0).unwrap();
    let drums = top.union(h.app.drum_editor_cell(M, 11, 15).unwrap());
    let ac = h
        .app
        .drum_editor_cell(M, AC, 0)
        .unwrap()
        .union(h.app.drum_editor_cell(M, AC, 15).unwrap());
    assert!((drums.height() - 11.0 * top.height()).abs() < 0.5);
    // the AC row under the drum rows, a divider's gap between
    let gap = ac.top() - drums.bottom();
    assert!((4.0..=16.0).contains(&gap), "{gap}");
    assert_eq!((ac.left(), ac.right()), (drums.left(), drums.right()));
    // Shift: under the AC row, centered under the grid, one size
    let (sl, sr) = (
        h.app.drum_editor_hit(M, "shift_left").unwrap(),
        h.app.drum_editor_hit(M, "shift_right").unwrap(),
    );
    assert!(sl.top() > ac.bottom());
    let mid = (sl.left() + sr.right()) / 2.0;
    assert!((mid - drums.center().x).abs() < 1.0, "{mid} {drums:?}");
    assert_eq!(sl.size(), sr.size());
    // the columns fill the window's width
    let w = h.app.drum_editor_window(M).unwrap();
    assert!(w.right() - drums.right() < 12.0, "{w:?} {drums:?}");
    assert!(top.width() >= 22.0, "{top:?}");
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(PANEL_W, PANEL_H + MENU_H));
    assert!(screen.contains_rect(w), "{w:?}");
    // a drag at the bottom right corner does not resize it
    let corner = w.max - Vec2::splat(2.0);
    h.drag(&[corner, corner + Vec2::new(80.0, 60.0)]);
    assert_eq!(h.app.drum_editor_window(M).unwrap().size(), w.size());
    // the close button closes it
    let w = h.app.drum_editor_window(M).unwrap();
    h.click(Pos2::new(w.right() - 14.0, w.top() + 12.0));
    assert!(!h.app.drum_editor(M).open, "closed");
}

#[test]
fn the_row_labels_follow_the_panels_switches() {
    let mut h = Harness::new();
    h.open_editor();
    let before = h.bytes();
    // all down (the lower legend), then three up
    for k in 3..=7 {
        h.app.panel.set_switch(&format!("tr808.slot{k}_switch"), 1);
    }
    h.calls(2);
    assert_eq!(
        h.app.drum_editor_labels(M),
        [
            "BD", "SD", "LT", "MT", "HT", "RS", "CP", "CB", "CY", "OH", "CH", "AC"
        ]
    );
    for k in [3, 5, 7] {
        h.app.panel.set_switch(&format!("tr808.slot{k}_switch"), 0);
    }
    h.calls(2);
    assert_eq!(
        h.app.drum_editor_labels(M),
        [
            "BD", "SD", "LC", "MT", "HC", "RS", "MA", "CB", "CY", "OH", "CH", "AC"
        ]
    );
    h.app.panel.set_switch("tr808.slot3_switch", 1);
    h.frame(vec![]);
    assert_eq!(h.app.drum_editor_labels(M)[2], "LT");
    assert_eq!(h.bytes(), before, "the switches are no pattern data");
}

#[test]
fn clicks_toggle_cells_and_each_is_one_undo() {
    let mut h = Harness::new();
    h.open_editor();
    let empty = h.bytes();
    assert!(link::is_empty_pattern(ROW, &empty));
    // BD on step 3
    h.click(h.cell(1, 3));
    assert_eq!(cell(&h.bytes(), 3, 1), 1);
    assert_eq!(h.app.undo_name(), Some("Edit Pattern"));
    h.undo();
    assert_eq!(h.bytes(), empty, "the click undone");
    h.click(h.cell(1, 3));
    // CH on step 3, AC on step 3
    h.click(h.cell(11, 3));
    h.click(h.cell(AC, 3));
    let b = h.bytes();
    assert_eq!(
        (cell(&b, 3, 1), cell(&b, 3, 11), cell(&b, 3, AC)),
        (1, 1, 1)
    );
    // a click on a lit cell clears it; Undo brings it back
    h.click(h.cell(11, 3));
    assert_eq!(cell(&h.bytes(), 3, 11), 0);
    h.undo();
    assert_eq!(h.bytes(), b);
    // heard and shown: the engine and the panel's step LEDs have it, the song is modified
    h.calls(1);
    assert_eq!(engine_808(&mut h.audio, 0), b);
    assert!(h.app.panel.steps808[1][3]);
    assert!(h.app.panel.steps808[0][3]);
    assert!(h.app.doc.modified);
}

#[test]
fn a_drag_paints_the_first_cells_new_value_along_its_row_only() {
    let mut h = Harness::new();
    h.open_editor();
    // SD from step 2: the mouse goes down into the CH row and to the AC row, the drag stays in
    // SD; steps skipped between frames are painted too
    h.drag(&[h.cell(2, 2), h.cell(9, 4), h.cell(AC, 6)]);
    let b = h.bytes();
    let sd: Vec<_> = (0..8).map(|s| cell(&b, s, 2)).collect();
    assert_eq!(sd, [0, 0, 1, 1, 1, 1, 1, 0]);
    assert_eq!(
        b.iter().filter(|&&v| v != 0).count(),
        5 + 1,
        "only SD (and the length)"
    );
    // one change
    h.undo();
    assert!(link::is_empty_pattern(ROW, &h.bytes()));
    h.drag(&[h.cell(2, 2), h.cell(2, 6)]);
    // from a lit cell it erases
    h.drag(&[h.cell(2, 3), h.cell(2, 5)]);
    let sd: Vec<_> = (0..8).map(|s| cell(&h.bytes(), s, 2)).collect();
    assert_eq!(sd, [0, 0, 1, 0, 0, 0, 1, 0]);
    h.undo();
    assert!((2..7).all(|s| cell(&h.bytes(), s, 2) == 1));
    // the AC row paints in its row too, also when the mouse goes up into the drum rows
    h.drag(&[h.cell(AC, 10), h.cell(5, 12)]);
    let b = h.bytes();
    assert!((10..=12).all(|s| cell(&b, s, AC) == 1));
    assert!((10..=12).all(|s| cell(&b, s, 5) == 0));
}

#[test]
fn steps_beyond_the_length_are_dimmed_and_editable() {
    let mut h = Harness::new();
    h.open_editor();
    assert!(!h.app.drum_editor_shown(M).1);
    h.app
        .drum_editor_act(M, DrumAct::Length { n: 4, drag: false });
    h.frame(vec![]);
    assert_eq!(h.bytes()[1], 4);
    assert!(h.app.drum_editor_shown(M).1);
    h.click(h.cell(6, 10));
    assert_eq!(cell(&h.bytes(), 10, 6), 1);
}

#[test]
fn shift_clear_copy_paste_length_shuffle_and_their_undo() {
    let mut h = Harness::new();
    h.open_editor();
    // BD on 0, SD on 1, CH on 3; length 4
    for (col, s) in [(1, 0), (2, 1), (11, 3)] {
        h.click(h.cell(col, s));
    }
    h.app
        .drum_editor_act(M, DrumAct::Length { n: 4, drag: false });
    h.frame(vec![]);
    let base = h.bytes();
    // Shift right wraps within the length: CH from step 3 to step 0
    h.press("shift_right");
    let b = h.bytes();
    assert_eq!((cell(&b, 0, 11), cell(&b, 1, 1), cell(&b, 2, 2)), (1, 1, 1));
    assert_eq!(h.app.undo_name(), Some("Shift Pattern"));
    h.undo();
    assert_eq!(h.bytes(), base);
    h.press("shift_left");
    assert_eq!(cell(&h.bytes(), 3, 1), 1, "BD from 0 to 3");
    h.undo();
    // Shuffle toggles byte 0; Undo
    h.press("Shuffle");
    assert_eq!(h.bytes()[0], 1);
    assert_eq!(h.app.undo_name(), Some("Pattern Shuffle"));
    h.undo();
    assert_eq!(h.bytes(), base);
    // Copy, then Paste into B1; Clear keeps the length and shuffle; Undo
    h.press("Copy");
    assert_eq!(h.app.clip.as_ref().unwrap().row, ROW);
    assert_eq!(h.app.clip.as_ref().unwrap().bytes, base);
    h.press("B");
    assert_eq!(h.app.drum_editor(M).pattern, 8);
    h.press("Paste");
    assert_eq!(h.bytes(), base);
    assert_eq!(h.app.undo_name(), Some("Paste Pattern"));
    h.press("Clear");
    let b = h.bytes();
    assert!(link::is_empty_pattern(ROW, &b));
    assert_eq!(b[1], 4, "the length kept");
    assert_eq!(h.app.undo_name(), Some("Clear Pattern"));
    assert_eq!(h.app.status.as_deref(), Some("TR-808: B1 cleared"));
    assert_eq!(
        h.app.clip.as_ref().unwrap().bytes,
        base,
        "Clear leaves the clipboard"
    );
    h.undo();
    assert_eq!(h.bytes(), base);
    // Paste is enabled only with an 808 pattern on the clipboard
    h.app.clip.as_mut().unwrap().row = 0;
    h.frame(vec![]);
    h.press("Paste");
    assert_eq!(h.app.undo_name(), None, "Paste disabled");
    assert_eq!(h.bytes(), base);
}

#[test]
fn pattern_mode_selections_follow_each_other_both_ways() {
    let mut h = Harness::new();
    h.open_editor();
    assert!(!h.app.panel.transport.song_mode);
    h.press("C");
    assert_eq!(h.app.panel.rows[ROW].pattern, 16);
    assert_eq!(h.app.drum_editor(M).pattern, 16);
    // the arrows run A1..D8 and stop at the ends
    h.press("number_next");
    assert_eq!(h.app.drum_editor(M).pattern, 17);
    assert_eq!(h.app.panel.rows[ROW].pattern, 17);
    h.app.drum_editor_act(M, DrumAct::Select(7));
    h.frame(vec![]);
    h.press("number_next");
    assert_eq!(h.app.drum_editor(M).pattern, 8, "A8 to B1");
    assert_eq!(h.app.panel.rows[ROW].bank, 1);
    h.app.drum_editor_act(M, DrumAct::Select(31));
    h.frame(vec![]);
    h.press("number_next");
    assert_eq!(h.app.drum_editor(M).pattern, 31, "D8 stops");
    h.app.drum_editor_act(M, DrumAct::Select(0));
    h.frame(vec![]);
    h.press("number_prev");
    assert_eq!(h.app.drum_editor(M).pattern, 0, "A1 stops");
    // a panel selection moves the editor; the other rows do not
    h.select(ROW, 4);
    h.frame(vec![]);
    assert_eq!(h.app.drum_editor(M).pattern, 4);
    h.select(3, 9);
    h.select(0, 12);
    h.frame(vec![]);
    assert_eq!(h.app.drum_editor(M).pattern, 4);
}

#[test]
fn song_mode_selections_are_independent_and_never_recorded() {
    let mut h = Harness::new();
    h.select(ROW, 3);
    h.open_editor();
    h.song_mode();
    h.app.panel.press("transport.record", 0.0);
    h.calls(2);
    let panel_pattern = h.app.panel.rows[ROW].pattern;
    let tracks = h.app.link.as_ref().unwrap().loaded_tracks().clone();
    h.press("B");
    h.app.drum_editor_act(M, DrumAct::Select(12));
    h.calls(2);
    assert_eq!(h.app.drum_editor(M).pattern, 12);
    assert_eq!(h.app.panel.rows[ROW].pattern, panel_pattern);
    // an edit is heard and changes no selection
    h.click(h.cell(1, 0));
    h.calls(2);
    assert_eq!(cell(&engine_808(&mut h.audio, 12), 0, 1), 1);
    assert_eq!(h.app.panel.rows[ROW].pattern, panel_pattern);
    assert_eq!(h.app.link.as_ref().unwrap().loaded_tracks(), &tracks);
    // a panel selection does not move the editor
    h.select(ROW, 6);
    h.calls(2);
    assert_eq!(h.app.drum_editor(M).pattern, 12);
}

#[test]
fn the_playing_step_is_shown_in_the_pattern_it_plays() {
    let mut h = Harness::new();
    h.open_editor();
    h.app.panel.press("transport.play", 0.0);
    let mut seen = None;
    for _ in 0..200 {
        h.calls(4);
        if let Some(s) = h.app.panel.playing_step808 {
            seen = Some(s);
            break;
        }
    }
    let s = seen.expect("the 808 plays");
    assert_eq!(h.app.drum_editor_shown(M).0, Some(s));
    // another pattern shown: no playing step
    h.song_mode();
    h.app.drum_editor_act(M, DrumAct::Select(20));
    h.calls(4);
    assert_eq!(h.app.drum_editor_shown(M).0, None);
}

#[test]
fn its_place_is_kept_in_the_settings_file() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("ui.toml");
    let mut h = Harness::new();
    h.app.settings = Settings::load_from(&file);
    h.open_editor();
    let w = h.app.drum_editor_window(M).unwrap();
    assert!(!file.exists(), "opening writes nothing");
    let title = Pos2::new(w.left() + 60.0, w.top() + 10.0);
    h.drag(&[
        title,
        title + Vec2::new(-10.0, 15.0),
        title + Vec2::new(-20.0, 30.0),
    ]);
    let moved = h.app.drum_editor_window(M).unwrap();
    assert_eq!(moved.min, w.min + Vec2::new(-20.0, 30.0));
    let saved = Settings::load_from(&file);
    let want = [moved.min.x.round() as i32, moved.min.y.round() as i32];
    assert_eq!(saved.pattern_editor_808_pos, Some(want));
    assert_eq!(saved.pattern_editor_pos, None, "the 303's is its own");
    let mut h2 = Harness::new();
    h2.app.settings = Settings::load_from(&file);
    h2.open_editor();
    let w2 = h2.app.drum_editor_window(M).unwrap();
    assert_eq!(w2.min, Pos2::new(want[0] as f32, want[1] as f32));
}

/// Every input Help > Keyboard Shortcuts lists for the 808 editor (`Input::ALL`, which the
/// help's test requires it to list) does what the help says. A new input added to `Input`
/// fails to compile here until it is tested.
#[test]
fn every_input_of_the_help_does_what_it_says() {
    for i in Input::ALL {
        let mut h = Harness::new();
        h.open_editor();
        match i {
            Input::OpenKey => {
                // song mode, the editor on another pattern than the 808's active one
                h.song_mode();
                h.app.panel.focus = ROW;
                let active = h.app.panel.rows[ROW].pattern;
                h.app.drum_editor_act(M, DrumAct::Select((active + 5) % 32));
                h.frame(vec![]);
                assert_ne!(h.app.drum_editor(M).pattern, active);
                h.key(egui::Key::E, Modifiers::COMMAND);
                assert_eq!(
                    h.app.drum_editor(M).pattern,
                    active,
                    "switched to the active one"
                );
                // closed, the key opens it
                let w = h.app.drum_editor_window(M).unwrap();
                h.click(Pos2::new(w.right() - 14.0, w.top() + 12.0));
                assert!(!h.app.drum_editor(M).open);
                h.key(egui::Key::E, Modifiers::COMMAND);
                assert!(h.app.drum_editor(M).open);
            }
            Input::Click => {
                h.click(h.cell(4, 7));
                assert_eq!(cell(&h.bytes(), 7, 4), 1);
                h.click(h.cell(4, 7));
                assert_eq!(cell(&h.bytes(), 7, 4), 0);
            }
            Input::Drag => {
                h.drag(&[h.cell(6, 0), h.cell(7, 3)]);
                let b = h.bytes();
                assert!((0..4).all(|s| cell(&b, s, 6) == 1 && cell(&b, s, 7) == 0));
            }
            Input::Modified => {
                for mods in [
                    Modifiers::SHIFT,
                    Modifiers::COMMAND,
                    Modifiers::ALT,
                    Modifiers::CTRL,
                    Modifiers::MAC_CMD,
                ] {
                    h.mods = mods;
                    let before = h.bytes();
                    h.click(h.cell(1, 5));
                    assert_eq!(cell(&h.bytes(), 5, 1), 1 - cell(&before, 5, 1), "{mods:?}");
                    let mut want = before.clone();
                    want[2 + 12 * 5 + 1] ^= 1;
                    assert_eq!(h.bytes(), want, "{mods:?}: the click alone");
                    h.drag(&[h.cell(3, 8), h.cell(3, 10)]);
                    let v = cell(&h.bytes(), 8, 3);
                    assert!((8..=10).all(|s| cell(&h.bytes(), s, 3) == v), "{mods:?}");
                    h.mods = Modifiers::NONE;
                    h.frame(vec![]);
                }
            }
            Input::ValueDrag | Input::ValueFineDrag => {
                let fine = i == Input::ValueFineDrag;
                let p = h.hit("Length");
                if fine {
                    h.mods = Modifiers::SHIFT;
                }
                // 60 points down: Length 16 goes down (a drag down lowers it)
                let path: Vec<Pos2> = (0..=6)
                    .map(|k| p + Vec2::new(0.0, 10.0 * k as f32))
                    .collect();
                h.drag(&path);
                h.mods = Modifiers::NONE;
                let n = h.bytes()[1];
                if fine {
                    assert!((14..16).contains(&n), "Shift: finer, {n}");
                } else {
                    assert!(n < 14, "{n}");
                    // a drag is one change
                    h.undo();
                    assert_eq!(h.bytes()[1], 16);
                }
            }
            Input::ValueType | Input::ValueCancel => {
                let p = h.hit("Length");
                h.click(p);
                h.frame(vec![Event::Text("7".into())]);
                assert_eq!(h.bytes()[1], 16, "not set while typing");
                // the panel's keys rest while typing: Space does not play
                h.key(egui::Key::Space, Modifiers::NONE);
                if i == Input::ValueType {
                    h.key(egui::Key::Enter, Modifiers::NONE);
                    h.frame(vec![]);
                    assert_eq!(h.bytes()[1], 7);
                    assert_eq!(h.app.undo_name(), Some("Pattern Length"));
                } else {
                    h.key(egui::Key::Escape, Modifiers::NONE);
                    h.frame(vec![]);
                    assert_eq!(h.bytes()[1], 16, "cancelled");
                }
                h.calls(4);
                assert!(!h.app.panel.transport.playing, "Space went to the field");
            }
            Input::Undo => {
                let before = h.bytes();
                h.click(h.cell(9, 9));
                h.key(egui::Key::Z, Modifiers::COMMAND);
                assert_eq!(h.bytes(), before);
            }
        }
    }
}
