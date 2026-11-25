//! The smallest main window (task-297): every window the user cannot resize fits in it with the
//! dialogs' own margin at the sides, measured at the minimum and at a large size.

use eframe::egui::{self, Event, Modifiers, Pos2, RawInput, Rect, Vec2};
use orb_ui::app::App;
use orb_ui::browser::Mode;
use orb_ui::layout::{DIALOG_MARGIN, MIN_WINDOW_H, MIN_WINDOW_W};
use orb_ui::model::Panel;

struct H {
    ctx: egui::Context,
    app: App,
    time: f64,
    size: Vec2,
}

impl H {
    fn new(size: Vec2) -> Self {
        let ctx = egui::Context::default();
        egui_extras::install_image_loaders(&ctx);
        let mut app = App::new(Panel::new(), None);
        app.menu = true;
        let mut h = Self {
            ctx,
            app,
            time: 0.0,
            size,
        };
        h.settle();
        h
    }

    fn frame(&mut self) {
        self.time += 0.016;
        let input = RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, self.size)),
            time: Some(self.time),
            events: vec![Event::ModifiersChanged(Modifiers::NONE)],
            ..Default::default()
        };
        let app = &mut self.app;
        let mut out = self.ctx.run_ui(input, |ui| app.show(ui));
        out.textures_delta.clear();
    }

    fn settle(&mut self) {
        for _ in 0..8 {
            self.frame();
        }
    }
}

/// The rectangles of the windows that cannot be resized, in a window of `size`: the four file
/// dialogs, About, Audio Output and the error box, by name.
fn windows(size: Vec2) -> Vec<(&'static str, Rect)> {
    let mut v = Vec::new();
    for (name, mode) in [
        ("open song", Mode::OpenSong),
        ("save song", Mode::SaveSong),
        ("choose mod", Mode::ChooseMod),
        ("export", Mode::Export),
    ] {
        let mut h = H::new(size);
        h.app.open_browser(mode);
        h.settle();
        v.push((
            name,
            h.app.browser_hit("dialog").expect("the dialog is drawn"),
        ));
    }
    let mut h = H::new(size);
    h.app.show_about();
    h.settle();
    let id = h.app.about_id();
    v.push(("about", h.ctx.memory(|m| m.area_rect(id)).unwrap()));
    let mut h = H::new(size);
    h.app.show_audio();
    h.settle();
    let id = h.app.audio_id();
    v.push(("audio output", h.ctx.memory(|m| m.area_rect(id)).unwrap()));
    let mut h = H::new(size);
    // a long message of many lines, and one more waiting
    let long = "a long error line with a_path_that_has_no_break_".repeat(8);
    h.app.fail(vec![long; 12].join("\n"));
    h.app.fail("another error");
    h.settle();
    v.push(("error box", h.app.error_box_rect().unwrap()));
    v
}

#[test]
fn every_window_that_cannot_be_resized_fits_the_smallest_main_window() {
    let min = Vec2::new(MIN_WINDOW_W, MIN_WINDOW_H);
    let screen = Rect::from_min_size(Pos2::ZERO, min);
    // half a point of rounding in the frames' strokes
    let inside = screen.shrink(DIALOG_MARGIN - 0.5);
    for (name, r) in windows(min) {
        eprintln!("{name}: {:?} at {:?}", r.size(), r.min);
        assert!(inside.contains_rect(r), "{name} {r:?} in {screen:?}");
    }
}

/// The minimum is not larger than it must be: the widest window and the highest, each with its
/// margin, give it (within a point).
#[test]
fn the_minimum_is_the_widest_and_highest_window_plus_the_margins() {
    let big = windows(Vec2::new(1400.0, 1000.0));
    let small = windows(Vec2::new(MIN_WINDOW_W, MIN_WINDOW_H));
    let widest = small.iter().map(|(_, r)| r.width()).fold(0.0, f32::max);
    let highest = small.iter().map(|(_, r)| r.height()).fold(0.0, f32::max);
    eprintln!("widest {widest} highest {highest}");
    assert!(
        widest + 2.0 * DIALOG_MARGIN >= MIN_WINDOW_W - 1.0,
        "{widest}"
    );
    assert!(
        highest + 2.0 * DIALOG_MARGIN >= MIN_WINDOW_H - 1.0,
        "{highest}"
    );
    // in a large window no window is smaller than at the minimum (the file dialogs grow)
    for ((name, b), (_, s)) in big.iter().zip(&small) {
        assert!(
            b.width() >= s.width() - 0.5 && b.height() >= s.height() - 0.5,
            "{name}"
        );
    }
}

/// Windows that grow with their content stay inside a window as small as the minimum and as
/// short as the one of a plugin host that gives less than the minimum height: the audio window
/// scrolls what does not fit.
#[test]
fn the_audio_window_is_bounded_by_the_window() {
    for size in [
        Vec2::new(MIN_WINDOW_W, MIN_WINDOW_H),
        Vec2::new(MIN_WINDOW_W, 200.0),
    ] {
        let mut h = H::new(size);
        h.app.show_audio();
        h.settle();
        let id = h.app.audio_id();
        let r = h.ctx.memory(|m| m.area_rect(id)).unwrap();
        let screen = Rect::from_min_size(Pos2::ZERO, size).shrink(DIALOG_MARGIN - 0.5);
        assert!(screen.contains_rect(r), "{size:?} {r:?}");
    }
}
