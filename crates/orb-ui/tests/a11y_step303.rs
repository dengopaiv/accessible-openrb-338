//! The 303 step editor, spoken (task-190, A11Y-6, docs/ui_spec.md section 20.5): the step
//! editor's node, its keys, what is said after each programming key, and F2 on a 303.
//!
//! AC1: a pattern programmed with the keys alone (screen reader mode) and the same steps
//! written by the mouse on a second app (the panel at 1:1, synthetic pointer events, as
//! `tests/a11y_sections.rs`) give equal engine events (`Panel::events`) and an equal F2 text.

#[path = "common/a11y.rs"]
mod a11y;

use a11y::AppTree;
use eframe::egui::accesskit::{Action, Role};
use eframe::egui::{Event, Id, Key, Modifiers, PointerButton, Pos2};
use orb_ui::a11y::step303;
use orb_ui::layout::{self, Ctl};
use orb_ui::model::Event as Out;
use orb_ui::screen_reader::ScreenReaderSetting;

fn ctl(kind: &str, id: &str) -> Ctl {
    layout::controls()
        .into_iter()
        .find(|c| c.id == id && c.kind == kind)
        .unwrap_or_else(|| panic!("{kind} {id}"))
}

/// The app with screen reader mode on or off, without the menu bar (panel pixels are points,
/// for the mouse).
fn app(screen_reader: bool) -> AppTree {
    let mut h = AppTree::new();
    h.app.menu = false;
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

fn key_event(key: Key, pressed: bool, modifiers: Modifiers) -> Event {
    Event::Key {
        key,
        physical_key: Some(key),
        pressed,
        repeat: false,
        modifiers,
    }
}

fn shift_tab(h: &mut AppTree) {
    h.frame(vec![key_event(Key::Tab, true, Modifiers::SHIFT)]);
    h.frame(vec![key_event(Key::Tab, false, Modifiers::SHIFT)]);
}

/// A mouse click on the trigger `id` of the panel.
fn click(h: &mut AppTree, id: &str) {
    let c = ctl("trigger", id);
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

/// The step editor's value in the last tree.
fn editor(h: &AppTree, k: usize) -> String {
    let n = h
        .tree()
        .by_id(step303::editor_id(k))
        .expect("the step editor");
    a11y::value(n).unwrap_or_default().to_string()
}

fn said(h: &AppTree) -> Option<String> {
    h.app.announcer.text().map(str::to_string)
}

/// Presses `k` and returns whether the announcer spoke, and what.
fn press(h: &mut AppTree, k: Key) -> Option<String> {
    let n = h.app.announcer.serial();
    h.key(k);
    (h.app.announcer.serial() != n).then(|| said(h).unwrap_or_default())
}

#[test]
fn the_step_editor_is_a_focusable_slider_in_its_section() {
    let mut a = app(true);
    for k in 0..2 {
        let id = step303::editor_id(k).accesskit_id();
        let t = a.tree();
        let n = t.node(id).expect("the step editor");
        assert_eq!(a11y::role(n), Role::Slider);
        assert_eq!(a11y::name(n), Some(step303::EDITOR_NAME));
        assert_eq!(a11y::value(n), Some("Step 1 of 16: rest, C"));
        assert!(a11y::supports(n, Action::Focus));
        let (_, g) = t.parent(id).expect("a parent");
        let section = format!("Bass Line {}", k + 1);
        assert_eq!(a11y::name(g), Some(section.as_str()));
    }
    // Tab reaches it from the knobs before it
    focus(&mut a, a11y::hit_id("knob", "tb303_1.accent"));
    a.key(Key::Tab);
    assert_eq!(a.tree().focused_id(), step303::editor_id(0).accesskit_id());
    // with the mode off it stays the display's label
    let m = app(false);
    let n = m.tree().by_id(step303::editor_id(0)).unwrap();
    assert_eq!(a11y::role(n), Role::Label);
}

/// AC1 and AC2.
#[test]
fn a_pattern_programmed_by_keys_equals_the_mouse_s() {
    let (mut m, mut a) = (app(false), app(true));
    focus(&mut a, step303::editor_id(0));
    assert_eq!(a.app.panel.focus, 0, "the section gave the row focus");
    take(&mut a);

    // Step 1 (time mode, the edit step stays): C, note, accent. The same note again is said by
    // the announcer (the value does not change); the others by the editor's value.
    assert_eq!(press(&mut a, Key::C).as_deref(), Some("Step 1: rest, C"));
    assert_eq!(press(&mut a, Key::Minus), None);
    assert_eq!(editor(&a, 0), "Step 1 of 16: C");
    assert_eq!(press(&mut a, Key::P), None);
    assert_eq!(editor(&a, 0), "Step 1 of 16: C, accent");
    for id in ["key0", "note_pause", "accent"] {
        click(&mut m, &format!("tb303_1.{id}"));
    }

    // Down, Down: step 3, D sharp, note, slide, octave up
    assert_eq!(press(&mut a, Key::ArrowDown), None);
    assert_eq!(editor(&a, 0), "Step 2 of 16: rest, C");
    a.key(Key::ArrowDown);
    for k in [Key::G, Key::Minus, Key::OpenBracket, Key::Quote] {
        assert_eq!(press(&mut a, k), None, "{k:?}");
    }
    assert_eq!(editor(&a, 0), "Step 3 of 16: D sharp, octave up, slide");
    for id in ["step", "step", "key3", "note_pause", "slide", "up"] {
        click(&mut m, &format!("tb303_1.{id}"));
    }

    // Enter (the section's Step): step 4, high C, note (`=`), octave down
    a.key(Key::Enter);
    for k in [Key::Slash, Key::Equals, Key::Semicolon] {
        a.key(k);
    }
    assert_eq!(editor(&a, 0), "Step 4 of 16: high C, octave down");
    for id in ["step", "key12", "note_pause", "down"] {
        click(&mut m, &format!("tb303_1.{id}"));
    }

    // Home: step 1; End: step 16 (the mouse: Back three times, then once, wrapping)
    a.key(Key::Home);
    assert_eq!(editor(&a, 0), "Step 1 of 16: C, accent");
    a.key(Key::End);
    assert_eq!(editor(&a, 0), "Step 16 of 16: rest, C");
    for k in [Key::K, Key::Minus] {
        a.key(k);
    }
    assert_eq!(editor(&a, 0), "Step 16 of 16: G sharp");
    // Down wraps to step 1, Up back to 16, Backspace (Back) to 15, Enter to 16; Left and
    // Right do nothing (decision task-209)
    a.key(Key::ArrowDown);
    assert_eq!(a.app.panel.synth[0].edit, 0);
    a.key(Key::ArrowUp);
    a.key(Key::ArrowLeft);
    a.key(Key::ArrowRight);
    assert_eq!(a.app.panel.synth[0].edit, 15);
    a.key(Key::Backspace);
    a.key(Key::Enter);
    assert_eq!(a.app.panel.synth[0].edit, 15);
    for id in ["back", "back", "back", "back", "key8", "note_pause"] {
        click(&mut m, &format!("tb303_1.{id}"));
    }

    // Pitch mode by Tab and Space on its button, then back to the editor: each key sets a note
    // and moves on; the step it set is said
    for _ in 0..3 {
        a.key(Key::Tab);
    }
    let pitch = a11y::hit_id("trigger", "tb303_1.pitch_button");
    assert_eq!(a.tree().focused_id(), pitch.accesskit_id());
    assert_eq!(
        press(&mut a, Key::Space).as_deref(),
        Some("Pitch mode on, step 1 of 16: C, accent")
    );
    for _ in 0..3 {
        shift_tab(&mut a);
    }
    assert_eq!(a.tree().focused_id(), step303::editor_id(0).accesskit_id());
    assert_eq!(press(&mut a, Key::B).as_deref(), Some("Step 1: E, accent"));
    assert_eq!(editor(&a, 0), "Step 2 of 16: rest, C, pitch mode");
    assert_eq!(press(&mut a, Key::N).as_deref(), Some("Step 2: rest, F"));
    for id in ["pitch_button", "key4", "key5"] {
        click(&mut m, &format!("tb303_1.{id}"));
    }

    // equal events
    let (em, ea) = (take(&mut m), take(&mut a));
    assert_eq!(em.len(), 14, "{em:?}");
    assert!(
        em.iter()
            .all(|e| matches!(e, Out::Step303 { synth: 0, .. }))
    );
    assert_eq!(ea, em, "the keys send what the mouse sends");
    assert_eq!(a.app.panel.synth[0].patterns, m.app.panel.synth[0].patterns);

    // equal text: F2 on both (the mouse's app in screen reader mode now, nothing focused, the
    // row focus on Bass Line 1 from the clicks)
    let text = "Bass Line 1, pattern A1, 16 steps, shuffle off. 1 E accent, 2 rest, \
                3 D sharp octave up slide, 4 high C octave down, 5 rest, 6 rest, 7 rest, \
                8 rest, 9 rest, 10 rest, 11 rest, 12 rest, 13 rest, 14 rest, 15 rest, \
                16 G sharp";
    assert_eq!(press(&mut a, Key::F2).as_deref(), Some(text));
    m.app.settings.screen_reader = ScreenReaderSetting::On;
    m.frame(vec![]);
    assert_eq!(m.app.panel.focus, 0);
    assert_eq!(press(&mut m, Key::F2).as_deref(), Some(text));
    assert!(take(&mut a).is_empty(), "F2 sends nothing");
}

#[test]
fn programming_keys_with_the_focus_elsewhere_are_announced() {
    let mut a = app(true);
    focus(&mut a, a11y::hit_id("knob", "tb303_2.cutoff"));
    assert_eq!(press(&mut a, Key::Minus).as_deref(), Some("Step 1: C"));
    assert_eq!(press(&mut a, Key::J).as_deref(), Some("Step 1: F sharp"));
    assert_eq!(
        press(&mut a, Key::Enter).as_deref(),
        Some("Step 2 of 16: rest, C")
    );
    assert_eq!(
        press(&mut a, Key::Backspace).as_deref(),
        Some("Step 1 of 16: F sharp")
    );
    // Space on the 303's accent button: the step it changed
    focus(&mut a, a11y::hit_id("trigger", "tb303_2.accent"));
    assert_eq!(
        press(&mut a, Key::Space).as_deref(),
        Some("Step 1: F sharp, accent")
    );
    // the Clear button
    focus(&mut a, a11y::hit_id("trigger", "tb303_2.clear"));
    assert_eq!(
        press(&mut a, Key::Space).as_deref(),
        Some("Pattern A1 cleared")
    );
    // F2 on the second Bass Line, its pattern in its length
    a.app.panel.rows[1].len[0] = 2;
    assert_eq!(
        press(&mut a, Key::F2).as_deref(),
        Some("Bass Line 2, pattern A1, 2 steps, shuffle off. 1 rest, 2 rest")
    );
    // a drum machine's F2 has no 303 steps
    focus(&mut a, a11y::hit_id("knob", "tr808.bd_level"));
    let t = press(&mut a, Key::F2).unwrap();
    assert!(t.starts_with("TR-808, pattern A1"), "{t}");
}

/// The review's case: playback moving the row to another pattern (`Panel::adopt_pattern`, as
/// `Link::follow_values` does) changes neither the focused editor's value nor the announcer,
/// with Options > Step Editor Follows Playback off (the default) or on ("move-along doesn't
/// speak", decision task-209). Off, the editor stays on its pattern until the user chooses
/// another; on, it moves to the playing one at the next input.
#[test]
fn playback_moving_the_pattern_is_silent() {
    for follows in [false, true] {
        let mut a = app(true);
        a.app.settings.step_editor_follows = follows;
        let s = &mut a.app.panel.synth[0].patterns[3][0];
        s.rest = false;
        s.note = 7;
        focus(&mut a, step303::editor_id(0));
        let first = "Step 1 of 16: rest, C";
        assert_eq!(editor(&a, 0), first);
        let n = a.app.announcer.serial();
        a.app.panel.adopt_pattern(0, 3);
        for _ in 0..3 {
            a.frame(vec![]);
        }
        assert_eq!(editor(&a, 0), first, "follows {follows}");
        assert_eq!(a.app.announcer.serial(), n, "follows {follows}");
        assert_eq!(a.app.panel.rows[0].pattern, 3, "the panel shows A4");
        take(&mut a);

        // F2 is input: on, the editor has moved to A4; off, F2 says where the editor is
        let f2 = press(&mut a, Key::F2).unwrap();
        let row = "Bass Line 1, pattern A4, 16 steps, shuffle off. ";
        if follows {
            assert!(f2.starts_with(&format!("{row}1 G, 2 rest")), "{f2}");
            assert_eq!(editor(&a, 0), "Step 1 of 16: G");
        } else {
            let editing = "Step editor on pattern A1: 1 rest, 2 rest";
            assert!(f2.starts_with(&format!("{row}{editing}")), "{f2}");
            assert_eq!(editor(&a, 0), first);
        }
        // a programming key edits the editor's pattern
        a.key(Key::C);
        let pattern = if follows { 3 } else { 0 };
        assert_eq!(
            take(&mut a),
            vec![Out::Step303 {
                synth: 0,
                pattern,
                step: 0,
                field: 0,
                value: 0,
            }]
        );
        if !follows {
            // the user chooses A4 on the panel: the editor goes there
            a.app.panel.press("pattern1.pattern4", 0.0);
            a.frame(vec![]);
            assert_eq!(editor(&a, 0), "Step 1 of 16: G");
        }
    }
    // with the mode off there is no pin: the row's pattern, as the original
    let mut m = app(false);
    m.app.panel.adopt_pattern(0, 3);
    m.frame(vec![]);
    assert!(m.app.panel.synth.iter().all(|s| s.pin.is_none()));
    assert_eq!(m.app.panel.edit_pattern(0), 3);
}

#[test]
fn the_editor_keys_move_the_edit_step_only_on_the_editor() {
    let mut a = app(true);
    focus(&mut a, step303::editor_id(1));
    a.app.panel.rows[1].len[0] = 8;
    a.key(Key::End);
    assert_eq!(editor(&a, 1), "Step 8 of 8: rest, C");
    a.key(Key::ArrowDown);
    assert_eq!(editor(&a, 1), "Step 9, past the 8 steps: rest, C");
    a.key(Key::ArrowUp);
    a.key(Key::ArrowUp);
    assert_eq!(a.app.panel.synth[1].edit, 6);
    // Left and Right do nothing on the editor (decision task-209): no move, no event, the
    // row focus stays
    take(&mut a);
    let row = a.app.panel.focus;
    a.key(Key::ArrowLeft);
    a.key(Key::ArrowRight);
    assert_eq!(a.app.panel.synth[1].edit, 6);
    assert!(take(&mut a).is_empty());
    assert_eq!(a.app.panel.focus, row);
    a.key(Key::Home);
    assert_eq!(a.app.panel.synth[1].edit, 0);
    // auto-repeat counts
    a.frame(vec![key_event(Key::ArrowDown, true, Modifiers::NONE)]);
    a.frame(vec![Event::Key {
        key: Key::ArrowDown,
        physical_key: Some(Key::ArrowDown),
        pressed: true,
        repeat: true,
        modifiers: Modifiers::NONE,
    }]);
    a.frame(vec![key_event(Key::ArrowDown, false, Modifiers::NONE)]);
    assert_eq!(a.app.panel.synth[1].edit, 2);
    assert!(take(&mut a).is_empty(), "moving sends nothing");
    assert_eq!(a.app.panel.synth[0].edit, 0, "the other 303's stays");
    // elsewhere Left and Right do nothing and Home does not move it
    focus(&mut a, a11y::hit_id("knob", "tb303_2.cutoff"));
    a.key(Key::ArrowLeft);
    a.key(Key::ArrowRight);
    assert_eq!(a.app.panel.synth[1].edit, 2);
    // with the mode off, the arrows are the original's (the row focus), not the editor's
    let mut m = app(false);
    m.key(Key::ArrowRight);
    m.key(Key::End);
    assert_eq!(m.app.panel.synth[0].edit, 0);
    assert_eq!(m.app.announcer.text(), None);
}

/// Task-211: with Options > Pitch Keys Make Notes on, a pitch key on a rest says the step as a
/// note ("Step 1 of 16: D"); off, it stays "rest, D"; with screen reader mode off, the model
/// never clears the rest.
#[test]
fn pitch_keys_make_notes_is_spoken_and_only_in_screen_reader_mode() {
    let mut a = app(true);
    a.app.settings.pitch_keys_make_notes = true;
    a.frame(vec![]);
    focus(&mut a, step303::editor_id(0));
    take(&mut a);
    a.key(Key::V);
    assert_eq!(editor(&a, 0), "Step 1 of 16: D");
    let ev = |field, value| Out::Step303 {
        synth: 0,
        pattern: 0,
        step: 0,
        field,
        value,
    };
    assert_eq!(take(&mut a), vec![ev(5, 0), ev(0, 2)]);
    // the setting off: a rest stays a rest
    a.app.settings.pitch_keys_make_notes = false;
    a.key(Key::Minus);
    a.frame(vec![]);
    assert_eq!(editor(&a, 0), "Step 1 of 16: rest, D");
    a.key(Key::G);
    assert_eq!(editor(&a, 0), "Step 1 of 16: rest, D sharp");
    // screen reader mode off: the model never clears the rest
    let mut m = app(false);
    m.app.settings.pitch_keys_make_notes = true;
    m.frame(vec![]);
    click(&mut m, "tb303_1.key2");
    assert!(m.app.panel.synth[0].patterns[0][0].rest);
}
