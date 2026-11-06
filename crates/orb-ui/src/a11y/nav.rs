//! The sections of the panel and the focus order of screen reader mode (task-188, A11Y-4,
//! docs/accessibility.md 3.2, docs/ui_spec.md 20.2).
//!
//! The panel is nine sections, in this order: Transport, the two Bass Lines, the TR-808, the
//! TR-909 (each machine with its pattern row and its mixer channel, decision A11Y-D3), Master
//! and compressor, PCF, Delay and Distortion. In screen reader mode each section is a Group node
//! with its name, F6 and Shift+F6 go to the first control of the next or previous section, and
//! Tab and Shift+Tab go through the controls of the whole panel in this order, wrapping at its
//! ends ([`order`]).

use eframe::egui::{self, accesskit};

use super::{drive, grid, hit_id, names};
use crate::layout::Ctl;

/// The spoken sections, in the navigation order (`names::section_name` gives each part's).
pub const SECTIONS: [&str; 9] = [
    "Transport",
    "Bass Line 1",
    "Bass Line 2",
    "TR-808",
    "TR-909",
    "Master and compressor",
    "PCF",
    "Delay",
    "Distortion",
];

/// The transport's controls in the order of docs/accessibility.md 3.2: the buttons a user
/// wants first, then the displays and the knob, the wind buttons last.
const TRANSPORT: [&str; 12] = [
    "transport.play",
    "transport.stop",
    "transport.record",
    "transport.loop",
    "transport.mode",
    "transport.tempo",
    "transport.shuffle",
    "transport.bar",
    "transport.loop_start",
    "transport.loop_length",
    "transport.rewind",
    "transport.ff",
];

/// A place the focus stops at: a control's node and its section.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Stop {
    /// The index of the section in [`SECTIONS`].
    pub section: usize,
    /// The egui id of the control's node (its hit area, a spin button's display or a pattern
    /// selector).
    pub id: egui::Id,
}

/// The index in [`SECTIONS`] of a layout section (`mixer2` -> 2, Bass Line 2).
pub fn section_index(layout_section: &str) -> Option<usize> {
    let name = names::section_name(layout_section)?;
    SECTIONS.iter().position(|s| *s == name)
}

/// The row (0..=3: the two 303s, the 808, the 909) of a section, for the machines' sections.
pub fn row_of_section(section: usize) -> Option<usize> {
    (1..=4).contains(&section).then(|| section - 1)
}

/// Whether a part takes the focus (`App::control`): the knobs, faders, spin buttons, buttons,
/// toggles and switches and the 303s' step editors (task-190), but not the parts joined into a
/// spin button or a pattern selector, the LEDs, meters, labels and click areas.
pub fn focusable(c: &Ctl) -> bool {
    match c.kind.as_str() {
        "display" => drive::is_spin(&c.id) || super::step303::editor_of_layout(&c.id).is_some(),
        kind => names::is_interactive(kind) && !drive::joined(c),
    }
}

/// The focus order of screen reader mode: the sections in order; in a section the panel's
/// order (`layout.tsv`: a machine's pattern row, the machine, its mixer channel), with a
/// machine's pattern selector first and the transport in the order of [`TRANSPORT`].
pub fn order(ctls: &[Ctl]) -> Vec<Stop> {
    let mut keyed = Vec::new();
    for (i, c) in ctls.iter().enumerate() {
        let Some(section) = section_index(&c.section) else {
            continue;
        };
        if c.kind == "grid"
            && c.name() == "pattern1"
            && let Some(row) = drive::row_of(&c.section)
        {
            keyed.push(((section, 0, 0), drive::selector_id(row)));
        }
        // a drum machine's grid (task-191) at the place of its step buttons
        if c.kind == "steps" && c.name() == "step1" {
            keyed.push(((section, 1, i), grid::grid_id(c.section == "tr808")));
        }
        if focusable(c) {
            let rank = if section == 0 {
                TRANSPORT
                    .iter()
                    .position(|t| *t == c.id)
                    .unwrap_or(TRANSPORT.len())
                    + 1
            } else {
                1
            };
            keyed.push(((section, rank, i), hit_id(c)));
        }
    }
    keyed.sort_by_key(|(k, _)| *k);
    keyed
        .into_iter()
        .map(|((section, _, _), id)| Stop { section, id })
        .collect()
}

/// The section of the stop `id`; None for a node that is not the panel's.
pub fn section_of(order: &[Stop], id: egui::Id) -> Option<usize> {
    order.iter().find(|s| s.id == id).map(|s| s.section)
}

/// The control after (`back` false) or before the focused one, wrapping at the ends of the
/// panel; with no control of the panel focused, the first (the last for `back`).
pub fn next_control(order: &[Stop], focused: Option<egui::Id>, back: bool) -> Option<egui::Id> {
    let n = order.len();
    if n == 0 {
        return None;
    }
    let at = focused.and_then(|f| order.iter().position(|s| s.id == f));
    let i = match (at, back) {
        (None, false) => 0,
        (None, true) => n - 1,
        (Some(i), false) => (i + 1) % n,
        (Some(i), true) => (i + n - 1) % n,
    };
    Some(order[i].id)
}

/// The first control of the section after (`back` false) or before the focused control's,
/// wrapping; with no control of the panel focused, of the first (the last for `back`) section.
pub fn next_section(order: &[Stop], focused: Option<egui::Id>, back: bool) -> Option<egui::Id> {
    let n = SECTIONS.len();
    let at = focused.and_then(|f| section_of(order, f));
    let s = match (at, back) {
        (None, false) => 0,
        (None, true) => n - 1,
        (Some(s), false) => (s + 1) % n,
        (Some(s), true) => (s + n - 1) % n,
    };
    order.iter().find(|x| x.section == s).map(|x| x.id)
}

/// The child `Ui` of a section: its node is a Group named as the section, the parent of the
/// section's controls in the accessibility tree, so a screen reader says the section's name
/// when the focus enters it.
pub fn group(section: usize, max_rect: egui::Rect) -> egui::UiBuilder {
    egui::UiBuilder::new()
        .id_salt(("orb_section", section))
        .max_rect(max_rect)
        .accessibility_role(accesskit::Role::Group)
        .accessibility_label(SECTIONS[section])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout;

    #[test]
    fn every_focusable_part_is_in_the_order_once() {
        let ctls = layout::controls();
        let order = order(&ctls);
        let parts = ctls.iter().filter(|c| focusable(c)).count();
        assert_eq!(
            order.len(),
            parts + 4 + 2,
            "the parts, the 4 pattern selectors and the 2 drum grids"
        );
        // each grid in its machine's section
        for (is808, s) in [(true, 3), (false, 4)] {
            assert_eq!(section_of(&order, grid::grid_id(is808)), Some(s));
        }
        for (i, a) in order.iter().enumerate() {
            assert!(order[i + 1..].iter().all(|b| b.id != a.id));
        }
        // the sections in order, each present
        let sections: Vec<usize> = order.iter().map(|s| s.section).collect();
        assert!(sections.windows(2).all(|w| w[0] <= w[1]));
        assert_eq!(sections.first(), Some(&0));
        assert_eq!(sections.last(), Some(&8));
        for (s, name) in SECTIONS.iter().enumerate() {
            assert!(sections.contains(&s), "{name}");
        }
    }

    #[test]
    fn the_first_controls_of_the_sections() {
        let ctls = layout::controls();
        let order = order(&ctls);
        let first = |s: usize| order.iter().find(|x| x.section == s).unwrap().id;
        let id = |kind: &str, lid: &str| super::super::hit_id_of(kind, lid);
        assert_eq!(first(0), id("trigger", "transport.play"));
        for r in 0..4 {
            assert_eq!(first(r + 1), drive::selector_id(r));
        }
        assert_eq!(first(5), id("fader", "master.fader"));
        assert_eq!(first(6), id("toggle", "pcf.on"));
        assert_eq!(first(7), id("toggle", "delay.on"));
        assert_eq!(first(8), id("toggle", "dist.on"));
    }

    #[test]
    fn moves_wrap_at_the_ends() {
        let order = order(&layout::controls());
        let (a, z) = (order[0].id, order[order.len() - 1].id);
        assert_eq!(next_control(&order, None, false), Some(a));
        assert_eq!(next_control(&order, None, true), Some(z));
        assert_eq!(next_control(&order, Some(z), false), Some(a));
        assert_eq!(next_control(&order, Some(a), true), Some(z));
        assert_eq!(next_section(&order, Some(z), false), Some(a));
        let last = order.iter().find(|s| s.section == 8).unwrap().id;
        assert_eq!(next_section(&order, Some(a), true), Some(last));
        assert_eq!(next_section(&order, None, true), Some(last));
        assert_eq!(row_of_section(0), None);
        assert_eq!(row_of_section(3), Some(2));
        assert_eq!(row_of_section(5), None);
    }
}
