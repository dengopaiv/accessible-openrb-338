//! The plugin's editor panel (task-201) in a window of its own, for looking at it and for
//! screenshots without a host:
//!
//! ```text
//! cargo run --release -p orb-plugin --example editor_preview -- [SONG.rbs] [--size WxH]
//!     [--ppp S] [--host-playing] [--midi] [--screenshot OUT.png]
//! ```
//!
//! A stand-in host runs the plugin's process side in step with the frames (no sound): the
//! song loaded as File > Open loads it (the mode at Play, pattern mode), `--host-playing`
//! plays the host's transport from bar 1 (the Sync LED), `--midi` sends a host message on
//! channel 10 (which nothing maps) in every buffer (the MIDI LED stays lit). The parameters the
//! panel sets are applied at once, as a host applies them. `--screenshot` saves the window after a
//! few frames and quits. The window is the editor's (`--size`, default 624x532 points: the menu bar
//! and the panel at 1:1; a plugin window keeps height = `orb_ui::embed::window_height(width)`).

use std::path::PathBuf;
use std::sync::Arc;

use eframe::egui;
use nice_plug::context::gui::GuiContext;
use nice_plug::prelude::*;
use orb_player::midi::Message;
use orb_plugin::core::{Core, HostEvent, HostTransport};
use orb_plugin::editor::editor_app;
use orb_plugin::instance::{self, OrbParams};
use orb_ui::app::App;
use orb_ui::settings::Settings;

/// A host that applies a parameter change at once.
struct Host;

impl GuiContextInner for Host {
    fn plugin_api(&self) -> PluginApi {
        PluginApi::Clap
    }

    unsafe fn raw_begin_set_parameter(&self, _param: ParamPtr) {}

    unsafe fn raw_set_parameter_normalized(&self, param: ParamPtr, normalized: f32) {
        unsafe { param._internal_set_normalized_value(normalized) };
    }

    unsafe fn raw_end_set_parameter(&self, _param: ParamPtr) {}

    fn get_state(&self) -> PluginState {
        unimplemented!("not in the preview")
    }

    fn set_state(&self, _state: PluginState) {}

    fn request_restart(&self) {}
}

struct Args {
    song: Option<PathBuf>,
    size: [f32; 2],
    ppp: Option<f32>,
    host_playing: bool,
    midi: bool,
    shot: Option<PathBuf>,
}

fn args() -> Args {
    let mut a = Args {
        song: None,
        size: [624.0, orb_ui::embed::window_height(624.0)],
        ppp: None,
        host_playing: false,
        midi: false,
        shot: None,
    };
    let mut it = std::env::args().skip(1);
    while let Some(s) = it.next() {
        match s.as_str() {
            "--size" => {
                let v = it.next().expect("--size WxH");
                let (w, h) = v.split_once('x').expect("--size WxH");
                a.size = [w.parse().unwrap(), h.parse().unwrap()];
            }
            "--ppp" => a.ppp = Some(it.next().and_then(|v| v.parse().ok()).expect("--ppp S")),
            "--host-playing" => a.host_playing = true,
            "--midi" => a.midi = true,
            "--screenshot" => a.shot = Some(it.next().expect("--screenshot OUT.png").into()),
            s if !s.starts_with('-') => a.song = Some(s.into()),
            s => panic!("unknown option {s}"),
        }
    }
    a
}

struct Preview {
    params: Arc<OrbParams>,
    core: Core,
    app: App,
    /// Host frames run so far.
    frames: u64,
    host_playing: bool,
    midi: bool,
    shot: Option<PathBuf>,
    shown: u32,
}

impl Preview {
    /// Runs the host frames the time since the last frame asks for, in 512-frame buffers.
    fn host(&mut self, now: f64) {
        const RATE: f64 = 44100.0;
        const BUF: usize = 512;
        let due = (now * RATE) as u64;
        let (mut l, mut r) = (vec![0f32; BUF], vec![0f32; BUF]);
        while self.frames + BUF as u64 <= due {
            let beats = self.frames as f64 / RATE * 120.0 / 60.0;
            let t = HostTransport {
                playing: self.host_playing,
                tempo: Some(120.0),
                pos_beats: Some(if self.host_playing { beats } else { 0.0 }),
            };
            let events: Vec<HostEvent> = self
                .midi
                .then_some(HostEvent {
                    frame: 0,
                    message: Message::NoteOn {
                        channel: 9,
                        note: 60,
                        velocity: 100,
                    },
                })
                .into_iter()
                .collect();
            self.core
                .process_with(&mut l, &mut r, &self.params.values(), &t, &events);
            self.frames += BUF as u64;
        }
    }
}

impl eframe::App for Preview {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui_extras::install_image_loaders(ui.ctx());
        let now = ui.input(|i| i.time);
        self.host(now);
        self.app.show(ui);
        ui.ctx().request_repaint();
        self.shown += 1;
        if self.shown == 30
            && let Some(path) = self.shot.take()
        {
            ui.ctx()
                .request_screenshot(Box::new(move |image: Arc<egui::ColorImage>| {
                    orb_ui::app::save_png(&path, &image).expect("writing the screenshot");
                    std::process::exit(0);
                }));
        }
    }
}

fn main() -> eframe::Result {
    let a = args();
    let params = Arc::new(OrbParams::new());
    let host = GuiContext::new(Arc::new(Host));
    let core = instance::core(&params, 44100);
    let mut app = editor_app(params.clone(), host, Settings::load());
    if let Some(song) = &a.song {
        let bytes = std::fs::read(song).expect("reading the song");
        app.embed()
            .expect("a plugin's editor")
            .load_song(Some(bytes))
            .expect("loading the song");
    }
    let viewport = egui::ViewportBuilder::default()
        .with_title("OpenRB-338 plugin editor (preview)")
        .with_inner_size(a.size);
    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };
    let ppp = a.ppp;
    eframe::run_native(
        "orb-plugin editor preview",
        options,
        Box::new(move |cc| {
            cc.egui_ctx.set_theme(egui::ThemePreference::Dark);
            if let Some(ppp) = ppp {
                cc.egui_ctx.set_pixels_per_point(ppp);
            }
            Ok(Box::new(Preview {
                params,
                core,
                app,
                frames: 0,
                host_playing: a.host_playing,
                midi: a.midi,
                shot: a.shot,
                shown: 0,
            }))
        }),
    )
}
