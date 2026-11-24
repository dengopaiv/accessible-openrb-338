//! The 303 pattern editor (task-274, docs/ui_spec.md section 23), headless: the Edit menu item
//! and Ctrl/Cmd+E open it, the grids edit the pattern by clicks and drags, Shift, Transpose,
//! Clear, Copy and Paste, every change is one Undo and goes through the panel's pattern path
//! (heard, saved, shown), and the selection rules of pattern and song mode.

use std::sync::{Arc, OnceLock};

use eframe::egui::{self, Event, Modifiers, PointerButton, Pos2, RawInput, Rect, Vec2};
use orb_engine::tables::Tables;
use orb_player::{Audio, CALL_FRAMES};
use orb_ui::app::App;
use orb_ui::drum_editor::Machine;
use orb_ui::layout::{PANEL_H, PANEL_W};
use orb_ui::link::{self, Link};
use orb_ui::menu::{Cmd, MENU_H};
use orb_ui::model::Panel;
use orb_ui::pattern_editor::{self, Act, Flag, PITCHES, flag, length, note};
use orb_ui::settings::Settings;

fn tables() -> Arc<Tables> {
    static T: OnceLock<Arc<Tables>> = OnceLock::new();
    T.get_or_init(|| Arc::new(Tables::new())).clone()
}

struct Harness {
    ctx: egui::Context,
    app: App,
    audio: Audio,
    time: f64,
    /// The 303 whose editor the helpers use (0 or 1).
    k: usize,
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
            k: 0,
        };
        h.calls(1);
        h
    }

    fn frame(&mut self, mut events: Vec<Event>) {
        self.time += 0.016;
        events.insert(0, Event::ModifiersChanged(Modifiers::NONE));
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

    fn button(p: Pos2, pressed: bool) -> Event {
        Event::PointerButton {
            pos: p,
            button: PointerButton::Primary,
            pressed,
            modifiers: Modifiers::NONE,
        }
    }

    fn click(&mut self, p: Pos2) {
        self.frame(vec![Event::PointerMoved(p)]);
        self.frame(vec![Self::button(p, true)]);
        self.frame(vec![Self::button(p, false)]);
    }

    /// A press at the first point, the mouse moved through the others (a frame each), and the
    /// release at the last.
    fn drag(&mut self, points: &[Pos2]) {
        let first = points[0];
        self.frame(vec![Event::PointerMoved(first)]);
        self.frame(vec![Self::button(first, true)]);
        for &p in &points[1..] {
            self.frame(vec![Event::PointerMoved(p)]);
        }
        let last = *points.last().unwrap();
        self.frame(vec![Self::button(last, false)]);
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
        self.app.command(Cmd::PatternEditor(self.k));
        self.frame(vec![]);
        self.frame(vec![]);
        self.frame(vec![]);
        assert!(self.app.pattern_editor(self.k).open);
    }

    /// The editor's pattern as the link holds it.
    fn bytes(&self) -> Vec<u8> {
        let e = self.app.pattern_editor(self.k);
        self.app
            .link
            .as_ref()
            .unwrap()
            .pattern(e.synth, e.pattern)
            .to_vec()
    }

    fn note_cell(&self, step: usize, pitch: u8) -> Pos2 {
        self.app
            .pattern_editor_note_cell(self.k, step, pitch)
            .expect("the piano roll is drawn")
            .center()
    }

    fn flag_cell(&self, f: Flag, step: usize) -> Pos2 {
        self.app
            .pattern_editor_flag_cell(self.k, f, step)
            .expect("the flag rows are drawn")
            .center()
    }

    fn hit(&self, name: &str) -> Pos2 {
        self.app
            .pattern_editor_hit(self.k, name)
            .unwrap_or_else(|| panic!("{name} is drawn"))
            .center()
    }

    /// Clicks the editor's button `name`.
    fn press(&mut self, name: &str) {
        let p = self.hit(name);
        self.click(p);
    }

    /// Edit > Undo.
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
}

/// The engine's patterns of the two 303s.
fn engine_303(audio: &mut Audio, k: usize, p: u8) -> Vec<u8> {
    let channels = &mut audio.current_mut().frame.devices.channels;
    channels[k].patterns_mut().unwrap()[98 * p as usize..][..98].to_vec()
}

#[test]
fn the_menu_items_and_cmd_e_open_each_303s_window_and_bring_it_to_the_front() {
    let mut h = Harness::new();
    // 303-2, A3: the focused row's selected pattern
    h.select(1, 2);
    assert_eq!(h.app.panel.focus, 1);
    // the Edit menu: found along the bar, 303-2's item clicked
    let mut found = None;
    for x in (6..300).step_by(4) {
        egui::Popup::close_all(&h.ctx);
        h.click(Pos2::new(x as f32, MENU_H / 2.0));
        h.frame(vec![]);
        if let Some((r, on)) = h.app.edit_item("303-2 Pattern Editor...") {
            found = Some((r, on));
            break;
        }
    }
    let (r, enabled) = found.expect("the Edit menu has the item");
    assert!(enabled);
    let (r1, _) = h
        .app
        .edit_item("303-1 Pattern Editor...")
        .expect("303-1's item");
    assert!(r1.top() < r.top(), "303-1's item first");
    assert!(h.app.edit_item("303 Pattern Editor...").is_none());
    h.click(r.center());
    h.frame(vec![]);
    let e = h.app.pattern_editor(1);
    assert!(e.open);
    assert_eq!((e.synth, e.pattern), (1, 2));
    assert!(!h.app.pattern_editor(0).open, "303-1's stays closed");
    assert_eq!(
        h.app.focused_editor(),
        Some(1),
        "the opened editor has the focus"
    );

    // Cmd+E with 303-1 focused opens 303-1's window beside it, on its selected pattern
    h.app.panel.rows[0].pattern = 5;
    h.app.panel.focus = 0;
    h.key(egui::Key::E, Modifiers::COMMAND);
    for _ in 0..3 {
        h.frame(vec![]);
    }
    let e = h.app.pattern_editor(0);
    assert!(e.open);
    assert_eq!((e.synth, e.pattern), (0, 5));
    assert!(h.app.pattern_editor(1).open, "both are open");
    assert!(h.app.pattern_editor_window(0).is_some() && h.app.pattern_editor_window(1).is_some());

    // File > Info opens over 303-2's; Cmd+E with 303-2 focused brings it back to the front
    h.app.command(Cmd::Info);
    h.frame(vec![]);
    h.frame(vec![]);
    let p = h.app.pattern_editor_note_cell(1, 8, 6).unwrap().center();
    let in_front = |h: &Harness| {
        h.ctx.layer_id_at(p).map(|l| l.id) == Some(egui::Id::unique(pattern_editor::salt(1)))
    };
    assert!(!in_front(&h), "the info window covers the editor");
    h.app.panel.focus = 1;
    h.key(egui::Key::E, Modifiers::COMMAND);
    h.frame(vec![]);
    assert!(in_front(&h), "303-2's editor is in front");
    assert_eq!(h.app.pattern_editor(1).pattern, 2);
    // with the focus on a drum machine the key opens its editor, not a 303's
    h.app.panel.focus = 3;
    h.key(egui::Key::E, Modifiers::COMMAND);
    assert_eq!(h.app.pattern_editor(0).synth, 0);
    assert_eq!(h.app.pattern_editor(1).synth, 1);
}

#[test]
fn the_top_row_and_the_window_that_does_not_resize() {
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
    ] {
        assert!(h.app.pattern_editor_hit(0, name).is_some(), "{name}");
    }
    // no 303 switch: each window is fixed to its 303 (task-290)
    for name in ["303-1", "303-2"] {
        assert!(h.app.pattern_editor_hit(0, name).is_none(), "{name}");
    }
    let w = h.app.pattern_editor_window(0).unwrap();
    // a drag at the bottom right corner does not resize it
    let corner = w.max - Vec2::splat(2.0);
    h.drag(&[corner, corner + Vec2::new(80.0, 60.0)]);
    let w2 = h.app.pattern_editor_window(0).unwrap();
    assert_eq!(w.size(), w2.size());
    // the close button closes it, and it gives up the focus
    h.app.command(Cmd::PatternEditor(0));
    h.frame(vec![]);
    assert_eq!(h.app.focused_editor(), Some(0));
    let close = Pos2::new(w2.right() - 14.0, w2.top() + 12.0);
    h.click(close);
    assert!(!h.app.pattern_editor(0).open, "closed");
    assert_eq!(h.app.focused_editor(), None);
}

#[test]
fn each_window_keeps_its_place_in_the_settings_file() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("OpenRB-338.toml");
    let mut h = Harness::new();
    h.app.settings = Settings::load_from(&file);
    h.open_editor();
    let w = h.app.pattern_editor_window(0).unwrap();
    assert!(!file.exists(), "opening writes nothing");
    // the title bar dragged 20 points left and 30 down (the window is nearly as wide as the
    // smallest main window, which holds it)
    let title = Pos2::new(w.left() + 60.0, w.top() + 10.0);
    h.drag(&[
        title,
        title + Vec2::new(-10.0, 15.0),
        title + Vec2::new(-20.0, 30.0),
    ]);
    let moved = h.app.pattern_editor_window(0).unwrap();
    assert_eq!(moved.min, w.min + Vec2::new(-20.0, 30.0));
    let saved = Settings::load_from(&file);
    let want = [moved.min.x.round() as i32, moved.min.y.round() as i32];
    assert_eq!(saved.pattern_editor_303_1_pos, Some(want));
    assert_eq!(saved.pattern_editor_303_2_pos, None, "303-2's is its own");
    let text = std::fs::read_to_string(&file).unwrap();
    assert!(text.contains("pattern_editor_303_1_pos"), "{text}");

    // 303-2's window: its own place
    h.app.command(Cmd::PatternEditor(1));
    for _ in 0..3 {
        h.frame(vec![]);
    }
    let w = h.app.pattern_editor_window(1).unwrap();
    let title = Pos2::new(w.left() + 60.0, w.top() + 10.0);
    h.drag(&[
        title,
        title + Vec2::new(5.0, 10.0),
        title + Vec2::new(10.0, 20.0),
    ]);
    let moved2 = h.app.pattern_editor_window(1).unwrap();
    let want2 = [moved2.min.x.round() as i32, moved2.min.y.round() as i32];
    let saved = Settings::load_from(&file);
    assert_eq!(saved.pattern_editor_303_2_pos, Some(want2));
    assert_eq!(saved.pattern_editor_303_1_pos, Some(want), "303-1's stays");

    // the next run opens each there
    let mut h2 = Harness::new();
    h2.app.settings = Settings::load_from(&file);
    h2.open_editor();
    let w2 = h2.app.pattern_editor_window(0).unwrap();
    assert_eq!(w2.min, Pos2::new(want[0] as f32, want[1] as f32));
    h2.k = 1;
    h2.open_editor();
    let w2 = h2.app.pattern_editor_window(1).unwrap();
    assert_eq!(w2.min, Pos2::new(want2[0] as f32, want2[1] as f32));
}

#[test]
fn clicks_make_notes_and_pauses_and_toggle_the_flags() {
    let mut h = Harness::new();
    h.open_editor();
    let empty = h.bytes();
    assert!(link::is_empty_pattern(0, &empty));
    let p = h.note_cell(3, 7);
    h.click(p);
    assert_eq!(note(&h.bytes(), 3), Some(7));
    // a note click is one change: Undo takes it back
    assert_eq!(h.app.undo_name(), Some("Edit Pattern"));
    h.undo();
    assert_eq!(h.bytes(), empty, "the note click undone");
    h.click(p);
    assert_eq!(note(&h.bytes(), 3), Some(7));
    // another pitch of the step moves the note
    let p = h.note_cell(3, 12);
    h.click(p);
    assert_eq!(note(&h.bytes(), 3), Some(12));
    // the lit cell makes a pause: not shown, its pitch kept; Undo brings the note back
    let lit = h.bytes();
    h.click(p);
    assert_eq!(note(&h.bytes(), 3), None);
    h.undo();
    assert_eq!(h.bytes(), lit, "the pause undone");
    h.click(p);
    let b = h.bytes();
    assert_eq!(note(&b, 3), None);
    assert_eq!(b[2 + 6 * 3], 12, "the pause keeps its pitch");
    // the panel's step views and the engine have it at once
    h.calls(1);
    assert_eq!(h.app.panel.synth[0].patterns[0][3].note, 12);
    assert!(h.app.panel.synth[0].patterns[0][3].rest);
    assert_eq!(engine_303(&mut h.audio, 0, 0), b);
    assert!(h.app.doc.modified);

    // Up and Down are independent; each flag toggles
    for f in Flag::ALL {
        let p = h.flag_cell(f, 5);
        h.click(p);
    }
    let b = h.bytes();
    assert!(
        Flag::ALL.iter().all(|&f| flag(&b, 5, f)),
        "{:?}",
        &b[32..38]
    );
    let all = b;
    let p = h.flag_cell(Flag::Up, 5);
    h.click(p);
    let b = h.bytes();
    assert!(!flag(&b, 5, Flag::Up) && flag(&b, 5, Flag::Down));
    // a flag click is one change: Undo takes it back
    assert_eq!(h.app.undo_name(), Some("Edit Pattern"));
    h.undo();
    assert_eq!(h.bytes(), all, "the flag click undone");
}

#[test]
fn drags_paint_the_piano_roll_along_the_mouse_and_a_flag_row_within_its_row() {
    let mut h = Harness::new();
    h.open_editor();
    // from step 0 at E to step 4 at G#, the mouse skipping steps between frames
    let path = [h.note_cell(0, 4), h.note_cell(2, 6), h.note_cell(4, 8)];
    h.drag(&path);
    let b = h.bytes();
    let notes: Vec<_> = (0..5).map(|s| note(&b, s)).collect();
    assert_eq!(notes, [Some(4), Some(5), Some(6), Some(7), Some(8)]);
    // the drag was one change
    assert_eq!(h.app.undo_name(), Some("Edit Pattern"));
    h.undo();
    assert!(
        link::is_empty_pattern(0, &h.bytes()),
        "one undo takes the whole drag back"
    );

    // a drag that starts on a lit cell paints notes; it makes no pause
    h.drag(&[h.note_cell(0, 4), h.note_cell(1, 4)]);
    h.drag(&[h.note_cell(0, 4), h.note_cell(1, 9)]);
    let b = h.bytes();
    assert_eq!((note(&b, 0), note(&b, 1)), (Some(4), Some(9)));

    // Accent from step 2 on: the mouse goes down into the Slide row, the drag stays in Accent
    let path = [
        h.flag_cell(Flag::Accent, 2),
        h.flag_cell(Flag::Slide, 4),
        h.flag_cell(Flag::Slide, 6),
    ];
    h.drag(&path);
    let b = h.bytes();
    let accents: Vec<_> = (0..8).map(|s| flag(&b, s, Flag::Accent)).collect();
    assert_eq!(accents, [false, false, true, true, true, true, true, false]);
    assert!((0..16).all(|s| !flag(&b, s, Flag::Slide)));
    // a drag from a lit cell paints the value the first cell took: off
    h.drag(&[h.flag_cell(Flag::Accent, 3), h.flag_cell(Flag::Accent, 5)]);
    let b = h.bytes();
    let accents: Vec<_> = (0..8).map(|s| flag(&b, s, Flag::Accent)).collect();
    assert_eq!(
        accents,
        [false, false, true, false, false, false, true, false]
    );
    h.undo();
    assert!((2..7).all(|s| flag(&h.bytes(), s, Flag::Accent)));
}

#[test]
fn steps_beyond_the_length_are_dimmed_and_editable() {
    let mut h = Harness::new();
    h.open_editor();
    assert!(!h.app.pattern_editor_shown(0).1, "16 steps: nothing dimmed");
    h.app.editor_act(0, Act::Length { n: 4, drag: false });
    h.frame(vec![]);
    assert_eq!(length(&h.bytes()), 4);
    assert!(h.app.pattern_editor_shown(0).1);
    let p = h.note_cell(10, 2);
    h.click(p);
    assert_eq!(note(&h.bytes(), 10), Some(2));
    // the panel's Steps display has the length (pattern mode: the selected pattern)
    assert_eq!(h.app.panel.rows[0].len[0], 4);
}

#[test]
fn shift_transpose_clear_copy_paste_and_their_undo() {
    let mut h = Harness::new();
    h.open_editor();
    // C, D, E on steps 0..2; length 4; shuffle on; Up on step 0
    for (s, n) in [(0, 0), (1, 2), (2, 4)] {
        let p = h.note_cell(s, n);
        h.click(p);
    }
    h.app.editor_act(0, Act::Length { n: 4, drag: false });
    h.frame(vec![]);
    h.press("Shuffle");
    assert_eq!(h.bytes()[0], 1);
    let p = h.flag_cell(Flag::Up, 0);
    h.click(p);
    let base = h.bytes();

    // Shift right wraps within the length 4: step 3 (a pause) goes to step 0
    h.press("shift_right");
    let b = h.bytes();
    let notes: Vec<_> = (0..4).map(|s| note(&b, s)).collect();
    assert_eq!(notes, [None, Some(0), Some(2), Some(4)]);
    assert!(flag(&b, 1, Flag::Up));
    assert_eq!(h.app.undo_name(), Some("Shift Pattern"));
    h.undo();
    assert_eq!(h.bytes(), base);
    h.press("shift_left");
    let b = h.bytes();
    assert_eq!(note(&b, 3), Some(0));
    h.undo();
    assert_eq!(h.bytes(), base);

    // Transpose down: C wraps to the high C, the Up flag stays
    h.press("transpose_down");
    let b = h.bytes();
    assert_eq!((note(&b, 0), note(&b, 1)), (Some(12), Some(1)));
    assert!(flag(&b, 0, Flag::Up));
    h.undo();
    assert_eq!(h.bytes(), base);
    for _ in 0..PITCHES {
        h.press("transpose_up");
    }
    assert_eq!(h.bytes(), base, "13 steps up come around");

    // Copy; Clear keeps the length, the shuffle and the clipboard
    h.press("Copy");
    let clip = h.app.clipboard.read().expect("copied");
    assert_eq!(orb_ui::pattern_json::parse(&clip).unwrap().bytes, base);
    h.press("Clear");
    let b = h.bytes();
    assert!(link::is_empty_pattern(0, &b));
    assert_eq!(&b[..2], &[1, 4]);
    assert_eq!(h.app.clipboard.read(), Some(clip));
    assert_eq!(h.app.undo_name(), Some("Clear Pattern"));
    h.undo();
    assert_eq!(h.bytes(), base);

    // Paste into 303-2's A2 in 303-2's window, through the Edit menu's clipboard; its Undo
    h.k = 1;
    h.open_editor();
    h.app.editor_act(1, Act::Select(1));
    h.frame(vec![]);
    assert_eq!(h.app.panel.rows[1].pattern, 1);
    h.press("Paste");
    assert_eq!(h.bytes(), base);
    h.calls(1);
    assert_eq!(engine_303(&mut h.audio, 1, 1), base);
    assert_eq!(h.app.undo_name(), Some("Paste Pattern"));
    h.undo();
    assert!(link::is_empty_pattern(1, &h.bytes()));
    // Edit > Paste Pattern takes the editor's copy too, into the focused editor's pattern
    assert_eq!(h.app.focused_editor(), Some(1));
    h.app.command(Cmd::PastePattern);
    assert_eq!(h.bytes(), base);
}

#[test]
fn a_shuffle_and_a_length_change_are_undone() {
    let mut h = Harness::new();
    h.open_editor();
    let base = h.bytes();
    h.press("Shuffle");
    assert_eq!(h.app.undo_name(), Some("Pattern Shuffle"));
    h.undo();
    assert_eq!(h.bytes(), base);
    // a drag of the Length value is one change
    h.app.editor_act(0, Act::Length { n: 12, drag: true });
    h.app.editor_act(0, Act::Length { n: 8, drag: true });
    h.app.editor_act(0, Act::Length { n: 5, drag: true });
    assert_eq!(length(&h.bytes()), 5);
    h.undo();
    assert_eq!(h.bytes(), base);
}

#[test]
fn a_panel_step_edit_forgets_the_editors_undo() {
    let mut h = Harness::new();
    h.open_editor();
    let p = h.note_cell(0, 3);
    h.click(p);
    assert!(h.app.undo_name().is_some());
    h.app.panel.press("tb303_1.accent", 0.0);
    h.frame(vec![]);
    assert_eq!(h.app.undo_name(), None);
}

#[test]
fn pattern_mode_selections_follow_each_other_both_ways() {
    let mut h = Harness::new();
    h.open_editor();
    assert!(!h.app.panel.transport.song_mode);
    // the editor's bank and number select on the panel
    h.press("C");
    assert_eq!(h.app.panel.rows[0].pattern, 16);
    assert_eq!(h.app.pattern_editor(0).pattern, 16);
    h.app.editor_act(0, Act::Select(19));
    h.calls(2);
    assert_eq!(h.app.panel.rows[0].pattern, 19);
    assert_eq!(h.app.panel.rows[0].bank, 2);
    // a selection of the other 303 does not move it (each window is fixed to its 303,
    // task-290); that 303's window follows it
    h.select(1, 4);
    h.frame(vec![]);
    let e = h.app.pattern_editor(0);
    assert_eq!((e.synth, e.pattern), (0, 19));
    h.k = 1;
    h.open_editor();
    let e = h.app.pattern_editor(1);
    assert_eq!((e.synth, e.pattern), (1, 4));
    h.select(1, 9);
    h.frame(vec![]);
    assert_eq!(
        h.app.pattern_editor(1).pattern,
        9,
        "a panel selection moves it"
    );
    h.k = 0;
    // a panel selection of its 303
    h.select(0, 2);
    h.frame(vec![]);
    assert_eq!(h.app.pattern_editor(0).pattern, 2);
    // the drum rows do not move it
    h.select(2, 7);
    h.frame(vec![]);
    let e = h.app.pattern_editor(0);
    assert_eq!((e.synth, e.pattern), (0, 2));
}

#[test]
fn song_mode_selections_are_independent_and_never_recorded() {
    let mut h = Harness::new();
    h.select(0, 3);
    h.open_editor();
    h.song_mode();
    // record on: still nothing of the editor is recorded
    h.app.panel.press("transport.record", 0.0);
    h.calls(2);
    let panel_pattern = h.app.panel.rows[0].pattern;
    let tracks = h.app.link.as_ref().unwrap().loaded_tracks().clone();
    h.press("B");
    h.app.editor_act(0, Act::Select(12));
    h.calls(2);
    assert_eq!(h.app.pattern_editor(0).pattern, 12);
    assert_eq!(
        h.app.panel.rows[0].pattern, panel_pattern,
        "the panel stays"
    );
    // an edit of the shown pattern is heard and changes no selection
    let p = h.note_cell(0, 5);
    h.click(p);
    h.calls(2);
    assert_eq!(note(&engine_303(&mut h.audio, 0, 12), 0), Some(5));
    assert_eq!(h.app.panel.rows[0].pattern, panel_pattern);
    assert_eq!(h.app.link.as_ref().unwrap().loaded_tracks(), &tracks);
    // a panel selection does not move the editor
    h.select(0, 6);
    h.calls(2);
    let e = h.app.pattern_editor(0);
    assert_eq!((e.synth, e.pattern), (0, 12));
}

#[test]
fn the_playing_step_is_shown_in_the_pattern_it_plays() {
    let mut h = Harness::new();
    h.open_editor();
    h.app.panel.press("transport.play", 0.0);
    let mut seen = None;
    for _ in 0..200 {
        h.calls(4);
        if let Some(s) = h.app.panel.playing_step303[0] {
            seen = Some(s);
            break;
        }
    }
    let s = seen.expect("the 303 plays");
    assert_eq!(h.app.pattern_editor_shown(0).0, Some(s));
    // stopped: none
    h.app.panel.press("transport.stop", 0.0);
    h.calls(4);
    h.calls(4);
    assert_eq!(h.app.pattern_editor_shown(0).0, None);
}

/// The user's review, round 1 (2026-11-21): the piano roll and the flag rows sit together with
/// only a divider between them; the Shift arrows are at the bottom, centered under the step
/// grid; the Transpose arrows are centered on the piano roll; the four arrows are one size; the
/// step columns fill the width up to the Transpose column.
#[test]
fn the_layout_of_the_users_first_review() {
    let mut h = Harness::new();
    h.open_editor();
    let first = h.app.pattern_editor_note_cell(0, 0, 12).unwrap();
    let last = h.app.pattern_editor_note_cell(0, 15, 0).unwrap();
    let grid = first.union(last);
    let flags = h
        .app
        .pattern_editor_flag_cell(0, Flag::Up, 0)
        .unwrap()
        .union(h.app.pattern_editor_flag_cell(0, Flag::Slide, 15).unwrap());
    let hit = |n: &str| h.app.pattern_editor_hit(0, n).unwrap();
    let (sl, sr) = (hit("shift_left"), hit("shift_right"));
    let (tu, td) = (hit("transpose_up"), hit("transpose_down"));
    // the flag rows right under the piano roll: a divider's gap, no buttons
    let gap = flags.top() - grid.bottom();
    assert!((4.0..=16.0).contains(&gap), "{gap}");
    for b in [sl, sr, tu, td] {
        assert!(
            !(b.top() < flags.top() && b.bottom() > grid.bottom()),
            "{b:?} between"
        );
    }
    // Shift: under the flag rows, centered under the grid
    assert!(sl.top() > flags.bottom() && sr.top() > flags.bottom());
    let mid = (sl.left() + sr.right()) / 2.0;
    assert!((mid - grid.center().x).abs() < 1.0, "{mid} {grid:?}");
    // Transpose: centered on the piano roll
    let mid = (tu.top() + td.bottom()) / 2.0;
    assert!((mid - grid.center().y).abs() < 1.0, "{mid} {grid:?}");
    // one size
    for b in [sr, tu, td] {
        assert_eq!(b.size(), sl.size());
    }
    // the columns fill the width: the Transpose column follows the grid and ends at the
    // window's inner edge
    let w = h.app.pattern_editor_window(0).unwrap();
    assert!(tu.left() - grid.right() < 8.0, "{tu:?} {grid:?}");
    assert!(w.right() - tu.right() < 12.0, "{w:?} {tu:?}");
    assert!(first.width() > 22.0, "{first:?}");
    // round 2: the larger labels fit their rows and columns, and the window fits the smallest
    // main window (624 x 510 points of panel under the menu bar)
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(PANEL_W, PANEL_H + MENU_H));
    assert!(screen.contains_rect(w), "{w:?}");
    let font = egui::FontId::proportional(15.5);
    for label in ["Accent", "Slide", "Down", "C#", "G#"] {
        let g = h
            .ctx
            .fonts_mut(|f| f.layout_no_wrap(label.into(), font.clone(), egui::Color32::WHITE));
        let column = grid.left() - w.left() - 8.0;
        assert!(
            g.size().x + 6.0 < column,
            "{label}: {:?} in {column}",
            g.size()
        );
        assert!(
            g.size().y <= first.height() + 1.0,
            "{label}: {:?}",
            g.size()
        );
    }
}

/// The pattern number's arrows step through the window's own 303, A1..D8 (each window is fixed
/// to its 303, task-290; before, one window ran 303-1 A1..D8 then 303-2 A1..D8); 8 goes on to
/// the next bank's 1, and left the reverse; they stop at A1 and D8, where the arrow is disabled
/// (a click does nothing). The banks follow, and in pattern mode the panel selects it.
#[test]
fn the_number_arrows_cross_banks_and_stop_at_the_ends_of_the_303() {
    for k in 0..2 {
        let mut h = Harness::new();
        h.k = k;
        h.open_editor();
        let at = |h: &Harness| {
            let e = h.app.pattern_editor(k);
            (e.synth, e.pattern)
        };
        // A1: left is the end
        h.press("number_prev");
        assert_eq!(at(&h), (k, 0), "A1 stops");
        // A8 on to B1, and back
        h.app.editor_act(k, Act::Select(7));
        h.frame(vec![]);
        h.press("number_next");
        assert_eq!(at(&h), (k, 8), "A8 to B1");
        h.calls(2);
        assert_eq!(h.app.panel.rows[k].pattern, 8);
        assert_eq!(h.app.panel.rows[k].bank, 1);
        h.press("number_prev");
        assert_eq!(at(&h), (k, 7), "B1 back to A8");
        // D8: right is the end, on the same 303
        h.app.editor_act(k, Act::Select(31));
        h.frame(vec![]);
        h.press("number_next");
        assert_eq!(at(&h), (k, 31), "D8 stops");
        h.calls(2);
        assert_eq!(h.app.panel.rows[1 - k].pattern, 0, "the other 303 stays");
    }
}

/// A pattern of row `r` that no other test writes: every step a note or an instrument, length
/// 7, shuffle on.
fn full(r: usize) -> Vec<u8> {
    let n = if r < 2 { 98 } else { 194 };
    let mut b = vec![1u8; n];
    b[1] = 7;
    if r < 2 {
        // the 303's steps: note 3, no slide, accent, no up or down, no rest
        for s in 0..16 {
            b[2 + 6 * s..2 + 6 * s + 6].copy_from_slice(&[3, 0, 1, 0, 0, 0]);
        }
    }
    b
}

impl Harness {
    /// Writes `bytes` into pattern `p` of row `r`, as the song held it.
    fn put(&mut self, r: usize, p: u8, bytes: &[u8]) {
        let link = self.app.link.as_mut().unwrap();
        link.set_pattern(r, p, bytes, &mut self.app.panel);
        self.frame(vec![]);
    }

    fn pattern(&self, r: usize, p: u8) -> Vec<u8> {
        self.app.link.as_ref().unwrap().pattern(r, p).to_vec()
    }

    /// Opens the pattern editor of row `r` (the Edit menu's command) and draws it.
    fn open_row_editor(&mut self, r: usize) {
        match r {
            0 | 1 => self.app.command(Cmd::PatternEditor(r)),
            2 => self.app.command(Cmd::DrumEditor(Machine::Tr808)),
            _ => self.app.command(Cmd::DrumEditor(Machine::Tr909)),
        }
        for _ in 0..3 {
            self.frame(vec![]);
        }
    }

    /// Where the editor window of row `r` was drawn last.
    fn row_window(&self, r: usize) -> Rect {
        match r {
            0 | 1 => self.app.pattern_editor_window(r),
            2 => self.app.drum_editor_window(Machine::Tr808),
            _ => self.app.drum_editor_window(Machine::Tr909),
        }
        .expect("the editor is open")
    }

    /// A click on row `q`'s focus area of the panel, where no window covers it.
    fn click_panel_row(&mut self, q: usize) {
        let l = self.app.skin().layout().clone();
        let area = l.rows.iter().find(|c| c.row() == Some(q)).unwrap();
        let windows: Vec<Rect> = (0..4)
            .filter_map(|r| match r {
                0 | 1 => self.app.pattern_editor_window(r),
                2 => self.app.drum_editor_window(Machine::Tr808),
                _ => self.app.drum_editor_window(Machine::Tr909),
            })
            .collect();
        // the panel is drawn 1:1 under the menu bar; a point on no control
        let on_control = |x: f32, y: f32| {
            l.controls
                .iter()
                .chain(&l.strips)
                .chain(&l.selects)
                .any(|c| {
                    x >= c.x - 2.0 && x <= c.x + c.w + 2.0 && y >= c.y - 2.0 && y <= c.y + c.h + 2.0
                })
        };
        let p = (0..area.w as i32)
            .flat_map(|dx| (0..area.h as i32).map(move |dy| (dx, dy)))
            .map(|(dx, dy)| (area.x + dx as f32, area.y + dy as f32))
            .filter(|&(x, y)| !on_control(x, y))
            .map(|(x, y)| Pos2::new(x, y + MENU_H))
            .find(|p| windows.iter().all(|w| !w.expand(2.0).contains(*p)))
            .expect("a free point on the row");
        self.click(p);
    }
}

/// Edit > Copy, Cut, Paste, Duplicate and Clear Pattern act on the focused pattern editor's
/// pattern, else on the panel's focused row's selected pattern (task-290), for each 303, the
/// 808 and the 909: an editor takes the focus as it opens and on a click into its window, and
/// loses it to a click on the panel.
#[test]
fn the_edit_menus_pattern_commands_follow_the_focused_editor() {
    for r in 0..4 {
        // another device's row with its own pattern
        let q = if r < 2 { 2 } else { 0 };
        let mut h = Harness::new();
        let mine = full(r);
        let other = full(q);
        h.put(r, 5, &mine);
        h.put(q, 0, &other);
        h.select(r, 5);
        h.open_row_editor(r);
        assert_eq!(h.app.focused_editor(), Some(r), "row {r}: opened, focused");
        // the panel's focus goes to another row by its button (not a click on the panel)
        h.select(q, 0);
        assert_eq!(h.app.panel.focus, q);
        assert_eq!(h.app.focused_editor(), Some(r));

        // Copy: the editor's pattern
        h.app.command(Cmd::CopyPattern);
        let clip = h.app.clipboard.read().unwrap();
        let got = orb_ui::pattern_json::parse(&clip).unwrap();
        assert_eq!(got.bytes, mine, "row {r}: the editor's pattern copied");
        // Clear: the editor's pattern, shuffle and length kept, one Undo
        h.app.command(Cmd::ClearPattern);
        let b = h.pattern(r, 5);
        assert!(link::is_empty_pattern(r, &b), "row {r}: cleared");
        assert_eq!(&b[..2], &mine[..2], "row {r}: shuffle and length stay");
        assert_eq!(h.pattern(q, 0), other, "row {r}: the panel's row untouched");
        assert_eq!(h.app.undo_name(), Some("Clear Pattern"));
        h.undo();
        assert_eq!(h.pattern(r, 5), mine);
        // Cut and its Undo
        h.app.command(Cmd::CutPattern);
        assert!(link::is_empty_pattern(r, &h.pattern(r, 5)));
        assert_eq!(h.pattern(q, 0), other);
        h.undo();
        assert_eq!(h.pattern(r, 5), mine);
        // Duplicate: into the editor's row's next empty pattern, which the editor shows
        h.app.command(Cmd::DuplicatePattern);
        assert_eq!(h.pattern(r, 6), mine, "row {r}: duplicated to A7");
        assert_eq!(h.app.panel.rows[r].pattern, 6);
        h.frame(vec![]);
        h.undo();
        h.frame(vec![]);
        assert!(link::is_empty_pattern(r, &h.pattern(r, 6)));
        assert_eq!(h.app.panel.rows[r].pattern, 5);
        // Paste: into the editor's pattern, the device checked against the editor's
        h.app
            .clipboard
            .copied(&orb_ui::pattern_json::to_json(q, &other));
        h.app.command(Cmd::PastePattern);
        assert_eq!(
            h.pattern(r, 5),
            mine,
            "row {r}: another device's pattern refused"
        );
        let status = h.app.status.clone().unwrap();
        assert!(status.contains("holds a"), "{status}");
        let empty = link::emptied_pattern(r, &mine);
        h.app
            .clipboard
            .copied(&orb_ui::pattern_json::to_json(r, &empty));
        h.app.command(Cmd::PastePattern);
        assert_eq!(
            h.pattern(r, 5),
            empty,
            "row {r}: pasted into the editor's pattern"
        );
        assert_eq!(h.pattern(q, 0), other);
        h.undo();
        assert_eq!(h.pattern(r, 5), mine);

        // a click on the panel takes the focus: the panel's focused row again
        h.click_panel_row(q);
        assert_eq!(
            h.app.focused_editor(),
            None,
            "row {r}: the panel has the focus"
        );
        assert_eq!(h.app.panel.focus, q);
        h.app.command(Cmd::CopyPattern);
        let got = orb_ui::pattern_json::parse(&h.app.clipboard.read().unwrap()).unwrap();
        assert_eq!(got.bytes, other, "row {r}: the panel's pattern copied");
        h.app.command(Cmd::ClearPattern);
        assert!(link::is_empty_pattern(q, &h.pattern(q, 0)));
        assert_eq!(h.pattern(r, 5), mine);
        h.undo();
        assert_eq!(h.pattern(q, 0), other);

        // a click into the editor's window gives it the focus back
        let w = h.row_window(r);
        h.click(Pos2::new(w.left() + 40.0, w.top() + 10.0));
        assert_eq!(h.app.focused_editor(), Some(r), "row {r}: clicked into");
        h.app.command(Cmd::CopyPattern);
        let got = orb_ui::pattern_json::parse(&h.app.clipboard.read().unwrap()).unwrap();
        assert_eq!(got.bytes, mine);
    }
}

/// With both 303 windows open, the one clicked last is the target; the menu bar's clicks do not
/// take the focus.
#[test]
fn of_two_303_windows_the_last_clicked_is_the_target() {
    let mut h = Harness::new();
    h.put(0, 1, &full(0));
    let mut two = full(1);
    two[2] = 9;
    h.put(1, 2, &two);
    h.select(0, 1);
    h.select(1, 2);
    h.open_row_editor(0);
    h.open_row_editor(1);
    assert_eq!(h.app.focused_editor(), Some(1), "the last opened");
    // the windows open centered: 303-1's is moved aside, so a click on it reaches it
    let w = h.row_window(1);
    let title = Pos2::new(w.left() + 60.0, w.top() + 10.0);
    h.drag(&[title, title + Vec2::new(0.0, 60.0)]);
    let w0 = h.row_window(0);
    let free = Pos2::new(w0.left() + 40.0, w0.top() + 10.0);
    assert!(!h.row_window(1).contains(free));
    h.click(free);
    assert_eq!(h.app.focused_editor(), Some(0));
    // through the Edit menu: Clear Pattern on 303-1's A2
    let mut found = None;
    for x in (6..300).step_by(4) {
        egui::Popup::close_all(&h.ctx);
        h.click(Pos2::new(x as f32, MENU_H / 2.0));
        h.frame(vec![]);
        if let Some((r, on)) = h.app.edit_item("Clear Pattern") {
            found = Some((r, on));
            break;
        }
    }
    let (r, enabled) = found.expect("the Edit menu has Clear Pattern");
    assert!(enabled);
    assert_eq!(
        h.app.focused_editor(),
        Some(0),
        "the menu bar keeps the focus"
    );
    h.click(r.center());
    h.frame(vec![]);
    assert!(link::is_empty_pattern(0, &h.pattern(0, 1)));
    assert_eq!(h.pattern(1, 2), two, "303-2's untouched");
    assert_eq!(h.app.status.as_deref(), Some("303 1: A2 cleared"));
    // an empty pattern cleared again still says so (the user's review, round 1), from the menu
    // and from the editor's Clear button
    h.app.status = None;
    h.app.command(Cmd::ClearPattern);
    assert_eq!(h.app.status.as_deref(), Some("303 1: A2 cleared"));
    h.app.status = None;
    h.k = 0;
    h.press("Clear");
    assert_eq!(h.app.status.as_deref(), Some("303 1: A2 cleared"));
    assert!(link::is_empty_pattern(0, &h.pattern(0, 1)));
}
