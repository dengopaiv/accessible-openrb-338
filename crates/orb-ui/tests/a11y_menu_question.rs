//! A question opened from the menu waits for a new key press (task-249): the Enter that chose
//! File > New or Edit > Initialize Song from Pattern Mode must not answer the question it
//! opens (it would Save, which on an untitled song opens Save As, or Initialize). Walked from
//! the keyboard as the user does (F10, the arrows, Enter), in both modes.

#[path = "common/a11y.rs"]
mod a11y;

use std::path::{Path, PathBuf};
use std::sync::Arc;

use a11y::{AppTree, Frames};
use eframe::egui::{Event, Key, Modifiers, Vec2};
use orb_engine::tables::Tables;
use orb_player::{Audio, CALL_FRAMES};
use orb_ui::app::App;
use orb_ui::layout::{PANEL_H, PANEL_W};
use orb_ui::link::Link;
use orb_ui::menu::{Cmd, MENU_H};
use orb_ui::model::Panel;
use orb_ui::screen_reader::ScreenReaderSetting;

struct H {
    t: AppTree,
    audio: Audio,
    _tmp: tempfile::TempDir,
}

fn songs() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/songs/edit_init")
}

impl H {
    /// An app with a song open in song mode (so that Initialize Song is enabled), in screen
    /// reader mode `sr`.
    fn new(sr: bool) -> Self {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("init_loop.rbs");
        std::fs::copy(songs().join("init_loop.rbs"), &path).unwrap();
        let samples = rbm::standard::builtin_samples();
        let (link, audio) = Link::new(Arc::new(Tables::new()), Some(samples.clone()));
        let mut app = App::with_link(Panel::new(), link, None);
        app.samples = Some(samples);
        app.menu = true;
        app.settings.screen_reader = if sr {
            ScreenReaderSetting::On
        } else {
            ScreenReaderSetting::Off
        };
        app.open_song(&path);
        assert!(app.errors.is_empty(), "{:?}", app.errors);
        let mut h = Self {
            t: AppTree {
                frames: Frames::new(Vec2::new(PANEL_W, PANEL_H + MENU_H)),
                app,
            },
            audio,
            _tmp: tmp,
        };
        h.calls(4);
        h
    }

    fn calls(&mut self, n: usize) {
        let mut buf = vec![0i16; 2 * CALL_FRAMES];
        for _ in 0..n {
            self.audio.render_i16(&mut buf);
            self.t.frame(vec![]);
        }
    }

    fn focus(&self) -> String {
        let n = self.t.tree().focused().expect("a focused node");
        n.label().unwrap_or("").to_string()
    }

    /// F10, then Right `titles_right` times, then Down opens the menu, Down until the item
    /// whose name starts with `item` has the focus. Does not press Enter.
    fn walk(&mut self, titles_right: usize, item: &str) {
        self.t.key(Key::F10);
        for _ in 0..titles_right {
            self.t.key(Key::ArrowRight);
        }
        self.t.key(Key::ArrowDown);
        self.t.frame(vec![]);
        for _ in 0..30 {
            if self.focus().starts_with(item) {
                return;
            }
            self.t.key(Key::ArrowDown);
        }
        panic!("no item {item}: at {:?}", self.focus());
    }
}

fn for_both(f: impl Fn(&mut H, bool)) {
    for sr in [false, true] {
        let mut h = H::new(sr);
        f(&mut h, sr);
    }
}

#[test]
fn file_new_from_the_menu_asks_and_waits_for_a_new_key() {
    for_both(|h, sr| {
        h.t.app.doc.modified = true;
        h.walk(0, "New");
        h.t.key(Key::Enter);
        h.calls(3);
        let ctx = format!("screen reader mode {sr}");
        assert!(h.t.app.prompting(), "the question is open ({ctx})");
        assert!(
            !h.t.app.browsing(),
            "no Save As opened by the Enter ({ctx})"
        );
        assert!(h.t.app.doc.modified, "nothing saved or discarded ({ctx})");
        h.t.key(Key::D);
        h.calls(2);
        assert!(!h.t.app.prompting(), "D answered ({ctx})");
        assert!(
            !h.t.app.doc.modified,
            "Don't Save made the new song ({ctx})"
        );
    });
}

#[test]
fn file_new_from_the_menu_then_enter_saves_as_before() {
    for_both(|h, sr| {
        h.t.app.doc.modified = true;
        h.walk(0, "New");
        h.t.key(Key::Enter);
        h.calls(2);
        assert!(h.t.app.prompting(), "open ({sr})");
        h.t.key(Key::Enter);
        h.calls(2);
        assert!(!h.t.app.prompting(), "a second Enter answers Save ({sr})");
    });
}

#[test]
fn file_new_from_the_menu_escape_cancels() {
    for_both(|h, sr| {
        h.t.app.doc.modified = true;
        h.walk(0, "New");
        h.t.key(Key::Enter);
        h.calls(2);
        assert!(h.t.app.prompting());
        h.t.key(Key::Escape);
        h.calls(2);
        assert!(!h.t.app.prompting(), "Escape cancels ({sr})");
        assert!(h.t.app.doc.modified, "the song is as it was ({sr})");
    });
}

#[test]
fn initialize_song_from_the_menu_asks_and_waits_for_a_new_key() {
    for_both(|h, sr| {
        assert!(h.t.app.can_init(), "song mode ({sr})");
        h.walk(1, "Initialize Song");
        h.t.key(Key::Enter);
        h.calls(3);
        assert!(h.t.app.confirming_init(), "the question is open ({sr})");
        assert!(
            !h.t.app.doc.modified,
            "nothing initialized by the Enter ({sr})"
        );
        h.t.key(Key::Enter);
        h.calls(3);
        assert!(!h.t.app.confirming_init(), "a new Enter answers ({sr})");
        assert!(h.t.app.doc.modified, "Initialize ran ({sr})");
    });
}

/// File > Open... in the in-app browser (mode off): Enter in the path bar chooses the file,
/// and that Enter does not answer the unsaved-changes question the choice opens.
#[test]
fn open_from_the_browser_asks_and_waits_for_a_new_key() {
    let mut h = H::new(false);
    h.t.app.doc.modified = true;
    let file = songs().join("init_zero.rbs");
    h.walk(0, "Open");
    h.t.key(Key::Enter);
    h.calls(2);
    assert!(h.t.app.browsing(), "the browser is open");
    h.t.key_with(Key::L, Modifiers::COMMAND);
    h.t.key_with(Key::A, Modifiers::COMMAND);
    h.t.frame(vec![Event::Text(file.display().to_string())]);
    h.t.key(Key::Enter);
    h.calls(3);
    assert!(h.t.app.prompting(), "the question is open");
    assert!(h.t.app.doc.modified, "nothing saved or opened by the Enter");
    h.t.key(Key::Escape);
    h.calls(2);
    assert!(!h.t.app.prompting(), "Escape cancels");
    assert!(h.t.app.doc.modified);
}

#[test]
fn file_new_from_the_menu_then_s_saves() {
    for_both(|h, sr| {
        h.t.app.doc.modified = true;
        h.walk(0, "New");
        h.t.key(Key::Enter);
        h.calls(2);
        assert!(h.t.app.prompting(), "open ({sr})");
        h.t.key(Key::S);
        h.calls(2);
        assert!(!h.t.app.prompting(), "S answered Save ({sr})");
        assert!(
            !h.t.app.browsing(),
            "the song has a file: no Save As ({sr})"
        );
        assert!(!h.t.app.doc.modified, "saved, then the new song ({sr})");
    });
}

/// Mods > Demo Song and the window's close open the unsaved-changes question through the
/// calls the menu item and the close request make; an Enter in the frame that first draws
/// it does not answer it.
#[test]
fn demo_song_and_close_wait_for_a_new_key() {
    for_both(|h, sr| {
        h.t.app.doc.modified = true;
        h.t.app.doc.kit = orb_cli::mods::Kit::Mod(PathBuf::from("no-such-mod.rbm"));
        h.t.app.command(Cmd::DemoSong);
        assert!(h.t.app.prompting(), "Demo Song asks ({sr})");
        h.t.key_down(Key::Enter);
        assert!(
            h.t.app.prompting(),
            "the opening Enter did not answer ({sr})"
        );
        h.t.key(Key::Escape);
        h.calls(2);
        assert!(!h.t.app.prompting());
        assert!(!h.t.app.request_close(), "close asks ({sr})");
        h.t.key_down(Key::Enter);
        assert!(
            h.t.app.prompting(),
            "the opening Enter did not answer ({sr})"
        );
        assert!(!h.t.app.close_allowed());
        h.t.key(Key::Escape);
        h.calls(2);
        assert!(!h.t.app.prompting());
        assert!(h.t.app.doc.modified);
    });
}

/// File > Open... in screen reader mode picks with the system dialog (here a stand-in); the
/// Enter that chose the item is in the frame that opens the question.
#[test]
fn open_with_the_system_dialog_asks_and_waits_for_a_new_key() {
    let mut h = H::new(true);
    h.t.app.doc.modified = true;
    let file = songs().join("init_zero.rbs");
    h.t.app
        .set_picker(Box::new(move |_, _, _| Some(file.clone())));
    h.walk(0, "Open");
    h.t.key(Key::Enter);
    h.calls(3);
    assert!(h.t.app.prompting(), "the question is open");
    assert!(h.t.app.doc.modified, "nothing saved or opened by the Enter");
}

/// A question answered in one frame and another opened in the next (a new Enter chose it)
/// is not answered by that Enter either.
#[test]
fn a_question_opened_right_after_an_answer_waits_too() {
    for_both(|h, sr| {
        h.t.app.doc.modified = true;
        h.t.app.new_song();
        h.calls(2);
        assert!(h.t.app.prompting());
        // answered in one frame, the next question opened in the very next frame
        h.t.key_down(Key::Escape);
        assert!(!h.t.app.prompting(), "cancelled ({sr})");
        h.t.app.new_song();
        h.t.key_down(Key::Enter);
        assert!(
            h.t.app.prompting(),
            "the Enter did not answer the next one ({sr})"
        );
    });
}

#[test]
fn an_initialize_question_opened_right_after_an_answer_waits_too() {
    for_both(|h, sr| {
        h.t.app.ask_init_for_test();
        h.calls(2);
        h.t.key_down(Key::Escape);
        assert!(!h.t.app.confirming_init(), "cancelled ({sr})");
        h.t.app.ask_init_for_test();
        h.t.key_down(Key::Enter);
        assert!(
            h.t.app.confirming_init(),
            "the Enter did not answer the next one ({sr})"
        );
        assert!(!h.t.app.doc.modified);
    });
}

#[test]
fn initialize_song_from_the_menu_escape_cancels() {
    for_both(|h, sr| {
        h.walk(1, "Initialize Song");
        h.t.key(Key::Enter);
        h.calls(3);
        assert!(h.t.app.confirming_init());
        h.t.key(Key::Escape);
        h.calls(2);
        assert!(!h.t.app.confirming_init(), "Escape cancels ({sr})");
        assert!(!h.t.app.doc.modified, "nothing initialized ({sr})");
    });
}
