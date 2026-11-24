//! The file browser's list keeps the selected row in view (task-285): after Up, after the
//! keys that move the selection and after type-to-select, in a folder of thousands of
//! entries; a click does not scroll.

use std::fs;
use std::path::Path;

use eframe::egui::{self, Event, Key, Modifiers, PointerButton, Pos2, RawInput, Rect, Vec2};
use orb_ui::browser::{Browser, Mode};
use orb_ui::settings::Settings;

struct Harness {
    ctx: egui::Context,
    time: f64,
}

impl Harness {
    fn new() -> Self {
        Self {
            ctx: egui::Context::default(),
            time: 0.0,
        }
    }

    fn frame(&mut self, b: &mut Browser, s: &mut Settings, events: Vec<Event>) {
        self.time += 0.016;
        let input = RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(624.0, 510.0))),
            time: Some(self.time),
            events,
            ..Default::default()
        };
        let ctx = self.ctx.clone();
        let mut out = self.ctx.run_ui(input, |_| {
            b.show(&ctx, s);
        });
        out.textures_delta.clear();
    }

    fn settle(&mut self, b: &mut Browser, s: &mut Settings) {
        for _ in 0..4 {
            self.frame(b, s, vec![]);
        }
    }

    fn key_mods(&mut self, b: &mut Browser, s: &mut Settings, k: Key, m: Modifiers) {
        let ev = |pressed| Event::Key {
            key: k,
            physical_key: Some(k),
            pressed,
            repeat: false,
            modifiers: m,
        };
        self.frame(b, s, vec![Event::ModifiersChanged(m), ev(true), ev(false)]);
        self.frame(b, s, vec![Event::ModifiersChanged(Modifiers::NONE)]);
    }

    fn key(&mut self, b: &mut Browser, s: &mut Settings, k: Key) {
        self.key_mods(b, s, k, Modifiers::NONE);
    }
}

fn big_parent(n: usize) -> tempfile::TempDir {
    let d = tempfile::tempdir().unwrap();
    for i in 0..n {
        fs::create_dir(d.path().join(format!("d{i:04}"))).unwrap();
    }
    d
}

fn browser(dir: &Path) -> (Browser, Settings) {
    let s = Settings::default();
    (Browser::new(Mode::OpenSong, &s).with_dir(dir), s)
}

/// The named row is fully inside the list once the layout has settled.
fn assert_visible(h: &mut Harness, b: &mut Browser, s: &mut Settings, name: &str, why: &str) {
    h.settle(b, s);
    let list = b.hit("list").expect("no list");
    let row = b
        .hit(&format!("row:{name}"))
        .unwrap_or_else(|| panic!("{why}: row {name} is not drawn"));
    assert!(
        row.min.y >= list.min.y - 0.5 && row.max.y <= list.max.y + 0.5,
        "{why}: row {name} {row:?} is outside the list {list:?}"
    );
}

#[test]
fn up_into_a_big_folder_shows_the_folder_that_was_left() {
    let d = big_parent(2400);
    let mut h = Harness::new();
    for k in [0usize, 5, 1200, 2395, 2399] {
        let name = format!("d{k:04}");
        let (mut b, mut s) = browser(&d.path().join(&name));
        h.settle(&mut b, &mut s);
        h.key_mods(&mut b, &mut s, Key::ArrowUp, Modifiers::ALT);
        assert_eq!(b.selected_name(), Some(name.as_str()));
        assert_visible(&mut h, &mut b, &mut s, &name, "after Alt+Up");
        // Centered, except where the scroll clamps at the start or the end.
        let list = b.hit("list").unwrap();
        let row = b.hit(&format!("row:{name}")).unwrap();
        let off = (row.center().y - list.center().y).abs();
        if (1000..=1400).contains(&k) {
            assert!(off <= 23.0, "{name} is {off} from the center of the list");
        } else if k < 100 {
            assert!(row.min.y < list.center().y, "{name} near the start");
        } else {
            assert!(row.max.y > list.center().y, "{name} near the end");
        }
    }
}

#[test]
fn plain_up_arrow_moves_the_selection_and_does_not_go_to_the_parent() {
    let d = big_parent(2400);
    let mut h = Harness::new();
    let (mut b, mut s) = browser(d.path());
    h.settle(&mut b, &mut s);
    h.key(&mut b, &mut s, Key::End);
    h.key(&mut b, &mut s, Key::ArrowUp);
    assert_eq!(b.dir(), d.path());
    assert_eq!(b.selected_name(), Some("d2398"));
    assert_visible(&mut h, &mut b, &mut s, "d2398", "ArrowUp");
}

#[test]
fn selection_moves_keep_the_selected_row_visible() {
    let d = big_parent(2400);
    let mut h = Harness::new();
    let (mut b, mut s) = browser(d.path());
    h.settle(&mut b, &mut s);
    h.key(&mut b, &mut s, Key::End);
    assert_visible(&mut h, &mut b, &mut s, "d2399", "End");
    h.key(&mut b, &mut s, Key::Home);
    assert_visible(&mut h, &mut b, &mut s, "d0000", "Home");
    for _ in 0..3 {
        h.key(&mut b, &mut s, Key::PageDown);
    }
    let n = b.selected_name().unwrap().to_string();
    assert_visible(&mut h, &mut b, &mut s, &n, "PageDown");
    for _ in 0..40 {
        h.key(&mut b, &mut s, Key::ArrowDown);
    }
    let n = b.selected_name().unwrap().to_string();
    assert_visible(&mut h, &mut b, &mut s, &n, "ArrowDown");
    h.frame(&mut b, &mut s, vec![Event::Text("d1777".into())]);
    assert_eq!(b.selected_name(), Some("d1777"));
    assert_visible(&mut h, &mut b, &mut s, "d1777", "type-to-select");
    let (mut b, mut s) = browser(&d.path().join("d2222"));
    h.settle(&mut b, &mut s);
    h.key(&mut b, &mut s, Key::Backspace);
    assert_visible(&mut h, &mut b, &mut s, "d2222", "Backspace");
}

#[test]
fn a_click_does_not_scroll_the_list() {
    let d = big_parent(2400);
    let mut h = Harness::new();
    let (mut b, mut s) = browser(d.path());
    h.settle(&mut b, &mut s);
    h.key(&mut b, &mut s, Key::End);
    h.settle(&mut b, &mut s);
    let list = b.hit("list").unwrap();
    let rows: Vec<_> = (2380..2400)
        .filter_map(|i| b.hit(&format!("row:d{i:04}")).map(|r| (i, r)))
        .collect();
    let (i, rect) = rows[2];
    let pos = rect.center();
    let button = |pressed| Event::PointerButton {
        pos,
        button: PointerButton::Primary,
        pressed,
        modifiers: Modifiers::NONE,
    };
    h.frame(&mut b, &mut s, vec![Event::PointerMoved(pos)]);
    h.frame(&mut b, &mut s, vec![button(true)]);
    h.frame(&mut b, &mut s, vec![button(false)]);
    h.settle(&mut b, &mut s);
    assert_eq!(b.selected_name(), Some(format!("d{i:04}").as_str()));
    assert_eq!(b.hit("list").unwrap(), list);
    assert_eq!(b.hit(&format!("row:d{i:04}")).unwrap(), rect);
}
