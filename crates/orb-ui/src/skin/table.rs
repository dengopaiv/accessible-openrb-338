//! A skin's layout table (docs/ui_spec.md section 22, assets/ui/README.md): one row per
//! control or per piece of the skin's geometry, `id section kind part x y w h [params]`,
//! tab-separated, with a header line. The rectangles are in the skin's units; `params` is
//! `key=value` pairs separated by spaces.

use egui::Pos2;

use crate::behavior::{self, Fader, Segment};

/// The kinds of rows that are not controls: the skin's size, its art layers and the areas the
/// controller tests that are not one control's (docs/ui_spec.md section 22). `pick` is the
/// classic skin's picture of a group of controls (the pattern grid, the bank row).
pub const GEOMETRY_KINDS: [&str; 10] = [
    "panel", "bg", "logo", "focus", "row", "fg", "strip", "select", "mark", "pick",
];

/// Whether a row of `kind` is a control (has an id the panel model knows, a state and maybe
/// input) rather than a piece of the skin's geometry.
pub fn is_control(kind: &str) -> bool {
    !GEOMETRY_KINDS.contains(&kind)
}

/// A row's parameters: `key=value` pairs in the table's order.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Params(Vec<(String, String)>);

impl Params {
    /// Parses `key=value key=value`; a word without `=` is an error.
    pub fn parse(s: &str) -> Result<Self, String> {
        s.split_whitespace()
            .map(|kv| {
                kv.split_once('=')
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .ok_or_else(|| format!("parameter {kv:?} is not key=value"))
            })
            .collect::<Result<_, _>>()
            .map(Self)
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.0
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    }

    /// A number parameter; None when absent or not a number.
    pub fn num(&self, key: &str) -> Option<f32> {
        self.get(key).and_then(|v| v.parse().ok())
    }

    /// A list of numbers separated by commas; None when absent or malformed.
    pub fn nums(&self, key: &str) -> Option<Vec<f32>> {
        self.get(key)?.split(',').map(|v| v.parse().ok()).collect()
    }
}

/// One row of a layout table: a control (its id is shared by every skin, so the model, the
/// keys and the status line are too) or a piece of the skin's geometry ([`GEOMETRY_KINDS`]).
#[derive(Clone, Debug, Default)]
pub struct Ctl {
    pub id: String,
    pub section: String,
    pub kind: String,
    pub part: String,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub params: Params,
}

/// The drag span of a knob without a `span` parameter (the original's, outside the
/// instruments).
pub const DEFAULT_KNOB_SPAN: f32 = 64.0;

impl Ctl {
    /// The part's name without its state: `btn_play_lit` -> `btn_play_`. Not for knobs,
    /// faders and displays, whose parts are composed.
    pub fn family(&self) -> &str {
        match self.part.rfind('_') {
            Some(i) => &self.part[..=i],
            None => &self.part,
        }
    }

    /// The control's name within its section (`tb303_1.cutoff` -> `cutoff`).
    pub fn name(&self) -> &str {
        self.id.split_once('.').map_or(&self.id, |(_, n)| n)
    }

    pub fn param(&self, key: &str) -> Option<&str> {
        self.params.get(key)
    }

    /// Whether a point (skin units) is inside the rectangle, the right and bottom edges out.
    pub fn contains(&self, p: Pos2) -> bool {
        (self.x..self.x + self.w).contains(&p.x) && (self.y..self.y + self.h).contains(&p.y)
    }

    /// A knob's vertical drag for its full range, in skin units (`span`).
    pub fn knob_span(&self) -> f32 {
        self.params.num("span").unwrap_or(DEFAULT_KNOB_SPAN)
    }

    /// A fader's geometry: its track is the rectangle's height, its handle `handle` high.
    pub fn fader(&self) -> Fader {
        Fader {
            track: self.h.round() as i32,
            handle: self.params.num("handle").map_or(11, |h| h.round() as i32),
        }
    }

    /// A step strip's cell width: the width over `cells` (16).
    pub fn cell(&self) -> f32 {
        self.w / self.params.num("cells").unwrap_or(16.0).max(1.0)
    }

    /// An instrument hit area's cells: `cells=x0:w:first,...` (skin units).
    pub fn segments(&self) -> Vec<Segment> {
        self.param("cells")
            .unwrap_or("")
            .split(',')
            .filter_map(|s| {
                let mut f = s.split(':');
                Some(Segment {
                    x0: f.next()?.parse().ok()?,
                    w: f.next()?.parse().ok()?,
                    first: f.next()?.parse().ok()?,
                })
            })
            .collect()
    }

    /// The row index (0..=3) of a focus bar or a row's click area (`row`).
    pub fn row(&self) -> Option<usize> {
        self.params.num("row").map(|r| r as usize)
    }

    /// The instrument hit area of a `select` row.
    pub fn select_area(&self) -> behavior::SelectArea {
        behavior::SelectArea {
            x: self.x,
            y: self.y,
            w: self.w,
            h: self.h,
            segments: self.segments(),
            count: self.params.num("count").map_or(12, |c| c as i32),
            open_left: self.param("open") == Some("1"),
        }
    }
}

/// Parses a layout table.
pub fn parse(text: &str) -> Result<Vec<Ctl>, String> {
    text.lines()
        .enumerate()
        .skip(1)
        .filter(|(_, line)| !line.trim().is_empty())
        .map(|(i, line)| {
            let f: Vec<&str> = line.split('\t').collect();
            if f.len() < 8 {
                return Err(format!("line {}: {} fields, not 8 or 9", i + 1, f.len()));
            }
            let n = |k: usize| {
                f[k].parse::<f32>()
                    .map_err(|_| format!("line {}: {:?} is not a number", i + 1, f[k]))
            };
            Ok(Ctl {
                id: f[0].into(),
                section: f[1].into(),
                kind: f[2].into(),
                part: f[3].into(),
                x: n(4)?,
                y: n(5)?,
                w: n(6)?,
                h: n(7)?,
                params: Params::parse(f.get(8).copied().unwrap_or(""))
                    .map_err(|e| format!("line {}: {e}", i + 1))?,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rows_and_params() {
        let t = "id\tsection\tkind\tpart\tx\ty\tw\th\tparams\n\
                 a.k\ta\tknob\tknob_x\t1\t2\t3\t4\tspan=36\n\
                 a.f\ta\tfader\tfader_track\t0\t0\t15\t45\thandle=11\n\
                 a.b\ta\ttrigger\tbtn_rest\t0\t0\t5\t5\n";
        let rows = parse(t).unwrap();
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].knob_span(), 36.0);
        assert_eq!(rows[2].knob_span(), DEFAULT_KNOB_SPAN);
        let f = rows[1].fader();
        assert_eq!((f.track, f.handle, f.travel()), (45, 11, 34));
        assert_eq!(rows[2].family(), "btn_");
        assert!(rows[2].contains(Pos2::new(0.0, 4.9)) && !rows[2].contains(Pos2::new(5.0, 1.0)));
        assert!(parse("h\nx\ty\n").is_err());
        assert!(Params::parse("a=1 b").is_err());
        let p = Params::parse("cells=94:22:0,116:42:1 n=1,2").unwrap();
        assert_eq!(p.nums("n"), Some(vec![1.0, 2.0]));
        let c = Ctl {
            params: p,
            ..Default::default()
        };
        assert_eq!(c.segments().len(), 2);
        assert_eq!(c.segments()[1].first, 1);
    }
}
