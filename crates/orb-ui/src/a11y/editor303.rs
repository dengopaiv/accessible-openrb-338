//! The 303 pattern editor windows in screen reader mode (task-235, the user's decision task-242,
//! option A: the whole window; docs/ui_spec.md section 20.8).
//!
//! With the mode on, the window of `crate::pattern_editor` draws this content instead of its
//! header and painted grids (with the mode off it is upstream's, untouched): named controls in
//! one tab order, each operable from the keyboard.
//!
//! - **Pattern**, a spin button ("A1"): Up and Down the next and the previous of the 303's 32
//!   patterns, Page Up and Page Down a bank, Home and End A1 and D8, as the panel's pattern
//!   selector (task-187). It stands for the header's bank buttons, number and arrows.
//! - **Step editor**, the roll as a list of the 16 steps, with the keys of the panel's step editor
//!   (`super::step303`, task-190): Up and Down the previous and the next step (wrapping at 16),
//!   Home and End step 1 and the pattern's last step, Left and Right nothing, Enter and Backspace
//!   the next and the previous step, and the programming keys of Program Synth mode (the pitch
//!   keys, `-` / `=`, `P`, `[`, `'`, `;`), Options > Pitch Keys Make Notes included. Its value says
//!   the step and what it holds, in the panel's words.
//! - **Length** (spin button, "16 steps"), **Shuffle** (check box), **Transpose up**, **Transpose
//!   down**, **Shift left**, **Shift right**, **Copy**, **Paste** and **Clear**.
//!
//! Every edit is an [`Act`] run by `App::editor_act`, the window's one path to
//! `App::write_pattern`. The pattern is the one the header shows; in screen reader mode the
//! window follows the pattern the 303's step editor edits (`Panel::edit_pattern`), not the one
//! playback moves the row to, so a focused control never changes without input.

use egui::{self, Key, Modifiers, Response, Sense, StrokeKind, Ui, accesskit, vec2};

use super::{drive, names, step303};
use crate::keys::{self, Action, KeyMode, SrAction};
use crate::pattern_editor::{self, Act, Flag, STEPS, StepEdit, step_pattern};

/// The window's controls, in their tab order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Part {
    Pattern,
    Steps,
    Length,
    Shuffle,
    TransposeUp,
    TransposeDown,
    ShiftLeft,
    ShiftRight,
    Copy,
    Paste,
    Clear,
}

impl Part {
    pub const ALL: [Part; 11] = [
        Part::Pattern,
        Part::Steps,
        Part::Length,
        Part::Shuffle,
        Part::TransposeUp,
        Part::TransposeDown,
        Part::ShiftLeft,
        Part::ShiftRight,
        Part::Copy,
        Part::Paste,
        Part::Clear,
    ];

    /// The control's name, as the screen reader says it.
    pub fn name(self) -> &'static str {
        match self {
            Part::Pattern => "Pattern",
            Part::Steps => step303::EDITOR_NAME,
            Part::Length => "Length",
            Part::Shuffle => "Shuffle",
            Part::TransposeUp => "Transpose up",
            Part::TransposeDown => "Transpose down",
            Part::ShiftLeft => "Shift left",
            Part::ShiftRight => "Shift right",
            Part::Copy => "Copy",
            Part::Paste => "Paste",
            Part::Clear => "Clear",
        }
    }
}

/// The egui id of control `part` of the window of 303 `k` (0 or 1).
pub fn id(k: usize, part: Part) -> egui::Id {
    egui::Id::unique(("orb_editor303_sr", k.min(1), part as u8))
}

/// The 303 (0 or 1) whose editor window has the control of egui id `f`.
pub fn owner(f: egui::Id) -> Option<usize> {
    (0..2).find(|&k| Part::ALL.iter().any(|&p| id(k, p) == f))
}

/// The Step editor's value: step `i` (0-based) of the pattern `bytes` and what it holds, in the
/// panel's step editor's words ("Step 5 of 16: C sharp, accent").
pub fn steps_value(bytes: &[u8], i: usize) -> String {
    let s = pattern_editor::step303_of(bytes, i);
    step303::step_value(i, pattern_editor::length(bytes), &s)
}

/// What the announcer says after a programming key whose step's value did not change (the same
/// note again; the panel's step editor says it alike): "Step 1: rest, C".
pub fn step_said(bytes: &[u8], i: usize) -> String {
    let s = pattern_editor::step303_of(bytes, i);
    format!("Step {}: {}", i + 1, step303::step_text(&s))
}

/// The Length value: "16 steps", "1 step".
pub fn length_value(bytes: &[u8]) -> String {
    let n = pattern_editor::length(bytes) as i32;
    names::display_text("pattern1.steps", Some(n))
}

/// F2 with the focus in the window of 303 `k`: its pattern `p` as the panel's F2 reads a 303's
/// row, from the editor's pattern: "303-1, pattern A3, 8 steps, shuffle off. 1 C accent, 2
/// rest, 3 D sharp slide" (the steps in the pattern's length).
pub fn readout(k: usize, p: u8, bytes: &[u8]) -> String {
    let len = pattern_editor::length(bytes);
    let steps: Vec<String> = (0..len)
        .map(|i| step303::compact(i, &pattern_editor::step303_of(bytes, i)))
        .collect();
    format!(
        "303-{}, pattern {}, {}, shuffle {}. {}",
        k.min(1) + 1,
        names::pattern_text(p),
        length_value(bytes),
        if bytes[0] != 0 { "on" } else { "off" },
        steps.join(", ")
    )
}

/// What the announcer says after the act `a` on pattern `p` of the window, which held `before`
/// and holds `after`, when the act's result is not the value of a focused control:
///
/// - Transpose and Shift: what they did ("Pattern A1 transposed up"), or "Pattern A1 unchanged"
///   when the bytes are as they were (a Shift of a pattern whose steps are all alike), so that the
///   key is known to be taken and nothing false is said;
/// - a programming key on the Step editor whose step reads as before (the same note again, a pitch
///   key on a rest): the step, "Step 1: rest, C", as the panel's step editor says it; a change of
///   the step's value is the focused Step editor's own to speak.
///
/// (Copy, Paste and Clear put their own words on the status line, which speaks them; the
/// values of Pattern, Length and Shuffle are spoken by the screen reader as they change.) The
/// app takes `before` and `after` around each act, so several keys of one frame are each
/// judged on their own.
pub fn act_said(a: &Act, p: u8, before: &[u8], after: &[u8]) -> Option<String> {
    let name = names::pattern_text(p);
    if let Act::StepEdit { step, .. } = a {
        let step = (*step).min(STEPS - 1);
        return (steps_value(before, step) == steps_value(after, step))
            .then(|| step_said(after, step));
    }
    if matches!(a, Act::Transpose { .. } | Act::Shift { .. }) && before == after {
        return Some(format!("Pattern {name} unchanged"));
    }
    Some(match a {
        Act::Transpose { up } => {
            format!(
                "Pattern {name} transposed {}",
                if *up { "up" } else { "down" }
            )
        }
        Act::Shift { right } => {
            format!(
                "Pattern {name} shifted {}",
                if *right { "right" } else { "left" }
            )
        }
        _ => return None,
    })
}

/// A key on the focused Step editor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StepKey {
    /// A move of the step (Up, Down, Home, End; Enter and Backspace are Step and Back).
    Move(step303::Move),
    /// A programming key: what it does to the step.
    Edit(StepEdit),
}

/// What the key `key` (its position `physical`, with `shift`) does on the focused Step editor
/// of 303 `k`: the moves of `step303::Move::of_key`, then the programming keys of the panel's
/// key map in Program Synth mode (`keys::map_screen_reader`, the row of 303 `k`), whatever the
/// keyboard mode, since the editor is a step editor. A key with Ctrl, Cmd or Alt is none (the
/// menus' shortcuts, Ctrl+Z among them, stay the app's), and so is an auto-repeat of a
/// programming key (the original ignores them; the moves repeat).
pub fn step_key(
    k: usize,
    key: Key,
    physical: Option<Key>,
    modifiers: Modifiers,
    repeat: bool,
) -> Option<StepKey> {
    if modifiers.ctrl || modifiers.command || modifiers.alt {
        return None;
    }
    if !modifiers.shift
        && let Some(m) = step303::Move::of_key(key)
    {
        return Some(StepKey::Move(m));
    }
    if repeat {
        return None;
    }
    let pos = keys::position(key, physical);
    let k = k.min(1);
    let Some(SrAction::Panel(Action::Press(id))) =
        keys::map_screen_reader(pos, modifiers.shift, false, false, KeyMode::Synth, k)
    else {
        return None;
    };
    let name = id.strip_prefix(&format!("tb303_{}.", k + 1))?;
    Some(match name {
        "step" => StepKey::Move(step303::Move::Step),
        "back" => StepKey::Move(step303::Move::Back),
        "slide" => StepKey::Edit(StepEdit::Flag(Flag::Slide)),
        "accent" => StepKey::Edit(StepEdit::Flag(Flag::Accent)),
        "up" => StepKey::Edit(StepEdit::Flag(Flag::Up)),
        "down" => StepKey::Edit(StepEdit::Flag(Flag::Down)),
        "note_pause" => StepKey::Edit(StepEdit::Rest),
        n => StepKey::Edit(StepEdit::Note(n.strip_prefix("key")?.parse().ok()?)),
    })
}

/// Step `i` moved by `m` in a pattern of `len` steps: Back and Step by one modulo 16 (the panel
/// step editor's Back and Step buttons), First step 1, Last the pattern's last step.
pub fn moved(i: usize, m: step303::Move, len: usize) -> usize {
    match m {
        step303::Move::Back => (i + STEPS - 1) % STEPS,
        step303::Move::Step => (i + 1) % STEPS,
        step303::Move::First => 0,
        step303::Move::Last => len.clamp(1, STEPS) - 1,
        step303::Move::Nothing => i,
    }
}

/// The pattern a value key `s` on the Pattern spin button chooses from `p`, as the panel's
/// pattern selector (`drive::step`'s `Target::Pattern`): Up the next, Down the previous, Page
/// Up and Page Down a bank (clamped at A1 and D8), Home A1, End D8.
pub fn pattern_step(p: u8, s: drive::Step) -> u8 {
    let p = i32::from(p.min(31));
    let to = match s {
        drive::Step::Up => return step_pattern(p as u8, true).unwrap_or(31),
        drive::Step::Down => return step_pattern(p as u8, false).unwrap_or(0),
        drive::Step::PageUp => p + 8,
        drive::Step::PageDown => p - 8,
        drive::Step::Home => 0,
        drive::Step::End => 31,
    };
    to.clamp(0, 31) as u8
}

/// The length a value key `s` on the Length spin button sets from `n`, as the panel's pattern
/// length (TASK-205): Up and Down one, Page Up and Page Down ten, Home 1, End 16 (clamped).
pub fn length_step(n: usize, s: drive::Step) -> u8 {
    let n = n as i32;
    let to = match s {
        drive::Step::Up => n + 1,
        drive::Step::Down => n - 1,
        drive::Step::PageUp => n + 10,
        drive::Step::PageDown => n - 10,
        drive::Step::Home => 1,
        drive::Step::End => STEPS as i32,
    };
    to.clamp(1, STEPS as i32) as u8
}

/// What the window's content gave in a frame.
#[derive(Debug, Default)]
pub struct Out {
    /// What it asks for, in order.
    pub acts: Vec<Act>,
    /// Escape with the focus in the window: it closes.
    pub close: bool,
}

/// The window of 303 `k` in screen reader mode, on pattern `p` holding `bytes`; `step` is the
/// Step editor's step, moved here, and `had_focus` whether a control of the window had the focus
/// at the end of the last frame (kept here). The window (egui id `window`) is made a Dialog
/// named `title`; with `due` the focus goes to its first control, Pattern.
pub fn window_ui(
    ui: &mut Ui,
    k: usize,
    window: egui::Id,
    title: &str,
    due: bool,
    (p, bytes): (u8, &[u8]),
    (step, had_focus): (&mut usize, &mut bool),
) -> Out {
    crate::app::focus_into_window(ui, due);
    super::window::window(ui, true, window, title);
    let mut out = Out::default();
    // (egui drops the focus on Escape as the frame begins: the last frame's focus counts)
    let focused = ui.memory(|m| m.focused()).and_then(owner) == Some(k);
    if (focused || *had_focus) && ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Escape)) {
        out.close = true;
    }
    let len = pattern_editor::length(bytes);
    *step = (*step).min(STEPS - 1);

    ui.horizontal(|ui| {
        // Pattern
        let pid = id(k, Part::Pattern);
        let text = names::pattern_text(p);
        let r = control(ui, pid, &format!("Pattern {text}"), false, 0.0);
        spin_node(ui, pid, Part::Pattern.name(), &text);
        if let Some(s) = value_key(ui, &r, pid) {
            let to = pattern_step(p, s);
            if to != p {
                out.acts.push(Act::Select(to));
            }
        }
        // the Step editor
        let sid = id(k, Part::Steps);
        let value = steps_value(bytes, *step);
        let r = control(ui, sid, &value, false, 280.0);
        ui.ctx().accesskit_node_builder(sid, |b| {
            b.set_role(accesskit::Role::Slider);
            b.set_orientation(accesskit::Orientation::Vertical);
            b.set_label(step303::EDITOR_NAME);
            b.set_value(value.clone());
        });
        if r.clicked() {
            r.request_focus();
        }
        if r.has_focus() {
            for e in ui.input(|i| i.events.clone()) {
                let egui::Event::Key {
                    key,
                    physical_key,
                    pressed: true,
                    repeat,
                    modifiers,
                } = e
                else {
                    continue;
                };
                match step_key(k, key, physical_key, modifiers, repeat) {
                    Some(StepKey::Move(m)) => *step = moved(*step, m, len),
                    Some(StepKey::Edit(edit)) => out.acts.push(Act::StepEdit { step: *step, edit }),
                    None => {}
                }
            }
        }
    });
    ui.horizontal(|ui| {
        let lid = id(k, Part::Length);
        let value = length_value(bytes);
        let r = control(ui, lid, &format!("Length {value}"), false, 0.0);
        spin_node(ui, lid, Part::Length.name(), &value);
        if let Some(s) = value_key(ui, &r, lid) {
            let n = length_step(len, s);
            if usize::from(n) != len {
                out.acts.push(Act::Length { n, drag: false });
            }
        }
        let shid = id(k, Part::Shuffle);
        let on = bytes[0] != 0;
        let r = control(ui, shid, Part::Shuffle.name(), on, 0.0);
        ui.ctx().accesskit_node_builder(shid, |b| {
            b.set_role(accesskit::Role::CheckBox);
            b.set_label(Part::Shuffle.name());
            b.set_toggled(on.into());
        });
        if r.clicked() {
            out.acts.push(Act::Shuffle);
        }
    });
    ui.horizontal(|ui| {
        for (part, act) in [
            (Part::TransposeUp, Act::Transpose { up: true }),
            (Part::TransposeDown, Act::Transpose { up: false }),
            (Part::ShiftLeft, Act::Shift { right: false }),
            (Part::ShiftRight, Act::Shift { right: true }),
        ] {
            if button(ui, k, part) {
                out.acts.push(act);
            }
        }
    });
    ui.horizontal(|ui| {
        for (part, act) in [
            (Part::Copy, Act::Copy),
            (Part::Paste, Act::Paste),
            (Part::Clear, Act::Clear),
        ] {
            if button(ui, k, part) {
                out.acts.push(act);
            }
        }
    });
    *had_focus = ui.memory(|m| m.focused()).and_then(owner) == Some(k);
    out
}

/// A button named as `part`; whether it was clicked (the mouse, Space or Enter on it, a screen
/// reader's Click).
fn button(ui: &mut Ui, k: usize, part: Part) -> bool {
    let bid = id(k, part);
    let r = control(ui, bid, part.name(), false, 0.0);
    ui.ctx().accesskit_node_builder(bid, |b| {
        b.set_role(accesskit::Role::Button);
        b.set_label(part.name());
    });
    r.clicked()
}

/// A spin button's node: `name` with `value` as its text value (no numeric value: UI Automation
/// would then raise two value changes for one key), Increment and Decrement as its Up and Down.
fn spin_node(ui: &Ui, id: egui::Id, name: &str, value: &str) {
    ui.ctx().accesskit_node_builder(id, |b| {
        b.set_role(accesskit::Role::SpinButton);
        b.set_label(name);
        b.set_value(value);
        b.add_action(accesskit::Action::Increment);
        b.add_action(accesskit::Action::Decrement);
    });
}

/// The value key on the focused spin button `r` (Up, Down, Page Up, Page Down, Home, End,
/// auto-repeat included), else a screen reader's Increment or Decrement on it.
fn value_key(ui: &mut Ui, r: &Response, id: egui::Id) -> Option<drive::Step> {
    use drive::Step;
    if r.clicked() {
        r.request_focus();
    }
    if r.has_focus() {
        for (key, s) in [
            (Key::ArrowUp, Step::Up),
            (Key::ArrowDown, Step::Down),
            (Key::PageUp, Step::PageUp),
            (Key::PageDown, Step::PageDown),
            (Key::Home, Step::Home),
            (Key::End, Step::End),
        ] {
            if ui.input_mut(|i| i.consume_key(Modifiers::NONE, key)) {
                return Some(s);
            }
        }
    }
    drive::requests(ui, id).into_iter().find_map(|q| match q {
        drive::Request::Increment => Some(Step::Up),
        drive::Request::Decrement => Some(Step::Down),
        drive::Request::SetValue(_) => None,
    })
}

/// A control of the window drawn as a button with `text` (at least `min_w` wide), lit when
/// `selected`, with the focus outlined; interacted with as `id` (a click; it takes the focus).
fn control(ui: &mut Ui, id: egui::Id, text: &str, selected: bool, min_w: f32) -> Response {
    let pad = ui.spacing().button_padding;
    let font = egui::TextStyle::Button.resolve(ui.style());
    let galley = ui
        .painter()
        .layout_no_wrap(text.to_owned(), font, egui::Color32::PLACEHOLDER);
    let size = vec2(
        (galley.size().x + 2.0 * pad.x).max(min_w),
        (galley.size().y + 2.0 * pad.y).max(ui.spacing().interact_size.y),
    );
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    let r = ui.interact(rect, id, Sense::click());
    if ui.is_rect_visible(rect) {
        let v = ui.style().interact_selectable(&r, selected);
        ui.painter().rect(
            rect,
            v.corner_radius,
            v.weak_bg_fill,
            v.bg_stroke,
            StrokeKind::Inside,
        );
        if r.has_focus() {
            ui.painter().rect_stroke(
                rect.expand(1.0),
                v.corner_radius,
                ui.visuals().selection.stroke,
                StrokeKind::Outside,
            );
        }
        let pos = egui::pos2(rect.left() + pad.x, rect.center().y - galley.size().y / 2.0);
        ui.painter().galley(pos, galley, v.text_color());
    }
    r
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pattern_editor::{PATTERN_BYTES, with_flag, with_length, with_note, with_shuffle};

    fn empty() -> Vec<u8> {
        let mut b = vec![0u8; PATTERN_BYTES];
        b[1] = 16;
        for s in 0..STEPS {
            b[2 + 6 * s + 5] = 1;
        }
        b
    }

    #[test]
    fn the_steps_read_as_the_panel_s_step_editor() {
        let b = with_flag(&with_note(&empty(), 4, 1), 4, Flag::Accent, true);
        assert_eq!(steps_value(&b, 0), "Step 1 of 16: rest, C");
        assert_eq!(steps_value(&b, 4), "Step 5 of 16: C sharp, accent");
        let b = with_length(&b, 8);
        assert_eq!(steps_value(&b, 11), "Step 12, past the 8 steps: rest, C");
        assert_eq!(step_said(&b, 4), "Step 5: C sharp, accent");
        assert_eq!(length_value(&b), "8 steps");
        assert_eq!(length_value(&with_length(&b, 1)), "1 step");
    }

    #[test]
    fn f2_reads_the_editor_s_pattern() {
        let b = with_flag(&with_note(&empty(), 0, 0), 0, Flag::Accent, true);
        let b = with_flag(&with_note(&b, 2, 3), 2, Flag::Slide, true);
        let b = with_shuffle(&with_length(&b, 3), true);
        assert_eq!(
            readout(1, 2, &b),
            "303-2, pattern A3, 3 steps, shuffle on. 1 C accent, 2 rest, 3 D sharp slide"
        );
    }

    #[test]
    fn the_step_keys_are_the_panel_s() {
        let none = Modifiers::NONE;
        let key = |k| step_key(0, k, Some(k), none, false);
        assert_eq!(
            key(Key::ArrowDown),
            Some(StepKey::Move(step303::Move::Step))
        );
        assert_eq!(key(Key::ArrowUp), Some(StepKey::Move(step303::Move::Back)));
        assert_eq!(key(Key::Home), Some(StepKey::Move(step303::Move::First)));
        assert_eq!(key(Key::End), Some(StepKey::Move(step303::Move::Last)));
        assert_eq!(
            key(Key::ArrowLeft),
            Some(StepKey::Move(step303::Move::Nothing))
        );
        assert_eq!(
            key(Key::ArrowRight),
            Some(StepKey::Move(step303::Move::Nothing))
        );
        assert_eq!(key(Key::Enter), Some(StepKey::Move(step303::Move::Step)));
        assert_eq!(
            key(Key::Backspace),
            Some(StepKey::Move(step303::Move::Back))
        );
        assert_eq!(key(Key::C), Some(StepKey::Edit(StepEdit::Note(0))));
        assert_eq!(key(Key::F), Some(StepKey::Edit(StepEdit::Note(1))));
        assert_eq!(key(Key::Slash), Some(StepKey::Edit(StepEdit::Note(12))));
        assert_eq!(key(Key::Minus), Some(StepKey::Edit(StepEdit::Rest)));
        assert_eq!(key(Key::Equals), Some(StepKey::Edit(StepEdit::Rest)));
        assert_eq!(
            key(Key::P),
            Some(StepKey::Edit(StepEdit::Flag(Flag::Accent)))
        );
        assert_eq!(
            key(Key::OpenBracket),
            Some(StepKey::Edit(StepEdit::Flag(Flag::Slide)))
        );
        assert_eq!(
            key(Key::Quote),
            Some(StepKey::Edit(StepEdit::Flag(Flag::Up)))
        );
        assert_eq!(
            key(Key::Semicolon),
            Some(StepKey::Edit(StepEdit::Flag(Flag::Down)))
        );
        // the 303-2 window has the same keys
        assert_eq!(
            step_key(1, Key::P, Some(Key::P), none, false),
            Some(StepKey::Edit(StepEdit::Flag(Flag::Accent)))
        );
        // not the window's: Tab, Space, the function keys, a key with Ctrl, a repeated note
        assert_eq!(key(Key::Tab), None);
        assert_eq!(key(Key::Space), None);
        assert_eq!(key(Key::F5), None);
        assert_eq!(key(Key::PageDown), None);
        assert_eq!(
            step_key(0, Key::Z, Some(Key::Z), Modifiers::CTRL, false),
            None
        );
        assert_eq!(step_key(0, Key::C, Some(Key::C), none, true), None);
        assert_eq!(
            step_key(0, Key::ArrowDown, Some(Key::ArrowDown), none, true),
            Some(StepKey::Move(step303::Move::Step)),
            "the moves repeat"
        );
    }

    #[test]
    fn the_moves_wrap_at_16_and_end_at_the_length() {
        use step303::Move;
        assert_eq!(moved(0, Move::Back, 16), 15);
        assert_eq!(moved(15, Move::Step, 16), 0);
        assert_eq!(moved(5, Move::Last, 8), 7);
        assert_eq!(moved(5, Move::First, 8), 0);
        assert_eq!(moved(5, Move::Nothing, 8), 5);
    }

    #[test]
    fn the_spin_buttons_step_as_the_panel_s() {
        use drive::Step;
        assert_eq!(pattern_step(0, Step::Up), 1);
        assert_eq!(pattern_step(7, Step::Up), 8, "A8 to B1");
        assert_eq!(pattern_step(0, Step::Down), 0, "A1 stays");
        assert_eq!(pattern_step(31, Step::Up), 31, "D8 stays");
        assert_eq!(pattern_step(3, Step::PageUp), 11);
        assert_eq!(pattern_step(3, Step::PageDown), 0);
        assert_eq!(pattern_step(12, Step::Home), 0);
        assert_eq!(pattern_step(12, Step::End), 31);
        assert_eq!(length_step(16, Step::Up), 16);
        assert_eq!(length_step(16, Step::Down), 15);
        assert_eq!(length_step(3, Step::PageUp), 13);
        assert_eq!(length_step(3, Step::PageDown), 1);
        assert_eq!(length_step(8, Step::Home), 1);
        assert_eq!(length_step(8, Step::End), 16);
    }

    #[test]
    fn transpose_and_shift_say_what_they_did() {
        use crate::pattern_editor::{shifted, transposed};
        let e = empty();
        let b = with_note(&e, 0, 4);
        let up = transposed(&b, true);
        assert_eq!(
            act_said(&Act::Transpose { up: true }, 0, &b, &up).as_deref(),
            Some("Pattern A1 transposed up")
        );
        let left = shifted(&b, false);
        assert_eq!(
            act_said(&Act::Shift { right: false }, 9, &b, &left).as_deref(),
            Some("Pattern B2 shifted left")
        );
        // nothing changed (every step alike): said truthfully
        assert_eq!(shifted(&e, true), e);
        assert_eq!(
            act_said(&Act::Shift { right: true }, 0, &e, &e).as_deref(),
            Some("Pattern A1 unchanged")
        );
        assert_eq!(
            act_said(&Act::Transpose { up: false }, 0, &e, &e).as_deref(),
            Some("Pattern A1 unchanged")
        );
        assert_eq!(act_said(&Act::Copy, 0, &e, &e), None);
    }

    /// Each programming key is judged on the bytes around it: of two keys of one frame on one
    /// step, the first a change and the second none, only the second is said.
    #[test]
    fn each_key_is_judged_on_its_own() {
        use crate::pattern_editor::with_step_edit;
        let e = empty();
        let key = |n| Act::StepEdit {
            step: 0,
            edit: StepEdit::Note(n),
        };
        // F (C sharp) on a rest: the value changes; the Step editor's value speaks
        let one = with_step_edit(&e, 0, StepEdit::Note(1), false);
        assert_eq!(act_said(&key(1), 0, &e, &one), None);
        // F again in the same frame: as before, the announcer says the step
        let two = with_step_edit(&one, 0, StepEdit::Note(1), false);
        assert_eq!(
            act_said(&key(1), 0, &one, &two).as_deref(),
            Some("Step 1: rest, C sharp")
        );
        // judged on the frame's first bytes, the second key would have been taken as a change
        assert_ne!(steps_value(&e, 0), steps_value(&two, 0));
    }

    #[test]
    fn every_control_has_its_own_id() {
        let mut ids: Vec<_> = (0..2)
            .flat_map(|k| Part::ALL.map(move |p| id(k, p)))
            .collect();
        assert!(ids.iter().all(|&i| owner(i).is_some()));
        assert_eq!(owner(id(1, Part::Clear)), Some(1));
        assert_eq!(owner(egui::Id::unique("other")), None);
        ids.sort_by_key(|i| i.value());
        ids.dedup();
        assert_eq!(ids.len(), 22);
    }
}
