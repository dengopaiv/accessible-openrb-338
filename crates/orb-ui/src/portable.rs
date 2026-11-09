//! Portable mode (task-223; the user's decision, task-222). When a file [`MARKER`]
//! (`portable.txt`) lies next to the running program, OpenRB-338 keeps its settings (`ui.toml`,
//! [`crate::settings::default_file`]) and its crash and run logs ([`crate::crash::log_dir`]) in
//! the folder [`DATA`] (`data`) beside it, instead of the platform's folders (`%APPDATA%` and
//! `%LOCALAPPDATA%` on Windows), and the settings store the folders that lie inside the program's
//! folder relative to it ([`relative`], [`resolve`]), so a copy on a USB stick keeps its songs and
//! mods folders on any PC, whatever the stick's drive letter. Folders elsewhere stay full paths.
//! Without the file nothing changes: upstream's packages have none. The fork's release zip
//! carries it (`scripts/package.sh`).

use std::path::{Component, Path, PathBuf};
use std::sync::OnceLock;

/// The file that makes the program's folder portable.
pub const MARKER: &str = "portable.txt";

/// The folder beside the program that holds the settings and the logs in portable mode.
pub const DATA: &str = "data";

/// `dir` when it holds the [`MARKER`] file: the portable folder of a program in `dir`.
pub fn root_in(dir: &Path) -> Option<PathBuf> {
    dir.join(MARKER).is_file().then(|| dir.to_path_buf())
}

/// The running program's portable folder (its own folder), when it is portable. Checked once.
pub fn root() -> Option<&'static Path> {
    static ROOT: OnceLock<Option<PathBuf>> = OnceLock::new();
    ROOT.get_or_init(|| {
        let exe = std::env::current_exe().ok()?;
        root_in(exe.parent()?)
    })
    .as_deref()
}

/// The data folder of the running program in portable mode.
pub fn data_dir() -> Option<PathBuf> {
    root().map(|r| r.join(DATA))
}

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
/// full one stays as it is.
pub fn resolve(base: &Path, stored: &Path) -> PathBuf {
    if stored.is_absolute() {
        stored.to_path_buf()
    } else if stored == Path::new(".") {
        base.to_path_buf()
    } else {
        base.join(stored)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn abs(s: &str) -> PathBuf {
        if cfg!(windows) {
            PathBuf::from(format!("C:\\{}", s.replace('/', "\\")))
        } else {
            PathBuf::from(format!("/{s}"))
        }
    }

    #[test]
    fn the_marker_makes_a_folder_portable() {
        let d = tempfile::tempdir().unwrap();
        assert_eq!(root_in(d.path()), None);
        std::fs::write(d.path().join(MARKER), "x").unwrap();
        assert_eq!(root_in(d.path()), Some(d.path().to_path_buf()));
        // a folder of that name is not the marker
        let e = tempfile::tempdir().unwrap();
        std::fs::create_dir(e.path().join(MARKER)).unwrap();
        assert_eq!(root_in(e.path()), None);
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
