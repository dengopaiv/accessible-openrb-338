//! Choosing the command of a window that is already open brings it to the front (task-283,
//! docs/ui_spec.md section 1): every window of the app, with another window opened over it.

use std::sync::{Arc, OnceLock};

use eframe::egui::{self, Event, Modifiers, Pos2, RawInput, Rect, Vec2};
use orb_engine::tables::Tables;
use orb_player::{Audio, CALL_FRAMES};
use orb_ui::app::App;
use orb_ui::drum_editor::Machine;
use orb_ui::layout::{PANEL_H, PANEL_W};
use orb_ui::link::Link;
use orb_ui::menu::{Cmd, MENU_H};
use orb_ui::model::Panel;

fn tables() -> Arc<Tables> {
    static T: OnceLock<Arc<Tables>> = OnceLock::new();
    T.get_or_init(|| Arc::new(Tables::new())).clone()
}

struct Harness {
    ctx: egui::Context,
    app: App,
    audio: Audio,
    time: f64,
}

impl Harness {
    fn new() -> Self {
        let ctx = egui::Context::default();
        egui_extras::install_image_loaders(&ctx);
        let samples = rbm::standard::builtin_samples();
        let (link, audio) = Link::new(tables(), Some(samples.clone()));
        let mut app = App::with_link(Panel::new(), link, None);
        app.samples = Some(samples);
        app.menu = true;
        let mut h = Self {
            ctx,
            app,
            audio,
            time: 0.0,
        };
        let mut out = vec![0i16; 2 * CALL_FRAMES];
        h.audio.render_i16(&mut out);
        h.frame();
        h
    }

    fn frame(&mut self) {
        self.time += 0.016;
        let input = RawInput {
            screen_rect: Some(Rect::from_min_size(
                Pos2::ZERO,
                Vec2::new(PANEL_W, PANEL_H + MENU_H),
            )),
            time: Some(self.time),
            events: vec![Event::ModifiersChanged(Modifiers::NONE)],
            ..Default::default()
        };
        let app = &mut self.app;
        let mut out = self.ctx.run_ui(input, |ui| app.show(ui));
        out.textures_delta.clear();
    }

    fn frames(&mut self) {
        for _ in 0..4 {
            self.frame();
        }
    }

    /// The window on top of the Middle order.
    fn top(&self) -> Option<egui::Id> {
        self.ctx
            .memory(|m| m.areas().top_layer_id(egui::Order::Middle))
            .map(|l| l.id)
    }

    fn rect(&self, id: egui::Id) -> Rect {
        egui::AreaState::load(&self.ctx, id).unwrap().rect()
    }
}

/// A window of the app: the command that opens it and its id now.
struct Win {
    name: &'static str,
    open: fn(&mut App),
    id: fn(&App) -> egui::Id,
}

fn windows() -> Vec<Win> {
    vec![
        Win {
            name: "Song Information",
            open: |a| a.command(Cmd::Info),
            id: |a| a.info_id(),
        },
        Win {
            name: "Mod Information",
            open: |a| a.command(Cmd::ModInfo),
            id: |a| a.mod_info_id(),
        },
        Win {
            name: "About",
            open: |a| a.command(Cmd::About),
            id: |a| a.about_id(),
        },
        Win {
            name: "Keyboard Shortcuts",
            open: |a| a.command(Cmd::KeyHelp),
            id: |a| a.key_help_id(),
        },
        // the plugin's window, which only a plugin's menu has
        Win {
            name: "Plugin Usage",
            open: |a| a.show_plugin_usage(),
            id: |a| a.plugin_usage_id(),
        },
        Win {
            name: "303-1 Pattern Editor",
            open: |a| a.command(Cmd::PatternEditor(0)),
            id: |_| egui::Id::unique(orb_ui::pattern_editor::salt(0)),
        },
        Win {
            name: "303-2 Pattern Editor",
            open: |a| a.command(Cmd::PatternEditor(1)),
            id: |_| egui::Id::unique(orb_ui::pattern_editor::salt(1)),
        },
        Win {
            name: "808 Pattern Editor",
            open: |a| a.command(Cmd::DrumEditor(Machine::Tr808)),
            id: |_| egui::Id::unique(Machine::Tr808.salt()),
        },
        Win {
            name: "909 Pattern Editor",
            open: |a| a.command(Cmd::DrumEditor(Machine::Tr909)),
            id: |_| egui::Id::unique(Machine::Tr909.salt()),
        },
    ]
}

#[test]
fn the_command_of_an_open_window_brings_it_to_the_front_unmoved() {
    let all = windows();
    for a in &all {
        for b in all.iter().filter(|b| b.name != a.name) {
            let mut h = Harness::new();
            (a.open)(&mut h.app);
            h.frames();
            (b.open)(&mut h.app);
            h.frames();
            let (ia, ib) = ((a.id)(&h.app), (b.id)(&h.app));
            assert_eq!(h.top(), Some(ib), "{} opened over {}", b.name, a.name);
            let before = h.rect(ia);
            (a.open)(&mut h.app);
            h.frames();
            assert_eq!(h.top(), Some(ia), "{} not in front over {}", a.name, b.name);
            assert_eq!(h.rect(ia), before, "{} moved or resized", a.name);
        }
    }
}
