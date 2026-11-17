//! The plugin's mode knob shows the vertical-resize cursor like the panel's other knobs
//! (task-241), headless: over the knob's drag area, and while a drag that left it goes on, but
//! not over a mode row.

use std::path::Path;
use std::sync::Arc;

use eframe::egui::{self, CursorIcon, Event, Modifiers, PointerButton, Pos2, RawInput, Rect, Vec2};
use orb_engine::tables::Tables;
use orb_ui::app::App;
use orb_ui::embed::{
    self, Embed, PLUGIN_MODE_KNOB, PLUGIN_MODE_LED_X, PLUGIN_MODE_ROWS, PluginMode, SongView,
};
use orb_ui::layout::PANEL_W;
use orb_ui::link::Link;
use orb_ui::settings::Settings;

struct Fake;

impl Embed for Fake {
    fn revision(&self) -> u64 {
        0
    }
    fn view(&mut self) -> SongView {
        SongView {
            song: None,
            kit: None,
            // 32 patterns of 98 bytes (the 303s) or 194 (the drum machines), all zero
            patterns: [
                vec![0; 32 * 98],
                vec![0; 32 * 98],
                vec![0; 32 * 194],
                vec![0; 32 * 194],
            ],
            snapshot: Default::default(),
        }
    }
    fn load_song(&mut self, _: Option<Vec<u8>>) -> Result<(), String> {
        Ok(())
    }
    fn set_mod(&mut self, _: Option<&Path>) -> Result<(), String> {
        Ok(())
    }
    fn mode(&self) -> Option<PluginMode> {
        Some(PluginMode::Gate)
    }
}

struct Harness {
    ctx: egui::Context,
    app: App,
    time: f64,
    win: Rect,
    _audio: orb_player::Audio,
}

impl Harness {
    fn new() -> Self {
        let ctx = egui::Context::default();
        egui_extras::install_image_loaders(&ctx);
        let (link, audio) = Link::new(Arc::new(Tables::new()), None);
        let app = App::embedded(Box::new(Fake), link, Settings::default());
        // scale 1: the panel fits the window less the margins exactly
        let w = PANEL_W + 2.0 * embed::MARGIN_X;
        let win = Rect::from_min_size(Pos2::ZERO, Vec2::new(w, embed::window_height(w)));
        let mut h = Self {
            ctx,
            app,
            time: 0.0,
            win,
            _audio: audio,
        };
        for _ in 0..3 {
            h.frame(vec![]);
        }
        h
    }

    /// The window point of a panel point.
    fn at(&self, x: f32, y: f32) -> Pos2 {
        embed::panel_area(self.win).min + Vec2::new(x, y)
    }

    /// A frame; the cursor it asked for.
    fn frame(&mut self, events: Vec<Event>) -> CursorIcon {
        self.time += 0.016;
        let input = RawInput {
            screen_rect: Some(self.win),
            time: Some(self.time),
            events,
            ..Default::default()
        };
        let app = &mut self.app;
        let mut out = self.ctx.run_ui(input, |ui| app.show(ui));
        out.textures_delta.clear();
        out.platform_output.cursor_icon
    }

    fn button(p: Pos2, pressed: bool) -> Event {
        Event::PointerButton {
            pos: p,
            button: PointerButton::Primary,
            pressed,
            modifiers: Modifiers::NONE,
        }
    }

    fn hover(&mut self, p: Pos2) -> CursorIcon {
        self.frame(vec![Event::PointerMoved(p)]);
        self.frame(vec![])
    }
}

#[test]
fn the_mode_knob_shows_the_vertical_resize_cursor_and_the_rows_do_not() {
    let mut h = Harness::new();
    let (cx, cy) = PLUGIN_MODE_KNOB;
    let knob = h.at(cx, cy);
    let row = h.at(PLUGIN_MODE_LED_X + 30.0, PLUGIN_MODE_ROWS[0]);
    let away = h.at(40.0, 300.0);
    assert_ne!(h.hover(away), CursorIcon::ResizeVertical);
    assert_eq!(h.hover(knob), CursorIcon::ResizeVertical);
    assert_ne!(h.hover(row), CursorIcon::ResizeVertical);
    // a drag that leaves the knob keeps the cursor until the button is up
    h.hover(knob);
    h.frame(vec![Harness::button(knob, true)]);
    let far = knob + Vec2::new(0.0, 40.0);
    h.frame(vec![Event::PointerMoved(far)]);
    assert_eq!(
        h.frame(vec![Event::PointerMoved(far)]),
        CursorIcon::ResizeVertical
    );
    h.frame(vec![Harness::button(far, false)]);
    assert_ne!(h.hover(away), CursorIcon::ResizeVertical);
}
