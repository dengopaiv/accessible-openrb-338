//! The UI's small settings file: the songs folder, the mods folder and the recent folders of
//! the file browser (task-046), and the startup sound preference (task-074).
//!
//! It is a TOML file (task-107) with the keys `songs_dir` and `mods_dir` (paths, absent when
//! unset), `recent` (an array of paths, newest first), `last_song_dir` and `last_mod_dir`
//! (paths, absent until a file is chosen: the folder of the last song chosen in Open, Save As
//! or Export, and of the last mod; task-144), `startup_sound` (a bool) and the table
//! `preferred_mods` (an INFO name to the file name preferred among mods that share it,
//! task-151) and `screen_reader` (`"auto"`, `"on"` or `"off"`; absent or unknown reads as
//! `auto`; task-185) and `speak_raw_values` (a bool, false by default: in screen reader mode a
//! knob or fader speaks its 0..127 value after its unit; task-192) and `step_editor_follows` (a
//! bool, false by default: in screen reader mode a 303's step editor follows the playing
//! pattern; task-190) and `pitch_keys_make_notes` (a bool, false by default: in screen reader
//! mode a 303's pitch key also turns a rest into a note; task-211) and `export_format` (`wav`,
//! `flac` or `ogg`: the system Export dialog offers it first; task-204), so it can be edited by
//! hand. Where it lives: `$ORB_UI_CONFIG` if set; else, in portable mode (a `portable.txt` next
//! to the program, [`crate::portable`], task-223), `data/ui.toml` beside the program; else
//! `OpenRB-338/ui.toml` in the platform's
//! configuration folder (`$XDG_CONFIG_HOME`, or `~/.config` on Linux, `~/Library/Application
//! Support` on macOS, `%APPDATA%` on Windows). In portable mode the folders (`songs_dir`,
//! `mods_dir`, `recent`, `last_song_dir`, `last_mod_dir`) that lie inside the program's folder
//! are written relative to it and read back inside it ([`Settings::portable_root`]), so a copy
//! on a USB stick finds them under any drive letter. Reading never fails: a missing or damaged
//! file gives the defaults (a damaged file stays as it is until the next save), a key of the
//! wrong type is skipped and the others kept, and unknown keys are
//! ignored. Saving reads the file again and applies only what this instance changed since it
//! last read or wrote it, so another instance's changes survive (task-229; saves in the same
//! instant can still race, there is no lock). A change is recorded by the setters
//! (`set_songs_dir`, `set_mods_dir`, `set_last_song_dir`, `set_last_mod_dir`,
//! `set_startup_sound`, `set_preferred_mod`), which win over the file even when the value
//! equals what this instance last saw; a field assigned directly counts as changed when it
//! differs from what was last read or written. The list `recent` is changed only through
//! `note_recent` (a folder is pushed onto the file's list; a direct assignment is not
//! merged unless nothing was noted). When the file is missing or damaged at save time, the
//! whole in-memory copy is written, so an unrelated save never resets this instance's values
//! to the defaults; any other read error (not "not found") makes the save fail and leaves the
//! file and these settings alone. The result is written through a unique temporary file in
//! the same folder and a rename, these settings are refreshed from it, and the errors are
//! returned for the browser to show. `base`, `noted` and `dirty` are public only so a struct
//! literal with `..Settings::default()` compiles outside the crate (its `base` is the defaults). A
//! path that is not valid UTF-8 cannot be stored in TOML (and a lossy string would name another
//! folder), so a save leaves it out: a `recent` entry is dropped, `songs_dir`, `mods_dir`,
//! `last_song_dir` and `last_mod_dir` are omitted and fall back to the default. An empty
//! `$ORB_UI_CONFIG` counts as unset. The old `ui.conf` is neither read nor deleted. The fork's
//! keys (`screen_reader`, `speak_raw_values`, `step_editor_follows`, `pitch_keys_make_notes`,
//! `export_format`) merge as a field assigned directly does; in portable mode the file's folders
//! are read inside the portable folder before the merge, so a relative and a full path to the
//! same folder are one value.

use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use serde::Serialize;

use crate::browser::ExportFormat;
use crate::screen_reader::ScreenReaderSetting;

/// How many recent folders are kept.
pub const MAX_RECENT: usize = 8;

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
    /// Screen reader mode (task-185): `auto` (default), `on` or `off`. Stored as the string
    /// `screen_reader`; an absent or unknown value reads as `auto`.
    pub screen_reader: ScreenReaderSetting,
    /// "Speak raw values" (task-192): in screen reader mode a knob or fader speaks its 0..127
    /// value after its unit ("1.24 kilohertz, 64"). Stored as the bool `speak_raw_values`.
    pub speak_raw_values: bool,
    /// "Step Editor Follows Playback" (task-190, task-191): in screen reader mode a 303's step
    /// editor and a drum grid move to the pattern their row plays, silently, at the next input;
    /// off (the default), they stay on the pattern last chosen by the user. Stored as the bool
    /// `step_editor_follows`.
    pub step_editor_follows: bool,
    /// "Pitch Keys Make Notes" (task-211): in screen reader mode a 303's pitch key on a rest
    /// also makes it a note; off (the default), it sets only the note, as the original. Stored
    /// as the bool `pitch_keys_make_notes`.
    pub pitch_keys_make_notes: bool,
    /// The format of the last export through the system dialog (task-204): that dialog offers
    /// it first and gives it to a name typed without extension. Stored as the string
    /// `export_format` (`wav`, `flac` or `ogg`); the in-app browser does not use it.
    pub export_format: ExportFormat,
    /// Where this was read from and is written to (`None`: never persisted, as in tests).
    pub file: Option<PathBuf>,
    /// The portable folder (task-223): a folder inside it is written relative to it, and a
    /// relative folder of the file is read inside it. `None` (not portable): full paths, as
    /// before.
    pub portable_root: Option<PathBuf>,
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
    screen_reader: ScreenReaderSetting,
    speak_raw_values: bool,
    step_editor_follows: bool,
    pitch_keys_make_notes: bool,
    export_format: ExportFormat,
}

/// Equal when the public fields are (the save bookkeeping is not compared).
impl PartialEq for Settings {
    fn eq(&self, o: &Self) -> bool {
        self.values() == o.values() && self.file == o.file && self.portable_root == o.portable_root
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
            screen_reader: ScreenReaderSetting::Auto,
            speak_raw_values: false,
            step_editor_follows: false,
            pitch_keys_make_notes: false,
            export_format: ExportFormat::Wav,
            file: None,
            portable_root: None,
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
    file_for(
        std::env::var_os("ORB_UI_CONFIG").as_deref(),
        crate::portable::data_dir().as_deref(),
    )
}

/// [`default_file`] for the value of `ORB_UI_CONFIG` (`config`; empty counts as unset) and the
/// portable data folder (`portable_data`, task-223), in that order of precedence, else the
/// platform's configuration folder.
pub fn file_for(config: Option<&std::ffi::OsStr>, portable_data: Option<&Path>) -> Option<PathBuf> {
    if let Some(p) = config.filter(|p| !p.is_empty()) {
        return Some(PathBuf::from(p));
    }
    if let Some(data) = portable_data {
        return Some(data.join("ui.toml"));
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
    base.map(|b| b.join("OpenRB-338").join("ui.toml"))
}

/// What the file holds, in the file's order.
#[derive(Serialize)]
struct Stored<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    songs_dir: Option<Cow<'a, Path>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    mods_dir: Option<Cow<'a, Path>>,
    startup_sound: bool,
    screen_reader: &'a str,
    speak_raw_values: bool,
    step_editor_follows: bool,
    pitch_keys_make_notes: bool,
    export_format: &'a str,
    recent: Vec<Cow<'a, Path>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    last_song_dir: Option<Cow<'a, Path>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    last_mod_dir: Option<Cow<'a, Path>>,
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
        match default_file() {
            Some(f) => Self::load_from_in(&f, crate::portable::root()),
            None => Self::default(),
        }
    }

    /// Reads `file`; a missing or unreadable file gives the defaults. Later writes go to
    /// `file`. Not portable: the folders are full paths.
    pub fn load_from(file: &Path) -> Self {
        Self::load_from_in(file, None)
    }

    /// [`Settings::load_from`] with the portable folder `root` (task-223): a relative folder of
    /// the file is read inside `root`, and later writes store the folders inside `root`
    /// relative to it.
    pub fn load_from_in(file: &Path, root: Option<&Path>) -> Self {
        let mut s = Self::read_in(file, root).ok().flatten().unwrap_or_default();
        s.file = Some(file.to_path_buf());
        s.portable_root = root.map(Path::to_path_buf);
        s.base = Box::new(s.values());
        s
    }

    /// [`Settings::read`] with the folders read inside the portable folder `root`, when there
    /// is one (task-223).
    fn read_in(file: &Path, root: Option<&Path>) -> Result<Option<Self>, std::io::Error> {
        let Some(mut s) = Self::read(file)? else {
            return Ok(None);
        };
        if let Some(b) = root {
            let full = |p: &mut PathBuf| *p = crate::portable::resolve(b, p);
            for p in [
                &mut s.songs_dir,
                &mut s.mods_dir,
                &mut s.last_song_dir,
                &mut s.last_mod_dir,
            ]
            .into_iter()
            .flatten()
            {
                full(p);
            }
            s.recent.iter_mut().for_each(full);
            // "songs" and a full path to the same folder (in any case, on Windows) are one
            // recent folder
            let mut seen: Vec<PathBuf> = Vec::new();
            s.recent.retain(|p| {
                let new = !seen.iter().any(|q| crate::portable::same(q, p));
                seen.push(p.clone());
                new
            });
        }
        Ok(Some(s))
    }

    /// The values of `file` as stored: `None` when it is missing or damaged, an error for any
    /// other read error.
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
        if let Some(v) = table
            .get("screen_reader")
            .and_then(|v| v.as_str())
            .and_then(ScreenReaderSetting::from_key)
        {
            s.screen_reader = v;
        }
        if let Some(b) = table.get("speak_raw_values").and_then(|v| v.as_bool()) {
            s.speak_raw_values = b;
        }
        if let Some(b) = table.get("step_editor_follows").and_then(|v| v.as_bool()) {
            s.step_editor_follows = b;
        }
        if let Some(b) = table.get("pitch_keys_make_notes").and_then(|v| v.as_bool()) {
            s.pitch_keys_make_notes = b;
        }
        if let Some(f) =
            ExportFormat::from_extension(table.get("export_format").and_then(|v| v.as_str()))
        {
            s.export_format = f;
        }
        s.songs_dir = table.get("songs_dir").and_then(path_of);
        s.mods_dir = table.get("mods_dir").and_then(path_of);
        s.last_song_dir = table.get("last_song_dir").and_then(path_of);
        s.last_mod_dir = table.get("last_mod_dir").and_then(path_of);
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
            screen_reader: self.screen_reader,
            speak_raw_values: self.speak_raw_values,
            step_editor_follows: self.step_editor_follows,
            pitch_keys_make_notes: self.pitch_keys_make_notes,
            export_format: self.export_format,
        }
    }

    /// The file's current values with this instance's changes since it last read or wrote
    /// applied: each key set through its setter or differing from the base overwrites that
    /// key, the noted folders are pushed onto the file's `recent`, a preferred mod that was
    /// set, changed or cleared sets, changes or clears only its entry. A missing or damaged
    /// file gives this instance's whole copy; another read error is returned.
    fn merged(&self, file: &Path) -> Result<Values, std::io::Error> {
        let v = self.values();
        let Some(disk) = Self::read_in(file, self.portable_root.as_deref())? else {
            return Ok(v);
        };
        let mut m = disk.values();
        let b = &*self.base;
        let set = |k: &str| self.dirty.contains(k);
        if set("startup_sound") || v.startup_sound != b.startup_sound {
            m.startup_sound = v.startup_sound;
        }
        // the fork's keys: changed when they differ from what was last read or written
        if v.screen_reader != b.screen_reader {
            m.screen_reader = v.screen_reader;
        }
        if v.speak_raw_values != b.speak_raw_values {
            m.speak_raw_values = v.speak_raw_values;
        }
        if v.step_editor_follows != b.step_editor_follows {
            m.step_editor_follows = v.step_editor_follows;
        }
        if v.pitch_keys_make_notes != b.pitch_keys_make_notes {
            m.pitch_keys_make_notes = v.pitch_keys_make_notes;
        }
        if v.export_format != b.export_format {
            m.export_format = v.export_format;
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
        if self.noted.is_empty() && v.recent != b.recent {
            // The list was assigned as a whole.
            m.recent = v.recent;
        }
        for dir in &self.noted {
            // the same folder in another case (Windows) is one entry, as read_in has it
            m.recent.retain(|d| !crate::portable::same(d, dir));
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
    /// take the merged values.
    pub fn save(&mut self) -> Result<(), String> {
        let Some(file) = self.file.clone() else {
            return Ok(());
        };
        let file = &file;
        let m = self
            .merged(file)
            .map_err(|e| format!("{}: cannot read: {e}", file.display()))?;
        let text = toml::to_string(&Stored {
            songs_dir: m.songs_dir.as_deref().and_then(|p| self.stored(p)),
            mods_dir: m.mods_dir.as_deref().and_then(|p| self.stored(p)),
            startup_sound: m.startup_sound,
            screen_reader: m.screen_reader.key(),
            speak_raw_values: m.speak_raw_values,
            step_editor_follows: m.step_editor_follows,
            pitch_keys_make_notes: m.pitch_keys_make_notes,
            export_format: m.export_format.extension(),
            recent: m.recent.iter().filter_map(|p| self.stored(p)).collect(),
            last_song_dir: m.last_song_dir.as_deref().and_then(|p| self.stored(p)),
            last_mod_dir: m.last_mod_dir.as_deref().and_then(|p| self.stored(p)),
            preferred_mods: m
                .preferred_mods
                .iter()
                .map(|(n, f)| (n.as_str(), f.as_str()))
                .collect(),
        })
        .map_err(|e| format!("{}: cannot encode: {e}", file.display()))?;
        if let Some(dir) = file.parent() {
            std::fs::create_dir_all(dir)
                .map_err(|e| format!("{}: cannot create: {e}", dir.display()))?;
        }
        // Written next to the file and renamed over it, so a crash never leaves half a file.
        // The name is unique per process and call, so two instances saving at once never
        // share a temporary file.
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let mut tmp = file.as_os_str().to_owned();
        tmp.push(format!(
            ".{}.{}.tmp",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        let tmp = PathBuf::from(tmp);
        std::fs::write(&tmp, text)
            .and_then(|()| std::fs::rename(&tmp, file))
            .map_err(|e| {
                let _ = std::fs::remove_file(&tmp);
                format!("{}: cannot write: {e}", file.display())
            })?;
        self.startup_sound = m.startup_sound;
        self.songs_dir = m.songs_dir.clone();
        self.mods_dir = m.mods_dir.clone();
        self.recent = m.recent.clone();
        self.last_song_dir = m.last_song_dir.clone();
        self.last_mod_dir = m.last_mod_dir.clone();
        self.preferred_mods = m.preferred_mods.clone();
        self.screen_reader = m.screen_reader;
        self.speak_raw_values = m.speak_raw_values;
        self.step_editor_follows = m.step_editor_follows;
        self.pitch_keys_make_notes = m.pitch_keys_make_notes;
        self.export_format = m.export_format;
        *self.base = m;
        self.noted.clear();
        self.dirty.clear();
        Ok(())
    }

    /// A folder as the file stores it: left out when TOML cannot hold it ([`storable`]),
    /// relative to [`Settings::portable_root`] when it lies inside it (task-223), else the full
    /// path.
    fn stored<'a>(&self, p: &'a Path) -> Option<Cow<'a, Path>> {
        let p = storable(p)?;
        match self
            .portable_root
            .as_deref()
            .and_then(|b| crate::portable::relative(b, p))
        {
            Some(r) => Some(Cow::Owned(r)),
            None => Some(Cow::Borrowed(p)),
        }
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
