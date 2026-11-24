//! The window: draws the panel with the current skin at the displayed size and runs the
//! original's interaction (docs/ui_spec.md section 2) on the [`Panel`] model through the one
//! controller (`controller`, task-266).
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
    self, Color32, Image, Pos2, Rect, Ui, Vec2, emath::GuiRounding,
    text_selection::LabelSelectionState,
};

use crate::art;
use crate::browser::{Browser, Mode, Poll};
use crate::document::Document;
use crate::keys::KeyMode;
use crate::layout::{PANEL_H, PANEL_W};
use crate::link::Link;
use crate::logo_anim;
use crate::menu::{Cmd, Job, MENU_H, ModEntry, Then};
use crate::model::Panel;
use crate::settings::Settings;
use crate::shortcuts;
use crate::skin::{Frame, Skin};

/// Where the panel is on screen: the point of the panel's origin and points per panel pixel
/// (per skin unit, for a skin whose units are not the panel's pixels).
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
        Self::fit_size(avail, ppp, Vec2::new(PANEL_W, PANEL_H))
    }

    /// [`View::fit`] for a skin of `size` units (its aspect is the panel's): the scale is
    /// points per skin unit.
    pub fn fit_size(avail: Rect, ppp: f32, size: Vec2) -> Self {
        let scale = (avail.width() / size.x)
            .min(avail.height() / size.y)
            .max(1.0 / size.x);
        let on_screen = size * scale;
        Self {
            origin: (avail.center() - on_screen / 2.0).round_to_pixels(ppp),
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
    /// The open file browser, if any (`open_browser`).
    browser: Option<Browser>,
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
    /// Where a direct paste reads the clipboard text from (task-287): a memory clipboard until
    /// the standalone binary and the plugin's editor switch it to the system's.
    pub clipboard: crate::clipboard::Clipboard,
    /// The JSON Edit > Copy Pattern or Cut Pattern took, for the system clipboard at the end of
    /// the frame.
    pub(crate) clip_text: Option<String>,
    /// What Edit > Undo puts back (`App::undo`).
    pub(crate) undo: Option<crate::menu::Undo>,
    /// The 303 pattern editor's window (Edit > 303 Pattern Editor..., task-274).
    pub(crate) editor: crate::pattern_editor::Editor,
    /// The drum pattern editors' windows, by `drum_editor::Machine::index` (Edit > 808 and 909
    /// Pattern Editor..., tasks 280 and 281).
    pub(crate) drum_editors:
        [crate::drum_editor::DrumEditor; crate::drum_editor::Machine::ALL.len()],
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
    /// The same for the Help > Keyboard Shortcuts window (`App::key_help_ui`).
    pub(crate) key_help_open: bool,
    pub(crate) key_help_serial: u32,
    /// The same for the plugin's Help > Plugin Usage window (`App::plugin_usage_ui`, task-221).
    pub(crate) plugin_usage_open: bool,
    pub(crate) plugin_usage_serial: u32,
    /// The ids of the open windows whose command was chosen again: they come to the front at the
    /// end of the frame (`App::raise_window`, task-283).
    pub(crate) to_front: Vec<egui::Id>,
    /// The classic skin of the kit last shown with View > Classic (task-267): it is
    /// made again when the kit changes.
    classic: Option<(orb_cli::mods::Kit, Arc<crate::skin::classic::ClassicSkin>)>,
    /// What the info window drew last (`App::info_shown`).
    pub(crate) info_shown: Option<crate::info::SongInfo>,
    pub(crate) events_seen: usize,
    pub(crate) allow_close: bool,
    /// The window title last sent (`App::update_title`).
    pub title: String,
    /// The computer keys that are down (`controller`).
    pub(crate) holds: Vec<crate::controller::Hold>,
    /// The look of the main window's panel (task-266): the modern skin unless set.
    skin: Arc<dyn Skin>,
    /// The crossfade between the skins in progress (task-268) and the classic overlay's alpha
    /// of the last frame (1 with the classic skin alone, 0 with the modern one alone).
    fade: Option<crate::skin::fade::Fade>,
    overlay_alpha: f32,
    /// Whether the last frame gave the skins the mod's splash picture (for tests).
    splash_in_frame: bool,
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
            browser: None,
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
            clipboard: crate::clipboard::Clipboard::memory(),
            clip_text: None,
            undo: None,
            editor: Default::default(),
            drum_editors: Default::default(),
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
            key_help_open: false,
            key_help_serial: 0,
            plugin_usage_open: false,
            plugin_usage_serial: 0,
            to_front: Vec::new(),
            classic: None,
            info_shown: None,
            events_seen: 0,
            allow_close: false,
            title: String::new(),
            holds: Vec::new(),
            skin: crate::skin::svg::modern(),
            fade: None,
            overlay_alpha: 0.0,
            splash_in_frame: false,
            shot,
            frame_stats: FrameStats::from_env(),
            last_px_scale: None,
            resize_until: f64::NEG_INFINITY,
            shown: 0,
            plugin: None,
        }
    }

    /// The skin the main window's panel is drawn with.
    pub fn skin(&self) -> &Arc<dyn Skin> {
        &self.skin
    }

    /// Draws the panel with `skin` from the next frame on; the same controller takes its
    /// input (task-266).
    pub fn set_skin(&mut self, skin: Arc<dyn Skin>) {
        self.skin = skin;
    }

    /// View > Classic / Modern (task-267): draws the main window's panel with the modern or the
    /// classic skin from the next frame on and saves the choice in the UI settings. Nothing in
    /// a plugin's editor, which has the modern skin only.
    pub fn set_theme(&mut self, theme: crate::settings::Theme) {
        if self.is_plugin() {
            return;
        }
        let changed = self.settings.theme != theme;
        self.settings.set_theme(theme);
        if let Err(e) = self.settings.save() {
            self.fail(e);
        }
        // the input goes to the target skin from now on; the classic skin stays until the fade
        // to the modern one is over (`show`)
        if theme == crate::settings::Theme::Modern {
            self.skin = crate::skin::svg::modern();
        }
        self.apply_theme();
        if changed && self.classic.is_some() {
            let to = f32::from(theme == crate::settings::Theme::Classic);
            self.fade = Some(crate::skin::fade::Fade::new(self.overlay_alpha, to));
        } else if theme == crate::settings::Theme::Modern {
            self.classic = None;
        }
    }

    /// Whether the last frame's skins got the mod's splash picture.
    pub fn splash_in_frame(&self) -> bool {
        self.splash_in_frame
    }

    /// The classic overlay's alpha of the last frame while a crossfade runs (task-268).
    pub fn fading(&self) -> Option<f32> {
        self.fade.map(|_| self.overlay_alpha)
    }

    /// The theme the main window shows: the settings' in the standalone app, Modern in a
    /// plugin's editor.
    pub fn theme(&self) -> crate::settings::Theme {
        if self.is_plugin() {
            crate::settings::Theme::Modern
        } else {
            self.settings.theme
        }
    }

    /// With the classic theme, the classic skin of the current kit: made when the kit changed
    /// (no mod: the built-in kit's bitmaps; a mod's set that cannot be used is reported once
    /// and the built-in bitmaps are drawn). The modern theme leaves the skin alone (a test may
    /// have set its own).
    fn apply_theme(&mut self) {
        if self.theme() != crate::settings::Theme::Classic {
            return;
        }
        let kit = self.doc.kit.clone();
        if let Some((_, skin)) = self.classic.as_ref().filter(|(k, _)| *k == kit) {
            // (kept through a fade to the modern skin, so the skin may be the modern one)
            self.skin = skin.clone();
            return;
        }
        let skin = crate::skin::classic::ClassicSkin::of_kit(&kit);
        if let Some(e) = &skin.bitmaps().error
            && let orb_cli::mods::Kit::Mod(path) = &kit
        {
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            self.fail(format!(
                "Classic theme: the panel bitmaps of {name} cannot be used ({e}); the built-in \
                 ones are shown."
            ));
        }
        self.skin = skin.clone();
        self.classic = Some((kit, skin));
    }

    /// Opens the file browser for `mode` over the panel (the menu calls this). Ctrl/Cmd+O opens it
    /// for a song.
    pub fn open_browser(&mut self, mode: Mode) {
        self.browser = Some(Browser::new(mode, &self.settings));
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
        // Clicks in the browser, the prompt or the box over it do not dismiss it.
        let blocked = self.browser.is_some() || self.prompt.is_some() || self.error_open();
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
        egui_extras::install_image_loaders(ui.ctx());
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

/// An image of the embedded part `name` painted into `rect` (the windows and the plugin's
/// own parts; a skin paints from its part set, [`paint_part_in`]).
pub(crate) fn paint_part(ui: &Ui, name: &str, rect: Rect) {
    paint_part_rotated(ui, name, rect, None);
}

/// `paint_part` multiplied by `tint` (the logo's fades, task-114).
pub(crate) fn paint_part_tinted(ui: &Ui, name: &str, rect: Rect, tint: Color32) {
    paint_part_in(ui, art::PartSet::embedded(), name, rect, None, tint);
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
    paint_part_in(
        ui,
        art::PartSet::embedded(),
        name,
        rect,
        rot,
        Color32::WHITE,
    );
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

/// The part `name` of `set` painted into `rect` (the control's rectangle; the art reaches its
/// bleed beyond it), turned by `rot` (radians, pivot in the rectangle's fractions) and
/// multiplied by `tint`.
pub fn paint_part_in(
    ui: &Ui,
    set: &art::PartSet,
    name: &str,
    rect: Rect,
    rot: Option<(f32, Vec2)>,
    tint: Color32,
) {
    // A part name no art has (a state the panel does not foresee) draws nothing: it must not
    // close the app (task-156). It is reported once per name.
    let Some(part) = set.part(name) else {
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
    /// Asks for the open window `id` (an `egui::Window` of the one viewport, Middle order) to
    /// come to the front at the end of this frame: its command was chosen while it is open.
    /// Its place, size and contents stay (task-283).
    pub(crate) fn raise_window(&mut self, id: egui::Id) {
        if !self.to_front.contains(&id) {
            self.to_front.push(id);
        }
    }

    fn raise_windows(&mut self, ctx: &egui::Context) {
        for id in self.to_front.drain(..) {
            ctx.move_to_top(egui::LayerId::new(egui::Order::Middle, id));
        }
    }

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
        // the classic theme follows the kit (task-267)
        self.apply_theme();
        let skin = self.skin.clone();
        let layout = skin.layout();
        // a plugin's panel keeps a margin of desk on each side (task-234)
        let area = if self.is_plugin() {
            crate::embed::panel_area(ui.ctx().content_rect())
        } else {
            avail
        };
        let view = View::fit_size(area, ppp, layout.size);
        let now = ui.input(|i| i.time);
        self.track_resize(ui.ctx(), view.scale * ppp, now);
        // the desk and the panel's shadows, around any skin
        crate::backdrop::paint(ui, avail, &view, layout.size);
        self.panel.held = None;

        // While the file browser, the prompt or the error box is open the panel takes no keys
        // and no clicks of its own.
        if self.is_plugin() {
            // a plugin's editor has no keyboard at all (task-201): the host keeps its keys
            self.holds.clear();
            if self.browser.is_none() && self.prompt.is_none() && !self.error_open() {
                self.focus_click(ui, &view, layout);
            }
        } else if self.browser.is_none() && self.prompt.is_none() && !self.error_open() {
            if shortcuts::consume(ui, &shortcuts::NEW) {
                self.new_song();
            } else if shortcuts::consume(ui, &shortcuts::OPEN) {
                self.open_browser(Mode::OpenSong);
            } else if !self.is_plugin() && shortcuts::consume(ui, &shortcuts::SAVE) {
                self.save();
            } else {
                // Ctrl/Cmd+F and +G: the Options menu's two keyboard modes.
                if shortcuts::consume(ui, &shortcuts::KEYS_SYNTH) {
                    self.key_mode = KeyMode::Synth;
                }
                if shortcuts::consume(ui, &shortcuts::KEYS_PATTERNS) {
                    self.key_mode = KeyMode::Patterns;
                }
                // Ctrl/Cmd+E: the focused row's pattern editor, Edit > 303, 808 or 909 Pattern
                // Editor... (tasks 274, 280 and 281)
                if shortcuts::consume(ui, &shortcuts::PATTERN_EDITOR) {
                    self.open_focused_editor();
                }
                if ui.ctx().text_edit_focused() {
                    // typing in a text field (File > Info, task-117): the other keys are the
                    // field's, not the panel's
                    self.holds.clear();
                } else {
                    self.pattern_keys(ui);
                    self.keyboard(ui);
                }
                self.focus_click(ui, &view, layout);
            }
        } else {
            self.holds.clear();
        }

        // All the panel's input first, through the one controller on the skin's layout, then
        // the skin draws the state (task-266).
        let pressed = self.panel_input(ui, &view, layout);
        let lit = self.note_lights(ui, now);
        if layout.logo.is_some() && self.logo_anim && self.logo_start.is_none() {
            // once per app start, from the first frame the logo is drawn
            self.logo_start = Some(now);
        }
        let logo_t = logo_anim::time_now(now, self.logo_start.filter(|_| self.logo_anim));
        // (a fade to the classic skin, which has no mod window, still draws the modern one
        // under it: the picture stays in the frame, task-268)
        let splash = if self.fade.is_some() || layout.controls.iter().any(|c| c.kind == "modview") {
            self.splash_texture(ui.ctx())
        } else {
            None
        };
        self.splash_in_frame = splash.is_some();
        // the crossfade's alpha of this frame and the classic skin it fades (task-268)
        let fading = match (self.fade.as_mut(), self.classic.as_ref()) {
            (Some(fade), Some((_, classic))) => Some((fade.alpha(now), classic.clone())),
            _ => None,
        };
        {
            let hidden = |id: &str| self.hidden(id);
            let frame = Frame {
                panel: &self.panel,
                pressed: &pressed,
                now,
                lit: &lit,
                plugin: self.is_plugin(),
                hidden: &hidden,
                logo_t,
                mod_name: &self.doc.mod_name,
                splash: splash.as_ref(),
            };
            if let Some((alpha, classic)) = &fading {
                // The crossfade (task-268): the modern skin, then the classic image over it.
                // Both are live; the desk and the shadows above are the target skin's.
                let modern = crate::skin::svg::modern();
                let mview = View::fit_size(area, ppp, modern.layout().size);
                modern.paint(ui, &mview, &frame);
                let cview = View::fit_size(area, ppp, classic.layout().size);
                classic.paint_over(ui, &cview, &frame, *alpha);
                ui.ctx().request_repaint();
            } else {
                skin.paint(ui, &view, &frame);
            }
        }
        self.overlay_alpha = match &fading {
            Some((alpha, _)) => *alpha,
            None => f32::from(self.theme() == crate::settings::Theme::Classic),
        };
        if self.fade.is_some_and(|f| f.done(now)) {
            let to = self.fade.take().map_or(0.0, |f| f.to);
            if to == 0.0 {
                self.classic = None;
            }
            self.overlay_alpha = to;
        }
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
        self.info_ui(&ctx);
        self.pattern_editor_ui(&ctx);
        self.drum_editors_ui(&ctx);
        self.about_ui(&ctx);
        self.key_help_ui(&ctx);
        self.plugin_usage_ui(&ctx);
        self.mod_info_ui(&ctx);
        self.raise_windows(&ctx);
        self.poll_export(&ctx);
        if let Some(text) = self.clip_text.take() {
            ctx.copy_text(text);
        }
        self.update_title(&ctx);
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
                std::process::exit(0);
            }));
        }
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
