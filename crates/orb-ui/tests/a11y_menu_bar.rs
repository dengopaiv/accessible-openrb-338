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
