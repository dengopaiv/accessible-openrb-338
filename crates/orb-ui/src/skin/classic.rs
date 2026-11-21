//! The classic skin (task-267, docs/ui_spec.md section 21): the original's panel drawn from a
//! kit's 58 bitmaps ([`super::bitmaps`]) at the original's geometry, scaled by nearest pixel
//! only. Its layout table (`assets/ui/classic.tsv`, the skin table format of
//! `skin::table`) holds the original's view rectangles and hit areas; this renderer draws each
//! row from the bitmap the row names, in the table's order. It has no input code: the one
//! controller (`controller.rs`) runs the input on its layout.
//!
//! A row's `part` is a bitmap id (`-`: nothing drawn, an input-only control), and its
//! parameters say how it is drawn:
//! - `n` the strip's frame count (1), `v=1` a vertical strip, `frame` a fixed frame (0);
//! - `src=sx,sy` a part of the frame from that offset, the row's size (the gain-reduction LEDs, one
//!   7 x 10 part each);
//! - `frames=a,b,...` maps the control's state to a frame, `-` for none: the state's index is
//!   off/on 0/1, rest/pressed 0/1, lit 1, a value, or a meter's frame;
//! - a knob draws frame `((n - 1) * p + 500) / 1000`; a fader its track (the part) and its handle
//!   `cap=` at the handle's place on the row's height (`handle=`); a display `digits=` digits
//!   `pitch=` apart (9) from the 11-frame strip;
//! - `pick` rows draw a strip whose frame is `base=` plus the index of the first control of `of=`
//!   that is on (0 when none is): the pattern grid and the bank row;
//! - `mark` rows draw their part when their drum machine's instrument `slot=` is selected (the
//!   909's selected-instrument LED).

use std::sync::{Arc, Mutex, OnceLock};

use egui::{Color32, ColorImage, Rect, TextureHandle, TextureOptions, Ui, pos2};

use super::bitmaps::{self, Bitmaps, Op};
use super::{Ctl, Frame, Layout, Skin, table};
use crate::app::View;
use crate::model::{Look, peak_meter_frame};

/// The classic skin's layout table.
pub const CLASSIC_TSV: &str = include_str!("../../../../assets/ui/classic.tsv");

/// A row to draw: the row, its control index in the layout (a control), and for a `pick` row
/// the control indices of its `of=` list.
struct Row {
    c: Ctl,
    control: Option<usize>,
    of: Vec<usize>,
}

/// What was composed last.
#[derive(Default)]
struct Cache {
    ops: Vec<Op>,
    tex: Option<TextureHandle>,
    composed: u64,
}

/// The classic skin: its layout table and a kit's bitmaps.
pub struct ClassicSkin {
    layout: Layout,
    rows: Vec<Row>,
    bitmaps: Bitmaps,
    cache: Mutex<Cache>,
}

impl ClassicSkin {
    /// The skin of the layout table `table` drawn from `bitmaps`.
    pub fn new(table: &str, bitmaps: Bitmaps) -> Result<Self, String> {
        let all = table::parse(table)?;
        let layout = Layout::from_rows(&all)?;
        let index = |id: &str| layout.controls.iter().position(|c| c.id == id);
        let mut n = 0;
        let mut rows = Vec::new();
        for c in all {
            let control = table::is_control(&c.kind).then(|| {
                n += 1;
                n - 1
            });
            let of = match c.param("of") {
                Some(list) => list
                    .split(',')
                    .map(|id| index(id).ok_or_else(|| format!("{}: no control {id}", c.id)))
                    .collect::<Result<_, _>>()?,
                None => Vec::new(),
            };
            if c.part != "-" && c.part.parse::<u16>().is_err() {
                return Err(format!("{}: part {:?} is no bitmap id", c.id, c.part));
            }
            rows.push(Row { c, control, of });
        }
        Ok(Self {
            layout,
            rows,
            bitmaps,
            cache: Mutex::new(Cache::default()),
        })
    }

    /// The classic skin of a kit: its bitmaps, the built-in ones where it has none.
    pub fn of_kit(kit: &orb_cli::mods::Kit) -> Arc<Self> {
        Arc::new(Self::new(CLASSIC_TSV, Bitmaps::of_kit(kit)).expect("classic.tsv"))
    }

    /// The classic skin with the built-in kit's bitmaps (made once).
    pub fn builtin() -> Arc<Self> {
        static SKIN: OnceLock<Arc<ClassicSkin>> = OnceLock::new();
        SKIN.get_or_init(|| {
            let b = Bitmaps::builtin().clone();
            Arc::new(Self::new(CLASSIC_TSV, b).expect("classic.tsv"))
        })
        .clone()
    }

    pub fn bitmaps(&self) -> &Bitmaps {
        &self.bitmaps
    }

    /// The scale N of the bitmap set: the panel is composed at 624N x 510N pixels.
    pub fn scale(&self) -> usize {
        self.bitmaps.scale()
    }

    /// How many times the panel image was composed (it is, only when what it shows changes).
    pub fn composed(&self) -> u64 {
        self.cache.lock().map_or(0, |c| c.composed)
    }

    /// The blits that draw the state of `f`, in the table's order.
    pub fn ops(&self, f: &Frame) -> Vec<Op> {
        let mut o = Vec::with_capacity(self.rows.len() + 64);
        for row in &self.rows {
            let c = &row.c;
            let Ok(id) = c.part.parse::<u16>() else {
                continue;
            };
            let n = c.params.num("n").map_or(1, |n| n as u16);
            let fixed = c.params.num("frame").map_or(0, |f| f as u16);
            let draw = |frame: u16| op(c, id, n, frame);
            match c.kind.as_str() {
                "panel" | "row" | "strip" | "select" | "logo" => {}
                "bg" | "fg" => o.push(draw(fixed)),
                "focus" => o.push(draw(u16::from(c.row() == Some(f.panel.focus)))),
                "pick" => {
                    let on = row.of.iter().position(|&i| {
                        let ctl = &self.layout.controls[i];
                        f.panel.look(ctl, f.now) == Look::OnOff(true)
                    });
                    let base = c.params.num("base").unwrap_or(0.0) as u16;
                    o.push(draw(on.map_or(0, |i| base + i as u16)));
                }
                "mark" => {
                    let slot = c.params.num("slot").unwrap_or(0.0) as usize;
                    let upper = c.param("upper") == Some("1");
                    if super::mark(f.panel, f.lit, &c.section, upper, slot).selected {
                        o.push(draw(fixed));
                    }
                }
                _ => {
                    if (f.hidden)(&c.id) {
                        continue;
                    }
                    let pressed = row
                        .control
                        .and_then(|i| f.pressed.get(i).copied())
                        .unwrap_or(false);
                    self.control(&mut o, f, c, (id, n), pressed);
                }
            }
        }
        o
    }

    /// A control's blits in its state.
    fn control(&self, o: &mut Vec<Op>, f: &Frame, c: &Ctl, (id, n): (u16, u16), pressed: bool) {
        let p = f.panel;
        match c.kind.as_str() {
            "knob" => o.push(op(c, id, n, bitmaps::knob_frame(n, p.knob(&c.id)))),
            "fader" => {
                o.push(op(c, id, 1, 0));
                if let Some(cap) = c.params.num("cap") {
                    let fd = c.fader();
                    let top = bitmaps::handle_top(p.fader(&c.id), fd.track, fd.handle);
                    let (x, y) = place(c);
                    let w = c.w as i32;
                    o.push(Op::pic(cap as u16, x, y + top, w, fd.handle));
                }
            }
            "display" => {
                let k = c.params.num("digits").unwrap_or(1.0) as usize;
                let pitch = c.params.num("pitch").unwrap_or(9.0) as i32;
                let (x, y) = place(c);
                for (i, d) in bitmaps::digits(p.display(&c.id), k).into_iter().enumerate() {
                    let x = x + pitch * i as i32;
                    o.push(Op::h(id, n, d, x, y, pitch, c.h as i32));
                }
            }
            _ => {
                let state = match p.look(c, f.now) {
                    Look::Plain => 0,
                    Look::OnOff(on) => u16::from(on),
                    Look::Push => u16::from(pressed),
                    Look::Lit => 1,
                    Look::Value(v) => v as u16,
                    Look::Meter(byte) => peak_meter_frame(byte, n),
                };
                let frame = match c.param("frames") {
                    Some(map) => match map.split(',').nth(state as usize) {
                        Some(f) => f.parse::<u16>().ok(),
                        None => None,
                    },
                    None => Some(state),
                };
                if let Some(frame) = frame {
                    o.push(op(c, id, n, frame));
                }
            }
        }
    }

    /// The panel image of the state of `f` at the set's scale.
    pub fn compose(&self, f: &Frame) -> ColorImage {
        bitmaps::compose(&self.bitmaps, &self.ops(f))
    }

    /// The panel image of `panel` at the time `now`, with nothing held and no note-mode
    /// lights (the standalone app's), at the set's scale.
    pub fn compose_panel(&self, panel: &crate::model::Panel, now: f64) -> ColorImage {
        let pressed = vec![false; self.layout.controls.len()];
        let lit = crate::embed::Lit::default();
        let hidden = |_: &str| false;
        self.compose(&Frame {
            panel,
            pressed: &pressed,
            now,
            lit: &lit,
            plugin: false,
            hidden: &hidden,
            logo_t: None,
            mod_name: "",
            splash: None,
        })
    }
}

/// Where a row draws: its `at=x,y`, else its rectangle's corner.
fn place(c: &Ctl) -> (i32, i32) {
    match c.params.nums("at").as_deref() {
        Some([x, y]) => (*x as i32, *y as i32),
        _ => (c.x as i32, c.y as i32),
    }
}

/// The blit of frame `frame` of the row's bitmap `id` (`n` frames) at its place.
fn op(c: &Ctl, id: u16, n: u16, frame: u16) -> Op {
    let (x, y) = place(c);
    let (sx, sy) = match c.params.nums("src").as_deref() {
        Some([sx, sy]) => (*sx as i32, *sy as i32),
        _ => (0, 0),
    };
    Op {
        id,
        n,
        vertical: c.param("v") == Some("1"),
        frame,
        sx,
        sy,
        x,
        y,
        w: c.w as i32,
        h: c.h as i32,
    }
}

impl Skin for ClassicSkin {
    fn layout(&self) -> &Layout {
        &self.layout
    }

    /// Composes the panel image when what it shows changed, and draws it at `view` scaled by
    /// nearest pixel (the user, 2026-11-21: always nearest).
    fn paint(&self, ui: &mut Ui, view: &View, f: &Frame) {
        let ops = self.ops(f);
        let Ok(mut cache) = self.cache.lock() else {
            return;
        };
        if cache.tex.is_none() || cache.ops != ops {
            let img = bitmaps::compose(&self.bitmaps, &ops);
            cache.composed += 1;
            match &mut cache.tex {
                Some(t) => t.set(img, TextureOptions::NEAREST),
                None => {
                    let t = ui
                        .ctx()
                        .load_texture("classic-panel", img, TextureOptions::NEAREST);
                    cache.tex = Some(t);
                }
            }
            cache.ops = ops;
        }
        if let Some(tex) = &cache.tex {
            let size = self.layout.size;
            let rect = view.rect(0.0, 0.0, size.x, size.y);
            let uv = Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0));
            ui.painter().image(tex.id(), rect, uv, Color32::WHITE);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The table parses, every bitmap it names is one of the 58, and every `of=` control
    /// exists.
    #[test]
    fn the_table_names_only_the_58_bitmaps() {
        let skin = ClassicSkin::builtin();
        for r in &skin.rows {
            if let Ok(id) = r.c.part.parse::<u16>() {
                assert!(bitmaps::IDS.contains(&id), "{}: {id}", r.c.id);
            }
            if let Some(cap) = r.c.params.num("cap") {
                assert!(bitmaps::IDS.contains(&(cap as u16)), "{}", r.c.id);
            }
        }
        assert_eq!(skin.layout().size, egui::vec2(624.0, 510.0));
    }
}
