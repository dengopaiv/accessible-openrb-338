//! The 303 pattern editor (task-274, docs/ui_spec.md section 23), headless: the Edit menu item
//! and Ctrl/Cmd+E open it, the grids edit the pattern by clicks and drags, Shift, Transpose,
//! Clear, Copy and Paste, every change is one Undo and goes through the panel's pattern path
//! (heard, saved, shown), and the selection rules of pattern and song mode.

use std::sync::{Arc, OnceLock};

use eframe::egui::{self, Event, Modifiers, PointerButton, Pos2, RawInput, Rect, Vec2};
use orb_engine::tables::Tables;
use orb_player::{Audio, CALL_FRAMES};
use orb_ui::app::App;
use orb_ui::layout::{PANEL_H, PANEL_W};
use orb_ui::link::{self, Link};
use orb_ui::menu::{Cmd, MENU_H};
use orb_ui::model::Panel;
use orb_ui::pattern_editor::{Act, Flag, PITCHES, flag, length, note};
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
        self.app.command(Cmd::PatternEditor);
        self.frame(vec![]);
        self.frame(vec![]);
        self.frame(vec![]);
        assert!(self.app.pattern_editor().open);
    }

    /// The editor's pattern as the link holds it.
    fn bytes(&self) -> Vec<u8> {
        let e = self.app.pattern_editor();
        self.app
            .link
            .as_ref()
            .unwrap()
            .pattern(e.synth, e.pattern)
            .to_vec()
    }

    fn note_cell(&self, step: usize, pitch: u8) -> Pos2 {
        self.app
            .pattern_editor_note_cell(step, pitch)
            .expect("the piano roll is drawn")
            .center()
    }

    fn flag_cell(&self, f: Flag, step: usize) -> Pos2 {
        self.app
            .pattern_editor_flag_cell(f, step)
            .expect("the flag rows are drawn")
            .center()
    }

    fn hit(&self, name: &str) -> Pos2 {
        self.app
            .pattern_editor_hit(name)
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
fn the_menu_item_and_cmd_e_open_it_on_the_selected_303s_pattern_and_bring_it_to_the_front() {
    let mut h = Harness::new();
    // TB-303 2, A3: the focused row's selected pattern
    h.select(1, 2);
    assert_eq!(h.app.panel.focus, 1);
    // the Edit menu: found along the bar, its item clicked
    let mut found = None;
    for x in (6..300).step_by(4) {
        egui::Popup::close_all(&h.ctx);
        h.click(Pos2::new(x as f32, MENU_H / 2.0));
        h.frame(vec![]);
        if let Some((r, on)) = h.app.edit_item("303 Pattern Editor...") {
            found = Some((r, on));
            break;
        }
    }
    let (r, enabled) = found.expect("the Edit menu has the item");
    assert!(enabled);
    h.click(r.center());
    h.frame(vec![]);
    let e = h.app.pattern_editor();
    assert!(e.open);
    assert_eq!((e.synth, e.pattern), (1, 2));

    // File > Info opens over it; Cmd+E brings the editor back to the front, on TB-303 1's
    // selected pattern when that row has the focus
    h.app.command(Cmd::Info);
    h.frame(vec![]);
    h.frame(vec![]);
    let p = h.note_cell(8, 6);
    let editor_layer = |h: &Harness| {
        h.ctx.layer_id_at(p).map(|l| l.id) == Some(egui::Id::unique("orb_pattern_editor"))
    };
    assert!(!editor_layer(&h), "the info window covers the editor");
    h.app.panel.rows[0].pattern = 5;
    h.app.panel.focus = 0;
    h.key(egui::Key::E, Modifiers::COMMAND);
    h.frame(vec![]);
    let e = h.app.pattern_editor();
    assert_eq!((e.synth, e.pattern), (0, 5));
    assert!(editor_layer(&h), "the editor is in front");
    // with the focus on a drum machine it stays on its 303
    h.app.panel.focus = 3;
    h.key(egui::Key::E, Modifiers::COMMAND);
    assert_eq!(h.app.pattern_editor().synth, 0);
}

#[test]
fn the_top_row_and_the_window_that_does_not_resize() {
    let mut h = Harness::new();
    h.open_editor();
    for name in [
        "303-1",
        "303-2",
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
        assert!(h.app.pattern_editor_hit(name).is_some(), "{name}");
    }
    let w = h.app.pattern_editor_window().unwrap();
    // a drag at the bottom right corner does not resize it
    let corner = w.max - Vec2::splat(2.0);
    h.drag(&[corner, corner + Vec2::new(80.0, 60.0)]);
    let w2 = h.app.pattern_editor_window().unwrap();
    assert_eq!(w.size(), w2.size());
    // the close button closes it
    h.app.command(Cmd::PatternEditor);
    h.frame(vec![]);
    let close = Pos2::new(w2.right() - 14.0, w2.top() + 12.0);
    h.click(close);
    assert!(!h.app.pattern_editor().open, "closed");
}

#[test]
fn its_place_is_kept_in_the_settings_file() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("ui.toml");
    let mut h = Harness::new();
    h.app.settings = Settings::load_from(&file);
    h.open_editor();
    let w = h.app.pattern_editor_window().unwrap();
    assert!(!file.exists(), "opening writes nothing");
    // the title bar dragged 20 points left and 30 down (the window is nearly as wide as the
    // smallest main window, which holds it)
    let title = Pos2::new(w.left() + 60.0, w.top() + 10.0);
    h.drag(&[
        title,
        title + Vec2::new(-10.0, 15.0),
        title + Vec2::new(-20.0, 30.0),
    ]);
    let moved = h.app.pattern_editor_window().unwrap();
    assert_eq!(moved.min, w.min + Vec2::new(-20.0, 30.0));
    let saved = Settings::load_from(&file);
    let want = [moved.min.x.round() as i32, moved.min.y.round() as i32];
    assert_eq!(saved.pattern_editor_pos, Some(want));
    assert!(
        std::fs::read_to_string(&file)
            .unwrap()
            .contains("pattern_editor_pos")
    );

    // the next run opens it there
    let mut h2 = Harness::new();
    h2.app.settings = Settings::load_from(&file);
    h2.open_editor();
    let w2 = h2.app.pattern_editor_window().unwrap();
    assert_eq!(w2.min, Pos2::new(want[0] as f32, want[1] as f32));
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
    assert!(!h.app.pattern_editor_shown().1, "16 steps: nothing dimmed");
    h.app.editor_act(Act::Length { n: 4, drag: false });
    h.frame(vec![]);
    assert_eq!(length(&h.bytes()), 4);
    assert!(h.app.pattern_editor_shown().1);
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
    h.app.editor_act(Act::Length { n: 4, drag: false });
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

    // Paste into TB-303 2's A2, through the Edit menu's clipboard; its Undo
    h.press("303-2");
    h.app.editor_act(Act::Select {
        synth: 1,
        pattern: 1,
    });
    h.frame(vec![]);
    assert_eq!(h.app.panel.rows[1].pattern, 1);
    h.press("Paste");
    assert_eq!(h.bytes(), base);
    h.calls(1);
    assert_eq!(engine_303(&mut h.audio, 1, 1), base);
    assert_eq!(h.app.undo_name(), Some("Paste Pattern"));
    h.undo();
    assert!(link::is_empty_pattern(1, &h.bytes()));
    // the panel's own Edit > Paste Pattern takes the editor's copy too
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
    h.app.editor_act(Act::Length { n: 12, drag: true });
    h.app.editor_act(Act::Length { n: 8, drag: true });
    h.app.editor_act(Act::Length { n: 5, drag: true });
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
    assert_eq!(h.app.pattern_editor().pattern, 16);
    h.app.editor_act(Act::Select {
        synth: 0,
        pattern: 19,
    });
    h.calls(2);
    assert_eq!(h.app.panel.rows[0].pattern, 19);
    assert_eq!(h.app.panel.rows[0].bank, 2);
    // the 303 switch shows the other 303's selected pattern
    h.select(1, 4);
    h.frame(vec![]);
    let e = h.app.pattern_editor();
    assert_eq!(
        (e.synth, e.pattern),
        (1, 4),
        "a panel selection moves the editor"
    );
    h.press("303-1");
    let e = h.app.pattern_editor();
    assert_eq!((e.synth, e.pattern), (0, 19));
    // a panel selection of the shown 303
    h.select(0, 2);
    h.frame(vec![]);
    assert_eq!(h.app.pattern_editor().pattern, 2);
    // the drum rows do not move it
    h.select(2, 7);
    h.frame(vec![]);
    let e = h.app.pattern_editor();
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
    h.app.editor_act(Act::Select {
        synth: 0,
        pattern: 12,
    });
    h.calls(2);
    assert_eq!(h.app.pattern_editor().pattern, 12);
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
    // the 303 switch keeps the pattern number in song mode
    h.press("303-2");
    let e = h.app.pattern_editor();
    assert_eq!((e.synth, e.pattern), (1, 12));
    // a panel selection does not move the editor
    h.select(1, 6);
    h.calls(2);
    let e = h.app.pattern_editor();
    assert_eq!((e.synth, e.pattern), (1, 12));
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
    assert_eq!(h.app.pattern_editor_shown().0, Some(s));
    // stopped: none
    h.app.panel.press("transport.stop", 0.0);
    h.calls(4);
    h.calls(4);
    assert_eq!(h.app.pattern_editor_shown().0, None);
}

/// The user's review, round 1 (2026-11-21): the piano roll and the flag rows sit together with
/// only a divider between them; the Shift arrows are at the bottom, centered under the step
/// grid; the Transpose arrows are centered on the piano roll; the four arrows are one size; the
/// step columns fill the width up to the Transpose column.
#[test]
fn the_layout_of_the_users_first_review() {
    let mut h = Harness::new();
    h.open_editor();
    let first = h.app.pattern_editor_note_cell(0, 12).unwrap();
    let last = h.app.pattern_editor_note_cell(15, 0).unwrap();
    let grid = first.union(last);
    let flags = h
        .app
        .pattern_editor_flag_cell(Flag::Up, 0)
        .unwrap()
        .union(h.app.pattern_editor_flag_cell(Flag::Slide, 15).unwrap());
    let hit = |n: &str| h.app.pattern_editor_hit(n).unwrap();
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
    let w = h.app.pattern_editor_window().unwrap();
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

/// The user's review, round 2: the pattern number's arrows step through one sequence, 303-1
/// A1..D8 then 303-2 A1..D8; 8 goes on to the next bank's 1, 303-1 D8 to 303-2 A1, and left the
/// reverse; they stop at 303-1 A1 and 303-2 D8, where the arrow is disabled (a click does
/// nothing). The 303 switch and the banks follow, and in pattern mode the panel selects it.
#[test]
fn the_number_arrows_cross_banks_and_303s_and_stop_at_the_ends() {
    let mut h = Harness::new();
    h.open_editor();
    let at = |h: &Harness| {
        let e = h.app.pattern_editor();
        (e.synth, e.pattern)
    };
    // 303-1 A1: left is the end
    h.press("number_prev");
    assert_eq!(at(&h), (0, 0), "303-1 A1 stops");
    // A8 on to B1, and back
    h.app.editor_act(Act::Select {
        synth: 0,
        pattern: 7,
    });
    h.frame(vec![]);
    h.press("number_next");
    assert_eq!(at(&h), (0, 8), "A8 to B1");
    h.calls(2);
    assert_eq!(h.app.panel.rows[0].pattern, 8);
    assert_eq!(h.app.panel.rows[0].bank, 1);
    h.press("number_prev");
    assert_eq!(at(&h), (0, 7), "B1 back to A8");
    // 303-1 D8 on to 303-2 A1: the 303 switch follows, the panel selects 303-2's A1
    h.app.editor_act(Act::Select {
        synth: 0,
        pattern: 31,
    });
    h.select(1, 5);
    h.press("303-1");
    assert_eq!(at(&h), (0, 31));
    h.press("number_next");
    assert_eq!(at(&h), (1, 0), "303-1 D8 to 303-2 A1");
    h.calls(2);
    assert_eq!(h.app.panel.rows[1].pattern, 0);
    assert_eq!(h.app.panel.rows[0].pattern, 31);
    h.calls(2);
    assert_eq!(at(&h), (1, 0), "the editor stays on 303-2");
    h.press("number_prev");
    assert_eq!(at(&h), (0, 31), "303-2 A1 back to 303-1 D8");
    // 303-2 D8: right is the end
    h.app.editor_act(Act::Select {
        synth: 1,
        pattern: 31,
    });
    h.frame(vec![]);
    h.press("number_next");
    assert_eq!(at(&h), (1, 31), "303-2 D8 stops");
}
