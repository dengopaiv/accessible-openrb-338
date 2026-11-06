//! The window: draws the panel from the SVG parts at the displayed size and runs the
//! original's interaction (docs/ui_spec.md section 2) on the [`Panel`] model.
//!
//! Rendering follows decision task-040: every part is an `egui::Image` of its SVG, loaded by
//! egui_extras' SVG loader (usvg + resvg) with a size hint of the displayed size in physical
//! pixels; a scale change gives a new hint and so a new, crisp raster. The panel keeps its
//! 624 x 510 aspect ratio and is letterboxed.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use orb_engine::samples::SampleSet;

use eframe::egui::{
    self, Color32, Image, Key, Pos2, Rect, Response, Sense, Shape, Stroke, Ui, Vec2,
    emath::GuiRounding, epaint,
};

use crate::a11y::{announce, drive, nav};
use crate::art::{self, KnobDesign};
use crate::behavior::{self, Mods};
use crate::browser::{self, Browser, Mode, Poll};
use crate::document::Document;
use crate::keys::{self, Action, KeyMode, SrAction};
use crate::layout::{self, Ctl, PANEL_H, PANEL_W};
use crate::link::Link;
use crate::logo_anim;
use crate::menu::{Job, MENU_H, ModEntry, Then};
use crate::model::Panel;
use crate::palette::BG;
use crate::settings::Settings;
use crate::shortcuts;

/// Where the panel is on screen: the point of the panel's origin and points per panel pixel.
#[derive(Clone, Copy, Debug)]
pub struct View {
    pub origin: Pos2,
    pub scale: f32,
}

impl View {
    /// The largest panel-shaped area centered in `avail`, snapped to whole pixels.
    pub fn fit(avail: Rect, ppp: f32) -> Self {
        let scale = (avail.width() / PANEL_W).min(avail.height() / PANEL_H);
        let size = Vec2::new(PANEL_W, PANEL_H) * scale;
        Self {
            origin: (avail.center() - size / 2.0).round_to_pixels(ppp),
            scale,
        }
    }

    pub fn rect(&self, x: f32, y: f32, w: f32, h: f32) -> Rect {
        Rect::from_min_size(
            self.origin + Vec2::new(x, y) * self.scale,
            Vec2::new(w, h) * self.scale,
        )
    }

    /// Screen point to panel pixels.
    pub fn to_panel(&self, p: Pos2) -> Pos2 {
        ((p - self.origin) / self.scale).to_pos2()
    }
}

/// Per-control gesture state, kept in egui's memory.
#[derive(Clone, Default)]
struct Gesture {
    down: bool,
    p0: i32,
    y0: f32,
    fine: bool,
    t0: f64,
    taken: u32,
    new_value: u8,
    last_step: i32,
}

/// A computer key that is down: it acts as a mouse press on its button.
struct Hold {
    /// The physical key (`keys::position`).
    key: Key,
    /// The layout id of the button shown pressed.
    id: Option<String>,
    /// A repeat or wind button: its id, when the key went down, the steps taken.
    arrow: Option<(&'static str, f64, u32)>,
    /// What the announcer was told after the key's press (an arrow held longer announces its
    /// final value at the release; task-189).
    seen: Option<announce::KeyState>,
}

/// The windows that take the focus as they open in screen reader mode (`App::window_focus`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Win {
    KeyHelp,
    Info,
    About,
    ModInfo,
}

impl Win {
    pub(crate) const ALL: [Win; 4] = [Win::KeyHelp, Win::Info, Win::About, Win::ModInfo];
}

/// Screen reader mode: moves the focus into a window (`due`, from `App::window_focus_due`).
/// Called at the start of the window's contents: the first widget after this point that takes
/// the focus gets it, the window's first control, as egui's Tab does with nothing focused.
pub(crate) fn focus_into_window(ui: &Ui, due: bool) {
    if !due {
        return;
    }
    ui.memory_mut(|m| {
        if let Some(f) = m.focused() {
            m.surrender_focus(f);
        }
        m.move_focus(egui::FocusDirection::Next);
    });
}

/// The top-left corner that centers a block of `size` (a text galley, wrapped lines included)
/// in `r`, horizontally and vertically (the mod name in the 808's mod window).
pub fn centered_in(r: Rect, size: Vec2) -> Pos2 {
    Pos2::new(r.center().x - size.x / 2.0, r.center().y - size.y / 2.0)
}

/// The time between two frames while the engine's side of the panel moves (task-153): 30 per
/// second, enough for the meters and the running lights (a sixteenth at 200 BPM is 75 ms).
pub const METER_FRAME: Duration = Duration::from_millis(33);

/// Where to save a screenshot and quit (`--screenshot`).
pub struct Shot {
    pub path: PathBuf,
    pub frames: u32,
    requested: bool,
}

impl Shot {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            frames: 0,
            requested: false,
        }
    }
}

pub struct App {
    pub panel: Panel,
    /// The player the panel drives (task-045); without one the model's events pile up in
    /// `Panel::events`.
    pub link: Option<Link>,
    /// The drum samples of the kit "Standard ReBirth" (`rbm::standard::builtin_samples`, the
    /// embedded standard mod's; None: silent drum machines). A mod's samples are laid over the
    /// built-in kit.
    pub samples: Option<Arc<SampleSet>>,
    /// The folders of the file browser (task-046).
    pub settings: Settings,
    /// Whether the system reported a screen reader at the last check (at start and whenever the
    /// window regains the focus; task-185). Read through `App::screen_reader_mode`.
    system_screen_reader: bool,
    /// Asks the system whether a screen reader runs; `App::new` has none (false), the program
    /// installs the real one (`App::use_system_services`).
    screen_reader_probe: fn() -> bool,
    /// The window's focus in the last frame, to see it regained.
    was_focused: bool,
    /// The open file browser, if any (`open_browser`).
    browser: Option<Browser>,
    /// The system's file dialog, used instead of the browser in screen reader mode (task-194);
    /// tests replace it (`App::set_picker`).
    picker: crate::system_dialog::Picker,
    /// The app's window, the owner of those dialogs (set every frame from eframe's `Frame`).
    parent: Option<crate::system_dialog::Parent>,
    /// A name the system dialog did not confirm that is a file already: the question
    /// "Replace ...?" waits for its answer (`App::replace_ui`, task-194).
    pub(crate) replace: Option<crate::system_dialog::Replace>,
    /// The open song: its file, its mod, whether it has unsaved changes (task-047).
    pub doc: Document,
    /// Whether a song has settled yet; until then the "previous kit" of the startup sound's
    /// rule is none and the first song's mod plays (task-099).
    pub(crate) settled: bool,
    /// The errors not closed yet: the first is in the message box (`App::error_box_ui`, task-130),
    /// the rest wait for it.
    pub errors: std::collections::VecDeque<String>,
    /// How many times each kind of repeating error was notified (`App::notify`).
    pub(crate) notified: std::collections::HashMap<String, u32>,
    /// The audio stream's health (None: no stream), checked every frame: a lost device or a
    /// stall is reported once (`App::check_audio`), as in `orb play` (task-139).
    pub audio_health: Option<orb_player::output::Health>,
    audio_reported: bool,
    /// Where the box's OK button was drawn last (`App::error_ok_rect`).
    pub(crate) error_ok: Option<Rect>,
    /// The errors of the action running, shown as one box when it ends (`App::action`).
    pub(crate) batch: Option<Vec<String>>,
    /// Where its close button was drawn last (`App::error_close_rect`).
    pub(crate) error_close: Option<Rect>,
    /// Where its Copy button was drawn last (`App::error_copy_rect`).
    pub(crate) error_copy: Option<Rect>,
    /// Where its message text was drawn last (`App::error_text_rect`).
    pub(crate) error_text: Option<Rect>,
    /// What was done, shown at the bottom of the window ("Saved ...").
    pub status: Option<String>,
    /// The last announcement for a screen reader (task-189, `a11y::announce`).
    pub announcer: announce::Announcer,
    /// egui's focus when this frame's keys were read (what `App::act` does not announce).
    pub(crate) key_focus: Option<egui::Id>,
    /// The Options menu's keyboard mode (Ctrl/Cmd+F / +G).
    pub key_mode: KeyMode,
    /// Whether the window draws the menu bar (the window of OpenRB-338 does; the headless tests
    /// of the panel at 1:1 do not).
    pub menu: bool,
    pub(crate) prompt: Option<Then>,
    pub(crate) after_save: Option<Then>,
    pub(crate) export: Option<Job>,
    pub(crate) mod_list: Option<Result<Vec<ModEntry>, String>>,
    /// Whether a mod file has a `Demo.rbs`, for the file last asked (the Mods menu's item).
    pub(crate) demo_cache: Option<(PathBuf, bool)>,
    /// The splash picture of the kit last shown in the 808's mod window (`App::mod_view`).
    pub(crate) splash: Option<(orb_cli::mods::Kit, Option<egui::TextureHandle>)>,
    /// Whether the File > Info window is open (`App::info_ui`).
    pub(crate) info_open: bool,
    /// The number of the info window's opens; it is in the window's id, so that each open
    /// starts with no stored place and egui centers it (`App::info_ui`).
    pub(crate) info_serial: u32,
    /// Whether the Help > About window is open, and the number of its opens (its id, as
    /// `info_serial`; `App::about_ui`).
    pub(crate) about_open: bool,
    pub(crate) about_serial: u32,
    /// Whether the logo plays its startup animation (task-114): the panel once, from the first
    /// frame it is drawn, and the About window at each open. Off by default, so the headless
    /// tests and `--screenshot` see the final state; `enable_logo_anim` turns it on.
    pub logo_anim: bool,
    /// The egui time the panel's animation started at, and the About window's (None until its
    /// first frame after the open).
    pub(crate) logo_start: Option<f64>,
    pub(crate) about_start: Option<f64>,
    pub(crate) about_start_due: bool,
    /// The Mods > Info window (task-106): open, the number of its opens (its id), the file
    /// details of the kit shown and what it drew of the splash last.
    pub(crate) mod_info_open: bool,
    pub(crate) mod_info_serial: u32,
    pub(crate) mod_details: Option<crate::mod_info::Cached>,
    pub(crate) mod_info_shown: Option<crate::mod_info::SplashShown>,
    /// The same for the Help > Keyboard Shortcuts window (`App::key_help_ui`).
    pub(crate) key_help_open: bool,
    pub(crate) key_help_serial: u32,
    /// What the info window drew last (`App::info_shown`).
    pub(crate) info_shown: Option<crate::info::SongInfo>,
    pub(crate) events_seen: usize,
    pub(crate) allow_close: bool,
    /// The window title last sent (`App::update_title`).
    pub title: String,
    holds: Vec<Hold>,
    ctls: Vec<Ctl>,
    /// The focus order of screen reader mode (`a11y::nav`, task-188).
    nav: Vec<nav::Stop>,
    /// The section the navigation focus was in at the last frame of screen reader mode: the
    /// row focus follows the section only when the focus enters another one, so Ctrl+Up and
    /// Ctrl+Down move the row without the focus undoing it (`App::follow_section`).
    nav_section: Option<usize>,
    /// Screen reader mode and the windows (Keyboard Shortcuts, File > Info, Help > About,
    /// Mods > Info; `App::window_focus`): which were open at the last frame (a bit per
    /// [`Win::ALL`]), the window that opened last and the frames it has been waiting for the
    /// focus, the window the focus is to move into in this frame, the window it last moved
    /// into, and the panel control that had the focus last, which gets it back when the
    /// windows close.
    windows_open: u8,
    window_new: Option<(Win, u32)>,
    window_focus_due: Option<Win>,
    focus_window: Option<Win>,
    /// Tab was pressed with the focus in a window (`App::window_focus`).
    window_tab: bool,
    panel_focus: Option<egui::Id>,
    shot: Option<Shot>,
    /// `ORB_UI_FRAME_STATS=1`: the frame rate and cost, printed (`FrameStats`).
    pub frame_stats: Option<FrameStats>,
}

impl App {
    pub fn new(panel: Panel, shot: Option<Shot>) -> Self {
        Self {
            panel,
            link: None,
            samples: None,
            settings: Settings::default(),
            system_screen_reader: false,
            screen_reader_probe: || false,
            was_focused: true,
            browser: None,
            picker: crate::system_dialog::no_picker(),
            parent: None,
            replace: None,
            doc: Document::new(),
            settled: false,
            errors: Default::default(),
            notified: Default::default(),
            audio_health: None,
            audio_reported: false,
            error_ok: None,
            batch: None,
            error_close: None,
            error_copy: None,
            error_text: None,
            status: None,
            announcer: Default::default(),
            key_focus: None,
            key_mode: KeyMode::default(),
            menu: false,
            prompt: None,
            after_save: None,
            export: None,
            mod_list: None,
            demo_cache: None,
            splash: None,
            info_open: false,
            info_serial: 0,
            about_open: false,
            about_serial: 0,
            logo_anim: false,
            logo_start: None,
            about_start: None,
            about_start_due: false,
            mod_info_open: false,
            mod_info_serial: 0,
            mod_details: None,
            mod_info_shown: None,
            key_help_open: false,
            key_help_serial: 0,
            info_shown: None,
            events_seen: 0,
            allow_close: false,
            title: String::new(),
            holds: Vec::new(),
            ctls: layout::controls(),
            nav: nav::order(&layout::controls()),
            nav_section: None,
            windows_open: 0,
            window_new: None,
            window_focus_due: None,
            focus_window: None,
            window_tab: false,
            panel_focus: None,
            shot,
            frame_stats: FrameStats::from_env(),
        }
    }

    /// Connects the app to the system: the real screen reader check (now and at every focus
    /// gain) and the system's file dialogs. `App::new` has neither, so a test never asks the
    /// real system or opens a real dialog; the program calls this once.
    pub fn use_system_services(&mut self) {
        self.screen_reader_probe = crate::screen_reader::system_reports_screen_reader;
        self.system_screen_reader = (self.screen_reader_probe)();
        self.picker = crate::system_dialog::system_picker();
    }

    /// Whether screen reader mode is on: the setting is `on`, or `auto` and the system reported
    /// a screen reader at the last check (task-185). [`App::speech`] reads it (task-192).
    pub fn screen_reader_mode(&self) -> bool {
        self.settings
            .screen_reader
            .active(self.system_screen_reader)
    }

    /// How knobs and faders speak their value (task-192): in their units in screen reader mode,
    /// with the 0..127 number too when the setting "Speak raw values" is on; the number alone
    /// with the mode off.
    pub fn speech(&self) -> crate::a11y::names::Speech {
        use crate::a11y::names::Speech;
        match (self.screen_reader_mode(), self.settings.speak_raw_values) {
            (false, _) => Speech::Plain,
            (true, false) => Speech::Units,
            (true, true) => Speech::UnitsAndRaw,
        }
    }

    /// Asks the system again whether a screen reader runs, when the window has just regained
    /// the focus (`focused` is this frame's state).
    fn check_screen_reader(&mut self, focused: bool) {
        if focused && !self.was_focused {
            self.system_screen_reader = (self.screen_reader_probe)();
        }
        self.was_focused = focused;
    }

    /// Opens the file browser for `mode` over the panel (the menu calls this). Ctrl/Cmd+O opens it
    /// for a song.
    ///
    /// In screen reader mode the system's file dialog opens instead ([`App::system_pick`]); the
    /// in-app browser is for the mode off.
    pub fn open_browser(&mut self, mode: Mode) {
        if self.screen_reader_mode() {
            self.system_pick(mode);
        } else {
            self.browser = Some(Browser::new(mode, &self.settings));
        }
    }

    /// Replaces the system's file dialog (for tests).
    pub fn set_picker(&mut self, picker: crate::system_dialog::Picker) {
        self.picker = picker;
    }

    /// Asks the system dialog for a file of `mode`, in the folder the browser would start in,
    /// and does with the answer what the browser's answer does: the browser's checks and notes
    /// ([`browser::check_choice`], [`browser::note_choice`]), then [`App::chosen`]. The dialog
    /// is modal, so this call waits for it: the frame stands still, the audio thread plays on.
    /// A cancelled dialog changes nothing, as a cancelled browser (a Save As that the prompt's
    /// Save started ends that action).
    fn system_pick(&mut self, mode: Mode) {
        use crate::system_dialog::{Request, with_extension};
        let start = browser::start_dir(mode, &self.settings);
        let Some(picked) = (self.picker)(Request::File(mode), &start, self.parent.as_ref()) else {
            self.after_save = None;
            return;
        };
        let path = match with_extension(mode, picked.clone()) {
            Ok(p) => p,
            Err(e) => {
                self.after_save = None;
                self.fail(e);
                return;
            }
        };
        // The dialog asked about replacing `picked`, not about the name it was given here.
        if path != picked && path.exists() {
            self.replace = Some(crate::system_dialog::Replace {
                mode,
                path,
                focused: false,
            });
            return;
        }
        self.finish_pick(mode, path);
    }

    /// Does with a path the system dialog chose (and the app confirmed) what the browser does
    /// with its answer.
    pub(crate) fn finish_pick(&mut self, mode: Mode, path: PathBuf) {
        if let Err(e) = browser::check_choice(mode, &path) {
            self.after_save = None;
            self.fail(e);
            return;
        }
        let failure = browser::note_choice(mode, &path, &mut self.settings);
        self.fail_many(failure);
        self.chosen(mode, path);
    }

    /// Options > Set Songs Folder... / Set Mods Folder...: the system's folder dialog, starting
    /// in the folder now set (else the browser's start folder); the answer is what the
    /// browser's "Use as songs folder" / "Use as mods folder" button sets, saved to `ui.toml`;
    /// a new Mods folder is read again.
    pub fn choose_folder(&mut self, which: crate::system_dialog::Folder) {
        use crate::system_dialog::{Folder, Request};
        let (current, mode) = match which {
            Folder::Songs => (&self.settings.songs_dir, Mode::OpenSong),
            Folder::Mods => (&self.settings.mods_dir, Mode::ChooseMod),
        };
        let start = current
            .clone()
            .filter(|d| d.is_dir())
            .unwrap_or_else(|| browser::start_dir(mode, &self.settings));
        let Some(dir) = (self.picker)(Request::Folder(which), &start, self.parent.as_ref()) else {
            return;
        };
        match which {
            Folder::Songs => self.settings.songs_dir = Some(dir),
            Folder::Mods => self.settings.mods_dir = Some(dir),
        }
        let failure = self.settings.save().err();
        self.fail_many(failure);
        if which == Folder::Mods {
            self.refresh_mods();
        }
    }

    /// Where the open browser drew its part `name` last (for tests; `Browser::hit`).
    pub fn browser_hit(&self, name: &str) -> Option<Rect> {
        self.browser.as_ref().and_then(|b| b.hit(name))
    }

    /// Whether a browser is open (the panel takes no input meanwhile).
    pub fn browsing(&self) -> bool {
        self.browser.is_some()
    }

    /// Runs the browser for a frame and acts on its answer.
    fn browse(&mut self, ctx: &egui::Context) {
        let Some(b) = &mut self.browser else {
            return;
        };
        let mode = b.mode();
        let mods_dir = self.settings.mods_dir.clone();
        let poll = b.show(ctx, &mut self.settings);
        // a failed settings write (the recent-folder note, "Use as ... folder"): the box
        let failures = b.take_failures();
        self.fail_many(failures);
        // the browser's "Use as mods folder" (task-101)
        if self.settings.mods_dir != mods_dir {
            self.refresh_mods();
        }
        match poll {
            Poll::Open => {}
            Poll::Cancelled => {
                self.browser = None;
                // a Save As that the prompt's Save started, and was cancelled, ends that action
                self.after_save = None;
            }
            Poll::Chosen(path) => {
                self.browser = None;
                self.chosen(mode, path);
            }
        }
    }

    /// The line at the bottom of the window: what was done. Errors are not here, they are in
    /// the message box (`App::error_box_ui`).
    fn status_bar(&mut self, ui: &Ui) {
        let Some(text) = &self.status else {
            return;
        };
        let avail = ui.ctx().content_rect();
        let bar = Rect::from_min_max(Pos2::new(avail.min.x, avail.max.y - 22.0), avail.max);
        ui.painter()
            .rect_filled(bar, 0.0, Color32::from_rgb(0x1f, 0x2d, 0x24));
        ui.painter().text(
            bar.left_center() + Vec2::new(8.0, 0.0),
            egui::Align2::LEFT_CENTER,
            text,
            egui::FontId::proportional(12.0),
            Color32::from_rgb(0xb8, 0xd8, 0xc0),
        );
        announce::describe_status(ui.ctx(), bar, text);
        // Clicks in the browser, the prompt or the box over it do not dismiss it.
        let blocked = self.browser.is_some()
            || self.prompt.is_some()
            || self.replace.is_some()
            || self.error_open();
        if !blocked && self.export.is_none() && ui.input(|i| i.pointer.any_click()) {
            self.status = None;
        }
    }

    /// Turns the logo's startup animation on (task-114), except with `--screenshot`, which
    /// captures the final state.
    pub fn enable_logo_anim(&mut self) {
        self.logo_anim = self.shot.is_none();
    }

    /// An app whose panel drives `link`.
    pub fn with_link(panel: Panel, link: Link, shot: Option<Shot>) -> Self {
        let mut app = Self::new(panel, shot);
        app.link = Some(link);
        app
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut Ui, frame: &mut eframe::Frame) {
        egui_extras::install_image_loaders(ui.ctx());
        let focused = ui.input(|i| i.focused);
        self.check_screen_reader(focused);
        self.parent = crate::system_dialog::Parent::of(frame);
        self.close_check(&ui.ctx().clone());
        self.show(ui);
        self.screenshot(ui.ctx());
        if let Some(s) = &mut self.frame_stats
            && let Some(line) = s.frame(ui.input(|i| i.time), frame.info().cpu_usage)
        {
            eprintln!("{line}");
        }
    }
}

/// `ORB_UI_FRAME_STATS=1` (task-153): every 2 s, the frames drawn per second and the mean CPU
/// time of a frame (eframe's `cpu_usage`: `App::ui`, tessellation and rendering, without the
/// vsync wait), on the standard error.
#[derive(Default)]
pub struct FrameStats {
    since: Option<f64>,
    frames: u32,
    cpu: f64,
}

impl FrameStats {
    /// On when `ORB_UI_FRAME_STATS` is `1`.
    pub fn from_env() -> Option<Self> {
        std::env::var_os("ORB_UI_FRAME_STATS")
            .is_some_and(|v| v == "1")
            .then(Self::default)
    }

    /// One frame at egui time `now`; `cpu` is eframe's measure of the previous frame.
    /// Returns the line to print when 2 s of frames are counted.
    pub fn frame(&mut self, now: f64, cpu: Option<f32>) -> Option<String> {
        let since = *self.since.get_or_insert(now);
        self.frames += 1;
        self.cpu += cpu.unwrap_or(0.0) as f64;
        let dt = now - since;
        if dt < 2.0 {
            return None;
        }
        let line = format!(
            "frame stats: {:.1} frames/s, {:.2} ms CPU per frame",
            self.frames as f64 / dt,
            self.cpu * 1000.0 / self.frames as f64
        );
        *self = Self {
            since: Some(now),
            ..Self::default()
        };
        Some(line)
    }
}

#[cfg(test)]
mod frame_stats_tests {
    use super::FrameStats;

    #[test]
    fn a_line_every_two_seconds_of_frames() {
        let mut s = FrameStats::default();
        // 30 frames a second, 2 ms each
        let mut lines = vec![];
        for i in 0..=120 {
            lines.extend(s.frame(i as f64 / 30.0, Some(0.002)));
        }
        assert_eq!(lines.len(), 2, "{lines:?}");
        assert_eq!(
            lines[0],
            "frame stats: 30.5 frames/s, 2.00 ms CPU per frame"
        );
    }
}

/// An image of the part `name` painted into `rect`.
pub(crate) fn paint_part(ui: &Ui, name: &str, rect: Rect) {
    paint_part_rotated(ui, name, rect, None);
}

/// `paint_part` multiplied by `tint` (the logo's fades, task-114).
pub(crate) fn paint_part_tinted(ui: &Ui, name: &str, rect: Rect, tint: Color32) {
    paint_part_with(ui, name, rect, None, tint);
}

/// The size to rasterize an image of `pixels` at when a side may be at most `max`: None when it
/// fits, else the size scaled down (both sides by the same factor) to fit.
fn capped_hint(pixels: Vec2, max: usize) -> Option<egui::load::SizeHint> {
    let max = max as f32;
    let big = pixels.x.max(pixels.y);
    if big <= max {
        return None;
    }
    let k = max / big;
    Some(egui::load::SizeHint::Size {
        width: (pixels.x * k).floor().max(1.0) as u32,
        height: (pixels.y * k).floor().max(1.0) as u32,
        maintain_aspect_ratio: false,
    })
}

fn missing_part(name: &str) {
    if art::report_missing(name) {
        eprintln!("OpenRB-338: no art part {name}; drawn as nothing");
    }
}

/// The textures of the art's parts by (part URI, pixel width, pixel height), kept in the
/// context's memory (`paint_part_with`, task-153). The map owns each texture (its handle:
/// egui frees a texture when its last handle drops) and drops the ones not drawn for
/// [`PART_TEXTURE_FRAMES`] frames, the sizes of a window size left behind.
#[derive(Clone, Default)]
pub struct PartTextures {
    map: std::collections::HashMap<(usize, u32, u32), PartTexture>,
    /// The pass of the last sweep.
    swept: u64,
}

#[derive(Clone)]
struct PartTexture {
    handle: egui::TextureHandle,
    source_size: Vec2,
    /// The egui pass it was last drawn in.
    used: u64,
}

/// A part texture not drawn for this many frames is dropped.
pub const PART_TEXTURE_FRAMES: u64 = 120;

impl PartTextures {
    fn id() -> egui::Id {
        egui::Id::unique("orb-ui part textures")
    }

    /// How many part textures the context's map holds (tests).
    pub fn count(ctx: &egui::Context) -> usize {
        ctx.data_mut(|d| {
            d.get_temp_mut_or_default::<PartTextures>(Self::id())
                .map
                .len()
        })
    }

    /// The texture of `key` drawn in pass `pass`, if the map has it. Once per
    /// [`PART_TEXTURE_FRAMES`] passes the textures unused that long are dropped.
    fn get(&mut self, key: &(usize, u32, u32), pass: u64) -> Option<egui::load::SizedTexture> {
        if pass >= self.swept + PART_TEXTURE_FRAMES {
            self.swept = pass;
            self.map.retain(|_, t| t.used + PART_TEXTURE_FRAMES > pass);
        }
        let t = self.map.get_mut(key)?;
        t.used = pass;
        Some(egui::load::SizedTexture::new(t.handle.id(), t.source_size))
    }
}

fn paint_part_rotated(ui: &Ui, name: &str, rect: Rect, rot: Option<(f32, Vec2)>) {
    paint_part_with(ui, name, rect, rot, Color32::WHITE);
}

fn paint_part_with(ui: &Ui, name: &str, rect: Rect, rot: Option<(f32, Vec2)>, tint: Color32) {
    // A part name no art has (a state the panel does not foresee) draws nothing: it must not
    // close the app (task-156). It is reported once per name.
    let Some(part) = art::part(name) else {
        missing_part(name);
        return;
    };
    // The SVG is rasterized at the rect's size in pixels (what `Image::paint_at` asks of the
    // loader), once per size, as egui_extras' SVG loader does it (`load_svg_bytes_with_size`
    // with the default options, then `load_texture` with the URI as the name); the texture is
    // then found here, not through the loaders (task-153: their lookup and `include_bytes`
    // were most of a part's cost per frame).
    let ctx = ui.ctx();
    let ppp = ui.pixels_per_point();
    let pixels = (rect.round_to_pixels(ppp).size() * ppp).round();
    let key = (part.uri.as_ptr() as usize, pixels.x as u32, pixels.y as u32);
    let pass = ctx.cumulative_pass_nr();
    let cached = ctx.data_mut(|d| {
        d.get_temp_mut_or_default::<PartTextures>(PartTextures::id())
            .get(&key, pass)
    });
    let texture = match cached {
        Some(t) => t,
        None => {
            // A rect too big (a frame drawn before the window settles, a huge window, a high
            // density) would make a texture wider than the GPU allows, which wgpu refuses with
            // a panic. Rasterize it smaller then and let egui scale the texture up: a part
            // this big is mostly plain fill.
            let max = ctx.input(|i| i.max_texture_side);
            let hint = capped_hint(pixels, max).unwrap_or(egui::load::SizeHint::Size {
                width: pixels.x as _,
                height: pixels.y as _,
                maintain_aspect_ratio: false,
            });
            let options = resvg::usvg::Options::default();
            let Ok(image) = egui_extras::image::load_svg_bytes_with_size(part.svg, hint, &options)
            else {
                missing_part(name);
                return;
            };
            let source_size = image.source_size;
            let handle = ctx.load_texture(part.uri, image, egui::TextureOptions::LINEAR);
            let texture = egui::load::SizedTexture::new(handle.id(), source_size);
            ctx.data_mut(|d| {
                let t = PartTexture {
                    handle,
                    source_size,
                    used: pass,
                };
                d.get_temp_mut_or_default::<PartTextures>(PartTextures::id())
                    .map
                    .insert(key, t)
            });
            texture
        }
    };
    let mut image = Image::from_texture(texture)
        .fit_to_exact_size(rect.size())
        .tint(tint);
    if let Some((angle, origin)) = rot {
        image = image.rotate(angle, origin);
    }
    image.paint_at(ui, rect);
}

impl App {
    /// Reports once, in the message box, that the sound has stopped for good (the rule of
    /// `orb_player::output::Health`); a transient stream error that the sound survives is only
    /// logged by the stream. It runs on every frame, so a device lost while the UI is idle is
    /// reported on the next frame it repaints.
    fn check_audio(&mut self) {
        if self.audio_reported {
            return;
        }
        if let Some(why) = self.audio_health.as_ref().and_then(|h| h.failure()) {
            self.audio_reported = true;
            self.fail(format!("Audio output: {why}"));
        }
    }

    /// Draws the whole panel in the free area of `ui` and handles the input.
    pub fn show(&mut self, ui: &mut Ui) {
        self.check_audio();
        // The theme is always dark (task-127): the colors of the windows assume it. Set on
        // every frame, so a system theme that changes at run time cannot switch it back.
        if ui.ctx().options(|o| o.theme_preference) != egui::ThemePreference::Dark {
            ui.ctx().set_theme(egui::ThemePreference::Dark);
        }
        crate::a11y::set_speech(ui.ctx(), self.speech());
        let mut avail = ui.ctx().content_rect();
        ui.painter().rect_filled(avail, 0.0, BG);
        if self.menu {
            // the panel takes the window below the menu bar
            avail.min.y += MENU_H;
        }
        let view = View::fit(avail, ui.ctx().pixels_per_point());
        let now = ui.input(|i| i.time);
        self.panel.held = None;
        let screen_reader = self.screen_reader_mode();
        if screen_reader {
            self.follow_section(ui);
            self.window_focus(ui);
        } else {
            self.nav_section = None;
            self.windows_open = 0;
            self.window_new = None;
            self.window_focus_due = None;
            self.focus_window = None;
        }

        // While the file browser, the prompt or the error box is open the panel takes no keys
        // and no clicks of its own.
        if self.browser.is_none()
            && self.prompt.is_none()
            && self.replace.is_none()
            && !self.error_open()
        {
            if shortcuts::consume(ui, &shortcuts::NEW) {
                self.new_song();
            } else if shortcuts::consume(ui, &shortcuts::OPEN) {
                self.open_browser(Mode::OpenSong);
            } else if shortcuts::consume(ui, &shortcuts::SAVE) {
                self.save();
            } else {
                // Ctrl/Cmd+F and +G: the Options menu's two keyboard modes.
                if shortcuts::consume(ui, &shortcuts::KEYS_SYNTH) {
                    self.set_key_mode(KeyMode::Synth);
                }
                if shortcuts::consume(ui, &shortcuts::KEYS_PATTERNS) {
                    self.set_key_mode(KeyMode::Patterns);
                }
                if ui.ctx().text_edit_focused() || crate::menu::menu_has_keys(ui.ctx()) {
                    // typing in a text field (File > Info, task-117), or walking the menu bar
                    // (task-184): the other keys are the field's or the menu's, not the panel's
                    self.holds.clear();
                } else {
                    self.keyboard(ui);
                }
                self.focus_click(ui, &view);
            }
        } else {
            self.holds.clear();
        }

        for (name, x, y) in layout::backgrounds() {
            let (w, h) = part_size(name);
            if name == logo_anim::LOGO_PART {
                // once per app start, from the first frame the logo is drawn
                if self.logo_anim && self.logo_start.is_none() {
                    self.logo_start = Some(now);
                }
                let t = logo_anim::time_now(now, self.logo_start.filter(|_| self.logo_anim));
                logo_anim::paint_logo(ui, view.rect(x, y, w, h), t);
                continue;
            }
            paint_part(ui, name, view.rect(x, y, w, h));
        }
        for r in 0..4 {
            let (x, y, w, h) = layout::focus_bar(r);
            let name = if self.panel.focus == r {
                "focus_on"
            } else {
                "focus_off"
            };
            paint_part(ui, name, view.rect(x, y, w, h));
        }

        let ctls = std::mem::take(&mut self.ctls);
        // Screen reader mode: each section's controls in a child Ui whose node is a Group
        // named as the section (task-188); the order of the drawing is the panel's as before.
        let mut groups: Vec<Ui> = if screen_reader {
            let r = ui.max_rect();
            (0..nav::SECTIONS.len())
                .map(|s| ui.new_child(nav::group(s, r)))
                .collect()
        } else {
            Vec::new()
        };
        for c in &ctls {
            let section = nav::section_index(&c.section);
            let target = match section.and_then(|s| groups.get_mut(s)) {
                Some(g) => g,
                None => &mut *ui,
            };
            self.control(target, &view, c, now);
        }
        drop(groups);
        self.ctls = ctls;
        self.drum_strips(ui, &view);
        self.instrument_selection(ui, &view);

        // Send what was done, take what the engine did (the panel follows playback).
        self.note_edits();
        if let Some(link) = &mut self.link {
            let before = link.recorded_revision();
            link.sync(&mut self.panel);
            // a merge of the recording changed the song's tracks: an edit
            self.doc.modified |= link.recorded_revision() != before;
            // The repaint rule (task-153, docs/ui_spec.md section 16): at the meter rate while
            // the engine's side of the panel moves, else not at all (input repaints). Without
            // a working audio stream the snapshot never moves.
            let sound = self.audio_health.is_some() && !self.audio_reported;
            if sound && link.moving(&self.panel) {
                ui.ctx().request_repaint_after(METER_FRAME);
            }
        }
        // the 303's lit note in pitch mode goes out by itself
        if let Some(t) = self.panel.next_change(now) {
            ui.ctx()
                .request_repaint_after(Duration::from_secs_f64(t - now));
        }
        self.status_bar(ui);
        let ctx = ui.ctx().clone();
        // before everything that takes keys, which it takes away (the box is drawn above all)
        self.error_box_ui(&ctx);
        if self.menu {
            self.menu_bar(ui);
        }
        self.browse(&ctx);
        self.prompt_ui(&ctx);
        self.replace_ui(&ctx);
        self.info_ui(&ctx);
        self.about_ui(&ctx);
        self.key_help_ui(&ctx);
        self.mod_info_ui(&ctx);
        self.poll_export(&ctx);
        self.update_title(&ctx);
        // last: every message of this frame is in
        announce::show(&ctx, &self.announcer);
        self.close_if_allowed(&ctx);
    }

    /// The Options menu's keyboard mode (Ctrl/Cmd+F / +G and the menu), announced (task-189).
    pub fn set_key_mode(&mut self, m: KeyMode) {
        self.key_mode = m;
        self.announce(match m {
            KeyMode::Synth => "Program synth from keyboard",
            KeyMode::Patterns => "Select patterns from keyboard",
        });
    }

    /// The computer keyboard (`keys`): a key press acts like a mouse press on its button, a
    /// held repeat button keeps repeating, and the button shows pressed while the key is down.
    /// Auto-repeat events and keys with Ctrl or Cmd (the menu's shortcuts) are ignored, as the
    /// original's handlers do.
    fn keyboard(&mut self, ui: &Ui) {
        let (events, now, focused) = ui.input(|i| (i.events.clone(), i.time, i.focused));
        if !focused {
            self.holds.clear();
        }
        let screen_reader = self.screen_reader_mode();
        // Screen reader mode, the focus on the panel or nowhere: Tab, Shift+Tab and the arrows
        // do not move egui's focus (it moves in the panel's order below, and the arrows change
        // values). Called before any widget of this frame says it takes the focus, which is
        // where egui would move it. A window's focus keeps egui's Tab.
        let on_panel = ui
            .memory(|m| m.focused())
            .is_none_or(|f| nav::section_of(&self.nav, f).is_some());
        // In a window only the arrows' moves are cancelled (they would leave the window for
        // the panel by position); Tab stays egui's.
        let arrow = || {
            events.iter().any(|e| {
                matches!(e, egui::Event::Key { key, pressed: true, .. }
                    if matches!(key, Key::ArrowUp | Key::ArrowDown | Key::ArrowLeft
                        | Key::ArrowRight))
            })
        };
        let tab = || {
            events.iter().any(|e| {
                matches!(
                    e,
                    egui::Event::Key {
                        key: Key::Tab,
                        pressed: true,
                        ..
                    }
                )
            })
        };
        let panel_control = ui
            .memory(|m| m.focused())
            .is_some_and(|f| nav::section_of(&self.nav, f).is_some());
        let window_has_keys = !panel_control && Win::ALL.iter().any(|&w| self.win_open(w));
        if screen_reader && (on_panel || (arrow() && !tab())) {
            ui.memory_mut(|m| m.move_focus(egui::FocusDirection::None));
        }
        self.key_focus = ui.memory(|m| m.focused());
        for ev in events {
            let egui::Event::Key {
                key,
                physical_key,
                pressed,
                repeat,
                modifiers,
            } = ev
            else {
                continue;
            };
            let k = keys::position(key, physical_key);
            if !pressed {
                self.release(k);
                continue;
            }
            // Screen reader mode: Up, Down, Page Up, Page Down, Home and End change the focused
            // slider, spin button or pattern selector, auto-repeat included, and do not reach
            // the original's keys (task-187).
            if !modifiers.any()
                && screen_reader
                && let Some(step) = drive::Step::of_key(key)
                && let Some(t) = self.focused_target(ui)
            {
                drive::step(&mut self.panel, &t, step, now);
                continue;
            }
            if screen_reader {
                // The key map of the mode (task-188, decision A11Y-D6). With the focus in a
                // window (Keyboard Shortcuts, Info, About, Mods > Info), the panel's keys wait,
                // as they do for the menus (TASK-202): Enter, Space, the arrows and the letters
                // are the window's; the mode's function keys still work (F2 to F4 are
                // task-189's, F10 the menu bar's, Escape the window's). Also with the focus
                // nowhere while a window is open (before the window takes it, or after a click
                // on its text).
                // F2, F3 and F4 read the pattern, the transport, the levels (task-189), with a
                // window open too (F2 then reads the row focus's pattern).
                if !repeat && !modifiers.any() && self.readout_key(ui, key) {
                    continue;
                }
                if window_has_keys
                    && !matches!(
                        k,
                        Key::F1
                            | Key::F2
                            | Key::F3
                            | Key::F4
                            | Key::F5
                            | Key::F6
                            | Key::F9
                            | Key::F11
                            | Key::F12
                    )
                {
                    continue;
                }
                let ctrl = modifiers.command || modifiers.ctrl;
                // Enter follows the section of the focused control, not the row focus (the
                // user's decision, task-207): in a 303's section that 303's Step (Program
                // Synth mode), in any other section Play. With no panel control focused, the
                // row focus decides, as with the mode off.
                let section = ui
                    .memory(|m| m.focused())
                    .and_then(|f| nav::section_of(&self.nav, f));
                let enter = k == Key::Enter && !ctrl;
                let mapped = match (enter, section) {
                    (true, Some(s)) => match nav::row_of_section(s) {
                        Some(r) if r < 2 => {
                            keys::map_screen_reader(k, modifiers.shift, ctrl, self.key_mode, r)
                        }
                        _ => Some(SrAction::Panel(Action::Press("transport.play".into()))),
                    },
                    _ => keys::map_screen_reader(
                        k,
                        modifiers.shift,
                        ctrl,
                        self.key_mode,
                        self.panel.focus,
                    ),
                };
                let Some(a) = mapped else {
                    continue;
                };
                match a {
                    // moves of the focus repeat as in any program
                    SrAction::Control(back) if on_panel => {
                        let f = ui.memory(|m| m.focused());
                        self.move_nav(ui, nav::next_control(&self.nav, f, back));
                    }
                    SrAction::Control(_) => {}
                    SrAction::Section(back) => {
                        let f = ui.memory(|m| m.focused());
                        self.move_nav(ui, nav::next_section(&self.nav, f, back));
                    }
                    // the original ignores auto-repeat for its keys; that stays
                    _ if repeat => {}
                    SrAction::Activate => self.activate(ui, now),
                    SrAction::Help => self.show_key_help(),
                    SrAction::Panel(a) => self.act(k, a, now),
                }
                continue;
            }
            if repeat || modifiers.command || modifiers.ctrl {
                continue;
            }
            if let Some(a) = keys::map(k, modifiers.shift, self.key_mode, self.panel.focus) {
                self.act(k, a, now);
            }
        }
        // The held repeat buttons: the held-button schedule of the mouse.
        let mut holds = std::mem::take(&mut self.holds);
        for h in &mut holds {
            if let Some((id, t0, taken)) = &mut h.arrow {
                let total = 1 + behavior::repeat_extra_steps(((now - *t0) * 1000.0) as u32);
                while *taken < total {
                    self.panel.arrow(id);
                    *taken += 1;
                }
                ui.ctx().request_repaint_after(Duration::from_millis(16));
            }
        }
        self.holds = holds;
    }

    /// The release of the key `k`: its held button is let go. A repeat or wind button that
    /// stepped more than once announces where it ended (task-189).
    fn release(&mut self, k: Key) {
        let (released, held): (Vec<Hold>, Vec<Hold>) = std::mem::take(&mut self.holds)
            .into_iter()
            .partition(|h| h.key == k);
        self.holds = held;
        for h in released {
            if let (Some(seen), Some((id, _, taken))) = (h.seen, h.arrow)
                && taken > 1
            {
                let after = seen.with_arrow_from(id, &announce::KeyState::of(&self.panel));
                if let Some(text) = announce::key_effect(&seen, &after, self.key_focus) {
                    self.announce(text);
                }
            }
        }
    }

    /// What a key of the original's map does (`keys::map`). What it changed is announced in
    /// screen reader mode when the focus is not on the control that shows it (task-189).
    fn act(&mut self, k: Key, action: Action, now: f64) {
        // (a press without the release before it, e.g. after the window lost the focus)
        self.holds.retain(|h| h.key != k);
        let before = announce::KeyState::of(&self.panel);
        let mut hold = Hold {
            key: k,
            id: None,
            arrow: None,
            seen: None,
        };
        match action {
            Action::PlayStop => {
                let id = if self.panel.transport.playing {
                    "transport.stop"
                } else {
                    "transport.play"
                };
                self.panel.press(id, now);
                hold.id = Some(id.into());
            }
            Action::Press(id) => {
                self.panel.press(&id, now);
                hold.id = Some(id);
            }
            Action::Arrow(id) => {
                self.panel.arrow(id);
                hold.id = Some(id.into());
                hold.arrow = Some((id, now, 1));
            }
            Action::Locate(end) => self.panel.locate_loop(end),
            // Up from row 1 goes to row 4 and Down from row 4 to row 1 (`sub_42D1D3`).
            Action::Focus(d) => {
                self.panel.focus = (self.panel.focus as i32 + d).rem_euclid(4) as usize
            }
            Action::Tap303(rest) => self.panel.tap_303(self.panel.focus, rest),
            Action::TapDrum(i) => self.panel.tap_drum(self.panel.focus == 2, i),
        }
        self.announce_key_effect(&before);
        if hold.arrow.is_some() {
            hold.seen = Some(announce::KeyState::of(&self.panel));
        }
        self.holds.push(hold);
    }

    /// Whether a computer key that is down holds the button `id` pressed.
    pub fn key_held(&self, id: &str) -> bool {
        self.holds.iter().any(|h| h.id.as_deref() == Some(id))
    }

    /// Screen reader mode: gives egui's focus to the panel's control `to` (Tab, F6) and the row
    /// focus to its section's machine.
    fn move_nav(&mut self, ui: &Ui, to: Option<egui::Id>) {
        if let Some(id) = to {
            ui.memory_mut(|m| m.request_focus(id));
            self.follow_section(ui);
        }
    }

    /// Screen reader mode: when the navigation focus is in another section than at the last
    /// frame, a machine's section gives that machine the row focus (docs/accessibility.md
    /// 3.2), so the synth, tap and Enter keys go to it. A screen reader says the section's name
    /// as the focus enters its group; TASK-189 adds the spoken confirmation.
    fn follow_section(&mut self, ui: &Ui) {
        let section = ui
            .memory(|m| m.focused())
            .and_then(|f| nav::section_of(&self.nav, f));
        if section.is_some() && section != self.nav_section {
            if let Some(r) = section.and_then(nav::row_of_section) {
                self.panel.focus = r;
            }
            self.nav_section = section;
        }
    }

    /// Screen reader mode: the focus and the windows Keyboard Shortcuts, File > Info, Help >
    /// About and Mods > Info. When one opens, the focus is to move into it
    /// ([`focus_into_window`], called at the start of its contents); while one has the focus,
    /// Tab is egui's there (`App::keyboard`), and Tab on its last widget goes back to its first
    /// rather than out to the panel; when the last closes, the panel
    /// control that had the focus last gets it back.
    fn window_focus(&mut self, ui: &Ui) {
        let focused = ui.memory(|m| m.focused());
        if let Some(f) = focused.filter(|&f| nav::section_of(&self.nav, f).is_some()) {
            self.panel_focus = Some(f);
        }
        let mask = Win::ALL
            .iter()
            .enumerate()
            .filter(|(_, w)| self.win_open(**w))
            .fold(0u8, |m, (i, _)| m | 1 << i);
        let open = mask != 0;
        let in_window = open && focused.is_some_and(|f| nav::section_of(&self.nav, f).is_none());
        // a window that opened since the last frame is the one the focus goes to (the newest,
        // also when another is open)
        let new = mask & !self.windows_open;
        if let Some(i) = (0..Win::ALL.len()).find(|i| new & 1 << i != 0) {
            self.window_new = Some((Win::ALL[i], 0));
        }
        let modal = self.browser.is_some()
            || self.prompt.is_some()
            || self.replace.is_some()
            || self.error_open();
        match self.window_new {
            Some((w, _)) if !self.win_open(w) => self.window_new = None,
            // under the browser, the prompt, Replace or the error box the move waits until it
            // closes: they have the keys
            Some(_) if modal => {}
            // the window's second frame: in its first, egui sizes it without showing its
            // widgets (a sizing pass), so none could take the focus
            Some((w, n)) if n >= 1 => {
                self.window_new = None;
                self.window_focus_due = Some(w);
                self.focus_window = Some(w);
            }
            Some((w, n)) => {
                self.window_new = Some((w, n + 1));
                // the second frame comes whether or not anything else repaints
                ui.ctx().request_repaint();
            }
            None => {}
        }
        if std::mem::take(&mut self.window_tab)
            && !in_window
            && let Some(w) = self.focus_window.filter(|&w| self.win_open(w))
        {
            // Tab on the window's last widget: egui gives the focus to the next widget that
            // takes it, which is the panel's first (the windows are drawn after the panel); it
            // goes back to the window's first control, so Tab cycles in the window's contents.
            // The title bar (its close button), which egui registers before the contents, is
            // reached with Shift+Tab
            self.window_focus_due = Some(w);
        }
        self.window_tab = in_window
            && ui.input(|i| {
                i.events.iter().any(|e| {
                    matches!(e, egui::Event::Key { key: Key::Tab, pressed: true, modifiers, .. }
                        if !modifiers.any())
                })
            });
        if !open && self.windows_open != 0 {
            self.window_focus_due = None;
            self.focus_window = None;
            let on_panel = focused.is_some_and(|f| nav::section_of(&self.nav, f).is_some());
            if let Some(back) = self.panel_focus.filter(|_| !on_panel) {
                ui.memory_mut(|m| m.request_focus(back));
            }
        }
        self.windows_open = mask;
    }

    /// Whether the window `w` is open.
    fn win_open(&self, w: Win) -> bool {
        match w {
            Win::KeyHelp => self.key_help_open,
            Win::Info => self.info_open,
            Win::About => self.about_open,
            Win::ModInfo => self.mod_info_open,
        }
    }

    /// Called by the window `w` of [`App::window_focus`] before it draws: whether, in screen
    /// reader mode, the focus is to go into it now ([`focus_into_window`] at the start of its
    /// contents).
    pub(crate) fn window_focus_due(&mut self, w: Win) -> bool {
        if self.window_focus_due == Some(w) {
            self.window_focus_due = None;
            true
        } else {
            false
        }
    }

    /// Space in screen reader mode: what a screen reader's Click does on the focused button,
    /// toggle or switch, one mouse down (`App::mouse_down`). Nothing on the other controls.
    fn activate(&mut self, ui: &Ui, now: f64) {
        let Some(f) = ui.memory(|m| m.focused()) else {
            return;
        };
        let c = self
            .ctls
            .iter()
            .find(|c| !drive::joined(c) && crate::a11y::hit_id(c) == f)
            .cloned();
        if let Some(c) = c {
            // what it did is announced as for a key of the original (task-189): Play and Stop
            // have no state of their own; a focused toggle or switch speaks its own
            let before = announce::KeyState::of(&self.panel);
            self.mouse_down(&c, now);
            self.announce_key_effect(&before);
        }
    }

    /// A click anywhere inside a row gives the row the focus (`sub_42ADBB`).
    fn focus_click(&mut self, ui: &Ui, view: &View) {
        let pressed = ui.input(|i| {
            i.pointer
                .button_pressed(egui::PointerButton::Primary)
                .then(|| i.pointer.press_origin())
                .flatten()
        });
        if let Some(p) = pressed {
            let p = view.to_panel(p);
            if let Some(r) = layout::row_at(p.x, p.y) {
                self.panel.focus = r;
            }
            // the logo (`sub_42A799`) plays the mod's startup sound, whatever the preference
            // (it sends msg 58 without testing it, 0x42A7D3..0x42A7F1)
            if layout::on_logo(p.x, p.y)
                && let Some(link) = &mut self.link
            {
                link.startup_sound();
            }
        }
    }

    fn control(&mut self, ui: &mut Ui, view: &View, c: &Ctl, now: f64) {
        let rect = view.rect(c.x, c.y, c.w, c.h);
        match c.kind.as_str() {
            "area" | "steps" => {}
            "modview" => {
                paint_part(ui, &c.part, rect);
                self.mod_view(ui, view, c);
            }
            "knob" => self.knob(ui, view, c, rect),
            "fader" => self.fader(ui, view, c, rect),
            "display" => {
                if drive::is_spin(&c.id) {
                    self.spin_button(ui, c, rect, now);
                } else {
                    if self.screen_reader_mode() {
                        // a hover-only widget, so that the label's node is in the section's
                        // group (the parent of a node is the Ui of its widget)
                        ui.interact(rect, crate::a11y::hit_id(c), Sense::hover());
                    }
                    crate::a11y::describe_display(ui.ctx(), rect, &self.panel, c);
                }
                let n = (c.w / 9.0) as usize;
                let frames = behavior::digit_frames(self.panel.display(&c.id), n);
                for (i, f) in frames.into_iter().enumerate() {
                    let x = c.x + c.w - 9.0 * (n - i) as f32;
                    paint_part(ui, f, view.rect(x, c.y, 9.0, 16.0));
                }
            }
            "led" | "meter" => {
                let name = self.panel.part_name(c, false, now);
                paint_part(ui, &name, rect);
            }
            kind => {
                if kind == "grid" && c.name() == "pattern1" {
                    // the row's selector, before its buttons: the Tab order is the panel's
                    self.pattern_selector(ui, view, c, now);
                }
                let joined = drive::joined(c);
                let r = if joined {
                    // part of a spin button or a pattern selector: the mouse works as with
                    // `Sense::click_and_drag`, but no focus and no node (task-187)
                    ui.interact(rect, crate::a11y::hit_id(c), Sense::CLICK | Sense::DRAG)
                } else {
                    self.hit(ui, c, rect)
                };
                let down = r.is_pointer_button_down_on() || self.key_held(&c.id);
                match kind {
                    "repeat" | "wind" => self.repeat(ui, c, &r),
                    "switch" | "toggle" | "radio" | "trigger" | "grid" => {
                        if self.first_press(ui, c, &r) {
                            self.panel.press(&c.id, now);
                        }
                    }
                    // A click anywhere on a two-position switch flips the position it shows.
                    "hswitch" | "vswitch" if self.first_press(ui, c, &r) => {
                        let v = 1 - self.panel.switch_position(&c.id);
                        self.panel.set_switch(&c.id, v);
                    }
                    _ => {}
                }
                if !joined {
                    // A screen reader's Click (its default action) is one mouse down. Only the
                    // AccessKit request: egui's own click of a focused widget on Space or Enter
                    // stays without effect, as before (task-187).
                    let ak_click = ui.input(|i| {
                        i.has_accesskit_action_request(r.id, egui::accesskit::Action::Click)
                    });
                    if ak_click {
                        self.mouse_down(c, now);
                    }
                    crate::a11y::describe(ui.ctx(), r.id, &self.panel, c);
                }
                let name = self.panel.part_name(c, down, now);
                paint_part(ui, &name, rect);
            }
        }
    }

    /// The 808's mod window: the current kit's `splash.jpg` (the embedded standard mod's for the
    /// built-in kit) in a square at the top, the song's mod name under it.
    fn mod_view(&mut self, ui: &Ui, view: &View, c: &Ctl) {
        const PAD: f32 = 4.0;
        let side = c.w - 2.0 * PAD;
        if let Some(tex) = self.splash_texture(ui.ctx()) {
            let r = view.rect(c.x + PAD, c.y + PAD, side, side);
            Image::new((tex.id(), r.size())).paint_at(ui, r);
        }
        let top = c.y + 2.0 * PAD + side;
        let r = view.rect(c.x + 2.0, top, c.w - 4.0, c.y + c.h - top - 1.0);
        let galley = ui.painter().layout(
            self.doc.mod_name.clone(),
            egui::FontId::proportional(7.0 * view.scale),
            Color32::from_rgb(0xd8, 0xd8, 0xd0),
            r.width(),
        );
        let pos = centered_in(r, galley.size());
        ui.painter()
            .with_clip_rect(r)
            .galley(pos, galley, Color32::WHITE);
    }

    /// The splash picture of the kit that plays, as a texture (cached per kit, shared by the
    /// 808's mod window and Mods > Info); None when there is none.
    pub(crate) fn splash_texture(&mut self, ctx: &egui::Context) -> Option<egui::TextureHandle> {
        if self.splash.as_ref().is_none_or(|(k, _)| *k != self.doc.kit) {
            let tex = crate::session::splash_image(&self.doc.kit)
                .map(|c| ctx.load_texture("mod-splash", c, egui::TextureOptions::LINEAR));
            self.splash = Some((self.doc.kit.clone(), tex));
        }
        self.splash.as_ref().and_then(|(_, t)| t.clone())
    }

    fn hit(&self, ui: &mut Ui, c: &Ctl, rect: Rect) -> Response {
        // The kind is part of the key: the 303's accent knob and accent flag button share
        // the layout id `tb303_N.accent` (`a11y::hit_id`).
        ui.interact(rect, crate::a11y::hit_id(c), Sense::click_and_drag())
    }

    /// What one mouse down on a button, toggle or switch does (a screen reader's Click). The
    /// pattern and bank buttons and the spin buttons' arrows have no node and get no Click
    /// (they are driven through their selector or spin button, `drive`).
    fn mouse_down(&mut self, c: &Ctl, now: f64) {
        match c.kind.as_str() {
            "repeat" | "wind" => self.panel.arrow(&c.id),
            "switch" | "toggle" | "radio" | "trigger" => self.panel.press(&c.id, now),
            "hswitch" | "vswitch" => {
                let v = 1 - self.panel.switch_position(&c.id);
                self.panel.set_switch(&c.id, v);
            }
            _ => {}
        }
    }

    /// The screen reader's requests to the value control `t` whose node is `id`, and, in
    /// screen reader mode with the focus on it, egui's focus kept from the Up and Down keys
    /// (they change the value: `App::keyboard`).
    fn drive_value(&mut self, ui: &Ui, r: &Response, t: &drive::Target, now: f64) {
        for req in drive::requests(ui, r.id) {
            drive::request(&mut self.panel, t, req, now);
        }
        if r.has_focus() && self.screen_reader_mode() {
            let filter = egui::EventFilter {
                vertical_arrows: true,
                ..Default::default()
            };
            ui.memory_mut(|m| m.set_focus_lock_filter(r.id, filter));
        }
    }

    /// A display with its up and down arrows: one spin button (task-187). The display takes
    /// the focus but no click; the arrows keep their mouse behavior (`App::control`).
    fn spin_button(&mut self, ui: &mut Ui, c: &Ctl, rect: Rect, now: f64) {
        let r = ui.interact(rect, crate::a11y::hit_id(c), Sense::FOCUSABLE);
        self.drive_value(ui, &r, &drive::Target::Spin(c.id.clone()), now);
        crate::a11y::describe_spin(ui.ctx(), &self.panel, c);
    }

    /// A row's pattern selector over its 8 pattern and 4 bank buttons (task-187): focusable,
    /// with no click of its own, so the buttons under it keep the mouse. `c` is the row's
    /// button `pattern1`, the selector's top left.
    fn pattern_selector(&mut self, ui: &mut Ui, view: &View, c: &Ctl, now: f64) {
        let Some(row) = drive::row_of(&c.section) else {
            return;
        };
        let (w, h) = drive::SELECTOR_SIZE;
        let rect = view.rect(c.x, c.y, w, h);
        let r = ui.interact(rect, drive::selector_id(row), Sense::FOCUSABLE);
        self.drive_value(ui, &r, &drive::Target::Pattern(row), now);
        crate::a11y::describe_selector(ui.ctx(), &self.panel, row, c);
    }

    /// The value control that has egui's focus, if any.
    fn focused_target(&self, ui: &Ui) -> Option<drive::Target> {
        let f = ui.memory(|m| m.focused())?;
        if let Some(row) = (0..4).find(|&r| drive::selector_id(r) == f) {
            return Some(drive::Target::Pattern(row));
        }
        self.ctls
            .iter()
            .filter_map(|c| Some((c, drive::target(c)?)))
            .find(|(c, _)| crate::a11y::hit_id(c) == f)
            .map(|(_, t)| t)
    }

    fn gesture(ui: &Ui, c: &Ctl) -> (egui::Id, Gesture) {
        let id = egui::Id::unique(("gesture", &c.kind, &c.id));
        (id, ui.data(|d| d.get_temp(id)).unwrap_or_default())
    }

    /// True on the frame the mouse goes down on the control (the action runs at mouse down).
    fn first_press(&self, ui: &Ui, c: &Ctl, r: &Response) -> bool {
        let (id, mut g) = Self::gesture(ui, c);
        let down = r.is_pointer_button_down_on();
        let first = down && !g.down;
        g.down = down;
        ui.data_mut(|d| d.insert_temp(id, g));
        first
    }

    /// Repeat and wind buttons: one step at mouse down, then the held-button schedule.
    fn repeat(&mut self, ui: &Ui, c: &Ctl, r: &Response) {
        let (id, mut g) = Self::gesture(ui, c);
        let down = r.is_pointer_button_down_on();
        let time = ui.input(|i| i.time);
        if down && !g.down {
            g.t0 = time;
            g.taken = 0;
        }
        g.down = down;
        if down {
            let t = ((time - g.t0) * 1000.0) as u32;
            let total = 1 + behavior::repeat_extra_steps(t);
            while g.taken < total {
                self.panel.arrow(&c.id);
                g.taken += 1;
            }
            ui.ctx().request_repaint_after(Duration::from_millis(16));
        }
        ui.data_mut(|d| d.insert_temp(id, g));
    }

    /// Where the pointer is, in panel pixels.
    fn pointer(ui: &Ui, view: &View) -> Option<Pos2> {
        ui.input(|i| i.pointer.latest_pos())
            .map(|p| view.to_panel(p))
    }

    fn press_origin(ui: &Ui, view: &View) -> Option<Pos2> {
        ui.input(|i| i.pointer.press_origin().or(i.pointer.latest_pos()))
            .map(|p| view.to_panel(p))
    }

    fn knob(&mut self, ui: &mut Ui, view: &View, c: &Ctl, rect: Rect) {
        let r = self.hit(ui, c, rect);
        let (id, mut g) = Self::gesture(ui, c);
        let down = r.is_pointer_button_down_on();
        if r.hovered() || down {
            ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeVertical);
        }
        if down {
            self.panel.held = Some(c.id.clone());
            if !g.down {
                // Mouse down grabs the knob where it is (no jump); Shift now means fine.
                g.p0 = self.panel.knob(&c.id);
                g.y0 = Self::press_origin(ui, view).map_or(0.0, |p| p.y);
                g.fine = ui.input(|i| i.modifiers.shift);
            }
            if let Some(p) = Self::pointer(ui, view) {
                let v = behavior::knob_drag(g.p0, g.y0, p.y, behavior::knob_span(&c.id), g.fine);
                self.panel.set_knob(&c.id, v);
            }
        }
        g.down = down;
        ui.data_mut(|d| d.insert_temp(id, g));
        // a screen reader's request in a frame where the mouse drags this knob is dropped: the
        // drag sets the position from its own start on every frame and would undo it anyway
        if !down && let Some(t) = drive::target(c) {
            self.drive_value(ui, &r, &t, ui.input(|i| i.time));
        }
        crate::a11y::describe(ui.ctx(), r.id, &self.panel, c);

        let k = art::knob_design(&c.part);
        let p = self.panel.knob(&c.id);
        paint_part(ui, &format!("{}_body", k.name), rect);
        if k.ring_r > 0.0 {
            let bipolar = c.name() == "pan";
            let a0 = if bipolar { 0.0 } else { -135.0 };
            paint_arc(ui, view, c, k, a0, behavior::knob_angle(p));
        }
        let angle = behavior::knob_angle(p).to_radians();
        let pivot = Vec2::new(k.cx / k.w, k.cy / k.h);
        paint_part_rotated(
            ui,
            &format!("{}_pointer", k.name),
            rect,
            Some((angle, pivot)),
        );
    }

    fn fader(&mut self, ui: &mut Ui, view: &View, c: &Ctl, rect: Rect) {
        let r = self.hit(ui, c, rect);
        let (id, mut g) = Self::gesture(ui, c);
        let down = r.is_pointer_button_down_on();
        if down {
            self.panel.held = Some(c.id.clone());
            if !g.down {
                let y = Self::press_origin(ui, view).map_or(0.0, |p| p.y) - c.y;
                let p = self.panel.fader(&c.id);
                if !behavior::fader_on_handle(p, y) {
                    self.panel.set_fader(&c.id, behavior::fader_jump(y));
                }
                g.p0 = self.panel.fader(&c.id);
                g.y0 = y + c.y;
            }
            if let Some(p) = Self::pointer(ui, view) {
                self.panel
                    .set_fader(&c.id, behavior::fader_drag(g.p0, g.y0, p.y));
            }
        }
        g.down = down;
        ui.data_mut(|d| d.insert_temp(id, g));
        // as for the knob: a request during a drag of this fader is dropped
        if !down && let Some(t) = drive::target(c) {
            self.drive_value(ui, &r, &t, ui.input(|i| i.time));
        }
        crate::a11y::describe(ui.ctx(), r.id, &self.panel, c);
        paint_part(ui, "fader_track", rect);
        let top = behavior::fader_handle_top(self.panel.fader(&c.id));
        paint_part(ui, "fader_handle", view.rect(c.x, c.y + top, 15.0, 11.0));
    }

    /// The step strips of the two drum machines: press sets the step to the opposite (808)
    /// or the cycled value (909) and a drag paints every crossed step with that value.
    fn drum_strips(&mut self, ui: &mut Ui, view: &View) {
        for is808 in [true, false] {
            let (origin, x0, y0, w, h) = if is808 {
                (layout::TR808_ORIGIN, 14.0, 91.0, 256.0, 17.0)
            } else {
                (layout::TR909_ORIGIN, 26.0, 94.0, 336.0, 16.0)
            };
            let rect = view.rect(origin.0 + x0, origin.1 + y0, w, h);
            let name = if is808 { "strip808" } else { "strip909" };
            // no FOCUSABLE: the strip is not a Tab stop (task-186; the grids become
            // readable in task-191), the mouse works as with `Sense::click_and_drag`
            let r = ui.interact(
                rect,
                egui::Id::unique(("hit", name)),
                Sense::CLICK | Sense::DRAG,
            );
            let gid = egui::Id::unique(("gesture", name));
            let mut g: Gesture = ui.data(|d| d.get_temp(gid)).unwrap_or_default();
            let down = r.is_pointer_button_down_on();
            if down {
                let Some(p) = (if g.down {
                    Self::pointer(ui, view)
                } else {
                    Self::press_origin(ui, view)
                }) else {
                    continue;
                };
                let x = p.x - origin.0;
                let step = if is808 {
                    behavior::tr808_step(x)
                } else {
                    behavior::tr909_step(x)
                };
                if let Some(step) = step {
                    let step = step as i32;
                    if !g.down {
                        g.new_value = if is808 {
                            u8::from(!self.panel.steps808[self.panel.inst808][step as usize])
                        } else {
                            let m = ui.input(|i| Mods {
                                ctrl: i.modifiers.ctrl,
                                alt: i.modifiers.alt,
                            });
                            self.panel.step909_new_value(step as usize, m)
                        };
                        g.last_step = step;
                    }
                    // Every step crossed since the last frame.
                    let (a, b) = (g.last_step.min(step), g.last_step.max(step));
                    for s in a..=b {
                        if is808 {
                            self.panel.set_step808(s as usize, g.new_value != 0);
                        } else {
                            self.panel.set_step909(s as usize, g.new_value);
                        }
                    }
                    g.last_step = step;
                }
            }
            g.down = down;
            ui.data_mut(|d| d.insert_temp(gid, g));
        }
        // The step parts are drawn here, after the hit tests, so a press shows at once.
        let ctls = std::mem::take(&mut self.ctls);
        for c in ctls.iter().filter(|c| c.kind == "steps") {
            let name = self.panel.part_name(c, false, 0.0);
            paint_part(ui, &name, view.rect(c.x, c.y, c.w, c.h));
        }
        self.ctls = ctls;
    }

    /// The legends and name rows select the instrument; dragging keeps selecting.
    /// Also draws the selection frames.
    fn instrument_selection(&mut self, ui: &mut Ui, view: &View) {
        let (o8, o9) = (layout::TR808_ORIGIN, layout::TR909_ORIGIN);
        /// (is the 808, section origin, [x y w h] in the section, name)
        type Area = (bool, (f32, f32), [f32; 4], &'static str);
        let areas: [Area; 4] = [
            (true, o8, [10.0, 71.0, 264.0, 14.0], "legend808"),
            (true, o8, [78.0, 43.0, 106.0, 12.0], "upper808"),
            (false, o9, [4.0, 4.0, 358.0, 14.0], "legend909"),
            (false, o9, [26.0, 74.0, 336.0, 15.0], "names909"),
        ];
        for (is808, o, [x, y, w, h], name) in areas {
            let rect = view.rect(o.0 + x, o.1 + y, w, h);
            // no FOCUSABLE, as the step strips (task-186)
            let r = ui.interact(
                rect,
                egui::Id::unique(("hit", name)),
                Sense::CLICK | Sense::DRAG,
            );
            if !r.is_pointer_button_down_on() {
                continue;
            }
            let Some(p) = Self::pointer(ui, view) else {
                continue;
            };
            let (sx, sy) = (p.x - o.0, p.y - o.1);
            let hit = if is808 {
                behavior::tr808_hit(sx, sy)
            } else {
                behavior::tr909_hit(sx, sy)
            };
            if let Some(i) = hit {
                self.panel.select_instrument(is808, i);
            }
        }
        // The selection frames.
        let i = self.panel.inst808;
        let upper = (3..=7).contains(&i)
            && self.panel.switch_position(&format!("tr808.slot{i}_switch")) == 0;
        let (fx, fy) = if upper {
            (o8.0 + 78.0 + 22.0 * (i - 3) as f32, o8.1 + 42.0)
        } else {
            (o8.0 + 10.0 + 22.0 * i as f32, o8.1 + 71.0)
        };
        paint_part(ui, "sel_22x14", view.rect(fx, fy, 22.0, 14.0));
        let i = self.panel.inst909;
        let (name, x, y, w, h) = match i {
            0 => ("sel_22x14", 4.0, 4.0, 22.0, 14.0),
            1..=5 => ("sel_42x14", 26.0 + 42.0 * (i - 1) as f32, 4.0, 42.0, 14.0),
            _ => ("sel_21x14", 236.0 + 21.0 * (i - 6) as f32, 4.0, 21.0, 14.0),
        };
        paint_part(ui, name, view.rect(o9.0 + x, o9.1 + y, w, h));
    }

    /// With `--screenshot`: after a few frames (the SVGs are loaded and rasterized) save the
    /// window to the file and quit.
    fn screenshot(&mut self, ctx: &egui::Context) {
        let Some(shot) = &mut self.shot else {
            return;
        };
        shot.frames += 1;
        ctx.request_repaint();
        if shot.frames == 8 && !shot.requested {
            shot.requested = true;
            let path = shot.path.clone();
            ctx.request_screenshot(Box::new(move |image: std::sync::Arc<egui::ColorImage>| {
                save_png(&path, &image).expect("writing the screenshot");
                std::process::exit(0);
            }));
        }
    }
}

fn part_size(name: &str) -> (f32, f32) {
    match name {
        "wood" => (16.0, 510.0),
        "bg_transport" => (592.0, 50.0),
        logo_anim::LOGO_PART => (logo_anim::LOGO_W, logo_anim::LOGO_H),
        "bg_pattern" | "bg_mixer" => (70.0, 115.0),
        "bg_tb303" | "bg_tr808" | "bg_tr909" => (366.0, 115.0),
        "bg_master" => (82.0, 98.0),
        "bg_pcf" => (82.0, 111.0),
        "bg_delay" | "bg_comp" => (82.0, 93.0),
        "bg_dist" => (82.0, 65.0),
        _ => panic!("no size for background {name}"),
    }
}

/// The value arc of a knob from angle `a0` to `a` degrees, painted over the ring track.
fn paint_arc(ui: &Ui, view: &View, c: &Ctl, k: &KnobDesign, a0: f32, a: f32) {
    if (a - a0).abs() < 0.5 {
        return;
    }
    let center = view.origin + Vec2::new(c.x + k.cx, c.y + k.cy) * view.scale;
    let r = k.ring_r * view.scale;
    let color = Color32::from_rgb(k.arc[0], k.arc[1], k.arc[2]);
    // The arcs are three quarters of the panel's vertices: each is tessellated once per look
    // (place, angles, width, color, density, clip) and then drawn as that mesh, which egui
    // passes through unchanged (task-153).
    let ctx = ui.ctx();
    let ppp = ctx.pixels_per_point();
    let clip = ui.clip_rect();
    let key = [
        center.x,
        center.y,
        r,
        a0,
        a,
        k.ring_w * view.scale,
        ppp,
        clip.min.x,
        clip.min.y,
        clip.max.x,
        clip.max.y,
    ]
    .map(f32::to_bits);
    let key = (key, color);
    let id = egui::Id::unique("orb-ui knob arcs");
    let cached = ctx.data_mut(|d| {
        d.get_temp_mut_or_default::<ArcMeshes>(id)
            .0
            .get(&key)
            .cloned()
    });
    let mesh = match cached {
        Some(m) => m,
        None => {
            let n = (((a - a0).abs() / 3.0).ceil() as usize).max(2);
            let points: Vec<Pos2> = (0..=n)
                .map(|i| {
                    let t = (a0 + (a - a0) * i as f32 / n as f32).to_radians();
                    center + Vec2::new(t.sin(), -t.cos()) * r
                })
                .collect();
            let line = Shape::line(points, Stroke::new(k.ring_w * view.scale, color));
            let prims = ctx.tessellate(vec![epaint::ClippedShape::new(clip, line)], ppp);
            let [p] = &prims[..] else {
                return;
            };
            let epaint::Primitive::Mesh(m) = &p.primitive else {
                return;
            };
            let m = Arc::new(m.clone());
            ctx.data_mut(|d| {
                let cache = &mut d.get_temp_mut_or_default::<ArcMeshes>(id).0;
                // automation sweeping a knob makes new looks: keep the cache bounded
                if cache.len() >= 4096 {
                    cache.clear();
                }
                cache.insert(key, m.clone());
            });
            m
        }
    };
    ui.painter().add(Shape::Mesh(mesh));
}

/// The knob arcs, tessellated (`paint_arc`), by look.
#[derive(Clone, Default)]
struct ArcMeshes(std::collections::HashMap<([u32; 11], Color32), Arc<epaint::Mesh>>);

fn save_png(path: &PathBuf, image: &egui::ColorImage) -> Result<(), image::ImageError> {
    let [w, h] = image.size;
    let mut raw = Vec::with_capacity(w * h * 4);
    for p in &image.pixels {
        raw.extend_from_slice(&p.to_array());
    }
    image::save_buffer(path, &raw, w as u32, h as u32, image::ColorType::Rgba8)
}
