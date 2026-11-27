//! Portable mode on Windows (task-289, GitHub issue 19; the marker, task-295): while a marker
//! file `OpenRB-338.portable` (any content, even empty) is in the folder of the program, the
//! file `OpenRB-338.toml` there keeps the settings and the crash logs next to it
//! instead of in the user's profile.
//!
//! - The program's folder is the one of its `OpenRB-338.exe`: the folder of the module that holds
//!   this code, from `GetModuleHandleExW` with the address of a function here and
//!   `GetModuleFileNameW` ([`folder_of`]).
//! - [`folder`] is that folder when the marker is in it. macOS and Linux never have one.
//!
//! The fork (accessible-openrb-338) adds two things, the user's choice of 2026-11-26 when
//! upstream's portable mode met the fork's (task-223): the folders inside the portable folder
//! are stored relative to it ([`relative`], [`resolve`]), so a copy on a USB stick finds its
//! songs and mods folders under any drive letter; and the fork's older marker `portable.txt`
//! ([`LEGACY_MARKER`]) still makes a folder portable, its settings `data\ui.toml` copied into
//! `OpenRB-338.toml` once ([`legacy_settings`]).
//!
//! The rules for what the file holds are in `settings.rs`.

use std::path::{Component, Path, PathBuf};

/// The settings file's name, in the per-user folder and in the portable one.
pub const FILE_NAME: &str = "OpenRB-338.toml";

/// The marker file that turns portable mode on when it is in the program's folder (task-295).
pub const MARKER_NAME: &str = "OpenRB-338.portable";

/// The folder of the program for the path of its module `module` (the exe): the exe's folder.
pub fn folder_of(module: &Path) -> Option<PathBuf> {
    module.parent().map(Path::to_path_buf)
}

/// The folder that would hold the portable file (the program's), or `None` off Windows.
pub fn candidate() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        static CANDIDATE: std::sync::OnceLock<Option<PathBuf>> = std::sync::OnceLock::new();
        CANDIDATE
            .get_or_init(|| module_path().and_then(|m| folder_of(&m)))
            .clone()
    }
    #[cfg(not(windows))]
    {
        None
    }
}

/// The portable folder: the program's folder when the marker is in it (Windows only).
pub fn folder() -> Option<PathBuf> {
    candidate().filter(|d| is_portable(d))
}

/// The fork's marker before upstream's (task-223, replaced in the merge of upstream f83c54f):
/// a folder with `portable.txt` and no [`MARKER_NAME`] is portable too, so a copy unpacked from
/// an older fork zip stays portable. Its settings were `data\ui.toml` ([`LEGACY_DATA`]); on the
/// first start without `OpenRB-338.toml` they are copied into it (`settings.rs`). Neither old
/// file is changed or deleted.
pub const LEGACY_MARKER: &str = "portable.txt";

/// The fork's old portable data folder (task-223), beside the program: `data\ui.toml` and
/// `data\logs`. Read once for the settings, never written.
pub const LEGACY_DATA: &str = "data";

/// Whether `dir` is a portable folder: it holds [`MARKER_NAME`] or the fork's older
/// [`LEGACY_MARKER`], as a file (a folder of that name is not a marker).
pub fn is_portable(dir: &Path) -> bool {
    dir.join(MARKER_NAME).is_file() || dir.join(LEGACY_MARKER).is_file()
}

/// The fork's old portable settings file `data\ui.toml` in `dir`, when it is there.
pub fn legacy_settings(dir: &Path) -> Option<PathBuf> {
    Some(dir.join(LEGACY_DATA).join("ui.toml")).filter(|f| f.is_file())
}

// The fork's relative folders (task-223): in portable mode the settings store the folders that
// lie inside the program's folder relative to it, so a copy on a USB stick keeps its songs and
// mods folders under any drive letter. Folders elsewhere stay full paths.

/// `path` relative to `base` when it is `base` (`.`) or lies inside it, else `None`. Windows
/// file names ignore case, so there the components are compared without case (`c:\OpenRB` is
/// inside `C:\openrb`); elsewhere exactly.
pub fn relative(base: &Path, path: &Path) -> Option<PathBuf> {
    relative_with(base, path, cfg!(windows))
}

/// [`relative`], with the case rule given.
pub fn relative_with(base: &Path, path: &Path, ignore_case: bool) -> Option<PathBuf> {
    let same = |a: Component, b: Component| {
        if ignore_case {
            a.as_os_str().to_string_lossy().to_lowercase()
                == b.as_os_str().to_string_lossy().to_lowercase()
        } else {
            a == b
        }
    };
    if !base.is_absolute() || !path.is_absolute() {
        return None;
    }
    let mut b = base.components();
    let mut p = path.components();
    loop {
        match (b.next(), p.next()) {
            (None, None) => return Some(PathBuf::from(".")),
            (None, Some(first)) => {
                let rest: Vec<Component> = std::iter::once(first).chain(p).collect();
                // "data\..\..\x" would lead out of the folder: such a path stays full
                if rest.iter().any(|c| matches!(c, Component::ParentDir)) {
                    return None;
                }
                return Some(rest.iter().collect());
            }
            (Some(x), Some(y)) if same(x, y) => {}
            _ => return None,
        }
    }
}

/// Whether `a` and `b` name the same folder by the system's rule of case ([`relative`]): on
/// Windows `E:\OpenRB\songs` and `e:\openrb\Songs` are one folder.
pub fn same(a: &Path, b: &Path) -> bool {
    a == b || relative(a, b).is_some_and(|r| r == Path::new("."))
}

/// A stored path read back: a relative one is taken inside `base` (`.` is `base` itself); a
/// full one stays as it is, and so does a rooted one without a drive letter (`\songs` on
/// Windows), which [`relative`] never writes: joined to `base` it would take `base`'s drive.
pub fn resolve(base: &Path, stored: &Path) -> PathBuf {
    if stored.is_absolute() || stored.has_root() {
        stored.to_path_buf()
    } else if stored == Path::new(".") {
        base.to_path_buf()
    } else {
        base.join(stored)
    }
}

/// The path of the module (exe or dll) that holds this function.
#[cfg(windows)]
fn module_path() -> Option<PathBuf> {
    use std::ffi::{OsString, c_void};
    use std::os::windows::ffi::OsStringExt;

    const FROM_ADDRESS: u32 = 0x4;
    const UNCHANGED_REFCOUNT: u32 = 0x2;
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetModuleHandleExW(flags: u32, address: *const c_void, module: *mut *mut c_void) -> i32;
        fn GetModuleFileNameW(module: *mut c_void, name: *mut u16, size: u32) -> u32;
    }
    let mut module = std::ptr::null_mut();
    // SAFETY: the address is of a function of this module; the handle is not retained
    // (UNCHANGED_REFCOUNT), the out pointer is a local.
    let ok = unsafe {
        GetModuleHandleExW(
            FROM_ADDRESS | UNCHANGED_REFCOUNT,
            module_path as *const c_void,
            &mut module,
        )
    };
    if ok == 0 {
        return None;
    }
    let mut size = 512usize;
    while size <= 32768 {
        let mut buf = vec![0u16; size];
        // SAFETY: `buf` holds `size` units.
        let n = unsafe { GetModuleFileNameW(module, buf.as_mut_ptr(), size as u32) } as usize;
        if n == 0 {
            return None;
        }
        if n < size {
            return Some(PathBuf::from(OsString::from_wide(&buf[..n])));
        }
        size *= 2;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_exe_folder_is_the_exes() {
        assert_eq!(
            folder_of(Path::new("/z/app/OpenRB-338.exe")),
            Some(PathBuf::from("/z/app"))
        );
    }

    #[test]
    fn there_is_no_portable_folder_off_windows() {
        if !cfg!(windows) {
            assert_eq!(candidate(), None);
            assert_eq!(folder(), None);
        }
    }

    fn abs(s: &str) -> PathBuf {
        if cfg!(windows) {
            PathBuf::from(format!("C:\\{}", s.replace('/', "\\")))
        } else {
            PathBuf::from(format!("/{s}"))
        }
    }

    #[test]
    fn either_marker_makes_a_folder_portable() {
        for marker in [MARKER_NAME, LEGACY_MARKER] {
            let d = tempfile::tempdir().unwrap();
            assert!(!is_portable(d.path()));
            std::fs::write(d.path().join(marker), "").unwrap();
            assert!(is_portable(d.path()), "{marker}");
            // a folder of that name is not the marker
            let e = tempfile::tempdir().unwrap();
            std::fs::create_dir(e.path().join(marker)).unwrap();
            assert!(!is_portable(e.path()), "{marker} as a folder");
        }
    }

    #[test]
    fn the_legacy_settings_are_data_ui_toml() {
        let d = tempfile::tempdir().unwrap();
        assert_eq!(legacy_settings(d.path()), None);
        std::fs::create_dir(d.path().join(LEGACY_DATA)).unwrap();
        std::fs::write(d.path().join(LEGACY_DATA).join("ui.toml"), "").unwrap();
        assert_eq!(
            legacy_settings(d.path()),
            Some(d.path().join(LEGACY_DATA).join("ui.toml"))
        );
    }

    #[test]
    fn a_folder_inside_is_stored_relative_and_read_back_full() {
        let base = abs("stick/OpenRB");
        let songs = abs("stick/OpenRB/songs/acid");
        let rel = relative_with(&base, &songs, false).unwrap();
        assert_eq!(rel, Path::new("songs").join("acid"));
        assert_eq!(resolve(&base, &rel), songs);
        assert_eq!(relative_with(&base, &base, false), Some(PathBuf::from(".")));
        assert_eq!(resolve(&base, Path::new(".")), base);
    }

    #[test]
    fn a_folder_elsewhere_stays_full() {
        let base = abs("stick/OpenRB");
        for other in ["stick/OpenRB2/songs", "stick", "music/songs"] {
            assert_eq!(relative_with(&base, &abs(other), false), None, "{other}");
        }
        assert_eq!(resolve(&base, &abs("music/songs")), abs("music/songs"));
        // a rooted path without a drive letter is not taken inside the folder either
        assert_eq!(resolve(&base, Path::new("/local")), PathBuf::from("/local"));
        // a way out of the folder through ".." is not inside it
        let out = base.join("..").join("x");
        assert_eq!(relative_with(&base, &out, false), None);
        // relative inputs are not compared
        assert_eq!(relative_with(Path::new("a"), Path::new("a/b"), false), None);
    }

    #[test]
    fn case_counts_only_where_the_system_ignores_it() {
        let base = abs("Stick/OpenRB");
        let songs = abs("stick/openrb/Songs");
        assert_eq!(
            relative_with(&base, &songs, true),
            Some(PathBuf::from("Songs"))
        );
        assert_eq!(relative_with(&base, &songs, false), None);
    }

    #[cfg(windows)]
    #[test]
    fn a_drive_letter_in_another_case_is_the_same_drive() {
        let base = PathBuf::from("E:\\OpenRB");
        let songs = PathBuf::from("e:\\OpenRB\\songs");
        assert_eq!(relative(&base, &songs), Some(PathBuf::from("songs")));
        // the stick on another letter: the relative path follows it
        assert_eq!(
            resolve(Path::new("F:\\OpenRB"), Path::new("songs")),
            PathBuf::from("F:\\OpenRB\\songs")
        );
    }
}
