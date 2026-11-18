//! The window: draws the panel from the SVG parts at the displayed size and runs the
//! original's interaction (docs/ui_spec.md section 2) on the [`Panel`] model.
//!
//! Rendering follows decision task-040: every part is an `egui::Image` of its SVG, rasterized as
//! egui_extras' SVG loader does it (usvg + resvg) at the displayed size in physical pixels; a
//! scale change gives a new, crisp raster. The panel keeps its 624 x 510 aspect ratio; the
//! space around it is the desk of `backdrop` (task-179).

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use orb_engine::samples::SampleSet;

use egui::{
    self, Color32, Image, Key, Pos2, Rect, Response, Sense, Ui, Vec2, emath::GuiRounding,
    text_selection::LabelSelectionState,
};

use crate::a11y::{announce, drive, grid, nav, step303};
use crate::art;
use crate::behavior::{self, Mods};
use crate::browser::{self, Browser, Mode, Poll};
use crate::document::Document;
use crate::embed::{LIT_808, LIT_909, Lit};
use crate::keys::{self, Action, KeyMode, SrAction};
use crate::layout::{self, Ctl, PANEL_H, PANEL_W};
use crate::link::Link;
use crate::logo_anim;
use crate::menu::{Cmd, Job, MENU_H, ModEntry, Then};
use crate::model::Panel;
use crate::numpad;
use crate::settings::Settings;
use crate::shortcuts;

/// Where the panel is on screen: the point of the panel's origin and points per panel pixel.
#[derive(Clone, Copy, Debug)]
pub struct View {
    pub origin: Pos2,
    pub scale: f32,
}

impl View {
    /// The largest panel-shaped area centered in `avail`, snapped to whole pixels. An empty or
    /// inverted `avail` (a plugin window the host made shorter than the menu bar, task-234)
    /// gets a panel one point wide, so the scale stays positive and finite (`to_panel`
    /// divides by it).
    pub fn fit(avail: Rect, ppp: f32) -> Self {
        let scale = (avail.width() / PANEL_W)
            .min(avail.height() / PANEL_H)
            .max(1.0 / PANEL_W);
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
    /// The mod chooser (task-195).
    Chooser,
}

impl Win {
    pub(crate) const ALL: [Win; 5] = [
        Win::KeyHelp,
        Win::Info,
        Win::About,
        Win::ModInfo,
        Win::Chooser,
    ];
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
    /// The pattern Edit > Copy Pattern or Cut Pattern took last.
    pub clip: Option<crate::menu::Clip>,
    /// Its marker, for the system clipboard at the end of the frame.
    pub(crate) clip_text: Option<String>,
    /// The copies made so far (the marker's count).
    pub(crate) clip_count: u32,
    /// What Edit > Undo puts back (`App::undo`).
    pub(crate) undo: Option<crate::menu::Undo>,
    /// Where the Edit menu drew its items last, and whether each was enabled
    /// (`App::edit_item`).
    pub(crate) edit_items: Vec<(&'static str, Rect, bool)>,
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
    /// The kit Mod Info shows when it was not asked for the playing one (Ctrl+I in the mod
    /// chooser, task-195), and its splash.
    pub(crate) mod_info_target: Option<orb_cli::mods::Kit>,
    pub(crate) info_splash: Option<(orb_cli::mods::Kit, Option<egui::TextureHandle>)>,
    /// The mod chooser (task-195) when open, and the number of its opens (its id).
    pub(crate) chooser: Option<crate::mod_chooser::Chooser>,
    pub(crate) chooser_serial: u32,
    /// The same for the Help > Keyboard Shortcuts window (`App::key_help_ui`).
    pub(crate) key_help_open: bool,
    pub(crate) key_help_serial: u32,
    /// The same for the plugin's Help > Plugin Usage window (`App::plugin_usage_ui`, task-221).
    pub(crate) plugin_usage_open: bool,
    pub(crate) plugin_usage_serial: u32,
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
    /// The step (0-based) of the drum grids' cursors, the 808's and the 909's (task-191,
    /// `a11y::grid`); the row is the machine's selected instrument.
    grid_step: [usize; 2],
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
    pub(crate) panel_focus: Option<egui::Id>,
    /// Screen reader mode, the menu bar (task-212): the focus was on a bar title or a menu was
    /// open at the end of the last frame, and the panel control that had the focus before.
    pub(crate) bar_active: bool,
    pub(crate) menu_return: Option<egui::Id>,
    /// A modal (the unsaved-changes prompt, the error box, Replace, the file browser) was open
    /// at the last frame (1), or closed at it so that the focus is to go back now (2), else 0
    /// (`App::window_focus`).
    modal_was: u8,
    /// The focus has been put on the default button of the open prompt / error box (once, on
    /// its first frame, in screen reader mode; task-193).
    pub(crate) prompt_focused: bool,
    pub(crate) error_focused: bool,
    shot: Option<Shot>,
    /// `ORB_UI_FRAME_STATS=1`: the frame rate and cost, printed (`FrameStats`).
    pub frame_stats: Option<FrameStats>,
    /// The panel's pixels per panel pixel in the last frame, and the egui time until which a
    /// change of it counts as a resize in progress (`RESIZE_SETTLE_S`, task-179).
    last_px_scale: Option<f32>,
    resize_until: f64,
    /// The frames `show` has drawn.
    shown: u64,
    /// The plugin, when this is a plugin's editor (`App::embedded`, task-201).
    pub(crate) plugin: Option<crate::embed::Plugin>,
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
            clip: None,
            clip_text: None,
            clip_count: 0,
            undo: None,
            edit_items: Vec::new(),
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
            mod_info_target: None,
            info_splash: None,
            chooser: None,
            chooser_serial: 0,
            key_help_open: false,
            key_help_serial: 0,
            plugin_usage_open: false,
            plugin_usage_serial: 0,
            info_shown: None,
            events_seen: 0,
            allow_close: false,
            title: String::new(),
            holds: Vec::new(),
            ctls: layout::controls(),
            nav: nav::order(&layout::controls()),
            nav_section: None,
            grid_step: [0; 2],
            windows_open: 0,
            window_new: None,
            window_focus_due: None,
            focus_window: None,
            window_tab: false,
            panel_focus: None,
            bar_active: false,
            menu_return: None,
            modal_was: 0,
            prompt_focused: false,
            error_focused: false,
            shot,
            frame_stats: FrameStats::from_env(),
            last_px_scale: None,
            resize_until: f64::NEG_INFINITY,
            shown: 0,
            plugin: None,
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
        let format = self.settings.export_format;
        let request = match mode {
            Mode::Export => Request::Export(format),
            _ => Request::File(mode),
        };
        let Some(picked) = (self.picker)(request, &start, self.parent.as_ref()) else {
            self.after_save = None;
            return;
        };
        let path = match with_extension(mode, picked.clone(), format) {
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
        // The next Export dialog offers the format of this one first (task-204).
        if mode == Mode::Export
            && let Some(f) = browser::ExportFormat::of_path(&path)
            && f != self.settings.export_format
        {
            self.settings.export_format = f;
            if let Err(e) = self.settings.save() {
                self.fail(e);
            }
        }
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
        // the setters: the choice wins over another instance's change to the file (task-230)
        match which {
            Folder::Songs => self.settings.set_songs_dir(Some(dir)),
            Folder::Mods => self.settings.set_mods_dir(Some(dir)),
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
        // nor does a click on a window or menu over the bar (task-235): the click's layer
        // must be the panel's
        let clicked = ui.input(|i| i.pointer.any_click().then(|| i.pointer.interact_pos()));
        if !blocked
            && self.export.is_none()
            && let Some(p) = clicked
            && p.is_none_or(|p| ui.ctx().layer_id_at(p).is_none_or(|l| l == ui.layer_id()))
        {
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

#[cfg(feature = "app")]
impl eframe::App for App {
    fn ui(&mut self, ui: &mut Ui, frame: &mut eframe::Frame) {
        // the run log's "first frame drawn" (task-214); an atomic load after the first
        crate::runlog::frame();
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
///
/// A texture missing in a frame is not rasterized where it is drawn: the frame draws a
/// placeholder handle, and `flush_part_textures` rasterizes every missing part of the frame
/// at once, on all the CPU's cores, and sets the handles before the frame goes to the GPU
/// (task-179: the redesign's filters made the one-by-one rasterizing too slow). During a
/// resize a part is drawn from the texture of its last size, scaled, until the size settles.
#[derive(Clone, Default)]
pub struct PartTextures {
    map: std::collections::HashMap<(usize, u32, u32), PartTexture>,
    /// The size each part (by URI) was last rasterized at: the texture a resize draws.
    latest: std::collections::HashMap<usize, (u32, u32)>,
    /// The pass of the last sweep.
    swept: u64,
    /// The textures drawn as placeholders in this frame, to rasterize (`flush_part_textures`).
    pending: Vec<PendingTexture>,
}

#[derive(Clone)]
struct PendingTexture {
    svg: &'static [u8],
    size: [u32; 2],
    /// A smaller size to rasterize first, when the full size is dense (`PREVIEW_DENSITY`): the
    /// full size follows on a background thread.
    preview: Option<[u32; 2]>,
    handle: egui::TextureHandle,
    options: egui::TextureOptions,
}

/// When a frame misses more than `PREVIEW_MIN_PARTS` parts and a part is denser than 1.25
/// times this many pixels per panel pixel, the part is first rasterized at this density and
/// then, on a background thread, at its full size (task-179): the window shows the whole panel
/// at once, a little soft, and sharp a moment later, instead of waiting for the filters at the
/// full size.
pub const PREVIEW_DENSITY: f32 = 1.0;
pub const PREVIEW_MIN_PARTS: usize = 8;

/// The process's start, for `ORB_UI_START_STATS` (set by `main`).
pub static PROCESS_START: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();

/// `ORB_UI_START_STATS=1` (task-179): prints when the first complete frame was drawn and when
/// the last part reached its full resolution, in ms since the process started.
fn start_stats(what: &str) {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    if *ON.get_or_init(|| std::env::var_os("ORB_UI_START_STATS").is_some_and(|v| v == "1"))
        && let Some(t0) = PROCESS_START.get()
    {
        eprintln!(
            "start stats: {what} at {:.0} ms",
            t0.elapsed().as_secs_f64() * 1000.0
        );
    }
}

/// The id under which a frame notes that its parts must be rasterized at their full size at
/// once (`--screenshot` takes no preview).
fn no_preview_id() -> egui::Id {
    egui::Id::unique("orb-ui no preview")
}

/// Rasterizes `svg` at `size` pixels as egui_extras' SVG loader does.
fn raster(svg: &[u8], size: [u32; 2]) -> Option<egui::ColorImage> {
    let hint = egui::load::SizeHint::Size {
        width: size[0],
        height: size[1],
        maintain_aspect_ratio: false,
    };
    let options = resvg::usvg::Options::default();
    egui_extras::image::load_svg_bytes_with_size(svg, hint, &options).ok()
}

/// Rasterizes the jobs (svg, size) on `threads` threads, the biggest first.
fn raster_all(jobs: &[(&'static [u8], [u32; 2])], threads: usize) -> Vec<Option<egui::ColorImage>> {
    let mut order: Vec<usize> = (0..jobs.len()).collect();
    order.sort_by_key(|&i| std::cmp::Reverse(jobs[i].1[0] * jobs[i].1[1]));
    let images: Vec<std::sync::Mutex<Option<egui::ColorImage>>> =
        jobs.iter().map(|_| Default::default()).collect();
    if threads <= 1 {
        for &i in &order {
            *images[i].lock().unwrap() = raster(jobs[i].0, jobs[i].1);
        }
    } else {
        let next = std::sync::atomic::AtomicUsize::new(0);
        std::thread::scope(|s| {
            for _ in 0..threads.min(jobs.len()) {
                s.spawn(|| {
                    loop {
                        let k = next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        let Some(&i) = order.get(k) else {
                            break;
                        };
                        let image = raster(jobs[i].0, jobs[i].1);
                        *images[i].lock().unwrap() = image;
                    }
                });
            }
        });
    }
    images
        .into_iter()
        .map(|m| m.into_inner().unwrap())
        .collect()
}

/// How long after the last change of the panel's pixel size a resize counts as going on: the
/// parts are drawn from the textures of their old size until then (task-179).
pub const RESIZE_SETTLE_S: f64 = 0.3;

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

    /// The texture of the part `uri` at the size it was last rasterized at, if it is kept.
    fn last_size(&mut self, uri: usize, pass: u64) -> Option<egui::load::SizedTexture> {
        let &(w, h) = self.latest.get(&uri)?;
        self.get(&(uri, w, h), pass)
    }
}

/// Rasterizes the part textures the frame drew as placeholders, on all the CPU's cores (one
/// thread where there are no threads, as in a browser), and sets their images. `App::show`
/// calls it at the end of every frame; a frame drawn outside it calls it before its next part.
pub fn flush_part_textures(ctx: &egui::Context) {
    let pending = ctx.data_mut(|d| {
        std::mem::take(
            &mut d
                .get_temp_mut_or_default::<PartTextures>(PartTextures::id())
                .pending,
        )
    });
    if pending.is_empty() {
        return;
    }
    let threads = std::thread::available_parallelism().map_or(1, |n| n.get());
    // Previews when many parts are missing at once (the start, the end of a resize), not for
    // a part or two (a piece of the logo fading in). Where there are no threads (a browser)
    // there is no background refinement either.
    let preview = threads > 1
        && pending.len() > PREVIEW_MIN_PARTS
        && !ctx.data(|d| d.get_temp(no_preview_id()).unwrap_or(false));
    let now: Vec<_> = pending
        .iter()
        .map(|p| (p.svg, p.preview.filter(|_| preview).unwrap_or(p.size)))
        .collect();
    let images = raster_all(&now, threads);
    let mut later = Vec::new();
    for (mut p, image) in pending.into_iter().zip(images) {
        if let Some(image) = image {
            p.handle.set(image, p.options);
        }
        if preview && p.preview.is_some() {
            later.push(p);
        }
    }
    if later.is_empty() {
        return;
    }
    // the full sizes, on a background thread; a texture dropped meanwhile is set for nothing
    let ctx = ctx.clone();
    std::thread::spawn(move || {
        let jobs: Vec<_> = later.iter().map(|p| (p.svg, p.size)).collect();
        let images = raster_all(&jobs, threads);
        for (mut p, image) in later.into_iter().zip(images) {
            if let Some(image) = image {
                p.handle.set(image, p.options);
            }
        }
        start_stats("all parts at full resolution");
        ctx.request_repaint();
    });
}

pub(crate) fn paint_part_rotated(ui: &Ui, name: &str, rect: Rect, rot: Option<(f32, Vec2)>) {
    paint_part_with(ui, name, rect, rot, Color32::WHITE);
}

/// The texture of `svg` (whose URI is `uri`) at `pixels`, from the context's map, or a new
/// placeholder whose image `flush_part_textures` rasterizes at the end of the frame. While the
/// window is being resized a missing size is drawn from the part's last texture instead.
pub(crate) fn part_texture(
    ctx: &egui::Context,
    uri: &'static str,
    svg: &'static [u8],
    pixels: Vec2,
    units: Vec2,
    options: egui::TextureOptions,
) -> Option<egui::load::SizedTexture> {
    let pass = ctx.cumulative_pass_nr();
    let (now, max) = ctx.input(|i| (i.time, i.max_texture_side));
    let (until, first) = ctx.data(|d| {
        (
            d.get_temp(resize_id()).unwrap_or(f64::NEG_INFINITY),
            d.get_temp(first_frame_id()).unwrap_or(false),
        )
    });
    let resizing = now < until;
    let key = (uri.as_ptr() as usize, pixels.x as u32, pixels.y as u32);
    let found = ctx.data_mut(|d| {
        let t = d.get_temp_mut_or_default::<PartTextures>(PartTextures::id());
        t.get(&key, pass)
            .or_else(|| resizing.then(|| t.last_size(key.0, pass)).flatten())
    });
    if found.is_some() || first {
        return found;
    }
    // A rect too big (a frame drawn before the window settles, a huge window, a high density)
    // would make a texture wider than the GPU allows, which wgpu refuses with a panic.
    // Rasterize it smaller then and let egui scale the texture up: a part this big is mostly
    // plain fill.
    let size = match capped_hint(pixels, max) {
        Some(egui::load::SizeHint::Size { width, height, .. }) => [width, height],
        _ => [pixels.x.max(1.0) as u32, pixels.y.max(1.0) as u32],
    };
    // dense: a preview at PREVIEW_DENSITY first (`flush_part_textures`)
    let preview = (size[0] as f32 > 1.25 * PREVIEW_DENSITY * units.x).then(|| {
        let p = (units * PREVIEW_DENSITY).round();
        [p.x.max(1.0) as u32, p.y.max(1.0) as u32]
    });
    let blank = egui::ColorImage::filled([1, 1], Color32::TRANSPARENT);
    let handle = ctx.load_texture(uri, blank, options);
    let texture = egui::load::SizedTexture::new(handle.id(), pixels);
    ctx.data_mut(|d| {
        let t = d.get_temp_mut_or_default::<PartTextures>(PartTextures::id());
        t.pending.push(PendingTexture {
            svg,
            size,
            preview,
            handle: handle.clone(),
            options,
        });
        t.map.insert(
            key,
            PartTexture {
                handle,
                source_size: pixels,
                used: pass,
            },
        );
        t.latest.insert(key.0, (key.1, key.2));
    });
    Some(texture)
}

/// The id under which `App::show` keeps the egui time a resize counts as going on until.
fn resize_id() -> egui::Id {
    egui::Id::unique("orb-ui resize until")
}

/// The id under which `App::show` notes that this is the window's first frame.
fn first_frame_id() -> egui::Id {
    egui::Id::unique("orb-ui first frame")
}

fn paint_part_with(ui: &Ui, name: &str, rect: Rect, rot: Option<(f32, Vec2)>, tint: Color32) {
    // A part name no art has (a state the panel does not foresee) draws nothing: it must not
    // close the app (task-156). It is reported once per name.
    let Some(part) = art::part(name) else {
        missing_part(name);
        return;
    };
    // `rect` is the control's rectangle; the art reaches `bleed` panel pixels beyond it.
    let k = rect.width() / part.w.max(f32::EPSILON);
    let art = rect.expand(part.bleed * k);
    // The SVG is rasterized at the art's size in pixels, once per size, as egui_extras' SVG
    // loader does it (`load_svg_bytes_with_size` with the default options); the texture is
    // then found here, not through the loaders (task-153: their lookup and `include_bytes`
    // were most of a part's cost per frame).
    let ctx = ui.ctx();
    let ppp = ui.pixels_per_point();
    let pixels = (art.round_to_pixels(ppp).size() * ppp).round();
    let units = Vec2::new(part.w, part.h) + Vec2::splat(2.0 * part.bleed);
    let linear = egui::TextureOptions::LINEAR;
    let Some(texture) = part_texture(ctx, part.uri, part.svg, pixels, units, linear) else {
        return;
    };
    let mut image = Image::from_texture(texture)
        .fit_to_exact_size(art.size())
        .tint(tint);
    if let Some((angle, origin)) = rot {
        // the pivot is given in the control's rectangle; the image is the art's
        let b = part.bleed;
        let pivot = Vec2::new(
            (origin.x * part.w + b) / (part.w + 2.0 * b),
            (origin.y * part.h + b) / (part.h + 2.0 * b),
        );
        image = image.rotate(angle, pivot);
    }
    image.paint_at(ui, art);
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
        // a frame drawn outside `show` may have left part textures to rasterize
        flush_part_textures(ui.ctx());
        // a plugin's editor: the plugin's song when it changed (task-201)
        self.embed_refresh();
        let mut avail = ui.ctx().content_rect();
        if self.menu {
            // the panel takes the window below the menu bar
            avail.min.y += MENU_H;
        }
        let ppp = ui.ctx().pixels_per_point();
        // a plugin's panel keeps a margin of desk on each side (task-234)
        let view = if self.is_plugin() {
            View::fit(crate::embed::panel_area(ui.ctx().content_rect()), ppp)
        } else {
            View::fit(avail, ppp)
        };
        let now = ui.input(|i| i.time);
        self.track_resize(ui.ctx(), view.scale * ppp, now);
        crate::backdrop::paint(ui, avail, &view);
        self.panel.held = None;
        let screen_reader = self.screen_reader_mode();
        // the patterns the 303 step editors (task-190) and the drum grids (task-191) edit,
        // before this frame's input acts
        self.pin_patterns(ui, screen_reader);
        if screen_reader {
            self.follow_section(ui);
            self.window_focus(ui);
        } else {
            self.nav_section = None;
            self.windows_open = 0;
            self.window_new = None;
            self.window_focus_due = None;
            self.focus_window = None;
            self.modal_was = 0;
            // the mode went off with the chooser open: it is a screen reader mode dialog
            self.chooser = None;
        }

        // While the file browser, the prompt or the error box is open the panel takes no keys
        // and no clicks of its own.
        if self.is_plugin() {
            // a plugin's editor has no keyboard at all (task-201): the host keeps its keys
            self.holds.clear();
            if self.browser.is_none() && self.prompt.is_none() && !self.error_open() {
                self.focus_click(ui, &view);
            }
        } else if self.browser.is_none()
            && self.prompt.is_none()
            && self.replace.is_none()
            && !self.error_open()
        {
            if shortcuts::consume(ui, &shortcuts::NEW) {
                self.new_song();
            } else if shortcuts::consume(ui, &shortcuts::OPEN) {
                self.open_browser(Mode::OpenSong);
            } else if !self.is_plugin() && shortcuts::consume(ui, &shortcuts::SAVE) {
                self.save();
            } else if screen_reader && shortcuts::consume(ui, &shortcuts::CHOOSE_MOD) {
                // Ctrl/Cmd+M: Mods > Choose Mod..., screen reader mode only (task-195)
                self.show_chooser();
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
                    self.pattern_keys(ui);
                    self.keyboard(ui);
                }
                self.focus_click(ui, &view);
            }
        } else {
            self.holds.clear();
        }
        // What no key event of this frame claimed is dropped, whether `keyboard` ran or not (a
        // dialog, a text field or the menu bar has the keys): a numpad `*` pressed there does
        // not fire later, and no entry tags a later main-keyboard key (numpad.rs).
        numpad::end_frame();

        for (name, x, y) in layout::backgrounds() {
            // the plugin's transport has its own background (task-201)
            let name = if name == "bg_transport" && self.is_plugin() {
                "bg_transport_plugin"
            } else {
                name
            };
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
            if self.hidden(&c.id) {
                continue;
            }
            let section = nav::section_index(&c.section);
            let target = match section.and_then(|s| groups.get_mut(s)) {
                Some(g) => g,
                None => &mut *ui,
            };
            self.control(target, &view, c, now);
        }
        // the drum grids, in their machines' sections (task-191)
        for (is808, s) in [(true, 3), (false, 4)] {
            if let Some(g) = groups.get_mut(s) {
                self.drum_grid(g, &view, &ctls, is808);
            }
        }
        drop(groups);
        self.ctls = ctls;
        for (name, x, y, w, h) in layout::foregrounds() {
            paint_part(ui, name, view.rect(x, y, w, h));
        }
        self.drum_strips(ui, &view);
        let lit = self.note_lights(ui, now);
        self.instrument_selection(ui, &view, &lit);
        // a plugin's LEDs, menus and resize corner (task-201)
        self.plugin_ui(ui, &view, now);

        // Send what was done, take what the engine did (the panel follows playback).
        self.note_edits();
        if let Some(link) = &mut self.link {
            let before = link.recorded_revision();
            link.sync(&mut self.panel);
            // a plugin's Play light is its preview, not the engine's play state: the host's
            // transport does not light it (task-217)
            if let Some(on) = self.plugin.as_ref().and_then(|p| p.embed.previewing()) {
                self.panel.transport.playing = on;
            }
            // a merge of the recording changed the song's tracks: an edit
            self.doc.modified |= link.recorded_revision() != before;
            // The repaint rule (task-153, docs/ui_spec.md section 16): at the meter rate while
            // the engine's side of the panel moves, else not at all (input repaints). Without
            // a working audio stream the snapshot never moves.
            // (a plugin's engine runs in the host)
            let sound =
                (self.audio_health.is_some() && !self.audio_reported) || self.plugin.is_some();
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
        self.plugin_usage_ui(&ctx);
        self.mod_info_ui(&ctx);
        self.chooser_ui(&ctx);
        self.poll_export(&ctx);
        if let Some(marker) = self.clip_text.take() {
            ctx.copy_text(marker);
        }
        self.update_title(&ctx);
        // last: every message of this frame is in
        announce::show(&ctx, &self.announcer);
        self.close_if_allowed(&ctx);
        // the part textures this frame drew as placeholders, rasterized together
        flush_part_textures(&ctx);
        self.shown += 1;
        if self.shown == 2 {
            // the first frame draws no part (`track_resize`): the second is the first complete
            start_stats("first complete frame");
        }
    }

    /// Notes a change of the panel's pixel scale (a resize, a move to another display): until
    /// it has not changed for `RESIZE_SETTLE_S`, a part missing at the new size is drawn from
    /// its texture at the old size, and the frame after the settling rasterizes the new size.
    ///
    /// The window's first frame rasterizes nothing: eframe draws it before the window has its
    /// size (a content area of 20000 x 20000 points was seen on macOS, task-179), and the
    /// filtered parts at that size took seconds. It draws the desk only and asks for the next
    /// frame at once.
    fn track_resize(&mut self, ctx: &egui::Context, px_scale: f32, now: f64) {
        let first = self.last_px_scale.is_none();
        if self
            .last_px_scale
            .is_some_and(|s| (s - px_scale).abs() > 1e-4)
        {
            self.resize_until = now + RESIZE_SETTLE_S;
        }
        self.last_px_scale = Some(px_scale);
        let until = self.resize_until;
        let shot = self.shot.is_some();
        ctx.data_mut(|d| {
            d.insert_temp(resize_id(), until);
            d.insert_temp(first_frame_id(), first);
            // a screenshot takes the parts at their full size, no preview
            d.insert_temp(no_preview_id(), shot);
        });
        if first {
            ctx.request_repaint();
        } else if now < until {
            ctx.request_repaint_after(Duration::from_secs_f64(until - now));
        }
    }

    /// The Edit menu's keys: Ctrl/Cmd+Z and +D, and egui's Cut, Copy and Paste events
    /// (docs/ui_spec.md section 16). A Cut or Copy while a window's text is selected is the
    /// text's.
    fn pattern_keys(&mut self, ui: &Ui) {
        if shortcuts::consume(ui, &shortcuts::UNDO) {
            self.command(Cmd::Undo);
        }
        if shortcuts::consume(ui, &shortcuts::DUPLICATE) {
            self.command(Cmd::DuplicatePattern);
        }
        let text_selected = ui
            .ctx()
            .plugin_opt::<LabelSelectionState>()
            .is_some_and(|s| s.lock().has_selection());
        for e in ui.input(|i| i.events.clone()) {
            match e {
                egui::Event::Cut if !text_selected => self.command(Cmd::CutPattern),
                egui::Event::Copy if !text_selected => self.command(Cmd::CopyPattern),
                egui::Event::Paste(text) => self.paste_text(&text),
                _ => {}
            }
        }
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
        // Numpad `*` has no egui key: it is Record, as Insert is (a press, and the release that
        // lets go of the button), numpad.rs (task-225). A window that has the keys keeps it.
        for pressed in numpad::take_multiply() {
            if !pressed {
                self.release(Key::Insert);
            } else if focused && !(screen_reader && window_has_keys) {
                let a = Action::Press("transport.record".into());
                self.act(Key::Insert, a, now);
            }
        }
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
            // whether the numpad's key (the hook of numpad.rs; always false off Windows)
            let numpad = numpad::claim(k, physical_key.is_some(), pressed);
            if !pressed {
                self.release(k);
                continue;
            }
            // Screen reader mode, the focus on a 303's step editor: Up, Down, Home and End move
            // its edit step, auto-repeat included; Left and Right do nothing (task-190).
            if !numpad && !modifiers.any() && screen_reader && self.step_editor_key(ui, key, now) {
                continue;
            }
            // Screen reader mode, the focus on a drum grid: the arrows, Home, End and Space
            // (with Ctrl and Alt) are the grid's (task-191), before the value keys and the key
            // map.
            if screen_reader && self.grid_key(ui, key, modifiers, repeat) {
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
                let speech = self.speech();
                drive::step(&mut self.panel, &t, step, speech, now);
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
                // The numpad's Enter is Play whatever has the focus (task-225).
                let enter = k == Key::Enter && !ctrl && !numpad;
                let mapped = match (enter, section) {
                    (true, Some(s)) => match nav::row_of_section(s) {
                        Some(r) if r < 2 => keys::map_screen_reader(
                            k,
                            modifiers.shift,
                            ctrl,
                            false,
                            self.key_mode,
                            r,
                        ),
                        _ => Some(SrAction::Panel(Action::Press("transport.play".into()))),
                    },
                    _ => keys::map_screen_reader(
                        k,
                        modifiers.shift,
                        ctrl,
                        numpad,
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
            if let Some(a) = keys::map(k, modifiers.shift, numpad, self.key_mode, self.panel.focus)
            {
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
        let before303 = step303::Before::of(&self.panel);
        let pressed = match &action {
            Action::Press(id) => Some(id.clone()),
            _ => None,
        };
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
        if let Some(id) = pressed {
            self.announce_303(&before303, &id, self.key_focus);
        }
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
        // a modal that closed gives the focus back: to the window under it, else to the panel
        // control that had it (the modal took it; task-193)
        // (the frame right after the modal's last open frame is too early: egui still treats
        // the panel as under the modal layer, and its widgets give up the focus; so the focus
        // goes back two frames after it, and a repaint is requested so that the second frame
        // comes without any input)
        let was = self.modal_was;
        self.modal_was = if modal {
            1
        } else if was == 1 {
            2
        } else {
            0
        };
        // (only at the closing transition: an open modal does not keep the app repainting)
        if self.modal_was == 2 {
            ui.ctx().request_repaint();
        }
        if was == 2 && !modal {
            if let Some(w) = self.focus_window.filter(|&w| self.win_open(w)) {
                self.window_focus_due = Some(w);
            } else if !open
                && let Some(back) = self.panel_focus
                && !focused.is_some_and(|f| nav::section_of(&self.nav, f).is_some())
            {
                ui.memory_mut(|m| m.request_focus(back));
            }
        }
        if !open && self.windows_open != 0 {
            self.window_focus_due = None;
            self.focus_window = None;
            let on_panel = focused.is_some_and(|f| nav::section_of(&self.nav, f).is_some());
            if let Some(back) = self.panel_focus.filter(|_| !on_panel) {
                ui.memory_mut(|m| m.request_focus(back));
            }
        }
        // Mod Info opened from the mod chooser (Ctrl+I) closed: the focus goes back into the
        // chooser under it (task-195)
        if open
            && self.windows_open & !mask != 0
            && !self.focus_window.is_some_and(|w| self.win_open(w))
            && self.chooser.is_some()
        {
            self.focus_window = Some(Win::Chooser);
            self.window_focus_due = Some(Win::Chooser);
        }
        self.windows_open = mask;
    }

    /// Screen reader mode, end of the menu bar's frame (task-212): `active` is whether the
    /// focus is on a bar title or a menu is open. When it turns on, the panel control that has
    /// the focus is remembered; when it turns off and the focus did not go to a window, a
    /// modal or a panel control (Escape, an item that opens no window), that control gets it
    /// back. A window that opened has its own return (`window_focus`).
    pub(crate) fn menu_bar_focus(&mut self, ui: &Ui, active: bool) {
        let focused = ui.memory(|m| m.focused());
        if active && !self.bar_active {
            self.menu_return = self.panel_focus;
        } else if !active && self.bar_active {
            let busy = self.browser.is_some()
                || self.prompt.is_some()
                || self.replace.is_some()
                || self.error_open()
                || Win::ALL.iter().any(|w| self.win_open(*w));
            let on_panel = focused.is_some_and(|f| nav::section_of(&self.nav, f).is_some());
            if let Some(back) = self.menu_return.filter(|_| !busy && !on_panel) {
                ui.memory_mut(|m| m.request_focus(back));
            }
        }
        self.bar_active = active;
    }

    /// Whether the window `w` is open.
    fn win_open(&self, w: Win) -> bool {
        match w {
            Win::KeyHelp => self.key_help_open,
            Win::Info => self.info_open,
            Win::About => self.about_open,
            Win::ModInfo => self.mod_info_open,
            Win::Chooser => self.chooser.is_some(),
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
            let before303 = step303::Before::of(&self.panel);
            self.mouse_down(&c, now);
            self.announce_key_effect(&before);
            // a 303's button says the step it changed (task-190)
            self.announce_303(&before303, &c.id, Some(f));
        }
    }

    /// Announces what the press of the 303 button `id` did since `before`
    /// (`step303::press_effect`), egui's focus being `focused` (task-190).
    fn announce_303(&mut self, before: &step303::Before, id: &str, focused: Option<egui::Id>) {
        if let Some(text) = step303::press_effect(before, &self.panel, id, focused) {
            self.announce(text);
        }
    }

    /// Keeps the pattern each 303's step editor (task-190, `Synth303::pin`) and each drum grid
    /// (task-191, `Panel::drum_pin`) edits: none with the mode off (the row's, as the
    /// original); in screen reader mode [`step303::pin_of`], so that playback moving a row to
    /// another pattern never changes an editor or a grid without input ("move-along doesn't
    /// speak", decision task-209). The frame's input is a key or mouse button press or an
    /// accessibility action. When a drum pin changes here, the link fills the step strip with
    /// its pattern at once, before the frame's input reads it (`Link::show_patterns`). A pattern
    /// chosen on the panel during the frame (`Panel::press` moves the pin) shows at the frame's
    /// end (`Link::sync`), as a row's pattern change always did: a step set later in that same
    /// frame reads the strip of before, as with the mouse.
    fn pin_patterns(&mut self, ui: &Ui, screen_reader: bool) {
        let input = ui.input(|i| {
            i.events.iter().any(|e| {
                matches!(
                    e,
                    egui::Event::Key { pressed: true, .. }
                        | egui::Event::PointerButton { pressed: true, .. }
                        | egui::Event::AccessKitActionRequest(_)
                )
            })
        });
        let follows = self.settings.step_editor_follows;
        self.panel.pitch_makes_notes = screen_reader && self.settings.pitch_keys_make_notes;
        let drums = self.panel.drum_pin;
        for r in 0..4 {
            let pin = screen_reader
                .then(|| step303::pin_of(&self.panel, r, follows, input))
                .flatten();
            *self.panel.pin_mut(r) = pin;
        }
        if self.panel.drum_pin != drums
            && let Some(link) = &mut self.link
        {
            link.show_patterns(&mut self.panel);
        }
    }

    /// Screen reader mode: a key on a focused 303 step editor (task-190, decision task-209): Up
    /// and Down are its Back and Step buttons (the previous and the next step), Home and End
    /// the first and the pattern's last step (`step303::move_editor`), Left and Right do
    /// nothing. True when the key was the editor's.
    fn step_editor_key(&mut self, ui: &Ui, key: Key, now: f64) -> bool {
        let Some(k) = ui.memory(|m| m.focused()).and_then(step303::editor_of) else {
            return false;
        };
        let Some(m) = step303::Move::of_key(key) else {
            return false;
        };
        step303::move_editor(&mut self.panel, k, m, now);
        true
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
            // A press on a window, a menu or any other layer over the panel is not the
            // panel's (task-235): a window's close button over the logo played the startup
            // sound, and a click on an Edit menu item moved the focus to the row under it.
            if ui.ctx().layer_id_at(p).is_some_and(|l| l != ui.layer_id()) {
                return;
            }
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
                } else if let Some(k) = step303::editor_of_layout(&c.id)
                    && self.screen_reader_mode()
                {
                    // the 303's step editor (task-190): focusable, no click of its own
                    ui.interact(rect, crate::a11y::hit_id(c), Sense::FOCUSABLE);
                    step303::describe(ui.ctx(), &self.panel, k);
                } else {
                    if self.screen_reader_mode() {
                        // a hover-only widget, so that the label's node is in the section's
                        // group (the parent of a node is the Ui of its widget)
                        ui.interact(rect, crate::a11y::hit_id(c), Sense::hover());
                    }
                    crate::a11y::describe_display(ui.ctx(), rect, &self.panel, c);
                }
                // n cells of 9 x 16, centered in the display's well
                let n = (c.w / 9.0) as usize;
                let frames = behavior::digit_frames(self.panel.display(&c.id), n);
                let x0 = c.x + (c.w - 9.0 * n as f32) / 2.0;
                for (i, f) in frames.into_iter().enumerate() {
                    let x = x0 + 9.0 * i as f32;
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
                        // a plugin's pattern drag-out (task-203); nothing in the standalone app
                        if kind == "grid" {
                            self.drag_out(ui, c, &r);
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
                        let before303 = step303::Before::of(&self.panel);
                        self.mouse_down(c, now);
                        self.announce_303(&before303, &c.id, Some(r.id));
                    }
                    crate::a11y::describe(ui.ctx(), r.id, &self.panel, c);
                }
                let name = self.panel.part_name(c, down, now);
                paint_part(ui, &name, rect);
            }
        }
    }

    /// The 808's mod window: the current kit's `splash.jpg` (the embedded standard mod's for the
    /// built-in kit) in the 76 x 76 square at the top under the window's glass, the song's mod
    /// name under it.
    fn mod_view(&mut self, ui: &Ui, view: &View, c: &Ctl) {
        let (px, py, side) = MOD_PICTURE;
        let r = view.rect(c.x + px, c.y + py, side, side);
        if let Some(tex) = self.splash_texture(ui.ctx()) {
            Image::new((tex.id(), r.size())).paint_at(ui, r);
        }
        paint_part(ui, "mod_view_glass", r);
        let top = c.y + MOD_NAME_TOP;
        let r = view.rect(c.x + 2.0, top, c.w - 4.0, c.y + c.h - top - 1.0);
        let galley = ui.painter().layout(
            self.doc.mod_name.clone(),
            egui::FontId::proportional(6.6 * view.scale),
            Color32::from_rgb(0xe8, 0xe2, 0xc2),
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
        let speech = self.speech();
        for req in drive::requests(ui, r.id) {
            drive::request(&mut self.panel, t, req, speech, now);
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
        // A double click sets the knob to its value in a new empty song (docs/ui_spec.md
        // section 2); the demo panel has no link and no such song.
        if r.double_clicked()
            && let Some(v) = self.link.as_ref().and_then(|l| l.knob_default(&c.id))
        {
            self.panel
                .set_knob(&c.id, behavior::knob_from_engine(v as i32));
        }
        // a screen reader's request in a frame where the mouse drags this knob is dropped: the
        // drag sets the position from its own start on every frame and would undo it anyway
        if !down && let Some(t) = drive::target(c) {
            self.drive_value(ui, &r, &t, ui.input(|i| i.time));
        }
        crate::a11y::describe(ui.ctx(), r.id, &self.panel, c);

        // the static body and the pointer turned to the value (no value arc since task-179)
        let k = art::knob_design(&c.part);
        let p = self.panel.knob(&c.id);
        paint_part(ui, &format!("{}_body", k.name), rect);
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
            // the keys' row: 16 cells of 16 px from x 14 (808) and of 21 px from x 26 (909)
            let (origin, x0, y0, w, h) = if is808 {
                (layout::TR808_ORIGIN, 14.0, 90.0, 256.0, 20.0)
            } else {
                (layout::TR909_ORIGIN, 26.0, 85.0, 336.0, 18.0)
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
    ///
    /// `lit` are the voices lit in the plugin's note mode (task-202): a green ring on the
    /// 808's chip of the sound played (the lower legend or the upper chip, by the voice and not
    /// by the switch), replacing the orange one when it is the selected chip, and a green LED
    /// on the 909's name row, replacing the red one on the selected instrument. AC is lit
    /// the same way. Nothing is lit in the standalone app.
    fn instrument_selection(&mut self, ui: &mut Ui, view: &View, lit: &Lit) {
        let (o8, o9) = (layout::TR808_ORIGIN, layout::TR909_ORIGIN);
        /// (is the 808, section origin, [x y w h] in the section, name)
        type Area = (bool, (f32, f32), [f32; 4], &'static str);
        let areas: [Area; 4] = [
            (true, o8, [10.0, 71.0, 264.0, 14.0], "legend808"),
            (true, o8, [78.0, 43.0, 106.0, 12.0], "upper808"),
            (false, o9, [4.0, 4.0, 358.0, 14.0], "legend909"),
            (false, o9, [26.0, 74.0, 336.0, 10.0], "names909"),
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
        // The marks of the selected instruments (task-179): the 808's chip, lighter with an
        // orange ring (the upper label's chip when the shared slot's switch is up), and the
        // 909's chip label lit with its LED in the AC box or the name row.
        let i = self.panel.inst808.min(11);
        let upper = (3..=7).contains(&i)
            && self.panel.switch_position(&format!("tr808.slot{i}_switch")) == 0;
        let selected = (upper, if upper { i - 3 } else { i });
        // the 808's chips lit in note mode, as (upper label, index): AC, then the sounds
        let mut lit808 = Vec::new();
        if lit.ac[LIT_808] {
            lit808.push((false, 0));
        }
        for bit in (0..16).filter(|b| lit.mask[LIT_808] & 1 << b != 0) {
            // the sounds are the legend's chips 1..=11, then the upper labels (LC..MA)
            lit808.push(if bit <= 10 {
                (false, bit + 1)
            } else {
                (true, bit - 11)
            });
        }
        let chip = |(upper, k): (bool, usize)| {
            let (names, (x, y, w, h)) = if upper {
                (&UPPER_808[..], UPPER_CHIP_808)
            } else {
                (&NAMES_808[..], LEGEND_808)
            };
            let x = x + 22.0 * k as f32;
            (names[k], view.rect(o8.0 + x, o8.1 + y, w, h))
        };
        let (name, r) = chip(selected);
        if !lit808.contains(&selected) {
            paint_part(ui, &format!("sel808_{name}"), r);
        }
        for c in lit808 {
            let (name, r) = chip(c);
            // the selected chip stays lighter; its green ring replaces the orange one
            let prefix = if c == selected {
                "litsel808_"
            } else {
                "lit808_"
            };
            paint_part(ui, &format!("{prefix}{name}"), r);
        }
        let i = self.panel.inst909.min(11);
        let (x, w) = chip_909(i);
        let r = view.rect(o9.0 + x, o9.1 + 4.6, w, 9.2);
        paint_part(ui, &format!("sel909_{}", NAMES_909[i]), r);
        // the 909's LEDs lit in note mode: the sounds are the name row's 1..=11, and AC's
        let mut lit909 = Vec::new();
        if lit.ac[LIT_909] {
            lit909.push(0);
        }
        lit909.extend(
            (0..11)
                .filter(|b| lit.mask[LIT_909] & 1 << b != 0)
                .map(|b| b + 1),
        );
        let led_rect = |i: usize| {
            let (cx, cy) = led_909(i);
            view.rect(o9.0 + cx - 2.5, o9.1 + cy - 2.5, 5.0, 5.0)
        };
        // the selection LED, unless the instrument is lit: the green LED replaces the red one
        if !lit909.contains(&i) {
            let led = if i == 0 { "led909ac_on" } else { "led909_on" };
            paint_part(ui, led, led_rect(i));
        }
        for i in lit909 {
            let led = if i == 0 { "led909ac_lit" } else { "led909_lit" };
            paint_part(ui, led, led_rect(i));
        }
    }

    /// Screen reader mode: a drum machine's grid (task-191, `a11y::grid`) in its section's
    /// `Ui`: a child `Ui` that takes the focus (its id is the grid's stop in the focus order)
    /// and whose node is the Grid, with a hover-only widget per cell (12 instruments by the
    /// pattern's length) so that the cells' nodes are its children. It senses no click: the
    /// step strip under it keeps the mouse (`App::drum_strips`).
    fn drum_grid(&mut self, ui: &mut Ui, view: &View, ctls: &[Ctl], is808: bool) {
        let section = if is808 { "tr808" } else { "tr909" };
        let steps: Vec<Rect> = (1..=16)
            .filter_map(|n| {
                let id = format!("{section}.step{n}");
                let c = ctls.iter().find(|c| c.kind == "steps" && c.id == id)?;
                Some(view.rect(c.x, c.y, c.w, c.h))
            })
            .collect();
        let Some(rect) = steps.iter().copied().reduce(|a, b| a.union(b)) else {
            return;
        };
        let gid = grid::grid_id(is808);
        let g = ui.new_child(
            egui::UiBuilder::new()
                .scope_id(gid)
                .max_rect(rect)
                .sense(Sense::FOCUSABLE)
                .accessibility_role(egui::accesskit::Role::Grid)
                .accessibility_label("Steps"),
        );
        let n = grid::length(&self.panel, is808).min(steps.len());
        let cursor = &mut self.grid_step[usize::from(!is808)];
        *cursor = (*cursor).min(n - 1);
        let cursor = *cursor;
        if g.memory(|m| m.has_focus(gid)) {
            // the arrows are the grid's (`App::grid_key`), not egui's focus moves
            let filter = egui::EventFilter {
                vertical_arrows: true,
                horizontal_arrows: true,
                ..Default::default()
            };
            g.memory_mut(|m| m.set_focus_lock_filter(gid, filter));
        }
        let ctx = g.ctx().clone();
        grid::describe_grid(&ctx, &self.panel, is808, cursor, rect);
        for i in 0..grid::ROWS {
            for (s, r) in steps.iter().enumerate().take(n) {
                g.interact(*r, grid::cell_id(is808, i, s), Sense::hover());
                grid::describe_cell(&ctx, &self.panel, is808, (i, s), *r);
            }
        }
    }

    /// Screen reader mode, the focus on a drum grid: what a key does there (`a11y::grid`).
    /// True when the key is the grid's. Space ignores auto-repeat, as the original's keys do.
    fn grid_key(&mut self, ui: &Ui, key: Key, modifiers: egui::Modifiers, repeat: bool) -> bool {
        let Some(is808) = ui.memory(|m| m.focused()).and_then(grid::machine_of) else {
            return false;
        };
        let Some(k) = grid::GridKey::of(key, modifiers) else {
            return false;
        };
        if !(repeat && matches!(k, grid::GridKey::Set(_))) {
            let step = &mut self.grid_step[usize::from(!is808)];
            grid::key(&mut self.panel, is808, step, k);
        }
        true
    }

    /// With `--screenshot`: after a few frames (the SVGs are loaded and rasterized) save the
    /// window to the file and quit. Each frame of the window calls it after [`App::show`].
    pub fn screenshot(&mut self, ctx: &egui::Context) {
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
                crate::runlog::end("the screenshot was saved");
                std::process::exit(0);
            }));
        }
    }
}

/// The size of a background part: its rectangle (a missing part has none and draws nothing).
fn part_size(name: &str) -> (f32, f32) {
    art::part(name).map_or((0.0, 0.0), |p| (p.w, p.h))
}

/// The 808's legend chips (the first's x, y, w, h; 22 px apart), its upper labels' chips, and
/// its instruments' and upper labels' names: the parts `sel808_<name>` (task-179).
const LEGEND_808: (f32, f32, f32, f32) = (11.6, 72.6, 17.8, 9.2);
const UPPER_CHIP_808: (f32, f32, f32, f32) = (78.6, 44.4, 17.8, 8.8);
const NAMES_808: [&str; 12] = [
    "AC", "BD", "SD", "LT", "MT", "HT", "RS", "CP", "CB", "CY", "OH", "CH",
];
const UPPER_808: [&str; 5] = ["LC", "MC", "HC", "CL", "MA"];
/// The 909's instruments: the parts `sel909_<name>`.
const NAMES_909: [&str; 12] = [
    "AC", "BD", "SD", "LT", "MT", "HT", "RS", "CP", "CH", "OH", "CC", "RC",
];
/// The mod window's picture square (x, y, side) and the top of its name, in the window.
const MOD_PICTURE: (f32, f32, f32) = (4.2, 3.7, 76.0);
const MOD_NAME_TOP: f32 = 86.0;

/// A 909 instrument's label on the chips at the top (y 4.6, 9.2 high): (x, w) in the section.
fn chip_909(i: usize) -> (f32, f32) {
    match i {
        0 => (6.4, 18.4),
        1..=5 => (26.8 + 42.0 * (i - 1) as f32, 39.6),
        _ => (
            236.8 + 42.0 * ((i - 6) / 2) as f32 + 19.8 * (i % 2) as f32,
            19.8,
        ),
    }
}

/// A 909 instrument's selection LED: its center in the section (the AC box's for AC, the name
/// row's for the others).
fn led_909(i: usize) -> (f32, f32) {
    match i {
        0 => (10.0, 53.8),
        1..=5 => (32.0 + 42.0 * (i - 1) as f32, 78.3),
        _ => (
            238.4 + 42.0 * ((i - 6) / 2) as f32 + if i.is_multiple_of(2) { 3.6 } else { 32.8 },
            78.3,
        ),
    }
}

#[cfg(test)]
mod geometry_tests {
    use super::*;

    /// The selection marks' places are the art generator's.
    #[test]
    fn the_selection_marks_match_the_art() {
        use orb_ui_art::parts as art;
        assert_eq!(LEGEND_808, art::LEGEND_808);
        assert_eq!(UPPER_CHIP_808, art::UPPER_CHIP_808);
        assert_eq!(NAMES_808, art::NAMES_808);
        assert_eq!(UPPER_808, art::UPPER_808);
        assert_eq!(NAMES_909, art::NAMES_909);
        assert_eq!(MOD_PICTURE, art::MOD_PICTURE);
        for (i, name) in NAMES_909.into_iter().enumerate() {
            let (x, w, _) = art::chip_909(i);
            let close =
                |a: (f32, f32), b: (f32, f32)| (a.0 - b.0).abs() < 1e-3 && (a.1 - b.1).abs() < 1e-3;
            assert!(close(chip_909(i), (x, w)), "chip {i}");
            assert!(close(led_909(i), art::led_909(i)), "led {i}");
            let sel = crate::art::part(&format!("sel909_{name}")).unwrap();
            assert!((sel.w - w).abs() < 1e-3, "sel909 {i}");
        }
        assert_eq!(crate::model::GR_OFF, art::GR_OFF);
    }
}

/// Saves an egui screenshot as a PNG (`--screenshot`).
pub fn save_png(path: &PathBuf, image: &egui::ColorImage) -> Result<(), image::ImageError> {
    let [w, h] = image.size;
    let mut raw = Vec::with_capacity(w * h * 4);
    for p in &image.pixels {
        raw.extend_from_slice(&p.to_array());
    }
    image::save_buffer(path, &raw, w as u32, h as u32, image::ColorType::Rgba8)
}
