//! Edit > Initialize Loop from Pattern Mode and Initialize Song from Pattern Mode (task-306,
//! docs/ui_spec.md sections 12 and 18): run from the Edit menu on the songs of
//! `tests/songs/edit_init`, orb's saved song equals byte for byte the file the original saved
//! after the same command on the same song (made on the test machine, the folder's README).

use std::path::{Path, PathBuf};
use std::sync::Arc;

use eframe::egui::{self, Event, Key, Modifiers, PointerButton, Pos2, RawInput, Rect, Vec2};
use orb_engine::tables::Tables;
use orb_player::{Audio, CALL_FRAMES};
use orb_ui::app::App;
use orb_ui::layout::{PANEL_H, PANEL_W};
use orb_ui::link::Link;
use orb_ui::menu::{Cmd, MENU_H};
use orb_ui::model::Panel;

const LOOP: &str = "Initialize Loop from Pattern Mode";
const SONG: &str = "Initialize Song from Pattern Mode";

fn dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/songs/edit_init")
}

struct Harness {
    ctx: egui::Context,
    app: App,
    audio: Audio,
    t: f64,
    _tmp: tempfile::TempDir,
}

impl Harness {
    /// The app with a copy of `tests/songs/edit_init/<name>` open.
    fn open(name: &str) -> Self {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join(name);
        std::fs::copy(dir().join(name), &path).unwrap();
        let ctx = egui::Context::default();
        egui_extras::install_image_loaders(&ctx);
        let samples = rbm::standard::builtin_samples();
        let (link, audio) = Link::new(Arc::new(Tables::new()), Some(samples.clone()));
        let mut app = App::with_link(Panel::new(), link, None);
        app.samples = Some(samples);
        app.menu = true;
        app.open_song(&path);
        assert!(app.errors.is_empty(), "{:?}", app.errors);
        let mut h = Self {
            ctx,
            app,
            audio,
            t: 0.0,
            _tmp: tmp,
        };
        h.calls(3);
        h
    }

    fn frame(&mut self, events: Vec<Event>) {
        self.t += 0.016;
        let input = RawInput {
            screen_rect: Some(Rect::from_min_size(
                Pos2::ZERO,
                Vec2::new(PANEL_W, PANEL_H + MENU_H),
            )),
            time: Some(self.t),
            events,
            ..Default::default()
        };
        let app = &mut self.app;
        let mut out = self.ctx.run_ui(input, |ui| app.show(ui));
        out.textures_delta.clear();
    }

    /// `n` engine calls, each followed by a UI frame.
    fn calls(&mut self, n: usize) {
        let mut buf = vec![0i16; 2 * CALL_FRAMES];
        for _ in 0..n {
            self.audio.render_i16(&mut buf);
            self.frame(vec![]);
        }
    }

    fn click(&mut self, p: Pos2) {
        let b = |pressed| Event::PointerButton {
            pos: p,
            button: PointerButton::Primary,
            pressed,
            modifiers: Modifiers::NONE,
        };
        self.frame(vec![Event::PointerMoved(p)]);
        self.frame(vec![b(true)]);
        self.frame(vec![b(false)]);
    }

    fn key(&mut self, key: Key) {
        let ev = |pressed| Event::Key {
            key,
            physical_key: Some(key),
            pressed,
            repeat: false,
            modifiers: Modifiers::NONE,
        };
        self.frame(vec![ev(true)]);
        self.frame(vec![ev(false)]);
    }

    /// The Edit menu opened (found along the bar); the item `text`'s place and whether it is
    /// enabled.
    fn edit_menu(&mut self, text: &str) -> (Rect, bool) {
        for x in (6..300).step_by(4) {
            egui::Popup::close_all(&self.ctx);
            self.click(Pos2::new(x as f32, MENU_H / 2.0));
            self.frame(vec![]);
            // the app keeps the places of the last drawn menu: only an open one counts
            if egui::Popup::is_any_open(&self.ctx)
                && let Some(found) = self.app.edit_item(text)
            {
                return found;
            }
        }
        panic!("the Edit menu has {text}");
    }

    /// The menu item `text` clicked.
    fn choose(&mut self, text: &str) {
        let (r, enabled) = self.edit_menu(text);
        assert!(enabled, "{text} is enabled in song mode");
        self.click(r.center());
    }

    fn saved(&mut self) -> Vec<u8> {
        self.app.song_bytes().unwrap()
    }
}

/// The bytes the original's save never writes, so that they hold whatever its shared chunk
/// buffer `unk_4DB948` held (OB-26 in docs/original_bugs.md): GLOB 483..512 (RBS42.txt's
/// reserved bytes; the packer `sub_433EC2` writes 0..=482) and USRI 544..712 (`sub_43489D`
/// writes 0..=543); and COMP 3..8, which the original's packer `sub_4347D1` zeroes. orb keeps
/// the bytes it read in all three (the user's decision of 2026-11-25, task-307: saves need not
/// match the original's byte for byte).
fn stale(file: &[u8]) -> [std::ops::Range<usize>; 3] {
    let at = |tag: &[u8]| file.windows(4).position(|w| w == tag).unwrap() + 8;
    let (g, u, c) = (at(b"GLOB"), at(b"USRI"), at(b"COMP"));
    [g + 483..g + 512, u + 544..u + 712, c + 3..c + 8]
}

/// `got` equals `want` but for the bytes the original leaves stale ([`stale`]), which must be
/// as orb read them from the song `song`; else the first differing byte is named.
fn same(got: &[u8], want: &[u8], song: &str) {
    let read = expected(&format!("{song}.rbs"));
    let mut want = want.to_vec();
    for (r, i) in stale(got).into_iter().zip(stale(&read)) {
        assert_eq!(
            got[r.clone()],
            read[i],
            "{song}: orb keeps the bytes it read"
        );
        want[r.clone()].copy_from_slice(&got[r]);
    }
    if got != want {
        let at = got.iter().zip(&want).position(|(a, b)| a != b);
        panic!(
            "{song}: the save differs from the original's at byte {at:?} (lengths {} and {})",
            got.len(),
            want.len()
        );
    }
}

fn expected(name: &str) -> Vec<u8> {
    std::fs::read(dir().join(name)).unwrap()
}

/// Initialize Loop: no question; the song is marked modified and its save is the original's.
#[test]
fn initialize_loop_saves_the_originals_file() {
    for song in ["init_loop", "init_zero"] {
        let mut h = Harness::open(&format!("{song}.rbs"));
        assert!(!h.app.doc.modified);
        h.choose(LOOP);
        assert!(!h.app.confirming_init());
        h.calls(3);
        assert!(h.app.doc.modified);
        assert_eq!(
            h.app.status.as_deref(),
            Some("Loop initialized from pattern mode")
        );
        let want = expected(&format!("{song}.after_init_loop.rbs"));
        same(&h.saved(), &want, song);
    }
}

/// Initialize Song asks first; Enter (the original's default button) initializes, and the save
/// is the original's.
#[test]
fn initialize_song_asks_then_saves_the_originals_file() {
    for song in ["init_loop", "init_zero"] {
        let mut h = Harness::open(&format!("{song}.rbs"));
        h.choose(SONG);
        assert!(h.app.confirming_init());
        h.calls(2);
        assert!(!h.app.doc.modified, "nothing before the answer");
        h.key(Key::Enter);
        assert!(!h.app.confirming_init());
        h.calls(3);
        assert!(h.app.doc.modified);
        let want = expected(&format!("{song}.after_init_song.rbs"));
        same(&h.saved(), &want, song);
    }
}

/// Cancel (Escape) changes nothing.
#[test]
fn initialize_song_cancelled_changes_nothing() {
    let mut h = Harness::open("init_loop.rbs");
    let before = h.saved();
    h.app.command(Cmd::InitSong);
    h.frame(vec![]);
    assert!(h.app.confirming_init());
    h.key(Key::Escape);
    assert!(!h.app.confirming_init());
    h.calls(3);
    assert!(!h.app.doc.modified);
    assert!(h.saved() == before);
}

/// Both items are disabled in pattern mode (the original enables them in song mode only), and
/// their commands do nothing there.
#[test]
fn pattern_mode_disables_them() {
    let mut h = Harness::open("init_loop.rbs");
    h.app.panel.set_switch("transport.mode", 0);
    h.calls(3);
    assert!(!h.app.panel.transport.song_mode);
    let before = h.saved();
    // the mode switch is an edit of its own
    h.app.doc.modified = false;
    for text in [LOOP, SONG] {
        let (_, enabled) = h.edit_menu(text);
        assert!(!enabled, "{text}");
    }
    egui::Popup::close_all(&h.ctx);
    h.app.command(Cmd::InitLoop);
    h.app.command(Cmd::InitSong);
    h.frame(vec![]);
    assert!(!h.app.confirming_init());
    h.calls(3);
    assert!(!h.app.doc.modified);
    assert!(h.saved() == before);
}

/// orb's Undo of the commands (task-306; the original has none): Edit > Undo, named after the
/// command, puts the song back, so its save is the one before the command byte for byte. The
/// song stays modified, as after a pattern Undo.
#[test]
fn undo_gives_back_the_song_before_the_command() {
    for song in ["init_loop", "init_zero"] {
        for (text, name) in [(LOOP, "Initialize Loop"), (SONG, "Initialize Song")] {
            let mut h = Harness::open(&format!("{song}.rbs"));
            let before = h.saved();
            let (_, on) = h.edit_menu("Undo");
            assert!(!on, "nothing to undo yet");
            egui::Popup::close_all(&h.ctx);
            h.choose(text);
            if text == SONG {
                h.key(Key::Enter);
            }
            h.calls(3);
            assert_eq!(h.app.undo_name(), Some(name));
            assert!(h.saved() != before);
            let (r, on) = h.edit_menu("Undo");
            assert!(on, "{name}: Undo is enabled");
            h.click(r.center());
            h.calls(3);
            assert_eq!(h.app.undo_name(), None, "one level");
            assert_eq!(h.app.status, Some(format!("{name} undone")));
            assert!(h.app.doc.modified);
            assert!(
                h.saved() == before,
                "{song}, {name}: the save before the command"
            );
        }
    }
}

/// A pattern command after an Initialize takes the Undo over, and the reverse.
#[test]
fn pattern_commands_and_initialize_share_the_one_undo() {
    let mut h = Harness::open("init_loop.rbs");
    h.app.command(Cmd::InitLoop);
    h.calls(3);
    assert_eq!(h.app.undo_name(), Some("Initialize Loop"));
    h.app.command(Cmd::ClearPattern);
    h.calls(3);
    assert_eq!(h.app.undo_name(), Some("Clear Pattern"));
    h.app.command(Cmd::InitLoop);
    h.calls(3);
    assert_eq!(h.app.undo_name(), Some("Initialize Loop"));
    // that Undo puts back the tracks only: the cleared pattern stays cleared
    let cleared = h.app.link.as_ref().unwrap().pattern(0, 9).to_vec();
    h.app.command(Cmd::Undo);
    h.calls(3);
    assert_eq!(h.app.link.as_ref().unwrap().pattern(0, 9), &cleared[..]);
}

/// Undo while the song plays: the song goes on and the tracks are the ones before the command.
#[test]
fn undo_while_playing() {
    let mut h = Harness::open("init_loop.rbs");
    let before = h.saved();
    h.app
        .link
        .as_mut()
        .unwrap()
        .send([orb_player::Command::Play]);
    h.calls(10);
    assert!(h.audio.current().playing());
    h.app.command(Cmd::InitSong);
    h.key(Key::Enter);
    h.calls(10);
    h.app.command(Cmd::Undo);
    h.calls(10);
    assert!(h.audio.current().playing());
    let tracks = |b: &[u8]| rbs::Song::parse(b).unwrap().tracks;
    assert_eq!(tracks(&h.saved()), tracks(&before));
}
