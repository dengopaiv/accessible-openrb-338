//! The menu bar from the keyboard (task-184, docs/accessibility_plan.md A11Y-11): F10 and a
//! lone Alt focus File, the arrows walk the bar and the items, Enter chooses, Escape closes.
//! What a screen reader hears is the focused node's role and name in the AccessKit tree.

#[path = "common/a11y.rs"]
mod a11y;

use a11y::AppTree;
use eframe::egui::accesskit::{Role, Toggled};
use eframe::egui::{Event, Key, Modifiers, Popup};

/// The role and name of the focused node.
fn focus(h: &AppTree) -> (Role, String) {
    let n = h.tree().focused().expect("a focused node");
    (a11y::role(n), a11y::name(n).unwrap_or("").to_string())
}

fn is(h: &AppTree, role: Role, name: &str) -> bool {
    focus(h) == (role, name.to_string())
}

/// Down on a bar button: the menu opens and egui moves the focus to its first item a frame
/// later; a window repaints by itself in between.
fn open_with_down(h: &mut AppTree) {
    h.key(Key::ArrowDown);
    h.frame(vec![]);
}

fn menu_open(h: &AppTree) -> bool {
    Popup::is_any_open(&h.frames.ctx)
}

#[test]
fn f10_focuses_file_right_moves_to_mods_down_opens_it_escape_closes() {
    let mut h = AppTree::new();
    assert!(!menu_open(&h));
    h.key(Key::F10);
    assert_eq!(focus(&h), (Role::Button, "File".to_string()));
    assert!(!menu_open(&h), "F10 only focuses");

    // upstream's Edit menu (merged by task-230) is the second title
    h.key(Key::ArrowRight);
    assert_eq!(focus(&h), (Role::Button, "Edit".to_string()));
    // and upstream's View (View > Classic / Modern, merged by task-233) the third
    h.key(Key::ArrowRight);
    assert_eq!(focus(&h), (Role::Button, "View".to_string()));
    h.key(Key::ArrowRight);
    assert_eq!(focus(&h), (Role::Button, "Mods".to_string()));
    assert!(!menu_open(&h));

    open_with_down(&mut h);
    assert!(menu_open(&h), "Down opens the menu");
    let (role, name) = focus(&h);
    assert_eq!(
        name, "Standard ReBirth",
        "the first item of Mods has the focus"
    );
    println!("first item of Mods: {role:?} {name:?}");

    h.key(Key::Escape);
    assert!(!menu_open(&h), "Escape closes");
    h.key(Key::Escape);
    assert!(!menu_open(&h));
}

#[test]
fn enter_on_a_title_opens_its_menu_and_keeps_the_focus_on_the_title() {
    let mut h = AppTree::new();
    h.key(Key::F10);
    h.key(Key::Enter);
    h.frame(vec![]);
    assert!(menu_open(&h));
    assert_eq!(focus(&h), (Role::Button, "File".to_string()));
}

#[test]
fn space_opens_a_title_and_left_of_file_does_not_wrap() {
    let mut h = AppTree::new();
    h.key(Key::F10);
    h.key(Key::ArrowLeft);
    assert_eq!(focus(&h), (Role::Button, "File".to_string()), "no wrap");
    h.key(Key::Space);
    h.frame(vec![]);
    assert!(menu_open(&h), "Space opens the menu");
    assert_eq!(focus(&h), (Role::Button, "File".to_string()));
}

#[test]
fn down_and_up_walk_the_items() {
    let mut h = AppTree::new();
    h.key(Key::F10);
    open_with_down(&mut h);
    assert!(menu_open(&h));
    assert!(is(&h, Role::Button, "New Ctrl+N"), "{:?}", focus(&h));
    h.key(Key::ArrowDown);
    assert!(is(&h, Role::Button, "Open... Ctrl+O"), "{:?}", focus(&h));
    h.key(Key::ArrowUp);
    assert!(is(&h, Role::Button, "New Ctrl+N"), "{:?}", focus(&h));
}

#[test]
fn enter_on_an_item_runs_it() {
    let mut h = AppTree::new();
    assert!(!h.app.key_help_is_open());
    h.key(Key::F10);
    // File, Edit, View, Mods, Options, Help
    for _ in 0..5 {
        h.key(Key::ArrowRight);
    }
    assert_eq!(focus(&h), (Role::Button, "Help".to_string()));
    open_with_down(&mut h);
    assert!(
        is(&h, Role::Button, "Keyboard Shortcuts..."),
        "{:?}",
        focus(&h)
    );
    h.key(Key::Enter);
    h.frame(vec![]);
    assert!(h.app.key_help_is_open(), "Enter chose the item");
    assert!(!menu_open(&h), "the menu closed");
    // the focus is not restored to the title
    let after = focus(&h);
    println!("focus after choosing an item: {after:?}");
    assert_eq!(after, (Role::Window, String::new()));
}

#[test]
fn a_lone_alt_focuses_file_and_alt_with_a_key_does_not() {
    let mut h = AppTree::new();
    h.frame(vec![Event::ModifiersChanged(Modifiers::ALT)]);
    h.frame(vec![Event::ModifiersChanged(Modifiers::NONE)]);
    h.frame(vec![]);
    assert_eq!(focus(&h), (Role::Button, "File".to_string()));

    // Alt+Tab-like use: a key between the press and the release is not a lone Alt
    let mut h = AppTree::new();
    h.frame(vec![Event::ModifiersChanged(Modifiers::ALT)]);
    h.frame(vec![Event::Key {
        key: Key::A,
        physical_key: Some(Key::A),
        pressed: true,
        repeat: false,
        modifiers: Modifiers::ALT,
    }]);
    h.frame(vec![Event::ModifiersChanged(Modifiers::NONE)]);
    h.frame(vec![]);
    assert_ne!(focus(&h), (Role::Button, "File".to_string()));
}

/// Options with the Options menu open, screen reader mode set to `mode`.
fn options_open(mode: orb_ui::screen_reader::ScreenReaderSetting) -> AppTree {
    let mut h = AppTree::new();
    h.app.settings.screen_reader = mode;
    h.frame(vec![]);
    h.key(Key::F10);
    for _ in 0..4 {
        h.key(Key::ArrowRight);
    }
    assert_eq!(focus(&h), (Role::Button, "Options".to_string()));
    open_with_down(&mut h);
    h
}

/// Task-192: "Speak Raw Values" and task-190: "Step Editor Follows Playback" are Options
/// items in screen reader mode only, both off by default.
#[test]
fn speak_raw_values_shows_in_screen_reader_mode_only() {
    use orb_ui::screen_reader::ScreenReaderSetting;
    for name in [
        "Speak Raw Values",
        "Step Editor Follows Playback",
        "Pitch Keys Make Notes",
    ] {
        let h = options_open(ScreenReaderSetting::Off);
        assert!(h.tree().by_name(name).next().is_none(), "{name}");
        let h = options_open(ScreenReaderSetting::On);
        let n = h
            .tree()
            .by_name(name)
            .next()
            .expect("the checkbox in screen reader mode")
            .1;
        assert_eq!(a11y::role(n), Role::CheckBox, "{name}");
        assert_eq!(a11y::toggled(n), Some(Toggled::False), "{name}");
    }
}

#[test]
fn the_options_items_read_their_state() {
    let mut h = AppTree::new();
    h.key(Key::F10);
    for _ in 0..4 {
        h.key(Key::ArrowRight);
    }
    assert_eq!(focus(&h), (Role::Button, "Options".to_string()));
    open_with_down(&mut h);
    let t = h.tree();
    let vintage = t
        .by_name("Vintage ReBirth Synth Sound")
        .next()
        .expect("the checkbox is in the tree")
        .1;
    assert_eq!(a11y::role(vintage), Role::CheckBox);
    assert_eq!(a11y::toggled(vintage), Some(Toggled::False));
    // the menu's radio buttons, not the panel's (the pattern, bank, PCF and Comp buttons are
    // radio buttons too since task-186)
    let panel: std::collections::HashSet<_> = orb_ui::layout::controls()
        .iter()
        .map(|c| a11y::hit_id(&c.kind, &c.id).accesskit_id())
        .collect();
    let radios: Vec<_> = t
        .by_role(Role::RadioButton)
        .filter(|(id, _)| !panel.contains(id))
        .map(|(_, n)| (a11y::name(n).unwrap_or("").to_string(), a11y::toggled(n)))
        .collect();
    println!("radios {radios:?}");
    assert_eq!(radios.len(), 2);
    assert!(radios.iter().any(|(_, t)| *t == Some(Toggled::True)));
    assert!(radios.iter().any(|(_, t)| *t == Some(Toggled::False)));
}

// ---- task-212: the focus returns where it was when the bar is left ----------------------

fn play_id() -> eframe::egui::Id {
    a11y::hit_id("trigger", "transport.play")
}

/// An app in screen reader mode with the focus on the Play button.
fn on_play() -> AppTree {
    let mut h = AppTree::new();
    h.app.settings.screen_reader = orb_ui::screen_reader::ScreenReaderSetting::On;
    h.frame(vec![]);
    h.frames.ctx.memory_mut(|m| m.request_focus(play_id()));
    for _ in 0..3 {
        h.frame(vec![]);
    }
    assert!(has_play(&h), "{:?}", focus(&h));
    h
}

fn has_play(h: &AppTree) -> bool {
    h.tree().focused_id() == play_id().accesskit_id()
}

/// A lone Alt: press and release.
fn lone_alt(h: &mut AppTree) {
    h.frame(vec![Event::ModifiersChanged(Modifiers::ALT)]);
    h.frame(vec![Event::ModifiersChanged(Modifiers::NONE)]);
    h.frame(vec![]);
}

#[test]
fn escape_on_a_title_gives_the_focus_back() {
    let mut h = on_play();
    h.key(Key::F10);
    assert_eq!(focus(&h), (Role::Button, "File".to_string()));
    h.key(Key::Escape);
    h.frame(vec![]);
    assert!(has_play(&h), "{:?}", focus(&h));

    let mut h = on_play();
    lone_alt(&mut h);
    assert_eq!(focus(&h), (Role::Button, "File".to_string()));
    h.key(Key::Escape);
    h.frame(vec![]);
    assert!(has_play(&h), "{:?}", focus(&h));
}

#[test]
fn escape_in_an_open_menu_gives_the_focus_back() {
    for alt in [false, true] {
        let mut h = on_play();
        if alt {
            lone_alt(&mut h);
        } else {
            h.key(Key::F10);
        }
        open_with_down(&mut h);
        assert!(menu_open(&h));
        h.key(Key::Escape);
        h.frame(vec![]);
        println!("alt {alt}, after Escape in the menu: {:?}", focus(&h));
        // one Escape closes the menu and gives the focus back
        assert!(!menu_open(&h));
        assert!(has_play(&h), "{:?}", focus(&h));
    }
}

#[test]
fn a_second_f10_or_alt_gives_the_focus_back() {
    // on a title
    let mut h = on_play();
    h.key(Key::F10);
    h.key(Key::F10);
    h.frame(vec![]);
    assert!(has_play(&h), "{:?}", focus(&h));
    let mut h = on_play();
    lone_alt(&mut h);
    lone_alt(&mut h);
    assert!(has_play(&h), "{:?}", focus(&h));
    // with a menu open
    let mut h = on_play();
    h.key(Key::F10);
    open_with_down(&mut h);
    assert!(menu_open(&h));
    h.key(Key::F10);
    h.frame(vec![]);
    assert!(!menu_open(&h), "the menu closed");
    assert!(has_play(&h), "{:?}", focus(&h));
    let mut h = on_play();
    lone_alt(&mut h);
    open_with_down(&mut h);
    assert!(menu_open(&h));
    lone_alt(&mut h);
    assert!(!menu_open(&h), "the menu closed");
    assert!(has_play(&h), "{:?}", focus(&h));
    // and F10 asks for the bar again afterwards
    h.key(Key::F10);
    assert_eq!(focus(&h), (Role::Button, "File".to_string()));
}

#[test]
fn an_item_that_opens_no_window_gives_the_focus_back() {
    let mut h = on_play();
    assert!(!h.app.settings.step_editor_follows);
    h.key(Key::F10);
    for _ in 0..4 {
        h.key(Key::ArrowRight);
    }
    open_with_down(&mut h);
    // past Vintage, Startup Sound, upstream's Audio Output... (task-233), Screen Reader Mode and
    // Speak Raw Values
    for _ in 0..5 {
        h.key(Key::ArrowDown);
    }
    println!("on: {:?}", focus(&h));
    assert!(
        is(&h, Role::CheckBox, "Step Editor Follows Playback"),
        "{:?}",
        focus(&h)
    );
    h.key(Key::Enter);
    for _ in 0..3 {
        h.frame(vec![]);
    }
    assert!(h.app.settings.step_editor_follows, "the item ran");
    assert!(!menu_open(&h));
    assert!(has_play(&h), "{:?}", focus(&h));
}

#[test]
fn an_item_that_opens_a_window_returns_to_the_control_when_it_closes() {
    let mut h = on_play();
    h.key(Key::F10);
    for _ in 0..5 {
        h.key(Key::ArrowRight);
    }
    open_with_down(&mut h);
    h.key(Key::Enter);
    for _ in 0..4 {
        h.frame(vec![]);
    }
    assert!(h.app.key_help_is_open());
    assert!(!has_play(&h));
    println!("in the window: {:?}", focus(&h));
    assert_ne!(
        focus(&h),
        (Role::Window, String::new()),
        "the focus is inside"
    );
    h.key(Key::Escape);
    for _ in 0..4 {
        h.frame(vec![]);
    }
    assert!(!h.app.key_help_is_open(), "Escape closed the window");
    assert!(has_play(&h), "{:?}", focus(&h));
}

// ---- task-247: Left and Right between the menus, as in Windows ----------------------------

const TITLES: [&str; 6] = ["File", "Edit", "View", "Mods", "Options", "Help"];

fn sr_app() -> AppTree {
    let mut h = AppTree::new();
    h.app.settings.screen_reader = orb_ui::screen_reader::ScreenReaderSetting::On;
    h.frame(vec![]);
    h
}

fn title(h: &AppTree, name: &str) -> bool {
    is(h, Role::Button, name)
}

/// The menu `menu` is open with the focus on its first item. Edit is the demo panel here (no
/// player link), so all its items are disabled and the focus stays on the title.
fn on_first_item(h: &AppTree, menu: &str) -> bool {
    let first = match menu {
        "File" => "New Ctrl+N",
        "View" => "Classic",
        "Mods" => "Choose Mod... Ctrl+M",
        "Options" => "Vintage ReBirth Synth Sound",
        "Help" => "Keyboard Shortcuts...",
        _ => return menu_open(h) && title(h, menu),
    };
    menu_open(h) && focus(h).1 == first
}

#[test]
fn left_and_right_on_a_title_wrap_and_never_leave_the_bar() {
    let mut h = sr_app();
    h.key(Key::F10);
    assert!(title(&h, "File"));
    for t in &TITLES[1..] {
        h.key(Key::ArrowRight);
        assert!(title(&h, t), "{:?}", focus(&h));
    }
    // Right from Help: File, not the Transport section
    h.key(Key::ArrowRight);
    assert!(title(&h, "File"), "{:?}", focus(&h));
    assert!(!menu_open(&h));
    // Left from File: Help; and back through all six
    h.key(Key::ArrowLeft);
    assert!(title(&h, "Help"), "{:?}", focus(&h));
    for t in TITLES[..5].iter().rev() {
        h.key(Key::ArrowLeft);
        assert!(title(&h, t), "{:?}", focus(&h));
    }
}

#[test]
fn left_and_right_in_an_open_menu_open_the_neighbor_with_its_first_item() {
    let mut h = sr_app();
    h.key(Key::F10);
    open_with_down(&mut h);
    assert!(on_first_item(&h, "File"), "{:?}", focus(&h));
    // right through all six menus and round to File
    for t in TITLES[1..].iter().chain(&TITLES[..1]) {
        h.key(Key::ArrowRight);
        h.frame(vec![]);
        assert!(on_first_item(&h, t), "{t}: {:?}", focus(&h));
    }
    // left: File -> Help, and back
    h.key(Key::ArrowLeft);
    h.frame(vec![]);
    assert!(on_first_item(&h, "Help"), "{:?}", focus(&h));
    for t in TITLES[..5].iter().rev() {
        h.key(Key::ArrowLeft);
        h.frame(vec![]);
        assert!(on_first_item(&h, t), "{t}: {:?}", focus(&h));
    }
    // after a walk down the items, Right still goes to the next menu's first item
    h.key(Key::ArrowDown);
    h.key(Key::ArrowRight);
    h.frame(vec![]);
    assert!(on_first_item(&h, "Edit"), "{:?}", focus(&h));
    h.key(Key::Escape);
    assert!(!menu_open(&h));
}

#[test]
fn right_from_the_help_menu_goes_to_file_not_to_the_panel() {
    let mut h = sr_app();
    h.key(Key::F10);
    for _ in 0..5 {
        h.key(Key::ArrowRight);
    }
    open_with_down(&mut h);
    assert!(on_first_item(&h, "Help"), "{:?}", focus(&h));
    h.key(Key::ArrowRight);
    h.frame(vec![]);
    assert!(on_first_item(&h, "File"), "{:?}", focus(&h));
}

#[test]
fn with_the_mode_off_the_bar_keys_are_eguis() {
    let mut h = AppTree::new();
    h.key(Key::F10);
    for _ in 0..5 {
        h.key(Key::ArrowRight);
    }
    assert!(title(&h, "Help"));
    h.key(Key::ArrowRight);
    assert!(!title(&h, "File"), "no wrap: {:?}", focus(&h));
    assert!(
        !title(&h, "Help"),
        "the focus left the bar: {:?}",
        focus(&h)
    );
}

#[test]
fn escape_after_moves_between_menus_gives_the_focus_back() {
    let mut h = on_play();
    h.key(Key::F10);
    open_with_down(&mut h);
    h.key(Key::ArrowRight);
    h.frame(vec![]);
    h.key(Key::ArrowRight);
    h.frame(vec![]);
    assert!(menu_open(&h));
    h.key(Key::Escape);
    h.frame(vec![]);
    assert!(!menu_open(&h));
    assert!(has_play(&h), "{:?}", focus(&h));
}

/// Left alone wraps: File to Help on the title, and in File's open menu to Help's first item.
#[test]
fn left_from_file_wraps_to_help() {
    let mut h = sr_app();
    h.key(Key::F10);
    h.key(Key::ArrowLeft);
    assert!(title(&h, "Help"), "{:?}", focus(&h));
    assert!(!menu_open(&h));

    let mut h = sr_app();
    h.key(Key::F10);
    open_with_down(&mut h);
    assert!(on_first_item(&h, "File"), "{:?}", focus(&h));
    h.key(Key::ArrowLeft);
    h.frame(vec![]);
    assert!(on_first_item(&h, "Help"), "{:?}", focus(&h));
}
