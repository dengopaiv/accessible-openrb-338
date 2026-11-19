//! The mod theme viewer (task-187): the panel composed from a kit's 58 bitmaps follows the
//! panel state (knob frames by the ui_spec formula, lit LEDs, displays, meters), a mod's
//! bitmaps fall back one by one to the built-in ones, and no input to the viewer changes
//! anything: it shows the "not allowed" cursor, and the song, the transport, the main panel and
//! the status line stay as they were. Everything uses the embedded standard mod; no MIDI, audio
//! device or file of the original. The third-party collection's test is gated on
//! `ORB_THIRDPARTY_MODS_DIR`.

use std::cell::RefCell;
use std::path::{Path, PathBuf};

use eframe::egui::{
    self, Color32, ColorImage, CursorIcon, Event, Key, Modifiers, PointerButton, Pos2, RawInput,
    Rect, Vec2, ViewportEvent, ViewportId, ViewportInfo,
};
use orb_cli::mods::Kit;
use orb_ui::app::App;
use orb_ui::layout::{PANEL_H, PANEL_W};
use orb_ui::menu::Cmd;
use orb_ui::model::Panel;
use orb_ui::settings::Settings;
use orb_ui::theme_view::{
    self, Bitmaps, IDS, Lights, Op, compose, gr_lit, header_meter_frame, knob_frame, ops,
};

/// Frame `frame` of the `n`-frame strip `id` (`vertical` or not), `w` x `h` from `(sx, sy)` of
/// its cell, as pixels.
fn frame_pixels(b: &Bitmaps, op: Op) -> Vec<Color32> {
    let img = b.get(op.id).unwrap();
    let (sx, sy) = op.source(img.size);
    let mut v = Vec::new();
    for j in 0..op.h {
        for i in 0..op.w {
            v.push(img.pixels[(sy + j) as usize * img.size[0] + (sx + i) as usize]);
        }
    }
    v
}

/// The composed pixels of a panel rectangle.
fn region(img: &ColorImage, x: i32, y: i32, w: i32, h: i32) -> Vec<Color32> {
    let mut v = Vec::new();
    for j in 0..h {
        for i in 0..w {
            v.push(img.pixels[(y + j) as usize * theme_view::W + (x + i) as usize]);
        }
    }
    v
}

/// Whether the composed panel shows `op` (its frame at its place).
fn shows(img: &ColorImage, b: &Bitmaps, op: Op) -> bool {
    region(img, op.x, op.y, op.w, op.h) == frame_pixels(b, op)
}

fn compose_panel(p: &Panel) -> ColorImage {
    compose(Bitmaps::builtin(), &ops(p, 0.0, Lights::default()))
}

#[test]
fn the_builtin_kit_has_all_58_bitmaps_at_their_sizes() {
    let b = Bitmaps::builtin();
    assert!(b.fallbacks.is_empty(), "{:?}", b.fallbacks);
    // a few sizes of docs/ui_spec.md section 21
    for (id, w, h) in [
        (12015, 109, 16),
        (12057, 592, 50),
        (12062, 16, 510),
        (12507, 1674, 30),
        (12552, 608, 19),
        (12554, 68, 504),
        (12557, 15, 792),
        (12571, 37, 20),
    ] {
        assert_eq!(b.get(id).unwrap().size, [w, h], "{id}");
    }
    assert_eq!(IDS.len(), 58);
}

#[test]
fn knobs_show_the_ui_spec_frame() {
    let b = Bitmaps::builtin();
    let mut p = Panel::new();
    // a 303 knob (54 frames), an 808 level knob (36), a 909 knob (41), a mixer pan (32)
    let cases = [
        ("tb303_1.cutoff", 500, 12507, 54, 195, 62, 30, 30),
        ("tb303_2.tune", 1000, 12507, 54, 162, 177, 30, 30),
        ("tr808.bd_level", 250, 12521, 36, 126, 289, 13, 13),
        ("tr808.bd_tone", 0, 12522, 36, 126, 311, 13, 13),
        ("tr909.flam", 777, 12565, 41, 99, 471, 12, 12),
        ("mixer3.pan", 333, 12552, 32, 466, 305, 18, 19),
        ("transport.shuffle", 640, 12552, 32, 27, 13, 18, 19),
    ];
    for (id, pos, ..) in cases {
        p.set_knob(id, pos);
    }
    let img = compose_panel(&p);
    for (id, pos, bmp, n, x, y, w, h) in cases {
        let f = ((n as i32 - 1) * pos + 500) / 1000;
        assert_eq!(knob_frame(n, pos) as i32, f);
        let op = Op::h(bmp, n, f as u16, x, y, w, h);
        assert!(shows(&img, b, op), "{id} at {pos} shows frame {f}");
        // and not the frame next to it
        let other = Op::h(bmp, n, if f > 0 { f as u16 - 1 } else { 1 }, x, y, w, h);
        assert!(!shows(&img, b, other), "{id}");
    }
    assert_eq!(knob_frame(54, 500), 27);
    assert_eq!(knob_frame(32, 0), 0);
    assert_eq!(knob_frame(32, 1000), 31);
}

#[test]
fn leds_toggles_and_lights_follow_the_state() {
    let b = Bitmaps::builtin();
    let mut p = Panel::new();
    p.transport.playing = false;
    p.synth[0].pitch_mode = false;
    let dark = compose_panel(&p);
    // the 303's pitch-mode LED (12506) and the Play button (12559)
    let pitch = |f| Op::h(12506, 2, f, 113, 116, 8, 8);
    let play = |f| Op::h(12559, 2, f, 112, 21, 32, 25);
    assert!(shows(&dark, b, pitch(0)));
    assert!(shows(&dark, b, play(0)));
    p.synth[0].pitch_mode = true;
    p.transport.playing = true;
    let lit = compose_panel(&p);
    assert!(shows(&lit, b, pitch(1)), "a lit LED");
    assert!(shows(&lit, b, play(1)), "Play lit while playing");

    // the row focus bar (12068) and the pattern grid (12554, frame = button + 1)
    p.focus = 2;
    p.adopt_pattern(2, 8 + 5);
    let img = compose_panel(&p);
    assert!(shows(&img, b, Op::h(12068, 2, 1, 86, 280, 4, 115)));
    assert!(shows(&img, b, Op::h(12068, 2, 0, 86, 50, 4, 115)));
    // (its last row lies under the bank row, drawn after it)
    assert!(shows(&img, b, Op::v(12554, 9, 6, 17, 299, 68, 39)));
    assert!(
        shows(&img, b, Op::v(12555, 4, 1, 17, 338, 68, 30)),
        "bank B"
    );

    // an 808 step lit in its group's color, and the running light inverts the playing step
    p.inst808 = 1;
    p.steps808[1][5] = true;
    p.playing_step808 = Some(9);
    let img = compose_panel(&p);
    assert!(shows(&img, b, Op::h(12524, 4, 1, 90 + 16 + 80, 371, 11, 5)));
    assert!(shows(
        &img,
        b,
        Op::h(12524, 4, 2, 90 + 16 + 144, 371, 11, 5)
    ));
    // a 909 accented step (value 2: frame 1) and a flam (value 3: frame 3)
    p.inst909 = 1;
    p.steps909[1][0] = 2;
    p.steps909[1][1] = 3;
    let img = compose_panel(&p);
    assert!(shows(&img, b, Op::h(12566, 4, 1, 120, 481, 14, 8)));
    assert!(shows(&img, b, Op::h(12566, 4, 3, 141, 481, 14, 8)));
}

#[test]
fn displays_show_their_values() {
    let b = Bitmaps::builtin();
    let mut p = Panel::new();
    p.transport.tempo = 141_000;
    p.transport.song_mode = false;
    let img = compose_panel(&p);
    // 12015: frame 0 blank, frame d + 1 the digit d
    let digit = |f, x, y| Op::h(12015, 11, f, x, y, 9, 16);
    for (i, d) in [1u16, 4, 1].into_iter().enumerate() {
        assert!(
            shows(&img, b, digit(d + 1, 60 + 9 * i as i32, 27)),
            "tempo 141"
        );
    }
    // pattern mode: the bar display is disabled, all blank
    for i in 0..3 {
        assert!(shows(&img, b, digit(0, 287 + 9 * i, 27)));
    }
    // song mode: bar 3 right aligned, leading zeros blank
    p.transport.song_mode = true;
    p.transport.position = 2 * 768 + 5;
    let img = compose_panel(&p);
    assert!(shows(&img, b, digit(0, 287, 27)));
    assert!(shows(&img, b, digit(0, 296, 27)));
    assert!(shows(&img, b, digit(4, 305, 27)));
    assert_eq!(theme_view::digits(Some(16), 2), [2, 7]);
    assert_eq!(theme_view::digits(Some(3), 2), [0, 4]);
    assert_eq!(theme_view::digits(None, 3), [0, 0, 0]);
}

#[test]
fn meters_and_faders_follow_the_state() {
    let b = Bitmaps::builtin();
    let mut p = Panel::new();
    p.meters.insert("master.meter_l".into(), 7);
    p.meters.insert("mixer2.meter".into(), 9);
    p.meters.insert("delay.meter".into(), 3);
    // the link's idle compressor (unity gain, byte 18: one LED left of the center)
    p.gr_leds = [0, 0, 0, 2, 1, 0, 0, 0, 0];
    let img = compose_panel(&p);
    assert!(shows(&img, b, Op::v(12557, 11, 7, 542, 74, 15, 56)));
    assert!(shows(&img, b, Op::v(12557, 11, 0, 576, 74, 15, 56)));
    assert!(shows(&img, b, Op::h(12558, 4, 3, 507, 169, 14, 15)));
    assert!(shows(&img, b, Op::h(12067, 4, 1, 590, 262, 13, 16)));
    // unity gain: the gain-reduction LEDs lit from the left up to "0" (the original's idle
    // compressor on the built-in kit), the others off
    let led = |i: i32, frame: u16| Op {
        sx: 7 * i,
        ..Op::v(12570, 2, frame, 534 + 7 * i, 442, 7, 10)
    };
    for i in 0..5 {
        assert!(shows(&img, b, led(i, 1)), "LED {i} lit");
    }
    for i in 5..9 {
        assert!(shows(&img, b, led(i, 0)), "LED {i} off");
    }
    // more gain lengthens the bar to the right, all nine at the byte's top
    p.gr_leds = [0, 3, 2, 2, 1, 0, 0, 0, 0];
    let img = compose_panel(&p);
    assert!(shows(&img, b, led(7, 1)));
    assert!(shows(&img, b, led(8, 0)));
    assert_eq!(gr_lit(&[0, 0, 2, 2, 1, 0, 0, 0, 0]), 6);
    assert_eq!(gr_lit(&[3, 3, 2, 2, 1, 0, 0, 0, 0]), 9);
    // a gain byte of 0 (the original with the master's COMP switch off), and before the engine
    // reports: none
    assert_eq!(gr_lit(&[0, 0, 0, 0, 1, 0, 0, 0, 0]), 0);
    assert_eq!(gr_lit(&[0; 9]), 0);
    assert_eq!(header_meter_frame(0), 0);
    assert_eq!(header_meter_frame(10), 3);
}

/// Review round 2 of task-187 ("PCF sliders are off"): the PCF's four faders are the short
/// track 12030 (45 high, travel 34) at (530 + 19i, 203), as the original draws them (the
/// screenshot matches 12030 there at 1.5); the mixer's and the master's are 12032 (54 high,
/// travel 43). The handle (12031) is at the top at 999, the bottom at 0, linear between.
#[test]
fn fader_handles_sit_on_their_tracks() {
    let b = Bitmaps::builtin();
    // (id, track bitmap, track height, x, y)
    let faders = [
        ("pcf.freq", 12030, 45, 530, 203),
        ("pcf.q", 12030, 45, 549, 203),
        ("pcf.amt", 12030, 45, 568, 203),
        ("pcf.dec", 12030, 45, 587, 203),
        ("mixer1.level", 12032, 54, 466, 109),
        ("mixer4.level", 12032, 54, 466, 454),
        ("master.fader", 12032, 54, 559, 75),
    ];
    // (position, the handle's top from the track's top for a 45- and a 54-high track)
    for (pos, top45, top54) in [(0, 34, 43), (999, 0, 0), (500, 17, 21), (250, 25, 32)] {
        assert_eq!(theme_view::handle_top(pos, 45), top45, "{pos}");
        assert_eq!(theme_view::handle_top(pos, 54), top54, "{pos}");
        let mut p = Panel::new();
        for (id, ..) in faders {
            p.set_fader(id, pos);
        }
        let img = compose_panel(&p);
        for (id, track, h, x, y) in faders {
            let top = if h == 45 { top45 } else { top54 };
            assert!(
                shows(&img, b, Op::pic(12031, x, y + top, 15, 11)),
                "{id} at {pos}"
            );
            // the track shows above the handle
            if top > 0 {
                assert!(
                    shows(&img, b, Op::pic(track, x, y, 15, top)),
                    "{id} at {pos}"
                );
            }
            // and nothing of it below its end: the PCF's labels (y 251..) are the background's
            if h == 45 {
                let bg = Op {
                    sx: x - 526,
                    sy: 103,
                    ..Op::pic(12064, x, 251, 15, 7)
                };
                assert!(shows(&img, b, bg), "{id}: the FREQ..DEC labels are clear");
            }
        }
    }
}

/// A JPEG of `w` x `h` in one color.
fn jpeg(w: u32, h: u32, rgb: [u8; 3]) -> Vec<u8> {
    let img = image::RgbImage::from_pixel(w, h, image::Rgb(rgb));
    let mut out = std::io::Cursor::new(Vec::new());
    img.write_to(&mut out, image::ImageFormat::Jpeg).unwrap();
    out.into_inner()
}

#[test]
fn a_mods_bitmaps_fall_back_one_by_one() {
    let red = jpeg(1674, 30, [255, 0, 0]);
    // 32 frames of 4 x 6 (the built-in knobs are 18 x 19)
    let odd = jpeg(32 * 5 - 1, 6, [0, 0, 255]);
    let b = Bitmaps::from_files(|name| match name {
        "12507.jpg" => Some(red.clone()),
        // a size the panel does not expect: drawn clipped, no panic
        "12552.jpg" => Some(odd.clone()),
        // broken: the built-in one
        "12015.jpg" => Some(b"\xFF\xD8not a jpeg".to_vec()),
        // missing: the built-in one
        "12057.jpg" => None,
        n => rbm::standard::builtin_file(n),
    });
    let ids: Vec<u16> = b.fallbacks.iter().map(|(id, _)| *id).collect();
    assert_eq!(ids, [12015, 12057]);
    let builtin = Bitmaps::builtin();
    assert_eq!(
        b.get(12015).unwrap().pixels,
        builtin.get(12015).unwrap().pixels
    );
    assert_eq!(
        b.get(12057).unwrap().pixels,
        builtin.get(12057).unwrap().pixels
    );
    let img = compose(&b, &ops(&Panel::new(), 0.0, Lights::default()));
    // the mod's knob strip is drawn: red-ish (JPEG) where the 303's knobs are
    let px = img.pixels[(62 + 15) * theme_view::W + 195 + 15];
    assert!(px.r() > 200 && px.g() < 60, "{px:?}");
    // the odd strip draws its own frame size, 4 x 6, at the knob's top-left corner, as the
    // original sizes a strip by its bitmap; the transport's background shows around it
    let px = img.pixels[(13 + 2) * theme_view::W + 27 + 1];
    assert!(px.b() > 200, "{px:?}");
    let bg = builtin.get(12057).unwrap();
    let (x, y) = (27 + 10, 13 + 10);
    assert_eq!(
        img.pixels[y * theme_view::W + x],
        bg.pixels[y * bg.size[0] + (x - 16)]
    );
    // a strip too narrow for its frames (3 px for 32) draws nothing, and does not panic
    let thin = jpeg(3, 700, [0, 0, 255]);
    let b = Bitmaps::from_files(|name| match name {
        "12552.jpg" => Some(thin.clone()),
        n => rbm::standard::builtin_file(n),
    });
    let img = compose(&b, &ops(&Panel::new(), 0.0, Lights::default()));
    assert_eq!(
        img.pixels[y * theme_view::W + x],
        bg.pixels[y * bg.size[0] + (x - 16)]
    );
}

/// Review round 4 of task-187: a mod's strip of another size than the built-in one is drawn
/// at its own frame size (the original sizes the view by its bitmap): ReBirthday's Sync and
/// MIDI LEDs (12041, 23 x 7: three frames of 7 x 7), its Loop switch (12551, 81 x 15: two of
/// 40 x 15) and its Flam button (12568, 35 x 21: two of 17 x 21), checked here with
/// synthetic strips of those sizes whose frames are told apart by color.
#[test]
fn a_strip_of_another_size_draws_its_own_frames() {
    // frame k of n in color k, the 1-px gaps white
    let strip = |n: u32, fw: u32, h: u32| {
        let colors = [[255u8, 0, 0], [0, 255, 0], [0, 0, 255]];
        let img = image::RgbImage::from_fn(n * (fw + 1) - 1, h, |x, _| {
            if x % (fw + 1) == fw {
                image::Rgb([255, 255, 255])
            } else {
                image::Rgb(colors[(x / (fw + 1)) as usize % 3])
            }
        });
        let mut out = std::io::Cursor::new(Vec::new());
        img.write_to(&mut out, image::ImageFormat::Png).unwrap();
        // the decoder takes JPEG only: re-encode at the best quality
        let dynimg = image::load_from_memory(&out.into_inner()).unwrap();
        let mut j = std::io::Cursor::new(Vec::new());
        image::codecs::jpeg::JpegEncoder::new_with_quality(&mut j, 100)
            .encode_image(&dynimg)
            .unwrap();
        j.into_inner()
    };
    let leds = strip(3, 7, 7);
    let looping = strip(2, 40, 15);
    let flam = strip(2, 17, 21);
    let b = Bitmaps::from_files(|name| match name {
        "12041.jpg" => Some(leds.clone()),
        "12551.jpg" => Some(looping.clone()),
        "12568.jpg" => Some(flam.clone()),
        n => rbm::standard::builtin_file(n),
    });
    let mut p = Panel::new();
    p.transport.looping = true;
    let img = compose(&b, &ops(&p, 0.0, Lights::default()));
    let px = |x: usize, y: usize| img.pixels[y * theme_view::W + x];
    let red = |c: Color32| c.r() > 200 && c.g() < 60 && c.b() < 60;
    let green = |c: Color32| c.g() > 200 && c.r() < 60 && c.b() < 60;
    // the Sync LED: frame 0 (off) 7 x 7 at (55, 2), the transport's art right of and under it
    assert!(red(px(55, 2)) && red(px(61, 8)), "{:?}", px(61, 8));
    let bg = Bitmaps::builtin().get(12057).unwrap();
    let under = |x: usize, y: usize| bg.pixels[y * bg.size[0] + x - 16];
    assert_eq!(
        px(62, 2),
        under(62, 2),
        "nothing of frame 1 (green) beside it"
    );
    assert_eq!(px(55, 9), under(55, 9));
    // the Loop switch: frame 1 (on) 40 x 15 at (331, 2)
    assert!(green(px(331, 2)) && green(px(370, 16)), "{:?}", px(370, 16));
    // the Flam button: frame 0 (off) 17 x 21 at (98, 488)
    assert!(red(px(98, 488)) && red(px(114, 508)), "{:?}", px(114, 508));
}

/// The embedded standard mod repacked with `change` applied to its files, under another name.
fn repacked(dir: &Path, name: &str, change: impl Fn(&str, &mut Vec<u8>) -> bool) -> PathBuf {
    let r = rbm::Rbm::parse(rbm::standard::EMBEDDED.to_vec()).unwrap();
    let t = rbm::KeyTable::embedded(&r).unwrap();
    let mut u = rbm::pack::unpack(&r, &t).unwrap();
    let mut keep_entries = Vec::new();
    let mut keep_files = Vec::new();
    for (e, mut f) in u.manifest.files.drain(..).zip(u.files.drain(..)) {
        if change(&e.name, &mut f) {
            keep_entries.push(e);
            keep_files.push(f);
        }
    }
    u.manifest.files = keep_entries;
    u.manifest.info.name = name.into();
    let bytes = rbm::pack::pack(&u.manifest, &keep_files).unwrap();
    let path = dir.join(format!("{name}.rbm"));
    std::fs::write(&path, bytes).unwrap();
    path
}

/// The app headless, the panel at 1:1 in a 624 x 510 window (no menu bar), and the input the
/// viewer's own native window gets (the immediate viewport renderer below runs the viewer's
/// pass with it).
struct Harness {
    ctx: egui::Context,
    app: App,
    time: f64,
}

/// The viewer window's side of the harness.
#[derive(Default)]
struct Window {
    /// The input of its next pass, and whether its close button is pressed then.
    events: Vec<Event>,
    close: bool,
    /// The cursor its last pass asked for.
    cursor: Option<CursorIcon>,
    /// Its passes so far, and the Focus commands it sent.
    passes: u32,
    focus: u32,
    /// The inner size the window reports (`None`: the panel's size), and the viewport id and
    /// the inner size asked by the builder of its last pass.
    size: Option<Vec2>,
    last: Option<(ViewportId, Option<Vec2>)>,
    /// The viewports the last root pass kept, and the root's repaint delay then.
    alive: Vec<ViewportId>,
    root_repaint: Option<std::time::Duration>,
    /// The fill of the window-wide rectangle its last pass painted.
    desk: Option<Color32>,
}

thread_local! {
    static VIEWER: RefCell<Window> = RefCell::new(Window::default());
}

fn passes() -> u32 {
    VIEWER.with(|v| v.borrow().passes)
}

const VIEWER_RECT: Rect = Rect {
    min: Pos2::ZERO,
    max: Pos2::new(PANEL_W, PANEL_H),
};

impl Harness {
    /// The viewer is a window of its own (egui's immediate viewport, as in the standalone
    /// app).
    fn new() -> Self {
        let ctx = egui::Context::default();
        egui_extras::install_image_loaders(&ctx);
        ctx.set_embed_viewports(false);
        egui::Context::set_immediate_viewport_renderer(|ctx, mut vp| {
            let (events, close, size) = VIEWER.with(|v| {
                let mut v = v.borrow_mut();
                v.passes += 1;
                v.last = Some((vp.ids.this, vp.builder.inner_size));
                (
                    std::mem::take(&mut v.events),
                    std::mem::take(&mut v.close),
                    v.size,
                )
            });
            let screen = size.map_or(VIEWER_RECT, |s| Rect::from_min_size(Pos2::ZERO, s));
            let mut info = ViewportInfo {
                inner_rect: Some(screen),
                ..Default::default()
            };
            if close {
                info.events.push(ViewportEvent::Close);
            }
            let input = RawInput {
                viewport_id: vp.ids.this,
                viewports: std::iter::once((vp.ids.this, info)).collect(),
                screen_rect: Some(screen),
                time: Some(ctx.input(|i| i.time)),
                events,
                ..Default::default()
            };
            let mut out = ctx.run_ui(input, |ui| (vp.viewport_ui_cb)(ui));
            out.textures_delta.clear();
            // the fill of the rectangle that covers the whole window: the letterbox
            let desk = out.shapes.iter().find_map(|c| match &c.shape {
                egui::Shape::Rect(r) if r.rect == screen => Some(r.fill),
                _ => None,
            });
            VIEWER.with(|v| {
                let mut v = v.borrow_mut();
                v.cursor = Some(out.platform_output.cursor_icon);
                v.desk = desk;
            });
        });
        let app = App::new(Panel::new(), None);
        let mut h = Self {
            ctx,
            app,
            time: 0.0,
        };
        h.frame(vec![]);
        h
    }

    /// A frame of the main window with `events`.
    fn frame(&mut self, events: Vec<Event>) -> CursorIcon {
        self.time += 0.016;
        let input = RawInput {
            viewport_id: ViewportId::ROOT,
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(PANEL_W, PANEL_H))),
            time: Some(self.time),
            events,
            ..Default::default()
        };
        let app = &mut self.app;
        let mut out = self.ctx.run_ui(input, |ui| app.show(ui));
        out.textures_delta.clear();
        // the root pass hands out the commands of its immediate viewports too
        let focus = out
            .viewport_output
            .iter()
            .filter(|(id, _)| **id != ViewportId::ROOT)
            .flat_map(|(_, o)| &o.commands)
            .filter(|c| **c == egui::ViewportCommand::Focus)
            .count() as u32;
        let alive = out
            .viewport_output
            .keys()
            .copied()
            .filter(|id| *id != ViewportId::ROOT)
            .collect();
        let root_repaint = out
            .viewport_output
            .get(&ViewportId::ROOT)
            .map(|o| o.repaint_delay);
        VIEWER.with(|v| {
            let mut v = v.borrow_mut();
            v.focus += focus;
            v.alive = alive;
            v.root_repaint = root_repaint;
        });
        out.platform_output.cursor_icon
    }

    /// A frame with `events` for the viewer's own window; the cursor its pass asked for.
    fn viewer_frame(&mut self, events: Vec<Event>) -> Option<CursorIcon> {
        VIEWER.with(|v| {
            let mut v = v.borrow_mut();
            v.events = events;
            v.cursor = None;
        });
        self.frame(vec![]);
        VIEWER.with(|v| v.borrow().cursor)
    }

    /// Everything the viewer's input must leave alone: the panel model (with the events the
    /// engine would hear), the status line, the errors and the open song.
    fn state(&self) -> String {
        format!(
            "{:?}\n{:?}\n{:?}\n{:?} {:?} {}",
            self.app.panel,
            self.app.status,
            self.app.errors,
            self.app.doc.path,
            self.app.doc.kit,
            self.app.doc.modified
        )
    }
}

fn button(p: Pos2, pressed: bool, modifiers: Modifiers) -> Event {
    Event::PointerButton {
        pos: p,
        button: PointerButton::Primary,
        pressed,
        modifiers,
    }
}

fn key(k: Key, pressed: bool) -> Event {
    Event::Key {
        key: k,
        physical_key: Some(k),
        pressed,
        repeat: false,
        modifiers: Modifiers::NONE,
    }
}

/// Every kind of input over the panel point `p`: a click, a shift click, a double click, a
/// drag up and down, the wheel, and keys.
fn all_input(p: Pos2) -> Vec<Vec<Event>> {
    let up = p - Vec2::new(0.0, 20.0);
    let shift = Modifiers::SHIFT;
    vec![
        vec![Event::PointerMoved(p)],
        vec![button(p, true, Modifiers::NONE)],
        vec![button(p, false, Modifiers::NONE)],
        vec![button(p, true, shift), button(p, false, shift)],
        vec![
            button(p, true, Modifiers::NONE),
            button(p, false, Modifiers::NONE),
        ],
        vec![
            button(p, true, Modifiers::NONE),
            button(p, false, Modifiers::NONE),
        ],
        vec![button(p, true, Modifiers::NONE)],
        vec![Event::PointerMoved(up)],
        vec![Event::PointerMoved(p + Vec2::new(0.0, 30.0))],
        vec![button(p + Vec2::new(0.0, 30.0), false, Modifiers::NONE)],
        vec![Event::MouseWheel {
            unit: egui::MouseWheelUnit::Line,
            delta: Vec2::new(0.0, 3.0),
            phase: egui::TouchPhase::Move,
            modifiers: Modifiers::NONE,
            source: egui::MouseWheelSource::Wheel,
        }],
        vec![key(Key::Space, true), key(Key::Space, false)],
        vec![key(Key::C, true), key(Key::C, false)],
        vec![key(Key::Enter, true), key(Key::Enter, false)],
    ]
}

/// The viewer points over controls of the main panel: Play, the 303's cutoff knob, a pattern
/// button, the mixer fader, an 808 step, the logo, a 303 key.
const POINTS: [(f32, f32); 7] = [
    (128.0, 33.0),
    (210.0, 77.0),
    (40.0, 80.0),
    (473.0, 130.0),
    (130.0, 380.0),
    (530.0, 25.0),
    (146.0, 147.0),
];

#[test]
fn the_native_viewer_opens_closes_and_ignores_all_input() {
    let mut h = Harness::new();
    assert!(!h.app.theme_viewer_is_open());
    h.app.command(Cmd::ThemeViewer);
    assert!(h.app.theme_viewer_is_open());
    let calls = passes();
    h.viewer_frame(vec![]);
    assert!(passes() > calls, "the viewer is a viewport of its own");
    assert!(h.app.theme_viewer_bitmaps().is_some());
    h.viewer_frame(vec![]);
    let before = h.state();
    let drawn = h.app.theme_viewer_ops().to_vec();
    for (x, y) in POINTS {
        for events in all_input(Pos2::new(x, y)) {
            let cursor = h.viewer_frame(events);
            assert_eq!(cursor, Some(CursorIcon::NotAllowed), "over ({x}, {y})");
            assert_eq!(h.state(), before, "input at ({x}, {y}) changed nothing");
            assert_eq!(
                h.app.theme_viewer_ops(),
                drawn,
                "no control of the viewer reacts"
            );
        }
    }
    assert!(h.app.status.is_none(), "no status line");
    // the window's close button closes it; the menu opens it again
    VIEWER.with(|v| v.borrow_mut().close = true);
    h.frame(vec![]);
    assert!(!h.app.theme_viewer_is_open());
    let calls = passes();
    h.frame(vec![]);
    assert_eq!(passes(), calls, "closed: no viewport");
    h.app.command(Cmd::ThemeViewer);
    h.viewer_frame(vec![]);
    assert!(h.app.theme_viewer_is_open());
    assert!(passes() > calls);
}

/// The user's review of task-187: Space in the viewer's window switches its scaling between
/// nearest pixel (the default with nothing saved, review round 4) and bilinear; the choice is
/// saved in the UI settings (`theme_viewer_filter`) and taken by the next run, a saved choice
/// wins over the default; the key starts no playback (Space in the main window does).
#[test]
fn space_in_the_viewer_switches_the_scaling_and_is_kept() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("ui.toml");
    // another key of the file survives the save (task-229's merge); no saved scaling
    std::fs::write(&file, "startup_sound = false\n").unwrap();
    let mut h = Harness::new();
    h.app.settings = Settings::load_from(&file);
    h.app.command(Cmd::ThemeViewer);
    h.viewer_frame(vec![]);
    assert_eq!(
        h.app.theme_viewer_nearest(),
        Some(true),
        "nearest by default"
    );
    let before = h.state();
    let space = || vec![key(Key::Space, true), key(Key::Space, false)];
    h.viewer_frame(space());
    h.viewer_frame(vec![]);
    assert_eq!(
        h.app.theme_viewer_nearest(),
        Some(false),
        "bilinear after Space"
    );
    assert!(
        !h.app.panel.transport.playing,
        "Space in the viewer plays nothing"
    );
    assert_eq!(h.state(), before, "nothing else changed");
    let saved = Settings::load_from(&file);
    assert!(!saved.viewer_nearest);
    assert!(!saved.startup_sound, "the file's other keys are kept");
    let text = std::fs::read_to_string(&file).unwrap();
    assert!(text.contains("theme_viewer_filter = \"linear\""), "{text}");
    // a held key's repeats do not switch it back
    // (egui marks a press of a key already down as a repeat)
    h.viewer_frame(vec![key(Key::Space, true)]);
    h.viewer_frame(vec![]);
    assert_eq!(h.app.theme_viewer_nearest(), Some(true), "pressed");
    for _ in 0..3 {
        h.viewer_frame(vec![key(Key::Space, true)]);
    }
    h.viewer_frame(vec![key(Key::Space, false)]);
    h.viewer_frame(vec![]);
    assert_eq!(h.app.theme_viewer_nearest(), Some(true), "held: no switch");
    h.viewer_frame(space());
    h.viewer_frame(vec![]);
    assert_eq!(h.app.theme_viewer_nearest(), Some(false));

    // the next run starts with the saved bilinear, not the default; Space goes to nearest
    let mut h = Harness::new();
    h.app.settings = Settings::load_from(&file);
    h.app.command(Cmd::ThemeViewer);
    h.viewer_frame(vec![]);
    assert_eq!(
        h.app.theme_viewer_nearest(),
        Some(false),
        "kept across runs"
    );
    h.viewer_frame(space());
    h.viewer_frame(vec![]);
    assert_eq!(h.app.theme_viewer_nearest(), Some(true));
    assert!(Settings::load_from(&file).viewer_nearest);

    // the control: Space in the main window does start playback
    h.frame(space());
    h.frame(vec![]);
    assert!(
        h.app.panel.transport.playing,
        "Space in the main window plays"
    );
    assert_eq!(
        h.app.theme_viewer_nearest(),
        Some(true),
        "and leaves the viewer"
    );
}

/// The user's review of task-187: the 808's mod window on the main panel is a link to the
/// viewer: the hand cursor over it, a click opens the viewer, a click while it is open
/// brings it to the front with the focus; the click changes nothing else but the row focus.
#[test]
fn the_mod_window_opens_and_focuses_the_viewer() {
    let mut h = Harness::new();
    // the macOS and Windows way (Linux opens a new window: the test below)
    h.app.set_theme_viewer_reopens(false);
    let c = orb_ui::layout::controls()
        .into_iter()
        .find(|c| c.id == "tr808.mod_view")
        .unwrap();
    let p = Pos2::new(c.x + c.w / 2.0, c.y + c.h / 2.0);
    // the panel fills the 624 x 510 window at 1:1 (no menu bar)
    h.frame(vec![Event::PointerMoved(p)]);
    assert_eq!(h.frame(vec![]), CursorIcon::PointingHand);
    h.app.panel.focus = 2;
    let before = h.state();
    h.frame(vec![button(p, true, Modifiers::NONE)]);
    h.frame(vec![button(p, false, Modifiers::NONE)]);
    assert!(h.app.theme_viewer_is_open(), "a click opens the viewer");
    assert_eq!(h.state(), before, "and changes nothing else");
    h.viewer_frame(vec![]);
    let focus = VIEWER.with(|v| v.borrow().focus);
    let calls = passes();
    h.frame(vec![button(p, true, Modifiers::NONE)]);
    h.frame(vec![button(p, false, Modifiers::NONE)]);
    h.frame(vec![]);
    assert!(h.app.theme_viewer_is_open());
    assert_eq!(VIEWER.with(|v| v.borrow().focus), focus + 1, "focused");
    assert_eq!(passes(), calls + 3, "one window");
}

/// The user's review of task-187: the menu item while the viewer is open brings its window to
/// the front with the focus (one `ViewportCommand::Focus`), and opens no second one.
#[test]
fn the_menu_item_focuses_the_open_viewer() {
    let mut h = Harness::new();
    // the macOS and Windows way (Linux opens a new window: the test below)
    h.app.set_theme_viewer_reopens(false);
    h.app.command(Cmd::ThemeViewer);
    h.viewer_frame(vec![]);
    h.viewer_frame(vec![]);
    let focus = VIEWER.with(|v| v.borrow().focus);
    assert_eq!(focus, 0, "a new window needs no Focus");
    h.app.command(Cmd::ThemeViewer);
    assert!(h.app.theme_viewer_is_open());
    let calls = passes();
    h.viewer_frame(vec![]);
    assert_eq!(passes(), calls + 1, "one window, one pass");
    assert_eq!(VIEWER.with(|v| v.borrow().focus), focus + 1, "focused");
    // once: the frames after it send no more
    h.viewer_frame(vec![]);
    h.viewer_frame(vec![]);
    assert_eq!(VIEWER.with(|v| v.borrow().focus), focus + 1);
}

/// Review round 3 of task-187: on Linux (`REOPEN_TO_FOCUS`) the menu item or the mod window's
/// click on an open viewer closes its window and opens a new one at the same inner size (a new
/// window gets the focus; GNOME ignores `Focus` without an activation token); no Focus is
/// sent. Both ways are driven here whatever the platform.
#[test]
fn on_linux_an_open_viewer_comes_back_as_a_new_window_of_its_size() {
    assert_eq!(theme_view::REOPEN_TO_FOCUS, cfg!(target_os = "linux"));
    let mut h = Harness::new();
    h.app.set_theme_viewer_reopens(true);
    h.app.command(Cmd::ThemeViewer);
    h.viewer_frame(vec![]);
    let (first, asked) = VIEWER.with(|v| v.borrow().last).unwrap();
    assert_eq!(
        asked,
        Some(Vec2::new(PANEL_W, PANEL_H)),
        "the panel's size at first"
    );
    assert_eq!(first, h.app.theme_viewer_viewport());
    // the user resizes the window
    VIEWER.with(|v| v.borrow_mut().size = Some(Vec2::new(700.0, 572.0)));
    h.viewer_frame(vec![]);
    h.viewer_frame(vec![]);
    let focus = VIEWER.with(|v| v.borrow().focus);
    for cmd in 0..2 {
        let before = h.app.theme_viewer_viewport();
        if cmd == 0 {
            h.app.command(Cmd::ThemeViewer);
        } else {
            // the 808's mod window, clicked
            let c = orb_ui::layout::controls()
                .into_iter()
                .find(|c| c.id == "tr808.mod_view")
                .unwrap();
            let p = Pos2::new(c.x + c.w / 2.0, c.y + c.h / 2.0);
            h.frame(vec![Event::PointerMoved(p)]);
            h.frame(vec![button(p, true, Modifiers::NONE)]);
            h.frame(vec![button(p, false, Modifiers::NONE)]);
        }
        h.viewer_frame(vec![]);
        let (id, asked) = VIEWER.with(|v| v.borrow().last).unwrap();
        assert!(h.app.theme_viewer_is_open());
        assert_ne!(id, before, "a new window");
        assert_eq!(id, h.app.theme_viewer_viewport());
        assert_eq!(
            asked,
            Some(Vec2::new(700.0, 572.0)),
            "at the old one's size"
        );
        let alive = VIEWER.with(|v| v.borrow().alive.clone());
        assert_eq!(alive, vec![id], "the old window is gone");
        assert_eq!(VIEWER.with(|v| v.borrow().focus), focus, "no Focus");
    }
}

/// Review round 4 of task-187: the TR-909 lights the selected instrument's LED (12567) at its
/// dot: BD's at (29, 76) of the section, as the original does, and it moves with the selection.
#[test]
fn the_909_lights_its_selected_instrument() {
    let b = Bitmaps::builtin();
    let mut p = Panel::new();
    p.inst909 = 1;
    let img = compose_panel(&p);
    assert!(
        shows(&img, b, Op::pic(12567, 90 + 29, 395 + 76, 6, 6)),
        "BD"
    );
    p.inst909 = 0;
    let img = compose_panel(&p);
    assert!(shows(&img, b, Op::pic(12567, 90 + 7, 395 + 52, 6, 6)), "AC");
    assert!(
        !shows(&img, b, Op::pic(12567, 90 + 29, 395 + 76, 6, 6)),
        "BD's is dark"
    );
    p.inst909 = 11;
    let img = compose_panel(&p);
    assert!(
        shows(&img, b, Op::pic(12567, 90 + 351, 395 + 76, 6, 6)),
        "RC"
    );
}

/// Review round 4 of task-187: the letterbox around the panel, where the window's shape
/// differs from the panel's, is black.
#[test]
fn the_letterbox_is_black() {
    let mut h = Harness::new();
    VIEWER.with(|v| v.borrow_mut().size = Some(Vec2::new(900.0, 510.0)));
    h.app.command(Cmd::ThemeViewer);
    h.viewer_frame(vec![]);
    assert_eq!(theme_view::DESK, Color32::BLACK);
    assert_eq!(VIEWER.with(|v| v.borrow().desk), Some(Color32::BLACK));
}

/// Review round 3 of task-187: a viewer window that opens is repainted at once for its first
/// passes (Wayland draws its frame and title bar only on a repaint), then the app is idle
/// again.
#[test]
fn a_new_viewer_window_repaints_at_once() {
    let mut h = Harness::new();
    for _ in 0..3 {
        h.frame(vec![]);
    }
    let idle = VIEWER.with(|v| v.borrow().root_repaint).unwrap();
    assert!(idle > std::time::Duration::ZERO, "idle before: {idle:?}");
    h.app.command(Cmd::ThemeViewer);
    for i in 0..3 {
        h.viewer_frame(vec![]);
        let d = VIEWER.with(|v| v.borrow().root_repaint).unwrap();
        assert_eq!(d, std::time::Duration::ZERO, "pass {i} repaints at once");
    }
    for _ in 0..3 {
        h.viewer_frame(vec![]);
    }
    let d = VIEWER.with(|v| v.borrow().root_repaint).unwrap();
    assert!(d > std::time::Duration::ZERO, "idle after: {d:?}");
}

#[test]
fn the_viewer_redraws_with_the_mod_and_the_song() {
    let dir = tempfile::tempdir().unwrap();
    // a mod whose 303 knob strip is red and that lacks the transport's background
    let red = jpeg(1674, 30, [255, 0, 0]);
    let path = repacked(dir.path(), "Red Knobs", |name, f| {
        if name.eq_ignore_ascii_case("12507.jpg") {
            *f = red.clone();
        }
        !name.eq_ignore_ascii_case("12057.jpg")
    });
    let mut h = Harness::new();
    h.app.command(Cmd::ThemeViewer);
    h.viewer_frame(vec![]);
    let builtin = h
        .app
        .theme_viewer_bitmaps()
        .unwrap()
        .get(12507)
        .unwrap()
        .clone();
    h.app.doc.kit = Kit::Mod(path);
    h.viewer_frame(vec![]);
    let b = h.app.theme_viewer_bitmaps().unwrap();
    assert_ne!(
        b.get(12507).unwrap().pixels,
        builtin.pixels,
        "the mod's bitmap"
    );
    assert_eq!(b.fallbacks.len(), 1, "{:?}", b.fallbacks);
    assert_eq!(b.fallbacks[0].0, 12057);
    // back to the built-in kit
    h.app.doc.kit = Kit::Standard;
    h.viewer_frame(vec![]);
    let b = h.app.theme_viewer_bitmaps().unwrap();
    assert_eq!(b.get(12507).unwrap().pixels, builtin.pixels);

    // the song moves: what is drawn follows the panel
    let drawn = h.app.theme_viewer_ops().to_vec();
    h.app.panel.adopt("tb303_1.cutoff", 0);
    h.app.panel.adopt("tb303_1.cutoff", 127);
    h.viewer_frame(vec![]);
    assert_ne!(h.app.theme_viewer_ops(), drawn);
    assert!(
        h.app
            .theme_viewer_ops()
            .contains(&Op::h(12507, 54, 53, 195, 62, 30, 30))
    );
}

/// Every mod of the user's third-party collection opens in the viewer without a panic
/// (`ORB_THIRDPARTY_MODS_DIR`, the recrypted mods; skipped without it).
#[test]
fn every_thirdparty_mod_opens_in_the_viewer() {
    let Some(dir) = orb_testkit::thirdparty_mods() else {
        return;
    };
    let mut mods = Vec::new();
    let mut stack = vec![dir.clone()];
    while let Some(d) = stack.pop() {
        for e in std::fs::read_dir(&d).unwrap() {
            let p = e.unwrap().path();
            if p.is_dir() {
                stack.push(p);
            } else if p.extension().is_some_and(|x| x.eq_ignore_ascii_case("rbm")) {
                mods.push(p);
            }
        }
    }
    mods.sort();
    assert!(!mods.is_empty(), "no .rbm in {}", dir.display());
    let mut p = Panel::new();
    p.transport.song_mode = true;
    p.transport.playing = true;
    let mut fell_back = 0;
    for m in &mods {
        let b = Bitmaps::of_kit(&Kit::Mod(m.clone()));
        let img = compose(&b, &ops(&p, 0.0, Lights::default()));
        assert_eq!(img.size, [theme_view::W, theme_view::H]);
        if !b.fallbacks.is_empty() {
            fell_back += 1;
            let mut why: Vec<&str> = b.fallbacks.iter().map(|(_, e)| e.as_str()).collect();
            why.sort();
            why.dedup();
            eprintln!(
                "{}: {} built-in bitmaps ({})",
                m.display(),
                b.fallbacks.len(),
                why.join("; ")
            );
        }
    }
    eprintln!(
        "{} mods composed, {fell_back} with built-in bitmaps",
        mods.len()
    );
}
