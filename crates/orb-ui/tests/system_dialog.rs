//! The system file dialogs of screen reader mode (task-194), headless: the dialog is replaced by
//! a fake `Picker` that records what it was asked and answers a prepared path. The mode decides
//! the path (system dialog or in-app browser), and a chosen file goes through `App::chosen`, as
//! the browser's does.

use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;

#[path = "common/a11y.rs"]
mod a11y;

use eframe::egui;
use eframe::egui::accesskit::{Action, NodeId, Role};
use orb_ui::app::App;
use orb_ui::browser::ExportFormat::{self, Flac, Ogg, Wav};
use orb_ui::browser::Mode;
use orb_ui::menu::Cmd;
use orb_ui::model::Panel;
use orb_ui::screen_reader::ScreenReaderSetting;
use orb_ui::settings::Settings;
use orb_ui::system_dialog::{Folder, Picker, Request, with_extension};

type Asked = Rc<RefCell<Vec<(Request, PathBuf)>>>;

/// A fake dialog: records each request with its start folder, answers `answer`.
fn fake(answer: Option<PathBuf>) -> (Picker, Asked) {
    let asked: Asked = Rc::default();
    let log = asked.clone();
    let picker: Picker = Box::new(move |request, start, _parent| {
        log.borrow_mut().push((request, start.to_path_buf()));
        answer.clone()
    });
    (picker, asked)
}

/// An app whose settings live in `dir/ui.toml`, with the screen reader setting `mode`.
fn app(dir: &Path, mode: ScreenReaderSetting) -> App {
    let mut app = App::new(Panel::new(), None);
    app.settings = Settings::load_from(&dir.join("ui.toml"));
    app.settings.screen_reader = mode;
    app
}

fn song() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/songs/diag/songs/303_accent0.rbs")
        .canonicalize()
        .unwrap()
}

#[test]
fn with_the_mode_on_open_uses_the_system_dialog_and_opens_the_file() {
    let d = tempfile::tempdir().unwrap();
    let mut app = app(d.path(), ScreenReaderSetting::On);
    let (picker, asked) = fake(Some(song()));
    app.set_picker(picker);
    app.command(Cmd::Open);
    assert!(!app.browsing(), "no in-app browser");
    assert_eq!(asked.borrow().len(), 1);
    assert_eq!(asked.borrow()[0].0, Request::File(Mode::OpenSong));
    // the path reached `open_song`; this app has no player, which is what it reports
    assert_eq!(app.error(), Some("There is no player to open the song in."));
    // the browser's bookkeeping: the folder is the last one and a recent one, in ui.toml
    let dir = song().parent().unwrap().to_path_buf();
    let saved = Settings::load_from(&d.path().join("ui.toml"));
    assert_eq!(saved.last_song_dir, Some(dir.clone()));
    assert_eq!(saved.recent.first(), Some(&dir));
}

#[test]
fn the_dialog_starts_in_the_folder_the_browser_starts_in() {
    let d = tempfile::tempdir().unwrap();
    let songs = d.path().join("songs");
    let mods = d.path().join("mods");
    std::fs::create_dir_all(&songs).unwrap();
    std::fs::create_dir_all(&mods).unwrap();
    let mut app = app(d.path(), ScreenReaderSetting::On);
    app.settings.songs_dir = Some(songs.clone());
    app.settings.mods_dir = Some(mods.clone());
    let (picker, asked) = fake(None);
    app.set_picker(picker);
    for mode in [
        Mode::OpenSong,
        Mode::SaveSong,
        Mode::Export,
        Mode::ChooseMod,
    ] {
        app.open_browser(mode);
    }
    let asked = asked.borrow();
    let want = [
        (Request::File(Mode::OpenSong), &songs),
        (Request::File(Mode::SaveSong), &songs),
        (Request::Export(Wav), &songs),
        (Request::File(Mode::ChooseMod), &mods),
    ];
    assert_eq!(asked.len(), 4);
    for ((req, start), (wreq, wstart)) in asked.iter().zip(want) {
        assert_eq!((req, start), (&wreq, wstart));
    }
    // cancelled dialogs change nothing
    assert!(!app.browsing());
    assert_eq!(app.doc.path, None);
    assert_eq!(app.error(), None);
}

#[test]
fn a_chosen_path_goes_through_the_same_chosen_as_the_browsers() {
    let d = tempfile::tempdir().unwrap();
    // a file to save or export with no song loaded: the error comes from save_to / export_to,
    // so the path reached them (App::chosen)
    let out = d.path().join("out.rbs");
    let mut app = app(d.path(), ScreenReaderSetting::On);
    let (picker, _) = fake(Some(out.clone()));
    app.set_picker(picker);
    app.open_browser(Mode::SaveSong);
    assert!(
        app.error().unwrap().contains("no song to save"),
        "{:?}",
        app.error()
    );
    // a mod that is not one: the browser's check, the browser's text
    let mut app2 = self::app(d.path(), ScreenReaderSetting::On);
    let junk = d.path().join("junk.rbm");
    std::fs::write(&junk, b"not a mod").unwrap();
    let (picker, _) = fake(Some(junk.clone()));
    app2.set_picker(picker);
    app2.open_browser(Mode::ChooseMod);
    assert!(app2.error().is_some(), "a bad mod is refused");
    // a file to open that does not exist
    let mut app3 = self::app(d.path(), ScreenReaderSetting::On);
    let (picker, _) = fake(Some(d.path().join("missing.rbs")));
    app3.set_picker(picker);
    app3.open_browser(Mode::OpenSong);
    assert!(
        app3.error().unwrap().contains("no such file"),
        "{:?}",
        app3.error()
    );
    assert_eq!(app3.doc.path, None);
}

#[test]
fn typed_names_get_the_browsers_extensions() {
    let p = |s: &str| PathBuf::from(s);
    assert_eq!(
        with_extension(Mode::SaveSong, p("a"), Wav).unwrap(),
        p("a.rbs")
    );
    assert_eq!(
        with_extension(Mode::SaveSong, p("a.RBS"), Wav).unwrap(),
        p("a.RBS")
    );
    assert_eq!(
        with_extension(Mode::SaveSong, p("a.txt"), Wav).unwrap(),
        p("a.txt.rbs")
    );
    assert_eq!(
        with_extension(Mode::Export, p("a"), Wav).unwrap(),
        p("a.wav")
    );
    assert_eq!(
        with_extension(Mode::Export, p("a.Flac"), Wav).unwrap(),
        p("a.Flac")
    );
    assert!(with_extension(Mode::Export, p("a.mp3"), Wav).is_err());
    assert_eq!(
        with_extension(Mode::OpenSong, p("a.x"), Wav).unwrap(),
        p("a.x")
    );
}

/// Task-204: after an export as FLAC the next Export dialog offers FLAC first, a name typed
/// without extension gets `.flac`, and the choice survives a restart (ui.toml).
#[test]
fn the_export_dialog_keeps_the_format_the_user_picked() {
    let d = tempfile::tempdir().unwrap();
    let mut app = app(d.path(), ScreenReaderSetting::On);
    let asked: Rc<RefCell<Vec<Request>>> = Rc::default();
    let answers: Rc<RefCell<Vec<PathBuf>>> = Rc::default();
    let (log, ans) = (asked.clone(), answers.clone());
    app.set_picker(Box::new(move |request, _, _| {
        log.borrow_mut().push(request);
        Some(ans.borrow_mut().remove(0))
    }));
    assert_eq!(app.settings.export_format, Wav);
    answers.borrow_mut().push(d.path().join("a.flac"));
    app.open_browser(Mode::Export);
    assert_eq!(asked.borrow()[0], Request::Export(Wav));
    assert_eq!(app.settings.export_format, Flac);
    // the next dialog offers FLAC first; a bare name gets .flac (the export itself fails here
    // for lack of a song, which is not the point; b.flac exists, so the app asks to replace)
    std::fs::write(d.path().join("b.flac"), b"x").unwrap();
    answers.borrow_mut().push(d.path().join("b"));
    app.open_browser(Mode::Export);
    assert_eq!(asked.borrow()[1], Request::Export(Flac));
    assert!(
        app.replacing(),
        "b became b.flac, which exists: the app asks"
    );
    // other dialogs are asked as before
    answers.borrow_mut().push(d.path().join("c.rbs"));
    app.open_browser(Mode::SaveSong);
    assert_eq!(asked.borrow()[2], Request::File(Mode::SaveSong));
    // remembered across a restart, and a later OGG replaces it
    let mut again = self::app(d.path(), ScreenReaderSetting::On);
    assert_eq!(again.settings.export_format, Flac);
    let (picker, _) = fake(Some(d.path().join("e.ogg")));
    again.set_picker(picker);
    again.open_browser(Mode::Export);
    let saved = Settings::load_from(&d.path().join("ui.toml"));
    assert_eq!(saved.export_format, Ogg);
    let _: ExportFormat = saved.export_format;
    // with_extension gives the remembered format to a bare name only
    assert_eq!(
        with_extension(Mode::Export, PathBuf::from("n"), Ogg).unwrap(),
        PathBuf::from("n.ogg")
    );
    assert_eq!(
        with_extension(Mode::Export, PathBuf::from("n.wav"), Ogg).unwrap(),
        PathBuf::from("n.wav")
    );
}

/// `App::new` asks the real system nothing and has no real dialog: with the setting `auto` the
/// mode is off whatever runs on this machine, and a dialog request is cancelled.
#[test]
fn a_new_app_is_inert() {
    let mut app = App::new(Panel::new(), None);
    assert_eq!(app.settings.screen_reader, ScreenReaderSetting::Auto);
    assert!(!app.screen_reader_mode());
    app.open_browser(Mode::OpenSong);
    assert!(app.browsing(), "the in-app browser");
    // the mode on, the dialog is the cancelling one: nothing happens
    let mut app = App::new(Panel::new(), None);
    app.settings.screen_reader = ScreenReaderSetting::On;
    app.open_browser(Mode::OpenSong);
    assert!(!app.browsing() && app.error().is_none());
}

/// An app with a player, so that a song can be saved.
fn linked(dir: &Path) -> (App, orb_player::Audio) {
    let tables = std::sync::Arc::new(orb_engine::tables::Tables::new());
    let (link, audio) = orb_ui::link::Link::new(tables, Some(rbm::standard::builtin_samples()));
    let mut app = App::with_link(Panel::new(), link, None);
    app.settings = Settings::load_from(&dir.join("ui.toml"));
    app.settings.screen_reader = ScreenReaderSetting::On;
    (app, audio)
}

/// One frame of the app's replace question with `events`.
fn replace_frame(app: &mut App, events: Vec<egui::Event>) {
    let ctx = egui::Context::default();
    let input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::Vec2::new(624.0, 510.0),
        )),
        time: Some(1.0),
        events,
        ..Default::default()
    };
    let _ = ctx.run_ui(input, |ui| app.replace_ui(&ui.ctx().clone()));
}

#[test]
fn a_name_the_dialog_did_not_confirm_asks_before_it_replaces_a_file() {
    let d = tempfile::tempdir().unwrap();
    let existing = d.path().join("a.txt.rbs");
    std::fs::write(&existing, b"old").unwrap();
    let picked = d.path().join("a.txt");

    // No: nothing is written, nothing changes
    let (mut app, _audio) = linked(d.path());
    let (picker, _) = fake(Some(picked.clone()));
    app.set_picker(picker);
    app.command(Cmd::SaveAs);
    assert!(app.replacing(), "the question is open");
    assert_eq!(std::fs::read(&existing).unwrap(), b"old", "not written yet");
    assert_eq!(app.error(), None);
    app.answer_replace(false);
    assert!(!app.replacing());
    assert_eq!(std::fs::read(&existing).unwrap(), b"old");
    assert_eq!(app.doc.path, None);

    // Yes (the key Y): the song is written over the file
    let (picker, _) = fake(Some(picked));
    app.set_picker(picker);
    app.command(Cmd::SaveAs);
    assert!(app.replacing());
    let y = egui::Event::Key {
        key: egui::Key::Y,
        physical_key: Some(egui::Key::Y),
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers::NONE,
    };
    replace_frame(&mut app, vec![y]);
    assert!(!app.replacing());
    assert_ne!(std::fs::read(&existing).unwrap(), b"old", "replaced");
    assert_eq!(app.doc.path.as_deref(), Some(existing.as_path()));

    // a name the dialog confirmed (it has the extension) is not asked about again
    let (picker, _) = fake(Some(existing.clone()));
    app.set_picker(picker);
    app.command(Cmd::SaveAs);
    assert!(!app.replacing());
}

/// An app tree, screen reader mode on, whose Save As picker answers `a.txt` while `a.txt.rbs`
/// exists (the question "Replace a.txt.rbs?" opens), with a panel control focused first.
fn question_tree(dir: &Path) -> (a11y::AppTree, PathBuf) {
    let existing = dir.join("a.txt.rbs");
    std::fs::write(&existing, b"old").unwrap();
    let mut h = a11y::AppTree::new();
    h.app.settings = Settings::load_from(&dir.join("ui.toml"));
    h.app.settings.screen_reader = ScreenReaderSetting::On;
    let (picker, _) = fake(Some(dir.join("a.txt")));
    h.app.set_picker(picker);
    // something holds the focus before the box opens
    let knob = a11y::hit_id("knob", "tb303_1.cutoff");
    h.send(knob, Action::Focus, None);
    assert_eq!(
        h.tree().focused_id(),
        knob.accesskit_id(),
        "the knob has the focus"
    );
    h.app.command(Cmd::SaveAs);
    h.frame(vec![]);
    h.frame(vec![]);
    assert!(h.app.replacing());
    (h, existing)
}

#[test]
fn the_question_takes_the_focus_on_no_and_is_a_named_dialog() {
    let d = tempfile::tempdir().unwrap();
    let (h, _) = question_tree(d.path());
    let t = h.tree();
    let no: Vec<NodeId> = t.by_name("No").map(|(id, _)| id).collect();
    assert_eq!(no.len(), 1, "one No button");
    assert_eq!(t.focused_id(), no[0], "the focus is on No");
    let dialogs: Vec<_> = t.by_role(Role::Dialog).collect();
    assert_eq!(dialogs.len(), 1, "one dialog node");
    assert_eq!(dialogs[0].1.label(), Some("Replace a.txt.rbs?"));
}

#[test]
fn n_escape_and_enter_answer_no_and_leave_the_file_alone() {
    for key in [egui::Key::N, egui::Key::Escape, egui::Key::Enter] {
        let d = tempfile::tempdir().unwrap();
        let (mut h, existing) = question_tree(d.path());
        h.key(key);
        assert!(!h.app.replacing(), "{key:?} closed the question");
        assert_eq!(std::fs::read(&existing).unwrap(), b"old", "{key:?}");
        assert_eq!(h.app.doc.path, None, "{key:?}");
        assert_eq!(h.app.error(), None, "{key:?}");
    }
}

#[test]
fn with_the_mode_off_the_in_app_browser_opens_and_the_dialog_is_not_asked() {
    let d = tempfile::tempdir().unwrap();
    let mut app = app(d.path(), ScreenReaderSetting::Off);
    let (picker, asked) = fake(Some(song()));
    app.set_picker(picker);
    for mode in [
        Mode::OpenSong,
        Mode::SaveSong,
        Mode::Export,
        Mode::ChooseMod,
    ] {
        app.open_browser(mode);
        assert!(app.browsing(), "{mode:?}");
    }
    assert!(asked.borrow().is_empty());
}

#[test]
fn the_folder_items_set_the_folders_as_the_browsers_buttons_do() {
    let d = tempfile::tempdir().unwrap();
    let songs = d.path().join("my songs");
    let mods = d.path().join("my mods");
    std::fs::create_dir_all(&songs).unwrap();
    std::fs::create_dir_all(&mods).unwrap();
    let mut app = app(d.path(), ScreenReaderSetting::On);
    let (picker, asked) = fake(Some(songs.clone()));
    app.set_picker(picker);
    app.command(Cmd::SetFolder(Folder::Songs));
    assert_eq!(asked.borrow()[0].0, Request::Folder(Folder::Songs));
    assert_eq!(app.settings.songs_dir, Some(songs.clone()));
    assert_eq!(app.settings.mods_dir, None);
    let (picker, asked) = fake(Some(mods.clone()));
    app.set_picker(picker);
    app.command(Cmd::SetFolder(Folder::Mods));
    assert_eq!(asked.borrow()[0].0, Request::Folder(Folder::Mods));
    assert_eq!(app.settings.mods_dir, Some(mods.clone()));
    let saved = Settings::load_from(&d.path().join("ui.toml"));
    assert_eq!(saved.songs_dir, Some(songs.clone()));
    assert_eq!(saved.mods_dir, Some(mods));
    // the next dialog starts in the folder now set
    let (picker, asked) = fake(None);
    app.set_picker(picker);
    app.command(Cmd::SetFolder(Folder::Songs));
    assert_eq!(asked.borrow()[0].1, songs);
    // a cancelled dialog changes nothing
    assert_eq!(Settings::load_from(&d.path().join("ui.toml")), saved);
    assert_eq!(app.error(), None);
}
