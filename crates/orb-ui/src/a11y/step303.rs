//! The 303 step editor, spoken (task-190, A11Y-6 of docs/accessibility_plan.md,
//! docs/accessibility.md 3.3, docs/ui_spec.md section 20.5).
//!
//! In screen reader mode the 303's edit step display (`tb303_N.edit_step`) is the "Step editor":
//! a focusable node whose value is the step being edited and what it holds ("Step 5 of 16: C
//! sharp, octave up, accent, slide"), with ", pitch mode" while the original's pitch mode is on.
//! Down and Up are the panel's Step and Back buttons (the next and the previous step, as in a
//! list), Left and Right do nothing, Home and End go to the first and the pattern's last step
//! ([`move_editor`], through `Panel::press`, as the mouse; the user's convention, decision
//! task-209). After each programming key the step it changed is spoken ([`press_effect`]), and
//! F2 reads the whole pattern ([`read_303_pattern`]).
//!
//! The editor works on its own pattern, [`Synth303::pin`](crate::model::Synth303::pin), which
//! `App` keeps: playback moving the row to another pattern does not change what the editor
//! holds, so the focused editor never speaks without input ("move-along doesn't speak",
//! task-209). Off (Options > Step Editor Follows Playback, the default), the pin moves only
//! when the user chooses a pattern; on, it moves to the playing pattern at the next input.

use eframe::egui::{self, Key, accesskit};

use super::{hit_id_of, names};
use crate::model::{Panel, Step303};

/// The name of the step editor's node.
pub const EDITOR_NAME: &str = "Step editor";

/// The note names of the 303's keys, index 0 (low C) to 12 (high C), as the panel's keyboard
/// (docs/ui_spec.md section 5).
const NOTES: [&str; 13] = [
    "C", "C sharp", "D", "D sharp", "E", "F", "F sharp", "G", "G sharp", "A", "A sharp", "B",
    "high C",
];

/// The spoken name of the note `n` (0 = C .. 12 = high C).
pub fn note_name(n: u8) -> &'static str {
    NOTES.get(usize::from(n)).copied().unwrap_or("C")
}

/// The 303 (0 or 1) whose step editor is the display `layout_id`.
pub fn editor_of_layout(layout_id: &str) -> Option<usize> {
    match layout_id {
        "tb303_1.edit_step" => Some(0),
        "tb303_2.edit_step" => Some(1),
        _ => None,
    }
}

/// The egui id of the step editor of 303 `k` (the display's hit area).
pub fn editor_id(k: usize) -> egui::Id {
    hit_id_of("display", &format!("tb303_{}.edit_step", k + 1))
}

/// The 303 whose step editor has the egui id `f`.
pub fn editor_of(f: egui::Id) -> Option<usize> {
    (0..2).find(|&k| editor_id(k) == f)
}

/// What a step holds, for the editor: "C sharp, octave up, accent, slide"; a rest says so first
/// and keeps its note and flags, which the panel's lights show too ("rest, C sharp, accent").
pub fn step_text(s: &Step303) -> String {
    let mut parts: Vec<&str> = Vec::new();
    if s.rest {
        parts.push("rest");
    }
    parts.push(note_name(s.note));
    parts.extend(flags(s));
    parts.join(", ")
}

/// The flags of a step in the panel's order of their buttons, spoken: octave down, octave up,
/// accent, slide.
fn flags(s: &Step303) -> Vec<&'static str> {
    [
        (s.down, "octave down"),
        (s.up, "octave up"),
        (s.accent, "accent"),
        (s.slide, "slide"),
    ]
    .into_iter()
    .filter_map(|(on, w)| on.then_some(w))
    .collect()
}

/// The step `i` (0-based) of 303 `k` in the pattern its editor edits (`Panel::edit_pattern`).
fn step_of(panel: &Panel, k: usize, i: usize) -> &Step303 {
    &panel.synth[k].patterns[usize::from(panel.edit_pattern(k))][i]
}

/// The length of the pattern 303 `k`'s editor edits (1..=16).
fn length(panel: &Panel, k: usize) -> usize {
    let p = usize::from(panel.edit_pattern(k));
    usize::from(panel.rows[k].len[p]).clamp(1, 16)
}

/// The step editor's value: "Step 5 of 16: C sharp, accent", ", pitch mode" at the end while
/// pitch mode is on. An edit step past the pattern's length (the original's Step and Back wrap
/// at 16 whatever the length) says so: "Step 12, past the 8 steps: rest, C".
pub fn editor_value(panel: &Panel, k: usize) -> String {
    let mut text = edit_step_text(panel, k);
    if panel.synth[k].pitch_mode {
        text.push_str(", pitch mode");
    }
    text
}

/// [`editor_value`] without the mode.
fn edit_step_text(panel: &Panel, k: usize) -> String {
    let i = usize::from(panel.synth[k].edit);
    let len = length(panel, k);
    let at = if i < len {
        format!("Step {} of {len}", i + 1)
    } else {
        format!("Step {}, past the {len} steps", i + 1)
    };
    format!("{at}: {}", step_text(step_of(panel, k, i)))
}

/// The step editor's node: a Slider (Up and Down move it, as a vertical slider) named
/// [`EDITOR_NAME`] with [`editor_value`] as its text value. No numeric value: UI Automation
/// would then raise two value changes for one move.
pub fn describe(ctx: &egui::Context, panel: &Panel, k: usize) {
    // (the text is made only while AccessKit is on: the closure runs only then)
    ctx.accesskit_node_builder(editor_id(k), |node| {
        node.set_role(accesskit::Role::Slider);
        node.set_orientation(accesskit::Orientation::Vertical);
        node.set_label(EDITOR_NAME);
        node.set_value(editor_value(panel, k));
    });
}

/// A move of the step editor by a key.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Move {
    /// Up: the Back button, the previous step.
    Back,
    /// Down: the Step button, the next step.
    Step,
    /// Home: step 1.
    First,
    /// End: the pattern's last step.
    Last,
    /// Left and Right: nothing. The editor is a list of steps, and the user's convention
    /// (decision task-209): "if there's nothing for the left and right to do, ... up and down
    /// should move on steps, and left and right do nothing".
    Nothing,
}

impl Move {
    /// The move of a key on the focused step editor (no modifiers); None for a key that is not
    /// the editor's (Page Up and Page Down among them).
    pub fn of_key(key: Key) -> Option<Self> {
        Some(match key {
            Key::ArrowUp => Move::Back,
            Key::ArrowDown => Move::Step,
            Key::Home => Move::First,
            Key::End => Move::Last,
            Key::ArrowLeft | Key::ArrowRight => Move::Nothing,
            _ => return None,
        })
    }
}

/// Moves the edit step of 303 `k` through the panel's Back and Step buttons (`Panel::press`,
/// the mouse's path; they move it by one modulo 16, `Synth303::move_edit`, and send nothing to
/// the engine). Home and End press them until the edit step is the first or the pattern's last.
pub fn move_editor(panel: &mut Panel, k: usize, m: Move, now: f64) {
    let id = |name: &str| format!("tb303_{}.{name}", k + 1);
    let target = match m {
        Move::Nothing => return,
        Move::Back => return panel.press(&id("back"), now),
        Move::Step => return panel.press(&id("step"), now),
        Move::First => 0,
        Move::Last => length(panel, k) - 1,
    };
    // at most 15 presses: Step and Back wrap modulo 16
    let from = usize::from(panel.synth[k].edit);
    let (button, n) = if target >= from {
        ("step", target - from)
    } else {
        ("back", from - target)
    };
    for _ in 0..n {
        panel.press(&id(button), now);
    }
}

/// What a 303 programming press is compared with (`App::act` takes it before the press).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Before {
    edit: [u8; 2],
    editor: [String; 2],
}

impl Before {
    pub fn of(panel: &Panel) -> Self {
        Self {
            edit: [panel.synth[0].edit, panel.synth[1].edit],
            editor: [editor_value(panel, 0), editor_value(panel, 1)],
        }
    }
}

/// What a press of the 303 button `id` (a key of the original or Space on the button) did,
/// to be announced: the step it changed ("Step 5: D sharp, accent"), the edit step it moved to
/// (Step, Back: the editor's value), pitch mode ("Pitch mode on, step 1 of 16: rest, C, pitch
/// mode"), a cleared pattern ("Pattern A1 cleared"). None for another button, and when the step
/// editor of that 303 has the focus (`focused`) and its value changed (for a programming key:
/// on the same step; for Step, Back, Pitch mode and Clear: at all): the screen reader speaks
/// the focused node's value change itself (docs/ui_spec.md 20.3). In pitch
/// mode a note key moves the edit step on, so the step it set is announced even then (the value
/// change then gives the next step).
pub fn press_effect(
    before: &Before,
    panel: &Panel,
    id: &str,
    focused: Option<egui::Id>,
) -> Option<String> {
    let (sec, name) = id.split_once('.')?;
    let k = match sec {
        "tb303_1" => 0,
        "tb303_2" => 1,
        _ => return None,
    };
    let s = &panel.synth[k];
    let value = editor_value(panel, k);
    let changed_focused = focused == Some(editor_id(k)) && value != before.editor[k];
    let edited = name.starts_with("key")
        || matches!(name, "slide" | "accent" | "up" | "down" | "note_pause");
    if edited {
        if changed_focused && s.edit == before.edit[k] {
            return None;
        }
        let i = usize::from(before.edit[k]);
        return Some(format!(
            "Step {}: {}",
            i + 1,
            step_text(step_of(panel, k, i))
        ));
    }
    if changed_focused {
        return None;
    }
    match name {
        "step" | "back" => Some(value),
        "pitch_button" if s.pitch_mode => Some(format!(
            "Pitch mode on, {}",
            lower_first(&edit_step_text(panel, k))
        )),
        "pitch_button" => Some("Pitch mode off".into()),
        "clear" => Some(format!(
            "Pattern {} cleared",
            // the pattern Clear emptied: the editor's (`Panel::press` clears `edit_pattern`)
            names::pattern_text(panel.edit_pattern(k))
        )),
        _ => None,
    }
}

fn lower_first(s: &str) -> String {
    let mut c = s.chars();
    c.next()
        .map(|f| f.to_lowercase().chain(c).collect())
        .unwrap_or_default()
}

/// One step for F2, compactly: "1 C accent", "3 D sharp octave up slide", "2 rest".
fn compact(i: usize, s: &Step303) -> String {
    if s.rest {
        return format!("{} rest", i + 1);
    }
    let mut parts = vec![(i + 1).to_string(), note_name(s.note).to_string()];
    parts.extend(flags(s).into_iter().map(str::to_string));
    parts.join(" ")
}

/// F2 on a 303's section (row `r` 0 or 1): the steps of the pattern its editor edits, in the
/// pattern's length, one after another, "1 C accent, 2 rest, 3 D sharp slide". When playback
/// has moved the row to another pattern than the editor's, that is said first: "Step editor on
/// pattern A1: 1 C accent, ...". None for the drum rows.
pub fn read_303_pattern(panel: &Panel, r: usize) -> Option<String> {
    if r >= 2 {
        return None;
    }
    let steps: Vec<String> = (0..length(panel, r))
        .map(|i| compact(i, step_of(panel, r, i)))
        .collect();
    let steps = steps.join(", ");
    let p = panel.edit_pattern(r);
    Some(if p == panel.rows[r].pattern {
        steps
    } else {
        format!("Step editor on pattern {}: {steps}", names::pattern_text(p))
    })
}

/// What `App` keeps the pin of row `r` at, each frame in screen reader mode before the
/// frame's input acts ([`Panel::pin`]: a 303's step editor, task-190, or a drum grid,
/// task-191). Only a frame with input (`input`) sets it, to the row's pattern: when there is
/// none yet, and when the editor follows playback (`follows`, the setting; the move is then not
/// spoken by itself, the input's own change goes with it). A frame without input leaves it as
/// it is, `None` included: a song just loaded (a new `Panel`, no pin) is on the row's pattern
/// until the first input, so the editor takes the pattern the engine reports for the song, not
/// the one the panel had before that report.
pub fn pin_of(panel: &Panel, r: usize, follows: bool, input: bool) -> Option<u8> {
    let pin = panel.pin(r);
    match pin {
        Some(_) if !(input && follows) => pin,
        None if !input => None,
        _ => Some(panel.rows[r].pattern),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_notes_follow_the_keyboard() {
        assert_eq!(note_name(0), "C");
        assert_eq!(note_name(1), "C sharp");
        assert_eq!(note_name(10), "A sharp");
        assert_eq!(note_name(11), "B");
        assert_eq!(note_name(12), "high C");
    }

    #[test]
    fn steps_read_as_words() {
        let mut s = Step303 {
            note: 1,
            rest: false,
            up: true,
            accent: true,
            slide: true,
            down: false,
        };
        assert_eq!(step_text(&s), "C sharp, octave up, accent, slide");
        assert_eq!(compact(4, &s), "5 C sharp octave up accent slide");
        s.rest = true;
        s.up = false;
        s.slide = false;
        assert_eq!(step_text(&s), "rest, C sharp, accent");
        assert_eq!(compact(5, &s), "6 rest");
        assert_eq!(step_text(&Step303::default()), "rest, C");
    }

    #[test]
    fn the_editor_value() {
        let mut p = Panel::new();
        assert_eq!(editor_value(&p, 0), "Step 1 of 16: rest, C");
        p.press("tb303_1.key3", 0.0);
        p.press("tb303_1.note_pause", 0.0);
        p.press("tb303_1.down", 0.0);
        assert_eq!(editor_value(&p, 0), "Step 1 of 16: D sharp, octave down");
        p.press("tb303_2.pitch_button", 0.0);
        assert_eq!(editor_value(&p, 1), "Step 1 of 16: rest, C, pitch mode");
        p.rows[0].len[0] = 8;
        for _ in 0..11 {
            p.press("tb303_1.step", 0.0);
        }
        assert_eq!(editor_value(&p, 0), "Step 12, past the 8 steps: rest, C");
    }

    #[test]
    fn moves_go_through_back_and_step() {
        let mut p = Panel::new();
        move_editor(&mut p, 0, Move::Back, 0.0);
        assert_eq!(
            p.synth[0].edit, 15,
            "Back from step 1 wraps to 16, as the original"
        );
        move_editor(&mut p, 0, Move::Step, 0.0);
        assert_eq!(p.synth[0].edit, 0);
        p.rows[0].len[0] = 12;
        move_editor(&mut p, 0, Move::Last, 0.0);
        assert_eq!(p.synth[0].edit, 11);
        move_editor(&mut p, 0, Move::First, 0.0);
        assert_eq!(p.synth[0].edit, 0);
        move_editor(&mut p, 0, Move::Nothing, 0.0);
        assert_eq!(p.synth[0].edit, 0);
        assert_eq!(Move::of_key(Key::ArrowDown), Some(Move::Step));
        assert_eq!(Move::of_key(Key::ArrowUp), Some(Move::Back));
        assert_eq!(Move::of_key(Key::ArrowLeft), Some(Move::Nothing));
        assert_eq!(Move::of_key(Key::ArrowRight), Some(Move::Nothing));
        assert_eq!(Move::of_key(Key::PageDown), None);
        assert!(p.events.is_empty(), "the edit step is the panel's own");
        assert_eq!(p.synth[1].edit, 0);
    }

    #[test]
    fn what_a_press_says() {
        let mut p = Panel::new();
        let b = Before::of(&p);
        p.press("tb303_1.key2", 0.0);
        assert_eq!(
            press_effect(&b, &p, "tb303_1.key2", None).as_deref(),
            Some("Step 1: rest, D")
        );
        // the focused editor speaks its own value change
        let ed = Some(editor_id(0));
        assert_eq!(press_effect(&b, &p, "tb303_1.key2", ed), None);
        // the same note again changes nothing: said by the announcer
        let b = Before::of(&p);
        p.press("tb303_1.key2", 0.0);
        assert_eq!(
            press_effect(&b, &p, "tb303_1.key2", ed).as_deref(),
            Some("Step 1: rest, D")
        );
        let b = Before::of(&p);
        p.press("tb303_1.step", 0.0);
        assert_eq!(
            press_effect(&b, &p, "tb303_1.step", None).as_deref(),
            Some("Step 2 of 16: rest, C")
        );
        assert_eq!(press_effect(&b, &p, "tb303_1.step", ed), None);
        let b = Before::of(&p);
        p.press("tb303_1.pitch_button", 0.0);
        assert_eq!(
            press_effect(&b, &p, "tb303_1.pitch_button", None).as_deref(),
            Some("Pitch mode on, step 1 of 16: rest, D")
        );
        // pitch mode: the key's step is said, the edit step has moved on
        let b = Before::of(&p);
        p.press("tb303_1.key12", 0.0);
        assert_eq!(p.synth[0].edit, 1);
        assert_eq!(
            press_effect(&b, &p, "tb303_1.key12", ed).as_deref(),
            Some("Step 1: rest, high C")
        );
        let b = Before::of(&p);
        p.press("tb303_1.clear", 0.0);
        assert_eq!(
            press_effect(&b, &p, "tb303_1.clear", None).as_deref(),
            Some("Pattern A1 cleared")
        );
        // the editor pinned to A3 while the row plays A2: Clear empties A3 and says so
        p.synth[0].pin = Some(2);
        p.adopt_pattern(0, 1);
        p.synth[0].patterns[2][0].accent = true;
        let b = Before::of(&p);
        p.press("tb303_1.clear", 0.0);
        assert!(!p.synth[0].patterns[2][0].accent);
        assert_eq!(
            press_effect(&b, &p, "tb303_1.clear", None).as_deref(),
            Some("Pattern A3 cleared")
        );
        assert_eq!(press_effect(&b, &p, "tb303_1.cutoff", None), None);
        assert_eq!(press_effect(&b, &p, "transport.play", None), None);
    }

    #[test]
    fn f2_reads_the_pattern_in_its_length() {
        let mut p = Panel::new();
        p.rows[1].len[0] = 3;
        p.press("tb303_2.key0", 0.0);
        p.press("tb303_2.note_pause", 0.0);
        p.press("tb303_2.accent", 0.0);
        p.press("tb303_2.step", 0.0);
        p.press("tb303_2.step", 0.0);
        p.press("tb303_2.key3", 0.0);
        p.press("tb303_2.note_pause", 0.0);
        p.press("tb303_2.slide", 0.0);
        assert_eq!(
            read_303_pattern(&p, 1).as_deref(),
            Some("1 C accent, 2 rest, 3 D sharp slide")
        );
        assert_eq!(read_303_pattern(&p, 2), None);
        // the editor pinned to A1 while the row plays A2: said first
        p.synth[1].pin = Some(0);
        p.adopt_pattern(1, 1);
        assert_eq!(
            read_303_pattern(&p, 1).as_deref(),
            Some("Step editor on pattern A1: 1 C accent, 2 rest, 3 D sharp slide")
        );
    }

    #[test]
    fn the_pin_moves_only_as_the_setting_says() {
        let mut p = Panel::new();
        // no pin yet (a new panel: a song just loaded): none until input, then the row's
        p.adopt_pattern(0, 4);
        assert_eq!(pin_of(&p, 0, false, false), None);
        assert_eq!(pin_of(&p, 0, true, false), None);
        assert_eq!(pin_of(&p, 0, false, true), Some(4));
        assert_eq!(pin_of(&p, 0, true, true), Some(4));
        p.synth[0].pin = Some(0);
        p.adopt_pattern(0, 5);
        assert_eq!(pin_of(&p, 0, false, false), Some(0));
        assert_eq!(
            pin_of(&p, 0, false, true),
            Some(0),
            "off: input does not move it"
        );
        assert_eq!(pin_of(&p, 0, true, false), Some(0), "on: not without input");
        assert_eq!(pin_of(&p, 0, true, true), Some(5), "on: at the next input");
        // a pattern chosen on the panel moves the pin, and the editor edits there
        p.press("pattern1.pattern3", 0.0);
        assert_eq!(p.synth[0].pin, Some(2));
        assert_eq!(p.edit_pattern(0), 2);
        p.adopt_pattern(0, 7);
        p.press("tb303_1.key4", 0.0);
        assert_eq!(p.synth[0].patterns[2][0].note, 4);
        assert_eq!(p.synth[0].patterns[7][0].note, 0);
    }
}
