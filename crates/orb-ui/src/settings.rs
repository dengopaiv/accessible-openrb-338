//! The UI's small settings file: the songs folder, the mods folder and the recent folders of
//! the file browser (task-046), and the startup sound preference (task-074).
//!
//! It is a TOML file (task-107) with the keys `songs_dir` and `mods_dir` (paths, absent when
//! unset), `recent` (an array of paths, newest first), `last_song_dir` and `last_mod_dir`
//! (paths, absent until a file is chosen: the folder of the last song chosen in Open, Save As
//! or Export, and of the last mod; task-144), `startup_sound` (a bool), `theme` (`"modern"`,
//! the default, or `"classic"`: the main window's skin, View > Classic / Modern, task-267; the old
//! `theme_viewer_filter` of the removed Theme Viewer is ignored and dropped at the next save),
//! `pattern_editor_303_1_pos` and `pattern_editor_303_2_pos` (two integers each, the 303-1 and
//! 303-2 pattern editor windows' left top in points, absent until the window is moved; the
//! standalone app's only, tasks 274 and 290),
//! `pattern_editor_808_pos` (the same of the 808 pattern editor, task-280),
//! `pattern_editor_909_pos` (the same of the 909 pattern editor, task-281),
//! the keys of the audio output (`audio_driver`, `audio_device`, `audio_device_name`,
//! `audio_sample_rate`, `audio_buffer_size`: [`AudioChoice`], Options > Audio Output, task-191)
//! and the table
//! `preferred_mods` (an INFO name to the file name preferred among mods that share it,
//! task-151), so it can be edited by hand. Where it lives (task-289):
//! 1. `$ORB_UI_CONFIG` if set;
//! 2. on Windows, `OpenRB-338.toml` in the program's folder while the marker file
//!    `OpenRB-338.portable` is there (portable mode, `portable.rs`; its content is ignored): when
//!    the local file is present it alone is used, read and written, and the per-user file is never
//!    touched; when it is absent, the per-user settings are read and written into it, and it is
//!    used from then on. A local file that does not parse is an error ([`Settings::load_error`]). A
//!    folder that cannot be written gives one notice ([`Settings::notice`]) and no per-user write.
//!    Without the marker a local file is ignored; the per-user file is used;
//! 3. `OpenRB-338/OpenRB-338.toml` in the platform's configuration folder (`$XDG_CONFIG_HOME`, or
//!    `~/.config` on Linux, `~/Library/Application Support` on macOS, `%APPDATA%` on Windows). When
//!    it is absent and `ui.toml` (its name before task-289) is in that folder, `ui.toml` is copied
//!    to it once; `ui.toml` is never changed or deleted, so an older version still finds its
//!    settings.
//!
//! Reading never fails: a missing or damaged file gives the defaults (a damaged file stays as
//! it is until the next save), a key of the wrong type is skipped and the others kept, and
//! unknown keys are
//! ignored. Saving reads the file again and applies only what this instance changed since it
//! last read or wrote it, so another instance's changes survive (task-229; saves in the same
//! instant can still race, there is no lock). A change is recorded by the setters
//! (`set_songs_dir`, `set_mods_dir`, `set_last_song_dir`, `set_last_mod_dir`,
//! `set_startup_sound`, `set_theme`, `set_pattern_editor_303_1_pos`,
//! `set_pattern_editor_303_2_pos`, `set_pattern_editor_808_pos`,
//! `set_pattern_editor_909_pos`, `set_preferred_mod`), which win over
//! the file even when the value equals what this instance last saw; a field assigned directly
//! counts as changed when it differs from what was last read or written. The list `recent` is
//! changed only through `note_recent` (a folder is pushed onto the file's list; a direct assignment
//! is not merged unless nothing was noted). When the file is missing or damaged at save time, the
//! whole in-memory copy is written, so an unrelated save never resets this instance's values
//! to the defaults; any other read error (not "not found") makes the save fail and leaves the
//! file and these settings alone. The result is written through a unique temporary file in
//! the same folder and a rename, these settings are refreshed from it, and the errors are
//! returned for the browser to show. `base`, `noted` and `dirty` are public only so a struct
//! literal with `..Settings::default()` compiles outside the crate (its `base` is the defaults). A
//! path that is not valid UTF-8 cannot be stored in TOML (and a lossy string would name another
//! folder), so a save leaves it out: a `recent` entry is dropped, `songs_dir`, `mods_dir`,
//! `last_song_dir` and `last_mod_dir` are omitted and fall back to the default. An empty
//! `$ORB_UI_CONFIG` counts as unset. The old `ui.conf` is neither read nor deleted.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use serde::Serialize;

use crate::portable::FILE_NAME;

/// How many recent folders are kept.
pub const MAX_RECENT: usize = 8;

/// The audio output the user chose in Options > Audio Output (task-191): the standalone app's
/// only. The default is the platform's first driver, the system default device, the device's
/// default rate and 2048 frames. Stored as the keys `audio_driver`, `audio_device` (cpal's text
/// id, e.g. `CoreAudio:...`), `audio_device_name` (what the status line names a missing device
/// by), `audio_sample_rate` (Hz) and `audio_buffer_size` (frames); the three optional keys are
/// absent for the default. A value of the wrong type or out of range is skipped.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AudioChoice {
    /// The driver's name; `None`: the platform's default.
    pub driver: Option<String>,
    /// The device's id; `None`: System default.
    pub device: Option<String>,
    /// The device's name when it was chosen.
    pub device_name: Option<String>,
    /// The rate asked for; `None`: Device default.
    pub rate: Option<u32>,
    /// The buffer size in frames.
    pub buffer: u32,
}

impl Default for AudioChoice {
    fn default() -> Self {
        AudioChoice {
            driver: None,
            device: None,
            device_name: None,
            rate: None,
            buffer: orb_player::output::BUFFER_FRAMES,
        }
    }
}

impl AudioChoice {
    /// The request that opens this choice.
    pub fn request(&self) -> orb_player::devices::Request {
        orb_player::devices::Request {
            driver: self.driver.clone(),
            device: self
                .device
                .as_ref()
                .map(|id| orb_player::devices::DeviceRef {
                    id: id.clone(),
                    name: self.device_name.clone().unwrap_or_else(|| id.clone()),
                }),
            rate: self.rate,
            buffer: self.buffer,
        }
    }

    /// The choice that a request stands for.
    pub fn from_request(r: &orb_player::devices::Request) -> Self {
        AudioChoice {
            driver: r.driver.clone(),
            device: r.device.as_ref().map(|d| d.id.clone()),
            device_name: r.device.as_ref().map(|d| d.name.clone()),
            rate: r.rate,
            buffer: r.buffer,
        }
    }
}

/// The buffer sizes and rates a file may hold: anything else is skipped.
const AUDIO_BUFFERS: std::ops::RangeInclusive<i64> = 16..=65536;
const AUDIO_RATES: std::ops::RangeInclusive<i64> = 8000..=768_000;

/// The look of the main window's panel (View > Classic / Modern, task-267): the standalone app's
/// only.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Theme {
    /// The SVG panel (`skin::svg::modern`).
    #[default]
    Modern,
    /// The original's panel from the kit's bitmaps (`skin::classic`).
    Classic,
}

impl Theme {
    /// The value stored in the file.
    pub fn name(self) -> &'static str {
        match self {
            Theme::Modern => "modern",
            Theme::Classic => "classic",
        }
    }

    pub fn from_name(s: &str) -> Option<Self> {
        match s {
            "modern" => Some(Theme::Modern),
            "classic" => Some(Theme::Classic),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Settings {
    /// The original's preference "Play Mod Startup Sound" (on by default: the preferences'
    /// constructor `sub_41CAA4` sets it, 0x41CB13): a mod change plays the mod's startup
    /// sound. Stored as the bool `startup_sound`.
    pub startup_sound: bool,
    /// Where songs are opened from and saved to by default (`None`: the documents folder).
    pub songs_dir: Option<PathBuf>,
    /// Where mods are looked for by default (`None`: the documents folder).
    pub mods_dir: Option<PathBuf>,
    /// The folders files were last chosen in, newest first.
    pub recent: Vec<PathBuf>,
    /// The folder of the last song chosen in Open, Save As or Export (task-144): where those
    /// dialogs start while it exists.
    pub last_song_dir: Option<PathBuf>,
    /// The folder of the last mod chosen in the mod dialog: where that dialog starts while it
    /// exists.
    pub last_mod_dir: Option<PathBuf>,
    /// The mod file preferred for an INFO name that several mods share (task-151): INFO name
    /// to file name. Stored as the table `preferred_mods`. Names are compared ignoring ASCII
    /// case ([`Settings::preferred_mod`]).
    pub preferred_mods: BTreeMap<String, String>,
    /// The main window's skin (View > Classic / Modern, task-267). Stored as the string `theme`,
    /// `"modern"` (the default) or `"classic"`; another value is skipped.
    pub theme: Theme,
    /// The 303-1 pattern editor window's left top in points (tasks 274 and 290), `None` until
    /// it is moved. Stored as the array `pattern_editor_303_1_pos`; the plugin neither reads
    /// nor writes it.
    pub pattern_editor_303_1_pos: Option<[i32; 2]>,
    /// The same of the 303-2 pattern editor's window (task-290), `pattern_editor_303_2_pos`.
    pub pattern_editor_303_2_pos: Option<[i32; 2]>,
    /// The same of the 808 pattern editor's window (task-280), `pattern_editor_808_pos`.
    pub pattern_editor_808_pos: Option<[i32; 2]>,
    /// The same of the 909 pattern editor's window (task-281), `pattern_editor_909_pos`.
    pub pattern_editor_909_pos: Option<[i32; 2]>,
    /// The audio output (Options > Audio Output, task-191).
    pub audio: AudioChoice,
    /// Where this was read from and is written to (`None`: never persisted, as in tests).
    pub file: Option<PathBuf>,
    /// Whether `file` is the portable `OpenRB-338.toml` next to the program (task-289): a
    /// failed write is then one notice, not an error, and never goes to the per-user file.
    #[doc(hidden)]
    pub portable: bool,
    /// Whether the notice of a failed portable write was given already.
    #[doc(hidden)]
    pub warned: bool,
    /// A message for the status line, taken by `App::show`: the portable folder cannot be
    /// written.
    #[doc(hidden)]
    pub notice: Option<String>,
    /// A message for the error box, taken by `App::show`: the portable file does not parse.
    #[doc(hidden)]
    pub load_error: Option<String>,
    /// The values as last read from or written to the file: a save changes only what differs
    /// from them (task-229).
    /// Public only so `..Settings::default()` works outside the crate; not for use.
    #[doc(hidden)]
    pub base: Box<Values>,
    /// Folders noted since the last read or write, oldest first: a save pushes them onto the
    /// list found in the file.
    #[doc(hidden)]
    pub noted: Vec<PathBuf>,
    /// What the setters recorded since the last read or write: the key names, and
    /// `pref:<lowercase INFO name>` for a preferred mod.
    #[doc(hidden)]
    pub dirty: BTreeSet<String>,
}

/// The stored values of [`Settings`].
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[doc(hidden)]
pub struct Values {
    startup_sound: bool,
    songs_dir: Option<PathBuf>,
    mods_dir: Option<PathBuf>,
    recent: Vec<PathBuf>,
    last_song_dir: Option<PathBuf>,
    last_mod_dir: Option<PathBuf>,
    preferred_mods: BTreeMap<String, String>,
    theme: Theme,
    pattern_editor_303_1_pos: Option<[i32; 2]>,
    pattern_editor_303_2_pos: Option<[i32; 2]>,
    pattern_editor_808_pos: Option<[i32; 2]>,
    pattern_editor_909_pos: Option<[i32; 2]>,
    audio: AudioChoice,
}

/// Equal when the public fields are (the save bookkeeping is not compared).
impl PartialEq for Settings {
    fn eq(&self, o: &Self) -> bool {
        self.values() == o.values() && self.file == o.file
    }
}

impl Eq for Settings {}

impl Default for Settings {
    fn default() -> Self {
        Self {
            startup_sound: true,
            songs_dir: None,
            mods_dir: None,
            recent: Vec::new(),
            last_song_dir: None,
            last_mod_dir: None,
            preferred_mods: BTreeMap::new(),
            theme: Theme::Modern,
            pattern_editor_303_1_pos: None,
            pattern_editor_303_2_pos: None,
            pattern_editor_808_pos: None,
            pattern_editor_909_pos: None,
            audio: AudioChoice::default(),
            file: None,
            portable: false,
            warned: false,
            notice: None,
            load_error: None,
            base: Box::new(Values {
                startup_sound: true,
                ..Values::default()
            }),
            noted: Vec::new(),
            dirty: BTreeSet::new(),
        }
    }
}

fn home() -> Option<PathBuf> {
    let var = if cfg!(windows) { "USERPROFILE" } else { "HOME" };
    std::env::var_os(var)
        .map(PathBuf::from)
        .filter(|p| p.is_dir())
}

/// The user's documents folder: `~/Documents` if it exists, else the home folder, else the
/// current folder.
pub fn documents_dir() -> PathBuf {
    let home = home();
    home.as_ref()
        .map(|h| h.join("Documents"))
        .filter(|d| d.is_dir())
        .or(home)
        .unwrap_or_else(|| PathBuf::from("."))
}

/// The user's home folder, when it is known.
pub fn home_dir() -> Option<PathBuf> {
    home()
}

/// The settings file's default place (see the module text).
pub fn default_file() -> Option<PathBuf> {
    if let Some(p) = std::env::var_os("ORB_UI_CONFIG").filter(|p| !p.is_empty()) {
        return Some(PathBuf::from(p));
    }
    let base = if cfg!(windows) {
        std::env::var_os("APPDATA").map(PathBuf::from)
    } else if let Some(x) = std::env::var_os("XDG_CONFIG_HOME").filter(|x| !x.is_empty()) {
        Some(PathBuf::from(x))
    } else if cfg!(target_os = "macos") {
        home().map(|h| h.join("Library/Application Support"))
    } else {
        home().map(|h| h.join(".config"))
    };
    base.map(|b| b.join("OpenRB-338").join(FILE_NAME))
}

/// The name the settings file had before task-289, in the same folder.
pub const LEGACY_FILE_NAME: &str = "ui.toml";

/// What the file holds, in the file's order.
#[derive(Serialize)]
struct Stored<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    songs_dir: Option<&'a Path>,
    #[serde(skip_serializing_if = "Option::is_none")]
    mods_dir: Option<&'a Path>,
    startup_sound: bool,
    recent: Vec<&'a Path>,
    #[serde(skip_serializing_if = "Option::is_none")]
    last_song_dir: Option<&'a Path>,
    #[serde(skip_serializing_if = "Option::is_none")]
    last_mod_dir: Option<&'a Path>,
    theme: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pattern_editor_303_1_pos: Option<[i32; 2]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pattern_editor_303_2_pos: Option<[i32; 2]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pattern_editor_808_pos: Option<[i32; 2]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pattern_editor_909_pos: Option<[i32; 2]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    audio_driver: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    audio_device: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    audio_device_name: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    audio_sample_rate: Option<u32>,
    audio_buffer_size: u32,
    // a table, so last
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    preferred_mods: BTreeMap<&'a str, &'a str>,
}

/// The path when TOML can hold it (valid UTF-8). A lossy string would name another folder, so
/// any other path is left out of the file.
fn storable(p: &Path) -> Option<&Path> {
    p.to_str().map(|_| p)
}

/// A path value of the table: a non-empty string.
fn path_of(v: &toml::Value) -> Option<PathBuf> {
    v.as_str().filter(|s| !s.is_empty()).map(PathBuf::from)
}

impl Settings {
    /// Reads the settings from the default file (the defaults when there is none).
    pub fn load() -> Self {
        if std::env::var_os("ORB_UI_CONFIG").is_some_and(|p| !p.is_empty()) {
            return default_file()
                .map(|f| Self::load_from(&f))
                .unwrap_or_default();
        }
        Self::load_resolved(crate::portable::candidate().as_deref(), default_file())
    }

    /// The settings of the portable file `OpenRB-338.toml` in `local` when it is there, else of
    /// `user`, the per-user file (the rules are in the module text). Public so the tests can
    /// run on any system.
    pub fn load_resolved(local: Option<&Path>, user: Option<PathBuf>) -> Self {
        let load_user = |user: Option<PathBuf>| match user {
            Some(f) => Self::load_user(&f),
            None => Self::default(),
        };
        let Some(dir) = local else {
            return load_user(user);
        };
        // no marker: the per-user settings, whatever a local OpenRB-338.toml holds
        if !dir.join(crate::portable::MARKER_NAME).is_file() {
            return load_user(user);
        }
        let file = dir.join(FILE_NAME);
        let problem = match std::fs::read_to_string(&file) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                // the first start: the per-user settings move in, read without writing any
                // per-user file (not even the copy of ui.toml)
                let mut s = match user {
                    Some(f) => Self::read_user(&f),
                    None => Self::default(),
                };
                s.file = Some(file);
                s.portable = true;
                // everything counts as changed, so the whole copy is written
                s.base = Self::default().base;
                if let Err(e) = s.save() {
                    s.notice = Some(e);
                }
                return s;
            }
            Ok(text) => match text.parse::<toml::Table>() {
                Ok(_) => {
                    let mut s = Self::load_from(&file);
                    s.portable = true;
                    return s;
                }
                Err(e) => format!("is damaged: {}", e.message().trim_end()),
            },
            Err(e) => format!("cannot be read: {e}"),
        };
        // Portable mode never falls back to the per-user file: defaults, and the file stays as
        // it is until a save.
        Self {
            load_error: Some(format!("The settings file {} {problem}", file.display())),
            file: Some(file),
            portable: true,
            ..Self::default()
        }
    }

    /// Reads the per-user file `user`; when it is absent and `ui.toml` (its old name) is in the
    /// same folder, its values are written to `user` first (a save: atomic) and `ui.toml`
    /// stays as it was. Without the write (an error) the old file's values are still used.
    fn load_user(user: &Path) -> Self {
        let mut s = Self::read_user(user);
        if s.file.as_deref() != Some(user) {
            // from ui.toml: written to the new file at once (when that fails they are saved
            // later)
            s.file = Some(user.to_path_buf());
            s.base = Self::default().base;
            let _ = s.save();
        }
        s
    }

    /// The values of the per-user file `user`, else of `ui.toml` beside it (then `file` is
    /// that one's path), else the defaults. Writes nothing.
    fn read_user(user: &Path) -> Self {
        let legacy = user
            .parent()
            .map(|d| d.join(LEGACY_FILE_NAME))
            .filter(|l| l.is_file());
        match legacy {
            Some(legacy) if !user.exists() => Self::load_from(&legacy),
            _ => Self::load_from(user),
        }
    }

    /// Reads `file`; a missing or unreadable file gives the defaults. Later writes go to
    /// `file`.
    pub fn load_from(file: &Path) -> Self {
        let mut s = Self::read(file).ok().flatten().unwrap_or_default();
        s.file = Some(file.to_path_buf());
        s.base = Box::new(s.values());
        s
    }

    /// The values of `file`: `None` when it is missing or damaged, an error for any other
    /// read error.
    fn read(file: &Path) -> Result<Option<Self>, std::io::Error> {
        let mut s = Self::default();
        let text = match std::fs::read_to_string(file) {
            Ok(t) => t,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            // Not UTF-8: damaged.
            Err(e) if e.kind() == std::io::ErrorKind::InvalidData => return Ok(None),
            Err(e) => return Err(e),
        };
        let Ok(table) = text.parse::<toml::Table>() else {
            return Ok(None);
        };
        if let Some(b) = table.get("startup_sound").and_then(|v| v.as_bool()) {
            s.startup_sound = b;
        }
        s.songs_dir = table.get("songs_dir").and_then(path_of);
        s.mods_dir = table.get("mods_dir").and_then(path_of);
        s.last_song_dir = table.get("last_song_dir").and_then(path_of);
        s.last_mod_dir = table.get("last_mod_dir").and_then(path_of);
        let pos = |key: &str| {
            table
                .get(key)
                .and_then(|v| v.as_array())
                .and_then(|a| match a.as_slice() {
                    [x, y] => Some([x.as_integer()?, y.as_integer()?]),
                    _ => None,
                })
                .and_then(|[x, y]| Some([i32::try_from(x).ok()?, i32::try_from(y).ok()?]))
        };
        s.pattern_editor_303_1_pos = pos("pattern_editor_303_1_pos");
        s.pattern_editor_303_2_pos = pos("pattern_editor_303_2_pos");
        s.pattern_editor_808_pos = pos("pattern_editor_808_pos");
        s.pattern_editor_909_pos = pos("pattern_editor_909_pos");
        let text = |key: &str| {
            table
                .get(key)
                .and_then(|v| v.as_str())
                .filter(|v| !v.is_empty())
                .map(str::to_string)
        };
        let number = |key: &str, range: &std::ops::RangeInclusive<i64>| {
            table
                .get(key)
                .and_then(|v| v.as_integer())
                .filter(|n| range.contains(n))
                .and_then(|n| u32::try_from(n).ok())
        };
        s.audio = AudioChoice {
            driver: text("audio_driver"),
            device: text("audio_device"),
            device_name: text("audio_device_name"),
            rate: number("audio_sample_rate", &AUDIO_RATES),
            buffer: number("audio_buffer_size", &AUDIO_BUFFERS)
                .unwrap_or(orb_player::output::BUFFER_FRAMES),
        };
        if let Some(t) = table
            .get("theme")
            .and_then(|v| v.as_str())
            .and_then(Theme::from_name)
        {
            s.theme = t;
        }
        if let Some(list) = table.get("recent").and_then(|v| v.as_array()) {
            for p in list.iter().filter_map(path_of) {
                if s.recent.len() < MAX_RECENT && !s.recent.contains(&p) {
                    s.recent.push(p);
                }
            }
        }
        if let Some(t) = table.get("preferred_mods").and_then(|v| v.as_table()) {
            for (name, file) in t {
                if let Some(f) = file.as_str().filter(|f| !f.is_empty()) {
                    s.preferred_mods.insert(name.clone(), f.to_string());
                }
            }
        }
        Ok(Some(s))
    }

    fn values(&self) -> Values {
        Values {
            startup_sound: self.startup_sound,
            songs_dir: self.songs_dir.clone(),
            mods_dir: self.mods_dir.clone(),
            recent: self.recent.clone(),
            last_song_dir: self.last_song_dir.clone(),
            last_mod_dir: self.last_mod_dir.clone(),
            preferred_mods: self.preferred_mods.clone(),
            theme: self.theme,
            pattern_editor_303_1_pos: self.pattern_editor_303_1_pos,
            pattern_editor_303_2_pos: self.pattern_editor_303_2_pos,
            pattern_editor_808_pos: self.pattern_editor_808_pos,
            pattern_editor_909_pos: self.pattern_editor_909_pos,
            audio: self.audio.clone(),
        }
    }

    /// The file's current values with this instance's changes since it last read or wrote
    /// applied: each key set through its setter or differing from the base overwrites that
    /// key, the noted folders are pushed onto the file's `recent`, a preferred mod that was
    /// set, changed or cleared sets, changes or clears only its entry. A missing or damaged
    /// file gives this instance's whole copy; another read error is returned.
    fn merged(&self, file: &Path) -> Result<Values, std::io::Error> {
        let v = self.values();
        let Some(disk) = Self::read(file)? else {
            return Ok(v);
        };
        let mut m = disk.values();
        let b = &*self.base;
        let set = |k: &str| self.dirty.contains(k);
        if set("startup_sound") || v.startup_sound != b.startup_sound {
            m.startup_sound = v.startup_sound;
        }
        if set("songs_dir") || v.songs_dir != b.songs_dir {
            m.songs_dir = v.songs_dir;
        }
        if set("mods_dir") || v.mods_dir != b.mods_dir {
            m.mods_dir = v.mods_dir;
        }
        if set("last_song_dir") || v.last_song_dir != b.last_song_dir {
            m.last_song_dir = v.last_song_dir;
        }
        if set("last_mod_dir") || v.last_mod_dir != b.last_mod_dir {
            m.last_mod_dir = v.last_mod_dir;
        }
        if set("theme") || v.theme != b.theme {
            m.theme = v.theme;
        }
        if set("pattern_editor_303_1_pos")
            || v.pattern_editor_303_1_pos != b.pattern_editor_303_1_pos
        {
            m.pattern_editor_303_1_pos = v.pattern_editor_303_1_pos;
        }
        if set("pattern_editor_303_2_pos")
            || v.pattern_editor_303_2_pos != b.pattern_editor_303_2_pos
        {
            m.pattern_editor_303_2_pos = v.pattern_editor_303_2_pos;
        }
        if set("pattern_editor_808_pos") || v.pattern_editor_808_pos != b.pattern_editor_808_pos {
            m.pattern_editor_808_pos = v.pattern_editor_808_pos;
        }
        if set("pattern_editor_909_pos") || v.pattern_editor_909_pos != b.pattern_editor_909_pos {
            m.pattern_editor_909_pos = v.pattern_editor_909_pos;
        }
        if set("audio") || v.audio != b.audio {
            m.audio = v.audio;
        }
        if self.noted.is_empty() && v.recent != b.recent {
            // The list was assigned as a whole.
            m.recent = v.recent;
        }
        for dir in &self.noted {
            m.recent.retain(|d| d != dir);
            m.recent.insert(0, dir.clone());
            m.recent.truncate(MAX_RECENT);
        }
        for name in b.preferred_mods.keys() {
            if !v.preferred_mods.contains_key(name) {
                m.preferred_mods
                    .retain(|n, _| !n.eq_ignore_ascii_case(name));
            }
        }
        for (name, f) in &v.preferred_mods {
            if b.preferred_mods.get(name) != Some(f) {
                m.preferred_mods
                    .retain(|n, _| !n.eq_ignore_ascii_case(name));
                m.preferred_mods.insert(name.clone(), f.clone());
            }
        }
        for key in &self.dirty {
            let Some(low) = key.strip_prefix("pref:") else {
                continue;
            };
            m.preferred_mods.retain(|n, _| !n.eq_ignore_ascii_case(low));
            if let Some((n, f)) = v
                .preferred_mods
                .iter()
                .find(|(n, _)| n.eq_ignore_ascii_case(low))
            {
                m.preferred_mods.insert(n.clone(), f.clone());
            }
        }
        Ok(m)
    }

    /// Sets `songs_dir`; the next save writes it whatever the file holds.
    pub fn set_songs_dir(&mut self, dir: Option<PathBuf>) {
        self.songs_dir = dir;
        self.dirty.insert("songs_dir".into());
    }

    /// Sets `mods_dir`; the next save writes it whatever the file holds.
    pub fn set_mods_dir(&mut self, dir: Option<PathBuf>) {
        self.mods_dir = dir;
        self.dirty.insert("mods_dir".into());
    }

    /// Sets `last_song_dir`; the next save writes it whatever the file holds.
    pub fn set_last_song_dir(&mut self, dir: Option<PathBuf>) {
        self.last_song_dir = dir;
        self.dirty.insert("last_song_dir".into());
    }

    /// Sets `last_mod_dir`; the next save writes it whatever the file holds.
    pub fn set_last_mod_dir(&mut self, dir: Option<PathBuf>) {
        self.last_mod_dir = dir;
        self.dirty.insert("last_mod_dir".into());
    }

    /// Sets `startup_sound`; the next save writes it whatever the file holds.
    pub fn set_startup_sound(&mut self, on: bool) {
        self.startup_sound = on;
        self.dirty.insert("startup_sound".into());
    }

    /// Sets the main window's skin (`theme`); the next save writes it whatever the file holds.
    pub fn set_theme(&mut self, theme: Theme) {
        self.theme = theme;
        self.dirty.insert("theme".into());
    }

    /// Sets the audio output choice (`audio_*`, task-191); the next save writes it whatever the
    /// file holds.
    pub fn set_audio(&mut self, audio: AudioChoice) {
        self.audio = audio;
        self.dirty.insert("audio".into());
    }

    /// Sets the 303-1 pattern editor window's place (`pattern_editor_303_1_pos`, task-290); the
    /// next save writes it whatever the file holds.
    pub fn set_pattern_editor_303_1_pos(&mut self, pos: Option<[i32; 2]>) {
        self.pattern_editor_303_1_pos = pos;
        self.dirty.insert("pattern_editor_303_1_pos".into());
    }

    /// Sets the 303-2 pattern editor window's place (`pattern_editor_303_2_pos`, task-290); the
    /// next save writes it whatever the file holds.
    pub fn set_pattern_editor_303_2_pos(&mut self, pos: Option<[i32; 2]>) {
        self.pattern_editor_303_2_pos = pos;
        self.dirty.insert("pattern_editor_303_2_pos".into());
    }

    /// Sets the 808 pattern editor window's place (`pattern_editor_808_pos`, task-280); the
    /// next save writes it whatever the file holds.
    pub fn set_pattern_editor_808_pos(&mut self, pos: Option<[i32; 2]>) {
        self.pattern_editor_808_pos = pos;
        self.dirty.insert("pattern_editor_808_pos".into());
    }

    /// Sets the 909 pattern editor window's place (`pattern_editor_909_pos`, task-281); the
    /// next save writes it whatever the file holds.
    pub fn set_pattern_editor_909_pos(&mut self, pos: Option<[i32; 2]>) {
        self.pattern_editor_909_pos = pos;
        self.dirty.insert("pattern_editor_909_pos".into());
    }

    /// The file preferred for the INFO name `name` (ignoring ASCII case), if one is set.
    pub fn preferred_mod(&self, name: &str) -> Option<&str> {
        self.preferred_mods
            .iter()
            .find(|(n, _)| n.eq_ignore_ascii_case(name))
            .map(|(_, f)| f.as_str())
    }

    /// Prefers `file` for `name` (`None` clears the preference), replacing an entry of the same
    /// name in another ASCII case.
    pub fn set_preferred_mod(&mut self, name: &str, file: Option<&str>) {
        self.preferred_mods
            .retain(|n, _| !n.eq_ignore_ascii_case(name));
        if let Some(f) = file {
            self.preferred_mods.insert(name.to_string(), f.to_string());
        }
        self.dirty
            .insert(format!("pref:{}", name.to_ascii_lowercase()));
    }

    /// Writes this instance's changes to the file (a no-op without one): the file is read
    /// again first and only what changed here since the last read or write is applied to it,
    /// so another instance's changes to other keys survive (task-229). These settings then
    /// take the merged values. The file is written atomically (a temporary file, then a rename).
    /// In portable mode a failure is not an error: it
    /// sets [`Settings::notice`] once and the values stay unsaved.
    pub fn save(&mut self) -> Result<(), String> {
        let Some(file) = self.file.clone() else {
            return Ok(());
        };
        match self.save_to(&file) {
            Ok(()) => Ok(()),
            Err(reason) if self.portable => {
                if !self.warned {
                    self.warned = true;
                    let dir = file.parent().unwrap_or(Path::new("."));
                    self.notice = Some(format!(
                        "Settings cannot be saved in {}: {reason}",
                        dir.display()
                    ));
                }
                Ok(())
            }
            Err(reason) => Err(format!("{}: {reason}", file.display())),
        }
    }

    fn save_to(&mut self, file: &Path) -> Result<(), String> {
        let m = self.merged(file).map_err(|e| format!("cannot read: {e}"))?;
        let text = toml::to_string(&Stored {
            songs_dir: m.songs_dir.as_deref().and_then(storable),
            mods_dir: m.mods_dir.as_deref().and_then(storable),
            startup_sound: m.startup_sound,
            recent: m.recent.iter().filter_map(|p| storable(p)).collect(),
            last_song_dir: m.last_song_dir.as_deref().and_then(storable),
            last_mod_dir: m.last_mod_dir.as_deref().and_then(storable),
            theme: m.theme.name(),
            pattern_editor_303_1_pos: m.pattern_editor_303_1_pos,
            pattern_editor_303_2_pos: m.pattern_editor_303_2_pos,
            pattern_editor_808_pos: m.pattern_editor_808_pos,
            pattern_editor_909_pos: m.pattern_editor_909_pos,
            audio_driver: m.audio.driver.as_deref(),
            audio_device: m.audio.device.as_deref(),
            audio_device_name: m.audio.device_name.as_deref(),
            audio_sample_rate: m.audio.rate,
            audio_buffer_size: m.audio.buffer,
            preferred_mods: m
                .preferred_mods
                .iter()
                .map(|(n, f)| (n.as_str(), f.as_str()))
                .collect(),
        })
        .map_err(|e| format!("cannot encode: {e}"))?;
        if let Some(dir) = file.parent() {
            std::fs::create_dir_all(dir)
                .map_err(|e| format!("cannot create {}: {e}", dir.display()))?;
        }
        write_atomic(file, text.as_bytes()).map_err(|e| format!("cannot write: {e}"))?;
        self.startup_sound = m.startup_sound;
        self.songs_dir = m.songs_dir.clone();
        self.mods_dir = m.mods_dir.clone();
        self.recent = m.recent.clone();
        self.last_song_dir = m.last_song_dir.clone();
        self.last_mod_dir = m.last_mod_dir.clone();
        self.preferred_mods = m.preferred_mods.clone();
        self.theme = m.theme;
        self.pattern_editor_303_1_pos = m.pattern_editor_303_1_pos;
        self.pattern_editor_303_2_pos = m.pattern_editor_303_2_pos;
        self.pattern_editor_808_pos = m.pattern_editor_808_pos;
        self.pattern_editor_909_pos = m.pattern_editor_909_pos;
        self.audio = m.audio.clone();
        *self.base = m;
        self.noted.clear();
        self.dirty.clear();
        Ok(())
    }

    /// Puts `dir` first in the recent folders.
    pub fn note_recent(&mut self, dir: &Path) {
        self.recent.retain(|d| d != dir);
        self.recent.insert(0, dir.to_path_buf());
        self.recent.truncate(MAX_RECENT);
        self.noted.retain(|d| d != dir);
        self.noted.push(dir.to_path_buf());
    }
}

/// Writes `bytes` to `file` through a temporary file in the same folder and a rename, so a
/// crash or a full disk never leaves a short or empty file. The temporary name is unique per
/// process and call, so two instances saving at once never share one.
fn write_atomic(file: &Path, bytes: &[u8]) -> std::io::Result<()> {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let mut tmp = file.as_os_str().to_owned();
    tmp.push(format!(
        ".{}.{}.tmp",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    let tmp = PathBuf::from(tmp);
    // synced before the rename, so a power loss cannot leave a renamed but empty file
    let write = || -> std::io::Result<()> {
        let mut f = std::fs::File::create(&tmp)?;
        std::io::Write::write_all(&mut f, bytes)?;
        f.sync_all()
    };
    write()
        .and_then(|()| std::fs::rename(&tmp, file))
        .inspect_err(|_| {
            let _ = std::fs::remove_file(&tmp);
        })
}
