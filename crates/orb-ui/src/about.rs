//! Help > About (task-102, laid out in task-115): the icon, the wordmark, the version and the
//! project link in a small centered window. The version string is computed by `build.rs`
//! (`crate::version`).

use egui::{self, Context, Key, Modifiers, RichText};

use crate::a11y::window as win;
use crate::app::{self, App};
use crate::logo_anim::{self, LOGO_H, LOGO_W};
use crate::palette::INK_DIM;

/// The wordmark part of the panel (`logo_anim::LOGO_PART`) is shown at this scale.
const LOGO_SCALE: f32 = 2.0;
/// The app icon part and the size it is shown at.
pub const ICON_PART: &str = "app_icon";
const ICON_SIZE: f32 = 64.0;
/// The content area below the title bar and the x of the version and the link. The mockup's
/// window is 360 wide; the link lines need about 90 more at this alignment.
/// Widened from 380 to 450 (task-138): "Song collection: <link>" ends at about 434 pt (x 108 + 326)
/// and with 380 the line grew the window or touched its edge.
const CONTENT_W: f32 = 450.0;
/// The width of the label column of the collection lines (the links start at TEXT_X + this).
const LABEL_W: f32 = 100.0;
/// 156 with one link line (task-115), plus two lines at the same 22 pt spacing (task-138),
/// plus 6 since the text rows moved down to clear the wordmark's plate (task-179).
const CONTENT_H: f32 = 206.0;
const TEXT_X: f32 = 108.0;
/// The gray plate under the wordmark (task-179): the part and its place in the content (the
/// logo canvas at 2x, 4 less below); the icon is centered on it.
const PLATE_PART: &str = "about_plate";
const PLATE: (f32, f32, f32, f32) = (92.0, 14.0, 268.0, 92.0);
const ICON_Y: f32 = 28.0;
/// The first text row's top: 6 below task-138's, so the version line clears the plate.
const ROW0: f32 = 110.0;

/// The name of the app.
pub const NAME: &str = "OpenRB-338";
/// The project page.
pub const URL: &str = "https://pingw33n.github.io/OpenRB-338/";
/// The user's public collections, shown under the project page as `label url` (task-138).
/// The user's song collection.
pub const SONGS_URL: &str = "https://github.com/pingw33n/rb-songs";
/// The user's mod collection.
pub const MODS_URL: &str = "https://github.com/pingw33n/rb-mods";
/// The label and the link of each collection line.
pub const COLLECTIONS: [(&str, &str); 2] = [
    ("Song collection:", SONGS_URL),
    ("Mod collection:", MODS_URL),
];
/// The version: `YYYY.MMDD.HHMM (<[dev-]hash[-dirty]>)` (`src/version.rs`).
pub const VERSION: &str = env!("ORB_UI_VERSION");

/// The side of the copy button, a text line's height.
const COPY_SIZE: f32 = 14.0;
/// How long the check mark shows after a copy.
const COPIED_FOR: f64 = 1.5;

fn copy_data_id() -> egui::Id {
    egui::Id::unique("orb_about_copy_version")
}

/// Where the copy button was drawn last frame (for tests to click it).
pub fn copy_button_rect(ctx: &Context) -> Option<egui::Rect> {
    ctx.data(|d| d.get_temp::<egui::Rect>(copy_data_id()))
}

/// Seconds since the last copy while its confirmation (the check mark, "Copied") still shows.
fn copied_since(ctx: &Context, now: f64) -> Option<f64> {
    let t = ctx.data(|d| d.get_temp::<f64>(copy_data_id().with("at")))?;
    Some(now - t).filter(|s| (0.0..COPIED_FOR).contains(s))
}

/// Whether the copy button shows its confirmation now (for tests).
pub fn copy_confirmation_shows(ctx: &Context) -> bool {
    copied_since(ctx, ctx.input(|i| i.time)).is_some()
}

/// The icon-only copy button (task-279): two overlapping sheets, a check mark for 1.5 s after a
/// click, a "Copy version" tooltip. A click copies `VERSION` through `ctx.copy_text`, which
/// eframe writes to the system clipboard.
fn copy_button(ui: &mut egui::Ui) {
    let (rect, resp) =
        ui.allocate_exact_size(egui::vec2(COPY_SIZE, COPY_SIZE), egui::Sense::click());
    let ctx = ui.ctx().clone();
    ctx.data_mut(|d| d.insert_temp(copy_data_id(), rect));
    let now = ctx.input(|i| i.time);
    let at_id = copy_data_id().with("at");
    if resp.clicked() {
        ctx.copy_text(VERSION.to_string());
        ctx.data_mut(|d| d.insert_temp(at_id, now));
    }
    let since = copied_since(&ctx, now);
    let copied = since.is_some();
    if let Some(s) = since {
        ctx.request_repaint_after(std::time::Duration::from_secs_f64(COPIED_FOR - s));
    }
    let color = if resp.hovered() || copied {
        egui::Color32::WHITE
    } else {
        INK_DIM
    };
    let p = ui.painter();
    let stroke = egui::Stroke::new(1.2, color);
    if copied {
        let c = rect.center();
        p.add(egui::Shape::line(
            vec![
                c + egui::vec2(-4.5, 0.5),
                c + egui::vec2(-1.5, 3.5),
                c + egui::vec2(4.5, -3.5),
            ],
            egui::Stroke::new(1.8, color),
        ));
    } else {
        // the back sheet up left, the front one down right
        let s = 8.5;
        let back = egui::Rect::from_min_size(rect.min + egui::vec2(1.5, 1.5), egui::vec2(s, s));
        let front = back.translate(egui::vec2(3.0, 3.0));
        p.rect_stroke(back, 1.0, stroke, egui::StrokeKind::Inside);
        p.rect_filled(front, 1.0, ui.visuals().window_fill);
        p.rect_stroke(front, 1.0, stroke, egui::StrokeKind::Inside);
    }
    // a screen reader names the icon-only button (the fork's m-7; no change to what is drawn)
    let name = if copied { "Copied" } else { "Copy version" };
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, name));
    resp.on_hover_text(name);
}

/// The id of the About window of an open (`App::about_serial`).
pub fn about_window_id(serial: u32) -> egui::Id {
    egui::Id::unique(("orb_about_window", serial))
}

impl App {
    /// Help > About.
    pub fn show_about(&mut self) {
        if !self.about_open {
            // a new id: no stored place, so the window opens at its default, the center
            self.about_serial += 1;
            // the logo animation starts with the window's first frame (`about_ui`)
            self.about_start_due = true;
        } else {
            self.raise_window(self.about_id());
        }
        self.about_open = true;
    }

    /// The id of the About window now (for tests, to look its place up).
    pub fn about_id(&self) -> egui::Id {
        about_window_id(self.about_serial)
    }

    /// Whether the About window is open.
    pub fn about_is_open(&self) -> bool {
        self.about_open
    }

    /// Draws the About window when it is open. Escape and the close button close it.
    pub fn about_ui(&mut self, ctx: &Context) {
        if !self.about_open {
            return;
        }
        if ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Escape)) {
            self.about_open = false;
            return;
        }
        let due = self.window_focus_due(crate::app::Win::About);
        let now = ctx.input(|i| i.time);
        if std::mem::take(&mut self.about_start_due) {
            self.about_start = Some(now);
        }
        let t = logo_anim::time_now(now, self.about_start.filter(|_| self.logo_anim));
        let mut open = true;
        let sr = self.screen_reader_mode();
        let id = self.about_id();
        // centered the way File > Info is (info.rs): pivot, default place, an id per open
        egui::Window::new(format!("About {NAME}"))
            .id(id)
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .pivot(egui::Align2::CENTER_CENTER)
            .default_pos(ctx.content_rect().center())
            // the default frame, like every other window (task-235): a zero inner margin
            // made the title bar flat and tight against the edge
            .show(ctx, |ui| {
                crate::app::focus_into_window(ui, due);
                win::window(ui, sr, id, &format!("About {NAME}"));
                // The layout of the mockup session (task-115), points from the top left of
                // the content: the icon, the wordmark at 2x, and under its subtitle the
                // version and the link, left-aligned. The images are SVG, so the loader
                // rasterizes them at the window's pixel density.
                let (rect, _) =
                    ui.allocate_exact_size(egui::vec2(CONTENT_W, CONTENT_H), egui::Sense::hover());
                let at = |x: f32, y: f32| rect.min + egui::vec2(x, y);
                // the icon fades in with the logo's animation (task-114)
                let icon_a = t.map_or(1.0, |t| logo_anim::Frame::at(t).icon);
                app::paint_part_tinted(
                    ui,
                    ICON_PART,
                    egui::Rect::from_min_size(at(18.0, ICON_Y), egui::vec2(ICON_SIZE, ICON_SIZE)),
                    egui::Color32::WHITE.gamma_multiply(icon_a),
                );
                // only the wordmark sits on the panel's gray (task-179); the rest of the
                // window keeps its own colors
                let (px, py, pw, ph) = PLATE;
                app::paint_part_tinted(
                    ui,
                    PLATE_PART,
                    egui::Rect::from_min_size(at(px, py), egui::vec2(pw, ph)),
                    egui::Color32::WHITE,
                );
                let logo_size = egui::vec2(LOGO_W, LOGO_H) * LOGO_SCALE;
                logo_anim::paint_logo(ui, egui::Rect::from_min_size(at(92.0, 14.0), logo_size), t);
                let line_at = |ui: &mut egui::Ui, x: f32, y: f32, f: &dyn Fn(&mut egui::Ui)| {
                    let r = egui::Rect::from_min_size(at(x, y), egui::vec2(CONTENT_W - x, 18.0));
                    ui.scope_builder(egui::UiBuilder::new().max_rect(r), f);
                };
                let line = |ui: &mut egui::Ui, y: f32, f: &dyn Fn(&mut egui::Ui)| {
                    line_at(ui, TEXT_X, y, f);
                };
                line(ui, ROW0, &|ui| {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 4.0;
                        let version = RichText::new(format!("Version {VERSION}")).color(INK_DIM);
                        win::line(ui, sr, version, false, None);
                        copy_button(ui);
                    });
                });
                line(ui, ROW0 + 22.0, &|ui| {
                    ui.hyperlink_to(URL, URL);
                });
                for (i, (label, url)) in COLLECTIONS.into_iter().enumerate() {
                    // the label and the link each in a rect of their own, so the links start at
                    // exactly the same x whatever the width of the label
                    let y = ROW0 + 44.0 + 22.0 * i as f32;
                    line(ui, y, &|ui| {
                        win::line(ui, sr, RichText::new(label).color(INK_DIM), false, None);
                    });
                    line_at(ui, TEXT_X + LABEL_W, y, &|ui| {
                        ui.hyperlink_to(url, url);
                    });
                }
            });
        if !open {
            self.about_open = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::art;
    use crate::logo_anim::LOGO_PART;

    /// The About window shows the panel's own wordmark part, at its aspect ratio.
    #[test]
    fn the_logo_part_is_the_panels_wordmark() {
        let part = art::part(LOGO_PART).expect("the logo part");
        let svg = std::str::from_utf8(part.svg).unwrap();
        let want =
            format!("width=\"{LOGO_W}\" height=\"{LOGO_H}\" viewBox=\"0 0 {LOGO_W} {LOGO_H}\"");
        assert!(svg.contains(&want), "{want} in the part's header");
        // the panel draws the same part over its transport background
        let skin = crate::skin::svg::modern();
        let logo = crate::skin::Skin::layout(&*skin).logo.clone();
        assert_eq!(logo.map(|l| l.part), Some(LOGO_PART.to_string()));
    }

    /// The wordmark's plate (task-179) is the art generator's, under the logo at 2x, and the
    /// text rows start below it.
    #[test]
    fn the_plate_is_under_the_wordmark() {
        let (x, y, w, h) = PLATE;
        assert_eq!(PLATE, orb_ui_art::parts::ABOUT_PLATE);
        let p = art::part(PLATE_PART).expect("the plate part");
        assert_eq!((p.w, p.h), (w, h));
        assert_eq!((x, y), (92.0, 14.0));
        assert_eq!(w, LOGO_W * LOGO_SCALE);
        assert!(ROW0 > y + h, "the version line clears the plate");
        assert_eq!(ICON_Y + ICON_SIZE / 2.0, y + h / 2.0);
    }
}
