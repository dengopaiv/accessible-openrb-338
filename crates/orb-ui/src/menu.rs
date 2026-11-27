//! The menu (task-047): load, save and export the song, choose its mod, the Vintage sound flag
//! and the keyboard mode, with the unsaved-changes prompt, and orb's own pattern commands.
//!
//! What each item does (docs/ui_spec.md section 18):
//! - **File**: New (Ctrl/Cmd+N: the state of the app's start), Open (Ctrl/Cmd+O), Save (Ctrl/Cmd+S;
//!   the texts come from [`crate::shortcuts`]; a song that was never saved goes to Save As), Save
//!   As, Info ([`crate::info`]), Export loop as audio file. The browser (task-046) returns paths
//!   only; the writes happen here and their errors are shown in the message box
//!   ([`crate::error_box`]).
//! - **Edit**: Undo (one level), Cut, Copy, Paste, Duplicate and Clear Pattern on the focused
//!   pattern editor's pattern, else on the focused row's selected pattern ([`App::undo`],
//!   [`App::copy_pattern`], [`App::paste_pattern`], [`App::duplicate_pattern`],
//!   [`App::clear_pattern`], [`App::pattern_target`], task-290), orb's own commands, and the 303-1,
//!   303-2, 808 and 909 Pattern Editor... (Ctrl/Cmd+E on the focused row's,
//!   [`crate::pattern_editor`], [`crate::drum_editor`]); the original's Initialize Loop and
//!   Initialize Song from Pattern Mode, in song mode only ([`App::init_from_pattern`], task-306); a
//!   plugin's Edit menu has the same items with no shortcut texts.
//! - **Mods**: "Standard ReBirth", the mods of the Mods folder (one entry per INFO menu name, as
//!   the original's Mods menu: `sub_40C163`, docs/rbm.md section 14), and a file picked in the
//!   browser. A choice is the original's `sub_42D80D` -> `sub_426FB3`: the song is marked modified,
//!   the new kit goes to the player as song msg 19 ([`crate::link::Link::set_mod`]) and a save
//!   writes the new name. With the preference "Play Mod Startup Sound" on
//!   ([`crate::settings::Settings::startup_sound`]) the new mod's startup sound plays, as it does
//!   when a song is opened and at the start ([`App::start`]), but only when the kit changes
//!   (task-099). "Demo Song" opens the `Demo.rbs` of the current mod as an unsaved song (task-094).
//! - **View** (the standalone app's only): Classic or Modern, the main window's skin (task-267,
//!   [`App::set_theme`]), kept in the UI settings.
//! - **Options**: the Vintage ReBirth Synth Sound flag (GLOB byte 482; Edit menu item 323 of the
//!   original) and the keyboard mode (Ctrl/Cmd+F, Ctrl/Cmd+G; docs/ui_spec.md section 10).

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::mpsc::{Receiver, TryRecvError};

use egui::{
    self, Align, Color32, Context, Frame, Id, Key, Layout, Margin, Modal, Modifiers, RichText,
    Stroke, Ui,
};
use orb_cli::encode::Format;
use orb_cli::info::SongFacts;
use orb_cli::mods::{self, Kit, ModOpts, STANDARD};
use orb_cli::render::{DEFAULT_IDLE_BLOCKS, DEFAULT_QUALITY, Renderer, load_mod};
use orb_engine::frame::LoadRefused;
use orb_engine::samples::SampleSet;

use egui::accesskit::Role;

use crate::a11y;
use crate::app::App;
use crate::browser::{ExportFormat, Mode};
use crate::document::{self, Document};
use crate::keys::KeyMode;
use crate::link::{emptied_pattern, next_empty_pattern};
use crate::palette::{INK, INK_DIM, SLATE_DARK, SLATE_LINE};
use crate::screen_reader::ScreenReaderSetting;
use crate::session;
use crate::settings::Theme;
use crate::shortcuts;
use crate::system_dialog::Folder;

/// The height of the menu bar in points.
pub const MENU_H: f32 = 22.0;

/// What the unsaved-changes prompt guards: the action that waits for its answer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Then {
    /// Open this song (it replaces the one open).
    Open(PathBuf),
    /// Open the demo song of this mod file ([`App::demo_song`]).
    Demo(PathBuf),
    /// Start over with the new song of the app's start ([`App::new_song`]).
    New,
    /// Close the window.
    Quit,
}

/// The prompt's answers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Answer {
    Save,
    DontSave,
    Cancel,
}

/// A menu command.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Cmd {
    New,
    Open,
    Save,
    SaveAs,
    Export,
    /// File > Info: the current song's information window.
    Info,
    /// Help > Keyboard Shortcuts.
    KeyHelp,
    /// Options > Audio Output... (the standalone app only, task-191).
    AudioOutput,
    /// Options > Output Routing... (a plugin's editor only, task-300).
    OutputRouting,
    /// Help > About.
    About,
    /// "Choose file..." of the Mods menu: the browser for a mod.
    ChooseModFile,
    /// "Choose Mod..." of the Mods menu, shown in screen reader mode (task-195): the dialog
    /// with a filter and the list of mods.
    ChooseMod,
    /// A kit from the Mods menu (or a mod file the browser chose).
    UseKit(Kit),
    /// A variant in the submenu of mods that share an INFO name (task-151): plays it, or when
    /// it is the active one, toggles the preference ([`App::choose_variant`]).
    ChooseVariant(PathBuf),
    /// "Demo Song" of the Mods menu: the current mod's `Demo.rbs`.
    DemoSong,
    /// "Refresh List" of the Mods menu: reads the Mods folder again (task-101).
    RefreshMods,
    /// "Info" of the Mods menu: the current mod's splash and details (task-106).
    ModInfo,
    /// View > Classic or View > Modern: the main window's skin (task-267,
    /// [`App::set_theme`]; nothing in a plugin's editor).
    SetTheme(Theme),
    ToggleVintage,
    /// "Play Mod Startup Sound" of the Options menu (the original's Edit > Preferences).
    ToggleStartupSound,
    /// "Screen Reader Mode" of the Options menu (task-185).
    SetScreenReader(ScreenReaderSetting),
    /// "Set Songs Folder..." / "Set Mods Folder..." of the Options menu (task-194): the
    /// system's folder dialog.
    SetFolder(crate::system_dialog::Folder),
    /// "Speak Raw Values" of the Options menu, shown in screen reader mode (task-192).
    ToggleSpeakRawValues,
    /// "Step Editor Follows Playback" of the Options menu, shown in screen reader mode
    /// (task-190; the drum grids too, task-191).
    ToggleStepEditorFollows,
    TogglePitchKeysMakeNotes,
    SetKeyMode(KeyMode),
    /// The plugin's pattern or note mode (the panel's mode selector, a plugin's editor only;
    /// task-217, task-231: the Options menu has no mode items).
    SetPluginMode(crate::embed::PluginMode),
    /// Edit > Undo ([`App::undo`]).
    Undo,
    /// Edit > 303-1 or 303-2 Pattern Editor..., by the 303 (0 or 1;
    /// [`App::open_pattern_editor`], tasks 274 and 290).
    PatternEditor(usize),
    /// Edit > 808 or 909 Pattern Editor... ([`App::open_drum_editor`], tasks 280 and 281).
    DrumEditor(crate::drum_editor::Machine),
    /// Edit > Cut Pattern ([`App::copy_pattern`]).
    CutPattern,
    CopyPattern,
    /// Edit > Paste Pattern from the app's clipboard ([`App::paste_pattern`]).
    PastePattern,
    DuplicatePattern,
    /// Edit > Clear Pattern, no shortcut ([`App::clear_pattern`], task-290).
    ClearPattern,
    /// Edit > Initialize Loop from Pattern Mode, the original's command 306 (song mode only;
    /// [`App::init_from_pattern`], task-306).
    InitLoop,
    /// Edit > Initialize Song from Pattern Mode, the original's command 307: asks first
    /// ([`App::init_song_ui`]).
    InitSong,
}

/// What Edit > Undo puts back: pattern `pattern` of row `row` held `bytes` before the command
/// `what`; after a Duplicate, `select` is the pattern it came from, until the user selects a
/// pattern ([`App::note_edits`]). With `tracks` it is an Initialize command's (task-306): the
/// engine keeps the track lists from before it, and no pattern is involved (`row` is
/// `usize::MAX`, so no pattern editor keeps it).
#[derive(Clone, Debug)]
pub struct Undo {
    what: &'static str,
    pub(crate) row: usize,
    pub(crate) pattern: u8,
    bytes: Vec<u8>,
    select: Option<u8>,
    tracks: bool,
}

/// The names of the four rows on the status line.
pub(crate) const ROW_NAMES: [&str; 4] = ["303-1", "303-2", "808", "909"];
/// The kind of each row's patterns: a pattern pastes into a row of its kind.
pub(crate) const ROW_KINDS: [&str; 4] = ["303", "303", "808", "909"];

/// The name of pattern `p` (0..=31): bank and button, A1 to D8.
pub(crate) fn pattern_name(p: u8) -> String {
    format!("{}{}", (b'A' + p / 8) as char, p % 8 + 1)
}

/// One entry of the Mods menu: a mod of the Mods folder, or the several mods that share an
/// INFO name (ignoring ASCII case; task-151) as one entry with a submenu.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModEntry {
    /// The mod's file; of mods that share a name, the file the name resolves to: the preferred
    /// one, else the first by file name bytes ([`ModEntry::resolve`]).
    pub path: PathBuf,
    /// The INFO menu name (a file that is no mod has none and is not listed).
    pub name: String,
    /// Empty for a mod with a name of its own; else every file of the name in byte order.
    pub variants: Vec<ModVariant>,
    /// Whether the mod of `path` embeds a `Demo.rbs` (task-195: the mod chooser says so).
    pub demo: bool,
}

/// One file of the submenu of mods that share a name.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModVariant {
    pub path: PathBuf,
    /// Whether the file embeds a `Demo.rbs`.
    pub demo: bool,
    /// The mod's INFO version ("1.2"); empty when it says "the app's version".
    pub version: String,
}

impl ModVariant {
    /// The file name.
    pub fn file_name(&self) -> String {
        self.path
            .file_name()
            .map(|f| f.to_string_lossy().into_owned())
            .unwrap_or_default()
    }

    /// The submenu text: the file name, then the version when there is one.
    pub fn label(&self) -> String {
        if self.version.is_empty() {
            self.file_name()
        } else {
            format!("{}  {}", self.file_name(), self.version)
        }
    }
}

impl ModEntry {
    /// Sets `path` to the file the entry's name resolves to: the variant named by `preferred`
    /// if there is one, else the first. Nothing for an entry with no variants.
    pub fn resolve(&mut self, preferred: Option<&str>) {
        let pick = self
            .variants
            .iter()
            .find(|v| Some(v.file_name().as_str()) == preferred)
            .or(self.variants.first());
        if let Some(v) = pick {
            self.path = v.path.clone();
            self.demo = v.demo;
        }
    }

    /// Whether `p` is this entry's mod or one of its variants.
    pub fn has(&self, p: &Path) -> bool {
        self.path == p || self.variants.iter().any(|v| v.path == p)
    }

    /// Every file of the entry.
    pub fn paths(&self) -> Vec<PathBuf> {
        if self.variants.is_empty() {
            vec![self.path.clone()]
        } else {
            self.variants.iter().map(|v| v.path.clone()).collect()
        }
    }
}

/// An export that runs on its own thread.
pub struct Job {
    rx: Receiver<Result<(), String>>,
    path: PathBuf,
}

impl App {
    // ---- the Mods folder ----------------------------------------------------------------

    /// The options `mods::resolve` and the Mods menu look for mods with: the settings' Mods
    /// folder first, then `ORB_MOD_DIR` and the default places (`ModOpts::mod_dir`).
    pub fn mod_opts(&self) -> ModOpts {
        ModOpts {
            mod_dir: self.settings.mods_dir.clone().filter(|d| d.is_dir()),
            ..ModOpts::default()
        }
    }

    /// The mods of the Mods folder, one entry per INFO name (ASCII case ignored), ordered by
    /// that name with ASCII case folded, lowercase first on a tie, then by path
    /// (`mod_name_order`, task-167). The files that share a name are the variants of one entry,
    /// in byte order of the file names, and the first of them owns the name (docs/rbm.md 11.3,
    /// task-151).
    pub fn list_mods(&self) -> Result<Vec<ModEntry>, String> {
        let dir = match self.mod_opts().mod_dir() {
            Ok(Some(d)) => d,
            Ok(None) => return Ok(Vec::new()),
            Err(orb_cli::Fail(_, m)) => return Err(m),
        };
        let found = mods::scan_with_demo(&dir).map_err(|orb_cli::Fail(_, m)| m)?;
        let mut groups: Vec<(String, Vec<(PathBuf, bool)>)> = Vec::new();
        for (path, name, demo) in found {
            let Some(name) = name else { continue };
            match groups.iter_mut().find(|(n, _)| mods::names_match(n, &name)) {
                Some((_, files)) => files.push((path, demo)),
                None => groups.push((name, vec![(path, demo)])),
            }
        }
        // by name, then by the owner's path as the last tie-break (it never fires, because
        // entries with equal names merged above, but it makes the order total)
        groups.sort_by(|(a, fa), (b, fb)| mod_name_order(a, b).then_with(|| fa[0].cmp(&fb[0])));
        Ok(groups
            .into_iter()
            .map(|(name, files)| {
                let variants = if files.len() < 2 {
                    Vec::new()
                } else {
                    files
                        .iter()
                        .map(|(p, demo)| ModVariant {
                            path: p.clone(),
                            demo: *demo,
                            version: session::mod_info(p)
                                .ok()
                                .and_then(|i| i.version)
                                .map(|v| crate::mod_info::format_version(Some(v)))
                                .unwrap_or_default(),
                        })
                        .collect()
                };
                let mut e = ModEntry {
                    path: files[0].0.clone(),
                    name,
                    variants,
                    demo: files[0].1,
                };
                e.resolve(self.settings.preferred_mod(&e.name));
                e
            })
            .collect())
    }

    /// The variant of `entry` the preference names, when it is set and its file is there.
    pub fn preferred_variant<'a>(&self, entry: &'a ModEntry) -> Option<&'a ModVariant> {
        let f = self.settings.preferred_mod(&entry.name)?;
        entry.variants.iter().find(|v| v.file_name() == f)
    }

    /// Clicking the active variant of a shared name (task-151): it becomes the preferred file
    /// of the INFO name, kept in the settings; if it already is, the preference is cleared.
    /// The result is on the status line.
    fn toggle_preferred(&mut self, path: &Path) {
        let name = match session::mod_info(path) {
            Ok(i) => i.name,
            Err(e) => return self.fail(e),
        };
        let Some(file) = path.file_name().and_then(|f| f.to_str()) else {
            return self.fail("The file name is not valid UTF-8 and cannot be stored.");
        };
        let was = self.settings.preferred_mod(&name) == Some(file);
        self.settings
            .set_preferred_mod(&name, (!was).then_some(file));
        self.set_status(if was {
            format!("No preferred file for the mod {name} any more")
        } else {
            format!("{file} is now the preferred file for the mod {name}")
        });
        // the Mods menu's entries follow the preference without reading the folder again
        let pref = self.settings.preferred_mod(&name).map(str::to_owned);
        if let Some(Ok(list)) = &mut self.mod_list {
            for e in list
                .iter_mut()
                .filter(|e| mods::names_match(&e.name, &name))
            {
                e.resolve(pref.as_deref());
            }
        }
        if let Err(e) = self.settings.save() {
            self.fail(e);
        }
    }

    /// A click on a variant of the Mods menu: the active one is made (or stops being) the
    /// preferred file; another one is played for the song, like any mod choice.
    pub fn choose_variant(&mut self, path: PathBuf) {
        if self.doc.kit == Kit::Mod(path.clone()) {
            self.action(|s| s.toggle_preferred(&path));
        } else {
            self.use_kit(Kit::Mod(path));
        }
    }

    /// Reads the Mods folder again (task-101): the Mods menu's list is read here and nowhere
    /// else, at the app's start, by Mods > Refresh List and when the Mods folder setting
    /// changes. The caches that depend on the folder's files go with it: the demo answer of
    /// the current mod and the splash picture of the mod window.
    pub fn refresh_mods(&mut self) {
        self.mod_list = Some(self.list_mods());
        self.demo_cache = None;
        self.splash = None;
    }

    /// Mods > Refresh List: [`App::refresh_mods`], then the result on the status line, as the
    /// export reports its end: "Mod list refreshed: N mods in <folder> (+a, -b)" with what
    /// changed against the previous list (by file), or a read error of the folder in the
    /// message box. The scan is synchronous and fast, so there is no "in progress" line.
    pub fn refresh_mods_reported(&mut self) {
        self.action(|s| s.refresh_mods_reported_now());
    }

    fn refresh_mods_reported_now(&mut self) {
        let paths = |l: &Option<Result<Vec<ModEntry>, String>>| -> Vec<PathBuf> {
            match l {
                Some(Ok(l)) => l.iter().flat_map(ModEntry::paths).collect(),
                _ => Vec::new(),
            }
        };
        let before = paths(&self.mod_list);
        self.refresh_mods();
        match &self.mod_list {
            Some(Ok(list)) => {
                let after = paths(&self.mod_list);
                let added = after.iter().filter(|p| !before.contains(p)).count();
                let removed = before.iter().filter(|p| !after.contains(p)).count();
                let folder = match self.mod_opts().mod_dir() {
                    Ok(Some(d)) => format!(" in {}", d.display()),
                    _ => String::new(),
                };
                let n = list.len();
                let s = if n == 1 { "" } else { "s" };
                self.set_status(format!(
                    "Mod list refreshed: {n} mod{s}{folder} (+{added}, -{removed})"
                ));
            }
            Some(Err(e)) => {
                self.status = None;
                self.fail(e.clone());
            }
            None => {}
        }
    }

    /// The Mods menu's list as last read by [`App::refresh_mods`]; None before the first read.
    pub fn mods(&self) -> Option<&Result<Vec<ModEntry>, String>> {
        self.mod_list.as_ref()
    }

    /// What the Mods menu checks for the current song (task-093): whether "Standard ReBirth" is
    /// checked, and the index in `list` of the checked mod. The song's resolved kit decides: a
    /// mod is the entry of its file, so a mod the song names in another ASCII case is checked,
    /// and a mod chosen
    /// from outside the folder, or a song whose mod is missing (the built-in kit under another
    /// name), checks nothing. The original checks the item whose mod object is the active
    /// song's (`sub_40B977`, 0x40BF0C).
    pub fn mod_checks(&self, list: &[ModEntry]) -> (bool, Option<usize>) {
        match &self.doc.kit {
            Kit::Standard => (self.doc.mod_name == STANDARD, None),
            Kit::Mod(p) => (false, list.iter().position(|e| e.has(p))),
        }
    }

    /// The file of the mod called `mod_name` in the Mods folder: of the files that share the
    /// name, the preferred one if it is there, else the first by file name bytes (task-151).
    /// The one rule of the UI: a song open, and the Mods menu's entries
    /// ([`ModEntry::resolve`]) use it.
    pub fn resolve_mod_file(&self, mod_name: &str) -> Result<Option<PathBuf>, orb_cli::Fail> {
        match self.mod_opts().mod_dir()? {
            Some(dir) => {
                let pref = self.settings.preferred_mod(mod_name);
                mods::find_preferring(&dir, mod_name, pref)
            }
            None => Ok(None),
        }
    }

    /// The kit a song names: the standard kit, or the mod of that name in the Mods folder.
    /// A mod that is not there gives the standard kit and an error.
    fn kit_for(&mut self, mod_name: &str) -> Kit {
        if mod_name == STANDARD {
            return Kit::Standard;
        }
        match self.resolve_mod_file(mod_name) {
            Ok(Some(p)) => Kit::Mod(p),
            Ok(None) => {
                self.fail(format!(
                    "The mod \"{mod_name}\" was not found in the Mods folder; the built-in kit \
                     plays. Choose it in the Mods menu."
                ));
                Kit::Standard
            }
            Err(orb_cli::Fail(_, m)) => {
                self.fail(m);
                Kit::Standard
            }
        }
    }

    /// The drum samples of a kit: the built-in ones, or a mod's on top of them.
    pub(crate) fn kit_samples(&self, kit: &Kit) -> Result<Option<Arc<SampleSet>>, String> {
        match kit {
            Kit::Standard => Ok(self.samples.clone()),
            Kit::Mod(path) => {
                // a mod's missing slots come from the built-in kit
                let builtin = self
                    .samples
                    .clone()
                    .unwrap_or_else(rbm::standard::builtin_samples);
                load_mod(path, &builtin)
                    .map(|s| Some(Arc::new(s)))
                    .map_err(|orb_cli::Fail(_, m)| m)
            }
        }
    }

    // ---- opening ------------------------------------------------------------------------

    /// Whether replacing the song asks first: unsaved changes, with a player. Never in a
    /// plugin's editor, whose song is kept in the host's project and which has no Save
    /// (task-201).
    fn asks_to_save(&self) -> bool {
        self.doc.modified && self.link.is_some() && !self.is_plugin()
    }

    /// Opens the song at `path`: with unsaved changes first asks what to do with them.
    pub fn open_song(&mut self, path: &Path) {
        if self.asks_to_save() {
            self.prompt = Some(Then::Open(path.to_path_buf()));
        } else {
            self.open_now(path);
        }
    }

    /// Opens the song at `path` in the player, replacing the document; a failure goes to the
    /// message box and leaves the song open as it was.
    pub fn open_now(&mut self, path: &Path) {
        self.action(|s| s.open_now_inner(path));
    }

    fn open_now_inner(&mut self, path: &Path) {
        if self.link.is_none() {
            self.fail("There is no player to open the song in.");
            return;
        }
        match session::open_song(path) {
            Ok(opened) => self.adopt(path, opened),
            Err(e) => self.fail(e),
        }
    }

    /// Makes a song that was read the open document: its mod is looked up, the song is loaded
    /// with that kit.
    pub fn adopt(&mut self, path: &Path, opened: session::Opened) {
        self.action(|s| s.adopt_as(Some(path.to_path_buf()), opened, None));
    }

    /// [`App::adopt`] for a song that may have no file (`path` is `None`: Untitled) and may be
    /// given its mod (`forced`: the mod's INFO name and kit) instead of the one it names.
    fn adopt_as(
        &mut self,
        path: Option<PathBuf>,
        opened: session::Opened,
        forced: Option<(String, Kit)>,
    ) {
        let (mod_name, kit) = match forced {
            Some(f) => f,
            None => {
                let name = opened.engine.mod_name.clone();
                let kit = self.kit_for(&name);
                (name, kit)
            }
        };
        // a mod that does not load (not recrypted, unreadable) plays the built-in kit
        let (kit, samples) = match self.kit_samples(&kit) {
            Ok(s) => (kit, s),
            Err(e) => {
                self.fail(e);
                (Kit::Standard, self.samples.clone())
            }
        };
        if self.is_plugin() {
            // the plugin's song (task-201): the mod first (the song is built with it), then
            // the song; the panel shows it at the next refresh, now
            let before = self.doc.kit.clone();
            if let Err(e) = self.embed_mod(&kit) {
                self.fail(e);
            }
            let file = opened.file.clone();
            match self.embed().map(|e| e.load_song(Some(file))) {
                Some(Ok(())) => {
                    self.embed_refresh();
                    if self.doc.kit != before {
                        self.mod_opened();
                    }
                }
                Some(Err(e)) => self.fail(e),
                None => {}
            }
            return;
        }
        let Some(link) = &mut self.link else {
            return;
        };
        // a song the node pool refuses opens no window (sub_40B575's STOP alert): the current
        // song, the panel and the document stay, and no startup sound plays
        if link.load(&opened.engine, samples, &mut self.panel).is_err() {
            self.fail(LoadRefused::ALERT);
            // the start song refused: the player keeps the built-in kit, which counts as the
            // kit that played, so a later song on it plays nothing (task-099)
            self.settled = true;
            return;
        }
        // the song document's constructor `sub_425C4D` opens the mod, then `sub_426C67`
        // (0x425FAB for a 1.x song, 0x426400 for an RBS song): the old song was closed first
        // (the unsaved prompt is the close's), so this song is its mod's only user
        self.settle(path, opened.source, opened.facts_1x, mod_name, kit);
        // task-189: the song and the mod that plays it
        let text = format!("Opened {}, mod {}", self.doc.name(), self.doc.mod_name);
        self.announce(text);
    }

    /// Whether the current song's mod has a `Demo.rbs` (the Mods menu's "Demo Song"): never for
    /// the built-in kit. The answer for a mod file is kept until another mod is asked.
    pub fn has_demo(&mut self) -> bool {
        let Kit::Mod(p) = &self.doc.kit else {
            return false;
        };
        if let Some((cp, has)) = &self.demo_cache
            && cp == p
        {
            return *has;
        }
        let has = session::mod_has_demo(p);
        self.demo_cache = Some((p.clone(), has));
        has
    }

    /// Mods > Demo Song: opens the demo of the current song's mod, like [`App::open_song`]
    /// opens a file (the unsaved prompt first).
    pub fn demo_song(&mut self) {
        let Kit::Mod(p) = self.doc.kit.clone() else {
            return;
        };
        if self.asks_to_save() {
            self.prompt = Some(Then::Demo(p));
        } else {
            self.open_demo_now(&p);
        }
    }

    /// Opens the `Demo.rbs` of the mod file `mod_path` as a song that was never saved
    /// (Untitled, so Save goes to Save As and never writes into the mod), on that mod. The
    /// load is [`App::adopt_as`]'s: the player stops the playing song, and a demo the node pool
    /// refuses opens nothing. A failure goes to the message box.
    pub fn open_demo_now(&mut self, mod_path: &Path) {
        self.action(|s| s.open_demo_inner(mod_path));
    }

    fn open_demo_inner(&mut self, mod_path: &Path) {
        if self.link.is_none() {
            self.fail("There is no player to open the song in.");
            return;
        }
        let label = format!("{}: {}", mod_path.display(), session::DEMO);
        let opened = session::mod_demo(mod_path).and_then(|b| session::open_bytes(&b, &label));
        let info = session::mod_info(mod_path);
        match (opened, info) {
            (Ok(mut opened), Ok(info)) => {
                // the song's mod is the mod it came from, whatever its GLOB names
                opened.source.glob.mod_name = info.name.clone();
                opened.source.glob.mod_ftp = info.ftp;
                opened.source.glob.mod_www = info.web;
                opened.engine.mod_name = info.name.clone();
                self.adopt_as(
                    None,
                    opened,
                    Some((info.name, Kit::Mod(mod_path.to_path_buf()))),
                );
            }
            (Err(e), _) | (_, Err(e)) => self.fail(e),
        }
    }

    /// The song is loaded in the player: its mod's startup sound plays when the preference is
    /// on and the kit changed (task-099), and the document is the song, not modified. The tail
    /// of [`App::adopt`] and [`App::fresh`].
    fn settle(
        &mut self,
        path: Option<PathBuf>,
        source: rbs::Song,
        facts_1x: Option<SongFacts>,
        mod_name: String,
        kit: Kit,
    ) {
        // only a changed kit plays its sound (task-099); the first song has no previous kit
        if !self.settled || self.doc.kit != kit {
            self.mod_opened();
        }
        self.settled = true;
        self.doc = Document {
            source,
            facts_1x,
            path,
            modified: false,
            mod_name,
            kit,
        };
        self.undo = None;
        self.events_seen = 0;
    }

    /// The app's start (`sub_480AA0`, docs/ui_spec.md section 18): the song named on the
    /// command line is opened ([`App::adopt`]); without one the new empty song stays, and its
    /// mod's startup sound plays as the original's Untitled song's on "Standard ReBirth"
    /// (`sub_40A50D` -> `sub_40A581` -> `sub_426CA8`, `sub_426C67` at 0x426D48) when the
    /// preference is on.
    pub fn start(&mut self, song: Option<(PathBuf, session::Opened)>) {
        self.refresh_mods();
        match song {
            Some((path, opened)) => self.adopt(&path, opened),
            None => self.fresh(),
        }
    }

    /// The state of an app started without a song: the empty song, Untitled, on the standard
    /// mod, not modified, stopped, the panel at its values; its mod's startup sound plays when
    /// the preference is on and the kit changed ([`App::mod_opened`], task-099). The settings
    /// stay. [`App::start`] and File > New
    /// ([`App::new_now`]) both come here. Without a player there is nothing to load: the
    /// panel and the document are made new.
    fn fresh(&mut self) {
        if self.is_plugin() {
            // the plugin's new song, on the plugin's mod (task-201)
            match self.embed().map(|e| e.load_song(None)) {
                Some(Ok(())) => self.embed_refresh(),
                Some(Err(e)) => self.fail(e),
                None => {}
            }
            return;
        }
        if self.link.is_none() {
            self.panel = crate::model::Panel::new();
            self.settle(
                None,
                rbs::Song::empty(),
                None,
                STANDARD.into(),
                Kit::Standard,
            );
            return;
        }
        let samples = self.samples.clone();
        if let Some(link) = &mut self.link {
            link.load_new(samples, &mut self.panel);
        }
        self.settle(
            None,
            rbs::Song::empty(),
            None,
            STANDARD.into(),
            Kit::Standard,
        );
    }

    /// File > New: with unsaved changes first asks what to do with them.
    pub fn new_song(&mut self) {
        if self.asks_to_save() {
            self.prompt = Some(Then::New);
        } else {
            self.new_now();
        }
    }

    /// Replaces the open song with the start state (`Link::load_new` stops the playing song).
    pub fn new_now(&mut self) {
        self.fresh();
        self.announce(format!("New song, mod {}", self.doc.mod_name));
    }

    /// `sub_426C67` after a song's mod was opened by its only user (word +1088 is 1,
    /// 0x426C6E): top-level msg 58, the mod's startup sound, when "Play Mod Startup Sound" is
    /// on (`sub_4098C3` at 0x426C85).
    fn mod_opened(&mut self) {
        if !self.settings.startup_sound {
            return;
        }
        if let Some(link) = &mut self.link {
            link.startup_sound();
        }
    }

    // ---- saving -------------------------------------------------------------------------

    /// The song as the panel shows it, as an `.rbs` file's bytes.
    pub fn song_bytes(&mut self) -> Result<Vec<u8>, String> {
        self.song_for_save()?
            .write()
            .map_err(|e| format!("cannot write the song: {e}"))
    }

    fn song_for_save(&mut self) -> Result<rbs::Song, String> {
        let Some(link) = &mut self.link else {
            return Err("There is no song to save.".into());
        };
        let before = link.recorded_revision();
        link.refresh_tracks();
        self.doc.modified |= link.recorded_revision() != before;
        Ok(document::compose(
            &self.doc.source,
            &self.panel,
            link,
            &self.doc.mod_name,
        ))
    }

    /// Save: to the song's file, or Save As for a song that has none.
    pub fn save(&mut self) {
        match self.doc.path.clone() {
            Some(p) => self.save_to(&p),
            None => self.open_browser(Mode::SaveSong),
        }
    }

    /// Writes the song to `path`; it becomes the song's file and the song is no longer modified.
    /// An error is shown in the message box and nothing else changes.
    pub fn save_to(&mut self, path: &Path) {
        self.action(|s| s.save_to_inner(path));
    }

    fn save_to_inner(&mut self, path: &Path) {
        let result = self.song_for_save().and_then(|song| {
            let bytes = song
                .write()
                .map_err(|e| format!("cannot write the song: {e}"))?;
            session::write_file(path, &bytes)?;
            Ok(song)
        });
        match result {
            Ok(song) => {
                self.doc.facts_1x = None;
                self.doc.source = song;
                self.doc.path = Some(path.to_path_buf());
                self.doc.modified = false;
                self.set_status(format!("Saved {}", path.display()));
                if let Some(t) = self.after_save.take() {
                    self.proceed(t);
                }
            }
            Err(e) => {
                self.after_save = None;
                self.fail(e);
            }
        }
    }

    // ---- exporting ----------------------------------------------------------------------

    /// Starts the export of the loop to `path` (the format is its extension) on its own thread:
    /// the song as saved is rendered by `orb render`'s code (`orb_cli::render`), so the file is
    /// the one `orb render` writes for the saved song.
    pub fn export_to(&mut self, path: &Path) {
        self.action(|s| s.export_to_inner(path));
    }

    fn export_to_inner(&mut self, path: &Path) {
        let Some(format) = ExportFormat::of_path(path).map(|f| match f {
            ExportFormat::Wav => Format::Wav,
            ExportFormat::Flac => Format::Flac,
            ExportFormat::Ogg => Format::Ogg,
        }) else {
            self.fail(format!(
                "{}: the export's extension must be .wav, .flac or .ogg",
                path.display()
            ));
            return;
        };
        if self.export.is_some() {
            self.fail("An export is still running.");
            return;
        }
        let song = self.song_bytes().and_then(|bytes| {
            let parsed = rbs::Song::parse(&bytes).map_err(|e| e.to_string())?;
            let mut song = orb_engine::load::convert(&parsed).map_err(|e| format!("{e:?}"))?;
            // the kit that plays, whatever the name says (a mod that was not found keeps its
            // name in the song)
            if matches!(self.doc.kit, Kit::Standard) {
                song.mod_name = STANDARD.into();
            }
            Ok(song)
        });
        let song = match song {
            Ok(s) => s,
            Err(e) => {
                self.fail(e);
                return;
            }
        };
        let mods = ModOpts {
            mod_file: match &self.doc.kit {
                Kit::Mod(p) => Some(p.clone()),
                Kit::Standard => None,
            },
            ..ModOpts::default()
        };
        let out = path.to_path_buf();
        let label = self.doc.name();
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let mut r = Renderer::new(mods);
            let result = r
                .render_to_file(
                    &label,
                    &song,
                    DEFAULT_IDLE_BLOCKS,
                    format,
                    DEFAULT_QUALITY,
                    &out,
                )
                .map_err(|orb_cli::Fail(_, m)| m);
            let _ = tx.send(result);
        });
        self.set_status(format!("Exporting {} ...", path.display()));
        self.export = Some(Job {
            rx,
            path: path.to_path_buf(),
        });
    }

    /// Takes the result of a running export.
    pub fn poll_export(&mut self, ctx: &Context) {
        let Some(job) = &self.export else {
            return;
        };
        match job.rx.try_recv() {
            Ok(Ok(())) => {
                let text = format!("Exported {}", job.path.display());
                self.set_status(text);
                self.export = None;
            }
            Ok(Err(e)) => {
                self.status = None;
                self.fail(e);
                self.export = None;
            }
            Err(TryRecvError::Empty) => {
                ctx.request_repaint_after(std::time::Duration::from_millis(50))
            }
            Err(TryRecvError::Disconnected) => {
                self.status = None;
                self.fail("The export stopped unexpectedly.");
                self.export = None;
            }
        }
    }

    /// Whether an export is running.
    pub fn exporting(&self) -> bool {
        self.export.is_some()
    }

    // ---- the mod ------------------------------------------------------------------------

    /// Changes the song's kit (`sub_426FB3`): the same kit again does nothing; otherwise the
    /// song is marked modified, its mod name becomes the kit's, and the player takes the kit's
    /// samples. A failure (a mod that is not recrypted) is shown and changes nothing.
    pub fn use_kit(&mut self, kit: Kit) {
        self.action(|s| s.use_kit_inner(kit));
    }

    fn use_kit_inner(&mut self, kit: Kit) {
        let name = match &kit {
            Kit::Standard => STANDARD.to_string(),
            Kit::Mod(p) => match session::mod_info(p) {
                Ok(info) => info.name,
                Err(e) => {
                    self.fail(e);
                    return;
                }
            },
        };
        if kit == self.doc.kit && name == self.doc.mod_name {
            return;
        }
        let samples = match self.kit_samples(&kit) {
            Ok(s) => s,
            Err(e) => {
                self.fail(e);
                return;
            }
        };
        // a plugin's editor: the plugin takes the mod into its state (task-201)
        if let Err(e) = self.embed_mod(&kit) {
            self.fail(e);
            return;
        }
        let Some(link) = &mut self.link else {
            self.fail("There is no player to change the mod of.");
            return;
        };
        if let Some(samples) = samples {
            link.set_mod(samples, &mut self.panel);
            // `sub_426C67` after the change (0x427030): orb has one song, the only user of
            // its mod; a kit that did not change plays nothing (task-099)
            if kit != self.doc.kit {
                self.mod_opened();
            }
        }
        // the save writes the mod's INFO URLs with its name (`sub_4270F7`, docs/rbm.md 14.1)
        let (ftp, web) = match &kit {
            Kit::Mod(p) => session::mod_info(p)
                .map(|i| (i.ftp, i.web))
                .unwrap_or_default(),
            Kit::Standard => Default::default(),
        };
        self.set_song_mod(kit, name, ftp, web);
        self.announce(format!("Mod {}", self.doc.mod_name));
    }

    /// Records the song's mod: the kit, the INFO name and the two URLs a save writes. The song
    /// is marked modified when what a save writes changes: the name or an URL. Another file of
    /// the same name and URLs (a variant of the Mods menu, task-151) leaves the song as it is;
    /// one with other URLs (dejenerate.rbm and dejenerate(b3).rbm) changes the save.
    pub fn set_song_mod(&mut self, kit: Kit, name: String, ftp: String, web: String) {
        let glob = &mut self.doc.source.glob;
        let changed = self.doc.mod_name != name || glob.mod_ftp != ftp || glob.mod_www != web;
        glob.mod_ftp = ftp;
        glob.mod_www = web.clone();
        if let Some(f) = &mut self.doc.facts_1x {
            f.mod_ftp = self.doc.source.glob.mod_ftp.clone();
            f.mod_www = web;
        }
        self.doc.kit = kit;
        if changed {
            self.doc.modified = true;
        }
        self.doc.mod_name = name;
    }

    // ---- commands -----------------------------------------------------------------------

    /// Runs a menu command.
    pub fn command(&mut self, cmd: Cmd) {
        // a plugin's editor has no Save, Save As, Export or keyboard (task-201)
        if self.is_plugin()
            && matches!(
                cmd,
                Cmd::Save
                    | Cmd::SaveAs
                    | Cmd::Export
                    | Cmd::KeyHelp
                    | Cmd::AudioOutput
                    | Cmd::SetKeyMode(_)
            )
        {
            return;
        }
        match cmd {
            Cmd::New => self.new_song(),
            Cmd::Open => self.open_browser(Mode::OpenSong),
            Cmd::Save => self.save(),
            Cmd::SaveAs => self.open_browser(Mode::SaveSong),
            Cmd::Export => {
                if self.panel.transport.song_mode {
                    self.open_browser(Mode::Export);
                } else {
                    self.fail(
                        "Export needs song mode: the original's export is disabled in pattern \
                         mode.",
                    );
                }
            }
            Cmd::Info => self.show_info(),
            Cmd::KeyHelp => self.show_key_help(),
            Cmd::AudioOutput => self.show_audio(),
            Cmd::OutputRouting => self.show_routing(),
            Cmd::About => self.show_about(),
            Cmd::ChooseModFile => self.open_browser(Mode::ChooseMod),
            Cmd::ChooseMod => self.show_chooser(),
            Cmd::UseKit(kit) => self.use_kit(kit),
            Cmd::ChooseVariant(p) => self.choose_variant(p),
            Cmd::DemoSong => self.demo_song(),
            Cmd::RefreshMods => self.refresh_mods_reported(),
            Cmd::ModInfo => self.show_mod_info(),
            Cmd::SetTheme(t) => self.set_theme(t),
            Cmd::ToggleVintage => {
                let on = !self.panel.vintage;
                self.panel.set_vintage(on);
            }
            Cmd::ToggleStartupSound => {
                let on = !self.settings.startup_sound;
                self.settings.set_startup_sound(on);
                if let Err(e) = self.settings.save() {
                    self.fail(e);
                }
            }
            Cmd::SetScreenReader(v) => {
                if self.settings.screen_reader != v {
                    self.settings.screen_reader = v;
                    if let Err(e) = self.settings.save() {
                        self.fail(e);
                    }
                }
            }
            Cmd::SetFolder(which) => self.choose_folder(which),
            Cmd::ToggleSpeakRawValues => {
                self.settings.speak_raw_values = !self.settings.speak_raw_values;
                if let Err(e) = self.settings.save() {
                    self.fail(e);
                }
            }
            Cmd::ToggleStepEditorFollows => {
                self.settings.step_editor_follows = !self.settings.step_editor_follows;
                if let Err(e) = self.settings.save() {
                    self.fail(e);
                }
            }
            Cmd::TogglePitchKeysMakeNotes => {
                self.settings.pitch_keys_make_notes = !self.settings.pitch_keys_make_notes;
                if let Err(e) = self.settings.save() {
                    self.fail(e);
                }
            }
            Cmd::SetKeyMode(m) => self.set_key_mode(m),
            Cmd::SetPluginMode(m) => {
                if let Some(e) = self.embed() {
                    e.set_mode(m);
                }
            }
            Cmd::Undo => self.undo(),
            Cmd::PatternEditor(k) => self.open_pattern_editor(k),
            Cmd::DrumEditor(m) => self.open_drum_editor(m),
            Cmd::CutPattern => self.copy_pattern(true),
            Cmd::CopyPattern => self.copy_pattern(false),
            Cmd::PastePattern => self.paste_pattern(),
            Cmd::DuplicatePattern => self.duplicate_pattern(),
            Cmd::ClearPattern => self.clear_pattern(),
            Cmd::InitLoop => self.init_from_pattern(false),
            Cmd::InitSong => {
                if self.can_init() {
                    self.confirm_init = true;
                }
            }
        }
    }

    // ---- patterns -----------------------------------------------------------------------

    /// The pattern the Edit menu's pattern commands act on (task-290), after the panel's
    /// pending events were noted and sent: the selected pattern of the focused pattern
    /// editor's row ([`App::focused_editor`]; the editor shows it), else of the panel's focused
    /// row; `None` without a link (the demo panel).
    pub fn pattern_target(&mut self) -> Option<(usize, u8)> {
        self.link.as_ref()?;
        self.note_edits();
        self.link.as_mut()?.forward(&mut self.panel);
        let r = self.focused_editor().unwrap_or(self.panel.focus);
        Some((r, self.panel.rows[r].pattern))
    }

    /// The row of the pattern editor that has the focus, when its window is open: 0 and 1 the
    /// 303-1's and the 303-2's, 2 the 808's, 3 the 909's. An editor takes the focus when it
    /// opens (the Edit menu, Ctrl/Cmd+E) and when a primary press goes down on its window; it
    /// loses it when a primary press goes down on the panel or its window closes (task-290).
    pub fn focused_editor(&self) -> Option<usize> {
        use crate::drum_editor::Machine;
        let r = self.editor_focus?;
        let open = match Machine::of_row(r) {
            Some(m) => self.drum_editors[m.index()].open,
            None => self.editors[r.min(1)].open,
        };
        open.then_some(r)
    }

    /// The editor of row `r` closed: it no longer has the focus.
    pub(crate) fn editor_unfocus(&mut self, r: usize) {
        if self.editor_focus == Some(r) {
            self.editor_focus = None;
        }
    }

    /// Writes `bytes` into pattern `p` of row `r` ([`crate::link::Link::set_pattern`]) for the
    /// command `what`: when the pattern changes the song is modified and Undo puts its former
    /// bytes back. False when it held `bytes` already.
    pub(crate) fn write_pattern(
        &mut self,
        r: usize,
        p: u8,
        bytes: &[u8],
        what: &'static str,
    ) -> bool {
        let Some(link) = &mut self.link else {
            return false;
        };
        let before = link.pattern(r, p).to_vec();
        let changed = link.set_pattern(r, p, bytes, &mut self.panel);
        if changed {
            self.doc.modified = true;
            self.undo = Some(Undo {
                what,
                row: r,
                pattern: p,
                bytes: before,
                select: None,
                tracks: false,
            });
        }
        changed
    }

    /// The command Edit > Undo undoes ("Paste Pattern"); `None` when there is none.
    pub fn undo_name(&self) -> Option<&'static str> {
        self.undo.as_ref().map(|u| u.what)
    }

    /// Edit > Undo, one level: the pattern the last Cut, Paste or Duplicate Pattern wrote gets
    /// its former bytes back, and after a Duplicate the pattern it came from is selected again
    /// (unless a pattern was selected since). The selection goes first: a pattern the row does
    /// not select then reaches the engine only once the engine no longer plays it
    /// ([`crate::link::Link::set_pattern_unplayed`]), so a playing row never plays the emptied
    /// copy for the rest of the bar. After Initialize Loop or Song (orb's addition, task-306)
    /// the engine swaps the track lists it kept back in ([`orb_player::Command::UndoInit`]).
    pub fn undo(&mut self) {
        // the panel's pending edits go first, and may forget the undo
        if self.pattern_target().is_none() {
            return;
        }
        let Some(u) = self.undo.take() else {
            return;
        };
        if u.tracks {
            // orb's own (task-306): the engine swaps the kept track lists back and applies them
            // at the position, as the command did
            if let Some(link) = &mut self.link {
                link.send([orb_player::Command::UndoInit]);
                self.doc.modified = true;
                self.set_status(format!("{} undone", u.what));
            }
            return;
        }
        if let Some(p) = u.select {
            // a click's selection: an edit, recorded in song mode with record on
            self.select_pattern(u.row, p);
            self.note_edits();
        }
        let Some(link) = &mut self.link else {
            return;
        };
        let changed = if self.panel.rows[u.row].pattern == u.pattern {
            link.set_pattern(u.row, u.pattern, &u.bytes, &mut self.panel)
        } else {
            link.set_pattern_unplayed(u.row, u.pattern, &u.bytes, &mut self.panel)
        };
        self.doc.modified |= changed;
        self.set_status(format!("{}: {} undone", ROW_NAMES[u.row], u.what));
    }

    /// Edit > Copy Pattern, and with `cut` Cut Pattern: the target pattern
    /// ([`App::pattern_target`]) goes to the system clipboard as JSON ([`crate::pattern_json`], at
    /// the end of the frame); Cut then empties its steps, the length and the shuffle flag stay.
    pub fn copy_pattern(&mut self, cut: bool) {
        if let Some((r, p)) = self.pattern_target() {
            self.copy_pattern_at(r, p, cut);
        }
    }

    /// [`App::copy_pattern`] of pattern `p` of row `r` (the 303 pattern editor's Copy, after
    /// the panel's events were sent).
    pub(crate) fn copy_pattern_at(&mut self, r: usize, p: u8, cut: bool) {
        let Some(link) = &self.link else {
            return;
        };
        let bytes = link.pattern(r, p).to_vec();
        let (row, name) = (ROW_NAMES[r], pattern_name(p));
        let text = crate::pattern_json::to_json(r, &bytes);
        // a memory clipboard has it at once; the system's gets it at the end of the frame
        self.clipboard.copied(&text);
        self.clip_text = Some(text);
        let empty = emptied_pattern(r, &bytes);
        if cut {
            self.write_pattern(r, p, &empty, "Cut Pattern");
            self.set_status(format!("{row}: {name} cut"));
        } else {
            self.set_status(format!("{row}: {name} copied"));
        }
    }

    /// Edit > Paste Pattern: the system clipboard's pattern (read directly,
    /// [`crate::clipboard::Clipboard::read`]) into the target pattern ([`App::pattern_target`]),
    /// when its row is of the pattern's device (a 303 pattern pastes into either 303).
    pub fn paste_pattern(&mut self) {
        if let Some((r, p)) = self.pattern_target() {
            self.paste_pattern_at(r, p);
        }
    }

    /// [`App::paste_pattern`] into pattern `p` of row `r` (the pattern editors' Paste, after
    /// the panel's events were sent).
    pub(crate) fn paste_pattern_at(&mut self, r: usize, p: u8) {
        match self.clipboard.read() {
            Some(text) => self.paste_text_at(r, p, &text),
            None => self.set_status("The clipboard holds no pattern"),
        }
    }

    /// `text`, the clipboard's text, into pattern `p` of row `r`: nothing is written, and the
    /// status says why, unless the text is a valid pattern of the row's device.
    pub(crate) fn paste_text_at(&mut self, r: usize, p: u8, text: &str) {
        use crate::pattern_json::{Refused, parse};
        let pattern = match parse(text) {
            Ok(pattern) => pattern,
            Err(Refused::NotPattern) => {
                self.set_status("The clipboard holds no pattern");
                return;
            }
            Err(Refused::Invalid(why)) => {
                self.set_status(format!("The clipboard's pattern is not valid: {why}"));
                return;
            }
        };
        if pattern.device != ROW_KINDS[r] {
            self.set_status(format!("The clipboard holds a {} pattern", pattern.device));
            return;
        }
        let bytes = pattern.bytes;
        let (row, name) = (ROW_NAMES[r], pattern_name(p));
        let text = if self.write_pattern(r, p, &bytes, "Paste Pattern") {
            format!("{row}: pasted into {name}")
        } else {
            format!("{row}: {name} holds the pattern already")
        };
        // through set_status: announced in screen reader mode (task-189, task-230)
        self.set_status(text);
    }

    /// Edit > Duplicate Pattern: the target pattern ([`App::pattern_target`]) goes into its
    /// row's next empty pattern ([`next_empty_pattern`]), which is then selected as a click on
    /// its button selects it; a focused editor shows the copy too. The clipboard stays as it
    /// is.
    pub fn duplicate_pattern(&mut self) {
        let Some((r, p)) = self.pattern_target() else {
            return;
        };
        let Some(link) = &self.link else {
            return;
        };
        let row = ROW_NAMES[r];
        let Some(to) = next_empty_pattern(r, &link.patterns()[r], p) else {
            self.set_status(format!("{row}: no empty pattern left"));
            return;
        };
        let bytes = link.pattern(r, p).to_vec();
        let before = link.pattern(r, to).to_vec();
        // the pattern first, then the selection: the engine never plays `to` before it holds it
        self.write_pattern(r, to, &bytes, "Duplicate Pattern");
        self.select_pattern(r, to);
        // the selection goes now, so the next frame's selections are the user's (which make
        // Undo leave the selection alone)
        self.note_edits();
        if let Some(link) = &mut self.link {
            link.forward(&mut self.panel);
        }
        // Undo selects `p` again, also when `to` held the bytes already
        self.undo = Some(Undo {
            what: "Duplicate Pattern",
            row: r,
            pattern: to,
            bytes: before,
            select: Some(p),
            tracks: false,
        });
        let (from, to) = (pattern_name(p), pattern_name(to));
        self.set_status(format!("{row}: {from} duplicated to {to}"));
    }

    /// Edit > Clear Pattern (no shortcut, task-290): the target pattern's steps are emptied as
    /// the editors' Clear empties them ([`emptied_pattern`]: the shuffle flag and the length
    /// stay), one write with Undo; the status line says "cleared" also when the pattern was
    /// empty already (then nothing is written and the Undo stays).
    pub fn clear_pattern(&mut self) {
        if let Some((r, p)) = self.pattern_target() {
            self.clear_pattern_at(r, p);
        }
    }

    /// [`App::clear_pattern`] of pattern `p` of row `r` (also the pattern editors' Clear, after
    /// the panel's events were sent).
    pub(crate) fn clear_pattern_at(&mut self, r: usize, p: u8) {
        let Some(link) = &self.link else {
            return;
        };
        let empty = emptied_pattern(r, link.pattern(r, p));
        // the status also when the pattern was empty already (the user's review, round 1)
        self.write_pattern(r, p, &empty, "Clear Pattern");
        self.set_status(format!("{}: {} cleared", ROW_NAMES[r], pattern_name(p)));
    }

    /// A paste of the system clipboard's `text` (Ctrl/Cmd+V, the standalone's Edit > Paste
    /// Pattern) into the target pattern ([`App::pattern_target`]). Nothing without a link.
    pub fn paste_text(&mut self, text: &str) {
        if self.link.is_none() {
            return;
        }
        if let Some((r, p)) = self.pattern_target() {
            self.paste_text_at(r, p, text);
        }
    }

    /// Whether the Initialize commands are enabled: a song in song mode, as the original
    /// enables commands 306 and 307 only when the song window's song-mode byte is set
    /// (`sub_42BC83`, 0x42BFB4 and 0x42BFC3, under the test of `+0x1060` at 0x42BF24).
    pub fn can_init(&self) -> bool {
        self.link.is_some() && self.panel.transport.song_mode
    }

    /// Edit > Initialize Loop from Pattern Mode (`song` false) and, once its question is
    /// answered, Initialize Song from Pattern Mode (`song` true): the original's commands 306 and
    /// 307 (`sub_42C204`, 0x42C59A and 0x42C5E3), song msgs 29 and 28 to the engine
    /// ([`orb_engine::frame::Engine::init_loop`], [`orb_engine::frame::Engine::init_song`]),
    /// then the song is marked modified (`sub_42590B`). Nothing outside song mode.
    pub fn init_from_pattern(&mut self, song: bool) {
        if !self.can_init() {
            return;
        }
        let Some(link) = &mut self.link else {
            return;
        };
        // the engine copies its lists into this pool before the command, for Undo; made here,
        // off the audio thread, with its whole capacity (task-306)
        let snapshot = Box::new(orb_engine::seq::Pool::new());
        link.send([if song {
            orb_player::Command::InitSong(snapshot)
        } else {
            orb_player::Command::InitLoop(snapshot)
        }]);
        self.doc.modified = true;
        self.undo = Some(Undo {
            what: if song {
                "Initialize Song"
            } else {
                "Initialize Loop"
            },
            row: usize::MAX,
            pattern: 0,
            bytes: Vec::new(),
            select: None,
            tracks: true,
        });
        self.set_status(if song {
            "Song initialized from pattern mode"
        } else {
            "Loop initialized from pattern mode"
        });
    }

    /// The question of Initialize Song from Pattern Mode, the original's alert 500128003 (the
    /// exe's `SALR` text, group 128 item 3; `sub_475D5C` at 0x42C5B7): its text, with the
    /// buttons Initialize (the default: Enter) and Cancel (Escape). Initialize sends the
    /// command (the original sends msg 28 only for the answer 33, 0x42C5CC).
    pub fn init_song_ui(&mut self, ctx: &Context) {
        if !self.confirm_init {
            return;
        }
        let key = |k: Key| ctx.input_mut(|i| i.consume_key(Modifiers::NONE, k));
        let mut answer = None;
        if key(Key::Enter) {
            answer = Some(true);
        } else if key(Key::Escape) {
            answer = Some(false);
        }
        let frame = Frame::new()
            .fill(SLATE_DARK)
            .stroke(Stroke::new(1.0, SLATE_LINE))
            .inner_margin(Margin::same(12))
            .corner_radius(3.0);
        Modal::new(Id::unique("orb_init_song"))
            .frame(frame)
            .backdrop_color(Color32::from_black_alpha(170))
            .show(ctx, |ui| {
                ui.set_width(320.0);
                ui.label(
                    RichText::new("Initialize Song from Pattern Mode")
                        .color(INK)
                        .strong()
                        .size(15.0),
                );
                ui.label(RichText::new(INIT_SONG_TEXT).color(INK_DIM));
                ui.add_space(8.0);
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui.button("Cancel").clicked() {
                        answer = Some(false);
                    }
                    if ui.button("Initialize").clicked() {
                        answer = Some(true);
                    }
                });
            });
        if let Some(yes) = answer {
            self.confirm_init = false;
            if yes {
                self.init_from_pattern(true);
            }
        }
    }

    /// Whether the question of Initialize Song from Pattern Mode is open.
    pub fn confirming_init(&self) -> bool {
        self.confirm_init
    }

    /// Where the Edit menu drew its items last, by text, and whether each was enabled (for
    /// tests).
    pub fn edit_item(&self, text: &str) -> Option<(egui::Rect, bool)> {
        self.edit_items
            .iter()
            .find(|(t, ..)| *t == text)
            .map(|&(_, r, on)| (r, on))
    }

    /// Selects pattern `p` of row `r` as a click on its button does (`Panel::press`): the bank
    /// shows it, the row takes the focus, the selection is sent with the panel's events.
    pub(crate) fn select_pattern(&mut self, r: usize, p: u8) {
        self.panel.rows[r].bank = p / 8;
        let button = format!("pattern{}.pattern{}", r + 1, p % 8 + 1);
        self.panel.press(&button, 0.0);
    }

    /// What a browser chose, by its mode (not an open: `open_song`).
    pub fn chosen(&mut self, mode: Mode, path: PathBuf) {
        match mode {
            Mode::OpenSong => self.open_song(&path),
            Mode::SaveSong => self.save_to(&path),
            Mode::Export => self.export_to(&path),
            Mode::ChooseMod => self.use_kit(Kit::Mod(path)),
        }
    }

    /// Notes the edits the panel made since the last frame: an edit marks the song modified. The
    /// transport's play, stop, locate and record are not edits.
    pub fn note_edits(&mut self) {
        let events = &self.panel.events[self.events_seen.min(self.panel.events.len())..];
        let edited = events.iter().any(|e| {
            use crate::model::Event;
            match e {
                Event::Press { .. } => false,
                Event::Set { id, .. } => !matches!(
                    id.as_str(),
                    "transport.position" | "transport.bar" | "transport.record"
                ),
                _ => true,
            }
        });
        // any other edit of a pattern forgets what Undo would put back
        let pattern_edited = events.iter().any(|e| {
            use crate::model::Event;
            match e {
                Event::Set { id, .. } => {
                    id.starts_with("pattern")
                        && (id.ends_with(".shuffle") || id.ends_with(".steps"))
                }
                Event::Press { .. } | Event::Pattern { .. } => false,
                _ => true,
            }
        });
        if pattern_edited {
            self.undo = None;
        }
        // after the user selects a pattern, Undo of a Duplicate leaves the selection alone
        if events
            .iter()
            .any(|e| matches!(e, crate::model::Event::Pattern { .. }))
            && let Some(u) = &mut self.undo
        {
            u.select = None;
        }
        self.doc.modified |= edited;
        // with a player the events are drained at the sync that follows
        self.events_seen = if self.link.is_some() {
            0
        } else {
            self.panel.events.len()
        };
    }

    // ---- the unsaved-changes prompt -----------------------------------------------------

    /// Does what the prompt guarded.
    fn proceed(&mut self, then: Then) {
        match then {
            Then::Open(p) => self.open_now(&p),
            Then::Demo(p) => self.open_demo_now(&p),
            Then::New => self.new_now(),
            Then::Quit => self.allow_close = true,
        }
    }

    /// The prompt's answer: Save saves (through Save As for a song with no file) and then does
    /// the guarded action; Don't Save does it at once; Cancel drops it.
    pub fn answer(&mut self, answer: Answer) {
        let Some(then) = self.prompt.take() else {
            return;
        };
        match answer {
            Answer::Cancel => {}
            Answer::DontSave => self.proceed(then),
            Answer::Save => {
                self.after_save = Some(then);
                self.save();
            }
        }
    }

    /// Whether the unsaved-changes prompt is open.
    pub fn prompting(&self) -> bool {
        self.prompt.is_some()
    }

    /// Whether a question is open (the unsaved-changes prompt or Initialize Song's): the panel
    /// then takes no keys and no clicks of its own.
    pub(crate) fn asking(&self) -> bool {
        self.prompt.is_some() || self.confirm_init
    }

    /// Draws the prompt and takes its keys: S or Enter saves, D or N does not, Esc cancels.
    pub fn prompt_ui(&mut self, ctx: &Context) {
        let Some(then) = self.prompt.clone() else {
            self.prompt_focused = false;
            return;
        };
        let sr = self.screen_reader_mode();
        let key = |k: Key| ctx.input_mut(|i| i.consume_key(Modifiers::NONE, k));
        let mut answer = None;
        if key(Key::S) || key(Key::Y) || key(Key::Enter) {
            answer = Some(Answer::Save);
        } else if key(Key::D) || key(Key::N) {
            answer = Some(Answer::DontSave);
        } else if key(Key::Escape) {
            answer = Some(Answer::Cancel);
        }
        let what = match then {
            Then::Open(_) | Then::Demo(_) => "before opening another song",
            Then::New => "before starting a new song",
            Then::Quit => "before closing",
        };
        let frame = Frame::new()
            .fill(SLATE_DARK)
            .stroke(Stroke::new(1.0, SLATE_LINE))
            .inner_margin(Margin::same(12))
            .corner_radius(3.0);
        let modal = Modal::new(Id::unique("orb_unsaved"))
            .frame(frame)
            .backdrop_color(Color32::from_black_alpha(170))
            .show(ctx, |ui| {
                ui.set_width(320.0);
                let question = format!("Save changes to {}?", self.doc.name());
                let detail = format!("The song has unsaved changes {what}.");
                // a named Dialog, the question as its name and the detail as its description
                // (screen reader mode; task-193)
                a11y::window::dialog(ui, sr, Role::Dialog, &question, Some(&detail));
                ui.label(RichText::new(&question).color(INK).strong().size(15.0));
                ui.label(RichText::new(&detail).color(INK_DIM));
                ui.add_space(8.0);
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui.button("Cancel").clicked() {
                        answer = Some(Answer::Cancel);
                    }
                    if ui.button("Don't Save").clicked() {
                        answer = Some(Answer::DontSave);
                    }
                    let save = ui.button("Save");
                    // once, on the first frame: Save, the default (Enter saves too), takes the
                    // focus from whatever had it, so that the screen reader reads the prompt
                    if sr && !self.prompt_focused {
                        save.request_focus();
                        self.prompt_focused = true;
                    }
                    if save.clicked() {
                        answer = Some(Answer::Save);
                    }
                });
            });
        self.prompt_rect = Some(modal.response.rect);
        if let Some(a) = answer {
            self.answer(a);
        }
    }

    /// The unsaved-changes prompt's rectangle as last drawn (for tests); `None` when it is not
    /// open.
    pub fn prompt_rect(&self) -> Option<egui::Rect> {
        self.prompt.as_ref().and(self.prompt_rect)
    }

    /// The window's close request: with unsaved changes it asks first.
    pub fn close_check(&mut self, ctx: &Context) {
        if ctx.input(|i| i.viewport().close_requested()) && !self.request_close() {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
        }
    }

    /// A request to close the window: true when it may close now; with unsaved changes it
    /// opens the prompt instead and is false until the answer allows it.
    pub fn request_close(&mut self) -> bool {
        if self.allow_close || !self.doc.modified {
            return true;
        }
        if self.prompt.is_none() {
            self.prompt = Some(Then::Quit);
        }
        false
    }

    /// Whether an answer of the prompt has allowed the window to close.
    pub fn close_allowed(&self) -> bool {
        self.allow_close
    }

    /// Closes the window after an answer that allowed it.
    pub fn close_if_allowed(&mut self, ctx: &Context) {
        if self.allow_close {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }

    /// The window's title: the song's name, marked when modified.
    pub fn update_title(&mut self, ctx: &Context) {
        let title = format!(
            "OpenRB-338 - {}{}",
            self.doc.name(),
            if self.doc.modified { " *" } else { "" }
        );
        if title != self.title {
            ctx.send_viewport_cmd(egui::ViewportCommand::Title(title.clone()));
            self.title = title;
        }
    }

    // ---- drawing ------------------------------------------------------------------------

    /// Draws the menu bar and runs what it chooses.
    pub fn menu_bar(&mut self, ui: &mut Ui) {
        let mut cmd: Option<Cmd> = None;
        // the Edit menu's places are this frame's only: none while it is closed
        self.edit_items.clear();
        let enabled =
            !self.browsing() && !self.prompting() && self.replace.is_none() && !self.error_open();
        // F10, or Alt pressed and released alone, focuses the File menu (task-184)
        let key = menu_bar_key(ui) && enabled;
        // screen reader mode (task-212): a second F10 or lone Alt, with the focus on a title or
        // a menu open, leaves the bar: the menu closes and the focus goes back to the control
        // that had it
        let sr = self.screen_reader_mode();
        let leave = key && sr && self.bar_active;
        let focus_file = key && !leave;
        if leave {
            egui::Popup::close_all(ui.ctx());
            if let Some(back) = self.menu_return {
                ui.memory_mut(|m| m.request_focus(back));
            }
            self.bar_active = false;
        }
        let mut bar: Vec<(Id, Id)> = vec![];
        if enabled {
            open_on_down(ui);
        }
        egui::Panel::top("orb_menu")
            .exact_size(MENU_H)
            .show(ui, |ui| {
                ui.add_enabled_ui(enabled, |ui| {
                    egui::MenuBar::new().ui(ui, |ui| {
                        self.menu_items(ui, &mut cmd, &mut bar, focus_file);
                    });
                });
            });
        let on_title = {
            let f = ui.memory(|m| m.focused());
            bar.iter().any(|(b, _)| Some(*b) == f)
        };
        ui.data_mut(|d| d.insert_temp(bar_id(), bar));
        match cmd {
            // the system clipboard's text comes back as a Paste event, as for Ctrl/Cmd+V. A
            // plugin's window (upstream's, removed in this fork, task-231) did not answer
            // `RequestPaste` (it read the clipboard only on a paste key), so there the
            // item reads the system clipboard directly (task-274, task-287)
            Some(Cmd::PastePattern) if !self.is_plugin() => {
                ui.ctx()
                    .send_viewport_cmd(egui::ViewportCommand::RequestPaste);
            }
            Some(c) => {
                self.command(c);
                // the panel shows the change in the next frame, also with no audio stream
                ui.ctx().request_repaint();
            }
            None => {}
        }
        if sr {
            if leave {
                // this frame's focus is still the title: the request above acts at the next
                self.bar_active = false;
            } else {
                let active = on_title || egui::Popup::is_any_open(ui.ctx());
                self.menu_bar_focus(ui, active);
            }
        } else {
            self.bar_active = false;
        }
    }

    /// The menus of the bar: File, Edit, View, Mods, Options and Help; what is chosen goes to `cmd`
    /// (Edit > Paste Pattern as [`Cmd::PastePattern`], which `menu_bar` turns into a paste of
    /// the system clipboard in the standalone app). A plugin's File menu has New, Open and Info
    /// only (task-201); its Edit menu is the standalone's without shortcut texts (task-274).
    /// Each menu's button goes into `bar` (the keyboard walk of the bar, task-184);
    /// `focus_file` focuses the File button.
    fn menu_items(
        &mut self,
        ui: &mut Ui,
        cmd: &mut Option<Cmd>,
        bar: &mut Vec<(Id, Id)>,
        focus_file: bool,
    ) {
        let plugin = self.is_plugin();
        let has_routing = self.has_routing();
        // a plugin's editor has no keyboard shortcuts (task-201): no shortcut texts
        let key = |s| (!plugin).then_some(s);
        let file = ui.menu_button("File", |ui| {
            if item(ui, "New", key(&shortcuts::NEW)) {
                *cmd = Some(Cmd::New);
            }
            if item(ui, "Open...", key(&shortcuts::OPEN)) {
                *cmd = Some(Cmd::Open);
            }
            if !plugin && item(ui, "Save", key(&shortcuts::SAVE)) {
                *cmd = Some(Cmd::Save);
            }
            if !plugin && item(ui, "Save As...", None) {
                *cmd = Some(Cmd::SaveAs);
            }
            // Ctrl/Cmd+I (task-310); disabled without a song, which a plugin always has
            if item_if(
                ui,
                plugin || self.link.is_some(),
                "Info...",
                key(&shortcuts::INFO),
            )
            .clicked()
            {
                *cmd = Some(Cmd::Info);
            }
            if plugin {
                return;
            }
            ui.separator();
            let ok = self.panel.transport.song_mode && !self.exporting();
            let r = ui.add_enabled(ok, egui::Button::new("Export Loop as Audio File..."));
            let r = if self.panel.transport.song_mode {
                r
            } else {
                r.on_disabled_hover_text("Export needs song mode.")
            };
            if r.clicked() {
                *cmd = Some(Cmd::Export);
                ui.close();
            }
        });
        if focus_file {
            file.response.request_focus();
        }
        note_button(bar, &file.response);
        let edit = ui.menu_button("Edit", |ui| {
            let song = self.link.is_some();
            // "Undo Paste Pattern": the item names what it undoes (in a plugin's editor too,
            // with no shortcut text: it takes no keys; task-274)
            let what = self.undo_name().filter(|_| song);
            let text = what.map_or("Undo".into(), |w| format!("Undo {w}"));
            let r = item_if(ui, what.is_some(), &text, key(&shortcuts::UNDO));
            self.edit_items.push(("Undo", r.rect, what.is_some()));
            if r.clicked() {
                *cmd = Some(Cmd::Undo);
            }
            ui.separator();
            // the pattern commands, in a plugin's editor too (the user's review of task-274,
            // round 3), there with no shortcut texts
            for (text, sc, c) in EDIT_ITEMS {
                let r = item_if(ui, song, text, sc.and_then(key));
                self.edit_items.push((text, r.rect, song));
                if r.clicked() {
                    *cmd = Some(c);
                }
            }
            ui.separator();
            // the original's commands 306 and 307, enabled in song mode only (task-306); not in a
            // plugin's editor, which has no song mode (the user's decision, 2026-11-25)
            let on = self.can_init();
            for (text, c) in INIT_ITEMS.into_iter().filter(|_| !plugin) {
                let r = item_if(ui, on, text, None);
                self.edit_items.push((text, r.rect, on));
                let r = if on || !song {
                    r
                } else {
                    r.on_disabled_hover_text("Needs song mode.")
                };
                if r.clicked() {
                    *cmd = Some(c);
                }
            }
            if !plugin {
                ui.separator();
            }
            // the pattern editors; Ctrl/Cmd+E opens the focused row's, so its text is on that
            // item (the 303-1's or the 303-2's with that 303 focused, the 808's or the 909's
            // with theirs; tasks 280, 281 and 290)
            let focus = self.panel.focus;
            let sc = key(&shortcuts::PATTERN_EDITOR);
            for k in 0..2 {
                let text = crate::pattern_editor::menu_item(k);
                let r = item_if(ui, song, text, sc.filter(|_| focus == k));
                self.edit_items.push((text, r.rect, song));
                if r.clicked() {
                    *cmd = Some(Cmd::PatternEditor(k));
                }
            }
            for m in crate::drum_editor::Machine::ALL {
                let text = m.menu_item();
                let r = item_if(ui, song, text, sc.filter(|_| focus == m.row()));
                self.edit_items.push((text, r.rect, song));
                if r.clicked() {
                    *cmd = Some(Cmd::DrumEditor(m));
                }
            }
        });
        note_button(bar, &edit.response);
        if !plugin {
            // the main window's look (task-267): the standalone app's only
            let view = ui.menu_button("View", |ui| {
                // two radio items directly in View, no submenu (the user's review, 2026-11-21)
                for (text, t) in [("Classic", Theme::Classic), ("Modern", Theme::Modern)] {
                    if ui.radio(self.settings.theme == t, text).clicked() {
                        *cmd = Some(Cmd::SetTheme(t));
                        ui.close();
                    }
                }
            });
            note_button(bar, &view.response);
        }
        let mods = ui.menu_button("Mods", |ui| {
            // task-195: only in screen reader mode
            if self.screen_reader_mode() {
                if item(ui, "Choose Mod...", Some(&shortcuts::CHOOSE_MOD)) {
                    *cmd = Some(Cmd::ChooseMod);
                }
                ui.separator();
            }
            let (std_on, checked) = match &self.mod_list {
                Some(Ok(l)) => self.mod_checks(l),
                _ => self.mod_checks(&[]),
            };
            if checked_item(ui, std_on, STANDARD).clicked() {
                *cmd = Some(Cmd::UseKit(Kit::Standard));
                ui.close();
            }
            match &self.mod_list {
                None => {}
                Some(Ok(list)) => {
                    egui::ScrollArea::vertical()
                        .max_height(300.0)
                        .show(ui, |ui| {
                            for (i, e) in list.iter().enumerate() {
                                let on = checked == Some(i);
                                if !e.variants.is_empty() {
                                    // mods that share a name: a submenu of files
                                    let mut b = egui::containers::menu::SubMenuButton::new(&e.name);
                                    b.button = b.button.selected(on);
                                    let active = match &self.doc.kit {
                                        Kit::Mod(p) => Some(p),
                                        Kit::Standard => None,
                                    };
                                    let pref = self.preferred_variant(e);
                                    b.ui(ui, |ui| {
                                        for v in &e.variants {
                                            let mut t = v.label();
                                            if pref.is_some_and(|p| p == v) {
                                                t.push_str("  (preferred)");
                                            }
                                            let me = active == Some(&v.path);
                                            if checked_item(ui, me, t).clicked() {
                                                *cmd = Some(Cmd::ChooseVariant(v.path.clone()));
                                                ui.close();
                                            }
                                        }
                                    });
                                    continue;
                                }
                                if checked_item(ui, on, &e.name).clicked() {
                                    *cmd = Some(Cmd::UseKit(Kit::Mod(e.path.clone())));
                                    ui.close();
                                }
                            }
                        });
                }
                Some(Err(e)) => {
                    ui.label(RichText::new(e).color(INK_DIM));
                }
            }
            ui.separator();
            if item(ui, "Choose file...", None) {
                *cmd = Some(Cmd::ChooseModFile);
            }
            if item(ui, "Refresh List", None) {
                *cmd = Some(Cmd::RefreshMods);
            }
            ui.separator();
            let has_demo = self.has_demo();
            let r = ui.add_enabled(has_demo, egui::Button::new("Demo Song"));
            if r.clicked() {
                *cmd = Some(Cmd::DemoSong);
                ui.close();
            }
            if item(ui, "Info...", None) {
                *cmd = Some(Cmd::ModInfo);
            }
        });
        note_button(bar, &mods.response);
        let options = ui.menu_button("Options", |ui| {
            let mut v = self.panel.vintage;
            if ui.checkbox(&mut v, "Vintage ReBirth Synth Sound").clicked() {
                *cmd = Some(Cmd::ToggleVintage);
                ui.close();
            }
            let mut st = self.settings.startup_sound;
            if ui.checkbox(&mut st, "Play Mod Startup Sound").clicked() {
                *cmd = Some(Cmd::ToggleStartupSound);
                ui.close();
            }
            // the keyboard modes: none in a plugin's editor (task-201); the plugin's mode is the
            // panel's selector only (task-231), not a menu item. Nor is Audio Output: a
            // plugin's audio is the host's (task-191). The plugin has its Output Routing
            // instead (task-300)
            if plugin {
                if has_routing {
                    ui.separator();
                    let text = format!("{}...", crate::routing_window::TITLE);
                    if item(ui, &text, None) {
                        *cmd = Some(Cmd::OutputRouting);
                    }
                }
                return;
            }
            if item(ui, "Audio Output...", None) {
                *cmd = Some(Cmd::AudioOutput);
            }
            ui.menu_button("Screen Reader Mode", |ui| {
                for v in ScreenReaderSetting::ALL {
                    let on = self.settings.screen_reader == v;
                    if ui.radio(on, v.label()).clicked() {
                        *cmd = Some(Cmd::SetScreenReader(v));
                        ui.close();
                    }
                }
            });
            // task-192: only in screen reader mode
            let mut raw = self.settings.speak_raw_values;
            if self.screen_reader_mode() && ui.checkbox(&mut raw, "Speak Raw Values").clicked() {
                *cmd = Some(Cmd::ToggleSpeakRawValues);
                ui.close();
            }
            // task-190: only in screen reader mode
            let mut follows = self.settings.step_editor_follows;
            if self.screen_reader_mode()
                && ui
                    .checkbox(&mut follows, "Step Editor Follows Playback")
                    .clicked()
            {
                *cmd = Some(Cmd::ToggleStepEditorFollows);
                ui.close();
            }
            // task-211: only in screen reader mode
            let mut makes = self.settings.pitch_keys_make_notes;
            if self.screen_reader_mode()
                && ui.checkbox(&mut makes, "Pitch Keys Make Notes").clicked()
            {
                *cmd = Some(Cmd::TogglePitchKeysMakeNotes);
                ui.close();
            }
            ui.separator();
            // the browser's "Use as ... folder" buttons, for the system dialog
            if self.screen_reader_mode() {
                for f in [Folder::Songs, Folder::Mods] {
                    let label = match f {
                        Folder::Songs => "Set Songs Folder...",
                        Folder::Mods => "Set Mods Folder...",
                    };
                    if item(ui, label, None) {
                        *cmd = Some(Cmd::SetFolder(f));
                    }
                }
                ui.separator();
            }
            let ctx = ui.ctx().clone();
            let synth = format!(
                "Program Synth ({})",
                shortcuts::text(&ctx, &shortcuts::KEYS_SYNTH)
            );
            let patterns = format!(
                "Select Patterns ({})",
                shortcuts::text(&ctx, &shortcuts::KEYS_PATTERNS)
            );
            if ui.radio(self.key_mode == KeyMode::Synth, synth).clicked() {
                *cmd = Some(Cmd::SetKeyMode(KeyMode::Synth));
                ui.close();
            }
            if ui
                .radio(self.key_mode == KeyMode::Patterns, patterns)
                .clicked()
            {
                *cmd = Some(Cmd::SetKeyMode(KeyMode::Patterns));
                ui.close();
            }
        });
        note_button(bar, &options.response);
        // Last in the bar. An item is one line of `HELP_ITEMS`.
        let help = ui.menu_button("Help", |ui| {
            for (text, c) in HELP_ITEMS {
                // no Keyboard Shortcuts in a plugin's editor (task-201)
                if plugin && *c == Cmd::KeyHelp {
                    continue;
                }
                if item(ui, text, None) {
                    *cmd = Some(c.clone());
                }
            }
        });
        note_button(bar, &help.response);
    }
}

/// Where the bar's buttons and their menus' popups were last frame: (button id, popup id).
fn bar_id() -> Id {
    Id::unique("orb_menu_bar_buttons")
}

fn note_button(bar: &mut Vec<(Id, Id)>, button: &egui::Response) {
    bar.push((button.id, egui::Popup::menu(button).get_id()));
}

/// True when the keyboard focus is on a button of the menu bar or a menu is open: the panel
/// then takes no keys (Up and Down are the menu's, not the row focus).
pub fn menu_has_keys(ctx: &Context) -> bool {
    let focused = ctx.memory(|m| m.focused());
    let bar: Vec<(Id, Id)> = ctx.data(|d| d.get_temp(bar_id())).unwrap_or_default();
    egui::Popup::is_any_open(ctx) || bar.iter().any(|(b, _)| Some(*b) == focused)
}

/// Down on a focused menu button of the bar opens its menu in this frame; egui's own menu
/// keys then move the focus to the first item (`MenuState::handle_keyboard`: the focus is on
/// the anchor and the direction is Down). egui itself opens a menu from the keyboard only with
/// Enter or Space and leaves the item focus to a second Down.
fn open_on_down(ui: &Ui) {
    let ctx = ui.ctx();
    let pending_id = Id::unique("orb_menu_down_pending");
    // the frame after the opening: the popup was shown once, so the Down reaches its items
    let pending: bool = ctx.data(|d| d.get_temp(pending_id)).unwrap_or(false);
    if pending {
        ctx.data_mut(|d| d.insert_temp(pending_id, false));
        if egui::Popup::is_any_open(ctx) {
            ctx.memory_mut(|m| m.move_focus(egui::FocusDirection::Down));
        }
        return;
    }
    let focused = ctx.memory(|m| m.focused());
    let down = ui.input(|i| i.key_pressed(Key::ArrowDown));
    if !down || egui::Popup::is_any_open(ctx) {
        return;
    }
    let bar: Vec<(Id, Id)> = ctx.data(|d| d.get_temp(bar_id())).unwrap_or_default();
    if let Some((_, popup)) = bar.iter().find(|(b, _)| Some(*b) == focused) {
        egui::Popup::open_id(ctx, *popup);
        // this Down is the opening, not a move to the panel's widget below the bar
        ctx.memory_mut(|m| m.move_focus(egui::FocusDirection::None));
        ctx.data_mut(|d| d.insert_temp(pending_id, true));
        ctx.request_repaint();
    }
}

/// True on the frame the user asks for the menu bar from the keyboard (task-184): F10, or Alt
/// pressed and released alone (the Windows convention; no other key or mouse press between).
/// F10 is consumed; the Alt state lives in egui's temporary data.
fn menu_bar_key(ui: &Ui) -> bool {
    let f10 = ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, Key::F10));
    let id = Id::unique("orb_menu_alt_alone");
    let (alt, dirty) = ui.input(|i| {
        let dirty = i.events.iter().any(|e| {
            matches!(e, egui::Event::Key { pressed: true, .. })
                || matches!(e, egui::Event::PointerButton { pressed: true, .. })
        });
        (i.modifiers.alt, dirty)
    });
    // None: Alt is up; Some(true): down and alone so far; Some(false): used by a key or click
    let before: Option<bool> = ui.data(|d| d.get_temp(id));
    let now = match (before, alt) {
        (None, true) => Some(!dirty),
        (Some(c), true) => Some(c && !dirty),
        _ => None,
    };
    let alone_released = !alt && before == Some(true);
    ui.data_mut(|d| {
        match now {
            Some(v) => {
                d.insert_temp(id, v);
            }
            None => d.remove::<bool>(id),
        };
    });
    f10 || alone_released
}

/// The pattern items of the Edit menu, in order, with their shortcuts (Clear Pattern has none,
/// task-290).
const EDIT_ITEMS: [(&str, Option<&egui::KeyboardShortcut>, Cmd); 5] = [
    ("Cut Pattern", Some(&shortcuts::CUT), Cmd::CutPattern),
    ("Copy Pattern", Some(&shortcuts::COPY), Cmd::CopyPattern),
    ("Paste Pattern", Some(&shortcuts::PASTE), Cmd::PastePattern),
    (
        "Duplicate Pattern",
        Some(&shortcuts::DUPLICATE),
        Cmd::DuplicatePattern,
    ),
    ("Clear Pattern", None, Cmd::ClearPattern),
];

/// The Edit menu's Initialize items, in the original's order (`SMNU` 128: ids 306 and 307).
const INIT_ITEMS: [(&str, Cmd); 2] = [
    ("Initialize Loop from Pattern Mode", Cmd::InitLoop),
    ("Initialize Song from Pattern Mode", Cmd::InitSong),
];

/// The original's question before Initialize Song from Pattern Mode (alert 500128003).
pub const INIT_SONG_TEXT: &str =
    "This will overwrite your entire Song - all knob and button automation will be cleared.";

/// The items of the Help menu, in order.
const HELP_ITEMS: &[(&str, Cmd)] = &[
    ("Keyboard Shortcuts...", Cmd::KeyHelp),
    ("About OpenRB-338...", Cmd::About),
];

/// A Mods menu entry that marks the playing kit (task-201). It looks and acts as
/// `selectable_label`; in the accessibility tree it is a checked menu item (egui already sets
/// `Toggled` from the selection), which a screen reader reads as "menu item, checked".
fn checked_item(ui: &mut Ui, on: bool, text: impl Into<egui::WidgetText>) -> egui::Response {
    let r = ui.selectable_label(on, text);
    ui.ctx().accesskit_node_builder(r.id, |node| {
        node.set_role(Role::MenuItemCheckBox);
    });
    r
}

/// A menu item with its shortcut; true when clicked (the menu closes).
fn item(ui: &mut Ui, text: &str, shortcut: Option<&egui::KeyboardShortcut>) -> bool {
    item_if(ui, true, text, shortcut).clicked()
}

/// [`item`], enabled or not; its response.
fn item_if(
    ui: &mut Ui,
    enabled: bool,
    text: &str,
    shortcut: Option<&egui::KeyboardShortcut>,
) -> egui::Response {
    let mut b = egui::Button::new(text);
    if let Some(sc) = shortcut {
        b = b.shortcut_text(shortcuts::text(ui.ctx(), sc));
    }
    let r = ui.add_enabled(enabled, b);
    if r.clicked() {
        ui.close();
    }
    r
}

/// The order of the Mods menu's top-level entries (task-167): the names with ASCII letters
/// folded to lowercase (no Unicode folding, no locale); names equal folded go lowercase first at
/// the first position where they differ ("abc", "Abc", "ABC"). The caller breaks the rest by the
/// file path.
pub fn mod_name_order(a: &str, b: &str) -> std::cmp::Ordering {
    let fold = |s: &str| {
        s.bytes()
            .map(|c| c.to_ascii_lowercase())
            .collect::<Vec<u8>>()
    };
    // lowercase letters are the larger bytes, so the tie-break compares the other way round
    fold(a)
        .cmp(&fold(b))
        .then_with(|| b.as_bytes().cmp(a.as_bytes()))
}

#[cfg(test)]
mod tests {
    use super::mod_name_order;

    fn sorted(names: &[&str]) -> Vec<String> {
        let mut v: Vec<String> = names.iter().map(|s| s.to_string()).collect();
        v.sort_by(|a, b| mod_name_order(a, b));
        v
    }

    #[test]
    fn fold_first_then_lowercase_first() {
        assert_eq!(sorted(&["ac", "Ab"]), ["Ab", "ac"]);
        assert_eq!(sorted(&["ABC", "Abc", "abc"]), ["abc", "Abc", "ABC"]);
        // the first difference in case decides, not the count of lowercase letters
        assert_eq!(sorted(&["aBc", "Abc", "abC"]), ["abC", "aBc", "Abc"]);
    }

    #[test]
    fn non_letters_keep_their_byte_order() {
        // digits and '[' (0x5B) and '_' (0x5F) are below the folded letters
        assert_eq!(
            sorted(&["beta", "_x", "Zed", "1a", "[b"]),
            ["1a", "[b", "_x", "beta", "Zed"]
        );
        // no Unicode folding: a non-ASCII letter is above every ASCII one
        assert_eq!(sorted(&["\u{e9}a", "za"]), ["za", "\u{e9}a"]);
    }
}
