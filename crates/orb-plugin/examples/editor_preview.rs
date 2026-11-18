//! The plugin's editor panel (task-201) in a window of its own, for looking at it and for
//! screenshots without a host:
//!
//! ```text
//! cargo run --release -p orb-plugin --example editor_preview -- [SONG.rbs] [--size WxH]
//!     [--ppp S] [--host-playing] [--midi] [--note CH,NOTE,VEL]... [--mode latch|gate|note]
//!     [--preview] [--format vst3|clap] [--idle] [--click X,Y]... [--scroll Y]
//!     [--screenshot OUT.png]
//! ```
//!
//! A stand-in host runs the plugin's process side in step with the frames (no sound): the
//! song loaded as File > Open loads it (the mode is kept), `--host-playing`
//! plays the host's transport from bar 1 (the Sync LED), `--midi` sends a host message on
//! channel 10 (which nothing maps) in every buffer (the MIDI LED stays lit); each `--note
//! CH,NOTE,VEL` sends that note-on (channel 1..4 the devices, 3 the 808, 4 the 909; for the
//! note-mode lights, task-202) in every buffer, so the voice keeps sounding. `--mode` sets the
//! plugin's mode after the load, `--preview` enters the preview (the panel's Play, task-217),
//! and each `--click X,Y` clicks the window at that point (in points; one click every 6 frames
//! from frame 10, e.g. to open a menu for a screenshot). The parameters the
//! panel sets are applied at once, as a host applies them. `--screenshot` saves the window after a
//! few frames and quits. The window is the editor's (`--size`, default 656x532 points,
//! `orb_plugin::editor::DEFAULT_SIZE`: the menu bar and the panel at 1:1 with its desk margins;
//! any size letterboxes the panel, and `orb_ui::embed::window_height(width)` is its exact fit).

use std::path::PathBuf;
use std::sync::Arc;

use eframe::egui;
use nice_plug::context::gui::GuiContext;
use nice_plug::prelude::*;
use orb_player::midi::Message;
use orb_plugin::core::{Core, HostEvent, HostTransport};
use orb_plugin::editor::editor_app;
use orb_plugin::instance::{self, Mode, OrbParams};
use orb_ui::app::App;
use orb_ui::settings::Settings;

/// A host that applies a parameter change at once.
struct Host(PluginApi);

impl GuiContextInner for Host {
    fn plugin_api(&self) -> PluginApi {
        self.0
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
    /// (channel 0..15, note, velocity) of each `--note`.
    notes: Vec<(u8, u8, u8)>,
    mode: Option<Mode>,
    preview: bool,
    /// `--format vst3|clap`: the plugin format the stand-in host reports (the logo; default clap).
    format: PluginApi,
    /// `--idle`: the harness asks for no frames itself (for measuring the panel's own repaints).
    idle: bool,
    clicks: Vec<egui::Pos2>,
    /// `--scroll Y`: scroll the window under the pointer down by Y points, after the clicks.
    scroll: f32,
    shot: Option<PathBuf>,
}

fn args() -> Args {
    let mut a = Args {
        song: None,
        size: [
            orb_plugin::editor::DEFAULT_SIZE.0 as f32,
            orb_plugin::editor::DEFAULT_SIZE.1 as f32,
        ],
        ppp: None,
        host_playing: false,
        midi: false,
        notes: Vec::new(),
        mode: None,
        preview: false,
        format: PluginApi::Clap,
        idle: false,
        clicks: Vec::new(),
        scroll: 0.0,
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
            "--note" => {
                let v = it.next().expect("--note CH,NOTE,VEL");
                let p: Vec<u8> = v.split(',').map(|n| n.parse().expect("a number")).collect();
                let [ch, note, vel] = p[..] else {
                    panic!("--note CH,NOTE,VEL")
                };
                a.notes.push((ch - 1, note, vel));
            }
            "--mode" => {
                a.mode = Some(match it.next().as_deref() {
                    Some("latch") => Mode::Latch,
                    Some("gate") => Mode::Gate,
                    Some("note") => Mode::Note,
                    _ => panic!("--mode latch|gate|note"),
                })
            }
            "--preview" => a.preview = true,
            "--format" => {
                a.format = match it.next().as_deref() {
                    Some("vst3") => PluginApi::Vst3,
                    Some("clap") => PluginApi::Clap,
                    _ => panic!("--format vst3|clap"),
                }
            }
            "--idle" => a.idle = true,
            "--click" => {
                let v = it.next().expect("--click X,Y");
                let (x, y) = v.split_once(',').expect("--click X,Y");
                a.clicks
                    .push(egui::pos2(x.parse().unwrap(), y.parse().unwrap()));
            }
            "--scroll" => a.scroll = it.next().expect("--scroll Y").parse().unwrap(),
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
    notes: Vec<(u8, u8, u8)>,
    idle: bool,
    clicks: Vec<egui::Pos2>,
    scroll: f32,
    shot: Option<PathBuf>,
    shown: u32,
    /// The time (egui's clock) from which the screenshot may be taken.
    ready_at: f64,
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
            let mut events: Vec<HostEvent> = self
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
            events.extend(
                self.notes
                    .iter()
                    .map(|&(channel, note, velocity)| HostEvent {
                        frame: 0,
                        message: Message::NoteOn {
                            channel,
                            note,
                            velocity,
                        },
                    }),
            );
            self.core
                .process_with(&mut l, &mut r, &self.params.values(), &t, &events);
            self.frames += BUF as u64;
        }
    }
}

impl eframe::App for Preview {
    /// The `--click`s: a press at frame `10 + 6k`, the release two frames later.
    fn raw_input_hook(&mut self, ctx: &egui::Context, raw: &mut egui::RawInput) {
        let f = self.shown as usize;
        if f < 10 {
            return;
        }
        // the scroll: the pointer to the middle, then the wheel, 14 frames after the last click
        let scroll_at = 10 + 6 * self.clicks.len() + 14;
        if self.scroll != 0.0 && f == scroll_at {
            raw.events
                .push(egui::Event::PointerMoved(ctx.content_rect().center()));
        } else if self.scroll != 0.0 && f == scroll_at + 2 {
            raw.events.push(egui::Event::MouseWheel {
                unit: egui::MouseWheelUnit::Point,
                delta: egui::vec2(0.0, -self.scroll),
                phase: egui::TouchPhase::Move,
                source: egui::MouseWheelSource::Unknown,
                modifiers: egui::Modifiers::NONE,
            });
        }
        let (k, r) = ((f - 10) / 6, (f - 10) % 6);
        let Some(&pos) = self.clicks.get(k) else {
            return;
        };
        let button = |pressed| egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::NONE,
        };
        match r {
            0 => raw.events.push(egui::Event::PointerMoved(pos)),
            1 => raw.events.push(button(true)),
            3 => raw.events.push(button(false)),
            _ => {}
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui_extras::install_image_loaders(ui.ctx());
        let now = ui.input(|i| i.time);
        self.host(now);
        self.app.show(ui);
        if !self.idle {
            ui.ctx().request_repaint();
        }
        self.shown += 1;
        // 30 frames, or 24 after the last click (a window opened by it has faded in), and 16
        // more after a scroll (it has settled, task-221)
        let shot_at = 30.max(10 + 6 * self.clicks.len() as u32 + 24)
            + if self.scroll != 0.0 { 16 } else { 0 };
        // and 1.5 s of the clock after the last click and scroll, for a machine under load,
        // where frames are slow and animations run by time
        let last = 10 + 6 * self.clicks.len() as u32 + if self.scroll != 0.0 { 16 } else { 0 };
        if self.shown == last {
            self.ready_at = now + 1.5;
        }
        if self.shown >= shot_at
            && now >= self.ready_at
            && self.shot.is_some()
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
    let host = GuiContext::new(Arc::new(Host(a.format)));
    let core = instance::core(&params, 44100);
    let mut app = editor_app(params.clone(), host, Settings::load());
    if let Some(song) = &a.song {
        let bytes = std::fs::read(song).expect("reading the song");
        app.embed()
            .expect("a plugin's editor")
            .load_song(Some(bytes))
            .expect("loading the song");
    }
    if let Some(m) = a.mode {
        params.set_mode(m);
    }
    if a.preview {
        params.set_preview(true);
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
                notes: a.notes,
                idle: a.idle,
                clicks: a.clicks,
                scroll: a.scroll,
                shot: a.shot,
                shown: 0,
                ready_at: 0.0,
            }))
        }),
    )
}
