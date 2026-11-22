//! The interaction math of the original's widgets (docs/ui_spec.md section 2), as pure
//! functions so that it is unit-tested without a window. The geometry (a knob's drag span,
//! a fader's track and handle, the drum machines' step cells and instrument hit areas) is the
//! current skin's, in its units (`skin::Ctl`): the original's numbers are the skins' data
//! (docs/ui_spec.md section 22).

/// Highest knob position (the knob keeps p in 0..=1000).
pub const KNOB_MAX: i32 = 1000;
/// Highest fader position (0..=999).
pub const FADER_MAX: i32 = 999;

/// A fader's geometry in skin units: the track's height and the handle's (the original's
/// mixer faders: 54 and 11).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Fader {
    pub track: i32,
    pub handle: i32,
}

impl Fader {
    /// The handle's travel: the track less the handle (43 for the original's 54 and 11).
    pub fn travel(&self) -> i32 {
        (self.track - self.handle).max(1)
    }

    /// The scale `k = 10000 / travel`, integer (`CBitmapFaderView`: 232 for a travel of 43).
    pub fn k(&self) -> i32 {
        10000 / self.travel()
    }
}

/// The knob position after a drag: grabbed at `p0` with the pointer at `y0`, now at `y`
/// (skin units, `span` the skin's drag for the full range). Up increases; the span is 5 times
/// larger with Shift held at mouse down.
pub fn knob_drag(p0: i32, y0: f32, y: f32, span: f32, fine: bool) -> i32 {
    let span = if fine { span * 5.0 } else { span };
    let d = (KNOB_MAX as f32 * (y0 - y) / span) as i32;
    (p0 + d).clamp(0, KNOB_MAX)
}

/// The value 0..=127 sent for the position (`sub_467F1A`).
pub fn knob_send(p: i32) -> i32 {
    (127 * p + 500) / 1000
}

/// The position for a value from the engine (`sub_4A637D`).
pub fn knob_from_engine(v: i32) -> i32 {
    (1000 * v + 63) / 127
}

/// The pointer angle of a position in degrees: 0 = up, clockwise positive, -135 to +135.
pub fn knob_angle(p: i32) -> f32 {
    -135.0 + 270.0 * p as f32 / KNOB_MAX as f32
}

/// The value 0..=127 sent for a fader position (`sub_469614`).
pub fn fader_send(p: i32) -> i32 {
    (127 * p + 500) / 1000
}

/// The handle's top edge, from the track's top (skin units).
pub fn fader_handle_top(p: i32, f: &Fader) -> f32 {
    (1.0 - p as f32 / FADER_MAX as f32) * f.travel() as f32
}

/// Whether a point `y` (from the track's top) is on the handle at position `p`.
pub fn fader_on_handle(p: i32, y: f32, f: &Fader) -> bool {
    let top = fader_handle_top(p, f);
    y >= top && y < top + f.handle as f32
}

/// Mouse down on the track outside the handle: the handle's center jumps to the pointer
/// (`p = (bottom - y - handle/2 + 2) * k / 10`, integer, clamped).
pub fn fader_jump(y: f32, f: &Fader) -> i32 {
    let y = y.floor() as i32;
    ((f.track - y - f.handle / 2 + 2) * f.k() / 10).clamp(0, FADER_MAX)
}

/// The fader position after a drag of the handle (up increases, no fine mode).
pub fn fader_drag(p0: i32, y0: f32, y: f32, f: &Fader) -> i32 {
    (p0 + (f.k() as f32 * (y0 - y) / 10.0) as i32).clamp(0, FADER_MAX)
}

/// The extra steps of a held repeat button after `t` ms (`sub_4A6C30`): one per 100 ms from
/// 400 ms, and from 2400 ms one more per 20 ms. The mouse down itself gives one step.
pub fn repeat_extra_steps(t_ms: u32) -> u32 {
    let a = if t_ms >= 400 {
        (t_ms - 400).min(2399) / 100
    } else {
        0
    };
    let b = if t_ms >= 2400 { (t_ms - 2400) / 20 } else { 0 };
    a + b
}

/// The pattern grid's button 1..=8 for a point in its 68 x 40 area (`sub_41A5B0`).
pub fn pattern_grid_button(dx: f32, dy: f32) -> u8 {
    let col = ((4.0 * dx / 68.0) as i32).clamp(0, 3);
    let row = ((2.0 * dy / 40.0) as i32).clamp(0, 1);
    (4 * row + col + 1) as u8
}

/// The position (0 or 1) a two-position switch draws for its controller value `v`, which a
/// song may hold out of 0..=1 (RB-AM.rbm's demo song has a 303 waveform of 127, task-156).
///
/// The original takes the value itself as the frame of the switch's two-frame strip, with no
/// clamp (the polls store the byte with `movzx` and `sub_424548` at 0x424548, the draw
/// 0x4243C2 passes it to `CBitmapSequence`'s frame rectangle 0x46DC43, `x = frame * (w + 1)`),
/// so a value above 1 blits from outside the strip. orb draws instead the position the engine
/// plays: the PCF's mode is low-pass (1) only at 1 (`cmp dl, 1` at 0x10016921), and every
/// other switch reads any value but 0 as its second position (the 303 waveform, the 808
/// selectors, the delay's sixteenths; docs/ui_spec.md section 2).
pub fn switch_position(id: &str, v: u8) -> u8 {
    if id == "pcf.mode" {
        u8::from(v == 1)
    } else {
        u8::from(v != 0)
    }
}

/// The modifier keys that matter for the TR-909's step buttons: the original's Ctrl (`ctrl`)
/// and Alt (`alt`).
#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
pub struct Mods {
    pub ctrl: bool,
    pub alt: bool,
}

/// The [`Mods`] of the keys held, as orb reads them for the 909's step buttons and the 909
/// editor (task-281, the user's decision: one rule across the app): the original's Ctrl is the
/// command key (egui's `command`: Cmd on macOS, Ctrl on Windows and Linux) and its Alt is Alt
/// (Option on macOS). macOS's Ctrl and Shift do nothing.
pub fn step_mods(m: egui::Modifiers) -> Mods {
    Mods {
        ctrl: m.command,
        alt: m.alt,
    }
}

/// The new value of a TR-909 step on mouse down, from its value `c` (`CTR909PatternView`):
/// accent: off <-> 2; Ctrl+Alt or the flam button: off <-> 3; Ctrl: off <-> 1; Alt: off <-> 2;
/// otherwise off -> 1 -> 2 -> off. (orb reads the original's Ctrl as Cmd on macOS,
/// [`step_mods`].)
pub fn tr909_step_value(c: u8, accent_row: bool, flam_on: bool, m: Mods) -> u8 {
    let toggle = |on_value: u8| if c == on_value { 0 } else { on_value };
    if accent_row {
        return if c != 0 { 0 } else { 2 };
    }
    if (m.ctrl && m.alt) || flam_on {
        toggle(3)
    } else if m.ctrl {
        toggle(1)
    } else if m.alt {
        toggle(2)
    } else {
        (c + 1) % 3
    }
}

/// A run of equal cells of an instrument hit area: from `x0`, `w` wide, numbered from `first`
/// (skin units; the run ends where the next one starts).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Segment {
    pub x0: f32,
    pub w: f32,
    pub first: i32,
}

/// An instrument hit area of a drum machine (a legend row, the 808's upper labels, the 909's
/// name row): its rectangle, its cells, the instruments' count, and whether a point left of
/// the first cell still counts as the first cell (the 909's rows: everything left of x 26 of
/// the section is AC).
#[derive(Clone, Debug, PartialEq)]
pub struct SelectArea {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub segments: Vec<Segment>,
    pub count: i32,
    pub open_left: bool,
}

/// The instrument under a point (skin units), tested against a drum machine's hit areas in
/// order: the first area whose rows hold `y` (and whose columns hold `x`, unless it is open
/// to the left) answers. The original's numbers (section pixels): the 808's legend row x
/// 10..274, y 71..85, 12 cells of 22, and upper labels x 78..184, y 43..55, the slots 3..7;
/// the 909's legend row y 4..18 and name row y 74..84, AC to x 26, BD..HT 42 apart from 26,
/// RS..RC 21 apart from 236 (docs/ui_spec.md sections 6 and 7).
pub fn select_hit(areas: &[SelectArea], x: f32, y: f32) -> Option<usize> {
    for a in areas {
        if !(a.y..a.y + a.h).contains(&y) {
            continue;
        }
        let inside = (a.x..a.x + a.w).contains(&x);
        if !inside && !(a.open_left && x < a.x) {
            continue;
        }
        let first = a.segments.first()?;
        let i = match a.segments.iter().rev().find(|s| s.x0 <= x) {
            Some(s) => s.first + ((x - s.x0) / s.w).floor() as i32,
            None if a.open_left => first.first,
            None => return None,
        };
        return (0..a.count).contains(&i).then_some(i as usize);
    }
    None
}

/// The step under `x` of a step strip starting at `x0` with cells `cell` wide (skin units):
/// 0..16 (the original's 808: 16 cells of 16 px from x 14 of the section; the 909: of 21 px
/// from x 26).
pub fn strip_step(x0: f32, cell: f32, x: f32) -> Option<usize> {
    let i = ((x - x0) / cell).floor();
    (0.0..16.0).contains(&i).then_some(i as usize)
}

/// The digits of a seven-segment display of `n` digits showing `value`, right-aligned with
/// blank (None) leading zeros; `None` is a disabled display (all blank).
pub fn digit_frames(value: Option<i32>, n: usize) -> Vec<Option<u8>> {
    let mut frames = vec![None; n];
    let Some(v) = value else {
        return frames;
    };
    let mut v = v.clamp(0, 10_i32.pow(n as u32) - 1);
    for i in (0..n).rev() {
        frames[i] = Some((v % 10) as u8);
        v /= 10;
        if v == 0 {
            break;
        }
    }
    frames
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skin::{Skin, svg::modern};

    /// The modern skin's control `id` (of `kind`).
    fn ctl(kind: &str, id: &str) -> crate::skin::Ctl {
        let skin = modern();
        let c = skin
            .layout()
            .controls
            .iter()
            .find(|c| c.id == id && c.kind == kind);
        c.unwrap_or_else(|| panic!("{kind} {id}")).clone()
    }

    fn knob_span(id: &str) -> f32 {
        ctl("knob", id).knob_span()
    }

    /// The original's mixer fader, the modern skin's.
    fn mixer_fader() -> Fader {
        ctl("fader", "mixer1.level").fader()
    }

    #[test]
    fn knob_cutoff_measured_in_the_original() {
        // "CUTOFF dragged to 0, then 31 px up, saved as 64".
        let p = knob_drag(0, 100.0, 69.0, knob_span("tb303_1.cutoff"), false);
        assert_eq!(p, 500);
        assert_eq!(knob_send(p), 64);
        // "RESO with Shift, 31 px up from 0, saved as 13".
        let p = knob_drag(0, 100.0, 69.0, knob_span("tb303_1.reso"), true);
        assert_eq!(p, 100);
        assert_eq!(knob_send(p), 13);
    }

    #[test]
    fn knob_clamps_ignores_nothing_and_has_per_machine_spans() {
        assert_eq!(knob_drag(900, 0.0, -1000.0, 64.0, false), KNOB_MAX);
        assert_eq!(knob_drag(100, 0.0, 1000.0, 64.0, false), 0);
        // Down decreases.
        assert!(knob_drag(500, 0.0, 10.0, 64.0, false) < 500);
        // the original's spans, the modern skin's data
        assert_eq!(knob_span("tb303_2.tune"), 62.0);
        assert_eq!(knob_span("tr808.bd_level"), 36.0);
        assert_eq!(knob_span("tr909.flam"), 41.0);
        assert_eq!(knob_span("mixer3.pan"), 64.0);
        assert_eq!(knob_span("transport.shuffle"), 64.0);
        let skin = modern();
        for c in skin.layout().controls.iter().filter(|c| c.kind == "knob") {
            let want = match c.section.as_str() {
                "tb303_1" | "tb303_2" => 62.0,
                "tr808" => 36.0,
                "tr909" => 41.0,
                _ => 64.0,
            };
            assert_eq!(c.knob_span(), want, "{}", c.id);
        }
    }

    #[test]
    fn knob_value_round_trip() {
        assert_eq!(knob_send(0), 0);
        assert_eq!(knob_send(1000), 127);
        for v in 0..=127 {
            assert_eq!(knob_send(knob_from_engine(v)), v);
        }
        assert_eq!(knob_angle(0), -135.0);
        assert_eq!(knob_angle(500), 0.0);
        assert_eq!(knob_angle(1000), 135.0);
    }

    #[test]
    fn fader_jump_drag_and_handle() {
        let f = &mixer_fader();
        // the original's: a track of 54, a handle of 11, k = 10000 / 43
        assert_eq!((f.track, f.handle, f.travel(), f.k()), (54, 11, 43, 232));
        for c in modern()
            .layout()
            .controls
            .iter()
            .filter(|c| c.kind == "fader")
        {
            assert_eq!(c.fader(), *f, "{}", c.id);
        }
        assert_eq!(fader_handle_top(FADER_MAX, f), 0.0);
        assert_eq!(fader_handle_top(0, f), 43.0);
        assert!(fader_on_handle(0, 45.0, f));
        assert!(!fader_on_handle(0, 30.0, f));
        assert_eq!(fader_jump(0.0, f), FADER_MAX);
        assert_eq!(fader_jump(54.0, f), 0);
        assert_eq!(fader_jump(5.0, f), FADER_MAX);
        // Mid-track.
        assert_eq!(fader_jump(27.0, f), (54 - 27 - 5 + 2) * 232 / 10);
        // Dragging the full travel up from the bottom reaches the top (within 3 steps).
        assert!(fader_drag(0, 50.0, 7.0, f) >= 990);
        assert_eq!(fader_drag(0, 50.0, 60.0, f), 0);
        assert_eq!(fader_send(FADER_MAX), 127);
        // a shorter track (the original's PCF: 45) has a shorter travel and a larger k
        let pcf = Fader {
            track: 45,
            handle: 11,
        };
        assert_eq!((pcf.travel(), pcf.k()), (34, 294));
        assert_eq!(fader_jump(45.0, &pcf), 0);
    }

    #[test]
    fn repeat_timing() {
        assert_eq!(repeat_extra_steps(0), 0);
        assert_eq!(repeat_extra_steps(399), 0);
        assert_eq!(repeat_extra_steps(400), 0);
        assert_eq!(repeat_extra_steps(500), 1);
        assert_eq!(repeat_extra_steps(1400), 10);
        assert_eq!(repeat_extra_steps(2399), 19);
        assert_eq!(repeat_extra_steps(2400), 20);
        // From 2400 ms one more per 20 ms: 22 (capped 100 ms steps at 2200 ms) + 10.
        assert_eq!(repeat_extra_steps(2400 + 200), 22 + 10);
    }

    #[test]
    fn pattern_grid() {
        assert_eq!(pattern_grid_button(1.0, 1.0), 1);
        assert_eq!(pattern_grid_button(67.0, 1.0), 4);
        assert_eq!(pattern_grid_button(1.0, 39.0), 5);
        assert_eq!(pattern_grid_button(67.0, 39.0), 8);
        assert_eq!(pattern_grid_button(-5.0, 90.0), 5);
    }

    #[test]
    fn tr909_step_cycle_and_modifiers() {
        let none = Mods::default();
        assert_eq!(tr909_step_value(0, false, false, none), 1);
        assert_eq!(tr909_step_value(1, false, false, none), 2);
        assert_eq!(tr909_step_value(2, false, false, none), 0);
        let ca = Mods {
            ctrl: true,
            alt: true,
        };
        assert_eq!(tr909_step_value(0, false, false, ca), 3);
        assert_eq!(tr909_step_value(3, false, false, ca), 0);
        let c = Mods {
            ctrl: true,
            alt: false,
        };
        assert_eq!(tr909_step_value(0, false, false, c), 1);
        assert_eq!(tr909_step_value(1, false, false, c), 0);
        let a = Mods {
            ctrl: false,
            alt: true,
        };
        assert_eq!(tr909_step_value(2, false, false, a), 0);
        assert_eq!(tr909_step_value(0, false, false, a), 2);
        assert_eq!(tr909_step_value(0, false, true, none), 3);
        // The accent row is on/off with the stored "on" = 2.
        assert_eq!(tr909_step_value(0, true, false, none), 2);
        assert_eq!(tr909_step_value(1, true, false, none), 0);
    }

    /// task-281: the original's Ctrl is the command key (Cmd on macOS, where egui sets
    /// `command` with `mac_cmd`; Ctrl elsewhere, where it sets it with `ctrl`), its Alt is Alt
    /// (Option); macOS's Ctrl (`ctrl` without `command`) and Shift are no modifier.
    #[test]
    fn the_909_step_modifiers_are_cmd_and_option_on_macos() {
        use egui::Modifiers;
        let mac_cmd = Modifiers {
            mac_cmd: true,
            command: true,
            ..Modifiers::NONE
        };
        let win_ctrl = Modifiers {
            ctrl: true,
            command: true,
            ..Modifiers::NONE
        };
        let low = Mods {
            ctrl: true,
            alt: false,
        };
        let high = Mods {
            ctrl: false,
            alt: true,
        };
        let flam = Mods {
            ctrl: true,
            alt: true,
        };
        assert_eq!(step_mods(mac_cmd), low);
        assert_eq!(step_mods(win_ctrl), low);
        assert_eq!(step_mods(Modifiers::ALT), high);
        assert_eq!(step_mods(mac_cmd | Modifiers::ALT), flam);
        assert_eq!(step_mods(win_ctrl | Modifiers::ALT), flam);
        // macOS's Ctrl, and Shift
        assert_eq!(step_mods(Modifiers::CTRL), Mods::default());
        assert_eq!(step_mods(Modifiers::SHIFT), Mods::default());
    }

    /// A drum machine's section origin and hit areas in the modern skin.
    fn areas(section: &str) -> ((f32, f32), Vec<SelectArea>) {
        let skin = modern();
        let l = skin.layout();
        let o = if section == "tr808" {
            crate::layout::TR808_ORIGIN
        } else {
            crate::layout::TR909_ORIGIN
        };
        let a = l.selects.iter().filter(|s| s.section == section);
        (o, a.map(|s| s.select_area()).collect())
    }

    /// The hit tests in section pixels, through the modern skin's areas.
    fn tr808_hit(x: f32, y: f32) -> Option<usize> {
        let (o, a) = areas("tr808");
        select_hit(&a, o.0 + x, o.1 + y)
    }

    fn tr909_hit(x: f32, y: f32) -> Option<usize> {
        let (o, a) = areas("tr909");
        select_hit(&a, o.0 + x, o.1 + y)
    }

    fn strip(section: &str) -> (f32, f32, f32) {
        let skin = modern();
        let s = skin
            .layout()
            .strips
            .iter()
            .find(|s| s.section == section)
            .unwrap();
        let o = if section == "tr808" {
            crate::layout::TR808_ORIGIN
        } else {
            crate::layout::TR909_ORIGIN
        };
        (o.0, s.x, s.cell())
    }

    fn tr808_step(x: f32) -> Option<usize> {
        let (o, x0, cell) = strip("tr808");
        strip_step(x0, cell, o + x)
    }

    fn tr909_step(x: f32) -> Option<usize> {
        let (o, x0, cell) = strip("tr909");
        strip_step(x0, cell, o + x)
    }

    /// The hit maths before the skin layer (task-266), constants of the original in section
    /// pixels: the skin's data must give the same answers everywhere.
    mod before {
        pub fn tr808_hit(x: f32, y: f32) -> Option<usize> {
            if (71.0..85.0).contains(&y) && (10.0..274.0).contains(&x) {
                return Some(((x - 10.0) / 22.0) as usize);
            }
            if (43.0..55.0).contains(&y) && (78.0..184.0).contains(&x) {
                return Some(((x - 78.0) / 22.0) as usize + 3);
            }
            None
        }

        pub fn tr909_hit(x: f32, y: f32) -> Option<usize> {
            if !((4.0..18.0).contains(&y) || (74.0..84.0).contains(&y)) {
                return None;
            }
            let dx = x as i32;
            let i = if dx < 26 {
                0
            } else if dx < 236 {
                (dx - 26) / 42 + 1
            } else {
                (dx - 236) / 21 + 6
            };
            (0..=11).contains(&i).then_some(i as usize)
        }

        pub fn tr808_step(x: f32) -> Option<usize> {
            let i = ((x - 14.0) / 16.0).floor();
            (0.0..16.0).contains(&i).then_some(i as usize)
        }

        pub fn tr909_step(x: f32) -> Option<usize> {
            let i = ((x - 26.0) / 21.0).floor();
            (0.0..16.0).contains(&i).then_some(i as usize)
        }
    }

    #[test]
    fn the_modern_skin_hits_as_the_constants_did() {
        let (_, a8) = areas("tr808");
        let (_, a9) = areas("tr909");
        let (o8, o9) = (crate::layout::TR808_ORIGIN, crate::layout::TR909_ORIGIN);
        let s8 = strip("tr808");
        let s9 = strip("tr909");
        // every quarter pixel of the sections and 100 px around them (a drag leaves the area)
        for yi in -400..(115 + 100) * 4 {
            let y = yi as f32 / 4.0;
            for xi in -400..(366 + 100) * 4 {
                let x = xi as f32 / 4.0;
                assert_eq!(
                    select_hit(&a8, o8.0 + x, o8.1 + y),
                    before::tr808_hit(x, y),
                    "808 {x} {y}"
                );
                assert_eq!(
                    select_hit(&a9, o9.0 + x, o9.1 + y),
                    before::tr909_hit(x, y),
                    "909 {x} {y}"
                );
            }
        }
        for xi in -400..(366 + 100) * 4 {
            let x = xi as f32 / 4.0;
            assert_eq!(
                strip_step(s8.1, s8.2, s8.0 + x),
                before::tr808_step(x),
                "{x}"
            );
            assert_eq!(
                strip_step(s9.1, s9.2, s9.0 + x),
                before::tr909_step(x),
                "{x}"
            );
        }
    }

    #[test]
    fn instrument_selection() {
        assert_eq!(tr808_hit(10.0, 75.0), Some(0));
        assert_eq!(tr808_hit(273.0, 75.0), Some(11));
        assert_eq!(tr808_hit(274.0, 75.0), None);
        assert_eq!(tr808_hit(78.0, 50.0), Some(3));
        assert_eq!(tr808_hit(183.0, 50.0), Some(7));
        assert_eq!(tr808_hit(50.0, 100.0), None);
        assert_eq!(tr909_hit(10.0, 10.0), Some(0));
        assert_eq!(tr909_hit(26.0, 10.0), Some(1));
        assert_eq!(tr909_hit(235.0, 10.0), Some(5));
        assert_eq!(tr909_hit(236.0, 75.0), Some(6));
        assert_eq!(tr909_hit(361.0, 83.0), Some(11));
        assert_eq!(tr909_hit(361.0, 86.0), None);
        assert_eq!(tr909_hit(362.0, 10.0), None);
        assert_eq!(tr909_hit(10.0, 50.0), None);
        assert_eq!(tr909_hit(100.0, 50.0), None);
        assert_eq!(tr808_step(13.0), None);
        assert_eq!(tr808_step(14.0), Some(0));
        assert_eq!(tr808_step(269.0), Some(15));
        assert_eq!(tr808_step(270.0), None);
        assert_eq!(tr909_step(26.0), Some(0));
        assert_eq!(tr909_step(361.0), Some(15));
        assert_eq!(tr909_step(362.0), None);
    }

    #[test]
    fn digits_right_aligned_with_blank_leading_zeros() {
        assert_eq!(digit_frames(Some(140), 3), [Some(1), Some(4), Some(0)]);
        assert_eq!(digit_frames(Some(3), 2), [None, Some(3)]);
        assert_eq!(digit_frames(Some(0), 2), [None, Some(0)]);
        assert_eq!(digit_frames(None, 2), [None, None]);
        assert_eq!(digit_frames(Some(1000), 3), [Some(9), Some(9), Some(9)]);
    }
}
