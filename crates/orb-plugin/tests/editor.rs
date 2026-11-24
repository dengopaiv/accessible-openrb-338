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
use orb_plugin::clip;
use orb_plugin::core::{Core, HostEvent, HostTransport};
use orb_plugin::drag::Pending;
use orb_plugin::editor::{
    self, DEFAULT_SIZE, HostRoute, OrbEditor, adjust_logical, editor_app, editor_app_routed,
    editor_app_with,
};
use orb_plugin::instance::{self, Kit, Mode, OrbParams};
use orb_ui::app::App;
use orb_ui::embed::{
    CORNER, DRAG_OUT, Embed, MARGIN_X, MARGIN_Y, MENU_H, MODE_DRAG_STEP, PLUGIN_HIDDEN,
    PLUGIN_MODE_KNOB, PLUGIN_MODE_LED_X, PLUGIN_MODE_ROWS, PluginFormat, PluginMode, corner_rect,
    on_corner, window_height,
};
use orb_ui::layout::{self, PANEL_H, PANEL_W};
use orb_ui::menu::Cmd;
use orb_ui::pattern_editor::Flag;
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
    /// The texts drawn in the last frame.
    texts: Vec<String>,
    /// What the frames put on the system clipboard (`OutputCommand::CopyText`, which
    /// egui-baseview writes to the clipboard).
    copied: Vec<String>,
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
            size: Vec2::new(DEFAULT_SIZE.0 as f32, DEFAULT_SIZE.1 as f32),
            commands: Vec::new(),
            cursor: egui::CursorIcon::Default,
            texts: Vec::new(),
            copied: Vec::new(),
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
        for c in &out.platform_output.commands {
            if let egui::OutputCommand::CopyText(t) = c {
                self.copied.push(t.clone());
            }
        }
        self.texts = out
            .shapes
            .iter()
            .filter_map(|s| match &s.shape {
                egui::Shape::Text(t) => Some(t.galley.text().to_string()),
                _ => None,
            })
            .collect();
        self.commands = out
            .viewport_output
            .values()
            .flat_map(|v| v.commands.clone())
            .collect();
    }

    /// The center of a control (the window is the menu bar and the panel at 1:1 with its
    /// margins).
    fn center(id: &str) -> Pos2 {
        let c = layout::controls().into_iter().find(|c| c.id == id).unwrap();
        at(c.x + c.w / 2.0, c.y + c.h / 2.0)
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

    /// Clicks along the menu bar from its left end until a menu with `item` is open (the
    /// titles' places follow their texts); the x clicked.
    fn open_menu_with(&mut self, item: &str) -> f32 {
        for x in (6..300).step_by(4) {
            egui::Popup::close_all(&self.ctx);
            self.click(Pos2::new(x as f32, MENU_H / 2.0));
            self.frame(vec![]);
            self.frame(vec![]);
            if egui::Popup::is_any_open(&self.ctx) && self.texts.iter().any(|t| t == item) {
                return x as f32;
            }
        }
        panic!("no menu has {item}");
    }

    fn value(&self, id: &str) -> i32 {
        self.params.param(id).unwrap().value()
    }
}

/// An instance with the diag song `name` loaded and its controls set, in note mode.
fn with(name: &str) -> OrbParams {
    let p = common::with_song(name);
    p.set_mode(Mode::Note);
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
fn play_and_stop_enter_and_leave_the_preview() {
    let mut h = Harness::new(with("303_accent0"));
    let values = h.params.values();
    let gestures = *h.host.gestures.lock().unwrap();
    assert!(!h.params.previewing());
    h.click(Harness::center("transport.play"));
    assert!(h.params.previewing(), "Play enters the preview");
    h.settle();
    h.frame(vec![]);
    assert!(
        h.app.panel.transport.playing,
        "the Play light follows the engine"
    );
    h.click(Harness::center("transport.stop"));
    assert!(!h.params.previewing(), "Stop leaves it");
    assert_eq!(h.params.mode(), Mode::Note, "the mode is not touched");
    assert_eq!(h.params.values(), values, "no parameter changed");
    assert_eq!(*h.host.gestures.lock().unwrap(), gestures, "no gesture");
    h.settle();
    h.frame(vec![]);
    assert!(!h.app.panel.transport.playing, "note mode: stopped");
}

/// task-217 AC #2: in the preview the panel's knobs and pattern buttons go to the preview set,
/// not to the host; Stop shows the host's values again, Play the set's.
#[test]
fn preview_edits_go_to_the_preview_set_without_host_gestures() {
    let mut h = Harness::new(with("303_accent0"));
    h.click(Harness::center("transport.play"));
    h.settle();
    h.frame(vec![]);
    let values = h.params.values();
    let gestures = *h.host.gestures.lock().unwrap();
    let before = h.value("d1c3");
    let c = Harness::center("tb303_1.cutoff");
    h.move_to(c);
    h.button(true);
    for k in 1..=10 {
        h.move_to(c - Vec2::new(0.0, 4.0 * k as f32));
    }
    h.button(false);
    h.click(Harness::center("pattern1.pattern3"));
    assert_eq!(*h.host.gestures.lock().unwrap(), gestures, "no gesture");
    assert_eq!(h.params.values(), values, "no host parameter changed");
    let set = h.params.preview_values().unwrap();
    let i = |id: &str| h.params.ids().iter().position(|x| x == id).unwrap();
    let cutoff = set[i("d1c3")];
    assert!(cutoff > before, "cutoff {before} -> {cutoff} in the set");
    assert_eq!(set[i("d1c1")], 2, "A3 in the set");
    h.settle();
    h.frame(vec![]);
    let knob = |h: &Harness| h.app.panel.knob("tb303_1.cutoff");
    assert_eq!(knob(&h), orb_ui::behavior::knob_from_engine(cutoff));
    // Stop: the panel follows the engine back to the host's value
    h.click(Harness::center("transport.stop"));
    h.settle();
    h.frame(vec![]);
    assert_eq!(knob(&h), orb_ui::behavior::knob_from_engine(before));
    // Play: the set again
    h.click(Harness::center("transport.play"));
    h.settle();
    h.frame(vec![]);
    assert_eq!(knob(&h), orb_ui::behavior::knob_from_engine(cutoff));
}

/// The user's Reaper check (2026-11-16): the host's start and stop do not change Play and
/// Stop; the Play light shows only the preview.
#[test]
fn the_play_light_is_the_preview_not_the_hosts_transport() {
    let p = with("303_accent0");
    p.set_mode(Mode::Latch);
    let mut h = Harness::new(p);
    h.playing = true;
    h.settle();
    h.frame(vec![]);
    assert!(
        h.core.audio().last_snapshot().playing,
        "orb follows the host"
    );
    assert!(!h.app.panel.transport.playing, "Play is not lit");
    h.click(Harness::center("transport.play"));
    h.settle();
    h.frame(vec![]);
    assert!(h.app.panel.transport.playing, "the preview lights Play");
    // the host stops: the preview and its light stay
    h.playing = false;
    h.settle();
    h.frame(vec![]);
    assert!(h.app.panel.transport.playing);
    h.click(Harness::center("transport.stop"));
    assert!(!h.params.previewing());
    h.playing = true;
    h.settle();
    h.frame(vec![]);
    assert!(
        !h.app.panel.transport.playing,
        "the host plays: Play stays dark"
    );
}

/// task-231 (the user's decision of 2026-11-17): the Options menu has no mode items; the
/// panel's selector is the only place to change the mode.
#[test]
fn the_options_menu_has_no_mode_items() {
    let mut h = Harness::new(with("303_accent0"));
    // the Options menu (File, Edit, Mods, Options, Help)
    h.open_menu_with("Vintage ReBirth Synth Sound");
    let texts = h.texts.join("|");
    assert!(texts.contains("Vintage ReBirth Synth Sound"), "{texts}");
    assert!(texts.contains("Play Mod Startup Sound"), "{texts}");
    for gone in ["Latch Pattern Mode", "Gate Pattern Mode", "Note Mode"] {
        assert!(!texts.contains(gone), "{gone} in {texts}");
    }
}

/// task-267: the themes are the standalone app's only. The plugin's bar has no View menu, its
/// Mods menu no Theme Viewer (removed), and the View > Classic command does nothing in the plugin:
/// the editor keeps the modern skin.
#[test]
fn the_plugin_has_no_themes() {
    let mut h = Harness::new(with("303_accent0"));
    h.frame(vec![]);
    let bar = h.texts.join("|");
    assert!(bar.contains("Mods") && !bar.contains("View"), "{bar}");
    // the Mods menu (File, Edit, Mods, Options, Help)
    h.open_menu_with("Choose file...");
    let texts = h.texts.join("|");
    assert!(texts.contains("Refresh List"), "the Mods menu: {texts}");
    assert!(!texts.contains("Theme Viewer"), "{texts}");
    let skin = h.app.skin().clone();
    h.app.command(orb_ui::menu::Cmd::SetTheme(
        orb_ui::settings::Theme::Classic,
    ));
    h.frame(vec![]);
    assert_eq!(h.app.theme(), orb_ui::settings::Theme::Modern);
    assert!(std::sync::Arc::ptr_eq(h.app.skin(), &skin));
}

/// The Edit menu of the plugin: its item `text` clicked (the menu opened first).
fn click_edit_item(h: &mut Harness, text: &str) {
    h.open_menu_with("Cut Pattern");
    let (r, enabled) = h.app.edit_item(text).expect("drawn");
    assert!(enabled, "{text}");
    h.click(r.center());
    h.frame(vec![]);
}

/// task-274, the user's review, round 3: the plugin's Edit menu has every item of the
/// standalone's, in its order and with its names, and no shortcut texts.
#[test]
fn the_edit_menu_has_the_standalones_items_without_shortcuts() {
    let mut h = Harness::new(with("303_accent0"));
    h.open_menu_with("Cut Pattern");
    let texts = h.texts.join("|");
    let items = [
        "Undo",
        "Cut Pattern",
        "Copy Pattern",
        "Paste Pattern",
        "Duplicate Pattern",
        "Clear Pattern",
        "303-1 Pattern Editor...",
        "303-2 Pattern Editor...",
        "808 Pattern Editor...",
        "909 Pattern Editor...",
    ];
    let ys: Vec<f32> = items
        .iter()
        .map(|t| h.app.edit_item(t).unwrap_or_else(|| panic!("{t}")).0.top())
        .collect();
    assert!(ys.windows(2).all(|w| w[0] < w[1]), "in order: {ys:?}");
    assert_eq!(h.app.edit_item("Undo").map(|i| i.1), Some(false));
    for key in ["Z", "X", "C", "V", "D", "E"] {
        for m in ["Ctrl+", "Cmd+"] {
            let k = format!("{m}{key}");
            assert!(!h.texts.contains(&k), "{k} in {texts}");
        }
    }
}

/// task-274, round 3: Copy, Paste, Cut, Duplicate and Undo of the plugin's Edit menu work as
/// the standalone's. Copy puts the pattern's JSON on the system clipboard (egui-baseview
/// writes `CopyText` to it); Paste reads the clipboard directly, as the plugin's window does
/// not answer egui's paste request, and asks for none.
#[test]
fn the_edit_menus_pattern_commands_work_in_the_plugin() {
    let mut h = Harness::new(with("303_accent0"));
    let pattern = |h: &Harness, p: u8| h.app.link.as_ref().unwrap().pattern(0, p).to_vec();
    let a1 = pattern(&h, 0);
    assert!(
        !orb_ui::link::is_empty_pattern(0, &a1),
        "the song's A1 has steps"
    );
    h.app.panel.focus = 0;
    click_edit_item(&mut h, "Copy Pattern");
    let json = h.app.clipboard.read().expect("copied");
    assert_eq!(orb_ui::pattern_json::parse(&json).unwrap().bytes, a1);
    assert_eq!(
        h.copied.last(),
        Some(&json),
        "the JSON on the system clipboard"
    );
    assert!(json.contains("\"device\": \"303\""));
    // A3 selected; the paste
    h.app.panel.press("pattern1.pattern3", 0.0);
    h.frame(vec![]);
    let a3 = pattern(&h, 2);
    click_edit_item(&mut h, "Paste Pattern");
    assert_eq!(pattern(&h, 2), a1);
    assert!(
        !h.commands
            .iter()
            .any(|c| matches!(c, ViewportCommand::RequestPaste)),
        "{:?}",
        h.commands
    );
    assert_eq!(h.app.undo_name(), Some("Paste Pattern"));
    click_edit_item(&mut h, "Undo");
    assert_eq!(pattern(&h, 2), a3);
    // Cut empties A1 and Undo brings it back
    h.app.panel.press("pattern1.pattern1", 0.0);
    h.frame(vec![]);
    click_edit_item(&mut h, "Cut Pattern");
    assert!(orb_ui::link::is_empty_pattern(0, &pattern(&h, 0)));
    click_edit_item(&mut h, "Undo");
    assert_eq!(pattern(&h, 0), a1);
    // Duplicate into the next empty pattern, which is selected
    click_edit_item(&mut h, "Duplicate Pattern");
    let to = h.app.panel.rows[0].pattern;
    assert_ne!(to, 0);
    assert_eq!(pattern(&h, to), a1);
}

/// task-274: the plugin's Edit menu opens the 303 pattern editors (no shortcut text); each
/// item opens its 303's window (task-290) on that 303's pattern, an edit in it goes into the
/// plugin's song, and Undo (the user's review, round 2) names the change and takes it back.
#[test]
fn the_edit_menu_opens_the_303_pattern_editors_and_undoes_their_changes() {
    for k in 0..2 {
        the_edit_menu_opens_a_303_pattern_editor(k);
    }
}

fn the_edit_menu_opens_a_303_pattern_editor(k: usize) {
    let mut h = Harness::new(with("303_accent0"));
    let item = orb_ui::pattern_editor::menu_item(k);
    h.open_menu_with(item);
    assert_eq!(h.app.edit_item("Undo").map(|i| i.1), Some(false));
    let (r, enabled) = h.app.edit_item(item).expect("drawn");
    assert!(enabled);
    h.click(r.center());
    for _ in 0..3 {
        h.frame(vec![]);
    }
    let e = h.app.pattern_editor(k);
    assert!(e.open);
    assert_eq!(e.synth, k);
    assert!(
        !h.app.pattern_editor(1 - k).open,
        "the other 303's stays closed"
    );
    assert_eq!(h.app.focused_editor(), Some(k));
    let p = e.pattern;
    let before = h.app.link.as_ref().unwrap().pattern(k, p).to_vec();
    let cell = h.app.pattern_editor_flag_cell(k, Flag::Accent, 15).unwrap();
    h.click(cell.center());
    let after = h.app.link.as_ref().unwrap().pattern(k, p).to_vec();
    assert_ne!(after, before, "the click edits the pattern");
    assert_eq!(
        orb_ui::pattern_editor::flag(&after, 15, Flag::Accent),
        !orb_ui::pattern_editor::flag(&before, 15, Flag::Accent)
    );
    // Edit > Undo Edit Pattern puts it back
    h.open_menu_with("Undo Edit Pattern");
    let (r, enabled) = h.app.edit_item("Undo").expect("drawn");
    assert!(enabled);
    h.click(r.center());
    h.frame(vec![]);
    assert_eq!(h.app.link.as_ref().unwrap().pattern(k, p), &before[..]);
    assert_eq!(h.app.undo_name(), None);
}

/// task-280: the plugin's Edit menu opens the 808 pattern editor (no shortcut text, with the
/// TR-808 focused too); a click in it edits the plugin's song, and Undo takes it back.
#[test]
fn the_edit_menu_opens_the_808_pattern_editor_and_undoes_its_changes() {
    use orb_ui::drum_editor::{Machine, cell};
    let m = Machine::Tr808;
    let mut h = Harness::new(with("303_accent0"));
    h.app.panel.focus = 2;
    h.open_menu_with("808 Pattern Editor...");
    for k in ["Ctrl+E", "Cmd+E"] {
        assert!(!h.texts.iter().any(|t| t == k), "{k}");
    }
    let (r, enabled) = h.app.edit_item("808 Pattern Editor...").expect("drawn");
    assert!(enabled);
    h.click(r.center());
    for _ in 0..3 {
        h.frame(vec![]);
    }
    let e = h.app.drum_editor(m);
    assert!(e.open);
    let p = e.pattern;
    assert_eq!(p, h.app.panel.rows[2].pattern);
    let before = h.app.link.as_ref().unwrap().pattern(2, p).to_vec();
    let c = h.app.drum_editor_cell(m, 1, 15).unwrap();
    h.click(c.center());
    let after = h.app.link.as_ref().unwrap().pattern(2, p).to_vec();
    assert_eq!(cell(&after, 15, 1), 1 - cell(&before, 15, 1).min(1));
    h.open_menu_with("Undo Edit Pattern");
    let (r, enabled) = h.app.edit_item("Undo").expect("drawn");
    assert!(enabled);
    h.click(r.center());
    h.frame(vec![]);
    assert_eq!(h.app.link.as_ref().unwrap().pattern(2, p), &before[..]);
}

/// task-281: the plugin's Edit menu opens the 909 pattern editor (no shortcut text, with the
/// TR-909 focused too; the plugin takes no keys, task-201, so the menu is its way in); its
/// entry switch and a click edit the plugin's song, and Undo takes it back.
#[test]
fn the_edit_menu_opens_the_909_pattern_editor_and_undoes_its_changes() {
    use orb_ui::drum_editor::{Entry, Machine, cell};
    let m = Machine::Tr909;
    let mut h = Harness::new(with("303_accent0"));
    h.app.panel.focus = 3;
    h.open_menu_with("909 Pattern Editor...");
    for k in ["Ctrl+E", "Cmd+E"] {
        assert!(!h.texts.iter().any(|t| t == k), "{k}");
    }
    let (r, enabled) = h.app.edit_item("909 Pattern Editor...").expect("drawn");
    assert!(enabled);
    h.click(r.center());
    for _ in 0..3 {
        h.frame(vec![]);
    }
    let e = h.app.drum_editor(m);
    assert!(e.open);
    assert!(!h.app.drum_editor(Machine::Tr808).open);
    let p = e.pattern;
    assert_eq!(p, h.app.panel.rows[3].pattern);
    let before = h.app.link.as_ref().unwrap().pattern(3, p).to_vec();
    // the switch on Flam, then a click on BD of step 16
    let flam = h.app.drum_editor_hit(m, "Flam").expect("the switch");
    h.click(flam.center());
    assert_eq!(h.app.drum_editor(m).entry, Entry::Flam);
    let c = h.app.drum_editor_cell(m, 1, 15).unwrap();
    h.click(c.center());
    let after = h.app.link.as_ref().unwrap().pattern(3, p).to_vec();
    let was = cell(&before, 15, 1);
    assert_eq!(cell(&after, 15, 1), if was.min(3) == 3 { 0 } else { 3 });
    h.open_menu_with("Undo Edit Pattern");
    let (r, enabled) = h.app.edit_item("Undo").expect("drawn");
    assert!(enabled);
    h.click(r.center());
    h.frame(vec![]);
    assert_eq!(h.app.link.as_ref().unwrap().pattern(3, p), &before[..]);
}

/// task-267: the 808's mod window is no link (the Theme Viewer is gone): no hand cursor.
#[test]
fn the_mod_window_is_no_link_in_the_plugin() {
    let mut h = Harness::new(with("303_accent0"));
    let p = Harness::center("tr808.mod_view");
    h.move_to(p);
    h.frame(vec![]);
    assert_ne!(h.cursor, egui::CursorIcon::PointingHand);
}

/// task-221: the plugin's Help menu has Plugin Usage (and About, no Keyboard Shortcuts); the
/// item opens the guide, a scrollable window that shows the trademark terms, and its close
/// button closes it. The standalone's Help menu has no Plugin Usage (orb-ui's tests).
#[test]
fn the_help_menu_opens_the_plugin_usage_page() {
    let mut h = Harness::new(with("303_accent0"));
    // the Help menu, last in the bar (File, Edit, Mods, Options, Help)
    let x = h.open_menu_with("About OpenRB-338...");
    let texts = h.texts.join("|");
    assert!(texts.contains("Plugin Usage..."), "{texts}");
    assert!(texts.contains("About OpenRB-338..."), "{texts}");
    assert!(!texts.contains("Keyboard Shortcuts"), "{texts}");
    assert!(!h.app.plugin_usage_is_open());

    // the item: the first of the menu, under the title
    h.click(Pos2::new(x + 40.0, MENU_H + 14.0));
    for _ in 0..30 {
        h.frame(vec![]);
    }
    assert!(h.app.plugin_usage_is_open(), "Plugin Usage opened");
    let texts = h.texts.join("|");
    assert!(texts.contains("Plugin Usage"), "{texts}");
    assert!(
        texts.contains("VST\u{ae} 3"),
        "the registered sign at the first VST: {texts}"
    );
    // the trademark sentence is at the bottom of the page (scrolled out of the first screen)
    let rect = h
        .ctx
        .memory(|m| m.area_rect(h.app.plugin_usage_id()))
        .unwrap();
    let screen = Rect::from_min_size(Pos2::ZERO, h.size);
    assert!(screen.contains_rect(rect), "{rect:?} in {screen:?}");

    // the window closes with the mouse: its title bar's close button
    h.click(Pos2::new(rect.right() - 12.0, rect.top() + 12.0));
    h.frame(vec![]);
    assert!(!h.app.plugin_usage_is_open(), "the close button closes it");
}

/// task-221: the page works at the smallest window the host may give (half the panel).
#[test]
fn the_plugin_usage_page_fits_the_smallest_window() {
    let mut h = Harness::new(with("303_accent0"));
    h.size = Vec2::new(312.0, 255.0 + MENU_H);
    h.app.command(orb_ui::menu::Cmd::PluginUsage);
    for _ in 0..30 {
        h.frame(vec![]);
    }
    assert!(h.app.plugin_usage_is_open());
    let rect = h
        .ctx
        .memory(|m| m.area_rect(h.app.plugin_usage_id()))
        .unwrap();
    let screen = Rect::from_min_size(Pos2::ZERO, h.size);
    assert!(screen.contains_rect(rect), "{rect:?} in {screen:?}");
    assert!(rect.width() >= 200.0 && rect.height() >= 150.0, "{rect:?}");
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
    p.set_mode(Mode::Latch);
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
    // the preview (task-217): Sync and Bar dark while the host plays; the MIDI LED still
    // blinks on input, which the preview ignores
    h.click(Harness::center("transport.play"));
    h.process(&[]);
    h.frame(vec![]);
    assert_eq!(h.app.plugin_leds().map(|l| l.0), Some(false), "Sync dark");
    assert_eq!(h.app.plugin_position(), None, "Bar dark");
    h.process(&[HostEvent {
        frame: 0,
        message: Message::NoteOn {
            channel: 0,
            note: 5,
            velocity: 100,
        },
    }]);
    h.frame(vec![]);
    assert_eq!(h.app.plugin_leds().map(|l| l.1), Some(true), "MIDI lit");
    h.click(Harness::center("transport.stop"));
    h.process(&[]);
    h.process(&[]);
    h.frame(vec![]);
    assert_eq!(h.app.plugin_leds().map(|l| l.0), Some(true), "Sync again");
    // note mode: no sync
    h.params.set_mode(Mode::Note);
    h.process(&[]);
    h.frame(vec![]);
    assert_eq!(
        h.app.plugin_leds().map(|l| l.0),
        Some(false),
        "Sync off in note mode"
    );
}

#[test]
fn file_open_and_new_keep_the_mode() {
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
    assert_eq!(h.params.mode(), Mode::Note, "File > Open keeps Note Mode");
    // every mode stays (task-228)
    for m in [Mode::Latch, Mode::Gate, Mode::Note] {
        h.params.set_mode(m);
        h.app.open_song(&path);
        h.frame(vec![]);
        assert_eq!(h.params.mode(), m, "File > Open keeps {m:?}");
    }
    // the controls are the song's
    let want = common::with_song("303_accent127");
    assert_eq!(h.value("d1c7"), want.param("d1c7").unwrap().value());
    assert_eq!(
        h.app.panel.knob("tb303_1.accent"),
        orb_ui::behavior::knob_from_engine(h.value("d1c7"))
    );
    // File > New: a new song, the mode kept
    for m in [Mode::Latch, Mode::Gate, Mode::Note] {
        h.params.set_mode(m);
        h.app.new_song();
        h.frame(vec![]);
        assert_eq!(h.params.mode(), m, "File > New keeps {m:?}");
    }
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
    p.set_mode(Mode::Latch);
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
    h.params.set_mode(Mode::Note);
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
    let corner = at(PANEL_W - CORNER / 2.0, PANEL_H - CORNER / 2.0);
    h.move_to(corner);
    h.button(true);
    assert_eq!(h.cursor, egui::CursorIcon::ResizeNwSe);
    h.move_to(corner + Vec2::new(300.0, 20.0));
    let size = inner_size(&h).expect("a resize request");
    // the larger of the two scales: (624 + 300) / 624; the bar and the margins keep their
    // size: the panel's exact fit
    let w = PANEL_W + 300.0;
    let g = 2.0 * MARGIN_X;
    assert_eq!(
        size,
        Vec2::new(w + g, (w * PANEL_H / PANEL_W).round() + MENU_H)
    );
    assert_eq!(size.y, window_height(size.x));
    h.button(false);
}

/// task-234: the grip is on the panel's corner, MARGIN_X (16 points) in from the window's
/// right edge, so the window's corner (where the host's own grip is) does not resize.
#[test]
fn the_grip_is_on_the_panels_corner_clear_of_the_windows() {
    let mut h = Harness::new(OrbParams::new());
    let win = Pos2::new(DEFAULT_SIZE.0 as f32, DEFAULT_SIZE.1 as f32);
    let panel = Rect::from_min_size(at(0.0, 0.0), Vec2::new(PANEL_W, PANEL_H));
    assert_eq!(panel.max, win - Vec2::new(MARGIN_X, MARGIN_Y));
    // the grip's area ends 16 points left of the window's right edge
    assert_eq!(win.x - corner_rect(panel).max.x, 16.0);
    assert!(on_corner(panel, panel.max - Vec2::splat(1.0)));
    assert!(!on_corner(panel, win - Vec2::splat(1.0)));
    // the window's corner: no resize
    h.move_to(win - Vec2::splat(2.0));
    h.button(true);
    h.move_to(win + Vec2::new(100.0, 50.0));
    assert_eq!(inner_size(&h), None, "the window's corner is the host's");
    h.button(false);
    // the panel's corner: a resize
    h.move_to(panel.max - Vec2::splat(3.0));
    h.button(true);
    assert_eq!(h.cursor, egui::CursorIcon::ResizeNwSe);
    h.move_to(panel.max + Vec2::new(100.0, 50.0));
    assert!(inner_size(&h).is_some(), "the grip resizes");
    h.button(false);
}

#[test]
fn the_window_is_the_bar_and_the_panel_at_any_size() {
    let mut h = Harness::new(OrbParams::new());
    let w = 1.5 * PANEL_W + 2.0 * MARGIN_X;
    h.size = Vec2::new(w, window_height(w));
    h.frame(vec![]);
    // a click on a pattern button at the larger scale, under the unscaled bar, hits it
    let c = Harness::center("pattern1.pattern2") - at(0.0, 0.0);
    h.click(at(0.0, 0.0) + c * 1.5);
    assert_eq!(h.value("d1c1"), 1, "A2");
}

/// task-234 review: a host that calls `set_size` with no `adjust_size` may make the window
/// shorter than the menu bar; the panel's area is then inverted, and the fit keeps a positive,
/// finite scale (a tiny panel), so frames and clicks still run.
#[test]
fn a_window_shorter_than_the_menu_bar_still_draws() {
    use orb_ui::app::View;
    for (w, h) in [(10.0, 10.0), (0.0, 0.0), (300.0, MENU_H), (1.0, 500.0)] {
        let avail = Rect::from_min_max(Pos2::new(0.0, MENU_H), Pos2::new(w, h));
        let view = View::fit(avail, 2.0);
        assert!(
            view.scale > 0.0 && view.scale.is_finite(),
            "{w}x{h}: {view:?}"
        );
        let p = view.to_panel(Pos2::new(w / 2.0, h / 2.0));
        assert!(p.x.is_finite() && p.y.is_finite(), "{w}x{h}: {p:?}");
        let mut hh = Harness::new(OrbParams::new());
        hh.size = Vec2::new(w, h);
        hh.frame(vec![]);
        hh.click(Pos2::new(w / 2.0, h / 2.0));
        hh.frame(vec![]);
    }
}

/// task-234: the host may give the editor any size; the panel is letterboxed under the menu
/// bar (the largest panel-shaped area within the margins, centered). REAPER keeps its
/// window at the size it asked for, e.g. 721 x 532: the panel fits inside it whole (no crop),
/// at least MARGIN_X from the left and right edges. A click on the letterboxed panel hits the
/// control under it, in a wide and in a tall window.
#[test]
fn the_panel_is_letterboxed_in_a_window_of_any_shape() {
    use orb_ui::app::View;
    let m = MARGIN_X;
    for (w, h, dx, dy, scale) in [
        // wide: the height decides, bands left and right
        (1100.0, 532.0, (1100.0 - PANEL_W) / 2.0, 0.0, 1.0),
        // REAPER's own size after a right-edge drag
        (721.0, 532.0, (721.0 - PANEL_W) / 2.0, 0.0, 1.0),
        // tall: the width decides, bands above and below
        (656.0, 800.0, m, (800.0 - MENU_H - PANEL_H) / 2.0, 1.0),
        // larger and wide: scale 2 by the height
        (
            1600.0,
            1020.0 + MENU_H,
            (1600.0 - 2.0 * PANEL_W) / 2.0,
            0.0,
            2.0,
        ),
    ] {
        let win = Rect::from_min_size(Pos2::ZERO, Vec2::new(w, h));
        let view = View::fit(orb_ui::embed::panel_area(win), 1.0);
        let panel = view.rect(0.0, 0.0, PANEL_W, PANEL_H);
        assert!(
            win.shrink2(Vec2::new(m - 0.5, -0.5)).contains_rect(panel),
            "{w}x{h}: the panel {panel:?} is inside the window's margins"
        );
        assert_eq!(view.scale, scale, "{w}x{h}");
        assert_eq!(view.origin, Pos2::new(dx, MENU_H + dy).round(), "{w}x{h}");
        let mut hh = Harness::new(OrbParams::new());
        hh.size = Vec2::new(w, h);
        hh.frame(vec![]);
        let c = Harness::center("pattern1.pattern2") - at(0.0, 0.0);
        hh.click(view.origin + c * scale);
        assert_eq!(hh.value("d1c1"), 1, "{w}x{h}: A2");
    }
}

#[test]
fn the_size_is_kept_in_the_state_and_the_host_gets_sizes_of_the_rule() {
    let p = Arc::new(OrbParams::new());
    let e = OrbEditor::new(p.clone());
    assert_eq!(e.logical_size(), DEFAULT_SIZE);
    assert_eq!(DEFAULT_SIZE, (624 + 32, 510 + MENU_H as u32));
    assert_eq!(
        window_height(DEFAULT_SIZE.0 as f32),
        DEFAULT_SIZE.1 as f32,
        "the default is the panel's exact fit at 1:1"
    );
    p.set_editor_size((936, 787));
    assert_eq!(e.logical_size(), (936, 787));
    // the host may resize the editor both ways, with no aspect ratio
    // (task-234); the editor's hint is the window's
    let hint = editor::resize_hint();
    assert!(hint.can_resize && !hint.preserve_aspect_ratio);
    assert!(hint.can_resize_horizontally && hint.can_resize_vertically);
    assert!(Editor::resize_hint(&e).can_resize);
    // the host's size is taken as it is, at least the minimum (half the panel and the bar)
    for s in [
        (624.0, 532.0),
        (721.0, 532.0),
        (1000.0, 540.0),
        (630.0, 1042.0),
        (312.0, 277.0),
        (2000.5, 900.25),
    ] {
        assert_eq!(adjust_logical(s), s, "{s:?}");
    }
    assert_eq!(adjust_logical((10.0, 10.0)), (312.0, 255.0 + MENU_H as f64));
    assert_eq!(adjust_logical((500.0, 100.0)), (500.0, 277.0));
    assert_eq!(adjust_logical((f64::NAN, -5.0)), (312.0, 277.0));
    // Live's (VST3) corner drags: the cursor's size asked on each mouse
    // event and the answer made the window's size. The answer is the request, so it follows
    // the cursor; the rule before task-234 (the side that changed more relative to the
    // window's current size decides) went back on the same drag.
    let changed = |new: (f64, f64), cur: (f64, f64)| -> (f64, f64) {
        let (pw, ph, m) = (PANEL_W as f64, PANEL_H as f64, MENU_H as f64);
        let dw = (new.0 - cur.0).abs() / cur.0.max(1.0);
        let dh = (new.1 - cur.1).abs() / cur.1.max(1.0);
        let w = if dh > dw {
            (new.1 - m) * pw / ph
        } else {
            new.0
        };
        let w = w.max(pw / 2.0).round();
        (w, window_height(w as f32) as f64)
    };
    let drag: Vec<(f64, f64)> = (1..=47)
        .map(|i| (624.0 + 8.0 * i as f64, (532.0 + 4.0 * i as f64).min(700.0)))
        .collect();
    let mut cur = (624.0, 532.0);
    let mut old = Vec::new();
    for &c in &drag {
        cur = changed(c, cur);
        old.push(cur.0);
    }
    assert!(
        old.windows(2).any(|p| p[1] < p[0]),
        "the old rule goes back on a growing drag: {old:?}"
    );
    for &c in &drag {
        assert_eq!(adjust_logical(c), c, "the answer is the request");
    }
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
    let corner = at(PANEL_W - CORNER / 2.0, PANEL_H - CORNER / 2.0);
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
    let w = (DEFAULT_SIZE.0 + 100) as f64;
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

/// The panel's resize corner is the one way to resize (task-234): a drag off the rule's
/// diagonal, growing then shrinking, through the host that sizes the window from inside the
/// request (as Reaper does; Live does it with `onSize`). Each size asked for is of the rule,
/// the host's check of it (`checkSizeConstraint`/`gui.adjust_size`, [`adjust_logical`]) gives
/// it back, and the widths follow the drag without going back.
#[test]
fn the_corner_drag_resizes_steadily_through_the_host() {
    let mut h = Harness::new(OrbParams::new());
    let sizes = Rc::new(RefCell::new(Vec::new()));
    let window: Rc<RefCell<Option<Harness>>> = Rc::new(RefCell::new(None));
    let set_size_ok = Rc::new(RefCell::new(Vec::new()));
    let host = Asked {
        sizes: sizes.clone(),
        window: window.clone(),
        set_size_ok: set_size_ok.clone(),
    };
    let (route, _proxy) =
        HostRoute::register_with(Box::new(host), Arc::new(Mutex::new(1.0)), post_nothing);
    h.app = editor_app_routed(
        h.params.clone(),
        GuiContext::new(h.host.clone()),
        settings(),
        Some(route.clone()),
    );
    *window.borrow_mut() = Some(h);
    let frame = |f: &dyn Fn(&mut Harness)| {
        let mut state = window.borrow_mut();
        f(state.as_mut().unwrap());
    };
    frame(&|h| h.frame(vec![]));
    let corner = at(PANEL_W - CORNER / 2.0, PANEL_H - CORNER / 2.0);
    frame(&|h| h.move_to(corner));
    frame(&|h| h.button(true));
    let mut p = corner;
    let mut turn = 0;
    for i in 0..90 {
        p += if i < 30 {
            Vec2::new(8.0, 4.0)
        } else {
            Vec2::new(-8.0, -3.0)
        };
        frame(&|h| h.move_to(p));
        // the posted callback, after the frame
        route.flush();
        if i == 29 {
            turn = sizes.borrow().len();
        }
    }
    frame(&|h| h.button(false));
    let sizes = sizes.borrow();
    assert!(
        sizes.len() > 40,
        "the host was asked along the drag: {sizes:?}"
    );
    assert!(set_size_ok.borrow().iter().all(|&ok| ok));
    for &(w, h) in sizes.iter() {
        assert_eq!(h, window_height(w as f32) as f64, "{w}x{h} is of the rule");
        assert_eq!(
            adjust_logical((w, h)),
            (w, h),
            "the host's check of {w}x{h}"
        );
    }
    for (i, pair) in sizes.windows(2).enumerate() {
        let ok = if i + 1 < turn {
            pair[1].0 > pair[0].0
        } else if i + 1 > turn {
            pair[1].0 < pair[0].0
        } else {
            true
        };
        assert!(ok, "the width went back at {i}: {sizes:?}");
    }
    assert_eq!(
        window.borrow().as_ref().unwrap().size,
        Vec2::new(
            sizes.last().unwrap().0 as f32,
            sizes.last().unwrap().1 as f32
        ),
        "the window has the last size"
    );
}

/// Where a test host finds the editor's proxy.
type ProxySlot = Rc<RefCell<Option<Box<dyn HostCallbacks>>>>;

/// A host that refuses every request and counts them; with `nested`, its request asks the
/// proxy again, as a host answering with `set_size` makes baseview do.
struct Refusing {
    asked: Rc<std::cell::Cell<u32>>,
    nested: Option<ProxySlot>,
}

impl HostCallbacks for Refusing {
    fn request_resize(&mut self, size: Size, scale: f64) -> Result<(), Box<dyn std::error::Error>> {
        self.asked.set(self.asked.get() + 1);
        if let Some(p) = &self.nested {
            let mut p = p.borrow_mut();
            let p = p.as_mut().expect("the proxy");
            assert!(
                p.request_resize(size, scale).is_ok(),
                "a request inside a request is not refused"
            );
        }
        Err("refused".into())
    }

    fn destroyed(&mut self) {}
}

/// baseview's own requests (task-224): not asked of the host, and so never refused, before the
/// window is parented and during the host's `set_size`; asked after, the refusal passed on.
#[test]
fn the_host_is_not_asked_for_sizes_before_the_parent_or_its_own() {
    let asked = Rc::new(std::cell::Cell::new(0));
    let host = Refusing {
        asked: asked.clone(),
        nested: None,
    };
    let (route, mut proxy) =
        HostRoute::register_with(Box::new(host), Arc::new(Mutex::new(1.0)), post_nothing);
    let size = Size::Logical(nice_plug::editor::dpi::LogicalSize::new(700.0, 594.0));
    assert!(
        proxy.request_resize(size, 2.0).is_ok(),
        "no parent: no host window"
    );
    assert_eq!(asked.get(), 0);
    route.set_parented();
    let r = route.host_sizing(|| proxy.request_resize(size, 2.0));
    assert!(r.is_ok(), "the host's own size");
    assert_eq!(asked.get(), 0);
    let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        route.host_sizing(|| panic!("set_size panics"))
    }));
    assert!(r.is_err());
    assert!(
        proxy.request_resize(size, 2.0).is_err(),
        "parented: asked, refused (also after a set_size that panicked)"
    );
    assert_eq!(asked.get(), 1);

    // baseview's request made while the host answers the corner's
    let asked = Rc::new(std::cell::Cell::new(0));
    let slot: ProxySlot = Rc::new(RefCell::new(None));
    let (route, proxy) = HostRoute::register_with(
        Box::new(Refusing {
            asked: asked.clone(),
            nested: Some(slot.clone()),
        }),
        Arc::new(Mutex::new(1.0)),
        post_nothing,
    );
    route.set_parented();
    *slot.borrow_mut() = Some(proxy);
    assert_eq!(route.request(700.0, 594.0), Some(true));
    assert_eq!(route.flush(), Some(false), "the corner's request, refused");
    assert_eq!(asked.get(), 1, "asked once; the nested request was not");
}

/// A harness whose editor has the pattern drag-out, and where it leaves its clip.
fn with_drag(params: OrbParams) -> (Harness, Pending) {
    let mut h = Harness::new(params);
    let pending = Pending::default();
    h.app = editor_app_with(
        h.params.clone(),
        GuiContext::new(h.host.clone()),
        settings(),
        None,
        Some(pending.clone()),
    );
    h.frame(vec![]);
    (h, pending)
}

/// task-203: a press on a pattern button that moves drags the pattern out once, as the clip of
/// the plugin's pattern; the press selects the pattern as a click does.
#[test]
fn a_pattern_button_dragged_out_gives_its_clip() {
    let (mut h, pending) = with_drag(with("303_accent0"));
    assert_eq!(h.value("d1c1"), 0);
    let at = Harness::center("pattern1.pattern3");
    h.move_to(at);
    h.button(true);
    assert_eq!(h.value("d1c1"), 2, "the press selects A3");
    // less than DRAG_OUT: still a click
    h.move_to(at + Vec2::new(DRAG_OUT - 1.0, 0.0));
    assert!(pending.take().is_none());
    h.move_to(at + Vec2::new(DRAG_OUT, 0.0));
    let path = pending.take().expect("the drag-out's clip");
    assert_eq!(path.file_name().unwrap(), "303-1 A3.mid");
    assert!(path.starts_with(clip::clips_dir()));
    // the clip of the plugin's pattern A3 with the values it plays
    let values = h.params.panel_values();
    let patterns = h.params.view(&values).1;
    let want = clip::clip(
        0,
        2,
        &patterns[0][2 * clip::LEN_303..3 * clip::LEN_303],
        &editor::clip_controls(&values),
    );
    assert_eq!(std::fs::read(&path).unwrap(), want);
    // once per press
    h.move_to(at + Vec2::new(60.0, 40.0));
    assert!(pending.take().is_none());
    h.button(false);
    let _ = std::fs::remove_dir_all(path.parent().unwrap());
    // the next press drags again: the 909's bank B, button 1
    h.click(Harness::center("pattern4.bankB"));
    let at = Harness::center("pattern4.pattern1");
    h.move_to(at);
    h.button(true);
    h.move_to(at + Vec2::new(0.0, -20.0));
    h.button(false);
    let path = pending.take().expect("the second drag-out's clip");
    assert_eq!(path.file_name().unwrap(), "909 B1.mid");
    let f = std::fs::read(&path).unwrap();
    assert_eq!(&f[..4], b"MThd");
    let _ = std::fs::remove_dir_all(path.parent().unwrap());
}

/// task-203 AC #3: a click without movement selects the pattern and drags nothing; nor does a
/// bank button, nor an editor without the drag-out.
#[test]
fn a_click_on_a_pattern_button_still_selects_it() {
    let (mut h, pending) = with_drag(with("303_accent0"));
    h.click(Harness::center("pattern1.pattern3"));
    assert_eq!(h.value("d1c1"), 2, "A3");
    assert!(pending.take().is_none(), "a click drags nothing");
    let at = Harness::center("pattern1.bankC");
    h.move_to(at);
    h.button(true);
    h.move_to(at + Vec2::new(30.0, 0.0));
    h.button(false);
    assert!(pending.take().is_none(), "a bank button drags nothing");
    // the editor of `editor_app` has no drag-out: a drag is a click
    let mut h = Harness::new(with("303_accent0"));
    let at = Harness::center("pattern1.pattern5");
    h.move_to(at);
    h.button(true);
    h.move_to(at + Vec2::new(30.0, 0.0));
    h.button(false);
    assert_eq!(h.value("d1c1"), 4, "A5");
}

/// The center of the mode selector's row `i` (0 Latch, 1 Gate, 2 Note) in the window: its label.
fn mode_row(i: usize) -> Pos2 {
    at(PLUGIN_MODE_LED_X + 30.0, PLUGIN_MODE_ROWS[i])
}

fn mode_knob() -> Pos2 {
    at(PLUGIN_MODE_KNOB.0, PLUGIN_MODE_KNOB.1)
}

/// The window's point of the panel's point (x, y) at the default size: the panel at 1:1 under
/// the menu bar, [`MARGIN_X`] in from the window's left edge ([`MARGIN_Y`] below the bar).
fn at(x: f32, y: f32) -> Pos2 {
    Pos2::new(MARGIN_X + x, MENU_H + MARGIN_Y + y)
}

/// task-231 AC 2: a click on a row of the mode selector sets the mode (`Embed::set_mode`, the
/// plugin's `OrbParams::mode`), and the selector shows the mode the plugin has.
#[test]
fn a_click_on_a_row_of_the_mode_selector_sets_the_mode() {
    let mut h = Harness::new(with("303_accent0"));
    assert_eq!(h.app.embed().unwrap().mode(), Some(PluginMode::Note));
    for (i, (mode, want)) in [
        (PluginMode::Latch, Mode::Latch),
        (PluginMode::Gate, Mode::Gate),
        (PluginMode::Note, Mode::Note),
    ]
    .into_iter()
    .enumerate()
    {
        // from another mode so that a click changes it
        h.app.command(Cmd::SetPluginMode(match mode {
            PluginMode::Latch => PluginMode::Note,
            _ => PluginMode::Latch,
        }));
        h.click(mode_row(i));
        assert_eq!(h.params.mode(), want, "row {i}");
        assert_eq!(h.app.embed().unwrap().mode(), Some(mode), "row {i}");
        // the rows' order
        assert_eq!(orb_ui::embed::MODE_ITEMS[i].0, mode);
    }
    // a click on the row of the mode it is in changes nothing
    h.click(mode_row(2));
    assert_eq!(h.params.mode(), Mode::Note);
}

/// task-231: a vertical drag on the knob steps through the modes (up toward Latch), a click on
/// it does nothing.
#[test]
fn a_drag_on_the_mode_knob_steps_the_modes() {
    let mut h = Harness::new(with("303_accent0"));
    h.click(mode_knob());
    assert_eq!(h.params.mode(), Mode::Note, "a click does nothing");
    let at = mode_knob();
    h.move_to(at);
    h.button(true);
    h.move_to(at + Vec2::new(0.0, -MODE_DRAG_STEP));
    assert_eq!(h.params.mode(), Mode::Gate);
    h.move_to(at + Vec2::new(0.0, -2.0 * MODE_DRAG_STEP));
    assert_eq!(h.params.mode(), Mode::Latch);
    h.move_to(at + Vec2::new(0.0, -80.0));
    assert_eq!(h.params.mode(), Mode::Latch, "stops at the end");
    h.move_to(at + Vec2::new(0.0, -MODE_DRAG_STEP));
    assert_eq!(
        h.params.mode(),
        Mode::Gate,
        "the drag goes back with the pointer"
    );
    h.button(false);
    assert_eq!(h.params.mode(), Mode::Gate);
}

/// task-231 AC #3: the mode chosen on the selector is saved with the state.
#[test]
fn the_mode_from_the_selector_is_saved_with_the_state() {
    let mut h = Harness::new(with("303_accent0"));
    h.click(mode_row(1));
    assert_eq!(h.params.mode(), Mode::Gate);
    let fields = h.params.serialize_fields();
    let b = OrbParams::new();
    b.deserialize_fields(&fields);
    assert_eq!(b.mode(), Mode::Gate);
}

/// task-231: the editor tells its plugin format (the logo) from the wrapper's context.
#[test]
fn the_editor_knows_its_plugin_format() {
    let mut h = Harness::new(with("303_accent0"));
    assert_eq!(h.app.embed().unwrap().format(), Some(PluginFormat::Vst3));
}

/// task-231: the whole line of a row is a hit area, from the knob's edge (x 14 right of the
/// knob's center is on the line of the Latch row, beyond the collar), and the knob's drag
/// area ends where the rows begin.
#[test]
fn a_click_on_a_rows_line_selects_it() {
    let mut h = Harness::new(with("303_accent0"));
    let near = |row: usize| at(PLUGIN_MODE_KNOB.0 + 14.0, PLUGIN_MODE_ROWS[row]);
    h.click(near(0));
    assert_eq!(h.params.mode(), Mode::Latch);
    h.click(near(1));
    assert_eq!(h.params.mode(), Mode::Gate);
    h.click(near(2));
    assert_eq!(h.params.mode(), Mode::Note);
    // a drag starting there is no knob drag
    h.move_to(near(2));
    h.button(true);
    h.move_to(near(2) + Vec2::new(0.0, -2.0 * MODE_DRAG_STEP));
    h.button(false);
    assert_eq!(h.params.mode(), Mode::Note);
}

/// task-231: with the file browser or the error box open the rows and the knob take no hit.
/// (A prompt, `App::prompt`, is crate-private: not reachable from here.)
#[test]
fn the_mode_selector_takes_no_hit_under_the_browser_or_the_error_box() {
    let mut h = Harness::new(with("303_accent0"));
    h.app.fail("an error");
    h.frame(vec![]);
    h.click(mode_row(0));
    assert_eq!(h.params.mode(), Mode::Note, "the error box is open");
    let at = mode_knob();
    h.move_to(at);
    h.button(true);
    h.move_to(at + Vec2::new(0.0, -2.0 * MODE_DRAG_STEP));
    h.button(false);
    assert_eq!(h.params.mode(), Mode::Note, "the error box is open");

    let mut h = Harness::new(with("303_accent0"));
    h.app.open_browser(orb_ui::browser::Mode::OpenSong);
    h.frame(vec![]);
    h.click(mode_row(0));
    assert_eq!(h.params.mode(), Mode::Note, "the browser is open");
}

/// A stand-in host that reports `PluginApi` and does nothing else.
struct ApiHost(PluginApi);

impl GuiContextInner for ApiHost {
    fn plugin_api(&self) -> PluginApi {
        self.0
    }
    unsafe fn raw_begin_set_parameter(&self, _param: ParamPtr) {}
    unsafe fn raw_set_parameter_normalized(&self, _param: ParamPtr, _normalized: f32) {}
    unsafe fn raw_end_set_parameter(&self, _param: ParamPtr) {}
    fn get_state(&self) -> PluginState {
        unimplemented!()
    }
    fn set_state(&self, _state: PluginState) {}
    fn request_restart(&self) {}
}

/// task-231: the logo's format is the wrapper's: VST3, and none in a standalone.
#[test]
fn the_format_follows_the_wrappers_plugin_api() {
    for (api, want) in [
        (PluginApi::Vst3, Some(PluginFormat::Vst3)),
        (PluginApi::Standalone, None),
    ] {
        let gui = GuiContext::new(Arc::new(ApiHost(api)));
        let e = editor::EditorEmbed::new(Arc::new(OrbParams::new()), gui);
        assert_eq!(Embed::format(&e), want, "{api:?}");
    }
}

/// task-279: the plugin editor shows the About window with the copy-version button, and a click
/// on it puts the version on the clipboard (`CopyText`, which egui-baseview writes).
#[test]
fn the_about_window_has_a_copy_version_button() {
    let mut h = Harness::new(with("303_accent0"));
    h.app.show_about();
    for _ in 0..6 {
        h.frame(vec![]);
    }
    assert!(h.app.about_is_open());
    let r = orb_ui::about::copy_button_rect(&h.ctx).expect("the button is drawn");
    assert!(h.copied.is_empty());
    h.click(r.center());
    assert_eq!(h.copied, vec![orb_ui::about::VERSION.to_string()]);
}
