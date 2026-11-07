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
    for _ in 0..3 {
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
    h.key(Key::ArrowRight);
    h.key(Key::ArrowRight);
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
    h.key(Key::ArrowRight);
    h.key(Key::ArrowRight);
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
    for _ in 0..2 {
        h.key(Key::ArrowRight);
    }
    open_with_down(&mut h);
    for _ in 0..4 {
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
    for _ in 0..3 {
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
