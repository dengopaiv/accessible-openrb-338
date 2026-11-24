//! Edit > Copy Pattern puts the pattern on the clipboard as JSON and Paste reads it back
//! (task-287, docs/ui_spec.md section 16). The apps here keep their memory clipboard
//! (`Clipboard::Memory`, the default): no test touches the real system clipboard.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use eframe::egui::{self, Event, Pos2, RawInput, Rect, Vec2};
use orb_engine::tables::Tables;
use orb_ui::app::App;
use orb_ui::layout::{PANEL_H, PANEL_W};
use orb_ui::link::{self, Link};
use orb_ui::menu::Cmd;
use orb_ui::model::Panel;
use orb_ui::pattern_json::{Refused, parse, to_json};

struct Harness {
    ctx: egui::Context,
    app: App,
    t: f64,
}

impl Harness {
    /// An app with the player's link and the empty song it starts with.
    fn new() -> Self {
        let ctx = egui::Context::default();
        egui_extras::install_image_loaders(&ctx);
        let samples = rbm::standard::builtin_samples();
        let (link, _audio) = Link::new(Arc::new(Tables::new()), Some(samples.clone()));
        let mut app = App::with_link(Panel::new(), link, None);
        app.samples = Some(samples);
        let mut h = Self { ctx, app, t: 0.0 };
        h.frame(vec![]);
        h
    }

    fn frame(&mut self, events: Vec<Event>) {
        self.t += 0.016;
        let input = RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(PANEL_W, PANEL_H))),
            time: Some(self.t),
            events,
            ..Default::default()
        };
        let app = &mut self.app;
        let mut out = self.ctx.run_ui(input, |ui| app.show(ui));
        out.textures_delta.clear();
    }

    /// Focuses row `r` (0..=3) and selects its pattern `p`.
    fn select(&mut self, r: usize, p: u8) {
        self.app
            .panel
            .press(&format!("pattern{}.pattern{}", r + 1, p + 1), 0.0);
        self.frame(vec![]);
    }

    fn pattern(&self, r: usize, p: u8) -> Vec<u8> {
        self.app.link.as_ref().unwrap().pattern(r, p).to_vec()
    }

    fn set(&mut self, r: usize, p: u8, bytes: &[u8]) {
        let link = self.app.link.as_mut().unwrap();
        link.set_pattern(r, p, bytes, &mut self.app.panel);
        self.frame(vec![]);
    }

    /// Edit > Copy Pattern of the selected pattern, as far as the clipboard.
    fn copy(&mut self) -> String {
        self.app.command(Cmd::CopyPattern);
        self.frame(vec![]);
        self.app.clipboard.read().expect("a copy")
    }
}

/// A pattern of row `r` with every byte a different value, including the ones the readable form
/// names nothing for.
fn odd_pattern(r: usize) -> Vec<u8> {
    let len = if r < 2 { 98 } else { 194 };
    let mut b: Vec<u8> = (0..len).map(|i| (i * 37 + 11) as u8).collect();
    // the length is the one byte a paste checks (1 to 16)
    b[1] = 13;
    b
}

fn songs_in(dir: &Path, out: &mut Vec<PathBuf>) {
    let mut entries: Vec<_> = std::fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    entries.sort();
    for p in entries {
        if p.is_dir() {
            songs_in(&p, out);
        } else if p.extension().is_some_and(|e| e == "rbs") {
            out.push(p);
        }
    }
}

/// Every pattern of every corpus song (tests/songs: the diag songs, which the app entries
/// share, and the shipped songs, which are the CD's songs committed with the corpus, so no
/// `ORB_CD_ISO` leg is needed: orb-testkit gives the CD's DAT, DLL, EXE and mods, not its
/// songs), as the link holds it after the load, gives back its bytes through the text.
#[test]
fn every_corpus_pattern_round_trips() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/songs");
    let mut songs = Vec::new();
    songs_in(&root, &mut songs);
    assert!(songs.len() > 100, "{} songs", songs.len());
    let mut h = Harness::new();
    let (mut patterns, mut bad_length) = (0, 0);
    for song in &songs {
        h.app.doc.modified = false;
        h.app.open_now(song);
        let link = h.app.link.as_ref().unwrap();
        for r in 0..4 {
            for p in 0..32 {
                let bytes = link.pattern(r, p).to_vec();
                let text = to_json(r, &bytes);
                if !(1..=16).contains(&bytes[1]) {
                    // a length outside 1 to 16: the copy writes it, the paste refuses it
                    assert!(
                        text.contains(&format!("\"length\": {},", bytes[1])),
                        "{text}"
                    );
                    assert!(
                        matches!(parse(&text), Err(Refused::Invalid(_))),
                        "{song:?} row {r} pattern {p}"
                    );
                    bad_length += 1;
                    continue;
                }
                let got = parse(&text).unwrap_or_else(|e| panic!("{song:?} {r} {p}: {e:?}"));
                assert_eq!(got.bytes, bytes, "{song:?} row {r} pattern {p}\n{text}");
                patterns += 1;
            }
        }
    }
    println!(
        "{} songs, {patterns} patterns round trip, {bad_length} with a length outside 1..16",
        songs.len()
    );
}

#[test]
fn a_copy_in_one_app_pastes_in_another_with_every_byte() {
    let mut a = Harness::new();
    let mut b = Harness::new();
    // the two apps share one clipboard, as two windows or plugin instances share the system's
    b.app.clipboard = a.app.clipboard.clone();
    for (r, kind) in [(0, "303"), (1, "303"), (2, "808"), (3, "909")] {
        let odd = odd_pattern(r);
        a.select(r, 3);
        a.set(r, 3, &odd);
        let text = a.copy();
        assert_eq!(parse(&text).unwrap().device, kind);
        assert!(text.contains(&format!("\"device\": \"{kind}\"")), "{text}");
        b.select(r, 5);
        assert!(link::is_empty_pattern(r, &b.pattern(r, 5)));
        b.app.command(Cmd::PastePattern);
        b.frame(vec![]);
        assert_eq!(b.pattern(r, 5), odd, "row {r}");
        assert_eq!(b.app.undo_name(), Some("Paste Pattern"));
    }
    // a 303 pattern pastes into the other 303 row
    a.select(0, 2);
    a.set(0, 2, &odd_pattern(0));
    a.copy();
    b.select(1, 7);
    b.app.command(Cmd::PastePattern);
    assert_eq!(b.pattern(1, 7), odd_pattern(0));
}

#[test]
fn cut_puts_the_text_on_the_clipboard_and_the_editors_paste_it_back() {
    let mut h = Harness::new();
    let odd = odd_pattern(2);
    h.select(2, 0);
    h.set(2, 0, &odd);
    h.app.command(Cmd::CutPattern);
    h.frame(vec![]);
    assert!(link::is_empty_pattern(2, &h.pattern(2, 0)));
    let text = h.app.clipboard.read().unwrap();
    assert_eq!(parse(&text).unwrap().bytes, odd);
    // the standalone's paste: eframe answers the paste key and the menu item with a Paste event
    h.select(2, 1);
    h.frame(vec![Event::Paste(text)]);
    assert_eq!(h.pattern(2, 1), odd);
}

/// A text that is no valid pattern of the focused row's device writes nothing and says why.
#[test]
fn a_refused_text_writes_nothing_and_says_why() {
    let mut h = Harness::new();
    let good = to_json(2, &odd_pattern(2));
    let wrong_device = to_json(3, &odd_pattern(3));
    let cases: Vec<(String, &str)> = vec![
        ("hello".into(), "The clipboard holds no pattern"),
        ("".into(), "The clipboard holds no pattern"),
        (
            "{\"openrb338\": \"pattern\"".into(),
            "The clipboard holds no pattern",
        ),
        ("{\"a\": 1}".into(), "The clipboard holds no pattern"),
        (wrong_device, "The clipboard holds a 909 pattern"),
        (
            good.replace("\"version\": 1", "\"version\": 2"),
            "The clipboard's pattern is not valid: version 2 is not supported",
        ),
        (
            good.replace("\"device\": \"808\"", "\"device\": \"TR-808\""),
            "The clipboard's pattern is not valid: \"device\" must be",
        ),
        (
            good.replacen("\"bd\": ", "\"bx\": ", 1),
            "The clipboard's pattern is not valid: step 1 has the unknown key \"bx\"",
        ),
        (
            good.replacen("\"ac\": ", "\"ac\": \"loud\", \"x\": ", 1),
            "The clipboard's pattern is not valid: step 1",
        ),
        (
            good.replace("\"length\": ", "\"length\": 300, \"len\": "),
            "The clipboard's pattern is not valid:",
        ),
        (
            good.replace("\"length\": 13", "\"length\": 0"),
            "The clipboard's pattern is not valid: \"length\" must be 1 to 16, not 0",
        ),
        (
            good.replace("\"length\": 13", "\"length\": 17"),
            "The clipboard's pattern is not valid: \"length\" must be 1 to 16, not 17",
        ),
    ];
    h.select(2, 4);
    let before = h.app.link.as_ref().unwrap().patterns().clone();
    h.app.doc.modified = false;
    for (text, status) in cases {
        h.app.clipboard.copied(&text);
        h.app.status = None;
        h.app.command(Cmd::PastePattern);
        h.frame(vec![]);
        let got = h.app.status.clone().unwrap_or_default();
        assert!(got.starts_with(status), "{text:?}: {got:?}");
        assert_eq!(h.app.link.as_ref().unwrap().patterns(), &before, "{text:?}");
        assert_eq!(h.app.undo_name(), None, "{text:?}");
        assert!(!h.app.doc.modified, "{text:?}");
    }
    // the same pattern, valid, pastes
    h.app.clipboard.copied(&good);
    h.app.command(Cmd::PastePattern);
    assert_eq!(h.pattern(2, 4), odd_pattern(2));
}

/// The readable form: an 808 step is one line of booleans, a 909 drum is a word.
#[test]
fn the_text_is_readable() {
    let mut b = vec![0u8; 194];
    b[0] = 1;
    b[1] = 12;
    b[2..14].copy_from_slice(&[2, 3, 0, 1, 2, 0, 0, 0, 0, 0, 0, 0]);
    let text = to_json(3, &b);
    assert!(
        text.starts_with("{\n  \"openrb338\": \"pattern\",\n  \"version\": 1,\n"),
        "{text}"
    );
    assert!(text.contains("\"shuffle\": true"), "{text}");
    assert!(text.contains("\"length\": 12"), "{text}");
    assert!(
        text.contains(concat!(
            "{\"ac\": true, \"bd\": \"flam\", \"sd\": \"off\", \"lt\": \"low\", ",
            "\"mt\": \"high\", \"ht\": \"off\""
        )),
        "{text}"
    );
    assert_eq!(text.lines().count(), 7 + 16 + 2, "one line per step");
}
