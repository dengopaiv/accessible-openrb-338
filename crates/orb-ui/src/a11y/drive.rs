//! What a screen reader's requests and the value keys do to the panel (task-187, A11Y-3,
//! docs/accessibility.md 3.3): the sliders (knobs and faders), the spin buttons (a display with
//! its up and down arrows) and the pattern selector of each row (its 8 pattern and 4 bank
//! buttons).
//!
//! Every operation goes through the `Panel` method the mouse uses (`set_knob`, `set_fader`,
//! `arrow`, `press`), so the events the engine hears, a song-mode recording included, are the
//! mouse's.

use eframe::egui::{self, accesskit};

use super::names::{self, Speech};
use crate::behavior;
use crate::layout::Ctl;
use crate::model::Panel;

/// A change of the focused value control by a key (screen reader mode).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    Up,
    Down,
    PageUp,
    PageDown,
    Home,
    End,
}

impl Step {
    /// The step of a key: Up, Down, Page Up, Page Down, Home and End.
    pub fn of_key(key: egui::Key) -> Option<Self> {
        Some(match key {
            egui::Key::ArrowUp => Self::Up,
            egui::Key::ArrowDown => Self::Down,
            egui::Key::PageUp => Self::PageUp,
            egui::Key::PageDown => Self::PageDown,
            egui::Key::Home => Self::Home,
            egui::Key::End => Self::End,
            _ => return None,
        })
    }
}

/// A value control a screen reader drives.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Target {
    /// A knob (`fader` false) or a fader, by its layout id.
    Slider { id: String, fader: bool },
    /// A spin button, by its display's layout id.
    Spin(String),
    /// The pattern selector of row 0..=3.
    Pattern(usize),
}

/// A screen reader's request to a value control.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Request {
    Increment,
    Decrement,
    SetValue(f64),
}

/// The displays that are spin buttons: each has the arrows `<id>_up` and `<id>_down` (the 303's
/// edit step display has none and stays a label).
pub const SPIN_DISPLAYS: [&str; 10] = [
    "transport.tempo",
    "transport.bar",
    "transport.loop_start",
    "transport.loop_length",
    "pattern1.steps",
    "pattern2.steps",
    "pattern3.steps",
    "pattern4.steps",
    "delay.steps",
    "pcf.pattern",
];

/// The size of a row's pattern selector (panel pixels): the area of its 8 pattern buttons and
/// 4 bank buttons, from the top left of the button `pattern1` (a unit test checks it against
/// `layout.tsv`).
pub const SELECTOR_SIZE: (f32, f32) = (62.0, 54.0);

/// Whether a display is a spin button.
pub fn is_spin(display_id: &str) -> bool {
    SPIN_DISPLAYS.contains(&display_id)
}

/// The value control of a layout part, if it is one (the pattern selector has no part of its
/// own: see [`selector_id`]).
pub fn target(c: &Ctl) -> Option<Target> {
    match c.kind.as_str() {
        "knob" | "fader" => Some(Target::Slider {
            id: c.id.clone(),
            fader: c.kind == "fader",
        }),
        "display" if is_spin(&c.id) => Some(Target::Spin(c.id.clone())),
        _ => None,
    }
}

/// Whether a part is joined into a spin button or a pattern selector: it keeps its mouse
/// behavior but takes no focus and has no node of its own (the arrows of the spin displays and
/// the pattern and bank buttons).
pub fn joined(c: &Ctl) -> bool {
    if c.kind == "grid" {
        return true;
    }
    matches!(c.kind.as_str(), "repeat" | "wind")
        && ["_up", "_down"]
            .iter()
            .any(|s| c.id.strip_suffix(s).is_some_and(is_spin))
}

/// The egui id of row `r`'s pattern selector node.
pub fn selector_id(r: usize) -> egui::Id {
    egui::Id::unique(("pattern selector", r))
}

/// The row (0..=3) of a pattern section's layout section (`pattern1` -> 0).
pub fn row_of(section: &str) -> Option<usize> {
    let r: usize = section.strip_prefix("pattern")?.parse().ok()?;
    (1..=4).contains(&r).then(|| r - 1)
}

/// The screen reader's requests to the node `id` in this frame, in their order.
pub fn requests(ui: &egui::Ui, id: egui::Id) -> Vec<Request> {
    let target = id.accesskit_id();
    ui.input(|i| {
        i.events
            .iter()
            .filter_map(|e| match e {
                egui::Event::AccessKitActionRequest(r)
                    if r.target_node == target && r.target_tree == accesskit::TreeId::ROOT =>
                {
                    match (r.action, &r.data) {
                        (accesskit::Action::Increment, _) => Some(Request::Increment),
                        (accesskit::Action::Decrement, _) => Some(Request::Decrement),
                        (
                            accesskit::Action::SetValue,
                            Some(accesskit::ActionData::NumericValue(v)),
                        ) => Some(Request::SetValue(*v)),
                        (accesskit::Action::SetValue, Some(accesskit::ActionData::Value(s))) => {
                            s.trim().parse().ok().map(Request::SetValue)
                        }
                        _ => None,
                    }
                }
                _ => None,
            })
            .collect()
    })
}

/// Does what a request asks of a value control. A spin button and a pattern selector take
/// Increment and Decrement only.
pub fn request(panel: &mut Panel, t: &Target, req: Request, speech: Speech, now: f64) {
    match (t, req) {
        (Target::Slider { id, fader }, Request::SetValue(v)) => {
            if v.is_finite() {
                set_slider(panel, id, *fader, v.round().clamp(0.0, 127.0) as i32);
            }
        }
        (_, Request::Increment) => step(panel, t, Step::Up, speech, now),
        (_, Request::Decrement) => step(panel, t, Step::Down, speech, now),
        _ => {}
    }
}

/// The first position from `from` in the direction `dir` (+1 or -1) whose spoken text differs
/// from the text at `v` (TASK-213, the user, 2026-11-07), or None when no position further on
/// reads differently. Only `Speech::Units` skips positions: with the raw number spoken every
/// position reads differently, and `Plain` is not screen reader mode.
fn next_different(
    panel: &Panel,
    id: &str,
    (v, from, dir): (i32, i32, i32),
    speech: Speech,
) -> Option<i32> {
    if speech != Speech::Units {
        return (0..=127).contains(&from).then_some(from);
    }
    let text = |p: i32| names::control_value_text(panel, id, p as u8, speech);
    let here = text(v);
    let mut p = from;
    while (0..=127).contains(&p) {
        if text(p) != here {
            return Some(p);
        }
        p += dir;
    }
    None
}

/// Does what a key asks of a value control. In screen reader mode without the raw numbers
/// (`Speech::Units`), Up and Down on a knob or fader go to the next position that reads
/// differently, and Page Up and Page Down to the first such position at least 8 away; with
/// none further on nothing moves (a move with the text unchanged would be silent); Home, End, the
/// raw numbers and the other controls are as they were.
pub fn step(panel: &mut Panel, t: &Target, s: Step, speech: Speech, now: f64) {
    match t {
        Target::Slider { id, fader } => {
            let Some(v) = panel.value(id) else {
                return;
            };
            let v = i32::from(v);
            let new = match s {
                Step::Up => next_different(panel, id, (v, v + 1, 1), speech),
                Step::Down => next_different(panel, id, (v, v - 1, -1), speech),
                Step::PageUp => next_different(panel, id, (v, (v + 8).min(127), 1), speech),
                Step::PageDown => next_different(panel, id, (v, (v - 8).max(0), -1), speech),
                Step::Home => Some(0),
                Step::End => Some(127),
            };
            if let Some(new) = new {
                set_slider(panel, id, *fader, new);
            }
        }
        // TASK-205 (the user, 2026-11-06): Page Up / Page Down are 10 clicks, Home / End click
        // to the end of a short range only (tempo and the transport positions: nothing)
        Target::Spin(display) => {
            let up = format!("{display}_up");
            let down = format!("{display}_down");
            match s {
                Step::Up => panel.arrow(&up),
                Step::Down => panel.arrow(&down),
                Step::PageUp | Step::PageDown => {
                    let id = if s == Step::PageUp { &up } else { &down };
                    for _ in 0..10 {
                        panel.arrow(id);
                    }
                }
                Step::Home | Step::End => {
                    let Some((lo, hi)) = short_range(display) else {
                        return;
                    };
                    let (target, id) = if s == Step::Home {
                        (lo, &down)
                    } else {
                        (hi, &up)
                    };
                    // one click per value, as many as the range has at most
                    for _ in 0..=hi - lo {
                        if panel.display(display).is_none_or(|v| v == target) {
                            break;
                        }
                        panel.arrow(id);
                    }
                }
            }
        }
        Target::Pattern(r) => {
            let p = i32::from(panel.rows[*r].pattern);
            let new = match s {
                Step::Up => p + 1,
                Step::Down => p - 1,
                Step::PageUp => p + 8,
                Step::PageDown => p - 8,
                Step::Home => 0,
                Step::End => 31,
            };
            select_pattern(panel, *r, new.clamp(0, 31) as u8, now);
        }
    }
}

/// The range of a spin button whose Home and End go to its ends (TASK-205): each row's pattern
/// length 1..=16, delay steps 1..=32 and PCF pattern 0..=54 (`Panel::arrow`'s clamps). None for
/// tempo, bar, loop start and loop length.
pub fn short_range(display: &str) -> Option<(i32, i32)> {
    match display {
        "delay.steps" => Some((1, 32)),
        "pcf.pattern" => Some((0, 54)),
        d if d.starts_with("pattern") && d.ends_with(".steps") => Some((1, 16)),
        _ => None,
    }
}

/// Sets a knob or fader to the value `v` (0..=127) as the mouse would store it:
/// `set_knob(id, knob_from_engine(v))`, `set_fader(id, fader_from_engine(v))`. Nothing when the
/// control already sends `v` (its position stays where the mouse left it).
pub fn set_slider(panel: &mut Panel, id: &str, fader: bool, v: i32) {
    if panel.value(id).map(i32::from) == Some(v) {
        return;
    }
    if fader {
        panel.set_fader(id, behavior::fader_from_engine(v));
    } else {
        panel.set_knob(id, behavior::knob_from_engine(v));
    }
}

/// Selects pattern `p` (0..=31) in row `r` with the clicks a mouse makes: the bank button when
/// the shown bank is another, then the pattern button. Nothing when `p` is already selected.
pub fn select_pattern(panel: &mut Panel, r: usize, p: u8, now: f64) {
    let row = &panel.rows[r];
    if row.pattern == p {
        return;
    }
    let n = r + 1;
    if row.bank != p / 8 {
        let bank = char::from(b'A' + p / 8);
        panel.press(&format!("pattern{n}.bank{bank}"), now);
    }
    panel.press(&format!("pattern{n}.pattern{}", p % 8 + 1), now);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout;
    use crate::model::Event;

    #[test]
    fn the_spin_displays_are_the_displays_with_arrows() {
        let ctls = layout::controls();
        let has = |id: &str| ctls.iter().any(|c| c.id == id);
        for c in ctls.iter().filter(|c| c.kind == "display") {
            let arrows = has(&format!("{}_up", c.id)) && has(&format!("{}_down", c.id));
            assert_eq!(is_spin(&c.id), arrows, "{}", c.id);
        }
        // every arrow belongs to a spin button; rewind and fast forward stay buttons
        for c in &ctls {
            if matches!(c.kind.as_str(), "repeat" | "wind") {
                let wind = matches!(c.id.as_str(), "transport.rewind" | "transport.ff");
                assert_eq!(joined(c), !wind, "{}", c.id);
            }
        }
        assert_eq!(ctls.iter().filter(|c| joined(c)).count(), 20 + 48);
    }

    #[test]
    fn the_selector_covers_the_rows_buttons() {
        let ctls = layout::controls();
        for r in 0..4 {
            let section = format!("pattern{}", r + 1);
            let grid: Vec<_> = ctls
                .iter()
                .filter(|c| c.section == section && c.kind == "grid")
                .collect();
            assert_eq!(grid.len(), 12);
            let first = grid.iter().find(|c| c.name() == "pattern1").unwrap();
            let x1 = grid.iter().map(|c| c.x + c.w).fold(0.0, f32::max);
            let y1 = grid.iter().map(|c| c.y + c.h).fold(0.0, f32::max);
            assert!(grid.iter().all(|c| c.x >= first.x && c.y >= first.y));
            assert_eq!((x1 - first.x, y1 - first.y), SELECTOR_SIZE, "{section}");
            assert_eq!(row_of(&section), Some(r));
        }
    }

    #[test]
    fn the_short_ranges_are_the_arrows_clamps() {
        for display in SPIN_DISPLAYS {
            let Some((lo, hi)) = short_range(display) else {
                assert!(display.starts_with("transport."), "{display}");
                continue;
            };
            let mut p = Panel::new();
            for _ in 0..100 {
                p.arrow(&format!("{display}_up"));
            }
            assert_eq!(p.display(display), Some(hi), "{display}");
            for _ in 0..100 {
                p.arrow(&format!("{display}_down"));
            }
            assert_eq!(p.display(display), Some(lo), "{display}");
        }
    }

    #[test]
    fn the_selector_presses_bank_and_pattern() {
        let mut p = Panel::new();
        p.events.clear();
        let t = Target::Pattern(1);
        step(&mut p, &t, Step::PageUp, Speech::Units, 0.0);
        assert_eq!((p.rows[1].pattern, p.rows[1].bank), (8, 1));
        step(&mut p, &t, Step::Down, Speech::Units, 0.0);
        assert_eq!((p.rows[1].pattern, p.rows[1].bank), (7, 0));
        step(&mut p, &t, Step::End, Speech::Units, 0.0);
        step(&mut p, &t, Step::Up, Speech::Units, 0.0);
        step(&mut p, &t, Step::Home, Speech::Units, 0.0);
        step(&mut p, &t, Step::Down, Speech::Units, 0.0);
        let pats: Vec<u8> = p
            .events
            .iter()
            .map(|e| match e {
                Event::Pattern { row: 1, pattern } => *pattern,
                e => panic!("{e:?}"),
            })
            .collect();
        // the ends clamp: Up at D8 and Down at A1 do nothing
        assert_eq!(pats, [8, 7, 31, 0]);
    }

    #[test]
    fn slider_steps_clamp() {
        let mut p = Panel::new();
        let t = Target::Slider {
            id: "tb303_1.cutoff".into(),
            fader: false,
        };
        p.events.clear();
        step(&mut p, &t, Step::Home, Speech::UnitsAndRaw, 0.0);
        assert_eq!(p.value("tb303_1.cutoff"), Some(0));
        step(&mut p, &t, Step::Down, Speech::UnitsAndRaw, 0.0);
        step(&mut p, &t, Step::PageUp, Speech::UnitsAndRaw, 0.0);
        assert_eq!(p.value("tb303_1.cutoff"), Some(8));
        step(&mut p, &t, Step::End, Speech::UnitsAndRaw, 0.0);
        step(&mut p, &t, Step::Up, Speech::UnitsAndRaw, 0.0);
        assert_eq!(p.value("tb303_1.cutoff"), Some(127));
        let f = Target::Slider {
            id: "pcf.freq".into(),
            fader: true,
        };
        request(
            &mut p,
            &f,
            Request::SetValue(126.6),
            Speech::UnitsAndRaw,
            0.0,
        );
        assert_eq!(p.value("pcf.freq"), Some(127));
        request(&mut p, &f, Request::Decrement, Speech::UnitsAndRaw, 0.0);
        assert_eq!(p.value("pcf.freq"), Some(126));
        request(
            &mut p,
            &f,
            Request::SetValue(f64::NAN),
            Speech::UnitsAndRaw,
            0.0,
        );
        assert_eq!(p.value("pcf.freq"), Some(126));
    }

    fn knob(id: &str) -> Target {
        Target::Slider {
            id: id.into(),
            fader: false,
        }
    }

    fn text(p: &Panel, id: &str) -> String {
        names::control_value_text(p, id, p.value(id).unwrap(), Speech::Units)
    }

    /// TASK-213: every Up press changes the spoken text; the knob is crossed in as many presses
    /// as it has distinct texts, and Down goes back one text per press.
    #[test]
    fn up_and_down_move_to_the_next_different_text() {
        for id in ["tr808.bd_tone", "tr808.bd_decay", "pcf.q", "tb303_1.cutoff"] {
            let t = knob(id);
            let mut p = Panel::new();
            step(&mut p, &t, Step::Home, Speech::Units, 0.0);
            let mut texts = vec![text(&p, id)];
            loop {
                step(&mut p, &t, Step::Up, Speech::Units, 0.0);
                let now = text(&p, id);
                if &now == texts.last().unwrap() {
                    break;
                }
                texts.push(now);
            }
            // the distinct texts of 0..=127, in order
            let mut want: Vec<String> = Vec::new();
            for v in 0..=127u8 {
                let s = names::control_value_text(&p, id, v, Speech::Units);
                if want.last() != Some(&s) {
                    want.push(s);
                }
            }
            assert_eq!(texts, want, "{id}");
            // Down walks the texts back, one text per press
            for want in texts.iter().rev().skip(1) {
                step(&mut p, &t, Step::Down, Speech::Units, 0.0);
                assert_eq!(&text(&p, id), want, "{id}");
            }
        }
        let mut p = Panel::new();
        let t = knob("tr808.bd_tone");
        step(&mut p, &t, Step::Home, Speech::Units, 0.0);
        step(&mut p, &t, Step::Up, Speech::Units, 0.0);
        assert!(p.value("tr808.bd_tone").unwrap() > 1);
        let t = knob("tb303_1.cutoff");
        step(&mut p, &t, Step::Home, Speech::Units, 0.0);
        step(&mut p, &t, Step::Up, Speech::Units, 0.0);
        assert_eq!(p.value("tb303_1.cutoff"), Some(1));
    }

    #[test]
    fn page_keys_land_on_a_different_text_and_the_ends_hold() {
        let id = "tr808.bd_tone";
        let t = knob(id);
        let mut p = Panel::new();
        step(&mut p, &t, Step::Home, Speech::Units, 0.0);
        let before = text(&p, id);
        step(&mut p, &t, Step::PageUp, Speech::Units, 0.0);
        let v = p.value(id).unwrap();
        assert!(v >= 8);
        assert_ne!(text(&p, id), before);
        step(&mut p, &t, Step::PageDown, Speech::Units, 0.0);
        assert!(v - p.value(id).unwrap() >= 8 || p.value(id) == Some(0));
        assert_ne!(
            text(&p, id),
            names::control_value_text(&p, id, v, Speech::Units)
        );
        // inside the last / first text group no key moves (the text would stay)
        step(&mut p, &t, Step::End, Speech::Units, 0.0);
        let last = text(&p, id);
        for k in [Step::Up, Step::PageUp] {
            step(&mut p, &t, k, Speech::Units, 0.0);
            assert_eq!(p.value(id), Some(127));
        }
        request(&mut p, &t, Request::SetValue(121.0), Speech::Units, 0.0);
        assert_eq!(text(&p, id), last);
        step(&mut p, &t, Step::PageUp, Speech::Units, 0.0);
        step(&mut p, &t, Step::Up, Speech::Units, 0.0);
        assert_eq!(p.value(id), Some(121));
        step(&mut p, &t, Step::Home, Speech::Units, 0.0);
        let first = text(&p, id);
        request(&mut p, &t, Request::SetValue(1.0), Speech::Units, 0.0);
        assert_eq!(text(&p, id), first);
        step(&mut p, &t, Step::PageDown, Speech::Units, 0.0);
        step(&mut p, &t, Step::Down, Speech::Units, 0.0);
        assert_eq!(p.value(id), Some(1));
        step(&mut p, &t, Step::Home, Speech::Units, 0.0);
        step(&mut p, &t, Step::Down, Speech::Units, 0.0);
        assert_eq!(p.value(id), Some(0));
        request(&mut p, &t, Request::SetValue(5.0), Speech::Units, 0.0);
        assert_eq!(p.value(id), Some(5));
    }

    /// Screen reader mode off (Plain): one position, 8 for Page, clamped, as upstream.
    #[test]
    fn the_mode_off_moves_by_position() {
        let id = "tr808.bd_tone";
        let t = knob(id);
        let mut p = Panel::new();
        let s = Speech::Plain;
        step(&mut p, &t, Step::Home, s, 0.0);
        step(&mut p, &t, Step::Down, s, 0.0);
        step(&mut p, &t, Step::PageDown, s, 0.0);
        assert_eq!(p.value(id), Some(0));
        step(&mut p, &t, Step::Up, s, 0.0);
        request(&mut p, &t, Request::Increment, s, 0.0);
        assert_eq!(p.value(id), Some(2));
        request(&mut p, &t, Request::Decrement, s, 0.0);
        assert_eq!(p.value(id), Some(1));
        step(&mut p, &t, Step::PageUp, s, 0.0);
        assert_eq!(p.value(id), Some(9));
        step(&mut p, &t, Step::End, s, 0.0);
        step(&mut p, &t, Step::PageUp, s, 0.0);
        step(&mut p, &t, Step::Up, s, 0.0);
        assert_eq!(p.value(id), Some(127));
        step(&mut p, &t, Step::PageDown, s, 0.0);
        assert_eq!(p.value(id), Some(119));
    }

    /// Speak Raw Values: one position per press, Increment and Decrement too.
    #[test]
    fn raw_values_move_one_position() {
        let id = "tr808.bd_tone";
        let t = knob(id);
        let mut p = Panel::new();
        let s = Speech::UnitsAndRaw;
        step(&mut p, &t, Step::Home, s, 0.0);
        step(&mut p, &t, Step::Up, s, 0.0);
        assert_eq!(p.value(id), Some(1));
        request(&mut p, &t, Request::Increment, s, 0.0);
        assert_eq!(p.value(id), Some(2));
        request(&mut p, &t, Request::Decrement, s, 0.0);
        step(&mut p, &t, Step::Down, s, 0.0);
        assert_eq!(p.value(id), Some(0));
        step(&mut p, &t, Step::PageUp, s, 0.0);
        assert_eq!(p.value(id), Some(8));
        // with the units only, Increment skips like the key
        step(&mut p, &t, Step::Home, Speech::Units, 0.0);
        request(&mut p, &t, Request::Increment, Speech::Units, 0.0);
        assert!(p.value(id).unwrap() > 1);
    }
}
