//! The panel as the plugin's editor (task-201, docs/plugin.md section 10): what orb-plugin's
//! editor gives the [`App`] ([`Embed`]) and how the panel differs in a plugin window.
//!
//! The plugin's panel is the standalone's, drawn by the same code, except:
//! - the transport has no Pattern/Song switch, Rewind, Fast forward, Record, bar arrows or loop,
//!   and the tempo display has no arrows ([`PLUGIN_HIDDEN`]; their keys do nothing); its background
//!   is `bg_transport_plugin`, with a Bar display of the host position ([`host_position`]; no Beat
//!   display, the user's decision of 2026-11-16);
//! - the menu bar is the standalone's, above the panel and not scaled with it ([`MENU_H`] points);
//!   File has New, Open and Info (no Save, Save As or Export), and nothing asks to save: the song
//!   lives in the host's project;
//! - the window is the menu bar and the panel, nothing else: its height is [`window_height`] of its
//!   width (a window of another shape, which the plugin's size hooks do not let the host make, is
//!   filled black around the panel);
//! - the Sync and MIDI LEDs light from the snapshot's flags (`host_sync`, `midi_in`);
//! - Options has the plugin's mode ([`PluginMode`], [`MODE_ITEMS`], task-217, task-218); Play and
//!   Stop are the plugin's preview, and the Play light shows only whether it runs
//!   ([`Embed::previewing`]), not the engine's play state;
//! - an invisible corner at the bottom right (the right wood's foot) resizes the window: the cursor
//!   changes over it and a faint grip shows while hovered or dragged, and a drag asks the host for
//!   a size of that rule.
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

/// The plugin's side of its editor (orb-plugin implements it): the song, the mod and the state
/// that the standalone app keeps by itself.
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
    /// The resize corner: asks the host for a window of `size` points (VST3
    /// `IPlugFrame::resizeView`, CLAP `gui.request_resize`); the host then sizes the window.
    /// The plugin makes the request after the frame, not from this call (a host may resize the
    /// window at once, which must not happen inside the frame). `Some(_)` when the request
    /// goes to the host, `None` when the plugin has no way to it (the window then resizes
    /// itself, `ViewportCommand::InnerSize`).
    fn request_size(&mut self, size: Vec2) -> Option<bool> {
        let _ = size;
        None
    }
    /// The plugin's pattern/note mode (task-217), which the Options menu shows and sets; `None`
    /// when the plugin has no such setting.
    fn mode(&self) -> Option<PluginMode> {
        None
    }
    /// The Options menu chose `mode`.
    fn set_mode(&mut self, mode: PluginMode) {
        let _ = mode;
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

/// The Options menu's items for [`PluginMode`], in order.
pub const MODE_ITEMS: [(PluginMode, &str); 3] = [
    (PluginMode::Latch, "Latch Pattern Mode"),
    (PluginMode::Gate, "Gate Pattern Mode"),
    (PluginMode::Note, "Note Mode"),
];

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
/// host's (`docs/plugin.md` section 8.2); else nothing, the dark display of the standalone's
/// pattern mode (note mode, or the host stopped: orb free-runs and has no host position). Bars
/// of four quarter notes, whatever the host's time signature (task-206).
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

/// The side of the invisible resize corner, in points.
pub const CORNER: f32 = 14.0;

/// The smallest window the corner asks for: half the panel (as the standalone's minimum).
pub const MIN_SCALE: f32 = 0.5;

/// While the editor is open, the panel is drawn at least this often, so that the LEDs follow
/// the host's MIDI and transport even when nothing else moves.
pub const POLL: Duration = Duration::from_millis(50);

/// The height of a plugin window `width` points wide: the panel at that width and the menu
/// bar, which does not scale (task-201).
pub fn window_height(width: f32) -> f32 {
    (width * PANEL_H / PANEL_W).round() + MENU_H
}

/// The size the corner asks for when a drag reaches `desired` (points): the panel as large as
/// the larger of the two scales the drag gives, at least [`MIN_SCALE`], in whole points, and
/// the menu bar above it ([`window_height`]).
pub fn corner_size(desired: Vec2) -> Vec2 {
    let k = (desired.x / PANEL_W)
        .max((desired.y - MENU_H) / PANEL_H)
        .max(MIN_SCALE);
    let w = (PANEL_W * k).round();
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
            for (i, f) in crate::behavior::digit_frames(p.position, n)
                .into_iter()
                .enumerate()
            {
                let r = view.rect(x0 + 9.0 * i as f32, y, 9.0, h);
                crate::app::paint_part(ui, f, r);
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
        self.resize_corner(ui);
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

    /// The invisible corner at the window's bottom right: the cursor shows it; a drag asks the
    /// host for a window of the panel's aspect (egui-baseview turns `InnerSize` into the
    /// plugin's resize request to the host).
    fn resize_corner(&mut self, ui: &mut Ui) {
        let win = ui.ctx().content_rect();
        let r = Rect::from_min_max(win.max - Vec2::splat(CORNER), win.max);
        let resp = ui.interact(r, Id::unique("orb plugin resize corner"), Sense::drag());
        if resp.hovered() || resp.dragged() {
            ui.ctx().set_cursor_icon(CursorIcon::ResizeNwSe);
            paint_grip(ui, r);
        }
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

/// The resize corner's grip, three diagonal strokes in the corner's rectangle `r`, shown while
/// the pointer is over the corner or drags it. The resize cursor egui asks for does not always
/// show in a host: baseview sets the cursor only when egui's changes (`CursorManager::
/// set_cursor`), and a host that sets its own cursor on mouse moves (Reaper on macOS, the
/// user's check of 2026-11-16) puts its arrow back while egui's stays the same.
pub(crate) fn paint_grip(ui: &Ui, r: Rect) {
    let stroke = egui::Stroke::new(1.0, GRIP);
    let p = ui.painter();
    for k in [3.0, 6.5, 10.0] {
        let a = egui::pos2(r.max.x - 2.0, r.max.y - 2.0 - k);
        let b = egui::pos2(r.max.x - 2.0 - k, r.max.y - 2.0);
        p.line_segment([a, b], stroke);
    }
}

/// The panel of a plugin window fills it; around a panel of another shape the window is black.
pub(crate) fn paint_surround(ui: &Ui, avail: Rect) {
    ui.painter().rect_filled(avail, 0.0, Color32::BLACK);
}

/// The window's point `p` is the corner's (tests).
pub fn on_corner(win: Rect, p: Pos2) -> bool {
    Rect::from_min_max(win.max - Vec2::splat(CORNER), win.max).contains(p)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_corner_scales_the_panel_and_keeps_the_menu_bar() {
        let m = MENU_H;
        assert_eq!(window_height(624.0), 510.0 + m);
        assert_eq!(
            corner_size(Vec2::new(624.0, 510.0 + m)),
            Vec2::new(624.0, 510.0 + m)
        );
        // the larger scale wins
        assert_eq!(
            corner_size(Vec2::new(1248.0, 600.0)),
            Vec2::new(1248.0, 1020.0 + m)
        );
        assert_eq!(
            corner_size(Vec2::new(700.0, 1020.0 + m)),
            Vec2::new(1248.0, 1020.0 + m)
        );
        // never below half the panel
        assert_eq!(
            corner_size(Vec2::new(10.0, 10.0)),
            Vec2::new(312.0, 255.0 + m)
        );
        // an odd width: the panel's height rounded, the bar added
        assert_eq!(window_height(700.0), (700.0f32 * 510.0 / 624.0).round() + m);
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
}
