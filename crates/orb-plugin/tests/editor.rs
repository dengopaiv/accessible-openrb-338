//! The editor (task-201) headless: orb-ui's panel of the plugin run frame by frame in an egui
//! context of the panel's size, with a stand-in host that applies the parameter changes at
//! once and a process side run between the frames.

mod common;

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use egui::{Event, Modifiers, PointerButton, Pos2, RawInput, Rect, Vec2, ViewportCommand};
use nice_plug::context::gui::GuiContext;
use nice_plug::editor::HostCallbacks;
use nice_plug::editor::dpi::Size;
use nice_plug::prelude::*;
use orb_cli::mods::Kit as UiKit;
use orb_player::midi::Message;
use orb_plugin::core::{Core, HostEvent, HostTransport};
use orb_plugin::editor::{
    self, DEFAULT_SIZE, HostRoute, OrbEditor, adjust_logical, editor_app, editor_app_routed,
};
use orb_plugin::instance::{self, Kit, OrbParams};
use orb_ui::app::App;
use orb_ui::embed::{CORNER, MENU_H, PLUGIN_HIDDEN, window_height};
use orb_ui::layout::{self, PANEL_H, PANEL_W};
use orb_ui::settings::Settings;

use common::{song_bytes, test_mod};

/// A host that applies a parameter change at once and counts the gestures.
#[derive(Default)]
struct Host {
    gestures: Mutex<u32>,
}

impl GuiContextInner for Host {
    fn plugin_api(&self) -> PluginApi {
        PluginApi::Vst3
    }

    unsafe fn raw_begin_set_parameter(&self, _param: ParamPtr) {
        *self.gestures.lock().unwrap() += 1;
    }

    unsafe fn raw_set_parameter_normalized(&self, param: ParamPtr, normalized: f32) {
        unsafe { param._internal_set_normalized_value(normalized) };
    }

    unsafe fn raw_end_set_parameter(&self, _param: ParamPtr) {}

    fn get_state(&self) -> PluginState {
        unimplemented!()
    }

    fn set_state(&self, _state: PluginState) {}

    fn request_restart(&self) {}
}

struct Harness {
    ctx: egui::Context,
    params: Arc<OrbParams>,
    host: Arc<Host>,
    core: Core,
    app: App,
    time: f64,
    pos: Pos2,
    /// The size of the window in points.
    size: Vec2,
    /// The viewport commands of the last frame.
    commands: Vec<ViewportCommand>,
    /// The cursor of the last frame.
    cursor: egui::CursorIcon,
    playing: bool,
    beats: f64,
}

/// The settings of a test: a folder of their own, nothing read from the user's.
fn settings() -> Settings {
    Settings::default()
}

impl Harness {
    fn new(params: OrbParams) -> Self {
        let params = Arc::new(params);
        let host = Arc::new(Host::default());
        let core = instance::core(&params, 44100);
        let gui = GuiContext::new(host.clone());
        let app = editor_app(params.clone(), gui, settings());
        let ctx = egui::Context::default();
        // the host runs a buffer before the editor acts: the activation's engine is then the
        // current one (a parameter change before the first engine block is lost when the
        // block's queued load replaces the engine after it; task-214)
        egui_extras::install_image_loaders(&ctx);
        let mut h = Harness {
            ctx,
            params,
            host,
            core,
            app,
            time: 0.0,
            pos: Pos2::ZERO,
            size: Vec2::new(PANEL_W, PANEL_H + MENU_H),
            commands: Vec::new(),
            cursor: egui::CursorIcon::Default,
            playing: false,
            beats: 0.0,
        };
        h.process(&[]);
        h.frame(vec![]);
        h
    }

    /// One host buffer of 512 frames with `events`.
    fn process(&mut self, events: &[HostEvent]) {
        let (mut l, mut r) = (vec![0f32; 512], vec![0f32; 512]);
        let t = HostTransport {
            playing: self.playing,
            tempo: Some(120.0),
            pos_beats: Some(self.beats),
        };
        self.core
            .process_with(&mut l, &mut r, &self.params.values(), &t, events);
        if self.playing {
            self.beats += 512.0 / 44100.0 * 2.0;
        }
    }

    /// Enough host buffers for the panel to take the engine's values again
    /// (`editor::FOLLOW_FRAMES`).
    fn settle(&mut self) {
        for _ in 0..(editor::FOLLOW_FRAMES / 512 + 2) {
            self.process(&[]);
        }
    }

    fn frame(&mut self, events: Vec<Event>) {
        self.time += 0.016;
        let input = RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, self.size)),
            time: Some(self.time),
            events,
            ..Default::default()
        };
        let app = &mut self.app;
        let mut out = self.ctx.run_ui(input, |ui| app.show(ui));
        out.textures_delta.clear();
        self.cursor = out.platform_output.cursor_icon;
        self.commands = out
            .viewport_output
            .values()
            .flat_map(|v| v.commands.clone())
            .collect();
    }

    /// The center of a control (the window is the menu bar and the panel at 1:1).
    fn center(id: &str) -> Pos2 {
        let c = layout::controls().into_iter().find(|c| c.id == id).unwrap();
        Pos2::new(c.x + c.w / 2.0, MENU_H + c.y + c.h / 2.0)
    }

    fn move_to(&mut self, p: Pos2) {
        self.pos = p;
        self.frame(vec![Event::PointerMoved(p)]);
    }

    fn button(&mut self, pressed: bool) {
        let pos = self.pos;
        self.frame(vec![Event::PointerButton {
            pos,
            button: PointerButton::Primary,
            pressed,
            modifiers: Modifiers::NONE,
        }]);
    }

    fn click(&mut self, p: Pos2) {
        self.move_to(p);
        self.button(true);
        self.button(false);
    }

    fn value(&self, id: &str) -> i32 {
        self.params.param(id).unwrap().value()
    }
}

/// An instance with the diag song `name` loaded and its controls set.
fn with(name: &str) -> OrbParams {
    let p = common::with_song(name);
    common::set(&p, "g1", 0);
    p
}

#[test]
fn the_editor_shows_the_plugins_song() {
    let p = with("303_accent127");
    // the song's 303 accent, as the parameter holds it
    let want = p.param("d1c7").unwrap().value();
    let h = Harness::new(p);
    let shown = h.app.panel.knob("tb303_1.accent");
    assert_eq!(
        shown,
        orb_ui::behavior::knob_from_engine(want),
        "the 303's accent knob shows the parameter"
    );
}

#[test]
fn play_and_stop_are_the_mode() {
    let mut h = Harness::new(with("303_accent0"));
    assert_eq!(h.value("g1"), 0);
    h.click(Harness::center("transport.play"));
    assert_eq!(h.value("g1"), 1, "Play is pattern mode");
    h.settle();
    h.frame(vec![]);
    assert!(
        h.app.panel.transport.playing,
        "the Play light follows the engine"
    );
    h.click(Harness::center("transport.stop"));
    assert_eq!(h.value("g1"), 0, "Stop is note mode");
}

#[test]
fn a_knob_sets_its_parameter_through_the_host() {
    let mut h = Harness::new(with("303_accent0"));
    let before = h.value("d1c3");
    let gestures = *h.host.gestures.lock().unwrap();
    let c = Harness::center("tb303_1.cutoff");
    h.move_to(c);
    h.button(true);
    for k in 1..=10 {
        h.move_to(c - Vec2::new(0.0, 4.0 * k as f32));
    }
    h.button(false);
    let after = h.value("d1c3");
    assert!(after > before, "cutoff {before} -> {after}");
    assert_eq!(
        h.app.panel.knob("tb303_1.cutoff"),
        orb_ui::behavior::knob_from_engine(after),
        "the panel sent what it shows"
    );
    assert!(
        *h.host.gestures.lock().unwrap() > gestures,
        "the host saw the gesture"
    );
    // the engine plays the parameter, and the panel keeps the value it set
    h.settle();
    h.frame(vec![]);
    assert_eq!(
        h.app.panel.knob("tb303_1.cutoff"),
        orb_ui::behavior::knob_from_engine(after)
    );
}

#[test]
fn a_pattern_button_sets_the_pattern_parameter() {
    let mut h = Harness::new(with("303_accent0"));
    assert_eq!(h.value("d1c1"), 0);
    h.click(Harness::center("pattern1.pattern3"));
    assert_eq!(h.value("d1c1"), 2, "A3");
}

#[test]
fn the_hidden_controls_take_no_clicks() {
    let mut h = Harness::new(with("303_accent0"));
    let before = h.params.values();
    for id in PLUGIN_HIDDEN {
        h.click(Harness::center(id));
    }
    assert_eq!(h.params.values(), before, "no parameter changed");
    assert!(!h.app.panel.transport.recording);
    assert!(!h.app.panel.transport.song_mode);
}

#[test]
fn the_editor_takes_no_keys() {
    let mut h = Harness::new(with("303_accent0"));
    let before = h.params.values();
    let key = |key, modifiers| Event::Key {
        key,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers,
    };
    use egui::Key::*;
    // Space is Play/Stop, the letters and digits the panel's keys in the standalone app,
    // Cmd/Ctrl+O, +N, +F and +G its shortcuts
    let mut events = vec![key(Space, Modifiers::NONE)];
    for k in [A, Q, W, R, Z, X, Num1, Num5, ArrowUp, ArrowDown, Tab, Enter] {
        events.push(key(k, Modifiers::NONE));
    }
    for k in [O, N, F, G, S] {
        events.push(key(k, Modifiers::COMMAND));
    }
    for e in events {
        h.frame(vec![e.clone()]);
        if let Event::Key { key, modifiers, .. } = e {
            h.frame(vec![Event::Key {
                key,
                physical_key: None,
                pressed: false,
                repeat: false,
                modifiers,
            }]);
        }
    }
    assert_eq!(h.params.values(), before, "no parameter changed");
    assert!(!h.app.browsing(), "no file browser");
    assert!(h.app.panel.events.is_empty(), "the panel did nothing");
    assert_eq!(h.app.panel.focus, 0, "the focus did not move");
}

#[test]
fn the_leds_follow_the_snapshot_flags() {
    let p = with("303_accent0");
    common::set(&p, "g1", 1);
    let mut h = Harness::new(p);
    h.process(&[]);
    h.frame(vec![]);
    assert_eq!(
        h.app.plugin_leds(),
        Some((false, false)),
        "host stopped, no MIDI"
    );
    // the host plays: orb follows its position (pattern mode)
    h.playing = true;
    h.process(&[]);
    h.process(&[]);
    h.frame(vec![]);
    assert_eq!(h.app.plugin_leds().map(|l| l.0), Some(true), "Sync lit");
    // a message nothing maps (channel 10) still lights the MIDI LED, for a moment
    h.process(&[HostEvent {
        frame: 0,
        message: Message::NoteOn {
            channel: 9,
            note: 60,
            velocity: 100,
        },
    }]);
    h.frame(vec![]);
    assert_eq!(h.app.plugin_leds().map(|l| l.1), Some(true), "MIDI lit");
    for _ in 0..10 {
        h.frame(vec![]);
    }
    assert_eq!(
        h.app.plugin_leds().map(|l| l.1),
        Some(false),
        "MIDI out again"
    );
    // note mode: no sync
    common::set(&h.params, "g1", 0);
    h.process(&[]);
    h.frame(vec![]);
    assert_eq!(
        h.app.plugin_leds().map(|l| l.0),
        Some(false),
        "Sync off in note mode"
    );
}

#[test]
fn file_open_loads_the_song_into_the_plugin_in_pattern_mode() {
    let mut h = Harness::new(OrbParams::new());
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("song.rbs");
    let bytes = song_bytes("303_accent127");
    std::fs::write(&path, &bytes).unwrap();
    // what the in-app browser does with the file chosen
    h.app.open_song(&path);
    h.frame(vec![]);
    assert!(h.app.errors.is_empty(), "{:?}", h.app.errors);
    assert_eq!(
        h.params.song().as_deref(),
        Some(&bytes[..]),
        "the state keeps the file"
    );
    assert_eq!(h.value("g1"), 1, "File > Open switches to pattern mode");
    // the controls are the song's
    let want = common::with_song("303_accent127");
    assert_eq!(h.value("d1c7"), want.param("d1c7").unwrap().value());
    assert_eq!(
        h.app.panel.knob("tb303_1.accent"),
        orb_ui::behavior::knob_from_engine(h.value("d1c7"))
    );
    // File > New: a new song, the mode kept
    h.app.new_song();
    h.frame(vec![]);
    assert_eq!(h.params.song(), None);
    assert!(!h.app.prompting(), "nothing asks to save in a plugin");
}

#[test]
fn a_mod_from_the_mods_menu_goes_into_the_state() {
    let mut h = Harness::new(OrbParams::new());
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.rbm");
    std::fs::write(&path, test_mod()).unwrap();
    h.app.use_kit(UiKit::Mod(path.clone()));
    h.frame(vec![]);
    assert!(h.app.errors.is_empty(), "{:?}", h.app.errors);
    match h.params.kit() {
        Kit::Mod { name, .. } => assert_eq!(name, "Test Kit"),
        k => panic!("{k:?}"),
    }
    assert_eq!(h.app.doc.mod_name, "Test Kit");
    // the editor opened again names the mod's file
    let gui = GuiContext::new(h.host.clone());
    let app = editor_app(h.params.clone(), gui, settings());
    assert_eq!(app.doc.kit, UiKit::Mod(path));
}

#[test]
fn the_menu_bar_is_above_the_panel() {
    let mut h = Harness::new(OrbParams::new());
    assert!(!egui::Popup::is_any_open(&h.ctx));
    // the first title, File, in the bar
    h.click(Pos2::new(14.0, MENU_H / 2.0));
    h.frame(vec![]);
    assert!(egui::Popup::is_any_open(&h.ctx), "the File menu opened");
}

#[test]
fn the_bar_shows_the_host_position_while_orb_follows_it() {
    let p = with("303_accent0");
    common::set(&p, "g1", 1);
    let mut h = Harness::new(p);
    h.frame(vec![]);
    assert_eq!(h.app.plugin_position(), None, "host stopped: dark");
    // the host plays from bar 3 beat 2 (quarter note 9)
    h.playing = true;
    h.beats = 9.0;
    h.process(&[]);
    h.process(&[]);
    h.frame(vec![]);
    assert_eq!(h.app.plugin_position(), Some(3));
    // note mode: dark
    common::set(&h.params, "g1", 0);
    h.process(&[]);
    h.frame(vec![]);
    assert_eq!(h.app.plugin_position(), None);
}

/// The InnerSize the last frame asked for.
fn inner_size(h: &Harness) -> Option<Vec2> {
    h.commands.iter().find_map(|c| match c {
        ViewportCommand::InnerSize(s) => Some(*s),
        _ => None,
    })
}

#[test]
fn the_corner_scales_the_panel_under_the_menu_bar() {
    let mut h = Harness::new(OrbParams::new());
    let corner = Pos2::new(PANEL_W - CORNER / 2.0, PANEL_H + MENU_H - CORNER / 2.0);
    h.move_to(corner);
    h.button(true);
    assert_eq!(h.cursor, egui::CursorIcon::ResizeNwSe);
    h.move_to(corner + Vec2::new(300.0, 20.0));
    let size = inner_size(&h).expect("a resize request");
    // the larger of the two scales: (624 + 300) / 624; the bar keeps its height
    let w = PANEL_W + 300.0;
    assert_eq!(size, Vec2::new(w, (w * PANEL_H / PANEL_W).round() + MENU_H));
    h.button(false);
}

#[test]
fn the_window_is_the_bar_and_the_panel_at_any_size() {
    let mut h = Harness::new(OrbParams::new());
    h.size = Vec2::new(PANEL_W * 1.5, window_height(PANEL_W * 1.5));
    h.frame(vec![]);
    // a click on a pattern button at the larger scale, under the unscaled bar, hits it
    let c = Harness::center("pattern1.pattern2");
    h.click(Pos2::new(c.x * 1.5, MENU_H + (c.y - MENU_H) * 1.5));
    assert_eq!(h.value("d1c1"), 1, "A2");
}

#[test]
fn the_size_is_kept_in_the_state_and_the_host_gets_sizes_of_the_rule() {
    let p = Arc::new(OrbParams::new());
    let e = OrbEditor::new(p.clone());
    assert_eq!(e.logical_size(), DEFAULT_SIZE);
    assert_eq!(DEFAULT_SIZE, (624, 510 + MENU_H as u32));
    p.set_editor_size((936, 787));
    assert_eq!(e.logical_size(), (936, 787));
    let hint = editor::resize_hint();
    assert!(hint.can_resize && !hint.preserve_aspect_ratio);
    // the host widens the window: the height follows; it heightens it: the width follows
    let cur = (624.0, 532.0);
    let (w, h) = adjust_logical((1000.0, 540.0), cur);
    assert_eq!((w, h), (1000.0, window_height(1000.0) as f64));
    let (w, h) = adjust_logical((630.0, 1042.0), cur);
    assert_eq!(w, ((1042.0 - MENU_H as f64) * 624.0 / 510.0).round());
    assert_eq!(h, window_height(w as f32) as f64);
    // never below half the panel
    assert_eq!(
        adjust_logical((10.0, 10.0), cur),
        (312.0, 255.0 + MENU_H as f64)
    );
    // the state keeps the size
    let json = p.main().doc.to_json();
    assert!(json.contains("\"editor_size\":[936,787]"), "{json}");
}

/// Host callbacks that record the sizes asked for and, like Reaper, size the window at once
/// from inside `request_resize`: the window is the harness behind a `RefCell`, which each
/// frame holds borrowed as egui-baseview 0.9 holds its window state (`window.rs` 584-592).
/// The host's set_size resizes the window and draws it (egui-baseview's `resized`, which
/// borrows that state again, `window.rs` 736, then the next frame): it needs the borrow, and
/// records whether it got it (egui-baseview panics when it does not).
struct Asked {
    sizes: Rc<RefCell<Vec<(f64, f64)>>>,
    window: Rc<RefCell<Option<Harness>>>,
    set_size_ok: Rc<RefCell<Vec<bool>>>,
}

impl HostCallbacks for Asked {
    fn request_resize(
        &mut self,
        new_size: Size,
        scale_factor: f64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let l = new_size.to_logical::<f64>(scale_factor);
        self.sizes.borrow_mut().push((l.width, l.height));
        // the host's synchronous set_size: the window takes the size and draws at it
        let ok = match self.window.try_borrow_mut() {
            Ok(mut w) => {
                let h = w.as_mut().expect("the window");
                h.size = Vec2::new(l.width as f32, l.height as f32);
                h.frame(vec![]);
                true
            }
            Err(_) => false,
        };
        self.set_size_ok.borrow_mut().push(ok);
        Ok(())
    }

    fn destroyed(&mut self) {}
}

/// The tests' post of a route's flush: nothing (the test flushes by hand, after the frame).
fn post_nothing(_: u64) -> bool {
    true
}

#[test]
fn the_corner_asks_the_host_after_the_frame_through_its_callbacks() {
    let mut h = Harness::new(OrbParams::new());
    let sizes = Rc::new(RefCell::new(Vec::new()));
    let window: Rc<RefCell<Option<Harness>>> = Rc::new(RefCell::new(None));
    let set_size_ok = Rc::new(RefCell::new(Vec::new()));
    let host = Asked {
        sizes: sizes.clone(),
        window: window.clone(),
        set_size_ok: set_size_ok.clone(),
    };
    // as `OrbEditor::spawn` does with nice-plug's host callbacks
    let (route, _proxy) =
        HostRoute::register_with(Box::new(host), Arc::new(Mutex::new(1.0)), post_nothing);
    h.app = editor_app_routed(
        h.params.clone(),
        GuiContext::new(h.host.clone()),
        settings(),
        Some(route.clone()),
    );
    *window.borrow_mut() = Some(h);
    // each frame runs with the window state borrowed, as in egui-baseview
    let frame = |f: &dyn Fn(&mut Harness)| {
        let mut state = window.borrow_mut();
        f(state.as_mut().unwrap());
    };
    frame(&|h| h.frame(vec![]));
    let corner = Pos2::new(PANEL_W - CORNER / 2.0, PANEL_H + MENU_H - CORNER / 2.0);
    frame(&|h| h.move_to(corner));
    frame(&|h| h.button(true));
    frame(&|h| h.move_to(corner + Vec2::new(100.0, 0.0)));
    assert!(
        sizes.borrow().is_empty(),
        "nothing is asked inside the frame"
    );
    frame(&|h| {
        assert_eq!(
            inner_size(h),
            None,
            "the window waits for the host to size it"
        )
    });
    // the posted callback, after the frame
    assert_eq!(route.flush(), Some(true));
    let w = (PANEL_W + 100.0) as f64;
    let size = (w, window_height(w as f32) as f64);
    assert_eq!(
        *sizes.borrow(),
        vec![size],
        "the host was asked once for the size of the rule"
    );
    assert_eq!(
        *set_size_ok.borrow(),
        [true],
        "the host's set_size from inside the request finds the window free"
    );
    frame(&|h| {
        assert_eq!(
            h.size,
            Vec2::new(size.0 as f32, size.1 as f32),
            "the window took it"
        )
    });
    // the drag goes on at the size the host gave: nothing new to ask
    frame(&|h| h.frame(vec![]));
    assert_eq!(route.flush(), None);
    assert_eq!(sizes.borrow().len(), 1);
    frame(&|h| h.button(false));
}
