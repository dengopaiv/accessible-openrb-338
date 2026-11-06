//! What a screen reader's requests and the value keys do to the panel (task-187, A11Y-3,
//! docs/accessibility.md 3.3): the sliders (knobs and faders), the spin buttons (a display with
//! its up and down arrows) and the pattern selector of each row (its 8 pattern and 4 bank
//! buttons).
//!
//! Every operation goes through the `Panel` method the mouse uses (`set_knob`, `set_fader`,
//! `arrow`, `press`), so the events the engine hears, a song-mode recording included, are the
//! mouse's.

use eframe::egui::{self, accesskit};

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
pub fn request(panel: &mut Panel, t: &Target, req: Request, now: f64) {
    match (t, req) {
        (Target::Slider { id, fader }, Request::SetValue(v)) => {
            if v.is_finite() {
                set_slider(panel, id, *fader, v.round().clamp(0.0, 127.0) as i32);
            }
        }
        (_, Request::Increment) => step(panel, t, Step::Up, now),
        (_, Request::Decrement) => step(panel, t, Step::Down, now),
        _ => {}
    }
}

/// Does what a key asks of a value control.
pub fn step(panel: &mut Panel, t: &Target, s: Step, now: f64) {
    match t {
        Target::Slider { id, fader } => {
            let Some(v) = panel.value(id) else {
                return;
            };
            let v = i32::from(v);
            let new = match s {
                Step::Up => v + 1,
                Step::Down => v - 1,
                Step::PageUp => v + 8,
                Step::PageDown => v - 8,
                Step::Home => 0,
                Step::End => 127,
            };
            set_slider(panel, id, *fader, new.clamp(0, 127));
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
        step(&mut p, &t, Step::PageUp, 0.0);
        assert_eq!((p.rows[1].pattern, p.rows[1].bank), (8, 1));
        step(&mut p, &t, Step::Down, 0.0);
        assert_eq!((p.rows[1].pattern, p.rows[1].bank), (7, 0));
        step(&mut p, &t, Step::End, 0.0);
        step(&mut p, &t, Step::Up, 0.0);
        step(&mut p, &t, Step::Home, 0.0);
        step(&mut p, &t, Step::Down, 0.0);
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
        step(&mut p, &t, Step::Home, 0.0);
        assert_eq!(p.value("tb303_1.cutoff"), Some(0));
        step(&mut p, &t, Step::Down, 0.0);
        step(&mut p, &t, Step::PageUp, 0.0);
        assert_eq!(p.value("tb303_1.cutoff"), Some(8));
        step(&mut p, &t, Step::End, 0.0);
        step(&mut p, &t, Step::Up, 0.0);
        assert_eq!(p.value("tb303_1.cutoff"), Some(127));
        let f = Target::Slider {
            id: "pcf.freq".into(),
            fader: true,
        };
        request(&mut p, &f, Request::SetValue(126.6), 0.0);
        assert_eq!(p.value("pcf.freq"), Some(127));
        request(&mut p, &f, Request::Decrement, 0.0);
        assert_eq!(p.value("pcf.freq"), Some(126));
        request(&mut p, &f, Request::SetValue(f64::NAN), 0.0);
        assert_eq!(p.value("pcf.freq"), Some(126));
    }
}
