//! The 808 and 909 pattern editor windows in screen reader mode (task-236, the user's decision
//! task-243, option A: the whole window; docs/ui_spec.md section 20.9).
//!
//! With the mode on, the window of `crate::drum_editor` draws this content instead of its header
//! and painted grid (with the mode off it is upstream's, untouched): named controls in one tab
//! order, each operable from the keyboard, as the 303 editor's (`super::editor303`, task-235).
//!
//! - **Pattern**, a spin button ("A1"), as the 303 window's: Up and Down the next and the previous
//!   of the 32 patterns, Page Up and Page Down a bank, Home and End A1 and D8.
//! - **Steps**, the grid of the 12 instruments (Accent first, then the 11 drum rows, as the panel's
//!   grid) by the pattern's length, with the panel grid's keys and words (`super::grid`, task-191):
//!   Up and Down the instrument, Left and Right the step, Home and End step 1 and the last, Space
//!   sets the cell (on the 909 with Ctrl, Alt, Shift and Ctrl+Alt).
//! - The 909's **Entry** (spin button: Cycle, Low, High, Flam), what a plain Space enters.
//! - **Length** (spin button, "16 steps"), **Shuffle** (check box), **Shift left**, **Shift
//!   right**, **Copy**, **Paste** and **Clear**.
//!
//! Every edit is a [`DrumAct`] run by `App::drum_editor_act`, the window's one path to
//! `App::write_pattern`. In screen reader mode the window follows the pattern the machine's grid
//! edits (`Panel::edit_pattern`), not the one playback moves the row to.

use egui::{self, Modifiers, Rect, Sense, Ui, accesskit, vec2};

use super::editor303::{control, length_step, named_button, pattern_step, spin_node, value_key};
use super::grid::{self, GridKey, ROWS};
use super::{drive, names};
use crate::behavior::Mods;
use crate::drum_editor::{self, DrumAct, Entry, Machine};
use crate::editor_frame::STEPS;
use crate::pattern_editor;

/// The window's controls, in their tab order (the 808's without [`Part::Entry`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Part {
    Pattern,
    Steps,
    Entry,
    Length,
    Shuffle,
    ShiftLeft,
    ShiftRight,
    Copy,
    Paste,
    Clear,
}

impl Part {
    pub const ALL: [Part; 10] = [
        Part::Pattern,
        Part::Steps,
        Part::Entry,
        Part::Length,
        Part::Shuffle,
        Part::ShiftLeft,
        Part::ShiftRight,
        Part::Copy,
        Part::Paste,
        Part::Clear,
    ];

    /// The controls of machine `m`'s window, in their tab order.
    pub fn of(m: Machine) -> Vec<Part> {
        Part::ALL
            .into_iter()
            .filter(|&p| p != Part::Entry || m.has_entry())
            .collect()
    }

    /// The control's name, as the screen reader says it.
    pub fn name(self) -> &'static str {
        match self {
            Part::Pattern => "Pattern",
            Part::Steps => "Steps",
            Part::Entry => "Entry",
            Part::Length => "Length",
            Part::Shuffle => "Shuffle",
            Part::ShiftLeft => "Shift left",
            Part::ShiftRight => "Shift right",
            Part::Copy => "Copy",
            Part::Paste => "Paste",
            Part::Clear => "Clear",
        }
    }
}

/// The egui id of control `part` of machine `m`'s window (of [`Part::Steps`], the grid's `Ui`
/// and node).
pub fn id(m: Machine, part: Part) -> egui::Id {
    egui::Id::unique(("orb_drum_editor_sr", m.index(), part as u8))
}

/// The egui id of the grid's cell of instrument `i` (0 Accent) and step `step` (0-based).
pub fn cell_id(m: Machine, i: usize, step: usize) -> egui::Id {
    egui::Id::unique(("orb_drum_editor_sr_cell", m.index(), i, step))
}

/// The machine whose editor window has the control of egui id `f`.
pub fn owner(f: egui::Id) -> Option<Machine> {
    Machine::ALL
        .into_iter()
        .find(|&m| Part::of(m).iter().any(|&p| id(m, p) == f))
}

/// The names of a machine's 12 instruments, Accent first, as the panel's grid names them now
/// (`grid::instrument_name`: the 808's slots 3..7 by their switches).
pub type Names = [String; ROWS];

/// The cell of instrument `i` (0 Accent, the step byte) and step `step` of `bytes`, in the panel
/// grid's words: "Bass drum, step 5, on", on the 909 "accent", "flam", "accent and flam".
pub fn cell_text(m: Machine, names: &Names, bytes: &[u8], i: usize, step: usize) -> String {
    grid::cell_words(&names[i], step, word(m, bytes, i, step))
}

fn word(m: Machine, bytes: &[u8], i: usize, step: usize) -> &'static str {
    grid::state_word(m == Machine::Tr808, i, drum_editor::cell(bytes, step, i))
}

/// The Entry value: the switch's name, with the word the panel's grid says for the value it
/// enters ("Low (on)", "High (accent)").
pub fn entry_value(e: Entry) -> &'static str {
    match e {
        Entry::Cycle => "Cycle",
        Entry::Low => "Low (on)",
        Entry::High => "High (accent)",
        Entry::Flam => "Flam",
    }
}

/// The entry a value key `s` on the Entry spin button chooses from `e`: Up the next of Cycle,
/// Low, High, Flam, Down the previous (stopping at the ends), Page Up and Page Down as Up and
/// Down, Home Cycle, End Flam.
pub fn entry_step(e: Entry, s: drive::Step) -> Entry {
    let i = Entry::ALL.iter().position(|&x| x == e).unwrap_or(0);
    let last = Entry::ALL.len() - 1;
    let to = match s {
        drive::Step::Up | drive::Step::PageUp => (i + 1).min(last),
        drive::Step::Down | drive::Step::PageDown => i.saturating_sub(1),
        drive::Step::Home => 0,
        drive::Step::End => last,
    };
    Entry::ALL[to]
}

/// What Space with the modifiers `mods` enters on the 909 (the 808 reads none): Ctrl Low, Alt
/// High, both Flam, as the panel grid's Space and the mouse's held keys; without them the
/// window's Entry `entry`.
pub fn key_entry(mods: Mods, entry: Entry) -> Entry {
    match (mods.ctrl, mods.alt) {
        (true, true) => Entry::Flam,
        (true, false) => Entry::Low,
        (false, true) => Entry::High,
        (false, false) => entry,
    }
}

/// F2 with the focus in machine `m`'s window: its pattern `p` as the panel's F2 reads a drum
/// row, from the editor's pattern: "909, pattern A1, 16 steps, shuffle off. Bass drum: 1, 5
/// accent" ("No steps" for an empty one).
pub fn readout(m: Machine, p: u8, names: &Names, bytes: &[u8]) -> String {
    let len = pattern_editor::length(bytes);
    let steps = grid::readout_of(len, |i| names[i].clone(), |i, s| word(m, bytes, i, s));
    format!(
        "{}, pattern {}, {}, shuffle {}. {steps}",
        m.name(),
        names::pattern_text(p),
        super::editor303::length_value(bytes),
        if bytes[0] != 0 { "on" } else { "off" },
    )
}

/// What the announcer says after the act `a` on pattern `p` of machine `m`'s window, which held
/// `before` and holds `after`, when the act's result is not the value of a focused control:
///
/// - Shift: what it did ("Pattern A1 shifted right"), or "Pattern A1 unchanged" when the bytes are
///   as they were (every step alike), so that the key is known to be taken;
/// - Space on a cell whose text reads as before: the cell ("Bass drum, step 1, on"); a change of
///   the cell is the grid's own to speak (its active cell's name changes).
///
/// (Copy, Paste and Clear put their own words on the status line, which speaks them; Pattern,
/// Entry, Length and Shuffle are spoken by the screen reader as their value changes.)
pub fn act_said(
    a: &DrumAct,
    m: Machine,
    p: u8,
    names: &Names,
    before: &[u8],
    after: &[u8],
) -> Option<String> {
    let name = names::pattern_text(p);
    match a {
        DrumAct::CellKey { column, step, .. } => {
            let (i, s) = ((*column).min(ROWS - 1), (*step).min(STEPS - 1));
            let text = cell_text(m, names, after, i, s);
            (cell_text(m, names, before, i, s) == text).then_some(text)
        }
        DrumAct::Shift { .. } if before == after => Some(format!("Pattern {name} unchanged")),
        DrumAct::Shift { right } => Some(format!(
            "Pattern {name} shifted {}",
            if *right { "right" } else { "left" }
        )),
        _ => None,
    }
}

/// What the window's content gave in a frame.
#[derive(Debug, Default)]
pub struct Out {
    /// What it asks for, in order.
    pub acts: Vec<DrumAct>,
    /// Escape with the focus in the window: it closes.
    pub close: bool,
}

/// What the window shows: the pattern `p`, its `bytes`, the instruments' names and the 909's
/// Entry.
pub struct View<'a> {
    pub p: u8,
    pub bytes: &'a [u8],
    pub names: &'a Names,
    pub entry: Entry,
}

/// The window's own state, kept by the app: the grid's row (0 Accent) and step, and whether a
/// control of the window had the focus at the end of the last frame.
pub struct State<'a> {
    pub row: &'a mut usize,
    pub step: &'a mut usize,
    pub had_focus: &'a mut bool,
}

/// The size of a cell of the grid's picture.
const CELL: egui::Vec2 = vec2(18.0, 13.0);

/// Machine `m`'s window in screen reader mode. The window (egui id `window`) is made a Dialog
/// named `title`; with `due` the focus goes to its first control, Pattern.
pub fn window_ui(
    ui: &mut Ui,
    m: Machine,
    window: egui::Id,
    title: &str,
    due: bool,
    v: View,
    mut s: State,
) -> Out {
    crate::app::focus_into_window(ui, due);
    super::window::window(ui, true, window, title);
    let mut out = Out::default();
    // (egui drops the focus on Escape as the frame begins: the last frame's focus counts)
    let focused = ui.memory(|mem| mem.focused()).and_then(owner) == Some(m);
    if (focused || *s.had_focus)
        && ui.input_mut(|i| i.consume_key(Modifiers::NONE, egui::Key::Escape))
    {
        out.close = true;
    }
    let len = pattern_editor::length(v.bytes);
    *s.row = (*s.row).min(ROWS - 1);
    *s.step = (*s.step).min(len - 1);

    // Pattern
    ui.horizontal(|ui| {
        let pid = id(m, Part::Pattern);
        let text = names::pattern_text(v.p);
        let r = control(ui, pid, &format!("Pattern {text}"), false, 0.0);
        spin_node(ui, pid, Part::Pattern.name(), &text);
        if let Some(st) = value_key(ui, &r, pid) {
            let to = pattern_step(v.p, st);
            if to != v.p {
                out.acts.push(DrumAct::Select(to));
            }
        }
    });
    // the grid
    steps_grid(ui, m, &v, &mut s, len, &mut out);
    ui.horizontal(|ui| {
        if m.has_entry() {
            let eid = id(m, Part::Entry);
            let value = entry_value(v.entry);
            let r = control(ui, eid, &format!("Entry {value}"), false, 0.0);
            spin_node(ui, eid, Part::Entry.name(), value);
            if let Some(st) = value_key(ui, &r, eid) {
                let to = entry_step(v.entry, st);
                if to != v.entry {
                    out.acts.push(DrumAct::Entry(to));
                }
            }
        }
        let lid = id(m, Part::Length);
        let value = super::editor303::length_value(v.bytes);
        let r = control(ui, lid, &format!("Length {value}"), false, 0.0);
        spin_node(ui, lid, Part::Length.name(), &value);
        if let Some(st) = value_key(ui, &r, lid) {
            let n = length_step(len, st);
            if usize::from(n) != len {
                out.acts.push(DrumAct::Length { n, drag: false });
            }
        }
        let shid = id(m, Part::Shuffle);
        let on = v.bytes[0] != 0;
        let r = control(ui, shid, Part::Shuffle.name(), on, 0.0);
        ui.ctx().accesskit_node_builder(shid, |b| {
            b.set_role(accesskit::Role::CheckBox);
            b.set_label(Part::Shuffle.name());
            b.set_toggled(on.into());
        });
        if r.clicked() {
            out.acts.push(DrumAct::Shuffle);
        }
    });
    ui.horizontal(|ui| {
        for (part, act) in [
            (Part::ShiftLeft, DrumAct::Shift { right: false }),
            (Part::ShiftRight, DrumAct::Shift { right: true }),
            (Part::Copy, DrumAct::Copy),
            (Part::Paste, DrumAct::Paste),
            (Part::Clear, DrumAct::Clear),
        ] {
            if named_button(ui, id(m, part), part.name()) {
                out.acts.push(act);
            }
        }
    });
    *s.had_focus = ui.memory(|mem| mem.focused()).and_then(owner) == Some(m);
    out
}

/// The grid "Steps": a child `Ui` that takes the focus (its id the control's), whose node is the
/// Grid of the 12 instruments by `len` steps with the cursor's cell its active descendant, as the
/// panel's grid (`App::drum_grid`); a small picture of the pattern with the cursor outlined.
fn steps_grid(ui: &mut Ui, m: Machine, v: &View, s: &mut State, len: usize, out: &mut Out) {
    let gid = id(m, Part::Steps);
    let size = vec2(CELL.x * STEPS as f32, CELL.y * ROWS as f32);
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    let g = ui.new_child(
        egui::UiBuilder::new()
            .scope_id(gid)
            .max_rect(rect)
            .sense(Sense::FOCUSABLE)
            .accessibility_role(accesskit::Role::Grid)
            .accessibility_label(Part::Steps.name()),
    );
    let has_focus = g.memory(|mem| mem.has_focus(gid));
    if has_focus {
        // the arrows are the grid's, not egui's focus moves
        let filter = egui::EventFilter {
            vertical_arrows: true,
            horizontal_arrows: true,
            ..Default::default()
        };
        g.memory_mut(|mem| mem.set_focus_lock_filter(gid, filter));
        for e in g.input(|i| i.events.clone()) {
            let egui::Event::Key {
                key,
                pressed: true,
                repeat,
                modifiers,
                ..
            } = e
            else {
                continue;
            };
            match GridKey::of(key, modifiers) {
                // Space ignores auto-repeat, as the panel's grid
                Some(GridKey::Set(mods)) if !repeat => out.acts.push(DrumAct::CellKey {
                    column: *s.row,
                    step: *s.step,
                    entry: key_entry(mods, v.entry),
                }),
                Some(GridKey::Set(_)) | None => {}
                Some(k @ (GridKey::Up | GridKey::Down)) => *s.row = grid::row_moved(*s.row, k),
                Some(k) => *s.step = grid::step_moved(*s.step, k, len),
            }
        }
    }
    let cell = |i: usize, st: usize| {
        Rect::from_min_size(rect.min + vec2(st as f32 * CELL.x, i as f32 * CELL.y), CELL)
    };
    let ctx = g.ctx().clone();
    grid::grid_node(&ctx, gid, len, cell_id(m, *s.row, *s.step), rect);
    for i in 0..ROWS {
        for st in 0..len {
            let r = cell(i, st);
            g.interact(r, cell_id(m, i, st), Sense::hover());
            let text = cell_text(m, v.names, v.bytes, i, st);
            grid::cell_node(&ctx, cell_id(m, i, st), text, (i, st), r);
        }
    }
    if g.is_rect_visible(rect) {
        let painter = g.painter();
        let vis = g.visuals();
        painter.rect_filled(rect, 0.0, vis.extreme_bg_color);
        for i in 0..ROWS {
            for st in 0..STEPS {
                let c = m.color(drum_editor::cell(v.bytes, st, i));
                if let Some(c) = c {
                    let c = crate::editor_frame::dimmed(c, st >= len);
                    painter.rect_filled(cell(i, st).shrink(2.0), 1.0, c);
                }
            }
        }
        let stroke = if has_focus {
            vis.selection.stroke
        } else {
            vis.widgets.noninteractive.bg_stroke
        };
        painter.rect_stroke(cell(*s.row, *s.step), 1.0, stroke, egui::StrokeKind::Inside);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::drum_editor::{AC, PATTERN_BYTES, shifted, with_cell};
    use crate::model::Panel;

    fn empty() -> Vec<u8> {
        let mut b = vec![0u8; PATTERN_BYTES];
        b[1] = 16;
        b
    }

    fn names(m: Machine) -> Names {
        let p = Panel::new();
        std::array::from_fn(|i| grid::instrument_name(&p, m == Machine::Tr808, i))
    }

    #[test]
    fn the_cells_read_as_the_panel_s_grid() {
        let m = Machine::Tr909;
        let n = names(m);
        let b = with_cell(&with_cell(&empty(), 4, 1, 2), 8, 1, 3);
        let b = with_cell(&with_cell(&b, 12, 1, 4), 0, 1, 1);
        let b = with_cell(&b, 2, AC, 2);
        assert_eq!(cell_text(m, &n, &b, 1, 0), "Bass drum, step 1, on");
        assert_eq!(cell_text(m, &n, &b, 1, 4), "Bass drum, step 5, accent");
        assert_eq!(cell_text(m, &n, &b, 1, 8), "Bass drum, step 9, flam");
        assert_eq!(cell_text(m, &n, &b, 0, 2), "Accent, step 3, on");
        assert_eq!(cell_text(m, &n, &b, 2, 2), "Snare drum, step 3, off");
        // the panel's grid says the same steps in the same words
        let mut p = Panel::new();
        p.steps909 = [[0; 16]; 12];
        for (i, s) in [(1, 0), (1, 4), (1, 8), (1, 12), (0, 2)] {
            p.steps909[i][s] = drum_editor::cell(&b, s, i);
        }
        assert_eq!(
            readout(m, 0, &n, &b),
            format!(
                "909, pattern A1, 16 steps, shuffle off. {}",
                grid::drum_readout(&p, false)
            )
        );
        assert_eq!(
            readout(m, 0, &n, &b),
            "909, pattern A1, 16 steps, shuffle off. Accent: 3. Bass drum: 1, 5 accent, 9 flam, \
             13 accent and flam"
        );
        // the 808: any byte but 0 is on; the slots by their switches
        let m = Machine::Tr808;
        let n = names(m);
        let b = with_cell(&empty(), 0, 11, 1);
        assert_eq!(cell_text(m, &n, &b, 11, 0), "Closed hi-hat, step 1, on");
        assert_eq!(n[3], grid::instrument_name(&Panel::new(), true, 3));
        assert_eq!(
            readout(m, 9, &n, &pattern_editor::with_length(&empty(), 4)),
            "808, pattern B2, 4 steps, shuffle off. No steps"
        );
    }

    #[test]
    fn the_entry_and_space_s_modifiers() {
        use drive::Step;
        assert_eq!(entry_step(Entry::Cycle, Step::Up), Entry::Low);
        assert_eq!(entry_step(Entry::Flam, Step::Up), Entry::Flam);
        assert_eq!(entry_step(Entry::Cycle, Step::Down), Entry::Cycle);
        assert_eq!(entry_step(Entry::High, Step::Home), Entry::Cycle);
        assert_eq!(entry_step(Entry::Low, Step::End), Entry::Flam);
        let mods = |ctrl, alt| Mods { ctrl, alt };
        assert_eq!(key_entry(mods(false, false), Entry::High), Entry::High);
        assert_eq!(key_entry(mods(true, false), Entry::High), Entry::Low);
        assert_eq!(key_entry(mods(false, true), Entry::Cycle), Entry::High);
        assert_eq!(key_entry(mods(true, true), Entry::Cycle), Entry::Flam);
        // the entry enters the word Entry's value names
        let m = Machine::Tr909;
        for (e, w) in [
            (Entry::Low, "on"),
            (Entry::High, "accent"),
            (Entry::Flam, "flam"),
        ] {
            let v = m.clicked(1, 0, e);
            assert_eq!(grid::state_word(false, 1, v), w);
            assert!(entry_value(e).contains(w) || e == Entry::Flam);
        }
    }

    #[test]
    fn shift_and_a_cell_say_only_what_the_value_cannot() {
        let m = Machine::Tr909;
        let n = names(m);
        let e = empty();
        let b = with_cell(&e, 0, 1, 1);
        let right = shifted(&b, true);
        let shift = |right| DrumAct::Shift { right };
        assert_eq!(
            act_said(&shift(true), m, 0, &n, &b, &right).as_deref(),
            Some("Pattern A1 shifted right")
        );
        assert_eq!(
            act_said(&shift(false), m, 9, &n, &right, &b).as_deref(),
            Some("Pattern B2 shifted left")
        );
        assert_eq!(
            act_said(&shift(true), m, 0, &n, &e, &e).as_deref(),
            Some("Pattern A1 unchanged")
        );
        let key = DrumAct::CellKey {
            column: 1,
            step: 0,
            entry: Entry::Cycle,
        };
        // a change: the grid's active cell says it
        assert_eq!(act_said(&key, m, 0, &n, &e, &b), None);
        // none (the same word): the announcer says the cell
        let three = with_cell(&e, 0, 1, 3);
        let five = with_cell(&e, 0, 1, 5);
        assert_eq!(
            act_said(&key, m, 0, &n, &three, &five).as_deref(),
            Some("Bass drum, step 1, flam")
        );
        assert_eq!(act_said(&DrumAct::Copy, m, 0, &n, &e, &e), None);
    }

    /// Space always changes the cell's word, whatever byte a song stored, on every row, with
    /// every entry, on both machines: `act_said`'s "cell unchanged" case is a guard only.
    #[test]
    fn space_always_changes_the_cell_s_word() {
        for m in Machine::ALL {
            for i in 0..ROWS {
                for v in 0..=255u8 {
                    for e in Entry::ALL {
                        let to = m.clicked(i, v, e);
                        let is808 = m == Machine::Tr808;
                        assert_ne!(
                            grid::state_word(is808, i, v),
                            grid::state_word(is808, i, to),
                            "{m:?} row {i} byte {v} {e:?} -> {to}"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn every_control_has_its_own_id() {
        assert_eq!(Part::of(Machine::Tr808).len(), 9);
        assert_eq!(Part::of(Machine::Tr909).len(), 10);
        assert!(!Part::of(Machine::Tr808).contains(&Part::Entry));
        let mut ids: Vec<_> = Machine::ALL
            .into_iter()
            .flat_map(|m| Part::of(m).into_iter().map(move |p| id(m, p)))
            .collect();
        assert!(ids.iter().all(|&i| owner(i).is_some()));
        assert_eq!(owner(id(Machine::Tr909, Part::Clear)), Some(Machine::Tr909));
        assert_eq!(owner(id(Machine::Tr808, Part::Entry)), None);
        assert_eq!(
            owner(super::super::editor303::id(
                0,
                super::super::editor303::Part::Clear
            )),
            None
        );
        ids.sort_by_key(|i| i.value());
        ids.dedup();
        assert_eq!(ids.len(), 19);
    }
}
