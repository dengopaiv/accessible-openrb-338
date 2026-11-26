//! The modern skin's layout table (`assets/ui/layout.tsv`, written by orb-ui-art): where its
//! controls sit, in panel pixels. The table also holds the skin's geometry (the backgrounds,
//! the focus bars and areas, the drum machines' strips and hit areas, the selection marks;
//! `skin::table`), which the modern skin reads (`skin::svg::modern`).

pub use crate::skin::Ctl;

/// The original panel's size: the modern skin's units, and the aspect every skin keeps.
pub const PANEL_W: f32 = 624.0;
pub const PANEL_H: f32 = 510.0;

/// The space the app's windows and dialogs keep between their frame and the window's edge.
pub const DIALOG_MARGIN: f32 = 12.0;

/// The smallest main window (inner size in points, the menu bar included; the standalone's
/// viewport minimum and the plugin editor's, task-297 and task-299; independent of the UI scale,
/// which sets pixels per point, and of the panel's own fit). Every window the user cannot
/// resize fits in it with [`DIALOG_MARGIN`] around: the widest are the 808 and 909 pattern
/// editors (512 wide), the highest the 303 pattern editor (461 high). The test
/// `tests/min_window.rs` measures them all and fails when a window outgrows this or the minimum
/// is larger than they need, so the numbers cannot drift.
pub const MIN_WINDOW_W: f32 = 536.0;
pub const MIN_WINDOW_H: f32 = 485.0;

/// The modern skin's layout table.
pub const LAYOUT_TSV: &str = include_str!("../../../assets/ui/layout.tsv");

/// The modern skin's controls (the table's rows that are controls, in its order): the ids the
/// panel model knows.
pub fn controls() -> Vec<Ctl> {
    crate::skin::table::parse(LAYOUT_TSV)
        .expect("layout.tsv parses")
        .into_iter()
        .filter(|c| crate::skin::table::is_control(&c.kind))
        .collect()
}

/// The origins of the drum machine sections in the modern skin (panel pixels), for the tests'
/// pointer math (the table's `tr808.bg` and `tr909.bg`).
pub const TR808_ORIGIN: (f32, f32) = (90.0, 280.0);
pub const TR909_ORIGIN: (f32, f32) = (90.0, 395.0);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skin::Skin;

    fn rows() -> Vec<Ctl> {
        crate::skin::table::parse(LAYOUT_TSV).unwrap()
    }

    #[test]
    fn layout_has_every_control() {
        let c = controls();
        // 348 before task-179, which added each 303's rest LED
        assert_eq!(c.len(), 350);
        assert!(c.iter().all(|c| c.w > 0.0 && c.h > 0.0));
        let play = c.iter().find(|c| c.id == "transport.play").unwrap();
        assert_eq!((play.x, play.y, play.w, play.h), (112.0, 21.0, 32.0, 25.0));
        assert_eq!(play.family(), "btn_play_");
        assert_eq!(play.name(), "play");
    }

    /// The backgrounds the modern skin draws, the foregrounds and the focus bars are the art
    /// generator's places, the parts' sizes.
    #[test]
    fn backgrounds_match_the_art_generator() {
        let parts = orb_ui_art::parts::all_parts();
        let l = orb_ui_art::layout::layout(&parts);
        let size = |name: &str| {
            let p = parts.iter().find(|p| p.name == name).unwrap();
            (p.w, p.h)
        };
        let rows = rows();
        let mut want: Vec<(String, f32, f32)> = l
            .backgrounds
            .into_iter()
            .filter(|(n, _, _)| !n.starts_with("focus_"))
            .collect();
        let mut got: Vec<(String, f32, f32)> = rows
            .iter()
            .filter(|r| r.kind == "bg" || r.kind == "logo")
            .map(|r| {
                assert_eq!((r.w, r.h), size(&r.part), "{}", r.id);
                (r.part.clone(), r.x, r.y)
            })
            .collect();
        want.sort_by(|a, b| a.partial_cmp(b).unwrap());
        got.sort_by(|a, b| a.partial_cmp(b).unwrap());
        assert_eq!(got, want);
        let fg: Vec<(String, f32, f32)> = rows
            .iter()
            .filter(|r| r.kind == "fg")
            .map(|r| {
                assert_eq!((r.w, r.h), size(&r.part), "{}", r.id);
                (r.part.clone(), r.x, r.y)
            })
            .collect();
        assert_eq!(fg, l.foregrounds);
        let bars: Vec<(f32, f32, f32, f32)> = rows
            .iter()
            .filter(|r| r.kind == "focus")
            .map(|r| (r.x, r.y, r.w, r.h))
            .collect();
        let want: Vec<_> = (0..4)
            .map(|r| (86.0, 50.0 + 115.0 * r as f32, 4.0, 115.0))
            .collect();
        assert_eq!(bars, want);
        // the drum machines' origins are their backgrounds'
        let bg = |id: &str| rows.iter().find(|r| r.id == id).map(|r| (r.x, r.y));
        assert_eq!(bg("tr808.bg"), Some(TR808_ORIGIN));
        assert_eq!(bg("tr909.bg"), Some(TR909_ORIGIN));
    }

    /// The rows' focus areas: x 16..526, y 50 + 115 r (`sub_42ADBB`).
    #[test]
    fn the_rows_focus_areas() {
        let skin = crate::skin::svg::modern();
        let l = skin.layout();
        let at = |x: f32, y: f32| l.row_at(egui::pos2(x, y));
        assert_eq!(at(100.0, 60.0), Some(0));
        assert_eq!(at(100.0, 164.9), Some(0));
        assert_eq!(at(100.0, 165.0), Some(1));
        assert_eq!(at(100.0, 509.0), Some(3));
        assert_eq!(at(100.0, 510.0), None);
        assert_eq!(at(10.0, 60.0), None);
        assert_eq!(at(525.9, 60.0), Some(0));
        assert_eq!(at(526.0, 60.0), None);
        assert_eq!(at(100.0, 20.0), None);
        assert_eq!(l.size, egui::vec2(PANEL_W, PANEL_H));
    }

    /// The logo: the hit rectangle of `sub_42A799` (`sub_479B06(458, 1, 608, 49)` in transport
    /// coordinates, clipped to the transport: panel x 474..607, y 1..48; docs/ui_spec.md
    /// section 3), which the logo part covers exactly.
    #[test]
    fn the_logo() {
        let skin = crate::skin::svg::modern();
        let l = skin.layout();
        let on_logo = |x: f32, y: f32| l.on_logo(egui::pos2(x, y));
        assert!(on_logo(474.0, 1.0) && on_logo(607.9, 48.9));
        assert!(!on_logo(473.9, 10.0) && !on_logo(608.0, 10.0));
        assert!(!on_logo(500.0, 0.9) && !on_logo(500.0, 49.0));
        use orb_ui_art::parts::{LOGO_H, LOGO_W, LOGO_X, LOGO_Y};
        assert_eq!((LOGO_X, LOGO_Y), (474.0, 1.0));
        assert!(on_logo(LOGO_X, LOGO_Y) && on_logo(LOGO_X + LOGO_W - 0.1, LOGO_Y + LOGO_H - 0.1));
        assert!(!on_logo(LOGO_X + LOGO_W, LOGO_Y) && !on_logo(LOGO_X, LOGO_Y + LOGO_H));
    }

    /// The selection marks' places are the art generator's.
    #[test]
    fn the_selection_marks_match_the_art() {
        use orb_ui_art::parts as art;
        let rows = rows();
        let mark = |id: &str| {
            rows.iter()
                .find(|r| r.id == id && r.kind == "mark")
                .unwrap()
        };
        let (o8, o9) = (TR808_ORIGIN, TR909_ORIGIN);
        let close = |a: f32, b: f32| (a - b).abs() < 1e-3;
        for (k, name) in art::NAMES_808.into_iter().enumerate() {
            let m = mark(&format!("tr808.mark_{name}"));
            let (x, y, w, h) = art::LEGEND_808;
            assert!(close(m.x, o8.0 + x + 22.0 * k as f32) && close(m.y, o8.1 + y));
            assert_eq!(
                (m.w, m.h, m.part.as_str()),
                (w, h, format!("sel808_{name}").as_str())
            );
        }
        for (k, name) in art::UPPER_808.into_iter().enumerate() {
            let m = mark(&format!("tr808.mark_{name}"));
            let (x, y, _, _) = art::UPPER_CHIP_808;
            assert!(close(m.x, o8.0 + x + 22.0 * k as f32) && close(m.y, o8.1 + y));
            assert_eq!(m.param("upper"), Some("1"));
        }
        for (i, name) in art::NAMES_909.into_iter().enumerate() {
            let (x, w, _) = art::chip_909(i);
            let m = mark(&format!("tr909.mark_{name}"));
            assert!(close(m.x, o9.0 + x) && close(m.w, w), "chip {i}");
            let sel = crate::art::part(&format!("sel909_{name}")).unwrap();
            assert!((sel.w - w).abs() < 1e-3, "sel909 {i}");
            let (cx, cy) = art::led_909(i);
            let m = mark(&format!("tr909.led_{name}"));
            assert!(
                close(m.x + 2.5, o9.0 + cx) && close(m.y + 2.5, o9.1 + cy),
                "led {i}"
            );
        }
        // the mod window's picture and the gain-reduction LEDs' colors
        let mv = rows.iter().find(|r| r.kind == "modview").unwrap();
        let (px, py, side) = art::MOD_PICTURE;
        assert_eq!(mv.params.nums("picture"), Some(vec![px, py, side, side]));
        for (i, color) in art::GR_COLORS.into_iter().enumerate() {
            let c = rows
                .iter()
                .find(|r| r.id == format!("comp.gr{}", i + 1))
                .unwrap();
            assert_eq!(c.param("off"), Some(format!("gr_led_off_{color}").as_str()));
        }
    }
}
