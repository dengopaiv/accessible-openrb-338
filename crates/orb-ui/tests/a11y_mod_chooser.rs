//! The mod chooser (task-195, A11Y-10, docs/ui_spec.md section 20.7): Ctrl/Cmd+M and Mods >
//! Choose Mod... in screen reader mode open a dialog with a filter field and the Mods menu's
//! list as a tree. The list equals the menu's entries in order, the filter narrows it, Enter
//! loads the kit the menu item loads, the variants are a tree, Ctrl+I opens Mod Info for the
//! selection without loading it, Escape closes. With the mode off none of it exists.
//!
//! The mods are copies of the embedded standard mod with another INFO name (a recrypted mod
//! that loads); the open time with the 311 entries of the rb-mods collection is measured when
//! `ORB_THIRDPARTY_MODS_DIR` is set (`cargo test --release -p orb-ui --test a11y_mod_chooser
//! -- --nocapture`).

#[path = "common/a11y.rs"]
mod a11y;

use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};
use std::time::Instant;

use a11y::Frames;
use eframe::egui::accesskit::{Action, NodeId, Role};
use eframe::egui::{Event, Key, Modifiers, Popup, Vec2};
use orb_cli::mods::{Kit, STANDARD};
use orb_engine::tables::Tables;
use orb_player::Audio;
use orb_ui::app::App;
use orb_ui::layout::{PANEL_H, PANEL_W};
use orb_ui::link::Link;
use orb_ui::menu::Cmd;
use orb_ui::mod_chooser::Item;
use orb_ui::model::Panel;
use orb_ui::screen_reader::ScreenReaderSetting;

fn tables() -> Arc<Tables> {
    static T: OnceLock<Arc<Tables>> = OnceLock::new();
    T.get_or_init(|| Arc::new(Tables::new())).clone()
}

/// The settings file of this process: not the user's (a variant choice saves the preference).
fn private_settings() {
    static ONCE: OnceLock<tempfile::TempDir> = OnceLock::new();
    ONCE.get_or_init(|| {
        let d = tempfile::tempdir().unwrap();
        // SAFETY: set once, before the first app of this test process reads it.
        unsafe { std::env::set_var("ORB_UI_CONFIG", d.path().join("ui.toml")) };
        d
    });
}

/// A recrypted mod called `name`: the embedded standard mod with the INFO name replaced.
fn fake_mod(name: &str) -> Vec<u8> {
    let mut b = rbm::standard::EMBEDDED.to_vec();
    let at = b
        .windows(8)
        .position(|w| w[..4] == *b"INFO" && w[4..] == 1280u32.to_be_bytes())
        .expect("the INFO chunk")
        + 8;
    b[at + 1003..at + 1067].fill(0);
    b[at + 1003..at + 1003 + name.len()].copy_from_slice(name.as_bytes());
    b
}

/// A folder of mods: (file name, INFO name).
fn folder(files: &[(&str, &str)]) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    for (f, n) in files {
        std::fs::write(dir.path().join(f), fake_mod(n)).unwrap();
    }
    dir
}

const FILES: [(&str, &str); 6] = [
    ("1.rbm", "Beta"),
    ("2.rbm", "alpha"),
    ("3.rbm", "Shared"),
    ("4.rbm", "SHARED"),
    ("5.rbm", "Zed"),
    ("6.rbm", "Gamma Ray"),
];

struct H {
    f: Frames,
    app: App,
    _audio: Audio,
}

impl H {
    fn new(screen_reader: bool, dir: Option<&Path>) -> Self {
        private_settings();
        let (link, audio) = Link::new(tables(), Some(rbm::standard::builtin_samples()));
        let mut app = App::with_link(Panel::new(), link, None);
        app.samples = Some(rbm::standard::builtin_samples());
        app.menu = true;
        app.settings.screen_reader = if screen_reader {
            ScreenReaderSetting::On
        } else {
            ScreenReaderSetting::Off
        };
        app.settings.mods_dir = dir.map(Path::to_path_buf);
        app.refresh_mods();
        let mut h = Self {
            f: Frames::new(Vec2::new(PANEL_W, PANEL_H)),
            app,
            _audio: audio,
        };
        h.settle(5);
        h
    }

    fn frame(&mut self, events: Vec<Event>) {
        let app = &mut self.app;
        self.f.run(events, |ui| app.show(ui));
    }

    fn settle(&mut self, n: usize) {
        for _ in 0..n {
            self.frame(vec![]);
        }
    }

    fn key_ev(key: Key, pressed: bool, modifiers: Modifiers) -> Event {
        Event::Key {
            key,
            physical_key: Some(key),
            pressed,
            repeat: false,
            modifiers,
        }
    }

    /// Presses and releases a key with modifiers, then lets the frames after it run.
    fn press_with(&mut self, key: Key, m: Modifiers) {
        self.frame(vec![Event::ModifiersChanged(m), Self::key_ev(key, true, m)]);
        self.frame(vec![
            Event::ModifiersChanged(Modifiers::NONE),
            Self::key_ev(key, false, Modifiers::NONE),
        ]);
        self.settle(3);
    }

    fn press(&mut self, key: Key) {
        self.press_with(key, Modifiers::NONE);
    }

    fn type_text(&mut self, s: &str) {
        self.frame(vec![Event::Text(s.into())]);
        self.settle(3);
    }

    fn tree(&self) -> &a11y::Tree {
        self.f.tree.as_ref().expect("a frame was run")
    }

    /// Ctrl/Cmd+M.
    fn open_chooser(&mut self) {
        self.press_with(Key::M, Modifiers::COMMAND);
    }

    /// The Tree node and the names of its items in order.
    fn items(&self) -> Vec<String> {
        let t = self.tree();
        let (tree_id, _) = t.by_role(Role::Tree).next().expect("a Tree node");
        t.node(tree_id)
            .unwrap()
            .children()
            .iter()
            .map(|c| a11y::name(t.node(*c).unwrap()).unwrap_or("").to_string())
            .collect()
    }

    fn focused(&self) -> (Role, String) {
        let n = self.tree().focused().expect("a focused node");
        (a11y::role(n), a11y::name(n).unwrap_or("").to_string())
    }

    /// The name of the active descendant of the tree.
    fn selected(&self) -> String {
        let t = self.tree();
        let (_, tree) = t.by_role(Role::Tree).next().expect("a Tree node");
        let id: NodeId = tree.active_descendant().expect("an active descendant");
        a11y::name(t.node(id).expect("the item"))
            .unwrap()
            .to_string()
    }

    fn focus_filter(&mut self) {
        // Shift+Tab from the list
        self.press_with(Key::Tab, Modifiers::SHIFT);
    }
}

fn path_of(dir: &Path, file: &str) -> PathBuf {
    dir.join(file)
}

/// The Mods menu's entries, in order, are the list's after "Standard ReBirth"; the dialog, the
/// filter and the tree have roles and names; the focus is in the filter; the selection is on
/// the playing mod.
#[test]
fn the_list_is_the_menus_entries_in_order() {
    let dir = folder(&FILES);
    let mut h = H::new(true, Some(dir.path()));
    let play = a11y::hit_id("trigger", "transport.play");
    h.f.ctx.memory_mut(|m| m.request_focus(play));
    h.settle(2);
    assert!(!h.app.chooser_is_open());
    h.open_chooser();
    assert!(h.app.chooser_is_open());

    let menu: Vec<String> = h
        .app
        .mods()
        .unwrap()
        .as_ref()
        .unwrap()
        .iter()
        .map(|e| e.name.clone())
        .collect();
    assert_eq!(menu, ["alpha", "Beta", "Gamma Ray", "Shared", "Zed"]);
    let shared = h.app.mods().unwrap().as_ref().unwrap();
    assert_eq!(shared[3].variants.len(), 2);

    let want = [
        "Standard ReBirth, current",
        "alpha",
        "Beta",
        "Gamma Ray",
        "Shared, variants: 2",
        "Zed",
    ];
    assert_eq!(h.items(), want);
    assert_eq!(h.app.chooser_texts().unwrap(), want);

    // roles and names
    let t = h.tree();
    let dialogs: Vec<_> = t.by_role(Role::Dialog).collect();
    assert_eq!(dialogs.len(), 1);
    assert_eq!(a11y::name(dialogs[0].1), Some("Choose Mod"));
    let (_, tree) = t.by_role(Role::Tree).next().unwrap();
    assert_eq!(a11y::name(tree), Some("Mods"));
    assert!(
        t.by_role(Role::TextInput)
            .any(|(_, n)| a11y::name(n) == Some("Filter mods"))
    );
    // the items: TreeItem, level 1, position in set; the merged one is collapsed
    let items: Vec<_> = t
        .node(t.by_role(Role::Tree).next().unwrap().0)
        .unwrap()
        .children()
        .to_vec();
    for (i, id) in items.iter().enumerate() {
        let n = t.node(*id).unwrap();
        assert_eq!(a11y::role(n), Role::TreeItem);
        assert_eq!(n.level(), Some(1));
        assert_eq!(n.position_in_set(), Some(i + 1));
        assert_eq!(n.size_of_set(), Some(6));
        let merged = i == 4;
        assert_eq!(n.is_expanded(), merged.then_some(false), "item {i}");
    }
    assert_eq!(h.selected(), "Standard ReBirth, current");
    // only the selected item is selected
    for (i, id) in items.iter().enumerate() {
        let sel = t.node(*id).unwrap().is_selected();
        assert_eq!(sel, Some(i == 0), "item {i}");
    }
    // the focus is in the filter
    assert_eq!(h.focused(), (Role::TextInput, "Filter mods".to_string()));

    // Escape closes, and the focus goes back to the control that had it
    h.press(Key::Escape);
    assert!(!h.app.chooser_is_open());
    assert_eq!(h.tree().focused_id(), play.accesskit_id());
    assert!(h.tree().by_role(Role::Tree).next().is_none());
}

/// Typing narrows the list by name, ASCII case ignored, live; clearing brings it back.
#[test]
fn the_filter_narrows_the_list() {
    let dir = folder(&FILES);
    let mut h = H::new(true, Some(dir.path()));
    h.open_chooser();
    h.type_text("A");
    // anywhere in the name: alpha, Beta, Gamma Ray, Shared (Standard ReBirth has an a too)
    assert_eq!(
        h.items(),
        [
            "Standard ReBirth, current",
            "alpha",
            "Beta",
            "Gamma Ray",
            "Shared, variants: 2"
        ]
    );
    h.type_text("m");
    assert_eq!(h.items(), ["Gamma Ray"]);
    // the selection went to the first item that shows
    assert_eq!(h.selected(), "Gamma Ray");
    h.type_text("zzz");
    assert!(h.items().is_empty());
    // clear: Ctrl+A, Backspace
    h.press_with(Key::A, Modifiers::COMMAND);
    h.press(Key::Backspace);
    assert_eq!(h.items().len(), 6);
}

/// Enter in the list or the filter loads the kit the menu item loads, and closes the dialog.
#[test]
fn enter_loads_the_kit_of_the_menu_item() {
    let dir = folder(&FILES);
    let zed = path_of(dir.path(), "5.rbm");
    let mut menu = H::new(true, Some(dir.path()));
    menu.app.command(Cmd::UseKit(Kit::Mod(zed.clone())));
    assert_eq!(menu.app.doc.kit, Kit::Mod(zed.clone()));

    // from the filter
    let mut h = H::new(true, Some(dir.path()));
    h.open_chooser();
    h.type_text("zed");
    h.press(Key::Enter);
    assert!(!h.app.chooser_is_open());
    assert_eq!(h.app.doc.kit, menu.app.doc.kit);
    assert_eq!(h.app.doc.mod_name, menu.app.doc.mod_name);
    assert_eq!(h.app.doc.modified, menu.app.doc.modified);
    assert!(h.app.errors.is_empty(), "{:?}", h.app.errors);

    // from the list: Down to the list, Down to the entry, Enter
    let mut h = H::new(true, Some(dir.path()));
    h.open_chooser();
    h.press(Key::ArrowDown);
    assert_eq!(
        h.focused(),
        (Role::Tree, "Mods".to_string()),
        "Down in the filter moves to the list"
    );
    h.press(Key::End);
    assert_eq!(h.selected(), "Zed");
    h.press(Key::Enter);
    assert!(!h.app.chooser_is_open());
    assert_eq!(h.app.doc.kit, Kit::Mod(zed));

    // "Standard ReBirth" is the standard kit
    h.open_chooser();
    assert_eq!(h.selected(), "Zed, current");
    h.press(Key::ArrowDown);
    h.press(Key::Home);
    h.press(Key::Enter);
    assert_eq!(h.app.doc.kit, Kit::Standard);
    assert_eq!(h.app.doc.mod_name, STANDARD);
}

/// With a filter that matches nothing, Enter and Ctrl+I act on no hidden item.
#[test]
fn nothing_acts_on_a_hidden_item() {
    let dir = folder(&FILES);
    let mut h = H::new(true, Some(dir.path()));
    h.open_chooser();
    h.type_text("zed");
    h.type_text("zzz");
    assert!(h.items().is_empty());
    h.press(Key::Enter);
    assert!(h.app.chooser_is_open());
    assert_eq!(h.app.doc.kit, Kit::Standard);
    h.press_with(Key::I, Modifiers::COMMAND);
    assert!(!h.app.mod_info_is_open());
    assert_eq!(h.app.doc.kit, Kit::Standard);
}

/// Tab and Shift+Tab move between the two stops; Escape closes from either.
#[test]
fn two_stops_and_escape_from_either() {
    let dir = folder(&FILES);
    let mut h = H::new(true, Some(dir.path()));
    h.open_chooser();
    assert_eq!(h.focused().0, Role::TextInput);
    h.press(Key::Tab);
    assert_eq!(h.focused(), (Role::Tree, "Mods".to_string()));
    h.focus_filter();
    assert_eq!(h.focused().0, Role::TextInput);
    h.press(Key::Escape);
    assert!(!h.app.chooser_is_open());
    h.open_chooser();
    h.press(Key::Tab);
    assert_eq!(h.focused().0, Role::Tree);
    h.press(Key::Escape);
    assert!(!h.app.chooser_is_open());
}

/// The variants are a tree: Right opens an entry, Left closes it or goes to the parent; Enter
/// on the entry takes the preferred variant, on a variant it is the menu's variant item.
#[test]
fn variants_are_a_tree() {
    let dir = folder(&FILES);
    let a = path_of(dir.path(), "3.rbm");
    let b = path_of(dir.path(), "4.rbm");
    let mut h = H::new(true, Some(dir.path()));
    h.open_chooser();
    h.press(Key::ArrowDown);
    h.press(Key::End);
    h.press(Key::ArrowUp);
    assert_eq!(h.selected(), "Shared, variants: 2");
    h.press(Key::ArrowRight);
    let items = h.items();
    assert_eq!(items.len(), 8, "{items:?}");
    assert_eq!(items[5], "3.rbm  1.0.0");
    assert_eq!(items[6], "4.rbm  1.0.0");
    assert_eq!(
        h.selected(),
        "Shared, variants: 2",
        "Right opens it, no move"
    );
    {
        let t = h.tree();
        let (tid, _) = t.by_role(Role::Tree).next().unwrap();
        let kids = t.node(tid).unwrap().children().to_vec();
        let parent = t.node(kids[4]).unwrap();
        assert_eq!(parent.is_expanded(), Some(true));
        let child = t.node(kids[5]).unwrap();
        assert_eq!(
            (child.level(), child.position_in_set(), child.size_of_set()),
            (Some(2), Some(1), Some(2))
        );
        assert_eq!(child.is_expanded(), None);
    }
    // Right again: the first variant; Left: back to the entry; Left: closed
    h.press(Key::ArrowRight);
    assert_eq!(h.selected(), "3.rbm  1.0.0");
    h.press(Key::ArrowLeft);
    assert_eq!(h.selected(), "Shared, variants: 2");
    h.press(Key::ArrowLeft);
    assert_eq!(h.items().len(), 6);

    // Enter on the merged entry: the preferred (first by bytes) variant, as the menu's entry
    h.press(Key::Enter);
    assert!(!h.app.chooser_is_open());
    assert_eq!(h.app.doc.kit, Kit::Mod(a.clone()));

    // Enter on a variant: the menu's variant item
    let mut menu = H::new(true, Some(dir.path()));
    menu.app.command(Cmd::ChooseVariant(b.clone()));
    h.open_chooser();
    assert_eq!(h.selected(), "Shared, variants: 2, current");
    h.press(Key::ArrowDown);
    h.press(Key::ArrowRight);
    h.press(Key::ArrowRight);
    h.press(Key::ArrowDown);
    assert_eq!(h.selected(), "4.rbm  1.0.0");
    h.press(Key::Enter);
    assert_eq!(h.app.doc.kit, menu.app.doc.kit);
    assert_eq!(h.app.doc.kit, Kit::Mod(b.clone()));

    // the active variant chosen again sets the preference, as in the menu; the list says so
    h.open_chooser();
    h.press(Key::ArrowDown);
    h.press(Key::ArrowRight);
    h.press(Key::ArrowRight);
    h.press(Key::ArrowDown);
    assert_eq!(h.selected(), "4.rbm  1.0.0, current");
    h.press(Key::Enter);
    assert_eq!(h.app.settings.preferred_mod("Shared"), Some("4.rbm"));
    h.open_chooser();
    h.press(Key::ArrowDown);
    h.press(Key::ArrowRight);
    let items = h.items();
    assert_eq!(items[4], "Shared, variants: 2, current");
    assert_eq!(items[5], "3.rbm  1.0.0");
    assert_eq!(items[6], "4.rbm  1.0.0, preferred, current");
}

/// Ctrl+I opens Mod Info for the selection without loading it; Mods > Info shows the playing
/// kit as before; Escape closes Mod Info first, then the dialog.
#[test]
fn ctrl_i_shows_mod_info_of_the_selection() {
    let dir = folder(&FILES);
    let mut h = H::new(true, Some(dir.path()));
    h.open_chooser();
    h.type_text("zed");
    h.press_with(Key::I, Modifiers::COMMAND);
    assert!(h.app.mod_info_is_open());
    assert_eq!(h.app.doc.kit, Kit::Standard, "nothing was loaded");
    assert!(h.app.chooser_is_open());
    let spoken = |h: &H, line: &str| {
        h.tree()
            .nodes()
            .any(|(_, n)| a11y::value(n) == Some(line) || a11y::name(n) == Some(line))
    };
    assert!(spoken(&h, "Name: Zed"));
    assert!(spoken(&h, "File: 5.rbm"));

    // from the list, for a file of a merged entry
    h.press(Key::Escape);
    assert!(!h.app.mod_info_is_open());
    assert!(h.app.chooser_is_open(), "Escape closed Mod Info only");
    // the focus came back to the stop that had it (the filter)
    assert_eq!(h.focused(), (Role::TextInput, "Filter mods".to_string()));
    h.press(Key::Escape);
    assert!(!h.app.chooser_is_open());

    h.open_chooser();
    h.type_text("shared");
    h.press(Key::ArrowDown);
    h.press_with(Key::I, Modifiers::COMMAND);
    assert!(
        spoken(&h, "File: 3.rbm"),
        "the preferred variant of a merged entry"
    );
    h.press(Key::Escape);
    h.press(Key::ArrowRight);
    h.press(Key::ArrowRight);
    h.press(Key::ArrowDown);
    assert_eq!(h.selected(), "4.rbm  1.0.0");
    h.press_with(Key::I, Modifiers::COMMAND);
    assert!(spoken(&h, "File: 4.rbm"));
    h.press(Key::Escape);

    // the standard kit: the selection of a dialog opened on it
    h.press(Key::Escape);
    assert!(!h.app.chooser_is_open());
    h.open_chooser();
    h.press_with(Key::I, Modifiers::COMMAND);
    assert!(spoken(&h, "Name: Standard ReBirth"));
    h.press(Key::Escape);
    h.press(Key::Escape);
    assert!(!h.app.chooser_is_open());
    assert!(!h.app.mod_info_is_open());

    // Mods > Info is the playing kit, as before
    h.app.show_mod_info();
    h.settle(4);
    assert!(spoken(&h, "Name: Standard ReBirth"));
    assert_eq!(h.app.doc.kit, Kit::Standard);
}

/// The Mods menu's first item is Choose Mod... with its shortcut; Enter on it opens the dialog.
#[test]
fn the_menu_item_opens_the_dialog() {
    let dir = folder(&FILES);
    let mut h = H::new(true, Some(dir.path()));
    h.press(Key::F10);
    h.press(Key::ArrowRight);
    h.press(Key::ArrowDown);
    assert!(Popup::is_any_open(&h.f.ctx));
    assert_eq!(
        h.focused(),
        (Role::Button, "Choose Mod... Ctrl+M".to_string())
    );
    h.press(Key::Enter);
    assert!(h.app.chooser_is_open());
    assert_eq!(h.items().len(), 6);
}

/// With the mode off the app is as before: no menu item, Ctrl+M does nothing.
#[test]
fn with_the_mode_off_there_is_no_chooser() {
    let dir = folder(&FILES);
    let mut h = H::new(false, Some(dir.path()));
    h.open_chooser();
    assert!(!h.app.chooser_is_open());
    h.app.command(Cmd::ChooseMod);
    h.settle(3);
    assert!(!h.app.chooser_is_open());
    h.press(Key::F10);
    h.press(Key::ArrowRight);
    h.press(Key::ArrowDown);
    assert!(Popup::is_any_open(&h.f.ctx));
    assert_eq!(h.focused(), (Role::Button, "Standard ReBirth".to_string()));
    assert!(
        h.tree()
            .nodes()
            .all(|(_, n)| !a11y::name(n).unwrap_or("").starts_with("Choose Mod"))
    );
    assert!(h.tree().by_role(Role::Tree).next().is_none());
}

/// The dialog closes when the mode goes off under it.
#[test]
fn it_closes_when_the_mode_goes_off() {
    let dir = folder(&FILES);
    let mut h = H::new(true, Some(dir.path()));
    h.open_chooser();
    assert!(h.app.chooser_is_open());
    h.app.settings.screen_reader = ScreenReaderSetting::Off;
    h.settle(2);
    assert!(!h.app.chooser_is_open());
}

/// A screen reader's Focus on the tree and Click on an item (the mouse's way in).
#[test]
fn a_screen_reader_can_focus_the_tree() {
    let dir = folder(&FILES);
    let mut h = H::new(true, Some(dir.path()));
    h.open_chooser();
    let (tree, _) = h.app.chooser_ids(Item::Standard);
    h.frame(vec![a11y::request(tree, Action::Focus, None)]);
    h.settle(2);
    assert_eq!(h.focused(), (Role::Tree, "Mods".to_string()));
}

/// task-195 AC 2: with the rb-mods collection the dialog opens with all its entries in under
/// 100 ms after the list is cached. Needs `ORB_THIRDPARTY_MODS_DIR` (recrypted mods).
#[test]
fn the_collection_opens_in_under_100_ms() {
    let Some(dir) = orb_testkit::thirdparty_mods() else {
        eprintln!("skipped: ORB_THIRDPARTY_MODS_DIR is not set");
        return;
    };
    private_settings();
    let mut h = H::new(true, None);
    h.app.settings.mods_dir = Some(dir);
    let t = Instant::now();
    h.app.refresh_mods();
    let read = t.elapsed();
    let n = h.app.mods().unwrap().as_ref().unwrap().len();
    // a frame without the dialog, for the baseline
    let t = Instant::now();
    h.frame(vec![]);
    let base = t.elapsed();

    // Ctrl+M: the frame that opens it, then the frames that size and show it
    let m = Modifiers::COMMAND;
    let mut times = Vec::new();
    for ev in [
        vec![Event::ModifiersChanged(m), H::key_ev(Key::M, true, m)],
        vec![
            Event::ModifiersChanged(Modifiers::NONE),
            H::key_ev(Key::M, false, Modifiers::NONE),
        ],
        vec![],
        vec![],
        vec![],
    ] {
        let t = Instant::now();
        h.frame(ev);
        times.push(t.elapsed());
    }
    assert!(h.app.chooser_is_open());
    let rows = h.items();
    assert_eq!(rows.len(), n + 1, "Standard ReBirth and every entry");
    // an unfiltered frame with the dialog open, and a filtered one
    let t = Instant::now();
    h.frame(vec![]);
    let steady = t.elapsed();
    let t = Instant::now();
    h.type_text("a");
    let typed = t.elapsed();
    eprintln!(
        "{n} entries ({} items): list read {read:?}; frame without the dialog {base:?}; \
         frames from Ctrl+M {times:?}; steady frame {steady:?}; frame with a letter typed \
         (and 3 settle frames) {typed:?}",
        rows.len()
    );
    let open = times.iter().max().copied().unwrap();
    assert!(
        open.as_millis() < 100,
        "the dialog opens in {open:?}, not under 100 ms"
    );
}
