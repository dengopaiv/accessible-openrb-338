//! The generic SVG skin (task-266, docs/ui_spec.md section 22): one layout table and one part
//! set, drawn by this renderer. The modern panel ([`modern`]) is `assets/ui/layout.tsv` and the
//! embedded `assets/ui`; another SVG theme is another table and part set, no code.
//!
//! The renderer knows no part by name: every part it draws is named by the table (a row's
//! part, its parameters) or composed from a row's part by the naming rules of the part set
//! (assets/ui/README.md):
//! - a state part is the row's part's family (its name up to the last `_`) and the state:
//!   `on`/`off`, `rest`/`pressed`, `lit`, a value `0`, `1`, ...; a row's `on=` and `off=` name its
//!   parts outright (`-` for none); a held key shows `<family><on|off>_pressed` when the set has
//!   it;
//! - a meter's state 0..=10 is mapped through its `map=` to the part's number;
//! - a knob is `<part>_body` and `<part>_pointer`, turned about the rectangle's center;
//! - a fader is the row's part (the track) and `<family>handle`, the handle part's size;
//! - a display's part is `<prefix>0..9,<blank>`: a digit is `<prefix><d>`, its cell the size of
//!   `<prefix>0`;
//! - the wordmark (`logo`) animates when the set has its pieces (`logo_anim`).
//!
//! The colors the renderer draws itself (the mod window's name) are the table's too.

use std::sync::{Arc, OnceLock};

use egui::{Color32, Image, Ui, Vec2};

use super::{Ctl, Frame, Layout, Skin, table};
use crate::app::{View, centered_in, paint_part_in};
use crate::art::PartSet;
use crate::behavior;
use crate::model::{Look, Panel, meter_state};

/// An SVG skin: its table's rows by drawing layer, its layout and its part set.
pub struct SvgSkin {
    layout: Layout,
    parts: Arc<PartSet>,
    /// The backgrounds and the wordmark, in the table's order.
    under: Vec<Ctl>,
    /// The rows' focus bars.
    focus: Vec<Ctl>,
    /// The static art over the controls.
    over: Vec<Ctl>,
    /// The drum machines' selection marks.
    marks: Vec<Ctl>,
}

impl SvgSkin {
    /// The skin of the layout table `table` (the text of a `layout.tsv`) drawn with `parts`.
    pub fn new(table: &str, parts: Arc<PartSet>) -> Result<Self, String> {
        let rows = table::parse(table)?;
        let of = |kinds: &[&str]| -> Vec<Ctl> {
            rows.iter()
                .filter(|r| kinds.contains(&r.kind.as_str()))
                .cloned()
                .collect()
        };
        Ok(Self {
            layout: Layout::from_rows(&rows)?,
            parts,
            under: of(&["bg", "logo"]),
            focus: of(&["focus"]),
            over: of(&["fg"]),
            marks: of(&["mark"]),
        })
    }

    /// The part set.
    pub fn parts(&self) -> &PartSet {
        &self.parts
    }

    /// The part this skin draws for a control in its current state (`pressed`: the mouse or a
    /// computer key holds it; `now` the time in seconds). Not for knobs, faders and displays,
    /// which are composed (their row's part is returned).
    pub fn part_name(&self, panel: &Panel, c: &Ctl, pressed: bool, now: f64) -> String {
        let fam = c.family();
        match panel.look(c, now) {
            Look::Plain => c.part.clone(),
            Look::OnOff(on) => {
                let key = if on { "on" } else { "off" };
                if let Some(p) = c.param(key) {
                    return p.to_string();
                }
                let base = format!("{fam}{key}");
                // a routing key shows pressed while held, its lamp alone shows the state
                let held = format!("{base}_pressed");
                if pressed && self.parts.part(&held).is_some() {
                    held
                } else {
                    base
                }
            }
            Look::Push => format!("{fam}{}", if pressed { "pressed" } else { "rest" }),
            Look::Lit => format!("{fam}lit"),
            Look::Value(v) => format!("{fam}{v}"),
            Look::Meter(byte) => {
                let state = meter_state(byte);
                let v = c
                    .params
                    .nums("map")
                    .and_then(|m| m.get(state as usize).copied())
                    .map_or(state, |v| v as u8);
                format!("{fam}{v}")
            }
        }
    }

    /// The part `name` (`-`: none) at `(x, y, w, h)` in skin units.
    fn paint_part(&self, ui: &Ui, name: &str, view: &View, (x, y, w, h): (f32, f32, f32, f32)) {
        if name != "-" {
            paint_part_in(
                ui,
                &self.parts,
                name,
                view.rect(x, y, w, h),
                None,
                Color32::WHITE,
            );
        }
    }

    fn paint_row(&self, ui: &Ui, name: &str, view: &View, c: &Ctl) {
        self.paint_part(ui, name, view, (c.x, c.y, c.w, c.h));
    }

    /// A control (not a step key): its parts in its state.
    fn paint_control(&self, ui: &Ui, view: &View, f: &Frame, c: &Ctl, pressed: bool) {
        match c.kind.as_str() {
            "area" | "steps" => {}
            "modview" => {
                self.paint_row(ui, &c.part, view, c);
                self.paint_mod_view(ui, view, f, c);
            }
            "knob" => {
                let p = f.panel.knob(&c.id);
                self.paint_row(ui, &format!("{}_body", c.part), view, c);
                let angle = behavior::knob_angle(p).to_radians();
                let rect = view.rect(c.x, c.y, c.w, c.h);
                let pointer = format!("{}_pointer", c.part);
                let pivot = Vec2::splat(0.5);
                paint_part_in(
                    ui,
                    &self.parts,
                    &pointer,
                    rect,
                    Some((angle, pivot)),
                    Color32::WHITE,
                );
            }
            "fader" => {
                self.paint_row(ui, &c.part, view, c);
                let handle = format!("{}handle", c.family());
                let top = behavior::fader_handle_top(f.panel.fader(&c.id), &c.fader());
                if let Some(h) = self.parts.part(&handle) {
                    self.paint_part(ui, &handle, view, (c.x, c.y + top, h.w, h.h));
                }
            }
            "display" => self.paint_display(ui, view, c, f.panel.display(&c.id)),
            _ => {
                let name = self.part_name(f.panel, c, pressed, f.now);
                self.paint_row(ui, &name, view, c);
            }
        }
    }

    /// A seven-segment display: `n` cells, the width over a digit part's, centered.
    fn paint_display(&self, ui: &Ui, view: &View, c: &Ctl, value: Option<i32>) {
        let mut spec = c.part.split(',');
        let prefix = spec.next().unwrap_or("").trim_end_matches("0..9");
        let blank = spec.next().unwrap_or("-");
        let Some(cell) = self.parts.part(&format!("{prefix}0")) else {
            return;
        };
        let n = (c.w / cell.w) as usize;
        let x0 = c.x + (c.w - cell.w * n as f32) / 2.0;
        for (i, d) in behavior::digit_frames(value, n).into_iter().enumerate() {
            let name = d.map_or(blank.to_string(), |d| format!("{prefix}{d}"));
            let x = x0 + cell.w * i as f32;
            self.paint_part(ui, &name, view, (x, c.y, cell.w, cell.h));
        }
    }

    /// The 808's mod window: the kit's splash picture in `picture` (x y w h in the window)
    /// under the `glass` part, the kit's name in `name` in the `font` size and `ink` color.
    fn paint_mod_view(&self, ui: &Ui, view: &View, f: &Frame, c: &Ctl) {
        if let Some([px, py, pw, ph]) = c.params.nums("picture").as_deref().and_then(rect4) {
            let r = view.rect(c.x + px, c.y + py, pw, ph);
            if let Some(tex) = f.splash {
                Image::new((tex.id(), r.size())).paint_at(ui, r);
            }
            if let Some(glass) = c.param("glass") {
                self.paint_part(ui, glass, view, (c.x + px, c.y + py, pw, ph));
            }
        }
        if let Some([nx, ny, nw, nh]) = c.params.nums("name").as_deref().and_then(rect4) {
            let r = view.rect(c.x + nx, c.y + ny, nw, nh);
            let size = c.params.num("font").unwrap_or(nh / 3.0);
            let ink = c.param("ink").and_then(hex_color).unwrap_or(Color32::WHITE);
            let galley = ui.painter().layout(
                f.mod_name.to_string(),
                egui::FontId::proportional(size * view.scale),
                ink,
                r.width(),
            );
            let pos = centered_in(r, galley.size());
            ui.painter()
                .with_clip_rect(r)
                .galley(pos, galley, Color32::WHITE);
        }
    }

    /// A selection mark: its part when its instrument is selected, else nothing.
    fn paint_mark(&self, ui: &Ui, view: &View, f: &Frame, c: &Ctl) {
        let slot = c.params.num("slot").unwrap_or(0.0) as usize;
        let upper = c.param("upper") == Some("1");
        if super::mark(f.panel, &c.section, upper, slot) {
            self.paint_row(ui, &c.part, view, c);
        }
    }
}

/// Four numbers as an array.
fn rect4(v: &[f32]) -> Option<[f32; 4]> {
    v.try_into().ok()
}

/// A color `rrggbb` (an optional `#`).
fn hex_color(s: &str) -> Option<Color32> {
    let s = s.trim_start_matches('#');
    let v = u32::from_str_radix(s, 16).ok().filter(|_| s.len() == 6)?;
    Some(Color32::from_rgb((v >> 16) as u8, (v >> 8) as u8, v as u8))
}

impl Skin for SvgSkin {
    fn layout(&self) -> &Layout {
        &self.layout
    }

    /// The layers: the backgrounds and the wordmark, the focus bars, the controls (each in its
    /// state), the static art over them, the step keys and the selection marks.
    fn paint(&self, ui: &mut Ui, view: &View, f: &Frame) {
        for r in &self.under {
            if r.kind == "logo" {
                let rect = view.rect(r.x, r.y, r.w, r.h);
                crate::logo_anim::paint_logo_in(ui, &self.parts, &r.part, rect, f.logo_t);
                continue;
            }
            self.paint_row(ui, &r.part, view, r);
        }
        for r in &self.focus {
            let on = r.row() == Some(f.panel.focus);
            let name = r.param("on").filter(|_| on).unwrap_or(&r.part);
            self.paint_row(ui, name, view, r);
        }
        let controls = &self.layout.controls;
        for (i, c) in controls.iter().enumerate() {
            let pressed = f.pressed.get(i).copied().unwrap_or(false);
            self.paint_control(ui, view, f, c, pressed);
        }
        for r in &self.over {
            self.paint_row(ui, &r.part, view, r);
        }
        for c in controls.iter().filter(|c| c.kind == "steps") {
            let name = self.part_name(f.panel, c, false, 0.0);
            self.paint_row(ui, &name, view, c);
        }
        for m in &self.marks {
            self.paint_mark(ui, view, f, m);
        }
    }
}

/// The modern skin: `assets/ui/layout.tsv` and the embedded parts.
pub fn modern() -> Arc<SvgSkin> {
    static SKIN: OnceLock<Arc<SvgSkin>> = OnceLock::new();
    SKIN.get_or_init(|| {
        let skin = SvgSkin::new(crate::layout::LAYOUT_TSV, PartSet::embedded().clone());
        Arc::new(skin.expect("the modern skin's layout.tsv"))
    })
    .clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every part the modern table names is in the embedded set.
    #[test]
    fn the_modern_table_names_only_parts_of_its_set() {
        let skin = modern();
        let rows = table::parse(crate::layout::LAYOUT_TSV).unwrap();
        let has = |n: &str| n == "-" || skin.parts().part(n).is_some();
        for r in &rows {
            match r.kind.as_str() {
                "panel" | "row" | "strip" | "select" | "display" => {}
                "knob" => {
                    assert!(has(&format!("{}_body", r.part)), "{}", r.id);
                    assert!(has(&format!("{}_pointer", r.part)), "{}", r.id);
                }
                _ => assert!(has(&r.part), "{}: {}", r.id, r.part),
            }
            for key in ["on", "off", "glass"] {
                if let Some(p) = r.param(key) {
                    assert!(has(p), "{} {key}={p}", r.id);
                }
            }
        }
        assert_eq!(
            hex_color("e8e2c2"),
            Some(Color32::from_rgb(0xe8, 0xe2, 0xc2))
        );
        assert_eq!(hex_color("e8e2c"), None);
    }
}
