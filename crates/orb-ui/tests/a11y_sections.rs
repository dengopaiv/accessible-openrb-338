//! The sections, the focus order and the keys of screen reader mode (task-188, A11Y-4,
//! docs/accessibility.md 3.2 and 3.4, docs/ui_spec.md section 10 "Screen reader mode").
//!
//! Where a key stands for a button, the same change is made on a second app by mouse (the
//! panel at 1:1, synthetic pointer events, as `tests/interaction.rs`), and the engine events
//! (`Panel::events`) must be equal.

#[path = "common/a11y.rs"]
mod a11y;

use a11y::AppTree;
use eframe::egui::accesskit::{Action, NodeId, Role};
use eframe::egui::{Event, Id, Key, Modifiers, PointerButton, Pos2};
use orb_ui::a11y::{drive, names, nav};
use orb_ui::keys::KeyMode;
use orb_ui::layout::{self, Ctl};
use orb_ui::model::Event as Out;
use orb_ui::screen_reader::ScreenReaderSetting;

fn ctl(kind: &str, id: &str) -> Ctl {
    layout::controls()
        .into_iter()
        .find(|c| c.id == id && c.kind == kind)
        .unwrap_or_else(|| panic!("{kind} {id}"))
}

/// The app with screen reader mode on or off, with the menu bar (`menu`) or without (then
/// panel pixels are points, for the mouse).
fn app(screen_reader: bool, menu: bool) -> AppTree {
    let mut h = AppTree::new();
    h.app.menu = menu;
    h.app.settings.screen_reader = if screen_reader {
        ScreenReaderSetting::On
    } else {
        ScreenReaderSetting::Off
    };
    h.frame(vec![]);
    h.app.panel.events.clear();
    h
}

fn take(h: &mut AppTree) -> Vec<Out> {
    std::mem::take(&mut h.app.panel.events)
}

fn key_event(key: Key, pressed: bool, repeat: bool, modifiers: Modifiers) -> Event {
    Event::Key {
        key,
        physical_key: Some(key),
        pressed,
        repeat,
        modifiers,
    }
}

/// Presses and releases a key with modifiers in two frames.
fn key_with(h: &mut AppTree, key: Key, modifiers: Modifiers) {
    h.frame(vec![key_event(key, true, false, modifiers)]);
    h.frame(vec![key_event(key, false, false, modifiers)]);
}

fn click(h: &mut AppTree, kind: &str, id: &str) {
    let c = ctl(kind, id);
    let p = Pos2::new(c.x + c.w / 2.0, c.y + c.h / 2.0);
    let button = |pressed| Event::PointerButton {
        pos: p,
        button: PointerButton::Primary,
        pressed,
        modifiers: Modifiers::NONE,
    };
    h.frame(vec![Event::PointerMoved(p)]);
    h.frame(vec![button(true)]);
    h.frame(vec![button(false)]);
}

fn focus(h: &mut AppTree, id: Id) {
    h.send(id, Action::Focus, None);
    assert_eq!(h.tree().focused_id(), id.accesskit_id());
}

fn focused(h: &AppTree) -> NodeId {
    h.tree().focused_id()
}

fn name_of(h: &AppTree, id: NodeId) -> String {
    let n = h.tree().node(id).expect("the node");
    // a label's text is its value (UI Automation names a Label from it)
    a11y::name(n)
        .or(a11y::value(n))
        .unwrap_or_default()
        .to_string()
}

/// The name of the section group the node is in.
fn group_of(h: &AppTree, id: NodeId) -> String {
    let (_, g) = h.tree().parent(id).expect("a parent");
    assert_eq!(a11y::role(g), Role::Group);
    a11y::name(g).unwrap_or_default().to_string()
}

/// The spoken name of a stop of the focus order.
fn stop_name(id: Id) -> String {
    if (0..4).any(|r| drive::selector_id(r) == id) {
        return "Pattern".into();
    }
    if (0..2).any(|k| orb_ui::a11y::step303::editor_id(k) == id) {
        return orb_ui::a11y::step303::EDITOR_NAME.into();
    }
    if orb_ui::a11y::grid::machine_of(id).is_some() {
        return "Steps".into();
    }
    let c = layout::controls()
        .into_iter()
        .find(|c| nav::focusable(c) && orb_ui::a11y::hit_id(c) == id)
        .expect("a control of the panel");
    names::name(&c).expect("a name")
}

#[test]
fn the_sections_are_named_groups_in_screen_reader_mode_only() {
    let a = app(true, true);
    let t = a.tree();
    let (first, _) = t.by_role(Role::Group).next().expect("a group");
    let (_, parent) = t.parent(first).expect("the groups' parent");
    let groups: Vec<String> = parent
        .children()
        .iter()
        .filter_map(|&id| t.node(id))
        .filter(|n| a11y::role(n) == Role::Group)
        .filter_map(|n| a11y::name(n).map(str::to_string))
        .collect();
    assert_eq!(
        groups,
        nav::SECTIONS,
        "in the order of docs/accessibility.md 3.2"
    );
    // every control of the panel is in its section's group, the 303's edit step label too
    for c in layout::controls() {
        if !nav::focusable(&c) && c.kind != "display" {
            continue;
        }
        let id = orb_ui::a11y::hit_id(&c).accesskit_id();
        assert_eq!(
            group_of(&a, id),
            names::section_name(&c.section).unwrap(),
            "{}",
            c.id
        );
    }
    for r in 0..4 {
        let id = drive::selector_id(r).accesskit_id();
        assert_eq!(group_of(&a, id), nav::SECTIONS[r + 1]);
    }

    let off = app(false, true);
    assert_eq!(
        off.tree().by_role(Role::Group).count(),
        0,
        "mode off: as before"
    );
}

#[test]
fn f6_visits_the_sections_in_order_and_wraps() {
    let mut a = app(true, true);
    let order = nav::order(&layout::controls());
    let first = |s: usize| order.iter().find(|x| x.section == s).unwrap().id;
    let mut rows = vec![];
    for step in 0..=nav::SECTIONS.len() {
        let s = step % nav::SECTIONS.len();
        a.key(Key::F6);
        let f = focused(&a);
        assert_eq!(f, first(s).accesskit_id(), "F6 #{step}");
        assert_eq!(name_of(&a, f), stop_name(first(s)));
        assert_eq!(group_of(&a, f), nav::SECTIONS[s]);
        rows.push(a.app.panel.focus);
    }
    assert_eq!(
        name_of(&a, focused(&a)),
        "Play",
        "the transport begins with Play"
    );
    // the row focus follows the machines' sections and stays elsewhere
    assert_eq!(rows, [0, 0, 1, 2, 3, 3, 3, 3, 3, 3]);
    // Shift+F6 goes back, wrapping
    key_with(&mut a, Key::F6, Modifiers::SHIFT);
    assert_eq!(focused(&a), first(8).accesskit_id());
    assert_eq!(group_of(&a, focused(&a)), "Distortion");
    key_with(&mut a, Key::F6, Modifiers::SHIFT);
    assert_eq!(group_of(&a, focused(&a)), "Delay");
    assert!(take(&mut a).is_empty(), "moving sends nothing");
}

#[test]
fn tab_visits_the_controls_of_the_panel_in_section_order() {
    let mut a = app(true, true);
    let order = nav::order(&layout::controls());
    // 186 controls, 4 pattern selectors, the 2 303 step editors (task-190), the 2 drum grids
    // (task-191)
    assert_eq!(order.len(), 194, "the panel's focusable controls");
    let mut sections = vec![];
    for (i, stop) in order.iter().enumerate() {
        a.key(Key::Tab);
        let f = focused(&a);
        assert_eq!(f, stop.id.accesskit_id(), "Tab #{i}");
        assert_eq!(name_of(&a, f), stop_name(stop.id), "Tab #{i}");
        assert_eq!(group_of(&a, f), nav::SECTIONS[stop.section]);
        if sections.last() != Some(&stop.section) {
            sections.push(stop.section);
        }
    }
    assert_eq!(sections, (0..9).collect::<Vec<_>>(), "section by section");
    // the end of the panel wraps to its start, not into the menu bar
    a.key(Key::Tab);
    assert_eq!(focused(&a), order[0].id.accesskit_id());
    key_with(&mut a, Key::Tab, Modifiers::SHIFT);
    assert_eq!(focused(&a), order[order.len() - 1].id.accesskit_id());
    key_with(&mut a, Key::Tab, Modifiers::SHIFT);
    assert_eq!(focused(&a), order[order.len() - 2].id.accesskit_id());
    // auto-repeat moves on
    a.frame(vec![key_event(Key::Tab, true, false, Modifiers::NONE)]);
    a.frame(vec![key_event(Key::Tab, true, true, Modifiers::NONE)]);
    a.frame(vec![key_event(Key::Tab, false, false, Modifiers::NONE)]);
    assert_eq!(focused(&a), order[0].id.accesskit_id());
    assert!(take(&mut a).is_empty(), "moving sends nothing");
}

#[test]
fn tab_moves_the_focus_while_a_303_plays_and_ctrl_tab_taps() {
    let (mut m, mut a) = (app(false, true), app(true, true));
    for h in [&mut m, &mut a] {
        h.app.panel.playing_step303 = [Some(3), None];
        h.app.panel.focus = 0;
    }
    let before = focused(&a);
    a.key(Key::Tab);
    assert_ne!(focused(&a), before, "Tab moved the focus");
    assert!(take(&mut a).is_empty(), "and wrote nothing");
    // Ctrl+Tab is the original's Tab, Ctrl+Shift+Tab its Shift+Tab
    m.key(Key::Tab);
    key_with(&mut a, Key::Tab, Modifiers::CTRL);
    let (em, ea) = (take(&mut m), take(&mut a));
    assert_eq!(em.len(), 1, "{em:?}");
    assert_eq!(ea, em, "a note");
    key_with(&mut m, Key::Tab, Modifiers::SHIFT);
    key_with(&mut a, Key::Tab, Modifiers::CTRL | Modifiers::SHIFT);
    let (em, ea) = (take(&mut m), take(&mut a));
    assert_eq!(em.len(), 1, "{em:?}");
    assert_eq!(ea, em, "a rest");
    assert_eq!(a.app.panel.focus, 0);
}

#[test]
fn f5_f9_f11_and_f12_are_the_transport_buttons() {
    let (mut m, mut a) = (app(false, false), app(true, false));
    for (key, kind, id) in [
        (Key::F12, "switch", "transport.mode"),
        (Key::F9, "toggle", "transport.record"),
        (Key::F11, "toggle", "transport.loop"),
        (Key::F5, "trigger", "transport.play"),
        (Key::F5, "trigger", "transport.stop"),
        (Key::F9, "toggle", "transport.record"),
        (Key::F12, "switch", "transport.mode"),
    ] {
        click(&mut m, kind, id);
        a.key(key);
        let (em, ea) = (take(&mut m), take(&mut a));
        assert!(!em.is_empty(), "{id} sends");
        assert_eq!(ea, em, "{key:?} = {id}");
    }
}

#[test]
fn space_presses_the_focused_control_and_does_not_play() {
    let (mut m, mut a) = (app(false, false), app(true, false));
    for (kind, id) in [
        ("toggle", "transport.loop"),
        ("toggle", "mixer2.mute"),
        ("hswitch", "tb303_1.wave"),
        ("radio", "mixer3.pcf"),
        ("trigger", "tb303_2.key3"),
    ] {
        focus(&mut a, a11y::hit_id(kind, id));
        take(&mut a);
        click(&mut m, kind, id);
        a.key(Key::Space);
        let (em, ea) = (take(&mut m), take(&mut a));
        assert!(!em.is_empty(), "{id} sends");
        assert_eq!(ea, em, "Space on {id}");
        assert!(!a.app.panel.transport.playing);
    }
    // a knob takes no press; nothing plays
    focus(&mut a, a11y::hit_id("knob", "tb303_1.cutoff"));
    take(&mut a);
    a.key(Key::Space);
    assert!(take(&mut a).is_empty());
}

#[test]
fn enter_keeps_the_original_meaning_whatever_has_the_focus() {
    let (mut m, mut a) = (app(false, false), app(true, false));
    // a 303's control focused in Program Synth mode: its Step
    focus(&mut a, a11y::hit_id("knob", "tb303_2.cutoff"));
    assert_eq!(a.app.panel.focus, 1, "the section gave the row focus");
    take(&mut a);
    let step = |h: &AppTree| h.app.panel.display("tb303_2.edit_step");
    let before = step(&a);
    click(&mut m, "trigger", "tb303_2.step");
    a.key(Key::Enter);
    let (em, ea) = (take(&mut m), take(&mut a));
    assert_eq!(ea, em, "Enter is the 303's Step");
    assert_ne!(step(&a), before, "the edit step moved");
    assert_eq!(step(&a), step(&m));
    assert!(!a.app.panel.transport.playing, "and nothing plays");
    // the original's keys keep working with the focus on any control: a pitch key
    m.app.panel.focus = 1;
    m.key(Key::V);
    a.key(Key::V);
    let (em, ea) = (take(&mut m), take(&mut a));
    assert!(!em.is_empty());
    assert_eq!(ea, em, "V is the 303's D");
    // a drum machine's section: Enter plays
    focus(&mut a, a11y::hit_id("knob", "tr808.bd_level"));
    take(&mut a);
    a.key(Key::Enter);
    assert!(a.app.panel.transport.playing);
    a.key(Key::F5);
    assert!(!a.app.panel.transport.playing);
    // Select Patterns mode: the pattern keys, whatever has the focus
    a.app.key_mode = KeyMode::Patterns;
    m.app.key_mode = KeyMode::Patterns;
    take(&mut a);
    m.key(Key::Num3);
    a.key(Key::Num3);
    let (em, ea) = (take(&mut m), take(&mut a));
    assert!(!em.is_empty());
    assert_eq!(ea, em, "3 selects pattern 3 of the first 303");
}

#[test]
fn ctrl_up_and_down_move_the_row_focus_without_the_focus() {
    let mut a = app(true, true);
    let cutoff = a11y::hit_id("knob", "tb303_1.cutoff");
    focus(&mut a, cutoff);
    assert_eq!(a.app.panel.focus, 0);
    key_with(&mut a, Key::ArrowDown, Modifiers::CTRL);
    assert_eq!(a.app.panel.focus, 1);
    key_with(&mut a, Key::ArrowUp, Modifiers::CTRL);
    key_with(&mut a, Key::ArrowUp, Modifiers::CTRL);
    a.frame(vec![]);
    assert_eq!(
        a.app.panel.focus, 3,
        "wraps, and the focus does not undo it"
    );
    assert_eq!(focused(&a), cutoff.accesskit_id());
    // entering another section gives its machine the row again
    a.key(Key::F6);
    assert_eq!(a.app.panel.focus, 1);
}

#[test]
fn the_arrows_do_not_move_the_focus_on_the_panel() {
    let mut a = app(true, true);
    let loop_ = a11y::hit_id("toggle", "transport.loop");
    focus(&mut a, loop_);
    let row = a.app.panel.focus;
    take(&mut a);
    for k in [
        Key::ArrowUp,
        Key::ArrowDown,
        Key::ArrowLeft,
        Key::ArrowRight,
    ] {
        a.key(k);
        assert_eq!(focused(&a), loop_.accesskit_id(), "{k:?}");
    }
    assert_eq!(a.app.panel.focus, row, "Up and Down are not the row focus");
    assert!(take(&mut a).is_empty());
}

#[test]
fn f1_opens_the_keyboard_shortcuts_in_screen_reader_mode() {
    let mut a = app(true, true);
    let cutoff = a11y::hit_id("knob", "tb303_1.cutoff");
    focus(&mut a, cutoff);
    a.key(Key::F1);
    // (the window's first frame is egui's sizing pass: the focus moves in at its second)
    a.frame(vec![]);
    assert!(a.app.key_help_is_open());
    // the focus went into the window, to its first line, not the panel's
    let order = nav::order(&layout::controls());
    let f = focused(&a);
    assert!(
        order.iter().all(|s| s.id.accesskit_id() != f),
        "not the panel"
    );
    assert!(
        name_of(&a, f).starts_with("Keyboard mode now"),
        "{}",
        name_of(&a, f)
    );
    // Tab stays in the window: the next line, not the panel's order (every line of the
    // window takes the focus, task-193)
    let first = f;
    a.key(Key::Tab);
    a.frame(vec![]);
    assert_ne!(focused(&a), first, "Tab moves to the next line");
    let second = focused(&a);
    assert!(
        order.iter().all(|s| s.id.accesskit_id() != second),
        "in the window"
    );
    // Shift+Tab goes back into the title bar, to its close button
    // (from the second line: back over the first, then to the title bar)
    key_with(&mut a, Key::Tab, Modifiers::SHIFT);
    assert_eq!(focused(&a), first, "Shift+Tab goes back a line");
    key_with(&mut a, Key::Tab, Modifiers::SHIFT);
    key_with(&mut a, Key::Tab, Modifiers::SHIFT);
    let f = focused(&a);
    assert!(
        order.iter().all(|s| s.id.accesskit_id() != f),
        "not the panel"
    );
    assert_eq!(
        a11y::role(a.tree().node(f).expect("the node")),
        Role::Button
    );
    // Space is egui's click of that button: the window closes, nothing plays
    a.key(Key::Space);
    a.frame(vec![]);
    assert!(!a.app.key_help_is_open(), "the close button closed it");
    assert!(!a.app.panel.transport.playing);
    assert_eq!(focused(&a), cutoff.accesskit_id(), "the focus is back");
    // Escape closes it and the focus is back where it was
    a.key(Key::F1);
    a.frame(vec![]);
    assert_ne!(focused(&a), cutoff.accesskit_id());
    a.key(Key::Escape);
    a.frame(vec![]);
    assert!(!a.app.key_help_is_open());
    assert_eq!(focused(&a), cutoff.accesskit_id(), "the focus is back");
    let mut m = app(false, true);
    m.key(Key::F1);
    assert!(
        !m.app.key_help_is_open(),
        "mode off: F1 does nothing, as before"
    );
}

/// File > Info opened with the focus on the panel: the focus goes to its first text field, and
/// Tab is egui's there (the next field), not the panel's order.
#[test]
fn tab_works_in_a_window_opened_from_the_panel() {
    let mut a = app(true, true);
    let play = a11y::hit_id("trigger", "transport.play");
    focus(&mut a, play);
    a.app.show_info();
    a.frame(vec![]);
    a.frame(vec![]);
    let order = nav::order(&layout::controls());
    let in_window = |a: &AppTree| {
        let f = focused(a);
        order.iter().all(|s| s.id.accesskit_id() != f)
    };
    assert!(in_window(&a), "the focus went into the window");
    let first = focused(&a);
    assert_eq!(a11y::role(a.tree().node(first).unwrap()), Role::TextInput);
    a.key(Key::Tab);
    assert!(in_window(&a), "Tab stays in the window");
    assert_ne!(focused(&a), first, "and moves");
    a.key(Key::Escape);
    a.frame(vec![]);
    assert!(!a.app.info_is_open());
    assert_eq!(focused(&a), play.accesskit_id(), "the focus is back");
}

/// Each window of the panel takes the focus as it opens, on a named node of its own, and
/// gives it back as it closes.
#[test]
fn every_window_takes_the_focus_and_gives_it_back() {
    let order = nav::order(&layout::controls());
    type Open = fn(&mut orb_ui::app::App);
    let opens: [(&str, Open); 4] = [
        ("keys", |a| a.show_key_help()),
        ("info", |a| a.show_info()),
        ("about", |a| a.show_about()),
        ("mod info", |a| a.show_mod_info()),
    ];
    for (what, open) in opens {
        let mut a = app(true, true);
        let play = a11y::hit_id("trigger", "transport.play");
        focus(&mut a, play);
        open(&mut a.app);
        a.frame(vec![]);
        a.frame(vec![]);
        let f = focused(&a);
        assert!(order.iter().all(|s| s.id.accesskit_id() != f), "{what}");
        let n = a.tree().node(f).expect("the node");
        println!("{what}: {:?} {:?}", a11y::role(n), name_of(&a, f));
        let named = !name_of(&a, f).is_empty() || a11y::role(n) == Role::TextInput;
        assert!(named, "{what}: a named node or a text field");
        a.key(Key::Escape);
        a.frame(vec![]);
        assert_eq!(
            focused(&a),
            play.accesskit_id(),
            "{what}: the focus is back"
        );
    }
}

/// With the focus in a window, the panel's keys wait (as for the menus, TASK-202): Enter,
/// Space, a pitch key and Up change nothing on the panel; the mode's F keys still work.
#[test]
fn the_panel_keys_wait_while_a_window_has_the_focus() {
    let mut a = app(true, true);
    focus(&mut a, a11y::hit_id("knob", "tb303_1.cutoff"));
    a.app.show_about();
    a.frame(vec![]);
    a.frame(vec![]);
    let f = focused(&a);
    assert_eq!(
        a11y::role(a.tree().node(f).unwrap()),
        Role::Label,
        "About's first line, the version"
    );
    take(&mut a);
    let step = a.app.panel.display("tb303_1.edit_step");
    let cutoff = a.app.panel.value("tb303_1.cutoff");
    for k in [Key::Enter, Key::Space, Key::V, Key::ArrowUp, Key::Insert] {
        a.key(k);
        assert_eq!(focused(&a), f, "{k:?}: the focus stays on the link");
    }
    key_with(&mut a, Key::Tab, Modifiers::CTRL);
    key_with(&mut a, Key::ArrowDown, Modifiers::CTRL);
    assert!(take(&mut a).is_empty(), "no engine event");
    assert!(!a.app.panel.transport.playing);
    assert_eq!(a.app.panel.focus, 0);
    assert_eq!(a.app.panel.display("tb303_1.edit_step"), step);
    assert_eq!(a.app.panel.value("tb303_1.cutoff"), cutoff);
    assert!(a.app.about_is_open());
    a.key(Key::F5);
    assert!(a.app.panel.transport.playing, "F5 still plays");
}

/// With a window open and the focus nowhere (before the window takes it, or after egui cleared
/// it), the panel's keys wait too.
#[test]
fn the_panel_keys_wait_while_a_window_is_open_and_nothing_has_the_focus() {
    let mut a = app(true, true);
    a.app.show_about();
    // the first frames: the focus has not moved in yet, and nothing has it
    a.frame(vec![]);
    assert!(a.frames.ctx.memory(|m| m.focused()).is_none());
    a.key(Key::Enter);
    a.key(Key::V);
    assert!(take(&mut a).is_empty(), "before the window took the focus");
    a.frame(vec![]);
    // the focus is cleared (as a click on the window's text may do)
    let f = a
        .frames
        .ctx
        .memory(|m| m.focused())
        .expect("the link has it");
    a.frames.ctx.memory_mut(|m| m.surrender_focus(f));
    a.frame(vec![]);
    assert!(a.frames.ctx.memory(|m| m.focused()).is_none());
    a.key(Key::Enter);
    a.key(Key::V);
    a.key(Key::Space);
    assert!(take(&mut a).is_empty(), "with the focus cleared");
    assert!(!a.app.panel.transport.playing);
    a.key(Key::F5);
    assert!(a.app.panel.transport.playing, "F5 still plays");
}

/// Typing in Info's focused text field: the characters and Left / Right are the field's, the
/// panel does not change (the pitch keys C and V, `[`).
#[test]
fn typing_in_an_info_field_does_not_reach_the_panel() {
    use orb_ui::info::{Field, field_value};
    let mut a = app(true, true);
    focus(&mut a, a11y::hit_id("knob", "tb303_1.cutoff"));
    a.app.show_info();
    a.frame(vec![]);
    a.frame(vec![]);
    assert_eq!(focused(&a), Field::Title.edit_id().accesskit_id());
    take(&mut a);
    let before = field_value(&a.app.doc, Field::Title);
    let step = a.app.panel.display("tb303_1.edit_step");
    let typed = |a: &mut AppTree, key: Key, text: &str| {
        a.frame(vec![
            key_event(key, true, false, Modifiers::NONE),
            Event::Text(text.into()),
        ]);
        a.frame(vec![key_event(key, false, false, Modifiers::NONE)]);
    };
    typed(&mut a, Key::C, "c");
    typed(&mut a, Key::V, "v");
    a.key(Key::ArrowLeft);
    typed(&mut a, Key::OpenBracket, "[");
    a.key(Key::ArrowRight);
    typed(&mut a, Key::B, "b");
    assert_eq!(
        field_value(&a.app.doc, Field::Title),
        format!("{before}c[vb"),
        "the field took the characters and the cursor moves"
    );
    assert!(take(&mut a).is_empty(), "no engine event");
    assert_eq!(a.app.panel.display("tb303_1.edit_step"), step);
    assert_eq!(a.app.panel.focus, 0);
    assert!(!a.app.panel.transport.playing);
}

/// A second window opened while one has the focus gets it; a window opened under a modal
/// (here the unsaved-changes prompt) gets it when the modal closes.
#[test]
fn a_second_window_and_a_window_under_a_modal_get_the_focus() {
    let mut a = app(true, true);
    focus(&mut a, a11y::hit_id("knob", "tb303_1.cutoff"));
    a.app.show_key_help();
    a.frame(vec![]);
    a.frame(vec![]);
    assert!(name_of(&a, focused(&a)).starts_with("Keyboard mode now"));
    a.app.show_about();
    a.frame(vec![]);
    a.frame(vec![]);
    // (its first line, the version: the lines of About take the focus, task-193)
    assert!(
        name_of(&a, focused(&a)).starts_with("Version "),
        "the newer window, About, has it"
    );

    let mut a = app(true, true);
    let cutoff = a11y::hit_id("knob", "tb303_1.cutoff");
    focus(&mut a, cutoff);
    a.app.fail("a test error");
    a.frame(vec![]);
    assert!(a.app.error_open());
    a.app.show_key_help();
    for _ in 0..4 {
        a.frame(vec![]);
    }
    assert!(
        !name_of(&a, focused(&a)).starts_with("Keyboard mode now"),
        "the move waits under the error box"
    );
    a.app.dismiss_error();
    assert!(a.app.key_help_is_open());
    a.frame(vec![]);
    a.frame(vec![]);
    assert!(
        name_of(&a, focused(&a)).starts_with("Keyboard mode now"),
        "then the window has the focus"
    );
    a.key(Key::Escape);
    a.frame(vec![]);
    assert_eq!(focused(&a), cutoff.accesskit_id());
}

/// The menu bar (F10) and a window take the focus and give it back without moving the row
/// focus that Ctrl+Up / Ctrl+Down set.
#[test]
fn the_menu_and_a_window_keep_the_row_focus() {
    let mut a = app(true, true);
    let cutoff = a11y::hit_id("knob", "tb303_1.cutoff");
    focus(&mut a, cutoff);
    key_with(&mut a, Key::ArrowDown, Modifiers::CTRL);
    key_with(&mut a, Key::ArrowDown, Modifiers::CTRL);
    assert_eq!(a.app.panel.focus, 2);
    a.key(Key::F10);
    assert_eq!(
        name_of(&a, focused(&a)),
        "File",
        "the menu bar has the focus"
    );
    a.key(Key::Escape);
    a.key(Key::Escape);
    assert_eq!(a.app.panel.focus, 2);
    focus(&mut a, cutoff);
    assert_eq!(a.app.panel.focus, 2, "back in the same section: kept");
    a.key(Key::F1);
    assert!(a.app.key_help_is_open());
    a.key(Key::Escape);
    a.frame(vec![]);
    assert_eq!(focused(&a), cutoff.accesskit_id());
    assert_eq!(a.app.panel.focus, 2, "a window kept it too");
}

/// The keys that stand for the original's ignore an auto-repeated press, as the original
/// does; the focus moves (Tab, F6) honor one.
#[test]
fn auto_repeat_moves_the_focus_only() {
    let held = |a: &mut AppTree, key: Key, m: Modifiers| {
        a.frame(vec![key_event(key, true, false, m)]);
        a.frame(vec![key_event(key, true, true, m)]);
        a.frame(vec![key_event(key, true, true, m)]);
        a.frame(vec![key_event(key, false, false, m)]);
    };
    let (mut m, mut a) = (app(false, false), app(true, false));
    for (key, kind, id) in [
        (Key::F12, "switch", "transport.mode"),
        (Key::F9, "toggle", "transport.record"),
        (Key::F11, "toggle", "transport.loop"),
        (Key::F5, "trigger", "transport.play"),
        (Key::F5, "trigger", "transport.stop"),
    ] {
        click(&mut m, kind, id);
        held(&mut a, key, Modifiers::NONE);
        assert_eq!(take(&mut a), take(&mut m), "{key:?} once");
    }
    let loop_ = a11y::hit_id("toggle", "transport.loop");
    focus(&mut a, loop_);
    take(&mut a);
    click(&mut m, "toggle", "transport.loop");
    held(&mut a, Key::Space, Modifiers::NONE);
    assert_eq!(take(&mut a), take(&mut m), "Space once");
    a.app.panel.playing_step303 = [Some(3), None];
    a.app.panel.focus = 0;
    m.app.panel.playing_step303 = [Some(3), None];
    m.app.panel.focus = 0;
    held(&mut a, Key::Tab, Modifiers::CTRL);
    m.key(Key::Tab);
    assert_eq!(take(&mut a), take(&mut m), "Ctrl+Tab once");
    // F6 and Tab move on each repeat
    let order = nav::order(&layout::controls());
    let first = |s: usize| order.iter().find(|x| x.section == s).unwrap().id;
    focus(&mut a, first(0));
    held(&mut a, Key::F6, Modifiers::NONE);
    assert_eq!(focused(&a), first(3).accesskit_id(), "three sections on");
    focus(&mut a, order[0].id);
    held(&mut a, Key::Tab, Modifiers::NONE);
    assert_eq!(focused(&a), order[3].id.accesskit_id(), "three controls on");
}

/// Enter goes by the section of the focused control (the user's decision, task-207): Play in
/// the Transport even with a 303's row focus, the 303's Step in its section.
#[test]
fn enter_follows_the_section() {
    let (mut m, mut a) = (app(false, false), app(true, false));
    focus(&mut a, a11y::hit_id("knob", "tb303_1.cutoff"));
    key_with(&mut a, Key::F6, Modifiers::SHIFT);
    assert_eq!(group_of(&a, focused(&a)), "Transport");
    assert_eq!(
        a.app.panel.focus, 0,
        "the row focus is the last machine entered"
    );
    take(&mut a);
    click(&mut m, "trigger", "transport.play");
    a.key(Key::Enter);
    let (em, ea) = (take(&mut m), take(&mut a));
    assert!(!em.is_empty());
    assert_eq!(ea, em, "Enter in the Transport is Play");
    a.key(Key::F5);
    take(&mut a);
    // an effect section: Play too
    focus(&mut a, a11y::hit_id("toggle", "delay.on"));
    a.key(Key::Enter);
    assert!(a.app.panel.transport.playing);
    a.key(Key::F5);
    // in Bass Line 1's section: its Step, though Ctrl+Down moved the row focus away
    focus(&mut a, a11y::hit_id("knob", "tb303_1.cutoff"));
    key_with(&mut a, Key::ArrowDown, Modifiers::CTRL);
    assert_eq!(a.app.panel.focus, 1);
    let step = |h: &AppTree| h.app.panel.display("tb303_1.edit_step");
    let before = step(&a);
    a.key(Key::Enter);
    assert_ne!(step(&a), before, "the first 303 stepped");
    assert!(!a.app.panel.transport.playing);
}

/// The order of the first and last stops of two sections, as docs/accessibility.md 3.2 lists
/// them (the transport) and as the panel shows them (Bass Line 1: the pattern row first, the
/// mixer channel last).
#[test]
fn the_order_of_two_sections_by_name() {
    let order = nav::order(&layout::controls());
    let names = |s: usize| -> Vec<String> {
        order
            .iter()
            .filter(|x| x.section == s)
            .map(|x| stop_name(x.id))
            .collect()
    };
    let t = names(0);
    assert_eq!(t[..3], ["Play", "Stop", "Record"]);
    assert_eq!(t.last().unwrap(), "Fast forward");
    let b = names(1);
    assert_eq!(b[..3], ["Pattern", "Pattern section on", "Shuffle"]);
    assert_eq!(b.last().unwrap(), "Channel 1 compressor");
}

/// AC3: with the mode off, Tab, Shift+Tab and Up/Down are the original's, F5 to F12 do
/// nothing, and egui moves its own focus on Tab as on main.
#[test]
fn with_the_mode_off_the_keys_are_as_before() {
    let mut m = app(false, true);
    // (before anything has egui's focus: on main the arrows also move egui's focus, up into
    // the menu bar, which then has the keys, TASK-202)
    m.key(Key::ArrowUp);
    assert_eq!(m.app.panel.focus, 3, "Up moves the row focus");
    m.key(Key::ArrowDown);
    m.key(Key::ArrowDown);
    assert_eq!(m.app.panel.focus, 1, "Down too, wrapping");
    m.app.panel.playing_step303 = [Some(3), None];
    m.app.panel.focus = 0;
    m.key(Key::Tab);
    assert_eq!(
        take(&mut m),
        vec![Out::Step303 {
            synth: 0,
            pattern: 0,
            step: 3,
            field: 5,
            value: 0
        }],
        "Tab taps"
    );
    assert_eq!(
        focused(&m),
        a11y::hit_id("knob", "transport.shuffle").accesskit_id(),
        "egui's own Tab: the first control of layout.tsv"
    );
    key_with(&mut m, Key::Tab, Modifiers::CTRL);
    assert!(take(&mut m).is_empty(), "Ctrl+Tab does nothing");
    for k in [Key::F1, Key::F5, Key::F6, Key::F9, Key::F11, Key::F12] {
        m.key(k);
    }
    assert!(take(&mut m).is_empty(), "no function key");
    assert!(!m.app.panel.transport.playing);
    assert_eq!(m.app.panel.focus, 0);
    assert_eq!(
        focused(&m),
        a11y::hit_id("knob", "transport.shuffle").accesskit_id(),
        "F6 moves nothing"
    );
    m.key(Key::Space);
    assert!(m.app.panel.transport.playing, "Space plays");
}
