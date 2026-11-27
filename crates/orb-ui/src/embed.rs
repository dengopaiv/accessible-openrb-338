//! The panel as the plugin's editor (task-201, upstream's plugin document (removed, task-231)
//! section 10): what the removed plugin crate's editor gives the [`App`] ([`Embed`]) and how the
//! panel differs in a plugin window.
//!
//! The plugin's panel is the standalone's, drawn by the same code, except:
//! - the transport has no Pattern/Song switch, Rewind, Fast forward, Record, bar arrows or loop,
//!   and the tempo display has no arrows ([`PLUGIN_HIDDEN`]; their keys do nothing); its background
//!   is `bg_transport_plugin`, with a Bar display of the host position ([`host_position`]; no Beat
//!   display, the user's decision of 2026-11-16);
//! - the menu bar is the standalone's, above the panel and not scaled with it ([`MENU_H`] points);
//!   File has New, Open and Info (no Save, Save As or Export), and nothing asks to save: the song
//!   lives in the host's project;
//! - the window is the menu bar and, under it, the panel letterboxed on the standalone's desk with
//!   at least [`MARGIN_X`] of desk left and right (task-234): the host may give any size, and the
//!   panel's exact fit is [`window_height`] of the width;
//! - the Sync and MIDI LEDs light from the snapshot's flags (`host_sync`, `midi_in`);
//! - the plugin's mode ([`PluginMode`], [`MODE_ITEMS`], task-217, task-218) is set by the mode
//!   selector of the transport (task-231), not by the Options menu; Play and Stop are the plugin's
//!   preview, and the Play light shows only whether it runs ([`Embed::previewing`]), not the
//!   engine's play state;
//! - a grip on the panel's bottom right corner (the right wood's foot, [`corner_rect`]) resizes the
//!   window: the cursor changes over it, and a drag asks the host for the panel's exact fit
//!   ([`corner_size`]).
//!
//! The song, its mod and the controls are the plugin's ([`Embed`]); the panel's commands go to
//! the plugin through the link's [`crate::link::Port`].

use std::path::{Path, PathBuf};
use std::time::Duration;

use egui::{self, Color32, CursorIcon, Id, Pos2, Rect, Sense, Ui, Vec2};
use orb_cli::mods::{Kit, STANDARD};
use orb_player::Snapshot;

use crate::app::{App, View};
use crate::document::Document;
use crate::layout::{PANEL_H, PANEL_W};
use crate::link::Link;
use crate::menu::Cmd;
pub use crate::menu::MENU_H;
use crate::model::Panel;
use crate::session;
use crate::settings::Settings;

/// What the panel shows of the plugin's song (`Embed::view`).
pub struct SongView {
    /// The song file as loaded; `None`: a new song.
    pub song: Option<Vec<u8>>,
    /// The mod: `None` for the built-in kit, else its INFO name and a file that holds it.
    pub kit: Option<(String, PathBuf)>,
    /// The four pattern arrays as the plugin holds them.
    pub patterns: [Vec<u8>; 4],
    /// The engine's values of the song (a snapshot of it).
    pub snapshot: Snapshot,
}

/// The plugin's side of its editor (the removed plugin crate implements it): the song, the mod and
/// the state that the standalone app keeps by itself.
pub trait Embed: Send {
    /// A number that changes when the plugin's song or state changes (a load, a state restore):
    /// the panel then shows it anew ([`Embed::view`]).
    fn revision(&self) -> u64;
    /// The plugin's song now.
    fn view(&mut self) -> SongView;
    /// File > Open (`Some`, the file's bytes) or File > New (`None`): the plugin's song, with the
    /// plugin's mod; File > Open and File > New keep the mode (task-228).
    fn load_song(&mut self, file: Option<Vec<u8>>) -> Result<(), String>;
    /// The Mods menu or a song's mod: a recrypted mod's file, `None` for the built-in kit. The
    /// plugin keeps the mod's bytes in its state; [`SongView::kit`] names this file while the
    /// editor is open.
    fn set_mod(&mut self, file: Option<&Path>) -> Result<(), String>;
    /// An error the plugin met on its own (a state that did not load), once.
    fn take_error(&mut self) -> Option<String> {
        None
    }
    /// The resize corner: asks the host for a window of `size` points (a
    /// `IPlugFrame::resizeView`); the host then sizes the window.
    /// The plugin makes the request after the frame, not from this call (a host may resize the
    /// window at once, which must not happen inside the frame). `Some(_)` when the request
    /// goes to the host, `None` when the plugin has no way to it (the window then resizes
    /// itself, `ViewportCommand::InnerSize`).
    fn request_size(&mut self, size: Vec2) -> Option<bool> {
        let _ = size;
        None
    }
    /// The plugin's pattern/note mode (task-217), which the panel's mode selector shows and sets;
    /// `None` when the plugin has no such setting.
    fn mode(&self) -> Option<PluginMode> {
        None
    }
    /// The mode selector chose `mode`.
    fn set_mode(&mut self, mode: PluginMode) {
        let _ = mode;
    }
    /// The plugin format the editor runs in (task-231): its logo shows on the transport. `None`
    /// (the default): no logo.
    fn format(&self) -> Option<PluginFormat> {
        None
    }
    /// Whether the plugin's preview runs (task-217): the Play light shows this, not whether
    /// the engine plays, so the host's transport does not change Play and Stop. `None`: the
    /// light follows the engine.
    fn previewing(&self) -> Option<bool> {
        None
    }
    /// A press on pattern button `pattern` (0..31) of section `row` (0..3) moved
    /// [`DRAG_OUT`] points: the plugin drags the pattern out to the host as a MIDI clip
    /// (task-203). Returns whether a drag starts; `false` (the default) when the plugin has
    /// none. Called once per press; the press has selected the pattern already.
    fn drag_pattern(&mut self, row: usize, pattern: u8) -> bool {
        let _ = (row, pattern);
        false
    }
    /// The plugin's output routing (task-300), which Options > Output Routing... shows and
    /// sets; `None` (the default) when the plugin has none, and then the menu has no such item.
    fn routing(&self) -> Option<orb_engine::frame::OutputRouting> {
        None
    }
    /// The routing window changed a box: the plugin's routing is now `r`.
    fn set_routing(&mut self, r: orb_engine::frame::OutputRouting) {
        let _ = r;
    }
}

/// How far (points) a press on a pattern button must move to drag the pattern out
/// ([`Embed::drag_pattern`]); a shorter movement is a click, which selects the pattern only.
/// egui's own click tolerance (`input_options.max_click_dist`) is 6 points too.
pub const DRAG_OUT: f32 = 6.0;

/// The plugin's mode outside its preview (task-217, task-218): what host notes do and when the
/// patterns play.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PluginMode {
    /// The patterns play while the host plays; notes select patterns until the next note.
    Latch,
    /// A section plays its pattern only while its note is held.
    Gate,
    /// Notes play the instruments.
    Note,
}

/// The plugin format of an editor ([`Embed::format`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PluginFormat {
    Vst3,
}

/// The modes of [`PluginMode`] in the order of the mode selector's rows (top to bottom), with
/// their full names.
pub const MODE_ITEMS: [(PluginMode, &str); 3] = [
    (PluginMode::Latch, "Latch Pattern Mode"),
    (PluginMode::Gate, "Gate Pattern Mode"),
    (PluginMode::Note, "Note Mode"),
];

/// The mode selector (task-231) in panel pixels: the art's `PLUGIN_MODE_*` moved by the
/// transport's x (16). The knob's center, its scale, the x of the LEDs and the y of the three
/// rows (Latch, Gate, Note), and the pointer's angle per row (degrees, 0 = up, clockwise).
pub const PLUGIN_MODE_KNOB: (f32, f32) = (16.0 + 250.0, 26.0);
pub const PLUGIN_MODE_SCALE: f32 = 1.6;
/// The knob's collar radius (the art's 7.7 times the scale): the knob's drag area is its square.
pub const PLUGIN_MODE_KNOB_RADIUS: f32 = 7.7 * PLUGIN_MODE_SCALE;
pub const PLUGIN_MODE_LED_X: f32 = 16.0 + 285.0;
pub const PLUGIN_MODE_ROWS: [f32; 3] = [13.0, 26.0, 39.0];
pub const PLUGIN_MODE_ANGLES: [f32; 3] = [45.0, 90.0, 135.0];

/// The plugin format's logo part in panel pixels (x, y, w, h): the art's `PLUGIN_LOGO` moved by
/// the transport's x.
pub const PLUGIN_LOGO: (f32, f32, f32, f32) = (16.0 + 344.0, 0.0, 112.0, 50.0);

/// A vertical drag on the mode knob moves one mode per this many panel pixels.
pub const MODE_DRAG_STEP: f32 = 12.0;

/// The mode index (0 Latch, 1 Gate, 2 Note) a drag of `dy` panel pixels (down is positive)
/// reaches from the mode `start`.
fn drag_mode(start: usize, dy: f32) -> usize {
    (start as f32 + (dy / MODE_DRAG_STEP).round()).clamp(0.0, 2.0) as usize
}

/// The transport controls the plugin's panel does not have (task-185's list, and Fast forward
/// and the bar display, which act only in song mode; task-213 decides on those two).
pub const PLUGIN_HIDDEN: &[&str] = &[
    "transport.mode",
    "transport.rewind",
    "transport.ff",
    "transport.record",
    "transport.bar",
    "transport.bar_up",
    "transport.bar_down",
    "transport.loop",
    "transport.loop_start",
    "transport.loop_start_up",
    "transport.loop_start_down",
    "transport.loop_length",
    "transport.loop_length_up",
    "transport.loop_length_down",
    "transport.tempo_up",
    "transport.tempo_down",
];

/// The Bar display of the plugin's transport in panel pixels (x, y, w, h): the art's
/// `PLUGIN_BAR` moved by the transport's x (16).
pub const PLUGIN_BAR: (f32, f32, f32, f32) = (16.0 + 184.0, 27.0, 27.0, 16.0);

/// The engine ticks of a bar: 768 (`docs/initial_investigation.md` section 5).
const BAR_TICKS: i32 = 768;

/// What the Bar display shows: the host position's bar (from 1) while orb follows it
/// (`host_sync`: pattern mode, orb and the host playing), where the engine's position is the
/// host's (upstream's plugin document (removed, task-231) section 8.2); else nothing, the dark
/// display of the standalone's pattern mode (note mode, or the host stopped: orb free-runs and has
/// no host position). Bars of four quarter notes, whatever the host's time signature (task-206).
pub fn host_position(s: &Snapshot) -> Option<i32> {
    if !s.host_sync {
        return None;
    }
    Some(s.tick.max(0) / BAR_TICKS + 1)
}

/// The Sync and MIDI LEDs (panel pixels): the transport's rectangles of docs/ui_spec.md
/// section 3 moved by 16.
pub const SYNC_LED: (f32, f32, f32, f32) = (55.0, 2.0, 11.0, 11.0);
pub const MIDI_LED: (f32, f32, f32, f32) = (90.0, 2.0, 11.0, 11.0);

/// How long the MIDI LED stays lit after a message, in seconds.
pub const MIDI_FLASH: f64 = 0.1;

/// The side of the resize corner (the grip on the panel's bottom right), in points.
pub const CORNER: f32 = 14.0;

/// The desk left and right of a plugin's panel, in points on each side, not scaled (task-234,
/// the user's choice of 2026-11-17): the grip on the panel's corner stays this far in from the
/// window's right edge, clear of the host's own resize grip at the window's corner.
pub const MARGIN_X: f32 = 16.0;
/// The desk above and below a plugin's panel at its exact fit: none, the panel touches the
/// menu bar and the window's bottom edge.
pub const MARGIN_Y: f32 = 0.0;

/// The smallest scale the corner asks for: half the panel. This floor is separate from the
/// host's minimum window (`layout::MIN_WINDOW_W/H`, 536 x 485, enforced by `adjust_logical`).
pub const MIN_SCALE: f32 = 0.5;

/// While the editor is open, the panel is drawn at least this often, so that the LEDs follow
/// the host's MIDI and transport even when nothing else moves.
pub const POLL: Duration = Duration::from_millis(50);

/// The height of a plugin window `width` points wide in which the panel fits exactly: the menu
/// bar, which does not scale (task-201), the [`MARGIN_Y`] above and below, and the panel at the
/// width less the [`MARGIN_X`] left and right.
pub fn window_height(width: f32) -> f32 {
    ((width - 2.0 * MARGIN_X) * PANEL_H / PANEL_W).round() + MENU_H + 2.0 * MARGIN_Y
}

/// The area a plugin's panel is fitted in (`View::fit`): the window `win` below the menu bar,
/// less [`MARGIN_X`] left and right and [`MARGIN_Y`] above and below.
pub fn panel_area(win: Rect) -> Rect {
    Rect::from_min_max(
        Pos2::new(win.min.x + MARGIN_X, win.min.y + MENU_H + MARGIN_Y),
        win.max - Vec2::new(MARGIN_X, MARGIN_Y),
    )
}

/// The resize corner's area: the bottom right [`CORNER`] of the panel `panel` (its wood's
/// foot, no control there), so [`MARGIN_X`] in from the window's right edge at the exact fit.
pub fn corner_rect(panel: Rect) -> Rect {
    Rect::from_min_max(panel.max - Vec2::splat(CORNER), panel.max)
}

/// The size the corner asks for when a drag reaches `desired` (points): the panel as large as
/// the larger of the two scales the drag gives, at least [`MIN_SCALE`], in whole points, with
/// the margins and the menu bar ([`window_height`]): the panel's exact fit.
pub fn corner_size(desired: Vec2) -> Vec2 {
    let k = ((desired.x - 2.0 * MARGIN_X) / PANEL_W)
        .max((desired.y - MENU_H - 2.0 * MARGIN_Y) / PANEL_H)
        .max(MIN_SCALE);
    let w = (PANEL_W * k).round() + 2.0 * MARGIN_X;
    Vec2::new(w, window_height(w))
}

/// How long a voice's light stays on at least, in seconds (task-202; tuned by eye).
pub const MIN_ON: f64 = 0.1;

/// How often the panel is drawn while a note-mode light is lit or the engine plays (else the
/// 50 ms [`POLL`] carries it: a hit is seen by its count, not by being sampled while it sounds).
pub const LIGHT_POLL: Duration = Duration::from_millis(16);

/// The 808 and the 909 in [`Lit`] and [`Lights`].
pub const LIT_808: usize = 0;
pub const LIT_909: usize = 1;

/// The drum voices that light in the panel in note mode (task-202).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Lit {
    /// Per device (808, 909): the sounds lit, bit `n` as in `orb_engine::tr808::sound` and
    /// `orb_engine::tr909::sound`.
    pub mask: [u32; 2],
    /// Per device: AC is lit, while a lit sound's last hit had accent.
    pub ac: [bool; 2],
}

impl Lit {
    /// Whether anything is lit.
    pub fn any(&self) -> bool {
        self.mask != [0, 0]
    }
}

/// What lights the drum voices in note mode: the engine's sounding mask per block (voice
/// active and gain above -60 dB, `orb_engine::frame::Sounding`) held on for at least
/// [`MIN_ON`] from every hit: the engine counts the hits of each sound
/// (`Sounding::hits`, every trigger, retriggers too, never lost between two snapshots) and a
/// count that changed since the last frame is an onset, which restarts the minimum. So a hit
/// that begins and ends between two frames lights for [`MIN_ON`], and a retrigger of a sounding
/// voice restarts the light. A voice that stops (a choke) goes dark when its minimum is over.
#[derive(Clone, Debug, Default)]
pub struct Lights {
    /// Per device and sound: the time until which the light is held on.
    until: [[f64; 16]; 2],
    /// Per device: the hit counts of the last frame; `None` before the first (the counts at
    /// that time are not hits seen).
    seen: Option<[[u32; 16]; 2]>,
    lit: Lit,
}

impl Lights {
    /// The lights at `now` from the devices' `sounding` (808, 909); `None` when the panel
    /// does not show them (pattern mode, the preview, no snapshot): everything dark.
    pub fn update(&mut self, sounding: Option<[orb_engine::frame::Sounding; 2]>, now: f64) -> Lit {
        let Some(sounding) = sounding else {
            *self = Self::default();
            return self.lit;
        };
        let mut lit = Lit::default();
        let before = self.seen.unwrap_or([sounding[0].hits, sounding[1].hits]);
        let mut seen = [[0; 16]; 2];
        for (d, s) in sounding.iter().enumerate() {
            seen[d] = s.hits;
            for (n, (hits, was)) in s.hits.iter().zip(&before[d]).enumerate() {
                let bit = 1u32 << n;
                if hits != was {
                    self.until[d][n] = now + MIN_ON;
                }
                if s.mask & bit != 0 || now < self.until[d][n] {
                    lit.mask[d] |= bit;
                    if s.ac & bit != 0 {
                        lit.ac[d] = true;
                    }
                }
            }
        }
        self.seen = Some(seen);
        self.lit = lit;
        lit
    }
}

/// The plugin's editor state of the [`App`].
pub(crate) struct Plugin {
    pub(crate) embed: Box<dyn Embed>,
    /// The note-mode lights of the drum voices (task-202).
    pub(crate) lights: Lights,
    /// The revision shown.
    revision: Option<u64>,
    /// The MIDI count last seen and the time until which the LED is lit.
    midi_seen: Option<u64>,
    midi_until: f64,
    /// The window size when a corner drag started.
    drag_from: Option<Vec2>,
    /// Whether the last frame lit the Sync and the MIDI LED.
    leds: (bool, bool),
    /// The host position's bar the last frame showed.
    position: Option<i32>,
    /// The size the corner drag asked for last.
    asked: Option<Vec2>,
}

impl App {
    /// The panel as a plugin's editor: `embed` is the plugin, `link` a link to its port
    /// ([`Link::with_port`]). The panel shows the plugin's song from the first frame.
    pub fn embedded(embed: Box<dyn Embed>, link: Link, settings: Settings) -> App {
        let mut app = App::new(Panel::new(), None);
        app.link = Some(link);
        app.settings = settings;
        // the standalone's menu bar, above the panel
        app.menu = true;
        app.samples = Some(rbm::standard::builtin_samples());
        app.plugin = Some(Plugin {
            embed,
            lights: Lights::default(),
            revision: None,
            midi_seen: None,
            midi_until: f64::NEG_INFINITY,
            drag_from: None,
            leds: (false, false),
            position: None,
            asked: None,
        });
        // the song is the plugin's: no startup sound when the editor opens
        app.settled = true;
        app.refresh_mods();
        app.embed_refresh();
        app
    }

    /// Whether this is a plugin's editor.
    pub fn is_plugin(&self) -> bool {
        self.plugin.is_some()
    }

    /// The plugin, in a plugin's editor.
    pub fn embed(&mut self) -> Option<&mut (dyn Embed + 'static)> {
        self.plugin.as_mut().map(|p| p.embed.as_mut())
    }

    /// Shows the plugin's song when it changed (its revision): the document (File > Info),
    /// the mod, the patterns and the values. Also takes the plugin's own errors into the box.
    pub(crate) fn embed_refresh(&mut self) {
        let Some(p) = &mut self.plugin else {
            return;
        };
        if let Some(e) = p.embed.take_error() {
            self.fail(e);
        }
        let Some(p) = &mut self.plugin else {
            return;
        };
        let rev = p.embed.revision();
        if p.revision == Some(rev) {
            return;
        }
        p.revision = Some(rev);
        let v = p.embed.view();
        let (mod_name, kit) = match v.kit {
            None => (STANDARD.to_string(), Kit::Standard),
            Some((name, path)) => (name, Kit::Mod(path)),
        };
        // the drum samples, for the startup sound of the logo
        let samples = match self.kit_samples(&kit) {
            Ok(s) => s,
            Err(_) => self.samples.clone(),
        };
        let opened = v
            .song
            .as_deref()
            .and_then(|b| session::open_bytes(b, "the plugin's song").ok());
        let (source, facts_1x) = match opened {
            Some(o) => (o.source, o.facts_1x),
            None => (rbs::Song::empty(), None),
        };
        self.doc = Document {
            source,
            facts_1x,
            path: None,
            modified: false,
            mod_name,
            kit,
        };
        self.events_seen = 0;
        if let Some(link) = &mut self.link {
            link.reset_view(v.patterns, &v.snapshot, samples, &mut self.panel);
        }
    }

    /// The plugin takes `kit` (nothing in the standalone app).
    pub(crate) fn embed_mod(&mut self, kit: &Kit) -> Result<(), String> {
        let file = match kit {
            Kit::Standard => None,
            Kit::Mod(p) => Some(p.as_path()),
        };
        match self.embed() {
            Some(e) => e.set_mod(file),
            None => Ok(()),
        }
    }

    /// Whether the last frame lit the Sync and the MIDI LED; `None` in the standalone app.
    pub fn plugin_leds(&self) -> Option<(bool, bool)> {
        self.plugin.as_ref().map(|p| p.leds)
    }

    /// The drum voices lit in note mode (task-202): the plugin's note mode (not its preview,
    /// which plays the patterns) and the 808's and 909's sounding masks of the last block.
    /// Nothing in the standalone app or in pattern mode. The panel is drawn every
    /// [`LIGHT_POLL`] only while a light is lit (a new hit lights, so a count that changed
    /// this frame is covered) or the engine plays; otherwise the [`POLL`] of
    /// [`App::plugin_ui`] carries it and a hit is seen by its count at the next frame.
    pub(crate) fn note_lights(&mut self, ui: &Ui, now: f64) -> Lit {
        let snap = self.link.as_mut().map(|l| {
            let s = l.player().snapshot();
            (s.sounding, s.playing)
        });
        let Some(p) = &mut self.plugin else {
            return Lit::default();
        };
        let note = p.embed.mode() == Some(PluginMode::Note) && p.embed.previewing() != Some(true);
        let sounding = snap.filter(|_| note).map(|(s, _)| [s[2], s[3]]);
        let lit = p.lights.update(sounding, now);
        if lit.any() || (note && snap.is_some_and(|(_, playing)| playing)) {
            ui.ctx().request_repaint_after(LIGHT_POLL);
        }
        lit
    }

    /// Whether the control `id` is hidden in a plugin's panel.
    pub fn hidden(&self, id: &str) -> bool {
        self.is_plugin() && PLUGIN_HIDDEN.contains(&id)
    }

    /// The plugin's own parts of the panel, after the panel: the LEDs, the menus in the
    /// transport and the resize corner. Nothing in the standalone app.
    pub(crate) fn plugin_ui(&mut self, ui: &mut Ui, view: &View, now: f64) {
        if self.plugin.is_none() {
            return;
        }
        self.mode_selector(ui, view);
        let snap = self.link.as_mut().map(|l| *l.player().snapshot());
        let Some(p) = &mut self.plugin else {
            return;
        };
        let (sync, midi) = snap.map_or((false, None), |s| (s.host_sync, Some(s.midi_in)));
        p.position = snap.as_ref().and_then(host_position);
        {
            // 3 cells of 9 x 16, centered in the display, as the panel's displays
            let ((x, y, w, h), n) = (PLUGIN_BAR, 3);
            let x0 = x + (w - 9.0 * n as f32) / 2.0;
            for (i, d) in crate::behavior::digit_frames(p.position, n)
                .into_iter()
                .enumerate()
            {
                let r = view.rect(x0 + 9.0 * i as f32, y, 9.0, h);
                let f = d.map_or("seg_blank".to_string(), |d| format!("seg_{d}"));
                crate::app::paint_part(ui, &f, r);
            }
        }
        if let Some(m) = midi {
            if p.midi_seen.is_some_and(|seen| seen != m) {
                p.midi_until = now + MIDI_FLASH;
            }
            p.midi_seen = Some(m);
        }
        let midi_lit = now < p.midi_until;
        p.leds = (sync, midi_lit);
        let (x, y, w, h) = SYNC_LED;
        if sync {
            crate::app::paint_part(ui, "led_transport_on", view.rect(x, y, w, h));
        }
        let (x, y, w, h) = MIDI_LED;
        if midi_lit {
            crate::app::paint_part(ui, "led_transport_on", view.rect(x, y, w, h));
        }
        ui.ctx().request_repaint_after(POLL);
        self.resize_corner(ui, view);
    }

    /// The mode selector of the plugin's transport (task-231) and the plugin format's logo: the
    /// background has the gray knob's body, the lines and the dark LEDs; here the pointer, the
    /// selected row's LED, the labels (the selected one bold) and the logo. The only way to
    /// set the mode (the Options menu has no mode items), through [`Embed::set_mode`]
    /// ([`Cmd::SetPluginMode`]):
    /// - a click on a row (its line, LED or label) selects its mode;
    /// - a vertical drag on the knob steps through the modes, one per [`MODE_DRAG_STEP`] panel
    ///   pixels, up toward Latch and down toward Note, from the mode the drag started in; a click
    ///   on the knob does nothing;
    /// - no keys (the plugin takes none).
    ///
    /// Nothing is hit while the file browser, a prompt or the error box is open.
    fn mode_selector(&mut self, ui: &mut Ui, view: &View) {
        let Some(p) = &self.plugin else {
            return;
        };
        let mode = p.embed.mode().unwrap_or(PluginMode::Note);
        let current = MODE_ITEMS.iter().position(|(m, _)| *m == mode).unwrap_or(2);
        if let Some(format) = p.embed.format() {
            let name = match format {
                PluginFormat::Vst3 => "plugin_logo_vst3",
            };
            let (x, y, w, h) = PLUGIN_LOGO;
            crate::app::paint_part(ui, name, view.rect(x, y, w, h));
        }
        let k = crate::art::knob_design("knob_gray");
        let (cx, cy) = PLUGIN_MODE_KNOB;
        let s = PLUGIN_MODE_SCALE;
        let knob = view.rect(cx - k.cx * s, cy - k.cy * s, k.w * s, k.h * s);
        // the drag area is the collar's square; the rows start at its right edge
        let rc = PLUGIN_MODE_KNOB_RADIUS;
        let hit = view.rect(cx - rc, cy - rc, 2.0 * rc, 2.0 * rc);
        let pivot = Vec2::new(k.cx / k.w, k.cy / k.h);
        let angle = PLUGIN_MODE_ANGLES[current].to_radians();
        crate::app::paint_part_rotated(ui, "knob_gray_pointer", knob, Some((angle, pivot)));
        let (lx, ly) = (PLUGIN_MODE_LED_X, PLUGIN_MODE_ROWS[current]);
        crate::app::paint_part(ui, "led_mode_on", view.rect(lx - 5.5, ly - 5.5, 11.0, 11.0));
        for (i, y) in PLUGIN_MODE_ROWS.into_iter().enumerate() {
            let state = if i == current { "on" } else { "off" };
            let r = view.rect(lx + 5.5, y - 5.0, 62.0, 10.0);
            crate::app::paint_part(ui, &format!("mode_label_{i}_{state}"), r);
        }
        if self.browsing() || self.prompt.is_some() || self.error_open() {
            return;
        }
        // the rows' hit areas: the whole line, from the collar's edge to the label's end
        let from = cx + rc;
        let mut chosen = None;
        for (i, y) in PLUGIN_MODE_ROWS.into_iter().enumerate() {
            let rect = view.rect(from, y - 6.5, lx + 5.5 + 62.0 - from, 13.0);
            let r = ui.interact(rect, Id::unique(("orb mode row", i)), Sense::click());
            if r.clicked() {
                chosen = Some(i);
            }
        }
        let id = Id::unique("orb mode knob");
        let r = ui.interact(hit, id, Sense::drag());
        // like `knob`: also during a drag that has left the area
        if r.hovered() || r.is_pointer_button_down_on() || r.dragged() {
            ui.ctx().set_cursor_icon(CursorIcon::ResizeVertical);
        }
        if r.drag_started() {
            ui.data_mut(|d| d.insert_temp(id, current));
        }
        if r.dragged()
            && let Some(d) = r.total_drag_delta()
            && let Some(start) = ui.data(|d| d.get_temp::<usize>(id))
        {
            chosen = Some(drag_mode(start, d.y / view.scale));
        }
        if let Some(i) = chosen.filter(|&i| i != current) {
            self.command(Cmd::SetPluginMode(MODE_ITEMS[i].0));
        }
    }

    /// The pattern drag-out (task-203): a press on a pattern button (`c`, a "grid" control
    /// `patternR.patternN`; `r` its response) that moves [`DRAG_OUT`] points or more from where
    /// it went down asks the plugin to drag the button's pattern out, once per press. The press
    /// has selected the pattern at mouse down, as a click does. Nothing in the standalone app,
    /// on a bank button or with the file browser, a prompt or the error box open.
    pub(crate) fn drag_out(&mut self, ui: &Ui, c: &crate::layout::Ctl, r: &egui::Response) {
        if self.plugin.is_none() || self.browsing() || self.prompt.is_some() {
            return;
        }
        let Some((sec, name)) = c.id.split_once('.') else {
            return;
        };
        let (Some(row), Some(button)) = (
            sec.strip_prefix("pattern")
                .and_then(|n| n.parse::<usize>().ok()),
            name.strip_prefix("pattern")
                .and_then(|n| n.parse::<u8>().ok()),
        ) else {
            return;
        };
        if !(1..=4).contains(&row) || !(1..=8).contains(&button) || self.error_open() {
            return;
        }
        let id = Id::unique(("orb plugin drag out", &c.id));
        if !r.is_pointer_button_down_on() {
            ui.data_mut(|d| d.remove::<bool>(id));
            return;
        }
        let moved = ui.input(|i| {
            let p = &i.pointer;
            p.press_origin()
                .zip(p.latest_pos())
                .is_some_and(|(a, b)| a.distance(b) >= DRAG_OUT)
        });
        if !moved || ui.data(|d| d.get_temp::<bool>(id)).is_some() {
            return;
        }
        ui.data_mut(|d| d.insert_temp(id, true));
        let pattern = self.panel.rows[row - 1].bank * 8 + (button - 1);
        if let Some(p) = &mut self.plugin {
            p.embed.drag_pattern(row - 1, pattern);
        }
    }

    /// The bar the Bar display showed in the last frame; `None` when dark or in the standalone
    /// app.
    pub fn plugin_position(&self) -> Option<i32> {
        self.plugin.as_ref().and_then(|p| p.position)
    }

    /// The resize grip on the panel's bottom right corner ([`corner_rect`], [`MARGIN_X`] in from
    /// the window's right edge and so clear of the host's own grip, task-234): always drawn, the
    /// cursor shows it; a drag asks the host for the panel's exact fit ([`corner_size`]).
    fn resize_corner(&mut self, ui: &mut Ui, view: &View) {
        let win = ui.ctx().content_rect();
        let r = corner_rect(view.rect(0.0, 0.0, PANEL_W, PANEL_H));
        let resp = ui.interact(r, Id::unique("orb plugin resize corner"), Sense::drag());
        let active = resp.hovered() || resp.dragged();
        if active {
            ui.ctx().set_cursor_icon(CursorIcon::ResizeNwSe);
        }
        paint_grip(ui, r, active);
        let Some(p) = &mut self.plugin else {
            return;
        };
        if resp.dragged() {
            let from = *p.drag_from.get_or_insert(win.size());
            if let Some(d) = resp.total_drag_delta() {
                let want = corner_size(from + d);
                if (want - win.size()).length() >= 1.0 && p.asked != Some(want) {
                    p.asked = Some(want);
                    // the plugin asks the host, which then sizes the window; without a way
                    // to the host the window resizes itself
                    if p.embed.request_size(want).is_none() {
                        ui.ctx()
                            .send_viewport_cmd(egui::ViewportCommand::InnerSize(want));
                    }
                }
            }
        } else {
            p.drag_from = None;
            p.asked = None;
        }
    }
}

/// The color of the resize corner's grip: a light gray, faint on the wood.
pub const GRIP: Color32 = Color32::from_rgba_premultiplied(150, 150, 150, 150);
/// The grip's color while the pointer is over it or drags it.
pub const GRIP_ACTIVE: Color32 = Color32::from_rgba_premultiplied(230, 230, 230, 230);

/// The resize corner's grip, three diagonal strokes in the corner's rectangle `r`, always
/// shown (task-234), lighter while the pointer is over the corner or drags it (`active`). The
/// resize cursor egui asks for does not always show in a host: baseview sets the cursor only
/// when egui's changes (`CursorManager::set_cursor`), and a host that sets its own cursor on
/// mouse moves (Reaper on macOS, the user's check of 2026-11-16) puts its arrow back while
/// egui's stays the same.
pub(crate) fn paint_grip(ui: &Ui, r: Rect, active: bool) {
    let stroke = egui::Stroke::new(1.0, if active { GRIP_ACTIVE } else { GRIP });
    let p = ui.painter();
    // 2 points in from the wood's right side; at the bottom, down to the wood's foot (the
    // window's bottom edge at the exact fit; half the stroke in, so it is not clipped; the
    // user's check of 2026-11-17)
    let bottom = r.max.y - 0.5;
    for k in [3.0, 6.5, 10.0] {
        let a = egui::pos2(r.max.x - 2.0, bottom - k);
        let b = egui::pos2(r.max.x - 2.0 - k, bottom);
        p.line_segment([a, b], stroke);
    }
}

/// The point `p` is on the resize corner of the panel `panel` (tests).
pub fn on_corner(panel: Rect, p: Pos2) -> bool {
    corner_rect(panel).contains(p)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_corner_scales_the_panel_and_keeps_the_menu_bar() {
        // the bar and the margins (2 x 16 across, none above and below) do not scale
        let (m, g) = (MENU_H, 2.0 * MARGIN_X);
        assert_eq!(MARGIN_Y, 0.0);
        assert_eq!(window_height(624.0 + g), 510.0 + m);
        assert_eq!(
            corner_size(Vec2::new(624.0 + g, 510.0 + m)),
            Vec2::new(624.0 + g, 510.0 + m)
        );
        // the larger scale wins
        assert_eq!(
            corner_size(Vec2::new(1248.0 + g, 600.0)),
            Vec2::new(1248.0 + g, 1020.0 + m)
        );
        assert_eq!(
            corner_size(Vec2::new(700.0, 1020.0 + m)),
            Vec2::new(1248.0 + g, 1020.0 + m)
        );
        // never below half the panel
        assert_eq!(
            corner_size(Vec2::new(10.0, 10.0)),
            Vec2::new(312.0 + g, 255.0 + m)
        );
        // an odd width: the panel's height rounded, the bar added
        assert_eq!(
            window_height(700.0),
            ((700.0 - g) * 510.0 / 624.0).round() + m
        );
        // upstream's plugin document (removed, task-231) section 10: round(1216 x 510 / 624) + 22
        assert_eq!(window_height(1248.0), 1016.0);
        // the exact fit: the panel fills the area within the margins, no band
        for w in [344.0, 656.0, 701.0, 1280.0] {
            let win = Rect::from_min_size(Pos2::ZERO, Vec2::new(w, window_height(w)));
            let area = panel_area(win);
            let v = View::fit(area, 1.0);
            let panel = v.rect(0.0, 0.0, PANEL_W, PANEL_H);
            assert!(
                (panel.width() - area.width()).abs() < 0.01,
                "{w}: {panel:?} {area:?}"
            );
            assert!(
                (panel.height() - area.height()).abs() < 1.0,
                "{w}: {panel:?} {area:?}"
            );
            // the panel touches the bar and the bottom edge
            assert!((panel.min.y - m).abs() < 1.0, "{w}: {panel:?}");
            assert!((win.max.y - panel.max.y).abs() < 1.0, "{w}: {panel:?}");
            // the grip is on the panel, MARGIN_X in from the window's right edge
            let c = corner_rect(panel);
            assert!(panel.contains_rect(c));
            assert!((win.max.x - c.max.x - MARGIN_X).abs() < 1.0, "{w}: {c:?}");
        }
    }

    #[test]
    fn the_host_position_is_the_bar_while_following_the_host() {
        let mut s = Snapshot {
            tick: 768 * 4 + 192 * 2 + 50,
            ..Snapshot::default()
        };
        assert_eq!(host_position(&s), None, "not following the host: dark");
        s.host_sync = true;
        assert_eq!(host_position(&s), Some(5));
        s.tick = 767;
        assert_eq!(host_position(&s), Some(1));
        s.tick = 0;
        assert_eq!(host_position(&s), Some(1));
    }

    /// The 808's sounding: `mask`, `ac` and the hit counts of sounds 0 and 1.
    fn sounding(mask: u32, ac: u32, hits: [u32; 2]) -> Option<[orb_engine::frame::Sounding; 2]> {
        let mut s = orb_engine::frame::Sounding {
            mask,
            ac,
            ..Default::default()
        };
        s.hits[..2].copy_from_slice(&hits);
        Some([s, orb_engine::frame::Sounding::default()])
    }

    fn lit808(l: &mut Lights, s: Option<[orb_engine::frame::Sounding; 2]>, now: f64) -> u32 {
        l.update(s, now).mask[LIT_808]
    }

    #[test]
    fn a_light_stays_on_for_the_minimum_and_follows_the_voice() {
        let mut l = Lights::default();
        assert_eq!(lit808(&mut l, sounding(0, 0, [0, 0]), 0.0), 0);
        // a hit: lit; it ends after 30 ms but the light is held to 100 ms
        assert_eq!(lit808(&mut l, sounding(0b1, 0, [1, 0]), 1.0), 1);
        assert_eq!(lit808(&mut l, sounding(0, 0, [1, 0]), 1.03), 1);
        assert_eq!(lit808(&mut l, sounding(0, 0, [1, 0]), 1.09), 1);
        assert_eq!(lit808(&mut l, sounding(0, 0, [1, 0]), 1.11), 0);
        // a long voice stays lit while it sounds, and dark when it ends
        assert_eq!(lit808(&mut l, sounding(0b1, 0, [2, 0]), 2.0), 1);
        assert_eq!(lit808(&mut l, sounding(0b1, 0, [2, 0]), 2.5), 1);
        assert_eq!(lit808(&mut l, sounding(0, 0, [2, 0]), 2.51), 0);
    }

    #[test]
    fn a_hit_between_two_reads_lights() {
        let mut l = Lights::default();
        lit808(&mut l, sounding(0, 0, [0, 0]), 0.0);
        // the voice began and ended between the frames: the mask is clear, the count moved
        assert_eq!(lit808(&mut l, sounding(0, 0, [1, 0]), 1.0), 1);
        assert_eq!(lit808(&mut l, sounding(0, 0, [1, 0]), 1.09), 1);
        assert_eq!(lit808(&mut l, sounding(0, 0, [1, 0]), 1.11), 0);
        // several hits between the frames count as one onset; another sound stays dark
        assert_eq!(lit808(&mut l, sounding(0, 0, [4, 0]), 2.0), 1);
    }

    #[test]
    fn a_retrigger_of_a_sounding_voice_restarts_the_light() {
        let mut l = Lights::default();
        lit808(&mut l, sounding(0, 0, [0, 0]), 0.0);
        assert_eq!(lit808(&mut l, sounding(0b1, 0, [1, 0]), 1.0), 1);
        // still sounding, hit again at 1.08: the minimum runs to 1.18
        assert_eq!(lit808(&mut l, sounding(0b1, 0, [2, 0]), 1.08), 1);
        assert_eq!(lit808(&mut l, sounding(0, 0, [2, 0]), 1.15), 1);
        assert_eq!(lit808(&mut l, sounding(0, 0, [2, 0]), 1.19), 0);
    }

    #[test]
    fn the_counts_at_the_start_are_not_hits() {
        let mut l = Lights::default();
        assert_eq!(lit808(&mut l, sounding(0, 0, [7, 3]), 0.0), 0);
        assert_eq!(lit808(&mut l, sounding(0, 0, [7, 3]), 0.01), 0);
        // dark in pattern mode and the preview, and the counts are taken anew
        assert!(!l.update(None, 1.0).any());
        assert_eq!(lit808(&mut l, sounding(0, 0, [9, 3]), 2.0), 0);
    }

    #[test]
    fn ac_is_lit_while_a_lit_voice_had_accent() {
        let mut l = Lights::default();
        lit808(&mut l, sounding(0, 0, [0, 0]), 0.0);
        // voice 0 plain, voice 1 accented: AC lit; a mask without the accented voice: dark
        let lit = l.update(sounding(0b11, 0b10, [1, 1]), 1.0);
        assert!(lit.ac[LIT_808]);
        assert!(!lit.ac[LIT_909]);
        let lit = l.update(sounding(0b01, 0b10, [1, 1]), 1.5);
        assert!(
            !lit.ac[LIT_808],
            "voice 1 is dark: its accent no longer counts"
        );
    }

    #[test]
    fn the_hidden_controls_are_in_the_layout() {
        let ids: Vec<String> = crate::layout::controls()
            .into_iter()
            .map(|c| c.id)
            .collect();
        for id in PLUGIN_HIDDEN {
            assert!(ids.iter().any(|i| i == id), "{id}");
        }
        // what the plugin keeps of the transport
        for id in [
            "transport.play",
            "transport.stop",
            "transport.tempo",
            "transport.shuffle",
        ] {
            assert!(!PLUGIN_HIDDEN.contains(&id), "{id}");
        }
    }

    #[test]
    fn the_displays_and_the_leds_match_the_art() {
        let (x, y, w, h) = orb_ui_art::parts::PLUGIN_BAR;
        assert_eq!(PLUGIN_BAR, (16.0 + x, y, w, h));
        let led = crate::art::part("led_transport_on").expect("the lit LED part");
        assert_eq!((led.w, led.h), (SYNC_LED.2, SYNC_LED.3));
        assert!(crate::art::part("bg_transport_plugin").is_some());
    }

    #[test]
    fn the_mode_selector_matches_the_art() {
        use orb_ui_art::parts as art;
        assert_eq!(
            PLUGIN_MODE_KNOB,
            (16.0 + art::PLUGIN_MODE_KNOB.0, art::PLUGIN_MODE_KNOB.1)
        );
        assert_eq!(PLUGIN_MODE_SCALE, art::PLUGIN_MODE_SCALE);
        // the art's collar radius (gray_knob_cap) is 7.7; the lines start at 15.8 from the center
        assert_eq!(PLUGIN_MODE_KNOB_RADIUS, 7.7 * art::PLUGIN_MODE_SCALE);
        const { assert!(PLUGIN_MODE_KNOB_RADIUS < 15.8) };
        assert_eq!(PLUGIN_MODE_LED_X, 16.0 + art::PLUGIN_MODE_LED_X);
        assert_eq!(PLUGIN_MODE_ROWS, art::PLUGIN_MODE_ROWS);
        for (i, (deg, _)) in art::PLUGIN_MODE_LABELS.into_iter().enumerate() {
            assert_eq!(PLUGIN_MODE_ANGLES[i], deg);
        }
        let (x, y, w, h) = art::PLUGIN_LOGO;
        assert_eq!(PLUGIN_LOGO, (16.0 + x, y, w, h));
        let p = crate::art::part("plugin_logo_vst3").expect("plugin_logo_vst3");
        assert_eq!((p.w, p.h), (w, h));
        let led = crate::art::part("led_mode_on").expect("the lit mode LED");
        assert_eq!((led.w, led.h), (11.0, 11.0));
        for i in 0..3 {
            for state in ["off", "on"] {
                let p = crate::art::part(&format!("mode_label_{i}_{state}")).unwrap();
                assert_eq!((p.w, p.h), (62.0, 10.0));
            }
        }
    }

    #[test]
    fn a_drag_steps_through_the_modes_and_stops_at_the_ends() {
        assert_eq!(drag_mode(2, 0.0), 2);
        assert_eq!(drag_mode(2, -MODE_DRAG_STEP), 1);
        assert_eq!(drag_mode(2, -2.0 * MODE_DRAG_STEP), 0);
        assert_eq!(drag_mode(2, -500.0), 0);
        assert_eq!(drag_mode(0, 500.0), 2);
        assert_eq!(drag_mode(1, MODE_DRAG_STEP * 0.4), 1);
        assert_eq!(drag_mode(1, MODE_DRAG_STEP * 0.6), 2);
    }
}
