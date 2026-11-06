//! The drum grids of screen reader mode (task-191, A11Y-7, docs/accessibility.md 3.3,
//! docs/ui_spec.md 20.6): in each drum machine's section one Grid node of 12 rows (the
//! instruments, Accent first) by the pattern's length in steps.
//!
//! The grid's row is the machine's selected instrument (`Panel::inst808` / `inst909`), so Up and
//! Down go through `Panel::select_instrument` and the panel's selection frame follows; its column
//! is a cursor of the app's, kept within the pattern's length. Space sets the step with the calls
//! a mouse down on the step button makes (`set_step808`; `step909_new_value` and `set_step909`,
//! with Ctrl and Alt as on the mouse and the flam button's effect), so the engine hears the same
//! events, a song-mode recording included.

use eframe::egui::{self, Rect, accesskit};

use super::names;
use crate::behavior::Mods;
use crate::model::Panel;

/// The rows of a grid: the 12 instruments of a machine.
pub const ROWS: usize = 12;

/// The 909's instruments in the panel's legend order (RBS column order, docs/ui_spec.md 7).
const NAMES909: [&str; ROWS] = [
    "Accent",
    "Bass drum",
    "Snare drum",
    "Low tom",
    "Mid tom",
    "High tom",
    "Rim shot",
    "Hand clap",
    "Closed hi-hat",
    "Open hi-hat",
    "Crash cymbal",
    "Ride cymbal",
];

/// The 808's instruments (docs/ui_spec.md 6); slots 3..=7 are named by their switch instead.
const NAMES808: [&str; ROWS] = [
    "Accent",
    "Bass drum",
    "Snare drum",
    "",
    "",
    "",
    "",
    "",
    "Cowbell",
    "Cymbal",
    "Open hi-hat",
    "Closed hi-hat",
];

/// The egui id of a machine's grid (the 808's when `is808`): the id of its `Ui` and of its
/// node, and its stop in the focus order.
pub fn grid_id(is808: bool) -> egui::Id {
    egui::Id::unique(("drum grid", is808))
}

/// The machine whose grid is `id`: Some(true) the 808, Some(false) the 909.
pub fn machine_of(id: egui::Id) -> Option<bool> {
    [true, false].into_iter().find(|&m| grid_id(m) == id)
}

/// The egui id of the cell of instrument `row` and step `step` (0-based).
pub fn cell_id(is808: bool, row: usize, step: usize) -> egui::Id {
    egui::Id::unique(("drum cell", is808, row, step))
}

/// The panel's row (2 the 808, 3 the 909) of a machine.
pub fn row_of(is808: bool) -> usize {
    if is808 { 2 } else { 3 }
}

/// The number of steps the grid offers: the length (1..=16) of the pattern it edits, the row's
/// or its pin ([`Panel::edit_pattern`], decision task-209).
pub fn length(panel: &Panel, is808: bool) -> usize {
    let r = row_of(is808);
    usize::from(panel.rows[r].len[usize::from(panel.edit_pattern(r)) % 32]).clamp(1, 16)
}

/// The selected instrument of a machine: the grid's row.
pub fn selected(panel: &Panel, is808: bool) -> usize {
    if is808 { panel.inst808 } else { panel.inst909 }
}

/// An instrument's name as the panel's legend gives it; the 808's slots 3..=7 by the voice
/// their switch plays now ("Low conga" or "Low tom", `names::switch_words`).
pub fn instrument_name(panel: &Panel, is808: bool, i: usize) -> String {
    if !is808 {
        return NAMES909[i].to_string();
    }
    if (3..=7).contains(&i) {
        let id = format!("tr808.slot{i}_switch");
        let word = names::switch_words(&id)[usize::from(panel.switch_position(&id).min(1))];
        let mut c = word.chars();
        return c
            .next()
            .map(|f| f.to_uppercase().collect::<String>() + c.as_str())
            .unwrap_or_default();
    }
    NAMES808[i].to_string()
}

/// The state of a step in words: "off" or "on"; on the 909 "on" (value 1), "accent" (2),
/// "flam" (3 and 5..=255, the flam hit) and "accent and flam" (4, docs/ui_spec.md 7). The
/// accent row reads "on" for any value but 0.
pub fn state_text(panel: &Panel, is808: bool, i: usize, step: usize) -> &'static str {
    if is808 {
        return if panel.steps808[i][step] { "on" } else { "off" };
    }
    match (i, panel.steps909[i][step]) {
        (_, 0) => "off",
        (0, _) | (_, 1) => "on",
        (_, 2) => "accent",
        (_, 4) => "accent and flam",
        _ => "flam",
    }
}

/// A cell's text: "Bass drum, step 5, on".
pub fn cell_text(panel: &Panel, is808: bool, i: usize, step: usize) -> String {
    format!(
        "{}, step {}, {}",
        instrument_name(panel, is808, i),
        step + 1,
        state_text(panel, is808, i, step)
    )
}

/// F2's drum part: each instrument that has steps within the pattern's length, with its
/// steps, "Bass drum: 1, 5, 9, 13. Closed hi-hat: 3, 7, 11, 15"; a 909 step that is not a plain
/// note says what it is ("5 accent", "9 flam"). "No steps" for an empty pattern.
pub fn drum_readout(panel: &Panel, is808: bool) -> String {
    let n = length(panel, is808);
    let mut parts = Vec::new();
    for i in 0..ROWS {
        let steps: Vec<String> = (0..n)
            .filter_map(|s| match state_text(panel, is808, i, s) {
                "off" => None,
                "on" => Some((s + 1).to_string()),
                state => Some(format!("{} {state}", s + 1)),
            })
            .collect();
        if !steps.is_empty() {
            parts.push(format!(
                "{}: {}",
                instrument_name(panel, is808, i),
                steps.join(", ")
            ));
        }
    }
    if parts.is_empty() {
        "No steps".to_string()
    } else {
        parts.join(". ")
    }
}

fn bounds(r: egui::Rect) -> accesskit::Rect {
    accesskit::Rect {
        x0: r.min.x.into(),
        y0: r.min.y.into(),
        x1: r.max.x.into(),
        y1: r.max.y.into(),
    }
}

/// The grid's node: role Grid (its `Ui`'s), named "Steps", 12 rows by the pattern's length,
/// over the step buttons (`rect`), its active descendant the cell of the selected instrument
/// and the cursor `step`: a screen reader speaks that cell as the focus.
pub fn describe_grid(ctx: &egui::Context, panel: &Panel, is808: bool, step: usize, rect: Rect) {
    let n = length(panel, is808);
    let active = cell_id(is808, selected(panel, is808), step.min(n - 1));
    ctx.accesskit_node_builder(grid_id(is808), |node| {
        node.set_role(accesskit::Role::Grid);
        node.set_label("Steps");
        node.set_row_count(ROWS);
        node.set_column_count(n);
        node.set_active_descendant(active.accesskit_id());
        node.set_bounds(bounds(rect));
    });
}

/// A cell's node (`id` [`cell_id`]): role GridCell, its text as the name ("Bass drum, step 5,
/// on"), its row and column, its step button's bounds.
pub fn describe_cell(
    ctx: &egui::Context,
    panel: &Panel,
    is808: bool,
    (i, step): (usize, usize),
    rect: Rect,
) {
    let text = cell_text(panel, is808, i, step);
    ctx.accesskit_node_builder(cell_id(is808, i, step), |node| {
        node.set_role(accesskit::Role::GridCell);
        node.set_label(text);
        node.set_row_index(i);
        node.set_column_index(step);
        node.set_bounds(bounds(rect));
    });
}

/// A key of the grid.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GridKey {
    /// Up: the instrument before (toward Accent).
    Up,
    /// Down: the instrument after.
    Down,
    /// Left: the step before.
    Left,
    /// Right: the step after.
    Right,
    /// Home: the first step.
    Home,
    /// End: the last step of the pattern.
    End,
    /// Space, with the mouse's modifiers: sets the step as a click does.
    Set(Mods),
}

impl GridKey {
    /// The grid's meaning of a key pressed with `modifiers`: the arrows, Home and End without
    /// modifiers; Space with or without Ctrl and Alt (the 909's mouse modifiers; the 808
    /// ignores them, as its mouse does); Shift+Space alone as Alt+Space, the accent key that
    /// does not open the window's system menu (decision task-208).
    pub fn of(key: egui::Key, modifiers: egui::Modifiers) -> Option<Self> {
        if key == egui::Key::Space {
            if modifiers.mac_cmd {
                return None;
            }
            if modifiers.shift {
                return (!modifiers.ctrl && !modifiers.alt).then_some(Self::Set(Mods {
                    ctrl: false,
                    alt: true,
                }));
            }
            return Some(Self::Set(Mods {
                ctrl: modifiers.ctrl,
                alt: modifiers.alt,
            }));
        }
        if modifiers.any() {
            return None;
        }
        Some(match key {
            egui::Key::ArrowUp => Self::Up,
            egui::Key::ArrowDown => Self::Down,
            egui::Key::ArrowLeft => Self::Left,
            egui::Key::ArrowRight => Self::Right,
            egui::Key::Home => Self::Home,
            egui::Key::End => Self::End,
            _ => return None,
        })
    }
}

/// Does what the key `k` asks of a machine's grid whose cursor (the step, 0-based) is `step`.
/// The moves stop at the ends; the cursor stays within the pattern's length.
pub fn key(panel: &mut Panel, is808: bool, step: &mut usize, k: GridKey) {
    let n = length(panel, is808);
    *step = (*step).min(n - 1);
    let i = selected(panel, is808);
    match k {
        GridKey::Up => panel.select_instrument(is808, i.saturating_sub(1)),
        GridKey::Down => panel.select_instrument(is808, (i + 1).min(ROWS - 1)),
        GridKey::Left => *step = step.saturating_sub(1),
        GridKey::Right => *step = (*step + 1).min(n - 1),
        GridKey::Home => *step = 0,
        GridKey::End => *step = n - 1,
        GridKey::Set(m) => set(panel, is808, *step, m),
    }
}

/// Sets the step `step` of the selected instrument as a mouse down on its button does
/// (`App::drum_strips`): the 808 the opposite of its state, the 909 the value
/// `step909_new_value` gives for the modifiers (and the flam button).
pub fn set(panel: &mut Panel, is808: bool, step: usize, m: Mods) {
    if is808 {
        let on = !panel.steps808[panel.inst808][step];
        panel.set_step808(step, on);
    } else {
        let v = panel.step909_new_value(step, m);
        panel.set_step909(step, v);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_follow_the_legend_and_the_slot_switches() {
        let mut p = Panel::new();
        assert_eq!(instrument_name(&p, true, 0), "Accent");
        assert_eq!(instrument_name(&p, true, 11), "Closed hi-hat");
        assert_eq!(instrument_name(&p, false, 10), "Crash cymbal");
        for (i, word) in [(3, "tom"), (6, "Rim shot")] {
            let id = format!("tr808.slot{i}_switch");
            for v in [0, 1] {
                p.set_switch(&id, v);
                let name = instrument_name(&p, true, i);
                let expect = names::switch_words(&id)[usize::from(p.switch_position(&id))];
                assert_eq!(name.to_lowercase(), expect, "{word}");
                assert!(name.chars().next().unwrap().is_uppercase());
            }
        }
    }

    #[test]
    fn cells_and_the_readout() {
        let mut p = Panel::new();
        p.steps909 = [[0; 16]; 12];
        p.steps909[1][0] = 1;
        p.steps909[1][4] = 2;
        p.steps909[1][8] = 3;
        p.steps909[1][12] = 4;
        p.steps909[0][2] = 2;
        assert_eq!(cell_text(&p, false, 1, 4), "Bass drum, step 5, accent");
        assert_eq!(cell_text(&p, false, 1, 8), "Bass drum, step 9, flam");
        assert_eq!(cell_text(&p, false, 0, 2), "Accent, step 3, on");
        assert_eq!(cell_text(&p, false, 2, 2), "Snare drum, step 3, off");
        assert_eq!(
            drum_readout(&p, false),
            "Accent: 3. Bass drum: 1, 5 accent, 9 flam, 13 accent and flam"
        );
        p.rows[3].len[usize::from(p.rows[3].pattern)] = 8;
        assert_eq!(drum_readout(&p, false), "Accent: 3. Bass drum: 1, 5 accent");
        p.steps909 = [[0; 16]; 12];
        assert_eq!(drum_readout(&p, false), "No steps");
    }

    #[test]
    fn moves_stop_at_the_ends_and_the_length() {
        let mut p = Panel::new();
        let mut s = 0;
        key(&mut p, true, &mut s, GridKey::Left);
        assert_eq!(s, 0);
        key(&mut p, true, &mut s, GridKey::End);
        assert_eq!(s, 15);
        p.rows[2].len[usize::from(p.rows[2].pattern)] = 5;
        key(&mut p, true, &mut s, GridKey::Right);
        assert_eq!(s, 4, "the cursor is kept within the length");
        p.select_instrument(true, 0);
        key(&mut p, true, &mut s, GridKey::Up);
        assert_eq!(p.inst808, 0);
        p.select_instrument(true, 11);
        key(&mut p, true, &mut s, GridKey::Down);
        assert_eq!(p.inst808, 11);
        key(&mut p, true, &mut s, GridKey::Up);
        assert_eq!(p.inst808, 10);
    }

    #[test]
    fn the_keys_of_the_grid() {
        use egui::{Key, Modifiers};
        assert_eq!(
            GridKey::of(Key::Space, Modifiers::CTRL | Modifiers::ALT),
            Some(GridKey::Set(Mods {
                ctrl: true,
                alt: true
            }))
        );
        // Shift+Space is the accent key, as Alt+Space (decision task-208); not with Ctrl or Alt
        assert_eq!(
            GridKey::of(Key::Space, Modifiers::SHIFT),
            Some(GridKey::Set(Mods {
                ctrl: false,
                alt: true
            }))
        );
        assert_eq!(
            GridKey::of(Key::Space, Modifiers::SHIFT | Modifiers::CTRL),
            None
        );
        assert_eq!(
            GridKey::of(Key::Space, Modifiers::SHIFT | Modifiers::ALT),
            None
        );
        assert_eq!(GridKey::of(Key::ArrowUp, Modifiers::CTRL), None);
        assert_eq!(
            GridKey::of(Key::ArrowLeft, Modifiers::NONE),
            Some(GridKey::Left)
        );
        assert_eq!(GridKey::of(Key::PageUp, Modifiers::NONE), None);
    }
}
