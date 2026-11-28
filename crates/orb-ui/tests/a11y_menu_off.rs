//! Menu items that are off (task-250). In screen reader mode they stay in the Up/Down order,
//! their node is disabled ("unavailable"), and Enter on one runs nothing, keeps the menu open
//! with the focus on it and announces why, once. With the mode off they are skipped and inert.

#[path = "common/a11y.rs"]
mod a11y;

use std::path::{Path, PathBuf};
use std::sync::Arc;

use a11y::{AppTree, Frames};
use eframe::egui::{Key, Vec2};
use orb_engine::tables::Tables;
use orb_player::{Audio, CALL_FRAMES};
use orb_ui::app::App;
use orb_ui::layout::{PANEL_H, PANEL_W};
use orb_ui::link::Link;
use orb_ui::menu::MENU_H;
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
}

impl H {
    /// Pattern mode: the Export and Initialize items are off.
    fn pattern_mode(&mut self) {
        self.t.app.panel.set_switch("transport.mode", 0);
        self.calls(2);
        assert!(!self.t.app.can_init());
    }

    /// Opens the menu `titles_right` titles from File and goes down to the item named `item`.
    /// Returns false when the arrows never reach it.
    fn reach(&mut self, titles_right: usize, item: &str) -> bool {
        self.t.key(Key::F10);
        for _ in 0..titles_right {
            self.t.key(Key::ArrowRight);
        }
        self.t.key(Key::ArrowDown);
        self.t.frame(vec![]);
        for _ in 0..30 {
            if self.focus().starts_with(item) {
                return true;
            }
            self.t.key(Key::ArrowDown);
        }
        false
    }

    fn disabled(&self) -> bool {
        self.t.tree().focused().unwrap().is_disabled()
    }

    fn said(&self) -> Option<String> {
        self.t.app.announcer.text().map(str::to_string)
    }

    fn menu_open(&self) -> bool {
        // "Save As..." is only in the tree while File's menu is open; so is "Redo"-less Undo
        // in Edit's: the focused item itself is in the open menu
        let f = self
            .t
            .tree()
            .focused()
            .map(|n| n.label().unwrap_or("").to_string());
        let open = |n: &str| self.t.tree().named().iter().any(|s| s.starts_with(n));
        f.is_some() && (open("Save As") || open("Undo"))
    }
}

#[test]
fn off_items_are_reached_disabled_and_say_why() {
    let cases = [
        (0, "Export Loop", "Needs song mode; F12 switches"),
        (1, "Initialize Song", "Needs song mode; F12 switches"),
        (1, "Initialize Loop", "Needs song mode; F12 switches"),
        (1, "Undo", "Nothing to undo"),
    ];
    for (titles, item, why) in cases {
        let mut h = H::new(true);
        h.pattern_mode();
        assert!(h.reach(titles, item), "{item} is reached by the arrows");
        assert!(h.disabled(), "{item} is marked disabled");
        let before = h.t.app.announcer.serial();
        h.t.key(Key::Enter);
        h.calls(2);
        assert!(h.focus().starts_with(item), "{item} keeps the focus");
        assert!(h.menu_open(), "the menu stays open on {item}");
        assert!(!h.t.app.confirming_init() && !h.t.app.browsing());
        assert_eq!(h.said().as_deref(), Some(why), "{item}");
        assert_eq!(h.t.app.announcer.serial(), before + 1, "said once: {item}");
        // Space does the same
        h.t.key(Key::Space);
        h.calls(2);
        assert!(h.focus().starts_with(item) && h.menu_open());
        assert_eq!(h.said().as_deref(), Some(why));
    }
}

#[test]
fn an_item_off_for_lack_of_a_song_says_so() {
    let mut h = H::new(true);
    h.t.app.link = None;
    h.calls(2);
    assert!(h.reach(0, "Info"), "Info is reached");
    assert!(h.disabled());
    h.t.key(Key::Enter);
    h.calls(2);
    assert_eq!(h.said().as_deref(), Some("Needs a song"));
    assert!(h.menu_open());
}

#[test]
fn an_item_that_is_on_is_not_disabled() {
    let mut h = H::new(true);
    assert!(h.reach(1, "Initialize Song"));
    assert!(!h.disabled());
}

#[test]
fn with_the_mode_off_items_that_are_off_are_skipped() {
    for (titles, item) in [(0, "Export Loop"), (1, "Initialize Song"), (1, "Undo")] {
        let mut h = H::new(false);
        h.pattern_mode();
        assert!(!h.reach(titles, item), "{item} is skipped");
    }
}

#[test]
fn export_with_no_song_says_it_needs_a_song() {
    let mut h = H::new(true);
    h.pattern_mode();
    h.t.app.link = None;
    h.calls(2);
    assert!(h.reach(0, "Export Loop"));
    assert!(h.disabled());
    h.t.key(Key::Enter);
    h.calls(2);
    assert_eq!(h.said().as_deref(), Some("Needs a song"));
}
