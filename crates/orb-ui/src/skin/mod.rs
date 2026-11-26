//! The skin layer (task-266, docs/ui_spec.md section 22): a skin is one look of the main
//! window's panel. It gives
//! - its [`Layout`]: the panel's size in its own units, its controls (the ids every skin shares, so
//!   the model, the keys and the status line are shared too; the kind; the rectangle; the kind's
//!   geometry as parameters) and the areas the controller tests that are no single control's (the
//!   rows' focus areas, the logo, the drum machines' step strips and instrument hit areas);
//! - a renderer ([`Skin::paint`]) that draws the panel's state ([`Frame`]) at a screen place.
//!
//! One controller (`controller.rs`) runs all the input on whatever layout the current skin
//! gives; a skin has no input code. A skin may move controls, add some and lack others: a
//! control's keyboard shortcuts work whatever the skin shows. The desk and the panel's shadows
//! (`backdrop`) are drawn around any skin.
//!
//! The modern panel is [`svg::modern`]: the generic SVG renderer over one layout table
//! (`assets/ui/layout.tsv`) and one part set (the embedded `assets/ui`). The classic panel is
//! [`classic::ClassicSkin`] (task-267): the original's geometry (`assets/ui/classic.tsv`) drawn
//! from a kit's bitmaps ([`bitmaps`]).

pub mod bitmaps;
pub mod classic;
pub mod fade;
pub mod svg;
pub mod table;

use egui::{Pos2, Ui, Vec2};

pub use table::{Ctl, Params};

use crate::app::View;
use crate::embed::{LIT_808, LIT_909, Lit};
use crate::model::Panel;

/// What a skin gives the controller: everything in the skin's units.
#[derive(Clone, Debug, Default)]
pub struct Layout {
    /// The panel's size (its aspect is the original's, 624:510).
    pub size: Vec2,
    /// The controls, in the order they take input and are drawn.
    pub controls: Vec<Ctl>,
    /// The drum machines' step strips (`strip`; the section names the machine).
    pub strips: Vec<Ctl>,
    /// The drum machines' instrument hit areas (`select`), in the order they are tested.
    pub selects: Vec<Ctl>,
    /// The rows' focus click areas (`row`).
    pub rows: Vec<Ctl>,
    /// The logo's click area (`logo`): it plays the mod's startup sound.
    pub logo: Option<Ctl>,
}

impl Layout {
    /// The layout of a table's rows: the `panel` row's size (required), the controls and the
    /// controller's areas.
    pub fn from_rows(rows: &[Ctl]) -> Result<Self, String> {
        let panel = rows
            .iter()
            .find(|r| r.kind == "panel")
            .ok_or("the table has no panel row (the skin's size)")?;
        let of =
            |kind: &str| -> Vec<Ctl> { rows.iter().filter(|r| r.kind == kind).cloned().collect() };
        Ok(Self {
            size: Vec2::new(panel.w, panel.h),
            controls: rows
                .iter()
                .filter(|r| table::is_control(&r.kind))
                .cloned()
                .collect(),
            strips: of("strip"),
            selects: of("select"),
            rows: of("row"),
            logo: rows.iter().find(|r| r.kind == "logo").cloned(),
        })
    }

    /// The row (0..=3) whose focus area holds `p`.
    pub fn row_at(&self, p: Pos2) -> Option<usize> {
        self.rows.iter().find(|r| r.contains(p)).and_then(Ctl::row)
    }

    /// Whether `p` is on the logo.
    pub fn on_logo(&self, p: Pos2) -> bool {
        self.logo.as_ref().is_some_and(|l| l.contains(p))
    }

    /// The instrument of the drum machine `section` (`tr808`, `tr909`) under `p`.
    pub fn select_hit(&self, section: &str, p: Pos2) -> Option<usize> {
        let areas: Vec<_> = self
            .selects
            .iter()
            .filter(|s| s.section == section)
            .map(Ctl::select_area)
            .collect();
        crate::behavior::select_hit(&areas, p.x, p.y)
    }

    /// The first control `id`.
    pub fn control(&self, id: &str) -> Option<&Ctl> {
        self.controls.iter().find(|c| c.id == id)
    }
}

/// The panel's state a renderer draws, for one frame.
pub struct Frame<'a> {
    pub panel: &'a Panel,
    /// Whether each control of the layout shows pressed (the mouse or a computer key holds
    /// it), by index in [`Layout::controls`].
    pub pressed: &'a [bool],
    /// The egui time, seconds (the 303's lit notes).
    pub now: f64,
    /// The voices lit in the plugin's note mode (none in the standalone app).
    pub lit: &'a Lit,
    /// Whether this is the plugin's editor.
    pub plugin: bool,
    /// The controls the panel does not show (the plugin's hidden transport controls).
    pub hidden: &'a dyn Fn(&str) -> bool,
    /// The wordmark's startup animation time (None: its final state).
    pub logo_t: Option<f32>,
    /// The kit's name and splash picture (the 808's mod window).
    pub mod_name: &'a str,
    pub splash: Option<&'a egui::TextureHandle>,
}

/// A look of the panel.
pub trait Skin: Send + Sync {
    /// The skin's layout, which the controller runs the input on.
    fn layout(&self) -> &Layout;

    /// Draws the panel's state at `view` (the skin's rectangle on screen).
    fn paint(&self, ui: &mut Ui, view: &View, frame: &Frame);

    /// Draws the panel over what is already drawn, with opacity `alpha` (0..=1): the
    /// crossfade's overlay (task-268). Only a skin that is one opaque image can fade as a
    /// whole; the default draws nothing for a part alpha.
    fn paint_over(&self, ui: &mut Ui, view: &View, frame: &Frame, alpha: f32) {
        if alpha >= 1.0 {
            self.paint(ui, view, frame);
        }
    }
}

/// A drum machine's selection mark: whether its instrument is the selected one and whether its
/// voice is lit in the plugin's note mode.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Mark {
    pub selected: bool,
    pub lit: bool,
}

/// The state of the selection mark `slot` of the drum machine `section` (`tr808`, `tr909`):
/// on the 808 the legend's chips (`upper` false, 0..=11) and the upper labels' (`upper`, the
/// slots 3..=7 as 0..=4); on the 909 its instruments 0..=11. The 808's selected instrument
/// shows on the upper label when its shared slot's switch is up; a lit voice shows on the chip
/// of the sound played (task-202): the legend's 1..=11, then the upper labels.
pub fn mark(panel: &Panel, lit: &Lit, section: &str, upper: bool, slot: usize) -> Mark {
    if section == "tr808" {
        let i = panel.inst808.min(11);
        let up =
            (3..=7).contains(&i) && panel.switch_position(&format!("tr808.slot{i}_switch")) == 0;
        let selected = (up, if up { i - 3 } else { i }) == (upper, slot);
        let lit = if upper {
            lit.mask[LIT_808] & 1 << (slot + 11) != 0
        } else if slot == 0 {
            lit.ac[LIT_808]
        } else {
            slot <= 11 && lit.mask[LIT_808] & 1 << (slot - 1) != 0
        };
        Mark { selected, lit }
    } else {
        let selected = !upper && panel.inst909.min(11) == slot;
        let lit = !upper
            && if slot == 0 {
                lit.ac[LIT_909]
            } else {
                slot <= 11 && lit.mask[LIT_909] & 1 << (slot - 1) != 0
            };
        Mark { selected, lit }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_marks_follow_the_selection_and_the_lit_voices() {
        let mut p = Panel::new();
        let none = Lit::default();
        p.inst808 = 1;
        assert!(mark(&p, &none, "tr808", false, 1).selected);
        assert!(!mark(&p, &none, "tr808", false, 2).selected);
        // LT (3) on the upper label when its slot's switch is up (0)
        p.inst808 = 3;
        p.set_switch("tr808.slot3_switch", 0);
        assert!(mark(&p, &none, "tr808", true, 0).selected);
        assert!(!mark(&p, &none, "tr808", false, 3).selected);
        p.set_switch("tr808.slot3_switch", 1);
        assert!(mark(&p, &none, "tr808", false, 3).selected);
        // the lit sounds: bit b is the legend's chip b + 1 up to 10, then the upper labels
        let mut lit = Lit::default();
        lit.mask[LIT_808] = 1 << 0 | 1 << 11;
        lit.ac[LIT_909] = true;
        lit.mask[LIT_909] = 1 << 10;
        assert!(mark(&p, &lit, "tr808", false, 1).lit);
        assert!(mark(&p, &lit, "tr808", true, 0).lit);
        assert!(!mark(&p, &lit, "tr808", false, 0).lit);
        assert!(mark(&p, &lit, "tr909", false, 0).lit);
        assert!(mark(&p, &lit, "tr909", false, 11).lit);
        p.inst909 = 4;
        assert!(mark(&p, &lit, "tr909", false, 4).selected);
    }
}
